# UI design preview

The desktop frontend uses a warm, illustrated travel-journal style: cream backgrounds, forest-green actions, sage accents, and a little terracotta. Manrope and DM Sans provide the main typography, with italic serif accents for the headings.

These are actual screenshots of the running Svelte application at a 1440-pixel desktop width. The street scenes are **illustrated demo content**, not Google Street View imagery. Configured live rooms replace those scenes with the interactive Google map and saved Street View viewpoints.

Click any screenshot to open it at full size. You do not need to run the app or forward a container port to view this page on GitHub.

## Home — a meeting point for curious people

An illustrated introduction, a short explanation of the three stages, create/join controls, and a solo-demo entry point.

![Home screen with illustrated street scene and create/join form](images/home.png)

## Lobby — the curious crew

Player presence, room sharing, search duration, and a persistent group-readiness footer. There are no host progression controls.

![Lobby showing players, readiness, and game instructions](images/lobby.png)

## Categories — dream it up

A shared editable category list with optional suggestions. Edits invalidate the group's previous readiness.

![Category collection screen with an editable list and inspiration suggestions](images/categories.png)

## Search — go get lost

A category checklist, prominent shared timer, a large exploration viewport, and an action to save the current framing. Multiple views can be saved for each category.

![Search workspace with category sidebar, timer, illustrated demo scene, and save controls](images/search.png)

## Final selection — your best shot

Choose among saved candidates after searching ends. One is selected automatically; categories without a finding need no extra action.

![Final selection screen with the selected viewpoint and candidate choices](images/selection.png)

## Voting — make the call

The group sees one locked viewpoint at a time. Voting buttons and participation counts sit alongside the finding; the footer shows readiness. Results are revealed before everyone proceeds to the next finding.

![Voting screen with the shared finding, match/reject buttons, and readiness footer](images/review.png)

## Results — the final tally

A simple leaderboard of accepted matches, including shared winners when scores tie.

![Final scoreboard with player rankings and accepted-match totals](images/results.png)

---

The screenshots are captured by the browser tests in `frontend/tests/game.spec.ts`. See the [frontend setup guide](../frontend/README.md) to run the interactive demo, or the [API contract](api.md) for the Axum integration.
