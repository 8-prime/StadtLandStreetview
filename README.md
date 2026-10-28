# StadtLandStreetview

A browser-based multiplayer game in which players explore Google Street View to find scenes that match categories created by the group. After a timed search, everyone reviews the submitted images and votes on whether they match. The player with the most accepted matches wins.

## Development

The full-viewport desktop frontend is implemented in **Svelte 5 + TypeScript + Vite**, under [`frontend/`](frontend/). It uses a street-spotter party-game identity, generated SVG characters, and paginated content instead of scrolling. The Rust backend's session layer is implemented under [`backend/`](backend/); Axum transport wiring is still pending.

```sh
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend dev
```

Open `http://localhost:5173` and choose **Try the solo demo** to walk through a game without a backend or Google Maps key. Ordinary local rooms can be joined from separate tabs in the same browser. Live Street View requires a restricted Maps JavaScript API browser key in `frontend/.env.local`.

- [Frontend setup, Maps configuration, and checks](frontend/README.md)
- [UI design preview — screenshots of every game phase](docs/ui-preview.md)
- [Brand identity and no-scroll layout principles](docs/brand.md)
- [Proposed Axum API contract](docs/api.md)
- [Backend session methods and handler integration](backend/README.md)

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

Each player gets a Google Maps view **without search**. Players can move anywhere on the map and place the Street View pin to explore street-level imagery.

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

Votes stay private until everyone is ready. The result is revealed, then everyone marks ready again to proceed to the next finding. After the final result, the group proceeds to the final scoreboard.

### Winning

Once all findings have been reviewed, count each player's accepted matches across all categories. **The player with the most accepted matches wins.**

The initial UI presents tied leaders as joint winners.

## Peer-based progression

There is no host controlling progression. Each player can set or unset their readiness for the current step. Once all active members are ready, the game advances and resets readiness. The exception is searching: its shared deadline ends the phase automatically, and readiness cannot shorten it.

Changing the shared category list resets everyone's readiness. Changing a final selection or vote resets that player's readiness. Disconnected players retain their membership and readiness requirement; explicitly leaving removes that requirement.

## Rules still to decide

The core game flow above is established. The following details remain open:

- Final limits and permissions for category collection. The prototype lets everyone add, edit, or remove categories before agreeing to the list.
- Production session expiration and any recovery mechanism for players who never reconnect.
- Whether joint winners should remain the final tie rule or be replaced by a tie-breaker.
