# Studio v1 Task 161: The Focus Ring Gap On An Accent Fill

Decision: musicindex ADR 0073 R-FOCUS-2 item 6 (musicindex `804589b`). The token is `focus-ring.studio.filled-offset` of musicindex-assets (`c81e250`). Studio adds no rule, so this task has no Studio ADR.

Status: Complete on 2026-11-16, with its screenshot visual check. The orchestrator session sent this task on 2026-11-16.

Operator choice (2026-11-16): a control with the accent fill or the focus fill has its 2 px focus ring 2 px outside the control. The page background shows in the gap. Each other control keeps the ring with no gap.

## Problem

In the Dark theme, `--focus` and `--accent` are both `#2d7bff`. The rule `:focus-visible` of `styles.css` gives each control a 2 px ring with no gap. A primary button has the accent fill, so its ring has the color of its fill and does not show. In the Light theme the two colors are almost the same.

## Work

1. Find each control that can get the focus and has the accent fill or the focus fill.
2. Give each one `outline-offset: 2px` for `:focus-visible`.
3. Copy `tokens.json` from musicindex-assets (`a4ffa5f`). The custom property `--focus-filled-offset` has the token value, and each such control uses it.
4. Add a `key` step to `tools/drive-ui.mjs`, so that a drive can move keyboard focus.

## Technical Choices

- `.button.primary` is the only such control. It is a `button`, an `a` or a `label`.
- A checkbox has the 2 px gap in each state, now through `--focus-filled-offset`. The gap stays on an unchecked box, so that the ring does not move when Space checks the box.
- The copy of `tokens.json` also gets the owner line that names musicindex ADR 0080, and loses `shape.edge-indicator`.
- The `.steps` items, the `.steps-line` marks, the `.welcome-icon.primary` icon, the done `.delete-step` number and `.change-mark` have the accent fill. None of them can get the focus.
- The label of the file button of `.import-pick` already has a 3 px ring with a 2 px gap.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Styles | extend | crates/api/ui/styles.css | One rule after `.button.primary`. |
| Shared tokens | extend | crates/api/ui/assets/tokens.json | A copy of the file of musicindex-assets. |
| Token guard | new | crates/api/ui-tests/focus-ring.test.mjs | Searched `type-dot.test.mjs` and `text-styles.test.mjs`. They examine other tokens, and neither examines the focus ring. |
| Browser drive | extend | tools/drive-ui.mjs | A `key` step sends a key press through `Input.dispatchKeyEvent`. |

## Acceptance Criteria

Mechanical:

1. A drive with Tab gives keyboard focus to "Review" (primary) and to "Versions" (plain) on the page of a feed. It does this at 390 px with touch and at 1280 px with a mouse, in Dark and in Light. The computed `outline-offset` is 2 px on "Review" and 0 px on "Versions", and `:focus-visible` matches.
2. `focus-ring.test.mjs`: `--focus-filled-offset` has the value of `tokens.json`, and the rules of `.button.primary` and the checkbox for `:focus-visible` use it. Each failure message names R-FOCUS-2 item 6 and gives the fix.
3. `tools/gate.sh` passes.

Visual, by the main session:

1. In the shots, the ring of "Review" shows outside the button, with a gap of the page color. The ring of "Versions" has no gap.

## Result

- Mechanical 2: `focus-ring.test.mjs` passes, 2 tests.
- Mechanical 1: met in each of the 8 cases, after the change to the token. "Review": `:focus-visible` true, a solid 2 px ring, offset 2 px. "Versions": offset 0 px. The ring is `rgb(45, 123, 255)` in Dark and `rgb(0, 91, 211)` in Light.
- Visual: met. The shots are in `.studio-dev/walk/runs/focus/shots/`, `focus-primary-*` and `focus-plain-*`, for `dark-390`, `dark-1280`, `light-390` and `light-1280`.
