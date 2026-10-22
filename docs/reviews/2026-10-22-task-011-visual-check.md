# Task 011 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 011](../tasks/studio-v1-task-011-album-editor.md), on the local desktop shell
- Feed: a copy of the MSP 2.0 album "B-Sides The Sch3m3s" from the test corpus, in a local folder
- Checker: operator
- Result: pass

## Results

| Check | Result | Note |
| --- | --- | --- |
| The operator edits each channel field, and the diff shows each change | Pass | Title, artist, description, cover image, website and language. |
| The operator edits each track field, and the diff shows each change | Pass | Title, description, audio address, size, type, duration, date and image. |
| The operator adds, removes and moves a track, and no existing GUID changes | Pass | The moved track kept GUID `8d5d862c`. The new track got a version 4 UUID with `isPermaLink="false"`. |
| The operator writes the draft, and the hosted file has the change | Pass | |
| A change of the hosted file by a different tool gives a conflict on write | Pass | The change by hand stayed in the file. "Start again from the hosted file" loaded it. |
| A rollback to a snapshot gives the original file | Pass | The agent compared the bytes after the write. The file was the same as the original. |
| Each control is 44 px or more, and each step works from the keyboard | Pass | The operator did the rollback with the keyboard only. A headless check found no control lower than 44 px. |
| The diff colors are clear in the dark and the light theme | Pass | |
| Each editor screen is clear in the light theme | Pass | The operator checked it after the commit of the task. |

## Defects Found During The Check

These defects were corrected before the commit of the task:

- A change of an attribute removed the space before `/>` of the tag. The writer now keeps the space of the source.
- A new track was on one line. A new element with child elements now gets the indentation of the document.
- A new track had no `length` and no `type` on its enclosure. [ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md) now makes Studio read them from the host of the file.
- Address fields accepted text such as `none`. ADR 0023 now stops a write while an address is not a full web address.
- A move did not change the track numbers in `podcast:episode`. [ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md) now sets each number from the track sequence.
- The diff used the accent color. The operator asked for green and red. Two diff tokens now give these colors.
- More than one track could be open. Now one track is open at a time.
- After a conflict, the old diff and the write button stayed on the screen. They are now hidden.

## Open Observation

- Two times, the first load of a new Studio address took more than 5 seconds in the browser of the operator. A click on the album after the load was fast. In headless Chromium, the full page loaded in 46 ms. The cause is not known. A browser mode that tries `https` first is possible. The operator sends the Network panel of the developer tools when it occurs again.
