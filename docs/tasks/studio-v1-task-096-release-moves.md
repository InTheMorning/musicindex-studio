# Studio V1 Task 096: Release Moves

Status: In progress. Packets 096a and 096b are complete, with the screenshot visual check of 096b.

Owner of the rules: [ADR 0148](../adr/0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md).

## Goal

A user moves a release, or all releases of a file host, in one action. The old files stay until the user reviews their deletion.

## Packets

- 096a: the core and the routes. The record of an old copy, the operation of the draft, the write, the plan, the three checks and the release of a deletion. **Move to** follows the new rule.
- 096b: the screens of one release. The sheet, the list of the moves that wait, the notice of a ready move and the notice of ADR 0148 item 15.
- 096c: Move All Releases, with the change set of ADR 0148 items 16 to 19.

## Files To Inspect

`crates/core/src/collection.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/change_set.rs`, `crates/api/src/routes.rs` (`add_operation`, the feed host route), `crates/api/ui/editor.js` (the Files page, `CopyRow`), `crates/api/ui/confirms.js`, `docs/user-guide/release-files.md`.

## Constraints

- No feed rule in the UI (ADR 0065).
- A write to a host occurs only through `pipeline::write_draft` (ADR 0007, ADR 0015).
- No deletion on a host occurs before each new file passes the confirmation (ADR 0043).
- A one-setting choice is a select menu. A confirmation is a sheet of `confirms.js`.

## Acceptance Criteria

Mechanical, with Rust tests in `studio-core` and `studio-api`:

- A write of a moved file sends the new copy and deletes no old copy. The collection then holds one record for each old copy.
- The record has the waiting time of the request. **Move to** has 30 days.
- The state of a move is not ready before its days are complete, and not ready when musicindex gives a different address. It is ready with no answer from musicindex after its days.
- The release route adds one deletion for each record, and only a written diff removes the record and the file.
- "Keep the Files" removes the records, and no file changes.
- A later write of a file to the path of a record removes that record.
- The plan leaves out a file that is not in the library and a file on the target, and a Blossom target keeps the feed file.
- A change set of a move has one feed row for each release on the file host. Discard puts back each draft and removes the operations.

Mechanical, with Node tests: the words of the plan and the state lines come from a model module.

Visual, by a screenshot at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()`, `$overflow()` and `$fieldText()` with each screenshot:

- The sheet, a move that waits, a ready move and the notice of a part move look like the mockup.
- The sheet and the change set of Move All Releases.

Human, by the operator: a move between two real file hosts, a waiting move on the next day and a reviewed deletion.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

A decision in ADR 0148 cannot hold with the pipeline, or a change needs a rule that the ADR does not state.
