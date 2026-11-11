# ADR 0172: A List Is One Tab Stop

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator. The orchestrator session relayed the decision of musicindex ADR 0061.
- Technical choices: items 2, 5 and 6 are choices of the planning model.
- Applies: musicindex ADR 0061 R-LISTKEY-1 items 1 to 6 and R-LISTKEY-2 items 1 to 3, at musicindex `5123573`.
- Uses: [ADR 0171](0171-a-closed-fold-is-a-row-and-enter-closes-the-field-list.md) for the types of row. [ADR 0157](0157-the-page-tabs-are-a-segmented-control.md) item 5 for the keys of a segmented control.

## Context

In Studio, each row of a list is its own Tab stop, and no list has arrow keys. A keyboard user must push Tab one time for each row to go through a long list. musicindex ADR 0061 makes a list or a tree one Tab stop for the three apps.

## Decision

1. Each list and each tree of item 2 follows musicindex ADR 0061 R-LISTKEY-1 for its keys and R-LISTKEY-2 for its roles.
2. The lists and the trees of Studio:
    - **Trees:** the left panel (the levels, a release and its tracks) and the folders of the file picker.
    - **Lists with expandable rows:** the credit and value rows (`entry-row`), the recipient rows and the rows of "Where this person is".
    - **Lists of rows that open a page or select a value:**
        - the rows of `ListRow` on the People page and on the page of a person
        - the rows of each `PickFold`
        - the rows of the Add feeds page
        - the results of `SearchResults`
        - the list view of Releases
3. The tiles view of Releases is a list too. Left and Right move between tiles in a row of tiles, and Up and Down move between rows of tiles.
4. A fold header (ADR 0171 item 2), a segmented control (ADR 0157 item 5), a select menu and the field list of ADR 0154 keep their own keys.
5. The list remembers its current row. When Tab goes back into the list, the focus goes to that row.
6. One module gives the keys of each list and each tree. A list does not write its own key handler.

## How This Decision Is Checked

Mechanical:

- A Node test of the key module gives the next row of a list model for Up, Down, Home and End. It also gives the open condition after Right and Left.
- A Node test gives, for a tree model, the parent row after Left on a child row.
- A browser drive presses Tab into each list and each tree of item 2. One Tab stop holds the list, and the next Tab leaves it.
- A browser drive reads the roles: `tree` and `treeitem` with `aria-expanded` on each tree, and a button with `aria-expanded` on each expandable row.

Visual, by the agent: the focus ring on the current row of each list, in the light and the dark theme, at the desktop width.

Human, by the operator: the keys on the left panel and on the Credits page with a real keyboard, in the AppImage.
