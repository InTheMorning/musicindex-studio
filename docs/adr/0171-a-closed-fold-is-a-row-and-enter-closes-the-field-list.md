# ADR 0171: A Closed Fold Is A Row, And Enter Closes The Field List

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator. The operator selected the row for a closed fold through the questionnaire of the rc.21 human test. The orchestrator session relayed the decisions of the open fold and of the scope.
- Technical choices: items 3 and 6 are choices of the planning model.
- Applies: musicindex ADR 0060 R-FOLD-1 and R-FOLD-2. musicindex ADR 0052 R-QUERY-7 item 4, at musicindex `1959db9`.
- Uses: [ADR 0154](0154-the-search-fields-have-a-field-list.md).

## Context

In Studio, a closed fold, such as "Payees with no person · 10" on the People page, looked like a section heading: small gray capitals. The operator found that it did not show that it can open. The field list also stayed open after Enter ran a search, and it covered the first result (findings 1 and 6 of the [human test of 0.1.0-rc.21](../reviews/2026-11-10-rc21-human-test-findings.md)).

## Decision

### The Folds

1. A closed fold of a section follows musicindex ADR 0060 R-FOLD-1. An open fold follows R-FOLD-2. Examples are `PickFold` in `crates/api/ui/pickers.js` and each other `details.disclosure` that holds a section of a page or a sheet.
2. A row that is an item in a list and can also open in place is not a fold header (musicindex ADR 0060 R-FOLD-1 item 5). In Studio, such rows are:
    - the credit and value rows (`entry-row`) and the recipient rows
    - the folders of the file picker
    - the rows of "Where this person is"
    - the expandable rows of the left panel
3. The fold styles are in one place in `crates/api/ui/styles.css`. A fold does not set its own header style.

### The Field List

4. Enter with no selected row runs the search and closes the field list (musicindex ADR 0052 R-QUERY-7 item 4).
5. This applies to each field of ADR 0154.
6. A new letter in the field opens the list again.

## How This Decision Is Checked

Mechanical:

- A Node test of the fold model gives the text, the count and the open condition of a fold header.
- A browser drive reads the computed style of each fold header of item 1. The text has the body size and no capitals. The height is the target of musicindex ADR 0024 R-TARGET-1 or more.
- A browser drive presses Enter with no selected row in a field of ADR 0154. The list closes and the results show.

Visual, by the agent: the People page, a feed page and a sheet with folds, closed and open. The screenshots use the light and the dark theme, at the desktop width and at 390 px.

Human, by the operator: the folds on the iPhone.
