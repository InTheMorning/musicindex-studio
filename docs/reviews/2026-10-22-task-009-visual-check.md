# Task 009 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 009](../tasks/studio-v1-task-009-more-discovery-methods.md), on the local desktop shell, with the live musicindex API
- Checker: operator, in Firefox on Linux
- Result: pass

## Results

| Check | Result | Note |
| --- | --- | --- |
| The first screen asks one question | Pass | No passphrase step comes first. |
| The result screen groups the feeds by host, and all feeds are selected at the start | Pass | The operator selected 2 albums in a musicindex search. Studio also found the publisher feed through `podcast:publisher`, and grouped the 3 feeds under `files.heycitizen.xyz`. |

## Defects Found During The Check

These defects were corrected:

- "Import from MSP 2.0" and three other secondary actions looked like captions. They are now links at the left of the form. Commit `a6d37e6`.
- The "Find my feeds" button was above the search results. It is now below them and shows the number of selected sources. Commit `cebb64c`.
- When each found feed was in the collection, the result screen showed a disabled "Add 0 feeds" button. It now shows a sentence.
