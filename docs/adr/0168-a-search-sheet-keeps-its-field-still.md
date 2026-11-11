# ADR 0168: A Search Sheet Keeps Its Field Still

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, after the human test of rc.21 steps 18 and 19. The operator found that the field moved while the operator typed.
- Technical choices: items 1 to 4 are choices of the planning model. They correct the defect of finding 7 of the [human test of 0.1.0-rc.21](../reviews/2026-11-10-rc21-human-test-findings.md).
- Uses: [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md), [ADR 0154](0154-the-search-fields-have-a-field-list.md), [ADR 0163](0163-one-sheet-finds-and-links-a-page-on-music-index.md).

## Context

The sheet **Find on Music Index** sits at the bottom of the window, and its height followed its content. When the field list opened, the sheet grew, and the field moved up about 280 px. After each letter, the list changed, and the field moved again. The sheet also showed "No feed on Music Index has this name." while the field list was open, before the results came.

## Decision

1. A sheet with a search field has the full height of a sheet from when it opens. Its top edge does not move while the user types, the field list opens or closes, or the results change.
2. The field list and the results scroll inside the sheet, below the field.
3. "No feed on Music Index has this name." shows only for a finished search: after Enter, or after the search of the text that the field holds. It does not show while the field list is open.
4. Each sheet with a search field follows items 1 to 3: **Find on Music Index**, **Link to an Artist or Label Page**, **Copy from a Feed** and **Find a Nostr profile**.

## How This Decision Is Checked

Mechanical: a browser drive opens each sheet of item 4 at 390 px with touch. It reads the top of the field before the list opens, after the list opens, after two letters and after the results. The four values are the same.

Mechanical: a Node test of the model gives no "no result" text while a search for the current text runs.

Visual, by the agent: screenshots of each step at 390 px and at the desktop width.

Human, by the operator: the same steps on the iPhone, with the keyboard open.
