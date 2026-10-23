# Phase 5 Visual Check

- Date: 2026-10-23
- Reviewed: the visual check of Phase 5 in the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md): the operator edits a published album feed completely
- Feed: the published file of the MSP 2.0 album "B-Sides The Sch3m3s", SHA-256 `0de945d6f9b14d91cdee50cce6d18942fe0b1b300644e67daaaaeec2f3d3df82`, in a new local folder, with a new data folder
- Checker: operator
- Result: pass

## Results

| Step | Edit | Result |
| --- | --- | --- |
| 1 | The first run with an empty collection | Pass |
| 2 | Connect the folder as a host. Studio offers the feed file. | Pass |
| 3 | Add the feed and open the editor | Pass |
| 4 | Change the description and the language | Pass after task 015 |
| 5 | Add a person, a support link and a license | Pass |
| 6 | Change the split of an album recipient | Pass |
| 7 | Move a track up. The track numbers follow the sequence (ADR 0024). | Pass |
| 8 | Change the title of a track | Pass |
| 9 | Add a recipient to one track with the picker, and give it a split | Pass |
| 10 | Add a track with the facts of its audio file from the host | Pass after the extension of ADR 0031 |
| 11 | Put the published version in the draft, and write it | Pass |

After step 11, the agent compared the file with the published file. The two files have the same bytes.

## Defects Found During The Check

- Step 4: the operator cleared the description and typed a new text. The automatic save of the empty field removed the element, and the new text added it before the first item. [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md) and [task 015](../tasks/studio-v1-task-015-hosted-position.md) put a new element at its position in the hosted file.
- Step 10: the new track had the sequence `guid`, `enclosure`, `podcast:episode`, `title`. The operator extended ADR 0031: a new item uses the sequence of the last hosted item.

## Observations

- The description field shows the line breaks and the indentation of the XML text. The writer keeps the text of the file. The operator did not ask for a change.
- A new element that the hosted file does not have, such as `podcast:license`, goes before the first item (ADR 0031 item 3).
