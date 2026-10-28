# Backend session layer

The library exposes `backend::session::SessionManager`, implementing the game rules
and personalized state from [`docs/api.md`](../docs/api.md). HTTP routes, credential
cookies, and the WebSocket transport still need to be wired in Axum; `main.rs` is
currently a placeholder.

Create one manager and clone it into application state. Its global `RwLock` only
protects room lookup/creation; each room has its own asynchronous `RwLock` and
broadcast channel. Snapshot reads can run concurrently. All game mutations and
notifications are ordered under that room's write lock. Socket writes and JSON
serialization happen in the handler, after the manager releases its locks.

## Methods for handlers

All methods are asynchronous. IDs of players, categories, findings, readiness
steps, and socket connections are `Uuid`; room codes are case-insensitive strings.
Except creation/join, pass the player ID derived from a verified credential.
Player IDs in snapshots and room codes are public identifiers, not credentials.

| API operation | Manager method | Result |
| --- | --- | --- |
| Create | `start_session(name, duration_sec)` | `SessionMembership { session_id, player_id }` |
| Join/resume | `join_session(id, name, existing_player)` | Player ID |
| Leave | `leave_session(id, player)` | `()` |
| Snapshot | `get_session(id, player)` | Personalized `SessionState` |
| Ready/unready | `ready(id, player, step_id, ready)` | `()` |
| Add category | `add_category(id, player, text)` | Category ID |
| Edit category | `update_category(id, player, category, text)` | `()` |
| Delete category | `delete_category(id, player, category)` | `()` |
| Save candidate | `add_finding(id, player, category, view)` | Finding ID |
| Delete candidate | `delete_finding(id, player, finding)` | `()` |
| Select candidate | `select_finding(id, player, category, finding)` | `()` |
| Vote | `vote(id, player, item, Vote::Accept / Vote::Reject)` | `()` |
| Open WebSocket | `connect(id, player)` | `SessionConnection { connection_id, updates }` |
| Close WebSocket | `disconnect(id, player, connection_id)` | `()` |

The creation `name` is the player's display name. Creation generates a unique
six-character uppercase room code and establishes that player's membership. The
handler returns `{ "id": membership.session_id }` and establishes a credential
associated with `membership.player_id` in an HttpOnly cookie.

Pass `None` for a new join, or `Some(player_id)` for a membership verified by the
credential layer. A disconnected member can resume during any phase. New joins
and return after explicit leave are restricted to lobby/category collection.
Disconnecting retains membership, findings, scores, and readiness requirements;
explicit leave removes readiness/voting requirements but retains submissions.

`SessionError` has `InvalidInput`, `Forbidden`, `NotFound`, and `Conflict` variants,
which handlers can map to 400, 403, 404, and 409 respectively. Use its `Display`
message in the contract's `{ "message": ... }` error response.

## WebSocket integration

Call `connect` after authenticating and accepting the socket. Each socket receives
a unique connection ID, so closing an older connection or a second tab cannot
mark a still-connected player offline. `connected` becomes false when their last
socket is disconnected.

Forward serialized `SessionEvent` values from `updates` to the socket. These contain
only `{ "type": "state-changed", "revision": ... }`; clients then GET their own
snapshot. Never broadcast a personalized `SessionState`. If a receiver reports
`RecvError::Lagged`, send another state-change notification so the client fetches
the current snapshot, then continue consuming notifications.

Run a receive loop alongside the sender and server ping/pong heartbeat. When
either socket direction fails, a close arrives, or the heartbeat expires, stop
the connection's tasks and call `disconnect` with its connection ID. Dropping the
receiver alone does not unregister the connection. The initial snapshot is fetched
with `get_session`, as the frontend already does after connecting.

## Rules and limits

The manager starts one timer when searching begins; no polling loop is required.
Expiry changes the phase and publishes a notification even if no clients are
making requests. Every action also checks expiry before applying its rules, so a
delayed timer cannot admit late findings. Readiness includes disconnected members,
uses an opaque step ID, and resets at transitions and setup membership/category
changes. Categories are shared and editable by any active peer during collection.

Names are trimmed and limited to 1–24 characters, category text to 1–100,
duration to 60–1200 seconds, and rooms to 12 active players, 12 categories, and
12 candidates per player/category. Viewpoints require a nonempty panorama ID,
finite coordinates (latitude ±90, longitude ±180), heading in [0, 360), pitch in
[−90, 90], and zoom in [0, 5]. Unknown viewpoint fields, including the frontend's
`demoScene`, are rejected during deserialization.

State is in memory. Session expiration/persistence remains undecided in the API
contract; the model tracks when all sockets become inactive but does not evict rooms.

## Checks

Run from `backend/`:

```sh
cargo test --offline
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
```
