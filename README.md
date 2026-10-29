# StadtLandStreetview

A browser-based multiplayer game in which players explore Google Street View to find scenes that match categories created by the group. After a timed search, everyone reviews the submitted images and votes on whether they match. The player with the most accepted matches wins.

## Development

The full-viewport desktop frontend is implemented in **Svelte 5 + TypeScript + Vite**, under [`frontend/`](frontend/). It uses a street-spotter party-game identity, generated SVG characters, and paginated content instead of scrolling. The Rust/Axum application under [`backend/`](backend/) embeds the built frontend and serves the entire site, game API, and WebSocket notifications from one process.

```sh
pnpm --dir frontend install --frozen-lockfile
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:3000`. Cargo builds and embeds the frontend in HTTP mode. A release build produces a single executable to deploy behind one reverse-proxy upstream; no separate static-file server is needed. Live Street View requires a restricted Maps JavaScript API browser key in `frontend/.env.local` before building.

For frontend hot reload or the local solo demo, run `pnpm --dir frontend dev` and open `http://localhost:5173`. The default Vite mode uses browser-local rooms and offers **Try the solo demo** without a backend or Google Maps key.

The root `Dockerfile` builds both parts and packages the combined executable:

```sh
docker build -t stadt-land-streetview .
docker run --rm -p 3000:3000 stadt-land-streetview
```

See [Docker configuration](backend/README.md#docker) for the Maps build argument
and reverse-proxy settings.

- [Frontend setup, Maps configuration, and checks](frontend/README.md)
- [UI design preview — screenshots of every game phase](docs/ui-preview.md)
- [Brand identity and no-scroll layout principles](docs/brand.md)
- [HTTP and WebSocket API contract](docs/api.md)
- [Backend setup, session methods, and configuration](backend/README.md)

## Game overview

The game has three main stages:

1. **Collect categories** as a group.
2. **Find matching Street View scenes** within a shared time limit, then select final submissions.
3. **Review and vote** on each finding, then determine the winner.

## Stage 1: Collect categories

Before searching begins, the players collect a shared list of categories. A category describes something players should try to find in Street View, for example:

- A person walking a dog.
- A car parked illegally.

All players search for the same categories. Each player's goal is to submit one matching image for each category.

## Stage 2: Explore and collect findings

Each player gets a Google Maps view **without search**, with a **Map / Satellite** switch for the overhead map. Players can move anywhere on the map and place the Street View pin to explore street-level imagery.

- There is **one overall time limit** for the search stage and **no separate time limit per category**.
- Players can work on the categories in any order and switch between them freely.
- A player can save their current Street View as a finding for any category.
- Players may save multiple candidate findings for the same category during the search.

### Final selection after time runs out

When the search timer expires, exploration and collection end. Before voting begins, players choose which saved finding to keep for each category.

Only **one final image per player per category** moves on to review. This selection happens **after the search time is up**, so players do not need to make their final choices while searching.

The first saved candidate is selected automatically. Players can change that choice before review. A category with no saved finding automatically has no submission and is skipped for that player.

## Stage 3: Review and vote

The group reviews final submissions together, **one finding at a time**, grouped by category. Everyone's submission for the first category is shown before moving on to the next category.

For three players, the order looks like this:

```text
Category 1: Player 1 → Player 2 → Player 3
Category 2: Player 1 → Player 2 → Player 3
Category 3: Player 1 → Player 2 → Player 3
...
```

For each finding, players vote on whether the image matches the category. The vote determines whether that finding counts as an accepted match.

All players see the **same server-controlled finding and framing**. Players cannot independently browse the other submissions during voting. Players vote on others' findings, not their own; accept votes must outnumber reject votes. Tied votes do not count as a match.

Casting a vote automatically counts as ready. Only the player whose finding is being shown marks ready without voting. Votes can be changed while waiting for the remaining players. Once everyone has responded, the finding is scored and the group immediately moves to the next finding, with no result-reveal or extra readiness step. After the final finding, the group proceeds directly to the final scoreboard.

### Winning

Once all findings have been reviewed, count each player's accepted matches across all categories. **The player with the most accepted matches wins.**

The initial UI presents tied leaders as joint winners.

## Peer-based progression

There is no host controlling progression. During setup and final selection, each player can set or unset their readiness for the current step. Once all active members are ready, the game advances and resets readiness. Searching ends automatically at its shared deadline and cannot be shortened by readiness. During review, voters are automatically ready after casting a vote, while the finding's owner confirms readiness separately.

Changing the shared category list resets everyone's readiness. Changing a final selection resets that player's readiness; changing a vote keeps the voter ready. Disconnected players retain their membership and response requirement; explicitly leaving removes that requirement.

## Rules still to decide

The core game flow above is established. The following details remain open:

- Final limits and permissions for category collection. The prototype lets everyone add, edit, or remove categories before agreeing to the list.
- Production session expiration and any recovery mechanism for players who never reconnect.
- Whether joint winners should remain the final tie rule or be replaced by a tie-breaker.
