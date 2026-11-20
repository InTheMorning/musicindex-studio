# Studio v1 Task 213: One Row For Each File Of A Restored Release, And No Download Of A File That Is Gone

Decision: [ADR 0231](../adr/0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 12 to 16.

Status: Done. The human check is open.

## Problem

Findings 4 and 7 of the [desktop check of tasks 205 to 209](../reviews/2026-11-20-rc29-human-check.md#desktop-check-of-tasks-205-to-209).

- Finding 4: the tab **Files** of "new Feed" shows remote rows with names that end in " 2". Each one is a second row of a library file. `record_remote_files` removes a remote row only when its address leaves the feed. A local row knows only its planned address on the media host of the feed (`inventory::planned_path`). The rows are on a different host.
- Finding 7: the move plan offers to download these rows. `download::source_of` gives the `source_url` of a remote row when no connected host has a copy, and `check_download` in `crates/api/src/routes.rs` accepts it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Remote records | extend | crates/core/src/remote.rs | `record_remote_files` near line 56. The first loop removes rows. The set `known` holds the addresses of the local rows. |
| Planned path | reuse | crates/core/src/inventory.rs | `planned_path_on` near line 443 gives the planned address of a file on one connector. `copies` gives the copies. |
| Library rows | extend | crates/core/src/collection.rs | `remove_library_file` near line 3617. The setters near line 3380 to 3453. A setter for `source_url` of a local row is not there. Add one adjacent to them. |
| Download source | extend | crates/core/src/download.rs | `source_of` near line 126. |
| Move plan text | extend | crates/api/ui/releasemove.js | The download row near line 70 and `fromText` near line 44. |
| Core tests | extend | crates/core/tests/feed_retire.rs | `a_deleted_feed_makes_no_remote_rows_and_the_restore_gives_its_path_again`. |
| Download tests | extend | crates/core/tests/ | Find the test file of `download::source_of` and extend it. |

## Changes

1. Item 12: in `record_remote_files`, a remote row joins a local row of the same feed when one of these is true:
   - The address of the remote row is on a connected file host. It is also the planned address of the local row on that host (`planned_path_on` with that connector).
   - The address of the remote row is the address of a copy of the local row.

   Studio then puts the address on the local row as its `source_url` and removes the remote row. Count the removal in `RemoteReport`.
2. Item 13: do not join a remote row that a draft operation names. Use the same test that ADR 0179 item 4 uses for the SHA-256 join. Find it in the code.
3. Item 15: `source_of` gives `None` for a remote file with an address on a connected file host and no copy on that host. The move plan then counts the file as a file that does not move. It says "Not on {host}. Studio cannot download it." Find where the plan gives its rows for a file that does not move (ADR 0148 item 3), and reuse that path.
4. Item 16: a joined file moves from the library. Its plan row says "Not on a file host. Studio sends it from the library." The text "Studio downloads them first" stays only for a file with a working download source. `fromText` gives "The library" when the plan sends from the library.

## Checks

Mechanical:

- Core test: extend the test of `feed_retire.rs` above with a second connected host. A remote row at the planned address of a local row on the second host goes at the next `record_remote_files`. The local row gets that `source_url`. A second call adds no row.
- Core test: a remote row that a draft operation names stays.
- Core test: `source_of` gives `None` for an address on a connected host with no copy. It still gives `Source::Web` for an address that is on no connected host.
- Node test: the plan texts of changes 3 and 4.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual (main session):

- drive-ui at 1280 px: the tab **Files** of a release with remote rows of local files shows each file once after a start. The move plan of a release with a file that is gone.

Human:

- The tab **Files** of "new Feed" after the first start of the new binary shows each file once.

## Result

Changes:

- `crates/core/src/collection.rs` line 3481: the new setter `set_library_file_source_url`.
- `crates/core/src/remote.rs` line 65: `record_remote_files` makes the map `claims`. It holds the planned address of each local file on each connected host. It also holds the address of each copy. A remote row at a claimed address joins the local row.
- `crates/core/src/remote.rs` line 168: the new function `join_remote_rows` does the join. It gives the address to the local row and removes the remote row. It counts the removal in `RemoteReport.removed`.
- `join_remote_rows` skips a remote row that a draft operation names. It joins only a row with `local` false, no SHA-256 and no removal time.
- `crates/core/src/download.rs` line 129: the new function `gone_from_host`. Line 162: `source_of` gives `None` for an address on a connected file host with no copy. It still gives `Source::Web` for an address on no connected file host.
- `crates/api/src/routes.rs` line 5462: `release_move_plan` puts a file that is gone from its file host in `stays`. The reason is `gone_from_host`, with the host name. `check_download` does not run for that file.
- The plan JSON has the new field `from_library` (line 5545). Each entry of `stays` has the new field `host`. The new type `Stay` replaces the tuple.
- The new function `feed_host_of` keeps `release_move_plan` below the line limit of clippy.
- `crates/api/ui/releasemove.js`: `fromText` gives "The library" when `from_library` is more than 0 and no file host gives old files. `planRows` has the row "Not on a file host. Studio sends it from the library." `stayRows` gives "Not on {host}. Studio cannot download it."
- The new Rust tests are in the modules of the code that they test. The old tests of these functions are there too.
- `crates/core/src/remote.rs` lines 853, 874 and 900: a join at the planned address on a second host, with a second call that adds no row. A draft operation that keeps the remote row. A join at the address of a copy.
- `crates/core/src/download.rs` line 460: `source_of` and `gone_from_host`.
- `crates/api/src/tests.rs` line 16222 and after: two plan tests.
- `crates/api/ui-tests/releasemove.test.mjs`: two new tests for the plan texts.

Checks:

- `cargo test -p studio-core`: 878 unit tests pass, and each test file passes.
- `cargo test -p studio-api`: 457 unit tests pass, and each other test file passes.
- `tools/node --check` on a copy of `crates/api/ui/releasemove.js`: passes.
- `tools/node --test crates/api/ui-tests/*.test.mjs`: 929 pass, 0 fail.
- `bash tools/gate.sh`: "gate: each check passed".
- Visual (2026-11-20, main session): drive-ui at 1280 px on a copy of `.studio-dev/v-207/data`. In the copy, Host A has the address of the media files, and the host has no copy of them. **Move This Release…** to Host B gives "Not on Host A. Studio cannot download it." for each file and no download row. `$edges()` gives no finding.
- The row "Not on a file host. Studio sends it from the library." has no screenshot. The API test and the node test cover it.
- Main session review: the join calls `planned_path_on` for each local file on each connected host. `host_sha` can read a host file when the inventory has no SHA-256, as `inventory::copies` already does for each file. Thus a read failure of one host stops the remote records of the feed until the next call, as before.
- Human: the tab **Files** of "new Feed" after the first start of the new binary is open.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Read the `rust-dev` skill before you change Rust code.
- Do not commit and do not stage. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
- Do not change `crates/api/ui/editor.js`, `crates/api/ui/app.js` or `crates/api/ui/pickers.js`. Another agent changes them at the same time.
