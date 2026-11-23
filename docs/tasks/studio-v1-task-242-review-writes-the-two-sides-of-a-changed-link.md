# Studio v1 Task 242: Review Writes The Two Sides Of A Changed Link

Decision: [ADR 0251](../adr/0251-review-writes-the-two-sides-of-a-changed-link.md) items 1 to 9.

Status: Done.

## Problem

- An artist page adds an existing release in the section **Releases**. Review of the artist page writes only the artist page. The link in the draft of the release needs a second review.
- **Artist or label** on a written release names a written artist page. Review of the release writes only the release. The artist page needs a second review.
- `also_write_pages` in `crates/api/src/routes.rs` gives rows only for a release that was never written, and only for a page draft with any change.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link match | extend | crates/core/src/publisher.rs | `party`, `names`, `release_parties` and `publishers_linking` find a link. The new function of step 2 uses them. Do not copy them. |
| Written copy | reuse | crates/core/src/pipeline.rs | `written_copy` gives the written file of ADR 0234 item 7. |
| Draft model | reuse | crates/core/src/pipeline.rs | `draft_model` gives the draft of a feed. |
| Rows of the diff route | extend | crates/api/src/routes.rs | `also_write_pages` gives the entries of ADR 0251 item 9 from the core function. |
| Row model | extend | crates/api/ui/writeresult.js | `alsoRows` gives the texts of ADR 0251 item 5. |
| Write order | extend | crates/api/ui/writeresult.js | `writeInOrder` gets the feeds that go before the reviewed feed. |
| Review sheet | extend | crates/api/ui/editor.js | The Review sheet near the use of `alsoRows` shows the rows and calls `writeInOrder`. |

## Changes

1. Read ADR 0251, ADR 0234 items 3 to 7 and ADR 0231 items 7 to 9 first. Then read `link`, `unlink`, `publishers_linking`, `party` and `names` in `crates/core/src/publisher.rs`, `written_copy` and `draft_model` in `crates/core/src/pipeline.rs`, `also_write_pages` and `unlisted_pages` in `crates/api/src/routes.rs`, `crates/api/ui/writeresult.js`, and the Review sheet in `crates/api/ui/editor.js` (search `alsoRows`).
2. Core, items 2 and 3. In `crates/core/src/publisher.rs`, add:
    - `pub enum LinkSide { Page, Release }` and `pub enum LinkChange { Added, Removed }`, serialized in snake case.
    - `pub struct ChangedLink { pub feed: FeedId, pub side: LinkSide, pub change: LinkChange }`. `side` is the side of the other feed `feed`.
    - `pub fn changed_links(collection: &Collection, library: Option<&Library>, id: FeedId) -> Result<Vec<ChangedLink>, PublisherError>`.
    - When `id` is a page (`medium` `publisher`): for each release row (`music` or `musicL`) with a draft, compare two values. The first is "the `podcast:publisher` items of the release draft name the page". The second is the same test on `written_copy` of the release. No written copy gives `false`.
    - When `id` is a release: for each page row with a draft, compare "the `remote_items` of the page draft name the release" with the same test on `written_copy` of the page.
    - `true` in the draft and `false` in the written copy gives `Added`. `false` in the draft and `true` in the written copy gives `Removed`. Equal values give no entry.
    - Another medium of `id` gives an empty list. Keep the order of `collection.feeds()`.
3. API, item 9. Change `also_write_pages` in `crates/api/src/routes.rs`:
    - Remove the condition `last_version.is_some()` and the condition `draft_changed`. Call `changed_links`.
    - Each entry is `{ "id", "title", "side", "artist", "change" }`. `title` is the draft title or the row title, as now. `artist` is `true` when the row of a page has the role `Artist`. For a release entry, `artist` is `false`.
    - Rename the function to `linked_feeds_to_write` and its comment to ADR 0251 item 2. Keep the JSON field name `also_write`.
4. UI, items 4 to 6. In `crates/api/ui/writeresult.js`:
    - `alsoRows(also, diffs, off, reviewTitle)`: each row is `checked: true` and `locked: true`. The `note` is the text of ADR 0251 item 5, from `side`, `artist`, `change`, the title of the row and `reviewTitle`. A row with no loaded diff stays out, as now.
    - Add `export const ALSO_WRITE_HINT = 'A link is written on its two feeds.'`.
    - `writeInOrder({ before = [], release, pages, onReleaseWritten })`: write each `before` item first, in order, then `release`, then `pages`. A failed `before` write writes nothing after it. Its error gets `pageTitle`, as a page error does. Keep the current behavior when `before` is empty.
5. UI, item 7. In the Review sheet of `crates/api/ui/editor.js`:
    - The rows with `side` `release` go in `before`, and the rows with `side` `page` go in `pages`.
    - Replace the hint text "A new release is written with its pages." with `ALSO_WRITE_HINT`.
    - Replace the error action "Studio could not examine the artist page." with "Studio could not examine the linked feeds."
    - After the write, the feed list and each written feed load again, as after the write of a new release with its pages. Make sure that the pencil mark of each written feed goes.
6. Tests:
    - Rust, core: test `changed_links` for these cases:
        - A page and a written release whose draft adds the link give `Added`.
        - An `unlink` of a written link gives `Removed`.
        - A page draft with a change that is not the link gives no entry.
        - A release that was never written gives `Added`.
    - Rust, API (`crates/api/src/tests.rs`): the test near the line `assert_eq!(artist_diff["also_write"], json!([]));` asserts the old rule of ADR 0231 item 7. Change it: the artist diff now gives the new release with `side` `release` and `change` `added`. Add the entry fields `side` and `change` to the release diff assertion.
    - Rust, API: add a test with two written feeds: a link from the page gives the release in the page diff and the page in the release diff.
    - Node: `alsoRows` for each line of the table of ADR 0251 item 5, each row locked. `writeInOrder` with `before`: the order of the calls, and a failed `before` write that calls no other function and has `pageTitle`.
    - Each failure message names ADR 0251 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0251 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Link rules stay in the core (ADR 0065). The UI does not compare links. It shows the values of the API.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- Run `tools/gate.sh` at the end and report its result.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The core tests of step 6 pass for `Added`, `Removed`, an unrelated draft change and a release that was never written.
- The API tests of step 6 pass, from the page side and from the release side.
- The Node tests of `alsoRows` and `writeInOrder` pass.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots at 1100 px and 390 px:

- Review of an artist page with one linked written release shows the locked row and the text of ADR 0251 item 5.
- Review of a written release with its artist page shows the locked row.
- After one write, neither feed has a pencil mark.

## Review Fixes

The main session read the diff of the first run and made these changes:

1. The texts of `alsoRows` were swapped. A row with `side` `page` is in Review of a release, and a row with `side` `release` is in Review of a page. The function `linkNote` in `crates/api/ui/writeresult.js` now gives the texts of ADR 0251 item 5, and the Node tests assert them.
2. The API test with two written feeds of step 6 was missing. The main session added `a_changed_link_of_two_written_feeds_gives_a_row_on_each_side`.
3. The Review sheet used `pageFeeds` after the diff removed it from the result of `writeInOrder`. Each write from Review then ended with the error "pageFeeds is not defined". The sheet now gives the IDs of `beforeFeeds` and `pageFeeds` to `writeResult`. A walk with a debug build found this error. No test found it.
