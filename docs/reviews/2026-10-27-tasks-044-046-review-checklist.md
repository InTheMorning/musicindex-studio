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
| 045a | Ready | Not started |
| 045b | Ready | Not started |
| 046a | Ready | Not started |
| 046b | Ready | Not started |
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
