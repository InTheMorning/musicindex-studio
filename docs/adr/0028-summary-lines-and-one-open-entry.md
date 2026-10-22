# ADR 0028: Summary Lines And One Open Entry

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

On 2026-10-22 the operator checked the album editor and the value editor ([task 011](../tasks/studio-v1-task-011-album-editor.md), [task 012](../tasks/studio-v1-task-012-value-editor.md)). The tracks, the recipients and the persons were long lists of open fields. The operator scrolled far to find a control, and closed open items by hand. The operator asked that no part of the app make the user do these two things.

The Apple guidance for disclosure controls says: "Use a disclosure control to hide details until they're relevant". It also says that the controls that persons use most go at the top.

## Decision

1. Each entry of a list, such as a track, a recipient, a person or a support link, shows as one summary line. The line shows the most important values of the entry. The user opens the line to edit the entry.
2. In each list, one entry is open at a time. When the user opens an entry, the open entry of that list closes.
3. An entry that the user adds starts open.
4. A section of the editor that the user does not change in each visit starts collapsed. Its summary line tells what it contains, for example the number of persons or a share bar of the recipients.
5. In an open group, the control that the user uses most comes first.

## Alternatives Considered

- Open lists, with a button that closes all entries. Rejected: the user then continues to scroll and to close items by hand.

## Consequences

- The UI uses `details` elements. A list gives its entries one `name`, so that the browser keeps one entry open.
- A test examines the UI files. Each `details` element must be an entry with a `name`, or a section with the class `disclosure`.

## Invariants

- No list opens all its entries at the same time.
