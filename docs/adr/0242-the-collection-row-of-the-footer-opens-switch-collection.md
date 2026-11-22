# ADR 0242: The Collection Row Of The Footer Opens Switch Collection

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator. During the human check of ADR 0241, the operator found two footer rows for one subject. The operator asked for one row: "Switch Collection…" with one collection, and the accent row "Collection: {name}" when a selection is possible. The operator selected the link of item 4 from drawings in the questionnaire.
- Technical choices: the main session selected items 2, 3 and 5.
- Bar: opposite choice by mistake. Without this record, an agent can follow ADR 0241 item 4 and ADR 0149 item 17, and show two rows again.
- Changes: [ADR 0149](0149-the-desktop-app-has-many-collections.md) item 17. The row opens the sheet "Switch Collection", not Settings.
- Changes: [ADR 0176](0176-the-user-controls-each-collection.md) item 12. The row also shows with one collection, with the text of item 1.
- Changes: [ADR 0241](0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md) items 3, 4, 5 and 7. One row replaces the two rows, and the sheet gets the link of item 4.
- Source: [the human check of ADR 0241](../reviews/2026-11-21-adr-0241-human-check.md), finding 1.

## Context

ADR 0241 item 4 put the row "Switch Collection…" after the row "Collection: {name}". The two rows did different things: the first row opened Settings, and the second row opened the sheet. With one collection, the footer had no collection row, so the sheet was not available.

## Decision

1. On the desktop with the list of collections, the footer has one collection row, after Settings. With one collection, the row has the text "Switch Collection…" and the Lucide icon `folder-open`. With two or more collections, the row has the text "Collection: {name}" in the accent color and the Lucide icon `folder`.
2. A click on the row opens the sheet "Switch Collection" of ADR 0241 item 5, with each number of collections.
3. The root page of a narrow window ends with this row, then "Lock Studio" or "Unlock Studio", then "Sign Out".
4. The footer of the sheet has the link "Manage Collections…" at the left of **Cancel**. The link closes the sheet and opens Settings, tab Data, which holds the section Collections.
5. The sheet has no "New Collection…", rename or delete. These stay in Settings, section Collections.
6. A server has no collection row. A desktop that opens one folder with `--data-dir` has no list of collections, and has no collection row.

## Alternatives Considered

- Keep the two rows. Not chosen: two rows for one subject, which the operator found during the check.
- A "New Collection…" button in the sheet. Not chosen by the operator. With the link of item 4, each change to the list stays in Settings.
- A menu on the row as an alternative to the sheet. Not chosen: a menu has no area for the text of the restart.

## Consequences

- A desktop with one collection gets one more footer row.
- The accent row no longer opens Settings. The link in the sheet does that.

## How This Decision Is Checked

Mechanical:

- `crates/api/ui-tests/shell.test.mjs`: `leftPanel` gives one collection row on the desktop. The row is "Switch Collection…" with one collection and "Collection: {name}" with two. It gives no row on a server and no row with no list (items 1 and 6). The root page gives the sequence of item 3.

Visual (a person, or the main session with screenshots):

- The footer of a desktop with one collection and with two collections. A click on the row opens the sheet with one collection and with two. "Manage Collections…" opens Settings, tab Data.
