# Rust / Axum backend

The binary serves the frontend and the HTTP and WebSocket endpoints in [`docs/api.md`](../docs/api.md).
`backend::api::router` connects authenticated handlers to
`backend::session::SessionManager`, which implements the game rules and personalized
snapshots. The frontend's HTTP adapter works with these routes directly.

## Run

From the repository root:

```sh
pnpm --dir frontend install --frozen-lockfile
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:3000`. Cargo's build script builds the frontend with Vite in
HTTP mode and embeds its HTML, JavaScript, CSS, fonts, and images in the executable.
It writes generated assets into Cargo's output directory, leaving `frontend/dist`
available for independent frontend builds. Vite does not run as a server.

Rust, Node.js 22.12+, and the installed frontend dependencies are required to build.
The resulting executable needs no Node.js, checkout, or static-assets directory
on the host. Build for deployment with:

```sh
cargo build --release --manifest-path backend/Cargo.toml
```

Deploy `backend/target/release/backend` (`backend.exe` on Windows) and point the
reverse proxy's single upstream at this process, including WebSocket upgrades.
`/` serves the UI, `/assets/*` serves the built assets, and `/api/*` serves the API.
The application uses hash room URLs, so missing files/routes return 404 rather
than the HTML shell. HTML and unhashed files revalidate; Vite assets have long-lived
immutable caching. Static responses support ETags, conditional GET, and HEAD.

Set `VITE_GOOGLE_MAPS_API_KEY` in the build environment or `frontend/.env.local`
before building to enable real Street View. The key is embedded at build time;
changing it requires rebuilding the executable. The embedded frontend always uses
the HTTP API, even if a local Vite configuration selects mock mode.

For optional hot reload during UI development, set `VITE_API_MODE=http` and
`API_PROXY_TARGET=http://127.0.0.1:3000` in `frontend/.env.local`, then run
`pnpm --dir frontend dev` alongside the backend. This is a development convenience;
normal hosting only runs the Rust executable.

The binary reads environment variables directly (it does not load `.env` files):

| Variable | Default | Meaning |
| --- | --- | --- |
| `SLS_BIND_ADDR` | `127.0.0.1:3000` | Listening address; use `0.0.0.0:3000` for LAN access |
| `SLS_PUBLIC_ORIGIN` | Unset | Exact frontend origin, e.g. `https://game.example`, if a proxy rewrites Host |
| `SLS_SECURE_COOKIES` | `false` | Set `true` when serving HTTPS, including behind a TLS proxy |

Forward the entire site to this server in production. API requests with an Origin header
must match `SLS_PUBLIC_ORIGIN`, or the request's Host if no public origin is set.
Preserve Host when proxying, or configure the public origin. No cross-origin CORS
access is enabled. Ctrl+C stops the HTTP server and closes registered sockets.

## Docker

Build from the repository root using the centralized `Dockerfile`:

```sh
docker build -t stadt-land-streetview .
docker run --rm -p 3000:3000 stadt-land-streetview
```

Open `http://localhost:3000`. Docker installs the pinned pnpm version and frontend
dependencies, then Cargo's build script builds the UI in HTTP mode and embeds it
in a release executable. Dependency and Cargo caches speed up later builds. The
final `scratch` image runs as an unprivileged user and contains the static musl
executable and a CA certificate bundle. Both build stages use Alpine; the build
explicitly enables static C-runtime linking and checks that the executable has
neither an ELF interpreter nor shared-library dependencies before packaging it.
No shell, package manager, or system C runtime is needed in the final image. It
listens on `0.0.0.0:3000` and uses SIGINT for graceful shutdown.

Supply the Google Maps browser key at build time:

```sh
docker build --build-arg VITE_GOOGLE_MAPS_API_KEY=your-browser-key -t stadt-land-streetview .
```

Local `.env` files, `node_modules`, build outputs, and Git metadata are excluded
from the build context. The browser key is part of the public JavaScript bundle;
configure its website/API restrictions in Google Cloud. Changing it requires
rebuilding the image.

Forward the entire host, including WebSocket upgrades, to this one container. For
a TLS reverse proxy, pass `-e SLS_SECURE_COOKIES=true` to `docker run`. If the proxy
rewrites Host, also pass `-e SLS_PUBLIC_ORIGIN=https://game.example`. Session state
and credentials remain in memory and reset when the container restarts.

## Identity

Create/join issue an opaque `sls_identity` session cookie with `HttpOnly`,
`SameSite=Strict`, and `Path=/api` (plus `Secure` when configured). The server stores
the credential's room-to-player mappings; public player IDs cannot authenticate
requests. One browser credential can belong to multiple rooms, and all its tabs
share the same player within a room. Use separate browser profiles/private sessions
to test different players. Joining again with the cookie resumes membership.

Explicit leave revokes that room membership, closes its sockets, and keeps other
room memberships intact. Joining after leave creates a new membership during
setup. Both credentials and game state are in memory and reset on restart.

API responses use `Cache-Control: no-store`. Game and extraction errors use the same
JSON `{ "message": ... }` envelope; invalid/missing credentials return 401 and
attempts to access another room without membership return 403. JSON bodies and
incoming WebSocket frames/messages are limited to 16 KiB.

## Session architecture

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

During review, casting or changing a vote automatically marks the voter ready.
Only the finding's owner uses `ready`; when all remaining members have responded,
the finding is scored and the session immediately opens the next finding (or the
final scoreboard). There is no result-reveal acknowledgement step. Completed
results are available in `SessionState.results`; the current review's legacy
`revealed` and `result` fields remain `false` and `null` for wire compatibility.

`SessionError` has `InvalidInput`, `Forbidden`, `NotFound`, and `Conflict` variants,
which handlers can map to 400, 403, 404, and 409 respectively. Use its `Display`
message in the contract's `{ "message": ... }` error response.

## WebSocket transport

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

The Axum transport implements this in `src/api/websocket.rs`. It pings every
15 seconds, expires a socket after 45 seconds without a pong, and bounds each
outgoing write to 10 seconds. Receive, send, heartbeat, and shutdown futures run
concurrently; ending one cancels the others before unregistering that connection.
Registration happens after a successful upgrade, so failed upgrades do not leave
phantom presence. Explicit leave closes all of that player's sockets.

## Rules and limits

The manager starts one timer when searching begins; no polling loop is required.
Expiry changes the phase and publishes a notification even if no clients are
making requests. Every action also checks expiry before applying its rules, so a
delayed timer cannot admit late findings. Readiness includes disconnected members,
uses an opaque step ID, and resets at transitions and setup membership/category
changes. During searching, unanimous readiness ends the search early and opens
selection; the original timer becomes a no-op after this transition. Saving or
changing evidence clears that player's readiness. Categories are shared and
editable by any active peer during collection.

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

The suite includes embedded frontend assets, caching, HEAD/conditional requests,
API/static routing, HTTP routes for a full round, and real loopback WebSocket tests
for authentication, notifications, multiple tabs, close cleanup, heartbeat expiry,
explicit leave, and shutdown. The WebSocket tests need permission to connect to
localhost sockets. Dependencies must already be cached when using `--offline`.
