# ADR 0171: A Closed Fold Is A Row, And Enter Closes The Field List

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator. The operator selected the row for a closed fold through the questionnaire of the rc.21 human test. The orchestrator session relayed the decisions of the open fold, the chevron at the start and the types of row.
- Amended: 2026-11-10. Items 1 to 6 follow musicindex `5123573`: the chevron is at the start, and R-FOLD-1 item 5 gives three types of row.
- Technical choices: items 6 and 9 are choices of the planning model.
- Applies: musicindex ADR 0060 R-FOLD-1 items 1 to 5, R-FOLD-2 and R-FOLD-3. musicindex ADR 0052 R-QUERY-7 item 4, at musicindex `1959db9`.
- Uses: [ADR 0154](0154-the-search-fields-have-a-field-list.md). [ADR 0172](0172-a-list-is-one-tab-stop.md) gives the keys of the expandable rows and the tree rows.

## Context

In Studio, a closed fold, such as "Payees with no person · 10" on the People page, looked like a section heading: small gray capitals. The operator found that it did not show that it can open. The field list also stayed open after Enter ran a search, and it covered the first result (findings 1 and 6 of the [human test of 0.1.0-rc.21](../reviews/2026-11-10-rc21-human-test-findings.md)).

## Decision

### The Folds

1. A fold header follows musicindex ADR 0060 R-FOLD-1 items 1 to 4 when it is closed, and R-FOLD-2 when it is open. Its chevron is at its start (R-FOLD-3 item 1).
2. The fold headers of Studio, by the test of R-FOLD-1 item 5:
    - Each `PickFold` in `crates/api/ui/pickers.js`, for example "Payees with no person" and "Hidden suggestions". These change from small capitals to a row.
    - Each other `details.disclosure` that names a part of a page, for example "Feed facts", "Advanced", "Identity" and "Properties". These have a row already. They get the chevron at the start.
3. These rows are expandable rows (R-FOLD-1 item 5), not fold headers: the credit and value rows (`entry-row`), the recipient rows and the rows of "Where this person is". They keep their chevron at the start.
4. These rows are tree rows (R-FOLD-1 item 5): the rows of the left panel (a release and its tracks) and the folders of the file picker. A tree row has no `›` at its end (R-FOLD-3 item 5).
5. A row that opens a different page keeps `›` at its end (R-FOLD-3 item 2). A control that opens a menu has `⌄` at its end (R-FOLD-3 item 3).
6. The fold styles are in one place in `crates/api/ui/styles.css`. A fold does not set its own header style.

### The Field List

7. Enter with no selected row runs the search and closes the field list (musicindex ADR 0052 R-QUERY-7 item 4).
8. This applies to each field of ADR 0154.
9. A new letter in the field opens the list again.

## How This Decision Is Checked

Mechanical:

- A Node test of the fold model gives the text, the count and the open condition of a fold header.
- A browser drive reads the computed style of each fold header of item 2. The text has the body size and no capitals. The height is the target of musicindex ADR 0024 R-TARGET-1 or more.
- A browser drive presses Enter with no selected row in a field of ADR 0154. The list closes and the results show.

Visual, by the agent: the People page, a feed page and a sheet with folds, closed and open. The screenshots use the light and the dark theme, at the desktop width and at 390 px.

Human, by the operator: the folds on the iPhone.
