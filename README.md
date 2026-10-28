# StadtLandStreetview

A browser-based multiplayer game in which players explore Google Street View to find scenes that match categories created by the group. After a timed search, everyone reviews the submitted images and votes on whether they match. The player with the most accepted matches wins.

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

### Winning

Once all findings have been reviewed, count each player's accepted matches across all categories. **The player with the most accepted matches wins.**

## Rules still to decide

The core game flow above is established. The following details remain open:

- How categories are proposed, chosen, and finalized, including how many to use.
- How the overall search time is chosen.
- Whether the final-selection step has its own time limit and how players signal they are ready.
- What happens when a player has no submission or has not selected a final candidate for a category.
- Whether players can vote on their own findings.
- How many votes are needed to accept a finding, and how tied votes are handled.
- How a tie for the highest number of accepted matches is handled.
- Whether a saved finding is a captured image or a reproducible Street View viewpoint, and which view details must be preserved.
