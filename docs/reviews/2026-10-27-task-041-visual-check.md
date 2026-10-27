# Task 041 Visual Check

- Date: 2026-10-27
- Reviewed: the visual criteria of [task 041](../tasks/studio-v1-task-041-msp-fields.md) and of packets 041b, 041d and 041e
- Decision: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md)
- Host: a local folder with the public address `https://music.example/`. A local test server gave the images, the audio files and the lyrics files.
- Feed: one album with each new channel element, and a track with "Explicit", two persons, two lyrics files, a season and two images
- Checker: the agent, with `tools/drive-ui.mjs` in headless Chromium. Each screenshot is in the dark theme and the light theme at 1100 px, and in the dark theme at 390 px with touch.
- Result: pass, after four corrections. One open item and one limit of the check stay.

## Procedure

| Step | Screen | Result |
| --- | --- | --- |
| 1 | The album page | Pass. "Explicit" is after "Hide from apps". "Album details" has the row "Feed settings" with its summary. |
| 2 | The page "Feed settings" | Pass after correction 1. The page shows the owner, the keywords, the lock with its owner email, the managing editor and the webmaster. |
| 3 | The track page | Pass after correction 2. "Explicit" is on after "Duration". "Credits for this track" shows two persons. "Lyrics" shows two entries, with one open. "Identity" shows the season. |
| 4 | "More artwork" of the album, with one entry open | Pass after correction 3. The purpose check boxes, the width, the height, the ratio and the type show. |
| 5 | "More artwork" of a track | Pass. The disclosure shows two images. |
| 6 | A new image address in an artwork entry | Pass. After the load, the entry shows the width, the height, the ratio `1/1` and the type `image/png`. |

## Corrections

1. The toolbar path showed "Visual Album ›" with no page name. `PAGE_NAMES` of `app.js` gets "Feed settings".
2. In an open disclosure of a group, a facts list and a two-column grid had no inner space. The season fields, "Track number" and "Track GUID" touched the edge of the group. "Feed facts" and the "Identity" of a live show had the same defect before task 041. A rule of `styles.css` gives them the space of a field.
3. The hint of "Ratio" put its label above the labels of "Width" and "Height". The field has no hint after the correction, because Studio fills the ratio after a new address.
4. A second set of screenshots confirmed corrections 1 to 3. "Feed facts" was open in that set.

## Open Items

- The open disclosure "More artwork" of a track has about 30 px of space above its first entry, and its entries have no lines between them. The operator judges if this needs a change.
- The purpose tokens show in a bold sans font in these screenshots, not in the code font. The label of a check box is bold. In this headless Chromium, a bold monospace font falls back to a sans font, because fontconfig has no default configuration. The binary holds the `<code>` element. The operator examines the code font in a desktop browser.
- The human test of task 041 with a published feed of the operator is open.
