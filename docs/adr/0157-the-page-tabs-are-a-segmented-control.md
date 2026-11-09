# ADR 0157: The Page Tabs Are A Segmented Control

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator, through the orchestrator session. The tabs of a feed page must look like the scope control of the search page of musicindex.org.
- Uses: musicindex ADR 0051 R-SEG-1 and R-SELECT-1, and the tokens of `../musicindex-assets` that it gives.
- Changes: [ADR 0135](0135-the-panel-holds-the-content-and-the-page-has-tabs.md) item 6, the look of a selected tab.

## Context

The pages of a feed are tabs on the feed page (ADR 0135). Each tab was its own rounded button. The selected tab had the accent tint and an accent outline. musicindex ADR 0051 gives one segmented control and one selected look to the three apps, and the search page of musicindex.org uses them.

## Decision

1. The tabs of a feed page are a segmented control of R-SEG-1 items 1 to 7. A tray with the fill `fill-soft` holds the segments. Each segment has the same width, no fill of its own and the muted text color.
2. The selected tab has the selected look of R-SELECT-1: the gradient, the inner edge, the top highlight, the text color and the semibold weight. It keeps `aria-current="page"`.
3. When the page is too narrow for the segments, the tray keeps its width. A label then wraps in its segment, as on musicindex.org. At 760 px or less, the tabs stay one select menu (ADR 0135 item 6).
4. Each other row of Studio that selects one view of the same content gets the same look. The sort menu, the view icons of Releases and the rows of the left panel keep their own look.
5. Each segmented control of Studio is one Tab stop: the selected segment (R-SEG-1 item 7). Left and Right move the focus to the next segment and wrap, and Home and End go to the first and the last segment. A key moves only the focus, and Enter or Space selects the segment. This is the manual activation of the scope control of musicindex.org. The elements, their roles and `aria-current` or `aria-pressed` do not change.
6. Studio copies `tokens.json` from `../musicindex-assets` with the values of musicindex ADR 0051.

## How This Decision Is Checked

Mechanical: the Node tests compare `styles.css` with `tokens.json`. A Node test of `shell.js` examines the keys of item 5.

Visual, by the agent:

- The album page at 1280 px and at 800 px, in the dark and the light theme.
- The keyboard focus on a tab.
- One image of the tabs adjacent to the scope control of musicindex.org.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).
