# Studio v1 Task 216: A Found Feed Is Added Or Copied, And A Removed New Release Leaves No Remote Item

Decision: [ADR 0233](../adr/0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) items 6 to 9, 11, 13 to 15.

Status: Done.

## Problem

The sheet **Find on Music Index** fills only five start values of a new release (ADR 0230 items 14 and 15). ADR 0233 asks for three things. The first is a copy of the full found feed. The second is a mark for a found feed on a connected file host or in the collection. The third is the releases of an artist row. The remove of an unwritten release leaves its remote item in the draft of its artist page.

Task 217 gives the UI. This task gives the core and the API.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed reader | reuse | crates/core/src/discover.rs | `read_feed` near line 280 reads a feed file with its items, enclosures and value blocks. |
| Make of a feed | extend | crates/core/src/pipeline.rs | `new_feed_with` and `FirstValues` near line 1075. Add the copied channel values and items. |
| Make route | extend | crates/api/src/routes.rs | `NewAlbumFeed` near line 1576 and `new_album_feed` near line 1603. |
| Import route | reuse | crates/api/src/routes.rs | `import_feeds` near line 2677 adds a feed that is yours (item 6). The UI calls it. |
| Host of an address | reuse | crates/core/src/outside.rs | `host_for_address` near line 260 gives the mark "Yours". |
| Music Index client | extend | crates/core/src/musicindex.rs | `MusicIndex::feed` near line 794 and `publisher_values` near line 1194. Add the remote items of a publisher feed (Stophammer `GET /v1/feeds/{guid}?include=remote_items`, `src/query.rs` near line 2827). |
| Search route | extend | crates/api/src/people.rs | `search_index` near line 776 and `musicindex_values` near line 892. |
| Remove of a feed | extend | crates/core/src/people.rs | `remove_feed` near line 3814 and its undo snapshot. |
| Link of a new release | reuse | crates/api/src/routes.rs | `link_new_release_pages` near line 1779 names the changes that item 15 removes. |

## Changes

1. Items 6 and 7: the search route gives two marks on each feed result: `yours` (the feed address is on a connected file host, from `host_for_address`) and `in_studio_id` (the id of the feed in the collection with the same `podcast:guid` or address). Give the feed address of each result when Music Index gives it.
2. Item 13: a new route `GET /api/v1/musicindex/releases/{guid}` gives the remote items of the publisher feed `guid` from Music Index, as feed results with the marks of change 1. Order: the order of the remote items.
3. Items 8, 9 and 11: `POST /api/v1/feeds/new` takes the new optional field `copy_from`, a feed address. The route reads the file with `discover::read_feed`. The first draft of the new feed gets from that file: the author, the description, the artwork, the website, the channel `podcast:value`, and each item with its title, enclosure, duration, artwork, description and item `podcast:value`. A publisher feed gets the `podcast:remoteItem` list. The form fields title and author win over the copied values. The new feed gets a new `podcast:guid` from its address, its own self link and a new guid for each item. The enclosure addresses do not change. Studio copies no audio file.
4. Item 14: when the read fails, the route makes the feed with the values of `description`, `image` and `website` as before, and gives `copy_failed: true` in its answer. It does not fail the make.
5. Change 3 also gives `copied: { tracks: n, recipients: m }` in the answer of the route. Add a route `GET /api/v1/feeds/copy-preview?url=...` that gives these two counts and the author before the make, for the line of item 8.
6. Item 15: `remove_feed` of a feed that was never written (no hosted file, no written version) also removes its remote item from the draft of each publisher feed that `link_new_release_pages` or `add_link` changed. When a draft then differs from the written file only in `lastBuildDate`, the draft goes. The undo snapshot keeps the earlier drafts of these feeds, and **Undo** puts them back.

## Checks

Mechanical:

- API test: the search gives `yours: true` for a result on a connected file host, and `in_studio_id` for a result in the collection.
- API test: the releases route gives the remote items of a publisher feed from a test Music Index.
- API test: a make with `copy_from` of a local test feed file with two items and two value recipients gives a first draft with these items, their enclosure addresses and the recipients. The `podcast:guid` and each item guid differ from the file.
- API test: a make with `copy_from` of an address that cannot be read gives `copy_failed: true` and a feed with the start values.
- Core or API test: make a release below an artist page, then remove the release. The draft of the artist page has no remote item of the release, and with no other change it has no draft. **Undo** gives back the release and the remote item.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass.

Visual: none. Task 217 gives the UI.

Human: none.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above, `crates/api/src/lib.rs` for the routes, the test files, and this task file. Do not change `crates/api/ui/`.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/core/src/pipeline.rs:1093` copies the selected feed values and items. It gives each copied item a new GUID.
- `crates/core/src/musicindex.rs:1235` reads remote items from Music Index.
- `crates/core/src/people.rs:3815` removes links to an unwritten release and saves linked drafts for Undo.
- `crates/api/src/routes.rs:1645` adds feed copy preview and copy status.
- `crates/api/src/people.rs:778` adds search marks and release lookup.
- `crates/api/src/lib.rs:385` registers the new routes.
- `crates/api/src/tests.rs:1447` checks removal and Undo.
- `crates/api/src/tests.rs:2588` checks feed copy.
- `crates/api/src/tests.rs:8120` checks the release lookup.
- `crates/api/src/tests.rs:8162` checks search marks.
- `crates/core/src/musicindex.rs:1626` checks remote item reads.
- `cargo test -p studio-core`: passed.
- `cargo test -p studio-api`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash tools/gate.sh`: passed.
- `git diff --check`: passed.

Main session changes after the review:

- The make route and the copy preview read the found feed file with no lock of the workspace (`copy_unlocked` and `read_feed_copy` in `crates/api/src/routes.rs`). Before, a slow file host stopped each other request during the read.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass after this change.
