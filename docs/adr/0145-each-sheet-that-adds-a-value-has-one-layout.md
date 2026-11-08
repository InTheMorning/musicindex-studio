# ADR 0145: Each Sheet That Adds A Value Has One Layout

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire and from the mockup in `.studio-dev/mockup-pickers/`. The operator selected one selection that fills the form, a row with a heading after the form, lists that fold, and all four sheets. The operator accepted items 9 to 13 from a list of proposed changes, and asked for the share on each row and for the name of a large node. The operator selected one search in all sheets, from a recommendation. The operator asked for the row in the credit picker and the Nostr picker.
- Technical choices: the limit of 6 rows in item 2 and the two routes of item 3 are choices of the planning model.
- Changes: [ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) item 2, the position of the row of musicindex. [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) items 20 and 21, "Pick From Your People". [ADR 0137](0137-the-people-list-fills-nostr-keys-and-open-entries.md) items 4 to 9, "Fill from a Person".
- Uses: [ADR 0123](0123-one-type-of-value-has-one-picker.md) and [ADR 0141](0141-the-form-of-add-a-wallet-comes-first.md) item 1, the form stays in view. [ADR 0117](0117-the-page-of-a-person-finds-values-on-musicindex.md) for the search of artist and label feeds. [ADR 0098](0098-the-people-module.md), a value from a person is a start value.
- Source: [findings of local use, finding 29](../reviews/2026-11-06-local-use-findings.md)

## Context

"Add a Recipient" had a search of musicindex, and "Add a Wallet" did not. The credits and the support links had two rows, "Add" and "Pick from your people", and an open credit had "Fill from a Person". So one job had a different path in each location.

The API of musicindex has few values for these sheets. `../musicindex/api.json` names no support link. ADR 0117 counted credits and Nostr keys on 0 of 176 publisher feeds.

## Decision

1. "Add a Recipient", "Add a Wallet", "Add a Credit" and "Add a Support Link" have one layout: the form, then a row under the heading "From Music Index", then the lists of values. `PickerLayout` of `crates/api/ui/pickers.js` gives it.
2. A list folds, and it shows its count. A list with 6 rows or more starts closed.
3. "Find on Music Index" and "Copy from a Feed" use one search: the artist and label feeds first, then the other feeds. `SearchResults` of `crates/api/ui/pickers.js` shows them. In "Find on Music Index", one selection fills the form, and the user then adds the values of the form. Two routes with no person give the search and the values of a feed: `GET /musicindex/search` and `GET /musicindex/values/{guid}`.
4. A feed with many payment addresses asks for one address. A feed with no value for the sheet gives a text, and the form does not change.
5. In "Add a Credit", an artist or label feed fills the name, the picture address, the information address and the Nostr key. Each other feed fills only the name of its artist and its picture, because its title is not the name of a person. In "Add a Support Link", a feed fills the address from its website.
6. In "Add a Recipient", the row is "Copy from a Feed", which selects many recipients and adds many rows. In "Change the Address", it is "Find on Music Index".
7. A selection of a row of a list fills the form. It adds nothing and does not close the sheet. At 760 px or less, the sheet scrolls to the form.
8. The credits and the support links have one row each: "Add a credit" and "Add a support link". An open credit has no "Fill from a Person".

### The Other Sheets

14. On the page of a person, "Add a Credit Name" and "Add a Support Link" have this layout too.
15. The sheet "Find a Nostr profile" keeps its own search, and it has the row "Find on Music Index" under the same heading. A selection uses the Nostr key of the feed, and a feed with no key gives a text. Its list "Your people" folds.

### The Sheet "Copy From A Feed"

9. A row names a known shared node with its provider, for example "fountain.fm node", and it gives the account. The core gives the provider from the table and the lookups of ADR 0095. The name is not in the feed.
10. A row that the list has already shows the text "Already in this list" and has no check box.
11. A row shows the share of the recipient in its source, for information. A new recipient still gets no split (ADR 0030).
12. A row shows a short address, on one line. The search runs during the typing, and its field stays in view. A result row of a feed shows its picture, its type word and its artist.
13. The tracks of a feed are a list that folds, above the recipients. The footer shows the number of selections only when it is more than 0.

## Alternatives Considered

- Only "Add a Recipient" and "Add a Wallet". Not chosen by the operator.
- Check boxes in "Add a Wallet", which add many wallets. Not chosen by the operator: no form shows the values before the add.
- No row in "Add a Support Link", because musicindex has no support link. Not chosen: the planning model recommended the row so that the sheets stay the same, and the operator agreed.

- A search of only artist and label feeds in "Find on Music Index". Built first, then not chosen by the operator: a wallet is often only on an album, and the sheets gave different results for one text.

## Consequences

- A new credit or support link starts from a sheet and not from an empty open row.
- A selection replaces each value that it gives, also a typed value. The user sees the result in the form before the add.
- One search sends more requests to musicindex, because it searches two kinds of feed.
- The sheet "Find on Music Index" on the page of a person (ADR 0117) keeps its own search of artist and label feeds, because it fills the fields of one person from that person's feed.
- "Add to People" appears only in the sheet of a recipient, because the other sheets do not hold a payee.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines the start of a fold and the values that each sheet takes from a feed. `crates/api/src/tests.rs` examines the two routes and the provider of a route.

Visual: the planning model drives each sheet against a local musicindex stub at 1280 px. It also drives "Add a Credit" at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
