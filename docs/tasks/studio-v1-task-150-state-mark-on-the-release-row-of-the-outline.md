# Studio v1 Task 150: The State Mark On The Release Row Of The Outline

Decision: none. This is Note 14 of the walk. [ADR 0165](../archive/adr/0165-the-left-panel-marks-the-state-of-each-feed.md) items 1, 2 and 6 own the state marks and their store, and they do not change. The outline is a part of the left panel.

Status: Complete on 2026-11-15, with the screenshot visual check. The visual gate for the operator is open.

## Result

- The delegate was `gpt-6-luna` with medium effort.
- The main session added one CSS rule. It puts the mark at the right side of the row, as in a level row.
- Screenshots of the main session: a feed with an outside change, at 1280 px and at 390 px, in the light theme and the dark theme. The level row and the release row show the cloud mark in the Warning color. The title is not cut, and `$overflow()` is empty.
- The accessible name of the row has the text of the mark, for example "Album 2, Album, Changed outside Studio".

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md).

## Goal

The row of the open release in the outline, above its tracks, shows the state mark of that feed. A release with the cloud mark in the list keeps the mark after the user opens it.

## Operator Choices

- Operator choice (2026-11-15): the release row above the track list shows the orange cloud mark, so that the user finds the way to **Read Again**.
- Recommendation of the planning model, open until the operator sees the screenshot: the row shows each state mark of ADR 0165 item 2 for that one feed, not only the cloud. One function then gives the mark in the two places.

## Code Locations

- `crates/api/ui/app.js`, `Outline`, lines 3247 to 3278. The button `.outline-feed` is the row.
- `crates/api/ui/app.js`, `OutsideContext`, `useStateMark` and `StateMark`, lines 1906 to 1920. `Level`, lines 3195 to 3220, is the existing use.
- `crates/api/ui/outsidecheck.js`, `STATE_MARKS`, `isUnchecked` and `stateMark`, lines 165 to 196.
- `crates/api/ui/styles.css`, `.state-mark` and its tone classes, lines 694 to 699, and the rules of `.outline-feed`.
- `crates/api/ui-tests/outsidecheck.test.mjs` and `crates/api/ui-tests/shell.test.mjs`, `an open album gives the outline of ADR 0135 item 1`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| State mark of a set of feeds | reuse | crates/api/ui/outsidecheck.js | `stateMark` gives the mark of a list with one feed. |
| Mark element and store | reuse | crates/api/ui/app.js | `useStateMark` and `StateMark` read `OutsideContext` and show the icon. |
| Mark styles | reuse | crates/api/ui/styles.css | `.state-mark` and the tone classes. |
| Outline row | extend | crates/api/ui/app.js | The row shows the mark and adds its text to the accessible name. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- `crates/api/ui/outsidecheck.js` (the rules of the marks do not change)
- The rows of the tracks

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In `Outline`, find the feed object of `OutsideContext` that has the id of the open feed, and call `useStateMark` with a list of that one feed. Read how `Level` makes its list, and give the same form.
2. Show `StateMark` in the row `.outline-feed`, at the right side of the text block, as `Level` does in its row.
3. Add the text of the mark to the `aria-label` of the row, after the label and the text of a change in the library folder.
4. Add a CSS rule only if the mark does not align in `.outline-feed`. Use existing spacing tokens.

## Acceptance Criteria

Mechanical:

1. A Node test in `crates/api/ui-tests/outsidecheck.test.mjs`: `stateMark` of a list with one feed gives the `outside` mark with the icon `cloud-alert` for a result with an outside change, and nothing for a feed that is up to date.
2. The same test file reads `app.js`: `Outline` uses `useStateMark` and `StateMark`. The failure message names ADR 0165 items 2 and 6 and gives the fix.
3. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. With a feed that has an outside change, at 1280 px: the list row and the release row of the outline show the same cloud mark in the Warning color, in Light and in Dark.
2. At 390 px, the outline page shows the mark and the title is not cut more than before.

## Test Commands

- `tools/node --test crates/api/ui-tests/outsidecheck.test.mjs crates/api/ui-tests/shell.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-150-state-mark-on-the-release-row-of-the-outline.md` (this file), each section
- `crates/api/ui/app.js`
- `crates/api/ui/outsidecheck.js`
- `crates/api/ui/styles.css`
- `crates/api/ui-tests/outsidecheck.test.mjs`

Goal:
- The row of the open release in the outline, above its tracks, shows the state mark of that feed. A release with the cloud mark in the list keeps the mark after the user opens it.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- `crates/api/ui/outsidecheck.js` (the rules of the marks do not change)
- The rows of the tracks

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `tools/node --test crates/api/ui-tests/outsidecheck.test.mjs crates/api/ui-tests/shell.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
