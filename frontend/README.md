# Frontend

Viewport-contained Svelte 5 + TypeScript party game, built with Vite and pnpm. The Rust backend can live independently in `../backend/`.

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

Ordinary rooms load Google Maps directly. Drag Pegman onto a covered street, frame the scene, choose a category, and save. Without a key, the live map shows configuration instructions and saving is disabled. A map-loading or authentication failure is shown in the map panel.

Saved viewpoints preserve panorama ID, position, heading, pitch, and zoom. Search, selection, and review use a 16:10 viewport. Selection/review views are locked to the saved framing. Panorama availability can change; unavailable views show an explanatory message.

## Two development modes

### Mock mode (default)

- **Try the solo demo** opens a round with two explicitly simulated peers and locally illustrated scenes. It never loads Google Maps. A labeled demo-only shortcut ends searching early.
- **Start a room** creates a local room. Copy its link into a separate tab in the **same browser profile and origin** to join as another player. Regular rooms use real Maps when configured.
- Mock rooms are serialized in `localStorage` and identities in per-tab `sessionStorage`. Refreshing retains the identity; closing a tab ends that tab's credential lifetime. Some browser “duplicate tab” operations copy session storage, so use a fresh tab and paste the room URL to create a separate peer.
- Storage events synchronize tabs; Web Locks serialize mock mutations where supported. Presence updates have a 45-second disconnect threshold. Disconnected peers still count toward readiness. Explicit leave removes that requirement.
- This is a local UI prototype: the room data in browser storage is inspectable, and links do not work across different browsers or computers. The mock is not a secure or persistent game server.

### Axum mode

```dotenv
VITE_API_MODE=http
API_PROXY_TARGET=http://localhost:3000
```

Restart Vite. HTTP actions use `/api` and the frontend subscribes to `/api/sessions/{id}/ws`. Production should route those paths to Axum on the frontend origin. No API hostname is hardcoded in the client.

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

Production assets are emitted to `dist/`, suitable for Axum static-file serving or a static host. Bricolage Grotesque and Nunito Sans fonts are bundled locally through Fontsource; the UI does not request Google Fonts.

The pnpm Rollup override pins a retrievable release because the registry advertised a newer tarball that returned 404 during setup. The lockfile pins the complete dependency graph.
