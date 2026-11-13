# ADR 0163: One Sheet Finds And Links A Page On Music Index

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire. The operator selected the drawing of one search sheet with the check box "Link this page". The operator also selected the drawing of a sheet with a fixed height and a field that stays at the top.
- Changes: [ADR 0117](0117-the-page-of-a-person-finds-values-on-musicindex.md) items 2, 8 and 13. [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md) items 4 and 5.
- Uses: [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for the pending changes. [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) for the layout of a sheet. [ADR 0154](0154-the-search-fields-have-a-field-list.md) for the field list. [ADR 0146](0146-link-words-already-marks-and-credit-edits.md) for the mark of a value that the person has.
- Changed by: [ADR 0192](0192-add-a-person-has-the-layout-of-the-other-pickers.md). Item 2: "Add a Person" has the layout of the other pickers, and the pages of the user come before the results.

## Context

The page of a person had two sheets with the same search of Music Index:

- **Find on Music Index** (ADR 0117) copies the picture, the website, the wallets and the Nostr key of one page into the fields one time.
- **Link to an Artist or Label Page** (ADR 0159) keeps a link to a page. Each subsequent change of the page shows at its field with **Use**.

On 2026-11-10, the operator used the two sheets for one person and searched two times. The operator asked for one flow.

The same check found two problems of the search sheets. The field list was in the flow below the field, so a click in the field made the sheet taller (item 4 of the [findings of 2026-11-10](../reviews/2026-11-10-local-use-findings.md)). The sheet also became taller when the results came.

## Decision

### One Search

1. Two buttons open the same sheet. **Find on Music Index** is at the head of the page of a person. **Link to an Artist or Label Page** is in the section of the linked pages. The title of the sheet is "Find on Music Index: {name}". This changes ADR 0117 item 2 and ADR 0159 item 4.
2. The sheet has three parts:
    - the search field, with the field list of ADR 0154 item 5
    - the results, in the levels of ADR 0117 item 4
    - the fold "Your artist and label pages" of ADR 0159 item 4
3. A row of the fold makes the link of ADR 0159 item 1, and the sheet closes.
4. A result opens the sheet "From Music Index: {title}" of ADR 0117 item 8. This changes ADR 0159 item 5: a result makes no link before this sheet.

### Link And Fill In One Sheet

5. The sheet "From Music Index: {title}" has the section "Link" above the section "Fill". The section "Link" has one check box row, "Link this page", which starts selected.
6. The row has one line. For a page that is not in the collection, the line is:
    "Read only. Subsequent changes of this page show on this person."

    For a feed of the collection, the line is "Your page. The details of this person go to this page." The link is the link of ADR 0159 item 1 or item 2.
7. When the person has a link to this page, the row has no check box. It shows the mark "Already linked" of ADR 0146 item 3.
8. The section "Fill" keeps ADR 0117 items 9 to 12.
9. The words of the primary button tell what it does:
    - "Link and Fill N Fields", with the link and N fields selected
    - "Link", with only the link selected
    - "Fill N Fields", with only fields selected

    With nothing selected, the button is not available. This changes ADR 0117 item 13.
10. The note is "Studio puts the values in the fields and the page in the links. Select Save to keep them. Studio changes no feed."
11. The link and each value are pending changes until Save (ADR 0116, ADR 0159 item 6).

### The Height Of A Search Sheet

12. Each sheet with a search field of ADR 0154 has the largest height of a sheet from the start. The height does not change when the results come.
13. The search field stays at the top of the body of the sheet. Only the rows below it scroll.
14. The field list is a floating layer at the field, as on musicindex.org. When the window has less space below the field, the list opens above it.

## Alternatives Considered

- **Link only.** No **Find on Music Index**: each value of a linked page shows with **Use**. Not chosen: the user selects **Use** one time for each value, and cannot decline a value.
- **Two buttons, and a check box in the fill sheet.** Not chosen: the user continues to find two buttons that start the same search.
- **A sheet that grows with its results.** Not chosen: the content moves while the user reads it.

## Consequences

- One function of the UI gives the words of item 9. One function gives the condition of the link row of items 5 to 7. `pickLink` keeps the patch of a link.
- The page of a person has no sheet "Link to an Artist or Label Page".
- The search sheets of a credit, of a picker and of **Add tracks** get the height of items 12 and 13.

## Acceptance Criteria

Mechanical:

- For the four cases of item 9, the function gives "Link and Fill 2 Fields", "Link", "Fill 1 Field" and no words.
- The function of items 5 to 7 gives these rows:
    - a page that is not in the collection: a selected row with the read-only line
    - a feed of the collection: a selected row with the line of a collection page
    - a page that the person has: the mark "Already linked"
- A fill with the link selected gives a patch with the link and with each selected value.
- The Node tests pass.

Visual, from agent screenshots at 1280 px in the dark and the light theme, and at 390 px:

- The two buttons open the same sheet.
- The sheet "From Music Index" shows the section "Link" above "Fill".
- A search sheet keeps its height while the results come, and the field stays at the top when the rows scroll.
- The field list floats at the field and does not change the height of the sheet.
