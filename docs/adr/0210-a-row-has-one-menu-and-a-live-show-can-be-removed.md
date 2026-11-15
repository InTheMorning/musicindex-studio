# ADR 0210: A Row Has One Menu, And A Live Show Can Be Removed

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire, from drawings. The operator selected "One Actions menu for each row" for the wallet rows. The operator selected "Ellipsis on the phone, 'Actions' on the desktop" for the button of a row menu. The operator selected "Button on the page of the show" for the removal of a live show.
- Technical choices: the planning model selected items 2, 4 to 8, 11, 12 and 14 to 16.
- Changes: [ADR 0183](0183-each-collection-row-has-one-actions-menu.md) item 1: the menu button of a collection row has two forms.
- Changes: [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) item 38: the page of a live show also removes the show.
- Uses: [ADR 0203](0203-people.md) item 10 for the default address. [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) for the menu control. The icon button follows musicindex ADR 0069 R-ACT-2 item 2, ADR 0068 R-ICON-1 item 2 and R-TARGET-1, and ADR 0026 R-ELLIPSIS-1. [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md) for the confirm. [ADR 0007](0007-draft-diff-write-and-announce.md) for the draft and the review.
- Source: design item 9 of the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), and finding 1 of the [local use findings of 2026-11-10](../reviews/2026-11-10-local-use-findings.md).

## Context

On the iPhone, the two address rows of a wallet put their buttons in a different sequence. The first row has the copy icon, then **Edit**. The second row has **Make Default**, then the copy icon, and **Edit** is alone on the next line.

The collection rows of Settings already have one menu button, **Actions** (ADR 0183). A text button uses much of the width of a row on a phone.

A live show cannot be removed. The tab **Live Shows** lists each show, and the page of a show has only edits (ADR 0205 item 38). A test show stays in the draft.

The operator asked if "…" is permitted as the button of a menu. The shared rules permit an icon button with the Lucide icon `ellipsis`. They do not permit the text character "…" as a label, because that character has a different meaning (musicindex ADR 0026 R-ELLIPSIS-1).

## Decision

### The Menu Of A Row

1. Each address row of a wallet has one menu button. The row has no other button.
2. The menu has **Make Default**, **Edit** and **Copy**, in that sequence. The default address has no **Make Default**.
3. The menu button of a row has two forms. In the narrow layout it is a round icon button with the Lucide icon `ellipsis`. In the wide layout it is the text button **Actions**.
4. The two forms are one control with one accessible name, which names its row, for example "Actions for hey@getalby.com". The layout selects the form. The label is not the text character "…".
5. On Touch, the target of the icon button has the default size of musicindex ADR 0068 R-TARGET-1.
6. The collection rows of ADR 0183 use the same control, with the same two forms.
7. An address row shows its label, the address, and one status line. The status line has the tag **Default**, when applicable, and the text of the use count.
8. **Remove Wallet** stays a button of the wallet. It is not in a menu.

### The Removal Of A Live Show

9. The page of a live show has the button **Remove the Show** at its end. It is destructive.
10. The button opens a confirm with the title `Remove “{title}”?` and the destructive button **Remove the Show**.
11. The text of the confirm is "Studio removes the show from the draft. The feed changes at the next write." For a show with the status `live`, the text also says "Apps lose the stream of this show at that write."
12. After the confirm, Studio removes the live item from the draft and shows the tab **Live Shows**. Review shows the removal before the write, as each change of the draft.
13. The tab **Live Shows** has no remove control on its rows.
14. The removal adds no rule to the UI. A rule of the core about a live item with a grant stays in the core.

### The Phone Menu Stays Open

15. The operator selected a menu button in the bar for a quick way to the left panel on the phone. musicindex ADR 0069 R-SHELL-2 item 1 gives Studio no menu button and no drawer in a narrow window.
16. This ADR does not decide the phone menu. A change of the shared rule goes to the orchestrator first.

## Alternatives Considered

- A status line and one line of buttons in a fixed sequence for each address row. Recommended by the planning model. Not chosen by the operator.
- The ellipsis icon at each width. Recommended by the planning model. Not chosen by the operator.
- The text button **Actions** at each width. Not chosen: it uses about 80 px of each row on a phone.
- A remove button on each row of the tab **Live Shows**. Not chosen: a destructive button is then adjacent to the row that opens the show.

## Consequences

- Each action of an address needs two taps.
- A row menu has two forms to check at each change.
- The copy of an address is a menu item, not the copy icon of the other rows of the page.
- A show that is live can be removed. The confirm tells the result.

## How This Decision Is Checked

Mechanical:

- A Node test proves the menu items of an address row, with and without **Make Default**.
- A Node scan test proves that the wallet rows and the collection rows use one menu control, and that no label is the character "…".
- A Node test proves the confirm of the removal for a show that is pending and for a show that is live.
- A Node test proves that the model has no live item of the removed show, and keeps each other show.
- The five commit checks pass.

Visual, by the planning model:

- A wallet with two addresses, and the collection rows, at 1280 px light and 390 px dark, with the open menu and the fit checks.
- The page of a live show with **Remove the Show**, its confirm, and Review after the removal.

Human, by the operator. This gate is open:

- On the iPhone, open a person with a wallet. Each address row has the ellipsis button, and its menu has each action.
- In the desktop app, remove a test live show and write. The feed has no live item of that show.
