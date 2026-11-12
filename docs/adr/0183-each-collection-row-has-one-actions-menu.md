# ADR 0183: Each Collection Row Has One Actions Menu

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, from the mockups in `.studio-dev/mockup-coll-2/site/`. The operator selected one menu on each row and the badge "✓ Current".
- Technical choices: items 3, 7 and 8 are choices of the planning model.
- Changes: [ADR 0176](0176-the-user-controls-each-collection.md) items 2, 3 and 10. [ADR 0149](0149-the-desktop-app-has-many-collections.md) item 6.
- Uses: [ADR 0176](0176-the-user-controls-each-collection.md) items 6 and 7 for the sheets and the export. [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) for the menu control.
- Source: finding 19 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, the section "Collections" showed nine buttons for two collections. **Copy This Collection…** and **Export This Collection…** were below the list. The page did not tell that "This" is the open collection.

The tag "Open now" of the open collection was blue text. It looked like an action.

## Decision

1. Each row of the section "Collections" has one menu button, **Actions**. The menu has the actions of that collection only.
2. The menu of the open collection has **Rename…**, **Copy…** and **Export…**.
3. The menu of a collection that is not open has **Open**, **Rename…**, **Export…** and **Replace with a File…**. A separator then comes before **Delete…**, which has the destructive role.
4. **Export…** of a collection that is not open makes the account file of ADR 0176 item 7. It does not open the collection.
5. Below the list, the section has only **New Collection…** and **Import an Account File…**.
6. The open collection has the badge "✓ Current", in the style of the badge "✓ Default" of a wallet. The tag "Open now" goes.
7. The menu uses the menu control of the sort menu (ADR 0153), with the roles `menu` and `menuitem`. The group of the list does not clip the open menu.
8. The accessible name of each menu button names its collection, for example "Actions for Main collection".

## Alternatives Considered

- Keep the buttons below the list, with the names "Copy the Open Collection…" and "Export the Open Collection…". Not chosen by the operator: the section then keeps many buttons, and the scope stays far from the row.

## Consequences

- Each action of a collection is on its row. The user sees the scope of each action.
- A user can export a collection that is not open with no delete.

## Acceptance Criteria

Mechanical:

- A Node test: the model of a collection row gives the items of item 2 for the open collection. It gives the items of item 3 for each other collection, in that sequence.
- A Node test: the model gives the badge "Current" for the open collection, and no tag "Open now".
- A Node test: the section gives only **New Collection…** and **Import an Account File…** below the list.
- An API test: the export of a collection that is not open gives its account file, and the open collection does not change.

Visual, by the planning model:

- The list and the two open menus at 1280 px and at 390 px, in the light and dark themes. The open menu is not clipped.

Human, by the operator. This gate stays open until it runs:

- None. The screenshots and the API test prove the change.
