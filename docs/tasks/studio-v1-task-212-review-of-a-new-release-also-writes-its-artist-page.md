# Studio v1 Task 212: Review Of A New Release Also Writes Its Artist Page

Decision: [ADR 0231](../adr/0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 7 to 11.

Status: Done. The human check is open.

## Problem

Finding 3 of the [desktop check of tasks 205 to 209](../reviews/2026-11-20-rc29-human-check.md#desktop-check-of-tasks-205-to-209).

- The create of a release calls `link_new_release_pages` in `crates/api/src/routes.rs` near line 1742. That calls `publisher::link`, which adds a `podcast:remoteItem` to the draft of the artist page. The write of the release writes only the release. Thus the artist page keeps a change that is not written, and it has the pencil mark.
- Inference, not examined in a run: when the artist page has no draft, the create can stop with the error `no_draft` after it made the feed row.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link of a new release | extend | crates/api/src/routes.rs | `link_new_release_pages` near line 1742. `ensure_draft` near line 2846. |
| Publisher link | reuse | crates/core/src/publisher.rs | `link` near line 214 edits the draft of the publisher feed. |
| Diff route | extend | crates/api/src/routes.rs | `get_diff` near line 3675. It gives the diff of one feed. |
| Write route | reuse | crates/api/src/routes.rs | `write_feed` near line 3696. The UI calls `/feeds/{id}/write` with `diff_id` and `signed_events`. |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` near line 1976. The write near line 2063. The announce choice `useAnnounceChoice`. The signing of planned events `signPlanned`. |
| Write result | extend | crates/api/ui/writeresult.js | `writeResult` gives `changedPage` (ADR 0218). |
| API tests | extend | crates/api/src/tests.rs | The tests that call `/api/v1/feeds/{id}/diff` and the create of a release. |
| UI tests | extend | crates/api/ui-tests/ | The tests of `writeresult.js` and the source tests of `editor.js`. |

## Changes

1. Item 11: `link_new_release_pages` calls `ensure_draft` for the artist page before `publisher::link`. Add an API test: the create of a release with an artist page that has no draft gives the link and no `no_draft`.
2. Item 7: the diff of a feed that has no first write gives a new field `also_write`. It is a list of the publisher feeds whose draft has changes and has a `podcast:remoteItem` to this feed. Each entry has `id` and `title`. Feeds with a first write give an empty list.
3. Item 7: when `also_write` is not empty, Review shows the check box "Also write {title}" for each entry, on at the start. The text under it is "Adds the link to {release} on the artist page." Review loads the diff of each entry and lists its changes under the check box. Use the same diff view that Review uses for the release.
4. Item 8: **Write to the File Host** writes the release first. Then, for each check box that is on, it writes the artist page with the diff ID of its own diff. Each write signs its own planned events with `signPlanned`. The announce choice applies to the two writes.
5. Item 9: when the write of the release fails, Studio does not write the artist page. When the write of the artist page fails, the release stays written. Review then shows the error of the artist page with its title.
6. Item 10: with the check box off, Review writes only the release, as before.
7. After the two writes, `writeResult` does not give `changedPage` for an artist page that is written.

## Checks

Mechanical:

- API test: change 1.
- API test: the diff of a new release with a linked artist page gives `also_write` with that page. The diff of a feed with a first write gives an empty list.
- API test: a write of the release, then a write of the artist page, leaves no draft change on the artist page.
- Node test (pure function): the check box rows and the order of the writes. A failed write of the release gives no write of the artist page.
- `cargo test -p studio-api`, `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual (main session):

- drive-ui at 1280 px and 390 px: Review of a new release with the check box and the changes of the artist page. After the write, the artist page has no pencil mark.

Human:

- Make a release on the real collection and write it with the check box on. The pencil mark of the artist page goes.

## Result

Changes:

- `crates/api/src/routes.rs` line 1749: `link_new_release_pages` calls `ensure_draft` for the artist page before `publisher::link` (change 1).
- `crates/api/src/routes.rs` line 3667: the new function `also_write_pages`. It uses `publisher::publishers_linking` and keeps each publisher feed with a draft that has changes. The flag of a new release is `last_version` is `None` in the feed row. A feed with a first write gives an empty list. The diff JSON has the new field `also_write` (line 3649), a list of `{ id, title }` (change 2).
- `crates/api/ui/writeresult.js`: the new pure functions `alsoRows`, `rowsToWrite`, `pageWriteFailed` and `writeInOrder`. `writeInOrder` writes the release first, then each page in order. It sets `pageTitle` on the error of a failed page write. `writeResult` has the new parameter `writtenIds`, and it gives no `changedPage` for a page that is written (change 7). The functions are in this file because it is already in the file list of `crates/api/src/ui.rs`.
- `crates/api/ui/editor.js`, `ReviewSheet`: it loads the diff of each `also_write` entry. It shows the check box "Also write {title}", on at the start, with the note and the changes of the page under it (change 3). The write button waits for these diffs. `write` uses `writeInOrder`. Each page write signs its own events with `currentSignerChoice` and `signPlanned`, and posts to `/feeds/{id}/write` with its own `diff_id` (change 4). The announce choice is a global setting, so it applies to both writes.
- `crates/api/ui/editor.js`: a failed page write shows the notice "Studio could not write {title}." and the release stays written (change 5). The conflict and refusal notices use only the error of the release write. With the check box off, only the release is written (change 6).
- Tests: `crates/api/src/tests.rs` line 1373 (change 1) and line 1400 (`also_write`, the empty lists, and the two writes that leave no draft change). `crates/api/ui-tests/writeresult.test.mjs`: seven new tests for the rows, the order of the writes, a failed release write and a failed page write.
- Red check: without the `ensure_draft` line, the test at line 1373 fails with `no_draft`. This proves the inference of item 11.

Checks:

- `cargo test -p studio-api`: 459 unit tests pass, and each other test file passes.
- `cargo clippy -p studio-api`: no warning.
- `tools/node --check` on a copy of `crates/api/ui/editor.js` and of `crates/api/ui/writeresult.js`: passes.
- `tools/node --test crates/api/ui-tests/*.test.mjs`: 936 pass, 0 fail.
- `bash tools/gate.sh`: "gate: each check passed".
- Visual (2026-11-20, main session): drive-ui at 1280 px and 390 px on a copy of `.studio-dev/v-207/data`. Host A of the copy has its own folder and a local web server. A step made the artist page "Demo Artist", wrote it, and made the release "Demo Single" under it.
- Review of "Demo Single" shows the check box "Also write Demo Artist", on, with the note and the change "Remote items, Added: Demo Single". The rows use the classes `group` and `check` and need no new CSS. `$edges()` gives no finding.
- After one press of **Write to the File Host**, the result says "Demo Single is written to Host A.", and `/api/v1/feeds` gives the draft state `none` for "Demo Artist". The sidebar shows "Demo Artist" with no pencil mark.
- Human: open.

Main session changes after the review:

- The API test at line 1400 now writes the artist page with the diff ID that it read before the write of the release, as Review does. The test passes, so that diff ID stays valid after the write of the release.
- Operator choice (2026-11-20, main session): `link_new_release_pages` also calls `ensure_draft` before `publisher::link` for a hosting or label page of the newest release. A written label page had the same `no_draft` fault. This is a defect correction of ADR 0231 item 11, not a new decision. The test `a_new_release_keeps_its_kind_and_inherits_the_newest_artists_pages` now writes the three pages before the create. Without the new line, the create stops with `no_draft` for the label page (red check).
- Each entry of `also_write` has the new field `artist`. For a label or hosting page, the note is "Adds the link to {release} on {title}.", because the words "the artist page" are not correct for it. The same test proves that Review offers the three pages.
- If the read of a page diff fails, Review shows the notice "Studio could not examine the artist page." and writes only the release.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Read the `rust-dev` skill before you change Rust code.
- Do not commit and do not stage. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
- Syntax check: copy each changed `crates/api/ui/*.js` file to `$TMPDIR/x.mjs`, then run `tools/node --check` on the copy.
