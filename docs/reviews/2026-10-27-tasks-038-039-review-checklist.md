# Tasks 038 And 039 Review Checklist, 2026-10-27

This record holds the orchestration of [task 038](../tasks/studio-v1-task-038-cover-and-text-space.md) and [task 039](../tasks/studio-v1-task-039-file-picker.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md) and [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) own the rules.

## Sequence

The operator asked for the orchestration on 2026-10-27. The packets run one at a time, because packets 039b and 039c both change `crates/api/ui/editor.js`:

1. Task 038.
2. Task 039a.
3. Task 039b.
4. Task 039c.

A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end.

## Packets

| Packet | Status | Review |
|---|---|---|
| 038 | Implemented | Accepted with two follow-up fixes. The four checks are Green: 898 Rust tests and 137 Node tests pass. |
| 039a | Implemented | Accepted after one escalation and one follow-up (`media_type` in both lists). |
| 039b | Implemented | Accepted. The four checks are Green: 908 Rust tests and 151 Node tests pass. |
| 039c | Implemented | Accepted. The four checks are Green: 916 Rust tests and 161 Node tests pass. |

## Decisions

### Task 038

- The coding model also changed `crates/api/ui/app.js` (the cover of the feed header) and `styles.css` (the rule `.field-note`). The packet did not name these files. Accepted: without the change, the header shows no cover for a feed in the DeMu form.
- `Feed::adopt` does not change for `RssImage`. Accepted: the writer changes the element in place, as for `podcast:locked` and `podcast:license`, so the source bytes stay.
- The release list and the outline read only `itunes:image`. The planning model asked for a fix in the same packet: `list_feeds` also gives `rss_image_url`, and `shell.js` uses `coverOf` of `cover.js`. The cover rule stays in one module.
- The host operations of ADR 0036 item 7 did not change the RSS image. The planning model asked for a fix in the same packet: a reference changes `rss_image.url`, and a dereference removes the full `rss_image`.
- ADR 0060 item 1 now says that the RSS image `url` is an address field and that the host operations change it. ADR 0023 has an amendment for the address field.

### Task 039a

- The coding model stopped on the escalation of the packet. No code gives a draft address its library row at a draft save. ADR 0038 item 3 records the rows at an add, a read, a write and the start.
- Decision of the planning model, in the limits of ADR 0062: the row of a host address comes after the next write, through the code that records the addresses of a written feed. The diff makes no upload for an address with no SHA-256, so the row is not necessary before the write. A host file that the user selects and replaces before a write was never in the feed, so it needs no deletion.
- ADR 0062 item 10, its mechanical check and packet 039a now give this rule. ADR 0038 does not change.
- The release list gave no media type and no duration. The library records no duration. Decision of the planning model: both picker lists give `media_type` from the existing `remote::media_type_of_name`. The player reads the duration from the metadata for each selection from "This release" or "Hosts", not only for a file that only a host holds. ADR 0062 items 15 and 16 and packets 039a to 039c now give this rule.
- Accepted as they are: the `name` of a release file is its path in the release folder. A feed with no draft and no release folder counts as a feed that uses no address. Such a feed gets its folder at its first read.

### Task 039b

- The replacement of ADR 0040 also served the recording of a live item. The coding model gave that recording the button "Choose…" and removed "Replace the recording" and "Add the recording". Accepted: without it, a live item cannot get its recording. ADR 0062 item 1 now names the recording of a live item.
- An upload or a drop to an audio field accepts a WAV file, because ADR 0033 converts it. The two picker lists do not offer WAV. Accepted. ADR 0062 item 3 now says this.
- `Field` in `app.js` got the optional property `rootProps`, so that a drop target stays the `.field` element. Accepted.
- A selected file with no size gives an empty size, and the address check of ADR 0023 item 5 marks it. Accepted.
- Open concern: the "Hosts" source requests one small image from `/images` for each listed image, to a maximum of 1000. The operator examines the speed in the visual check.
- The user guide pages `tracks.md`, `publisher-feeds.md` and `live-shows.md` still name the removed buttons. One documentation step after packet 039c changes the user guide for all of task 039.

### Task 039c

- The coding model found a defect of tasks 038 and 039b: `cover.js` and `picker.js` were not in `FILES` of `crates/api/src/ui.rs`, so the editor did not load. It added the three new scripts and the guard `each_ui_script_of_the_ui_folder_is_served`. Accepted.
- `futures-util` is a new direct dependency of `studio-api`, for `Body::from_stream`. Accepted: `Cargo.lock` holds it through axum, so no crate is new to the build.
- A ticket holds no user ID. Each user has a workspace with its own tickets, and the server sends `/media/audio` to the workspace of the session user. A ticket of a different user is thus unknown. A server test confirms it. Accepted.
- The guard of ADR 0057 acts at the audio route, also after a redirect, not when Studio makes the ticket. ADR 0062 item 27 asks for the guard on the host address, and the route satisfies it. Accepted.
- The UI gets a ticket when the user presses play, and for the metadata read. It gets no ticket at each change of the address field. Accepted.
- A host that gives 200 for a range request gives the full file, and the player cannot move the position on that host. Accepted as a limit. The operator examines it in the visual check.
- `Enclosure` is shared, so the recording of a live item also has the player and "Properties". The live page has no duration field, so a failed metadata read shows no notice there. Accepted.

## Documentation

- A coding model changed the user guide pages `edit-album.md`, `tracks.md`, `publisher-feeds.md`, `live-shows.md` and `playlists.md`. No page names a removed button. The planning model confirmed the labels in `editor.js`.

## Visual Checks

- 2026-10-27, the planning model: `studio-desktop` with an empty data folder served `cover.js`, `picker.js`, `audio.js` and `editor.js` with status 200. `tools/drive-ui.mjs` loaded the first-run page in headless Chromium with no console error. This check covers no editor page.
- The visual checks of tasks 038 and 039 are open. The operator walks them.
