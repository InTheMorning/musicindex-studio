# Task 012 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 012](../tasks/studio-v1-task-012-value-editor.md), on the local desktop shell
- Feeds: copies of the MSP 2.0 album "B-Sides The Sch3m3s" and of the corpus feed "v4vmusic", with its medium changed to `music`, in a local folder
- Checker: operator
- Result: pass

## Results

| Check | Result | Note |
| --- | --- | --- |
| The operator edits each recipient field of the album, and the diff shows each change | Pass | The diff changed only the edited recipient line. |
| The operator gives a track its own recipients, and the diff shows them in that track only | Pass | The copied recipients kept the attribute sequence of the file. |
| A track with value time splits shows their number, and a write keeps them | Pass | The agent compared the file after the write. Only the edited recipient line changed. |
| The percentage of each recipient agrees with its shares | Pass | |
| One recipient of a list is open at a time, and the credits start collapsed | Pass | |
| The share bars and their legend are clear in the dark and the light theme | Pass | |

## Changes During The Check

The operator asked for these changes, and the agent made them before the commit of the task:

- The editor does not show payment blocks. It shows the recipients of the first block directly, with no method, payment type or suggested amount. Studio keeps those values.
- The type of a recipient is a list. A recipient of type `lnaddress` does not show the custom key and value, and the address hint follows the type.
- The editor does not edit value time splits. v4vmm is to send them to Studio.
- The album recipients, the recipients of a track, the credits and the earlier versions are collapsed groups with a summary line. In a track, "Use the album recipients" comes first.
- Each recipient has a meter. Each collapsed recipients line has a share bar and a legend. A recipient below 5 % folds into "Other". A label appears immediately on hover.
- [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md): each list entry is one summary line, and one entry of a list is open at a time.

These defects were corrected:

- A copy of recipients changed the attribute sequence of each line. A new element now gets the attribute sequence of the first source element with the same name.
- A field with no hint stretched when it was adjacent to a field with a hint. The fields of a row now have the same height.
- A new entry closed at insertion when its list had an open entry. It now opens after the insertion.

## Notes For Later

- The recipient picker and the import from musicindex: [task 013](../tasks/studio-v1-task-013-recipient-picker.md).
- A check that a payment route works. It is in the open questions of the plan.
