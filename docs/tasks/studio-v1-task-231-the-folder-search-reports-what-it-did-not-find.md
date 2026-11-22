# Studio v1 Task 231: The Folder Search Reports What It Did Not Find

Decision: [ADR 0243](../adr/0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) items 13 to 15. Findings 7 and 8 of [the tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md).

Status: Done.

## Problem

A list of a connector goes 4 folders down and stops at 1000 files. A WordPress site keeps its feeds in `wp-content/uploads/2023/08/`, and these feeds stay read-only. After **Connect**, the File hosts page does not tell which feeds stay read-only, and does not tell when the list stopped at the limit.

Two small defects are also in this task. Two URLs break inside a word (musicindex ADR 0065 R-FIT-3). The form of a new hosting page shows the Type "Artist or label".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| List limits | extend | crates/core/src/connector.rs | `LIST_MAX_DEPTH` and `LIST_MAX_FILES` change. `FileList.truncated` exists. |
| Folder match | extend | crates/api/src/routes.rs | `match_connector` also gives the feeds that it did not find and `truncated`. |
| File hosts page | extend | crates/api/ui/app.js | `Hosts` shows the result of the match after **Connect**. `FolderFeeds` is the pattern for a list of feeds below it. |
| Publisher form | extend | crates/api/ui/app.js | `MakeFeedForm` hides the Type menu when the form starts with a role. |
| Fit exception | reuse | tools/fit-check.mjs | `data-fit-allow-break` is the exception of `wordBreaks`. `HostCard` folder values already use it. |

## Changes

1. Read ADR 0243 items 13 to 15, ADR 0019 and the functions above first.
2. Set `LIST_MAX_DEPTH` to 6 and `LIST_MAX_FILES` to 5000. Keep the depth check of each connector as it is. Update the doc comments, which cite ADR 0019 item 3, to also cite ADR 0243 item 13.
3. Tests in `crates/core/src/connector/local.rs` and `sftp.rs` (or `s3.rs` for the file limit, as today): a file in `a/b/c/d/e/feed.xml` (5 folders down) is in the list. A file one folder deeper is not. The file limit test uses the new limit.
4. `match_connector` gives two new fields:
   - `not_found`: each read-only feed of the collection that the match did not link, when the host of its public address is the host of `folder_address`. Each entry has `id`, `title` and `url`. With no `folder_address`, the list is empty.
   - `truncated`: true when the list of the connector stopped at the file limit.
5. Rust tests in `crates/api/src/tests.rs`: a folder with one of two read-only feeds of the same host gives the other feed in `not_found`. A read-only feed of a different host is not in `not_found`. A list at the file limit gives `truncated: true`.
6. On the File hosts page after **Connect**, below "Studio can now write {N} feeds…": when `not_found` has entries, show the section with the heading "{N} feeds stay read-only. Studio did not find them in this folder.", one row for each feed with its title and its URL, and the button **Add a File Host for Them**. The button opens the sheet **＋ Add a File Host**. When `truncated` is true, show "Studio stopped at 5000 files. Connect a smaller folder, for example the folder that holds your feeds." Put the texts in a pure function with a Node test.
7. Finding 7: give the attribute `data-fit-allow-break` to the public URL element in `HostCard` and to the URL element in `FeedAddress`. Give it also to the URL of each row of step 6.
8. Finding 8: when `MakeFeedForm` opens for a publisher page with a start role (Artist, Label or Hosting from a menu or an add row), it does not show the Type menu. Technical choice (main session): hide the field, and do not change the text "Artist or label". Add a Node test of `makeFields` or of the function that selects the fields.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0243 and an item. A test failure message names ADR 0243, the item and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Button labels use title case. Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The tests of steps 3, 5, 6 and 8 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots, in the dark theme at 1280 px and at 390 px):

- The section of step 6 after **Connect** of a folder that does not hold each feed of its host.
- The fit check gives no word break for the two URLs of step 7.
- The form of a new hosting page has no Type menu.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

## Corrections Of The Main Session

The main session found these defects in the screenshots and corrected them:

1. The heading said "1 feeds stay read-only". `folderMatchReport` now gives "1 feed stays read-only. Studio did not find it in this folder." for one feed. The Node test examines one feed, two feeds and no feed.
2. The title and the URL of a feed were on one line, with no space between them. Each row now has the class `feed-line`: the title is on the first line, and the URL is a `sub` line below it.
3. **Add a File Host for Them** opened the add form at the end of the page, out of view. The form now scrolls into view when it opens.
4. After **Connect**, the scroll put the first notice of the result below the sticky bar. The start of the result now has the class `match-result` with `scroll-margin-top: calc(var(--bar-h) + 12px)`, as the releases list has.

Operator choice: one feed gives the singular heading, as one page gives "Move 1 Artist" in task 230.

Known limit: the match of a folder with 5000 files took 4 s in the check, and the page then reloaded its data for more than 15 s. The notice of the file limit shows when the reload ends.
