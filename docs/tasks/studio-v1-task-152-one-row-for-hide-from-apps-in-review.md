# Studio v1 Task 152: One Row For "Hide From Apps" In Review

Decision: none. This is Note 10 of the walk. [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md) items 3 to 5 own the switch and its two tags, and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) puts the rule in the core. They do not change.

Status: Complete on 2026-11-15, with the screenshot visual check. The visual gate for the operator is open.

## Result

- The delegate was `gpt-6-luna`. The first run, with medium effort, had no Rust test for criteria 1 to 3. The second run, with high effort, added the three tests.
- Correction of the main session: the code of the delegate removed the change of each block that did not change its `id`. A block with an `id` then lost its row when the switch also changed. The function `is_switch_block` reads the element of the change, and only a block with no `id` goes into the row "Hide from apps". The test `a_platform_block_keeps_its_row_when_the_switch_changes` guards this.
- The guard row counts with the tag changes, so the change of the switch is not an uncovered change.
- The gate passed on the tree with this change.
- Screenshots of the main session: Review of a draft with the switch set to on, at 1280 px in the light theme and at 390 px in the dark theme. The simplified view shows one row "Hide from apps" with "No → Yes" and no guard row. The detailed view shows the lines `<itunes:block>Yes</itunes:block>` and `<podcast:block>yes</podcast:block>`.
- The fit check at 390 px reports word breaks in the XML lines of the detailed view. This task does not change those lines.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md).

## Goal

When the switch "Hide from apps" changes, the simplified view of Review shows one row "Hide from apps" with "No → Yes" or "Yes → No". It does not show the rows "Block in Apple Podcasts" and "Blocks" for that change.

## Operator Choices

- Operator choice (2026-11-15): one row with the name "Hide from apps". Its values are "Yes" and "No", not "On" and "Off".
- Operator choice (2026-11-15): the words "Block in Apple Podcasts" and "Blocks" go.
- Recommendation of the planning model, open until the operator sees the screenshot: a tag change that does not change the state of the switch keeps its own row. The row names are then "itunes:block tag" and "podcast:block tags". A block with an `id` is such a change.

## Code Locations

- `crates/core/src/feed.rs`, `Feed::hidden`, line 294, and `Feed::set_hidden`, lines 310 to 336. `set_hidden` writes "Yes" or "No" in `itunes_block`, and "yes" or "no" in each block with no `id`.
- `crates/core/src/review.rs`, `FieldChange` and `FieldGroup`, lines 70 to 107, `FEED`, lines 413 to 445, and `diff_models`, lines 567 to 605.
- `crates/api/ui/review.js`, `KEY_WORDS`, lines 155 to 178, and `fieldRows`, lines 277 to 309.
- `crates/core/tests/edits.rs`, `set_hidden_true_adds_both_tags` and `set_hidden_false_writes_no_and_keeps_the_platform_block`.
- `crates/api/ui-tests/review.test.mjs`, `the word table has a word for each key of the feed model`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Rule of the switch | reuse | crates/core/src/feed.rs | `Feed::hidden` gives the state before and after. The review adds no second rule. |
| Field group of the review | extend | crates/core/src/review.rs | `diff_models` gives one group `hidden` in place of the tag changes that the switch explains. |
| Word table | extend | crates/api/ui/review.js | `KEY_WORDS` gets `hidden`, and the two tag rows get new names. |
| Row element | reuse | crates/api/ui/editor.js | `FieldRow` shows the row. No change. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- `Feed::hidden`, `Feed::set_hidden` and `Feed::set_platform_hidden`
- The detailed view and the guard row of ADR 0114 item 4
- `crates/api/ui/editor.js`

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. Load the `rust-dev` skill before the Rust change.
2. In `diff_models`, calculate the state of the switch for the old model and the new model with the rule of `Feed::hidden`. If the review model cannot give a `Feed`, stop and report.
3. When the two states differ: add one group of the kind `Field` with the name `hidden` and one change from "No" to "Yes" or from "Yes" to "No". Remove the change of `itunes_block`, and remove each change of a block with no `id`. Keep each change of a block with an `id`.
4. When the two states agree, change nothing in the groups.
5. Make sure that the count of uncovered changes for the guard row does not count the removed tag changes as uncovered. Read how `walk_field` marks a covered change.
6. In `KEY_WORDS`, add `hidden: 'Hide from apps'`. Change `itunes_block` to 'itunes:block tag' and `blocks` to 'podcast:block tags'.

## Acceptance Criteria

Mechanical:

1. A Rust test in `crates/core/src/review.rs`: a model before and after `set_hidden(true)` gives one group `hidden` with "No" and "Yes", and no group `itunes_block` and no change of a block with no `id`.
2. A Rust test: a change of only a block with an `id` gives a `blocks` change and no group `hidden`.
3. A Rust test: a change of only `itunes_block`, while a block with no `id` keeps the feed not hidden, gives the `itunes_block` change and no group `hidden`.
4. The existing test `the_table_has_each_key_of_a_model_with_each_field_set` passes. If it needs a table entry for `hidden`, add the entry.
5. A Node test in `crates/api/ui-tests/review.test.mjs`: `fieldRows` of a group `hidden` gives the label "Hide from apps" and the text "No → Yes". No value of `KEY_WORDS` contains "Apple Podcasts" or is "Blocks".
6. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. Review of a draft with the switch set to on, simplified view: one row "Hide from apps" with "No → Yes". The guard row does not show for this change.
2. The detailed view of the same draft shows the two tag lines as before.

## Test Commands

- `cargo test -p studio-core review`
- `cargo test -p studio-core --test edits`
- `tools/node --test crates/api/ui-tests/review.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-152-one-row-for-hide-from-apps-in-review.md` (this file), each section
- `crates/core/src/feed.rs`
- `crates/core/src/review.rs`
- `crates/api/ui/review.js`
- `crates/core/tests/edits.rs`
- `crates/api/ui-tests/review.test.mjs`

Goal:
- When the switch "Hide from apps" changes, the simplified view of Review shows one row "Hide from apps" with "No → Yes" or "Yes → No". It does not show the rows "Block in Apple Podcasts" and "Blocks" for that change.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- `Feed::hidden`, `Feed::set_hidden` and `Feed::set_platform_hidden`
- The detailed view and the guard row of ADR 0114 item 4
- `crates/api/ui/editor.js`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `cargo test -p studio-core review`
- `cargo test -p studio-core --test edits`
- `tools/node --test crates/api/ui-tests/review.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
