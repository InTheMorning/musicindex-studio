# Studio v1 Task 244: The New Page Of A Linked Release, And A New Artist Choice

Decision: [ADR 0251](../adr/0251-review-writes-the-two-sides-of-a-changed-link.md) items 9, 10, 12 and 13.

Status: Done.

## Problem

- After **Link and Continue** in the Stop panel of task 243, the draft of the selected page C has the new link. Review of the page A writes the release B but not C. C stays "Saved, not written".
- The menu of the Stop panel and the menu of the refusal panel of ADR 0218 have no choice that makes an artist page. The refusal panel shows **Make an Artist Page…** only when the collection has no page.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Changed links of a feed | reuse | crates/core/src/publisher.rs | `changed_links` gives the pages whose link to B changed. Call it for each release row. |
| Rows of the diff route | extend | crates/api/src/routes.rs | `linked_feeds_to_write` adds the pages of item 12 and the field `via`. |
| Row texts | extend | crates/api/ui/writeresult.js | `alsoRows` and `linkNote` give the text of item 12. |
| Panel models | extend | crates/api/ui/feedcheck.js | `pageRefusalView` and `linkedRefusalView` get the choice "New artist page…". |
| Artist page form | reuse | crates/api/ui/editor.js | `DraftToolbar` opens `AddFeedRowSheet` with `ARTIST_PAGE_FORM`, and `madeArtist` links the new page. Extend `madeArtist` with a target release. |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` shows the two panels and the rows. |

## Changes

1. Read ADR 0251 items 4, 5, 7, 9, 10, 12 and 13, and ADR 0218 items 3 and 6. Then read `linked_feeds_to_write` in `crates/api/src/routes.rs`, `changed_links` in `crates/core/src/publisher.rs`, `alsoRows` and `linkNote` in `crates/api/ui/writeresult.js`, `pageRefusalView` and `linkedRefusalView` in `crates/api/ui/feedcheck.js`, and `ReviewSheet` and `DraftToolbar` in `crates/api/ui/editor.js`.
2. API, items 9 and 12. In `linked_feeds_to_write`, each entry gets `"via": null`. When A is a page, for each entry with `side` `release` (B), call `publisher::changed_links` for B. For each result with `side` `page` whose feed is not A and is not in the list yet, add an entry with the fields of item 9 and `"via": B.0`. Put the new entries after the entries of item 2.
3. UI rows, item 12. In `writeresult.js`, a row whose `via` is not null uses the title of the entry with `id` `via` in place of the reviewed title. Its text is "Adds the link to {B} on {C}." or "Removes the link to {B} from {C}.", with the title of C and not "the artist page". The row is locked and on, as each row.
4. UI panels, items 10 and 13. In `feedcheck.js`:
    - Export `NEW_PAGE_CHOICE = 'new'`.
    - `pageRefusalView(reason, selected)`: `menu` is always `true`, `button` is always "Link and Continue", `linkDisabled` is `!selected`, and it gives `newChoice: 'New artist page…'`. Remove the parameter `candidates`.
    - `linkedRefusalView(row)`: `menu` is always `true`, and it gives `newChoice: 'New artist page…'`. Remove the parameter `candidates`.
5. UI sheet, items 10 and 13. In `editor.js`:
    - Each of the two menus has the last option `[NEW_PAGE_CHOICE, view.newChoice]` after the pages.
    - In the refusal panel, **Link and Continue** with `NEW_PAGE_CHOICE` selected closes Review and calls `onMakeArtist()`, as the old **Make an Artist Page…** did. With a page selected, it calls `linkAndContinue` as now.
    - In a Stop panel, **Link and Continue** with `NEW_PAGE_CHOICE` selected closes Review and calls `onMakeArtist({ release: row.id })`.
    - In `DraftToolbar`, `making` holds `null` or the request. With a `release`, the form starts with an empty name, and `madeArtist` calls `linkPublisherPage(page.id, release)` and does not call `setLinkedPage`. Without a `release`, the behavior stays as now. After the page is made, Review of A opens again, and it loads the diff and the diffs of the rows again.
6. Tests:
    - Rust, API: extend `a_linked_release_refusal_is_in_the_page_review_until_the_link_returns`, or add a test next to it. After the unlink, make a second artist page C, write it, and link it to B with `POST /api/v1/feeds/{C}/links`. The diff of A then gives B with `refusal` `null` and `via` `null`, and C with `side` `page` and `via` the id of B.
    - Rust, API: each test that compares a full `also_write` entry gets `"via": null`.
    - Node, `writeresult.test.mjs`: `alsoRows` gives the row of item 12 locked, with the added and the removed text.
    - Node, `feedcheck.test.mjs`: `pageRefusalView` and `linkedRefusalView` give `menu` `true` and `newChoice` "New artist page…". `pageRefusalView` gives `linkDisabled` `true` with no selection and `false` with a selection.
    - Each failure message names ADR 0251 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0251 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- Run `tools/gate.sh` at the end and report its result.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The API test of step 6 gives the entry of C with `via` the id of B.
- Each `also_write` entry has the field `via`.
- The Node tests of step 6 pass.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots at 1100 px and 390 px:

- The Stop panel menu ends with "New artist page…". The refusal panel of a release Review has the menu and "New artist page…", also with no other page.
- After **Link and Continue** with HeyCitizen, Review of Big Sky Choir shows the locked row "Also write HeyCitizen". One write writes the three feeds.
- "New artist page…" opens the form. After the page is made, Review of A shows the new page as a row.
