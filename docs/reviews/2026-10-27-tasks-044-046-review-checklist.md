# Tasks 044 To 046 Review Checklist, 2026-10-27

This record holds the orchestration of [task 044](../tasks/studio-v1-task-044-outline-head.md), [task 045](../tasks/studio-v1-task-045-page-sequence-and-artwork.md) and [task 046](../tasks/studio-v1-task-046-read-only-editor.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0067](../adr/0067-a-read-only-feed-opens-in-the-editor.md) to [ADR 0070](../adr/0070-a-cover-opens-the-artwork-sheet.md) own the rules.

## Sequence

The packets run one at a time, because each packet changes the UI or the write pipeline:

1. Task 044.
2. Task 045a.
3. Task 045b.
4. Task 046a.
5. Task 046b.
6. Task 046c.

A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. Before each commit of a UI packet, the planning model makes the screenshots and runs `$edges()`. On 2026-10-27 the operator said to commit each packet after it passes.

## Packets

| Packet | Status | Review |
|---|---|---|
| 044 | Complete | Pass. See "Packet 044". |
| 045a | Complete | Pass. See "Packet 045a". |
| 045b | Complete | Pass. See "Packet 045b". |
| 046a | Complete | Pass after one correction. See "Packet 046a". |
| 046b | Complete | Pass after one correction. See "Packet 046b". |
| 046c | Ready | Not started |

## The Review Of Each Packet

- No feed rule in the UI (ADR 0065). Each new UI function that changes the model gives only a start value or a display value.
- Each removed function has no caller in `crates/api/ui/`.
- The corpus round-trip tests pass with no change.
- The write of a feed with a connector does not change (packets 046a and 046b).
- The public address of a read-only feed is read only through the `Web` of the workspace (ADR 0057).

## Decisions

### Before The Packets

- The operator accepted ADR 0067 to ADR 0070 on 2026-10-27.
- After the acceptance, the planning model found that ADR 0067 item 7 said that a moved feed keeps its path. A read-only feed has no path. Item 7 now gives the path of ADR 0035 item 13: the release folder, then the file name of the public address. The ADR records the correction.
- The `showReleases` function in `app.js` goes back in the history when Releases is the entry before. ADR 0069 item 7 forbids this. Task 044 adds `releasesView`, which keeps the scroll of Releases with no history back.
- Task 045 does the Artwork sheet first, because the page lists of packet 045b contain the row "Artwork".
- The section headings of ADR 0068 item 2 are "Album details" and "Feed". The publisher page gets the row "Feed facts", which it did not have.
- A read-only draft keeps its host choice as the draft feed move of ADR 0038 item 16. The feed row stays read-only until the write succeeds.
- `FeedMoveMethod::None` gives the move of ADR 0067 item 8.
- The write of a read-only draft compares the SHA-256 of the public file with the draft base version (ADR 0007 item 3).
- A folder match keeps the draft. When the host has the bytes of the draft base, the base version becomes the version of the host.

### Packet 044

- The diff follows the packet. The planning model divided one long comment line in `shell.js`.
- The four commit checks pass: Cargo 961 tests, Node 190 tests.
- The visual check used `.studio-dev/visual-044` in the dark theme, the light theme and at 390 px. The head has the selected fill on the album page and no fill on a track page. "‹ Releases" opens Releases with the row of the feed marked. At 390 px, the head is 59 px high, and the outline page has no row "Album". `$edges()` gives an empty list.
- `drive-ui.mjs` cannot send a key. A temporary copy in the scratchpad sent Tab and Enter. Tab reaches the head, the focus ring shows, and Enter opens the album page.
- Open: the list of Releases in the check has five feeds and does not scroll. So the check did not examine the scroll position after "‹ Releases".

### Packet 045a

- The diff follows the packet. The coding model added the component `ArtworkRow` for the row "Artwork", and a prop `feed` on `ArtworkSheet` to know a playlist and a publisher feed. Both are in the limits of ADR 0070.
- The coding model was not sure that a track sheet stays open after a save gives the track its GUID. The lookup uses `isTrack`, which also matches the key `cid-…`, and the local model keeps `_cid`. So the sheet stays open.
- The planning model changed the preview to a radius of 14 px and the shadow of a large image. A preview of 240 px gets 14 px from musicindex ADR 0021 R-ART-4 items 2 and 3.
- In the visual check, each sheet group with fields had 32 px between its edge and its rows. A page group has 16 px. `.sheet .group` and `.group > .field` both gave 16 px. The planning model added `.sheet .group.rows` with no padding. The groups of the Artwork sheet, "Add a track" and "Add a live show" have this class. The Versions sheet and the playlist picker keep the padding, because their lists have no inner space of their own.
- The four commit checks pass: Cargo 961 tests, Node 193 tests.
- The visual check used `.studio-dev/visual-044` in the dark theme, the light theme and at 390 px. A click on the album cover and on the track cover opens the sheet of the correct item. Escape and "Done" close it. "Use the Album Cover" shows only for a track with its own image, and it clears the field. The sheet fits the window at each size. `$edges()` gives an empty list on the page and in each sheet.
- Open, for the operator to judge: the sheet shows the image two times, as the large preview and as the small preview of the image field.

### Packet 045b

- The diff follows the packet. The coding model added the helpers `showSections` and `showEach`, which show a list through a map from the ID to a render function.
- The coding model changed `crates/api/src/ui.rs`, which the packet did not name. The server sends each UI file from the list `FILES`, and a Rust test requires each UI script in that list. So `pages.js` needs an entry. The planning model accepts this change.
- The planning model divided one long comment line in `editor.js`.
- The four commit checks pass: Cargo 961 tests, Node 200 tests.
- The visual check used `.studio-dev/visual-044`, with a new label feed and a new playlist, in the dark theme, the light theme and at 390 px. On the five pages, the headings and the field sequence agree with the packet. `$edges()` gives an empty list on each page.

### Packet 046a

- The first diff put an upload that waits for a host into `BlockedDelete`, as the packet text said. That type is for a refused deletion. The change made `connector_id` and `operation_id` optional in each refused deletion. The planning model rejected it. The coding model restored `BlockedDelete`, and added `WaitingUpload` and the diff field `waiting_uploads`. The paths of the list are in the bytes of the diff ID.
- No path reaches a write check of `waiting_uploads`. A write with no host choice stops at "choose a host for this feed first", and with a host choice each upload has a planned host. The planning model removed that check, because no entry point reaches it.
- A feed with no host has no planned address for a library file. So `waiting_upload` compares the end of each draft address with the library path. Packet 046b removed this rule (see "Packet 046b").
- The write of a feed with a connector does not change. Each test of it passes with no change.
- The four commit checks pass: Cargo 975 tests, Node 200 tests. The core pipeline tests went from 92 to 105.
- The tests examine each criterion of the packet. The write of a moved read-only draft sends only GET requests to the old address. The announcements give the new address, then the old address.

### Packet 046b

- The routes follow the packet. The draft routes and the diff route work for a feed with no connector. The draft response gives `has_host`, and the host route accepts `null`.
- The planning model accepts `502 network` for a write that the network policy refuses. `refresh_public` gives the same code for the same refusal.
- The old test `each_draft_route_of_a_read_only_feed_gives_read_only` stated a rule that ADR 0067 replaced. The coding model replaced it with a test of the routes that still need a host.
- In the review, the planning model found a defect of packet 046a. The import route refuses a feed with no connector, so no library file of a read-only draft is an upload. The rule of `waiting_upload` also listed each file that Studio downloaded from the feed. So `waiting_uploads` gave only false results. The coding model removed it from the core and the API.
- The import of a file into a read-only feed gives 409 `read_only` and "choose a host for this feed first, in Review". The planning model moved this check before the encode, so that Studio does no work for a refused file.
- These routes still need a host and give 409 `read_only` for a feed with no connector: the snapshots, a snapshot to the draft, the host operations and the hosts of the files. Packet 046c shows their error text.
- The four commit checks pass: Cargo 982 tests, Node 200 tests.
