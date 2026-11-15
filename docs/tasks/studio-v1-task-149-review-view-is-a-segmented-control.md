# Studio v1 Task 149: The View Of Review Is A Segmented Control

Decision: none. This is Note 13 of the walk. musicindex ADR 0051 R-SEG-1 owns the segmented control, and [ADR 0157](../adr/0157-the-page-tabs-are-a-segmented-control.md) gives its use in Studio. The task adds no rule.

Status: Not started. The packet is ready for a delegate.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md).

## Goal

The select menu "View" of Review becomes a segmented control with the segments "Simplified" and "Detailed". One click changes the view.

## Operator Choices

- Operator choice (2026-11-15): the view of Review is a segmented control, not a select menu.
- The hint that names the selected view stays. The button **Show the Whole File** stays at the right side in the detailed view.

## Code Locations

- `crates/api/ui/editor.js`, `DiffView`, lines 1585 to 1629. The view bar is at lines 1608 to 1620.
- `crates/api/ui/review.js`, `VIEWS`, `VIEW_LABELS`, `VIEW_HINTS`, lines 12 to 17, and `normalizeView`, lines 34 to 36.
- `crates/api/ui/editor.js`, `PlaylistPickerSheet`, lines 3004 to 3007: an existing segmented control with `aria-pressed`, one Tab stop and `onSegmentKey`.
- `crates/api/ui/app.js`, `onSegmentKey`, lines 2036 to 2044, and `crates/api/ui/shell.js`, `segmentKey`, lines 600 to 609.
- `crates/api/ui/styles.css`, `.segmented` and `.segment`, lines 1481 to 1515, and the rules of `.viewbar`.
- `crates/api/ui-tests/review.test.mjs`, `moreLabel and normalizeView`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Segmented control styles | reuse | crates/api/ui/styles.css | `.segmented` and `.segment` give the form. |
| Segment keys | reuse | crates/api/ui/app.js | `onSegmentKey` moves the focus with the arrow keys, Home and End. |
| View names | reuse | crates/api/ui/review.js | `VIEWS`, `VIEW_LABELS` and `VIEW_HINTS` give the segments and the hint. |
| View bar | extend | crates/api/ui/editor.js | The `select` element becomes the group of segments. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- The rows of the two views
- `crates/api/ui/assets/tokens.json`

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In the view bar of `DiffView`, replace the `select` element and its `label` with a `div` of the classes `segmented` and a `role="group"` with `aria-label="View"` and `aria-describedby` of the hint.
2. Make one `button` of the class `segment` for each entry of `VIEWS`, with `aria-pressed`, the tab index of `PlaylistPickerSheet` and `chooseView` on a click. Use `onSegmentKey` for the keys. If `editor.js` cannot import it, report that and stop.
3. Keep the hint element with its id and `VIEW_HINTS[view]`. Keep the button of the whole file.
4. Change `.viewbar` rules only when the layout needs it. Remove a rule that only the `select` element used.

## Acceptance Criteria

Mechanical:

1. A Node test in `crates/api/ui-tests/review.test.mjs` reads `editor.js`: the view bar has no `select` element, has the class `segmented`, uses `VIEWS` and `onSegmentKey`, and keeps `VIEW_HINTS`. The failure message names musicindex ADR 0051 R-SEG-1 and gives the fix.
2. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. Review of a draft at 1280 px and at 390 px, in Light and in Dark: the two segments, the hint and the button fit, and one click changes the view.
2. The arrow keys move the focus between the segments, and the focus ring shows.

## Test Commands

- `tools/node --test crates/api/ui-tests/review.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-149-review-view-is-a-segmented-control.md` (this file), each section
- `crates/api/ui/editor.js`
- `crates/api/ui/review.js`
- `crates/api/ui/app.js`
- `crates/api/ui/styles.css`
- `crates/api/ui-tests/review.test.mjs`

Goal:
- The select menu "View" of Review becomes a segmented control with the segments "Simplified" and "Detailed". One click changes the view.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- The rows of the two views
- `crates/api/ui/assets/tokens.json`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `tools/node --test crates/api/ui-tests/review.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
