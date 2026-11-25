# Studio v1 Task 257: The Generator Tag

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md).
Decision: [ADR 0265](../adr/0265-a-changed-feed-names-studio-as-its-generator.md) items 1 to 13. Read [ADR 0127](../adr/0127-a-changed-feed-gets-a-new-build-date.md) items 2 to 6 and 21 first. The tag follows the path of the build date.

Status: built. The screenshot check of Review passed. The human check of the operator is open.

Operator choices (the main session):

- A new tag goes at the end of the channel, before the items (ADR 0012 item 3). When the date is also new, the tag comes before `lastBuildDate`, because the core sets the tag first. RSS 2.0 gives no sequence for channel elements.
- A core unit test in `crates/core/src/merge.rs` covers ADR 0265 item 13, the merge of a hosted file.

## Goal

When Studio makes the diff of a changed feed, the channel `generator` gets the text `Music Index Studio VERSION`. A feed with no change keeps its bytes.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed model | extend | crates/core/src/feed.rs | Add `generator: Option<String>` with `#[serde(default)]`. `Feed::read` reads the channel `generator`. `write_hosted` writes it directly after `lastBuildDate`. |
| Generator text | extend | crates/core/src/feed.rs | One public constant `GENERATOR`, from `concat!("Music Index Studio ", env!("CARGO_PKG_VERSION"))` (item 2). |
| Review field table | extend | crates/core/src/review.rs | Add `child("generator", Ns::None, "generator")` after `last_build_date` in `FEED`. |
| Stamp compare and base stamp | extend | crates/core/src/pipeline.rs | `without_build_date`, `same_apart_from_date`, `with_base_date` and `revision` also treat `generator` (items 5, 8 and 9). Keep one function for each task. Do not add a copy. |
| Set at the diff | extend | crates/core/src/pipeline.rs | `set_build_date` sets `generator` to `GENERATOR` when `changes_feed` is true, before the checks of the date and also with `checked` of `None` (items 6 and 7). Save the draft only when the tag changes. |
| Model from the UI | extend | crates/core/src/pipeline.rs | The two places with `sent.last_build_date.clone_from(&current.last_build_date)` also keep `generator` of `current` (item 1). |
| Granted write | extend | crates/core/src/pipeline.rs | Step 3 of the granted write sets `changed.generator` to `GENERATOR` (item 11). |
| Grant check | extend | crates/core/src/grant.rs | `covered` also makes `generator` equal (item 11). |
| Hosted merge | extend | crates/core/src/merge.rs | The skip list also has `generator` (item 13). |
| Review field name | extend | crates/api/ui/review.js | The field name map gets `generator: 'Generator'` (item 12). |
| Core tests | extend | crates/core/tests/pipeline.rs | Next to the build date tests (near line 5900). Reuse the helpers of those tests. |
| Node tests | extend | crates/api/ui-tests/review.test.mjs | The field name "Generator". |

## Constraints

- No feed rule in the UI (ADR 0065). The UI only shows the name of the field.
- The corpus round-trip tests must pass with no change: a read and a write with no edit give the same bytes.
- Existing tests that compare the full bytes of a changed feed can now see a `generator` line. Change such a test only to expect the tag, and say which tests changed in the report.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A test message names the ADR item and the fix, for example: `ADR 0265 item 6: the diff of a changed feed must set the generator. Set it in pipeline::set_build_date.`
- Do not commit. Leave the change in the working tree and report it.

## Acceptance criteria

Mechanical:

1. A core test: a read and a write with no edit give the same bytes, and a save with no edit gives no tag.
2. A core test: the diff of a changed draft of a feed with `<generator>MSP 2.0 - Music Side Project Studio</generator>` gives `GENERATOR` in the same position. It does this also with `checked` of `None`. A second diff gives the same diff ID.
3. A core test: a changed feed with no tag gets one `generator` element directly after `lastBuildDate`, or in the position of ADR 0012 item 3.
4. A core test: after a diff, the next save of the editor with its revision passes.
5. A core test: an edit that the user undoes gives the bytes of the base, and `set_build_date` then adds no tag.
6. A core test: a model from the UI with a different `generator` does not change the tag.
7. A core test: a granted write sets the tag and passes `grant::covered`.
8. A core test: `grant::covered` is true when only the live item and `generator` differ.
9. A Node test: the field name of `generator` is "Generator".
10. `tools/gate.sh` passes.

Visual, by the main session:

- Review shows the line "Generator" of a changed feed at 1280 px.

Human, by the operator:

- After a write to a real file host, the hosted file has the tag of ADR 0265 item 2.
