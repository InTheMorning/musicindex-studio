# Studio v1 Task 148: The Track Count In The Releases List

Decision: none. This is Note 6 of the walk. [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md) items 11 and 14 own the kind and the type label, and they do not change.

Status: Complete on 2026-11-15, with the screenshot visual check. The visual gate for the operator is open.

## Result

- The delegate was `gpt-6-luna` with medium effort. The main session added the Rust test `the_feed_list_gives_the_track_count_of_a_release_and_none_for_a_publisher_feed`.
- Deviation: that test proves the count 1 for a hosted release with one track and the count 0 for a new release. It proves `null` for a publisher feed. The playlist test proves the count 2.
- A feed gets its medium from a read. A feed with no read has no medium, and it shows no count.
- Screenshots of the main session: the rows view and the tiles view at 1280 px and at 390 px, in the light theme and the dark theme. Each line shows the kind and the count. No text is cut, and `$overflow()` is empty. A playlist row shows "Playlist · 0 tracks".
- The fit check at 390 px reports that "Back" and a tile overlap. The cause is the check method: the list was scrolled, and the tile was below the bar that stays at the top.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md).

## Goal

Each row and each tile of the releases list shows the track count of the release after its kind, for example "Album · 7 tracks".

## Operator Choices

- Operator choice (2026-11-15): each row of the releases list shows the number of tracks.
- Recommendation of the planning model, open until the operator sees the screenshot: the text is "1 track" or "N tracks", after the type label and before "Hidden from apps". A playlist shows the count of its remote items with the same words. A publisher feed and a feed that Studio does not edit show no count.

## Code Locations

- `crates/api/src/routes.rs`, `list_feeds`, lines 1389 to 1485. Lines 1423 to 1433 calculate `track_count` for a feed with the medium `music` and do not send it.
- `crates/core/src/feed.rs`, lines 137 to 139: a playlist has the medium `musicL` and `remote_items`.
- `crates/api/ui/shell.js`, `releasesList`, lines 622 to 698, and `feedKind`, lines 51 to 56.
- `crates/api/ui/app.js`, `ReleasesPage`, lines 3436 to 3451: one markup for rows and tiles.
- `crates/api/src/tests.rs`, `the_feed_list_gives_the_draft_state_and_the_cover_image`.
- `crates/api/ui-tests/shell.test.mjs`, `the Releases list gives the draft label of each draft state`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed list reply | extend | crates/api/src/routes.rs | Adds the field `track_count` to each feed value. |
| Releases list model | extend | crates/api/ui/shell.js | `releasesList` gives each row the text of the count. |
| Releases row and tile | extend | crates/api/ui/app.js | The line `.sub` shows the text of the model. |
| Row styles | reuse | crates/api/ui/styles.css | `.releases-text .sub` already has the style of the line. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- `crates/core/` (the kind rule does not change)
- The left panel and the head of the feed page

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In `list_feeds`, set `value["track_count"]`: the count of `items` for the medium `music`, the count of `remote_items` for the medium `musicL`, and `null` for each other feed. Use the count that the handler already calculates for `music`.
2. In `shell.js`, add an exported pure function `trackCountText(count)`. It gives "1 track", "N tracks", or `null` when the count is not a number.
3. In `releasesList`, give each row the field `countText` from `trackCountText(feed.track_count)`.
4. In `ReleasesPage`, show the count in the first `.sub` line: type label, then ` · ` and the count, then the existing parts. Show no separator when a part is absent.

## Acceptance Criteria

Mechanical:

1. A Rust test in `crates/api/src/tests.rs`: the feed list gives `track_count` 2 for a release with two tracks, the count of the remote items for a playlist, and `null` for a publisher feed.
2. A Node test in `crates/api/ui-tests/shell.test.mjs`: `trackCountText` gives "1 track", "7 tracks", "0 tracks" and `null`. `releasesList` gives `countText` for a release and `null` for a row with no count.
3. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. Rows view and tiles view at 1280 px and at 390 px: the line shows the kind and the count, and no text is cut.
2. A playlist row shows the count of its tracks.

## Test Commands

- `cargo test -p studio-api the_feed_list`
- `tools/node --test crates/api/ui-tests/shell.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-148-track-count-in-the-releases-list.md` (this file), each section
- `crates/api/src/routes.rs`
- `crates/core/src/feed.rs`
- `crates/api/ui/shell.js`
- `crates/api/ui/app.js`
- `crates/api/src/tests.rs`
- `crates/api/ui-tests/shell.test.mjs`

Goal:
- Each row and each tile of the releases list shows the track count of the release after its kind, for example "Album · 7 tracks".

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- `crates/core/` (the kind rule does not change)
- The left panel and the head of the feed page

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `cargo test -p studio-api the_feed_list`
- `tools/node --test crates/api/ui-tests/shell.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
