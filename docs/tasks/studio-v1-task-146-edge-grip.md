# Studio v1 Task 146: The Edge Grip Of The Narrow Shell

Decision: musicindex ADR 0071, "The Edge Grip Shows That A Swipe Goes Back" (musicindex `bfeb0c4`). [ADR 0206](../adr/0206-the-shell-and-the-editor-pages.md) item 11 names it for the narrow shell of Studio. Studio adds no rule, so this task has no Studio ADR.

Status: Complete on 2026-11-14, with its screenshot visual check. The visual gate for the operator is open.

The orchestrator session sent this task on 2026-11-14. The operator selected the grip on that day.

## Result

- `crates/api/ui/assets/tokens.json` is a copy of `tokens/tokens.json` of musicindex-assets `a86639e`. It has the entry `shape.edge-grip`.
- `showsEdgeGrip(nav, narrow, coarse, mirrored)` in `crates/api/ui/shell.js` gives the conditions of R-GRIP-2. `mirrored` is the state of the shell that says that the browser history has one entry for each entry of the stack. After a change of the window width, for example a turn of the phone, it is false until the next load, and no grip shows. The shell in `crates/api/ui/app.js` shows one element `.edge-grip` with `aria-hidden="true"`.
- The rule `.edge-grip` in `crates/api/ui/styles.css` has the values of the token. The element has a fixed position, no pointer events and no animation.
- The vertical position uses `--bar-h`, the height of a bar with one row. The bar of a feed has two rows at this width. There the grip is a small distance above the center of the content area.

- Walk: `steps-146.json` with `setup-none.py`, at 390 px with a coarse pointer after a new load. The pill showed on Releases, on the outline and on a part of a feed, in Dark and in Light. It was 5 px by 48 px, 4 px from the left edge, with no pointer events. A sheet showed above it. The root page, a fine pointer at 390 px and a window of 1280 px showed no pill.

## Open Finding: Back Before The First Input

The website session reported that a browser can skip a history entry that a page made with no input of the user. Studio makes its root entry at the start. Thus Back on the start page, before the first touch, possibly leaves Studio.

The check tool of the agent cannot push the Back button of a browser, so the agent did not examine this. On the start page, `showsEdgeGrip` gives true, because the root entry is below. If the browser skips that entry, the grip shows a swipe that leaves Studio. R-GRIP-2 item 3 does not permit that state. Only a person with a phone can examine it. No history code changed.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Shared tokens | reuse | crates/api/ui/assets/tokens.json | The copy of the shared file gives the values of the grip. |
| Navigation state | extend | crates/api/ui/shell.js | A pure function gives the conditions of the grip from the state. |
| Shell | extend | crates/api/ui/app.js | The shell shows the element. It already has `nav`, `narrow` and `coarse`. |
| Styles | extend | crates/api/ui/styles.css | One rule gives the form of the grip. |

## Acceptance Criteria

Mechanical:

1. A Node test of `showsEdgeGrip`: true on a page above the root page with the Touch profile in the narrow layout. False on the root page, with no entry below, with the Desktop profile and in a wide window. False also when the browser history does not agree with the stack. Each failure message names musicindex ADR 0071 and gives the fix.
2. A Node test: the copy of `tokens.json` has the entry `shape.edge-grip` with the values of R-GRIP-3, and the rule `.edge-grip` has those values, `pointer-events: none` and no animation.
3. The same test: the shell shows the element with `aria-hidden="true"` only through `showsEdgeGrip`.
4. The five commit checks pass.

Visual, by the main session:

1. At 390 px with a coarse pointer, a page above the root page shows the pill at the left edge, in Light and in Dark. The root page shows no pill.
2. At 390 px with a fine pointer, and at 1280 px, no pill shows.
3. An open sheet shows above the pill.

Visual, by the operator. This gate is open:

1. On the iPhone, open Studio and touch a row. The pill shows at the left edge, near the vertical center.
2. Examine the pill in Light and in Dark, and on a page with art at the screen edge.
3. Swipe from the left edge until the root page shows. That page has no pill.
4. Open Studio in a new tab, touch nothing, and swipe from the left edge. The root page must show. If the browser leaves Studio, the finding above is a defect.
