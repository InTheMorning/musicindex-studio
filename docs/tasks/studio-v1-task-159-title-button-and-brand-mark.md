# Studio v1 Task 159: The Title Button And The Brand Mark Of The Narrow Shell

Decision: musicindex ADR 0079, "The Toolbar Title Opens The Root Page" (musicindex `6e4e071`), R-TITLE-1 and R-TITLE-2. It supersedes musicindex ADR 0071, the edge indicator. [ADR 0206](../archive/adr/0206-the-shell-and-the-editor-pages.md) item 11 names it for the narrow shell of Studio. Studio adds no rule, so this task has no Studio ADR.

Status: Complete on 2026-11-16, with its screenshot visual check. musicindex ADR 0080 supersedes musicindex ADR 0079 on the same day. [Task 160](studio-v1-task-160-the-mark-before-back.md) moves the mark out of the title button, before Back. The visual gate for the operator moves to task 160. The orchestrator session sent this task on 2026-11-16.

## Work

1. Remove the edge indicator: the element `.edge-indicator` in the shell, its rule in `styles.css`, `showsEdgeIndicator` in `shell.js` and `edgeindicator.test.mjs`. The copy of `tokens.json` keeps `shape.edge-indicator` until musicindex-assets removes it. Do not change the copy.
2. R-TITLE-1: at 760 px or less, the `Bar` of each page other than the root page puts its title in one `button`. The button starts after the back control and fills the first bar row. It is 44 px high or more. Its accessible name is the visible title, then ", Destinations".
3. The action of the button goes back to the root entry. When the browser history agrees with the stack (`mirrored`), it calls `history.go(-n)`, and `popstate` moves the stack. If not, it moves the stack in place to the root entry, as `back` does, and it does not call `history.go`. A page with pending changes asks first (ADR 0116 item 10).
4. R-TITLE-2: the button starts with the one-color musicindex mark, 24 px high, then the title.
5. The root page and a wide window do not change.

## Technical Choices

- The mark is the token `--mark-mask` of `styles.css`. It is the shape of `brand/musicindex-mark-shape.svg` of musicindex-assets, which Studio already uses for missing art (musicindex ADR 0021 R-ART-2). The bar uses it as a CSS mask, so that the mark has the text color of each theme. `brand/musicindex-mark.svg` has the fixed fill `#F2F2F2`, which does not show on the light bar. Thus no new asset file.
- The button height is `var(--target)`: 44 px with touch and 28 px with a mouse. R-TITLE-1 item 3 cites the Touch target of musicindex ADR 0068 R-TARGET-1, and the Studio guard of musicindex ADR 0041 R-SCALE-4 refuses a fixed 44 px.
- In Studio, the bar title shows at all times (Studio has no rule like musicindex ADR 0048 R-SITE-7). Thus the button shows the mark and then the title at all times. R-TITLE-2 items 3 and 4 have no separate state in Studio.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Navigation state | extend | crates/api/ui/shell.js | A pure function gives the stack at the root entry and the count of entries back to it. The edge indicator function goes. |
| Shell | extend | crates/api/ui/app.js | The shell gives `narrow` and the root action to the bar through `NavContext`. `Bar` puts the title in the button. |
| Styles | extend | crates/api/ui/styles.css | One rule gives the button and the mark. The edge indicator rule goes. |
| Brand mark | reuse | crates/api/ui/styles.css | The token `--mark-mask` is the mark shape of musicindex-assets. The art placeholders use it. |
| Shared tokens | reuse | crates/api/ui/assets/tokens.json | No change. musicindex-assets removes `shape.edge-indicator`. |

## Acceptance Criteria

Mechanical:

1. A Node test of the navigation state: the title button shows only in a narrow window on a page other than the root page. The root action gives index 0 and the root view, and keeps the entries after it. The count of entries back is the index of the current entry. Each failure message names musicindex ADR 0079 and gives the fix.
2. A browser drive at 390 px (the drive of musicindex ADR 0079, "How This Decision Is Checked", with the Studio pages):
   - Open a feed from Releases, then a part of the feed. Activate the title button. Then the root page shows, `history.state.index` is 0, and `history.length` does not change.
   - After a change of width, the stack is not mirrored. One click on the title button shows the root page.
   - The title button is a `button` element, 44 px high or more. Its left edge is right of the right edge of the back control. Its accessible name starts with its visible text and ends with ", Destinations".
   - The mark is the first child of the button, and it is visible at the top of the page and after a scroll.
   - On the root page, no title button and no mark exist.
   - No `.edge-indicator` element is in the DOM.
3. The same drive at 761 px: the title is not a button and has no Tab stop.
4. The fit walk gives no finding at 390 px and 761 px.
5. `tools/gate.sh` passes.

Visual, by the main session:

1. At 390 px, in Light and in Dark: the mark and the title follow the back control, and the mark has the text color.
2. At 390 px, the bar of a feed has two rows. The button fills the first row only.

Visual, by the operator. This gate is open:

1. On the iPhone, open a release, a part of it, and a second page. Touch the title area. The root page "Studio" shows in one action.
2. On that page, swipe from the left edge. The browser leaves Studio.
3. On a page other than the root page, the mark shows at the top left, after the back control. No pill shows at the left screen edge.
4. With a keyboard at a narrow width, press Tab after the back control. The title area gets the focus ring. Enter opens the root page.

## Result

- `shell.js` has `showsTitleButton` and `rootNav`. `Bar` in `app.js` puts the first child, the title, in the button `.bar-home`, after the mark `.bar-mark`. The other children stay after the button. The shell gives the root action `toRoot` through `NavContext`.
- The edge indicator, its rule and its test are removed. The copy of `tokens.json` did not change.
- Mechanical 1: `titlebutton.test.mjs` passes, 3 tests.
- Mechanical 2, walk data of `setup-none.py` at 390 px with touch:
  - Releases, then the outline page of "Album 2": `history.state.index` 2, `history.length` 3. After the title button: the root page "Studio", index 0, and `history.length` 3.
  - The button is a `BUTTON`, 44 px high and 302 px wide. Its left edge is at 72 px, and the back control ends at 60 px. Its first child is `bar-mark`, 24 px high. Its name is "Album 2, Destinations".
  - On Releases after a scroll of 1500 px, the mark is at 16 px from the top and 24 px high.
  - The root page has no `.bar-home` and no `.bar-mark`. No page had an `.edge-indicator`.
  - After a change to 1280 px and back to 390 px, `mirrored` is false. The title button showed the root page, index 0, with the same `history.length`.
- Mechanical 3: at 761 px, no `.bar-home`. The title is a `SPAN` with no Tab stop.
- Mechanical 4: the fit check gave no finding at 390 px and 761 px.
- Visual, by the main session: met. The screenshots show the mark and the title after the back control, in Light and in Dark. On the feed bar, the button fills the first row, and the draft toolbar is on the second row.
- Observation: in the drive, `history.forward()` did not move after the root action. It also did not move after the existing back control, so the cause is not this change. Visual step 1 of the operator can examine Forward on a phone.
