# The street spotters club

**Find the weird. Bring your friends.**

Stadt Land Streetview is a party game for people who treat a badly parked hatchback like a major discovery. The visual identity combines street signage with a cast of slightly nosy original characters.

## Visual vocabulary

- **Logo:** three stacked, slightly misaligned street signs. Bold lettering, directional arrows, dark outlines, and solid offset shadows. The compact version is used in the game HUD.
- **Characters:** flat, outlined SVG spotters with compatible hats, hair, glasses, clothing, skin tones, cameras, notebooks, and binoculars. Distinct silhouettes and accessories should read at avatar size.
- **Typography:** locally bundled Bricolage Grotesque for logo/headings; Nunito Sans for readable controls and supporting text. Strong hierarchy, short labels, tabular countdown digits.
- **Surfaces:** crisp solid fills, thick asphalt outlines, small hard shadows, evidence-ticket details, and restrained street-map dots. The Google map and submitted viewpoints remain the focus of active play.
- **Voice:** short and situational. “Eyes on everything.” “Evidence secured.” “Match or massive stretch?” Instructional copy stays explicit where a player needs to act.

| Color | Value | Use |
| --- | --- | --- |
| Electric blue | `#3657dd` | Game canvas and identity |
| Traffic yellow | `#ffce43` | Primary actions, current assignment, spotter honors |
| Asphalt navy | `#202b45` | Type, outlines, header, camera letterboxing |
| Paper cream | `#fff9e9` | Game boards and inputs |
| Verdict mint | `#b7dfaa` | Ready and accepted states |
| Verdict coral | `#f58c79` | Rejection and timer urgency |

## Generated player identity

`frontend/src/lib/components/spotter.ts` derives each character's parts from the **player ID**. This is deterministic: rendering, refreshing, and joining another client do not reroll a player's appearance. No avatar endpoint, extra DTO field, remote image service, or character-reroll control is needed.

`Spotter.svelte` is original SVG artwork. It supports idle, ready, searching, winner, and offline moods. Names and state labels accompany the portraits; expression and color do not carry essential information on their own.

## A game canvas, not a landing page

The browser viewport is the game boundary. There is no document scrolling and no nested scrolling region. The supported initial desktop design target is **1280 × 720 and larger**.

1. The logo/status HUD, active board, and readiness/action footer have reserved layout space.
2. Longer collections use visible previous/next controls: four categories, three candidate views, or six scoreboard rows per page.
3. Up to twelve active players fit in the lobby grid and readiness footer.
4. Search, selection, and review fit a constant **16:10 viewpoint** by both width and height. Letterboxing is intentional; independent cropping is not.
5. Instruction dialogs are paginated. Error messages and transient notifications do not shift the game board.
6. Browser full-screen mode is optional; fitting the viewport does not depend on it.

Browser tests verify the main game phases and maximum-content fixtures at 1280 × 720, 1440 × 900, and 1920 × 1080. The [UI preview](ui-preview.md) contains actual viewport screenshots.
