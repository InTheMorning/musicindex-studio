# Studio v1 Task 223: A Found Release Gives The Page Of Each Role

Decision: [ADR 0238](../adr/0238-a-found-release-gives-the-page-of-each-role.md) items 1 to 9.

Status: Done. The main session checked the three visual criteria with screenshots on 2026-11-21, with the release "Wavlake Exclusive" from Music Index.

## Problem

In the Artist, Label and Hosting forms, a selected release result makes a release, not a page. `AddFindSheet` in `crates/api/ui/app.js` imports a "Yours" release, opens an "In Studio" release, and copies a different release with its tracks. ADR 0238 gives the new behavior.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Add find sheet | extend | crates/api/ui/app.js | `AddFindSheet` gets the found feed branch of a page form. |
| Make feed form | extend | crates/api/ui/app.js | `MakeFeedForm` sends `from_found` and shows the lines of ADR 0238 items 5 and 7. |
| Add form model | extend | crates/api/ui/addfeeds.js | A new pure function gives the start values and the lines for each role. Node tests reach it. |
| Copy preview route | extend | crates/api/src/routes.rs | `feed_copy_preview` also gives the values of ADR 0238 item 4 and the pages of `podcast:publisher`. |
| New feed route | extend | crates/api/src/routes.rs | `new_album_feed` reads `from_found`, as it reads `copy_from`. |
| First values | extend | crates/core/src/pipeline.rs | `FirstValues` gets the start values of a found feed and the remote item of ADR 0238 item 6. |
| Publisher link | reuse | crates/core/src/publisher.rs | `publisher::link` links two drafts. Its `upsert` adds no second link. |
| Search results list | reuse | crates/api/ui/pickers.js | `SearchResults` shows the rows. No change. |

## Changes

1. Read ADR 0238 and ADR 0233 first.
2. `crates/api/src/routes.rs`, `feed_copy_preview`: add the query flag `found=true`. With the flag, the route reads the feed file also when the address is on a connected host. It gives `author`, `link`, `guid`, `medium`, the counts, and `pages`. `pages` is each `podcast:remoteItem` of `podcast:publisher`, with its GUID, its address, its title and its `rel`. Without the flag, the route does not change.
3. `crates/api/ui/addfeeds.js`: add a pure function `foundPageValues(role, found, row)`. It gives `title`, `website`, `copiesRecipients`, `line` and `emptyLine` from the table of ADR 0238 item 4 and the lines of item 5. Add a pure function that keeps the pages of `pages` with the role of the form or with no `rel`. Give both functions Node tests in `crates/api/ui-tests/`.
4. `crates/api/ui/app.js`, `AddFindSheet`: in a page form (`type === 'publisher'`), a feed result (`isFeed === true`) skips the "In Studio" and "Yours" branches. Get the address from the row or from `/musicindex/values/{guid}`, as `copy` does now. Then read `/feeds/copy-preview?found=true`.
5. When the kept pages of step 3 are not empty, the sheet shows a list with **‹ Results**, one row for each page, and the last row "Make a new {role} page from {title}". A page row uses the existing `copy` path with the page address.
6. `onPick` gets a new shape for a found feed: `{ found: { url, guid, title }, preview, values }`. `MakeFeedForm` fills the name and the website from step 3. It keeps `copyFrom` empty and sends `from_found: url` in `POST /feeds/new`. It shows the lines of ADR 0238 items 5 and 7.
7. `crates/api/src/routes.rs`: `NewAlbumFeed` gets `from_found: Option<String>`. Refuse it with `invalid_feed` when the medium is not `publisher`, or when `copy_from` is also present. Read the feed with no lock, as `copy_unlocked` does.
8. `crates/core/src/pipeline.rs`: a found feed gives the first draft only the values of ADR 0238 item 4 for the role. The Artist role gets `link` and the channel `values`. The Hosting role gets only the name. No role gets an image, a description or items.
9. In the route: when a collection feed has the GUID of the found feed, call `publisher::link(new page, that feed)` after `finish_new_feed`. Otherwise, add one `podcast:remoteItem` to the draft of the new page, with the GUID, the address, the title and the medium of the found feed. When the read failed and the GUID and the address are known, add the remote item also.
10. The Review button "Make an Artist Page…" opens the same form (`editor.js`, `madeArtist`). It links the release after the make. Confirm that the second link adds no second entry.
11. Rust tests in `crates/api/src/tests.rs`, near the tests of `copy-preview` (about line 2884 and line 8468):
    - Artist: a make with `from_found` of a feed that is not in the collection gives a remote item, the link and the recipients, and no items.
    - Hosting: the draft has no recipients, no website and no items.
    - A make with `from_found` of a collection release links both drafts.
    - A `from_found` with medium `music` is refused.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0238 and an item.
- Follow the `rust-dev` skill and the lints of each crate.
- Button labels use title case. The UI text does not use the word "feed".
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- Node tests: `foundPageValues` gives the values of the ADR 0238 item 4 table for each of the three roles. The page filter keeps the pages of the role and the pages with no `rel`.
- The Rust tests of step 11 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- In the Artist form, a release selection shows the line of ADR 0238 item 5, and the name is the artist.
- In the Hosting form, the name is empty, and the form asks for the person who hosts the release.
- A release with a page in `podcast:publisher` or a `publisher_feed_guid` on Music Index shows the list of ADR 0238 item 2.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
