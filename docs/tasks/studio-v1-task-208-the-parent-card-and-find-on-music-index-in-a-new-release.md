# Studio v1 Task 208: The Parent Card And Find On Music Index In A New Release

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 8 to 16 and item 33.

Status: Done. Two points are open for the operator. Refer to the Result.

## Problem

Findings 3 and 4 of the [check after rc.29](../reviews/2026-11-20-rc29-human-check.md). The form "Make a new release" does not show the parent page. "Start from" copies a release of the collection (ADR 0077). The operator wants "Find on Music Index" there.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Release form | extend | crates/api/ui/app.js | The form near lines 1440 to 1530. `start.publisher_id` is sent near line 1488. |
| Sheet title | extend | crates/api/ui/addfeeds.js | `makeTitle` near line 70. `startChoices` near line 94 goes. |
| Parent art | reuse | crates/api/ui/app.js | `FeedArt` near line 385. `LevelFeedTile` near line 3605 shows a publisher with art. |
| Parent list | reuse | crates/api/ui/editor.js | The artist picker near line 4183 sets `start.publisher_id`. Reuse its list for **Change…**. |
| Music Index pick | reuse | crates/api/ui/pickers.js | `FindOneSheet` near line 286 with a `choices` function. `editor.js` near line 292 shows a caller for a feed. The search route is `/musicindex/search` (`crates/api/src/people.rs:776`). |
| Picker row | reuse | crates/api/ui/pickers.js | `PickerLayout` near line 222 gives the row "Find on Music Index". |
| New release route | extend | crates/api/src/routes.rs | `NewAlbumFeed` near line 1572 and `new_album_feed` near line 1593. Remove `from` and the copy branch near lines 1659 to 1679. Add the optional values of item 15. |
| Core copy | extend | crates/core/src/pipeline.rs | `copy_feed` near line 1104 goes with its tests (`crates/core/tests/pipeline.rs` near 5335 to 5449, `crates/core/tests/placeholder_guid.rs` near 334, `crates/api/src/tests.rs` near 10799). |

## Changes

1. Items 8 to 10: the sheet title is "New release for {parent}" or "New release". The first row is a parent card with the art, the name and the type of the parent, and **Change…**. With no parent, the card says "No artist page" and has **Select…**, and **Make the Release** stays off.
2. Item 11: the row "Find on Music Index" with "Fill the form from a release." follows the card. It opens `FindOneSheet` with a `choices` function for a release.
3. Item 12: the fields are "Title", "Kind", "Artist name", "File host" and "Feed file". "Artist name" starts with the name of the parent.
4. Item 13: remove "Start from", `startChoices`, the route field `from`, `copy_feed` and their tests.
5. Items 14 and 15: a pick fills "Title", "Artist name", and "Kind" when the found feed gives a track count. The route takes the description, the artwork address and the website of the pick. The core puts them in the first draft of the new release. When the search result does not give a value, report it and leave it out.
6. Move `docs/adr/0077-a-new-album-can-start-from-an-album.md` to `docs/archive/adr/` with `git mv`. Update the links to it in `docs/README.md`, `docs/adr/0212-the-add-pages.md`, `docs/adr/0213-a-release-starts-on-the-page-of-its-artist.md`, `docs/adr/0127-a-changed-feed-gets-a-new-build-date.md` and `docs/adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md`. Move its line in `docs/README.md` to the section "Archive" with "Replaced by ADR 0230". Remove the rule about the copy from `AGENTS.md` near line 177.

## Checks

Mechanical:

- Node test: the sheet title with and without a parent. The form cannot make a release with no parent.
- Node test: the `choices` function of a release gives "Title", "Artist name" and "Kind".
- API test: a new release with the values of item 15 has them in its first draft. A request with `from` is refused or ignored. Report which.
- No code calls `copy_feed`. `bash tools/gate.sh` passes.

Visual:

- drive-ui screenshots at 1280 px and 390 px of the form with a parent, with no parent, and after a pick. Run `$edges()` on each.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

Changes 1 to 6 are done. Change 6 moves ADR 0077 to the archive and updates the links to it, the document index and `AGENTS.md`.

- The release form shows the parent card first. With no parent, the card says "No artist page" and has **Select…**. **Make the Release** stays off.
- The sheet title is "New release for {parent}" or "New release".
- The row "Find on Music Index" follows the card. It uses `FindOneSheet` and `PickerLayout`, which has the new options `lead` and `musicindexFirst`.
- The fields are "Title", "Kind", "Artist name", "File host" and "Feed file". A parent change sets "Artist name" until the user types in it.
- "Start from", `startChoices`, the route field `from`, `copy_feed` and their tests are removed.
- The route takes the optional `description`, `image` and `website`. `pipeline::new_feed_with` puts them in the first draft as `description`, `itunes:image` and `link`.
- `ReleaseArtistPicker` in `editor.js` is removed. **Select…** and **Change…** use the new `ParentSheet` in `app.js`.

Answers to the open points:

- A request with `from` is ignored, not refused. The route struct has no `deny_unknown_fields`. An API test checks this.
- The search result of `/musicindex/search` gives `title`, `artist` and `image_url`. The route `/musicindex/values/{guid}` adds `website` and `feed_url`.
- Music Index gives no description and no track count in these answers. A pick leaves out "Kind" and the description. The code uses them when they are present.
- The rule that a track count gives a kind (1 to 3 single, 4 to 6 EP, more album) is a choice of this task. ADR 0230 item 14 does not name it. It needs a review.
- ADR 0230 item 9 says the list has artist pages and label pages. The route accepts only an artist page (ADR 0213 item 7). The list has artist pages only. It needs a decision.

Checks:

- Mechanical: the Node tests (920 pass), the API tests for the first-draft values and for `from`, and `bash tools/gate.sh` pass.
- Visual (2026-11-20): drive-ui screenshots at 1280 px and 390 px on a copy of the real data. With no parent, the form shows "New release" and the card "No artist page".
- Visual, continued: **Select…** opens the sheet with HeyCitizen. After the selection, the title is "New release for HeyCitizen", the card shows HeyCitizen and Artist, and "Artist name" is HeyCitizen. **Find on Music Index** opens the search sheet. `$edges()` gives no finding.

Open:

- The sandbox has no network. Thus the search results and a pick from Music Index are not checked. This is a human check.
- The operator must decide two points: the track-count rule for "Kind", and the artist-only list of item 9 (refer to the open points above). Until then, the code keeps the artist-only list and the track-count rule.
