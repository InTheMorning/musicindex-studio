# Studio v1 Task 160: The Mark Before Back In The Narrow Shell

Decision: musicindex ADR 0080, "The Interface Rules And The Shell Of The Three Apps, With The Mark Before Back" (musicindex `72b4e4a`), R-SHELL-2 item 6, R-TITLE-1 and R-TITLE-2. It supersedes musicindex ADR 0069 and ADR 0079, and keeps each rule ID. [ADR 0206](../archive/adr/0206-the-shell-and-the-editor-pages.md) items 10 to 12 name it for the narrow shell of Studio. Studio adds no rule, so this task has no Studio ADR.

Status: Complete on 2026-11-16, with its screenshot visual check. The visual gate for the operator is open. The orchestrator session sent this task on 2026-11-16.

Operator choices of 2026-11-16, relayed by the orchestrator session:

- In a narrow window, each page other than the root page has this order: the home button (the mark), Back, Forward where the profile has it, and the title button. In each other layout, Back stays first.
- The home button and the title button have the same action: they open the root page.
- The left edge of the visible mark is on the content margin. The target of the home button keeps its size and extends into the margin.

## Work

1. Move the mark out of the title button of [task 159](studio-v1-task-159-title-button-and-brand-mark.md). The mark is the content of a new home button, the first control of the bar.
2. The accessible name of the home button is "Destinations". The title button keeps its name: the visible title, then ", Destinations".
3. The two buttons use the root action of task 159.
4. Change each current Studio reference to musicindex ADR 0069 or ADR 0079 so that it names musicindex ADR 0080. Do not change the archived records. Also do not change the dated reviews, the research notes or the complete task files. They record what was read on their date.

## Technical Choices

- The home button target is `var(--target)` wide and high: 44 px with touch and 28 px with a mouse. The mark is in the middle of the target. The start margin is half of the mark width less the target width, a negative value. Thus the left edge of the mark is on the bar padding, 16 px at 390 px.
- The shell function `showsTitleButton` becomes `showsRootButtons`, because it now controls two buttons. The class `.bar-home` is the home button, and the class `.bar-title-button` is the title button.
- The copy of `tokens.json` names musicindex ADR 0069 in its `owner` field. musicindex-assets owns that copy, so this task does not change it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Navigation state | extend | crates/api/ui/shell.js | The function of task 159 gives the visibility of the two buttons. `rootNav` does not change. |
| Bar | extend | crates/api/ui/app.js | `Bar` adds the home button before the back control. The title button loses the mark. |
| Styles | extend | crates/api/ui/styles.css | The rule of task 159 splits into the home button and the title button. |
| Brand mark | reuse | crates/api/ui/styles.css | The token `--mark-mask` of task 159. |

## Acceptance Criteria

Mechanical:

1. `rootbuttons.test.mjs`: the two buttons show only in a narrow window on a page other than the root page. The root action gives the root entry. The template of `Bar` has the order home button, Back, Forward, title button, and the home button has the name "Destinations". Each failure message names musicindex ADR 0080 and gives the fix.
2. A browser drive at 390 px with touch:
   - The first three controls of the bar are the home button "Destinations", Back and the title button.
   - The home button is 44 px wide and high. The mark is 24 px high, and its left edge is on the bar padding. The home button ends before the back control, with the bar gap.
   - The home button and the title button each show the root page, with `history.state.index` 0 and the same `history.length`.
   - After a change of width, the stack is not mirrored. The home button shows the root page.
   - The root page has no home button and no title button.
3. The same drive at 390 px with a mouse: the order is the home button, Back, Forward, then the title button, each 28 px high.
4. The same drive at 761 px: no home button and no title button. Back is the first control. The title is not a Tab stop.
5. The fit walk gives no finding at 390 px and 761 px.
6. `tools/gate.sh` passes.

Visual, by the main session:

1. At 390 px, in Light and in Dark: the mark is at the left, then Back, then the title.
2. The left edge of the mark is on the same line as the left edge of the page content.

Visual, by the operator. This gate is open:

1. On the iPhone, open a release and a part of it. The mark shows at the top left, before Back. Touch the mark. The root page "Studio" shows in one action.
2. Open a part again. Touch the title. The root page shows.
3. On the root page, swipe from the left edge. The browser leaves Studio.
4. With a keyboard at a narrow width, press Tab from the top of the page. The focus goes to the mark, then Back, then Forward, then the title. Enter on the mark opens the root page.

## Result

- `Bar` in `app.js` puts the home button `.bar-home` first, with the mark `.bar-mark`. The title button `.bar-title-button` follows the back control and Forward.
- Mechanical 1: the Node tests pass, 834 tests, 0 failures.
- Mechanical 2, walk data of `setup-none.py` at 390 px with touch:
  - The first three controls are `bar-home` "Destinations", `back` "Back" and `bar-title-button` "Album 2, Destinations".
  - The home button is 44 by 44 px, from 7 px. The mark is 24 px high and 25.9 px wide, from 16 px. The bar padding is 16 px. The back control is from 63 px to 107 px, and the title button starts at 119 px.
  - The home button: the root page "Studio", index 0, the same `history.length`. The title button from index 2: the root page, index 0, the same `history.length`.
  - After a change to 1280 px and back to 390 px, `mirrored` is false. The home button showed the root page, index 0.
  - The root page has no `.bar-home` and no `.bar-title-button`.
- Mechanical 3: at 390 px with a mouse, the controls are `bar-home`, Back, Forward and `bar-title-button`, each 28 px high.
- Mechanical 4: at 761 px, no `.bar-home` and no `.bar-title-button`. The first control is "Back". The title is a `SPAN` with tab index -1.
- Mechanical 5: the fit check gave no finding at 390 px and 761 px.
- Visual, by the main session: met. The shots show the mark at the left edge of the content, then Back, then the title, in Light and in Dark.
