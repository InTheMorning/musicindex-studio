# ADR 0101: A Suggestion Groups The Items Of One Name

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Changes: [ADR 0098](0098-the-people-module.md) item 12, the grouping of the suggestions. Items 13 to 16 stay.
- Uses: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 3 for the sheet "Make a Person".

## Context

- ADR 0098 item 12 groups each credit by its strongest identity: the `npub`, else the name with the information address, else the name.
- In the human test of `0.1.0-rc.9` on 2026-11-01, one artist had three suggestions with the name "HeyCitizen". One had the same Nostr key, and two had the same name. The same payee joined each of the three (task 063c).
- The operator made a person from one suggestion. That person did not get the keysend address of the artist, because that address was in a different suggestion.
- A person is one suggestion for the user. A split by the form of a credit is not a fact that the user can see in the feeds.

## Decision

1. A suggestion groups the credits with one normal name (task 063, Words). Credits with equal normal names join one suggestion. This applies to a credit with an information address, a credit with an `npub` and a credit with neither.
2. When two or more different `npub` values have one normal name, each `npub` makes its own suggestion, because they can be two people. The credits of that name with no `npub` then make one more suggestion.
3. A credit with an `npub` and no name makes a suggestion of its `npub`, as before.
4. A payee joins the suggestion of its normal name. When a name has more than one suggestion (item 2), the payee joins the suggestion with no `npub`. When that suggestion does not exist, the payee joins each suggestion of its name. A payee also joins through the wallet of a payee that joined by name, as before.
5. The identity of a suggestion is `name:{normal name}`, or `npub:{npub}` for a suggestion of item 2 or item 3. A dismissal from before this ADR with a different identity does not hide the new suggestion. Thus a dismissed suggestion can come back one time.
6. The evidence lists `SameNpub` when the suggestion has one `npub`. It lists `SameName` when a payee joined by name, and `SameWallet` when a payee joined through a wallet.
7. Each item that has the normal name of the suggestion starts selected in the sheet "Make a Person". An item of a different name starts not selected (ADR 0099 item 3).

## Alternatives Considered

- Hide the name-only suggestion when a suggestion of an `npub` has the same name. Not chosen: the suggestion of the name with an information address still shows apart.
- Keep ADR 0098 item 12. Not chosen by the operator: one person gave three suggestions in the first human test.
- Merge two `npub` values of one name. Not chosen: two keys with one name can be two people, and the user confirms each link (ADR 0098 item 14).

## Consequences

- One person with one key gives one suggestion with each of its credits and payees.
- Two people with one name and no key give one suggestion. The user clears the items of the other person in the sheet.
- The core tests of the suggestions change for items 1 to 5.

## Acceptance Criteria

Mechanical:

- A core test: three credits of one name give one suggestion. One credit has an `npub`, one has an information address, and one has neither. A payee of that name joins it one time.
- A core test: two different `npub` values with one name give two suggestions, and the credits of that name with no `npub` give a third. A payee of that name joins the third.
- A core test: a credit with only an `npub` gives a suggestion of its `npub`.
- A core test: a dismissal of `name:{normal name}` hides the suggestion until a new item comes.

Human, on `studio.musicindex.org`: the operator sees one suggestion "HeyCitizen" with each credit, payee and key of the artist.
