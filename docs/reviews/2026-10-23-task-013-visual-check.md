# Task 013 Visual Check

- Date: 2026-10-23
- Reviewed: the visual criteria of [task 013](../tasks/studio-v1-task-013-recipient-picker.md), on the local desktop shell
- Feeds: a copy of the check data of task 011, with the albums "B-Sides The Sch3m3s" and "v4vmusic" in a local folder, and the live musicindex API
- Checker: operator
- Result: pass

## Results

| Check | Result | Note |
| --- | --- | --- |
| The operator adds two recipients from the list to a track, and the recipients of the editor stay | Pass | The two recipients came at the end, with no split. |
| The operator imports selected recipients from a musicindex track, and only the selected ones are added | Pass after changes | See the next section. |
| The operator saves a recipient, and it is in the list of a different feed | Pass | The operator saved "HeyCitizenNode" in B-Sides and added it to v4vmusic. |

## Changes During The Check

The operator asked for these changes, and the agent made them before the check continued:

- [ADR 0030](../adr/0030-an-added-recipient-gets-no-split.md): an added recipient gets no split. A recipient from musicindex keeps its fee mark. This changes ADR 0027 item 7.
- The musicindex search is above the list of the user's recipients, which can be long.
- The musicindex part shows the name of the open feed, or of the open track and its feed.
- After each move between the hits, a feed and a track, the sheet scrolls to the top of the musicindex results.
- A handle in the footer opens the list of the selections, with the source of each. The list does not move the parts of the sheet.
- The rows of the picker have fixed columns. The status and the Remove button are in the same position in each row.
