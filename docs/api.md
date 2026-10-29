# HTTP and WebSocket API contract

This contract is implemented by the Axum backend in `backend/src/api/` and the frontend adapter in `frontend/src/lib/api/http.ts`. See `backend/README.md` for startup and configuration. The local mock uses the same frontend types and game progression rules.

## Transport and identity

- Base path: `/api` on the frontend origin. The Rust executable embeds and serves the frontend at `/`, with API and WebSocket routes on the same listener. Vite can optionally proxy `/api` to Axum during UI development, including WebSocket upgrades.
- Create and join establish an opaque player credential in an HttpOnly cookie. The server derives the acting player from this credential. A room code is not a player credential.
- The cookie is shared across browser tabs and maps to separate memberships per room. Explicit leave revokes only that room's membership; disconnecting preserves it. Use separate browser profiles to test different players in the same room.
- JSON request/response bodies use camelCase. Timestamps are Unix **milliseconds**.
- Actions return `204 No Content` (or a JSON body with a successful response). Errors use a non-2xx status and `{ "message": "Human-readable explanation" }`.
- Creation returns `201` with `{ id }`; adding a category or finding returns `201` with its `{ id }`. Missing/invalid credentials return `401`, forbidden actions `403`, missing resources `404`, and phase/stale-step conflicts `409`. Malformed requests and extraction failures also use the JSON error envelope.
- Session IDs are URL-safe, case-insensitive room codes. The join form normalizes typed codes to uppercase. Returned IDs should use a canonical uppercase form.
- Enforce permissions, current step, ownership, deadlines, and input validation on the server. The browser's disabled controls are only presentation.

## Endpoints

All paths below are relative to `/api`.

| Method | Path | Request / response |
| --- | --- | --- |
| POST | `/sessions` | `{ name, durationSeconds }` → `{ id }`; establish membership |
| POST | `/sessions/{sessionId}/join` | `{ name }`; establish or resume membership |
| POST | `/sessions/{sessionId}/leave` | Explicitly leave; preserve submissions and scores |
| GET | `/sessions/{sessionId}` | A personalized `SessionState` snapshot |
| WebSocket | `/sessions/{sessionId}/ws` | State-change notifications; cookie-authenticated |
| PUT | `/sessions/{sessionId}/ready` | `{ stepId, ready }` |
| POST | `/sessions/{sessionId}/categories` | `{ text }` |
| PATCH | `/sessions/{sessionId}/categories/{categoryId}` | `{ text }` |
| DELETE | `/sessions/{sessionId}/categories/{categoryId}` | No body |
| POST | `/sessions/{sessionId}/findings` | `{ categoryId, view }` |
| DELETE | `/sessions/{sessionId}/findings/{findingId}` | Own finding only |
| PUT | `/sessions/{sessionId}/selections/{categoryId}` | `{ findingId }`; own finding in this category only |
| PUT | `/sessions/{sessionId}/review/{itemId}/vote` | `{ vote: "accept" \| "reject" }` |

There are no host progression endpoints, separate viewer endpoints, or endpoints to browse other findings during review.

### Viewpoint

```json
{
  "panoramaId": "google-panorama-id",
  "position": { "lat": 52.52, "lng": 13.405 },
  "heading": 145.2,
  "pitch": -5,
  "zoom": 1
}
```

Findings have server-generated `id`, `playerId`, and `createdAt` in addition to `categoryId` and `view`. The `demoScene` field in the TypeScript model is local-demo-only; HTTP submissions reject it.

## State snapshot

The complete frontend types are in `frontend/src/lib/api/types.ts`. `SessionState` contains:

| Field | Meaning |
| --- | --- |
| `id`, `revision` | Session ID and monotonically increasing state revision |
| `stepId` | Opaque ID for the current readiness agreement |
| `phase` | `lobby`, `categories`, `searching`, `selection`, `review`, or `finished` |
| `meId` | Authenticated player's ID |
| `demo` | `false` for backend rooms |
| `durationSeconds`, `serverTime`, `searchEndsAt` | Duration, server clock, and deadline (`null` before searching) |
| `players` | `{ id, name, color, ready, connected, score, left }[]`; color is a palette index 0–5 |
| `categories` | Ordered `{ id, text }[]` |
| `ownFindings` | Requesting player's candidates only |
| `selections` | Requesting player's `{ [categoryId]: findingId }` mapping |
| `review` | Current shared item, or `null` outside review |
| `results` | Completed finding results only |

`review` contains:

```ts
{
  id: string;                  // Review item ID, also used in vote requests
  finding: Finding;            // Only the finding currently shown to everyone
  index: number;               // Zero-based index in the review queue
  total: number;
  revealed: boolean;           // Always false; retained for wire compatibility
  ownVote: 'accept' | 'reject' | null;
  votesCast: number;
  eligibleVoters: number;
  result: ReviewResult | null; // Always null; completed results are in SessionState.results
}
```

`ReviewResult` is `{ itemId, categoryId, playerId, accepted, accepts, rejects }`. The item ID can be the finding ID if findings occur only once in the review queue.

Do not expose other players' candidate collections, future review findings, private votes, or unrevealed vote tallies in snapshots or WebSocket messages. Everyone's `review.id` is the same; personalized fields such as `ownVote` differ.

## WebSocket behavior

The frontend fetches a snapshot after connecting and after **any** WebSocket message. A minimal notification such as `{ "type": "state-changed" }` is sufficient. Broadcast after every mutation and timer-driven transition.

The client reconnects with exponential backoff (up to 15 seconds) and resynchronizes with GET. Snapshots older than the last applied revision are ignored. HTTP actions are disabled while the socket is disconnected. Use server WebSocket ping/pong for liveness; a separate HTTP heartbeat is not required.

## Progression rules used by the UI

1. All active members ready in the lobby → category collection.
2. All ready with at least one category → searching. Establish one server deadline.
3. Deadline reached or all active members ready during searching → final selection. Search readiness is reversible until everyone is ready; no findings are required to mark done.
4. All ready after selection → first review item, or finished if nobody submitted a finding.
5. All eligible voters have voted and the finding's owner is ready (unless they left) → calculate the result, award any point, and immediately open the next item, or finish after the last item. There is no intermediate result-reveal step.

Reset readiness and assign a new `stepId` at each transition. A category edit or membership change during setup also invalidates readiness and should issue a new `stepId`. Reject stale readiness requests; do not apply them to a newer step. Readiness requests set a value rather than toggling it, making retries idempotent.

- Categories are editable only during collection. The prototype allows any peer to edit.
- Candidates can be saved only while searching, before the deadline or unanimous early completion. Saving a candidate clears that player's search readiness. Removing candidates is allowed during searching and selection.
- The first saved candidate is the default selection. Removing the selected candidate selects the next remaining one. Zero candidates implicitly means no submission.
- Selecting or deleting a candidate clears that player's readiness during searching and selection.
- Lock final selections when review begins. Build the review queue category-first, then player-first; skip missing submissions.
- Players cannot vote on their own findings. Casting a vote sets the voter's readiness automatically; only the owner uses the readiness endpoint during review. Reject readiness requests from other players, including attempts to unset readiness independently of a vote.
- Votes can change until everyone has responded; changing a vote keeps the voter ready. Reject vote requests for a previous item after progression, and clear votes and readiness for the new item.
- Accept if accept votes outnumber reject votes. Ties and zero votes fail. Award one point per accepted finding, once.
- Final scores count accepted matches. The UI presents tied leaders as joint winners.
- Disconnection preserves membership and the readiness requirement. Explicit leave excludes that player from future readiness/voting requirements but retains their findings and score. Re-evaluate progression after a leave.

The UI currently offers durations of 1, 5, 10, 15, or 20 minutes, limits names to 24 characters and category text to 100, and supports up to 12 peers, 12 categories, and 12 saved candidates per player/category. These are prototype limits and can be revised together with the server.
