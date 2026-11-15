# Studio v1 Task 151: An Added Live Item Shows Its Times In Review

Decision: none. This is Note 12 of the walk. [ADR 0114](../adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md) owns the field view and the local time, and it does not change.

Status: Not started. The packet is ready for a delegate.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md).

## Goal

In the simplified view of Review, the row of an added live item or a removed live item shows its state, its start and its end. Each time shows in local time.

## Operator Choices

- Operator choice (2026-11-15): the row of an added live item shows its times in a form that a user can read.
- Recommendation of the planning model, open until the operator sees the screenshot: the text is "Added: {status}, {start} to {end}". A removed live item shows the same parts after "Removed". A part that the item does not have is left out.

## Code Locations

- `crates/api/ui/review.js`, `brief`, `valueText` and `changeRow`, lines 201 to 247. For an added entry with a name in `change.entry`, the text is only "Added".
- `crates/api/ui/review.js`, `TIME_KEYS`, `localTime` and `localTimeNote`, lines 55 to 81.
- `crates/core/src/review.rs`, `item_groups`, lines 689 to 756: a live item is in a group of the kind `live`, and the added object is in `change.new`.
- `crates/core/src/feed.rs`, `LiveItem`, lines 866 to 879: `status`, `start`, `end`.
- `crates/api/ui-tests/review.test.mjs`, `a title, an added entry and a removed entry have a line` and `a build date in words shows the local time`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Local time of a feed time | reuse | crates/api/ui/review.js | `localTime` gives the text. Its `zone` parameter is for the tests. |
| Change row | extend | crates/api/ui/review.js | `changeRow` adds the parts for a group of the kind `live`. |
| Row element | reuse | crates/api/ui/editor.js | `FieldRow` shows the text of the row. No change. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- `crates/core/`
- `crates/api/ui/editor.js`
- The detailed view

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. Read one `live` group in the test data of `review.test.mjs`, or make one from `item_groups`, to learn the form of `group.kind` and of the added object.
2. In `review.js`, add a pure function `liveParts(value, zone)`. It gives the list of the present parts: the status, and the start and the end through `localTime` joined with " to ". A time that `localTime` cannot read shows as its text.
3. In `changeRow`, for an added change or a removed change of a whole live item, set the text to the verb, then ": " and the parts joined with ", ". With no part, the text stays the verb. Give `changeRow` an optional last parameter `zone` for the tests.
4. Change no other row.

## Acceptance Criteria

Mechanical:

1. A Node test in `crates/api/ui-tests/review.test.mjs`: an added live item with the status `pending`, a start and an end gives "Added: pending, {local start} to {local end}" in the zone `UTC`. A removed live item gives the same parts after "Removed". An added track keeps the text "Added".
2. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. Review of a draft that adds a live show, simplified view, at 1280 px and at 390 px: the row shows the state and the two local times, and the text wraps and is not cut.

## Test Commands

- `tools/node --test crates/api/ui-tests/review.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-151-an-added-live-item-shows-its-times-in-review.md` (this file), each section
- `crates/api/ui/review.js`
- `crates/core/src/review.rs`
- `crates/core/src/feed.rs`
- `crates/api/ui-tests/review.test.mjs`

Goal:
- In the simplified view of Review, the row of an added live item or a removed live item shows its state, its start and its end. Each time shows in local time.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- `crates/core/`
- `crates/api/ui/editor.js`
- The detailed view

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
