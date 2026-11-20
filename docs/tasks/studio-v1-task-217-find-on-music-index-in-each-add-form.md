# Studio v1 Task 217: Find On Music Index In Each Add Form

Decision: [ADR 0233](../adr/0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) items 1 to 14.

Status: Done.

## Problem

The menu **＋ Add** ends with **Add One You Have…**, and only the form of a release has **Find on Music Index**. A selection fills five start values, and an artist row gives "Select a release. This row is not a release." ADR 0233 puts the search in each form. A selection adds or copies the found feed. The folder and backup imports move into the sheet.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Add menu model | extend | crates/api/ui/addfeeds.js | `addMenuEntries` and the rows of the add page near lines 10 to 36. |
| Add menu and forms | extend | crates/api/ui/app.js | `AddMenu` near line 3684, `AddFeedSheet` near line 3084, `MakeFeedForm`, the release search near line 1553, `AddFeeds` near line 3117 and the view `add-feed` near line 4212. |
| Search sheet | extend | crates/api/ui/pickers.js | `FindOneSheet` and `FindOneForm` near lines 275 to 312. |
| Choices of a selection | extend | crates/api/ui/addfeeds.js | `releaseFindChoices` near line 122. |
| Import flows | reuse | crates/api/ui/app.js | `FindFeeds`, `FoundFeeds`, `AddFeedForm` and `MspImport`. The sheet rows of item 3 open them. |
| Left panel and level models | extend | crates/api/ui/shell.js | The `add-feed` view near lines 198, 317 and 873. |
| UI tests | extend | crates/api/ui-tests/addfeeds.test.mjs | The tests near lines 21 to 103. |

## Changes

1. Item 1: the menu has Release, Artist, Label, Show and Playlist only. Remove the view `add-feed`, the page `AddFeeds` and each link to them. Where Studio opened that page with no file host (`app.js` near line 4182), the form shows the step to connect a file host (item 5).
2. Item 2: each form starts with the row "Find on Music Index", after the parent card of a release. The search filters the results to the kind of the form. A result with no kind shows in each form.
3. Item 3: below the results, the rows **Add from a File Host Folder…** (only with a file host) and **Add from MSP 2.0 Backups…** open the earlier flows.
4. Item 4: a feed address in the search field reads that feed, as `FindFeeds` did.
5. Items 6 and 7: a result with `yours` shows the mark "Yours". A selection calls the import route and opens the added feed. A result with `in_studio_id` shows "In Studio" and opens that feed.
6. Items 8 and 12: a selection of a different result calls the copy preview of task 216. The form shows "Copies {n} tracks and {m} payment recipients from {title}." When the author differs from the name of the parent, it also shows the warning of item 12. **Make the Release** sends `copy_from`.
7. Item 13: in the form of a release, a selection of an artist row shows the releases of that artist (route of task 216) with the back row **‹ Results**.
8. Item 14: an answer with `copy_failed` shows "Studio could not read the feed file of {title}. No tracks and no payment recipients are copied."
9. Remove the message "Select a release. This row is not a release." and each test that names it.

## Checks

Mechanical:

- UI tests of the checks of ADR 0233 for the menu, the kinds, the sheet rows, the marks and the artist row.
- `node --test crates/api/ui-tests/`, a syntax check of each changed `ui/*.js` file as a `.mjs` copy, and `bash tools/gate.sh` pass.

Visual, by the main session with the screenshot harness: the visual checks of ADR 0233.

Human: none in this task.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments, UI text and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above, `crates/api/ui/style.css` if necessary, and this task file. Do not change Rust files.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

The ChatGPT run stopped at its time limit with no Result section. The main session read its change, corrected it and completed the task.

- `crates/api/ui/addfeeds.js`: `addMenuEntries` has the five values of item 1. `addSearchRows` and `addRowIsArtist` give the role rows of each form (item 2) and the artist row of item 13. `addSheetRows` gives the rows of item 3. `addResultMark` gives the marks of items 6 and 7. `addChoices`, `ADD_PAGE_TITLE`, `releaseFindChoices` and `kindOfTrackCount` are removed.
- `crates/api/ui/app.js`: `AddFindSheet` uses `SearchResults` of `pickers.js` for the results. It adds a typed address or a feed that is yours (items 4 and 6), opens a feed that is in Studio (item 7), shows the releases of an artist row with **‹ Results** (item 13), and copies each other feed (items 8 and 14). `MakeFeedForm` shows the copy line, the payments warning and the read failure line, and sends `copy_from`. A form with no file host shows **Connect a File Host** (item 5). The page `AddFeeds` and the view `add-feed` are removed.
- `crates/api/ui/pickers.js`: `SearchResults` takes `markOf`. The unused parameter `noun` of `FindOneSheet` is removed.
- `crates/api/ui/shell.js`: the view `add-feed` is removed from the left panel and the narrow stack.
- `crates/api/src/routes.rs`: the copy preview gives `yours: true` for an address on a connected file host, and reads no file. The search of Music Index gives no feed address (Stophammer `/v1/search`), so the list cannot show the mark "Yours". The selection finds it.
- `crates/core/src/musicindex.rs`: defect of task 216. `feed_remote_items` read `feed_guid`, `feed_url` and `title`. Stophammer gives `remote_feed_guid`, `remote_feed_url`, `remote_feed_title`, `remote_feed_image_url` and `remote_release_artist`. The test fixtures now use the Stophammer names, from a real answer of `api.musicindex.org`.
- `docs/user-guide/add-feeds.md` and `docs/user-guide/find-your-way.md` describe the sheet.

Checks:

- `tools/node --test crates/api/ui-tests/*.test.mjs`: 942 pass. A syntax check of a `.mjs` copy of `app.js`, `addfeeds.js` and `pickers.js` passes.
- `cargo test -p studio-core`, `cargo test -p studio-api` (468 tests), `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass.
- Screenshots with the real Music Index, `.studio-dev/v-207/out38` to `out41`, at 1100 px and 390 px: the menu has five values. The release sheet shows the artist HeyCitizen and the feeds. The artist row shows its five releases with **‹ Results**. A selection fills the form with "Copies 4 tracks and 3 payment recipients from HeyCitizen's Test Album." The Artist form starts with the find row. The folder row opens the folder flow. `$edges()` gives no finding.

Open:

- The screenshots do not show the mark "Yours" or the payments warning. The harness has no artist page and no file host at a Music Index address.
- Two releases of HeyCitizen show only their guid, because Music Index gives no title for them.
- The sheet title stays "New release" while the folder flow or the MSP flow is open.
- `feedWords` in `crates/api/ui/people.js` keeps its `noun` parameter for ADR 0231 item 3, but no sheet now passes "Release".
