# ADR 0241: Sign Out And Switch Collection Are Rows Of The Panel Footer

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator. In the chat, the operator asked for a control to sign out of the public server. The operator also asked for a control on the desktop to change between collections. The operator selected item 1 and item 4 from drawings in the questionnaire.
- Technical choices: the main session selected items 2, 3 and 5 to 8. The operator did not receive them as a question.
- Bar: opposite choice by mistake. Without this record, an agent can keep Sign Out only in Settings, where the operator did not find it.
- Changes: [ADR 0203](0203-people.md) item 36. The sequence of the bottom group gets the rows of items 1 and 4.
- Changes: [ADR 0226](0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) item 7. The root page of a narrow window ends with the rows of item 3.
- Uses: [ADR 0004](0004-server-sign-in-with-nostr-or-email.md) for the sign-in of a server. [ADR 0149](0149-the-desktop-app-has-many-collections.md) item 14 for "Open" of a collection. [ADR 0176](0176-the-user-controls-each-collection.md) item 12 for the row "Collection: {name}".
- Changed by: [ADR 0242](0242-the-collection-row-of-the-footer-opens-switch-collection.md). Items 3, 4, 5 and 7: one collection row opens the sheet, and the sheet links to Settings.

## Context

On 2026-11-21, the operator did not find the control to sign out of the public server. The button "Sign Out" is only in Settings, tab Account, adjacent to "Delete the Account" (`AccountPanel` in `crates/api/ui/app.js`). The footer of the left panel has People, File hosts, Settings and "Lock Studio". "Lock Studio" locks the vault, but the account stays signed in.

On the desktop, each collection has its own vault and its own passphrase (ADR 0149 item 7). A user opens a different collection only from Settings, section Collections. The operator uses a different collection for each person or project, and wants one step to change between them.

## Decision

### Sign Out On A Server

1. On a server, the footer of the left panel has the row "Sign Out" after "Lock Studio" or "Unlock Studio". The row has the Lucide icon `user` of musicindex-assets. The shared set has no `log-out` icon, and this record adds no shared icon.
2. "Sign Out" calls the route `POST /auth/v1/signout`, as the button of Settings does. Then Studio shows the sign-in page. The button of Settings, tab Account, stays.
3. The root page of a narrow window ends with the collection rows of item 4, then "Lock Studio" or "Unlock Studio", then "Sign Out".

### Switch Collection On The Desktop

4. On the desktop, when the list has two or more collections, the footer has the row "Switch Collection…" after the row "Collection: {name}". The row has the Lucide icon `folder-open`.
5. "Switch Collection…" opens the sheet "Switch Collection". The sheet has one row for each collection, in the order of Settings. The open collection has the text "Open now" and no button. Each other collection has the button **Open**. The sheet has **Cancel**.
6. **Open** does "Open" of ADR 0149 item 14: the rules of a write that runs, the restart on that collection, and the locked vault of that collection.
7. The sheet has no "New Collection…", rename or delete. These stay in Settings, section Collections.
8. A desktop has no "Sign Out", and a server has no "Switch Collection…".

## Alternatives Considered

- One footer row with the account name and a menu: Account Settings, Lock Studio, Sign Out. Not chosen: one more selection to sign out.
- On the desktop, a page that lists the collections at each start, and a row "Close Collection". Not chosen: one more step at each start, and a larger change before a collection opens.
- No change on the desktop. Not chosen: the operator wants one step to change collections.

## Consequences

- A person who opens a different collection gets its locked vault, so the next person needs the passphrase of that collection.
- The footer of a server gets one more row. The footer of a desktop with one collection does not change.

## How This Decision Is Checked

Mechanical:

- `crates/api/ui-tests/shell.test.mjs`: `leftPanel` gives "Sign Out" in server mode and not on the desktop (items 1 and 8). It gives "Switch Collection…" on the desktop with two collections, and not with one collection or on a server (items 4 and 8). The root page of a narrow window gives the order of item 3.
- A Node test of the model of the sheet: one row for each collection, and no button on the open collection (item 5).

Visual (a person, or the main session with screenshots):

- The footer of a server with "Sign Out", in the dark theme, at 1280 px and at 390 px. A selection of "Sign Out" shows the sign-in page.
- The footer of a desktop with two collections, and the sheet "Switch Collection".
