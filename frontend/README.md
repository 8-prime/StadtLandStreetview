# Frontend

Viewport-contained Svelte 5 + TypeScript party game, built with Vite and pnpm. The Rust application in `../backend/` builds and embeds the frontend for hosting from one executable.

## Game canvas and identity

The entire application fits the browser viewport, with **no page or nested panel scrolling**. The initial desktop target is **1280 × 720** and larger. Header, phase content, and footer occupy reserved grid rows; the readiness controls never overlap the game board. Category lists, candidate collections, and larger scoreboards have explicit page controls. Twelve-player lobbies fit in a bounded grid.

Street View exploration, final selection, and synchronized review all fit the same **16:10 camera aperture** into the available width and height. Intentional letterboxing preserves framing instead of cropping it to fit a panel.

The street-spotter identity uses original sign lettering, electric blue, traffic yellow, asphalt outlines, and flat SVG characters. `Spotter.svelte` draws compatible character parts chosen deterministically by `makeSpotter(player.id)`: the same identity looks the same across clients and refreshes, without a new API field or image service. Readiness, disconnection, searching, and winning affect the pose/expression. There is no character reroll.

See the [brand notes](../docs/brand.md) and [screenshot preview](../docs/ui-preview.md).

## Run locally

Requires Node.js 22.12+ and pnpm 10.11.0 (the version is pinned in `package.json`). With Corepack available, `corepack enable pnpm` enables the package manager.

From this directory:

```sh
pnpm install --frozen-lockfile
pnpm dev
```

Open the Vite address, normally `http://localhost:5173`. No backend or Maps key is required for the solo demo.

## Google Maps

Create `frontend/.env.local` using `.env.example` as a reference:

```dotenv
VITE_GOOGLE_MAPS_API_KEY=your-browser-key
VITE_API_MODE=mock
```

Restart Vite after changing environment variables. Production values are supplied **at build time**.

Enable the Maps JavaScript API and billing in your Google Cloud project. Restrict the browser key to the development/production websites and permitted APIs. The key is intentionally public in the browser; `.env.local` is gitignored to keep environment-specific configuration out of source control. Axum does not need this browser key.

Ordinary rooms load Google Maps directly. Use the **Map / Satellite** control to switch the overhead map between roads and satellite imagery. Drag Pegman onto a covered street, frame the scene, choose a category, and save. Without a key, the live map shows configuration instructions and saving is disabled. A map-loading or authentication failure is shown in the map panel.

The search readiness bar lets players mark **Done searching** and undo while waiting. Searching ends when every active player is ready or the shared timer expires, then everyone chooses their final evidence with readiness reset. Saving or changing evidence clears that player's readiness.

Room creation has an optional **Neighbourhood radio** setting, off by default and shown to everyone in the lobby. When enabled, a small walkie-talkie beside the search timer shows anonymous signals from the nearest active Street View player: **Someone nearby** within 5 km and **Very close!** within 250 m. It shows no identities, direction, or exact distance. Leaving Street View clears the signal, and stale positions expire after 15 seconds. Position messages use a separate lightweight channel; they do not reload game snapshots, recreate Maps objects, or call a Google distance service. The illustrated solo demo uses simulated peer positions.

Saved viewpoints preserve panorama ID, position, heading, pitch, and zoom. Search, selection, and review use a 16:10 viewport. Selection/review views are locked to the saved framing. Panorama availability can change; unavailable views show an explanatory message.

Each player keeps one map and one panorama alive for the current match in their browser tab. Screens move the same canvas between exploration, selection, and review, updating the panorama and camera instead of constructing another viewer per finding. Empty categories and illustrated scenes do not discard these objects. Leaving the room or changing rooms/player identity releases them; refreshing the page creates new instances. A player resuming directly into selection/review only creates a panorama. This avoids repeated instance loads counted by [Google Maps billing](https://developers.google.com/maps/faq#usage_maploads); imagery still loads as the player navigates.

## Two development modes

### Mock mode (default)

- **Try the solo demo** opens a round with two explicitly simulated peers and locally illustrated scenes. It never loads Google Maps. Simulated peers mark ready during searching too, so **Done searching** ends the search through the same readiness flow as regular rooms.
- **Start a room** creates a local room. Copy its link into a separate tab in the **same browser profile and origin** to join as another player. Regular rooms use real Maps when configured.
- Mock rooms are serialized in `localStorage` and identities in per-tab `sessionStorage`. Refreshing retains the identity; closing a tab ends that tab's credential lifetime. Some browser “duplicate tab” operations copy session storage, so use a fresh tab and paste the room URL to create a separate peer.
- Storage events synchronize tabs; Web Locks serialize mock mutations where supported. Presence updates have a 45-second disconnect threshold. Disconnected peers still count toward readiness. Explicit leave removes that requirement.
- This is a local UI prototype: the room data in browser storage is inspectable, and links do not work across different browsers or computers. The mock is not a secure or persistent game server.

### Axum mode

The normal hosted application needs only the Rust server. From the repository root:

```sh
pnpm --dir frontend install --frozen-lockfile
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:3000`. Cargo builds the production UI in HTTP mode and embeds
the assets, including fonts, in the executable. No Vite server or separate static
directory is required at runtime. Build a deployable binary with
`cargo build --release --manifest-path backend/Cargo.toml`. Set the Google Maps key
before building; the embedded frontend always uses HTTP mode.

For optional UI hot reload through Vite, configure:

```dotenv
VITE_API_MODE=http
API_PROXY_TARGET=http://127.0.0.1:3000
```

Start the backend from the repository root with `cargo run --manifest-path backend/Cargo.toml`.
Restart Vite. HTTP actions use `/api` and the frontend subscribes to `/api/sessions/{id}/ws`. The hosted Rust app serves both paths and all frontend files on the same origin. No API hostname is hardcoded in the client.

HTTP mode uses an HttpOnly browser cookie: tabs in the same browser profile share
the same player within a room. Test different players using separate browser
profiles or private sessions. Refreshing/reconnecting resumes membership;
explicit leave revokes it for that room. See the [backend setup](../backend/README.md)
for listening-address, public-origin, and HTTPS cookie settings.

See [`../docs/api.md`](../docs/api.md) for endpoint payloads, cookie identity, snapshot types, WebSocket notifications, readiness transitions, and visibility rules. The HTTP adapter is intentionally isolated so it can evolve with the Rust server.

## Project layout

```text
src/
  App.svelte          Routing, connection lifecycle, actions, shared shell
  screens/            Home, lobby, categories, search, selection, review, results
  lib/
    api/types.ts      Shared frontend DTOs and action types
    api/http.ts       Axum transport adapter
    api/mock.ts       Browser-local prototype transport
    api/engine.ts     Local prototype state machine (not server logic)
    maps/             Lazy Google Maps loader and Street View component
    components/       Generated SVG spotters, sign logo, pagination, category rail
  style.css           Fixed-viewport layouts, brand, focus and reduced-motion styles
```

The application uses hash room URLs, so static hosting does not require route fallback configuration. The current phase and review item come from the session snapshot, never a player-controlled route.

## Checks and production build

```sh
pnpm check
pnpm test
pnpm exec playwright install chromium
pnpm test:e2e
pnpm build
pnpm preview
```

On systems with an existing Chromium, run `CHROMIUM_PATH=/path/to/chromium pnpm test:e2e` instead of downloading a browser. Browser tests use mock mode with no Maps key. Run them without a live-Maps `.env.local` configuration.

State-machine tests cover readiness, stale steps, deadlines, private snapshots, selections, review ordering, scoring, departures, and empty rounds. Browser tests exercise the solo game, cross-tab agreement, stable portraits, refresh recovery, and keyboard interaction. Viewport checks run at **1280 × 720, 1440 × 900, and 1920 × 1080**, including maximum-length categories, twelve peers, twelve candidates, paginated scoreboards, and dialogs. They check for offscreen/clipped controls, document or nested scrolling, and incorrect camera aspect ratios. Actual Google Maps requests need a configured key and a manual integration check.

Standalone `pnpm build` emits assets to `dist/`. Cargo builds its own HTTP-mode bundle
inside its output directory and embeds it in the Rust executable; deploying that
executable does not require `dist/`. Bricolage Grotesque and Nunito Sans fonts are
bundled locally through Fontsource; the UI does not request Google Fonts.

The pnpm Rollup override pins a retrievable release because the registry advertised a newer tarball that returned 404 during setup. The lockfile pins the complete dependency graph.
