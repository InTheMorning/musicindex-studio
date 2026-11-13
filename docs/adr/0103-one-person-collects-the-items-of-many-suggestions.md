# ADR 0103: One Person Collects The Items Of Many Suggestions

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the mockups in `.studio-dev/mockup-101/site/merge.html`.
- Changes: [ADR 0098](0098-the-people-module.md) items 12 and 14, the grouping of the suggestions and the make of a person. [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) items 3 and 4, the sheet "Make a Person" and the page of a person.
- Supersedes: [ADR 0101](../archive/adr/0101-a-suggestion-groups-the-items-of-one-name.md). Items 1 to 7 of ADR 0101 are items 1 to 7 of this ADR.
- Uses: [ADR 0102](0102-delete-a-person-changes-no-feed.md) for "Delete Person", and [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tiers.
- Changed by: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md). Items 8 and 9: one sheet "Check: {name}" with the menu "Person" replaces the sheets "Make a Person" and "Add to a Person".
- Changed by: [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md). Item 14: "Delete Person" is the first button of its row, before "Merge with a Person".

## Context

- The operator made the People module so that the scattered identities of one person become one person with few steps.
- In the human test of `0.1.0-rc.9` on 2026-11-01, one artist gave three suggestions "HeyCitizen". A suggestion could only make a new person. Studio had no way to add items to a person that exists, and no merge of two people. The operator said that a merge of identities was "the exact opposite of why i built this."
- The same test found the Nostr key of a different account, "City Radio", in a credit of the artist. "Make a Person" showed only the short key, with no profile name.
- The operator accepted the mockups of `.studio-dev/mockup-101/site/merge.html` on 2026-11-01. The operator selected the three ways to collect a person, and the warning for a key with a different profile name.

## Decision

### The Grouping Of The Suggestions

1. A suggestion groups the credits with one normal name (task 063, Words). Credits with equal normal names join one suggestion. This applies to a credit with an information address, a credit with an `npub` and a credit with neither.
2. When two or more different `npub` values have one normal name, each `npub` makes its own suggestion, because they can be two people. The credits of that name with no `npub` then make one more suggestion.
3. A credit with an `npub` and no name makes a suggestion of its `npub`.
4. A payee joins the suggestion of its normal name. When a name has more than one suggestion (item 2), the payee joins the suggestion with no `npub`. When that suggestion does not exist, the payee joins each suggestion of its name. A payee also joins through the wallet of a payee that joined by name.
5. The identity of a suggestion is `name:{normal name}`, or `npub:{npub}` for a suggestion of item 2 or item 3. A dismissal from before this ADR with a different identity does not hide the new suggestion.
6. The evidence lists `SameNpub` when the suggestion has one `npub`. It lists `SameName` when a payee joined by name, and `SameWallet` when a payee joined through a wallet.
7. Each item that has the normal name of the suggestion starts selected in the sheet "Make a Person". An item of a different name starts not selected (ADR 0099 item 3).

### Add To A Person

8. The sheet "Make a Person" gets the button "Add to a Person" (action), between "Not This Person" and "Make the Person". It opens the sheet "Add to a Person: {name}", with the select menu "Person".
9. The sheet lists each item of the suggestion. An item that the person has already shows "Already in {person}" and has no check box. Each other item shows "New" and starts with the selection of item 7. The primary button reads "Add 1 Item" or "Add N Items".
10. The core adds the selected items to the person in one transaction, as `make_person` adds them. A conflict of one item adds no item. The core changes no name, picture or default address of the person.

### More From Your Feeds

11. The page of a person gets the section "More from your feeds", with the tier `content`, before "Payment". It lists each item of the feeds that is not a link of a person and that matches this person:
    - a credit whose normal name is the name of the person or one of its credit names,
    - a credit whose `npub` is a key of the person,
    - a payee whose normal name is the name of the person or one of its credit names,
    - a payee whose wallet key is the wallet key of a wallet of the person.
12. Each row gives the item, its feeds and the cause of the match, and the button "Add" (action). "Add" adds that one item, as item 10 does.
13. The section uses only the data of the collection: the index of the credits, the index of the recipients and the stored lookups. It makes no relay request.

### Merge People

14. The last section of the page of a person (ADR 0102 item 1) gets the button "Merge with a Person" (normal), before "Delete Person". It opens the sheet "Merge with {name}", with the select menu "Person to merge", the links of the selected person, and the result.
15. The core moves each link of the other person to this person, then deletes the other person, in one transaction. The name, the picture, the usual role, the information address and the default address of this person stay. A change set of the other person gets this person.
16. A merge changes no feed.

### The Profiles Of The Keys

17. A Nostr key shows the profile name and the picture of its newest kind 0, and a copy button with the full `npub`. This applies in "Make a Person", in "Add to a Person", in "More from your feeds" and on the page of a person.
18. Some keys have a profile name that does not contain the normal name of the person or of the suggestion. Such a key shows the warning "The profile name is not "{name}". Check this key." in the error color. In "Make a Person" and "Add to a Person" it starts not selected. A key with no profile shows no warning.

## Alternatives Considered

- ADR 0101 only, one suggestion for each name. Not chosen: it does not join two names of one person, and it gives no way to add items to a person that exists.
- "Add to a Person" and "More from your feeds" with no merge. Not chosen by the operator: two people that exist, as `0.1.0-rc.9` can make them, then need a delete and new adds.
- Hide a key with a different profile name. Not chosen by the operator: a band or a label can have a profile name that is not the name of the person.
- A match by the profile name of a key in "More from your feeds". Not chosen: it needs relay requests for each key of each feed. The name of the credit that holds the key gives the same match with no request.

## Consequences

- The core gets the add of items to a person, the items that match a person, and the merge of two people. The API gets their routes.
- The UI resolves each shown key through the lookup relays, and shows the warning of item 18. The warning is a display aid, not a feed rule (ADR 0065).
- Task 067 gives the work.

## Acceptance Criteria

Mechanical:

- The core tests of the acceptance criteria of ADR 0101, items 1 to 7 of this ADR.
- A core test: an add to a person adds only the selected new items, keeps the name and the default address, and adds nothing on a conflict.
- A core test: the items of "More from your feeds" match by name, credit name, `npub` and wallet key, and leave out each link of a person.
- A core test: a merge moves each link, keeps the fields of this person, gives the change sets of the other person to this person, and deletes the other person.
- A Node test: the warning of item 18 for a profile name that differs, and no warning with no profile.

Visual: the four screens of the mockups, in the two themes, at 1100 px and at 390 px with touch.

Human, on `studio.musicindex.org`: the operator collects "HeyCitizen" into one person with the real Nostr key, the Alby wallet with its keysend form, and both credits. The key of "City Radio" shows its warning, and the operator removes it.
