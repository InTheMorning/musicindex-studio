# Studio v1 Task 119: Each Collection Row Has One Actions Menu

Decision: [ADR 0183](../adr/0183-each-collection-row-has-one-actions-menu.md).

Status: Not started.

## Goal

Each row of the section "Collections" has one **Actions** menu with the actions of that collection. The open collection has the badge "✓ Current". Below the list, the section has only **New Collection…** and **Import an Account File…**.

## Mockups

The operator selected the mockup in `.studio-dev/mockup-coll-2/` (ignored by Git, on the computer of the operator):

- `site/coll.html` and `site/coll.js`: the page and the menu. Copy the layout, not the code.
- `shots/list-*.png`: the rows with the badge "✓ Current" and **Actions**.
- `shots/menu-open-*.png`: the menu of the open collection.
- `shots/menu-other-*.png`: the menu of a collection that is not open, with the separator before **Delete…**.

The code uses the tokens and the components of `crates/api/ui/styles.css`. Compare the badge "✓ Default" of a wallet.

## Constraints

- The menu uses the menu control of the sort menu (ADR 0153), with the roles `menu` and `menuitem`. Do not make a second menu control.
- **Export…** of a collection that is not open does not open it (ADR 0183 item 4).
- The sheets of ADR 0176 items 6 and 7 do not change.

## Files To Read First

- ADR 0183, ADR 0176 and ADR 0153.
- `crates/api/ui/collectionlist.js` and `crates/api/ui-tests/collectionlist.test.mjs`.
- `crates/api/ui/app.js`: the section "Collections", with **Copy This Collection…** and **Export This Collection…**, and the sort menu.
- `collections::routes` in `crates/api/src/` for the export of a collection.

## Work

- The row model gives the menu items of ADR 0183 item 2 for the open collection. It gives the items of item 3 for each other collection, with the separator and the destructive role of **Delete…**.
- The row model gives the badge "Current" for the open collection. Remove the tag "Open now".
- **Copy…** and **Export…** of the open collection open the current sheets of **Copy This Collection…** and **Export This Collection…**. Remove these two buttons below the list.
- **Export…** of a collection that is not open uses the export of ADR 0176 item 7.
- The menu button has the accessible name "Actions for {name}".
- The section group does not clip the open menu. Examine the last row at 390 px.
- Change the Node tests of ADR 0176 that the ADR changes. Add the Node tests and the API test of ADR 0183.

## Acceptance Criteria

Mechanical: the criteria of ADR 0183.

Visual, by the planning model:

- The visual criterion of ADR 0183: the list and the two open menus at 1280 px and at 390 px, in the light and dark themes. Compare the screenshots with the mockup.

Human: none (ADR 0183).

## Stop And Ask When

- The sort menu control cannot hold a separator or a destructive item without a change of its behavior on the People list.
