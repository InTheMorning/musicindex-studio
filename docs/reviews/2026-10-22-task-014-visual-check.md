# Task 014 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 014](../tasks/studio-v1-task-014-editor-layout.md), on the local desktop shell
- Feeds: a copy of the check data of task 011, with the MSP 2.0 album "B-Sides The Sch3m3s" in a local folder
- Checker: operator
- Result: pass

## Results

| Check | Result | Note |
| --- | --- | --- |
| The operator opens a track from the tree, and the editor shows only that track | Pass | |
| The operator edits a field at the end of a long page, and "Review" stays in view | Pass | |
| The operator reviews and writes a change from the Review sheet | Pass | After the write, the page stayed on the same track, because the view identifies the track by its GUID. |
| The operator puts an earlier version in the draft from the Versions sheet | Pass | |
| The operator adds a track from the sheet, and the size and the type come from the host | Pass after two corrections | See the next section. |
| On a phone width, the menu button opens the tree, and a tree item closes the drawer and opens its page | Pass | The operator used an iPhone XR emulation, 414 px wide. The Review sheet filled the width. |
| The three background levels are easy to read in the dark and the light theme | Pass | |

## Corrections During The Check

The agent found two defects during the check and corrected them before the check continued:

- A page change kept the scroll position of the page before it. After the correction, each page starts at its top.
- A new track has a key from its client ID. After its first save, the server gives it a GUID, and the editor then showed the album page. After the correction, the client ID continues to find the track, and the view gets the new key.

## Observation

The host gave a size of 4,239,916 bytes for the audio file of "Mary Jo". The feed gives 4,228,749 bytes. The length in the feed is probably out of date. The editor shows the length of the feed and does not correct it.
