# ADR 0111: The Page Of A Person Checks The Feeds

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from the mockups in `.studio-dev/mockup-people-2/site/`.
- Amended: 2026-11-02 by operator decision. Item 15: the change of the payees of the name reads "Replace other addresses named "{name}"". The old label said that a payee pays a wallet, and the operator did not understand it.
- Changes: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) items 3 to 5 and 7. These are the sheet of a suggestion, the page of a person, "Where used" and the page of a wallet with no person. [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md) items 8 and 9, "Add to a Person". [ADR 0098](0098-the-people-module.md) item 20, with one more operation.
- Changed by: [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md). Item 2: the page keeps pending changes until Save. [ADR 0118](0118-member-of-offers-a-credit-on-the-artist-feed.md). Item 3: the hint of "Member of". [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md). Item 3: the section gets a list of links, when its code is complete.
- Uses: [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) for the change set, [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) for the wallets, [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tiers, and [ADR 0112](0112-a-credit-role-comes-from-a-list.md) for the roles.

## Context

- The operator tested the People module of `0.1.0-rc.10` on 2026-11-02, and found these gaps:
  - The page of a person has no "Save Person". A field saves when the user leaves it, and nothing shows that the save occurred.
  - Four buttons that change many feeds are at the top of the page: Use This Address, Rename, Add to All Releases and Remove from Feeds. The user seldom needs them.
  - An item of a suggestion goes to a person that exists only through a second button of the sheet "Make a Person". The operator made a new person first, and then merged it.
  - "Member of" does not say what it does, or which "HeyCitizen" it names.
  - A payee with no person cannot go to a person.
  - Studio does not show how the details of a person go into the feeds.
- The operator kept the HeyCitizen feeds with the Nostr key of a different account. The operator wants Studio to show that each feed has the correct key and Lightning node address. Studio then corrects them in one pass.
- The core has the facts for that check: the index of the credits (ADR 0098 item 10), the index of the recipients, the wallets of the person and the keysend lookups.
- The mockups are in `.studio-dev/mockup-people-2/site/`. Git ignores that folder. The operator selected these choices:
  - the save when the user leaves a field, with a "Saved" status
  - the section "Artist or label"
  - one check page, and no separate "Update the Credits"
  - the role list of ADR 0112

## Decision

### The Page Of A Person

1. The page of a person shows its sections in this sequence:
    - Identity: the picture, the name, the usual role, the artist or label with its type dot and its role, and the Nostr key.
    - Identity: the navigation row "Check your feeds" (item 9).
    - Content: "More from your feeds" (ADR 0103 item 11), then "Payment".
    - Details: "Credits", then "Support links", then "Artist or label".
    - Setup: "Change your feeds" (item 4).
    - Limits: "Merge with a Person" and "Delete Person".
2. A field of the page saves when the user leaves it. After a save, the field shows the status "Saved". After a failure, it shows the error. The page has no "Save Person" button, because each sheet of the page saves its own item.
3. The section "Artist or label" has the select menu "Member of", with each publisher feed and its first role, for example "HeyCitizen · Artist". Its hint is "The artist or label that this person is in. Studio changes no feed." The membership of ADR 0098 item 6 does not change.
4. The section "Change your feeds" holds one navigation row for each operation of ADR 0098 item 20. Each row has one line that says what the operation does. Its note is "Each operation shows the feeds and a review before Studio writes." The page has no operation button at its top.

### The Sheet Of A Suggestion

5. A suggestion row opens the sheet "Check: {name}". It replaces the sheets "Make a Person" and "Add to a Person". Its first field is the select menu "Person", with "A new person" and each person.
6. When a person has the normal name of the suggestion, or one of its credit names, the menu starts on that person. Else it starts on "A new person".
7. With a person in the menu, the sheet lists the items with the rules of ADR 0103 item 9. The primary button reads "Add 1 Item" or "Add N Items". With "A new person", the sheet has the fields "Name" and "Member of", and the primary button reads "Make the Person". "Not This Person" stays.

### Payees With No Person

8. The page of a wallet with no person gets the section "Person", with the navigation row "Add to a Person". It opens a sheet with the menu "Person", as item 5 gives. The operations of the wallet move to a section "Change your feeds", as item 4 gives. The sheet "Add a Wallet" of the page of a person lists the payees with no person first, then the form for a different address.

### Check Your Feeds

9. The page "Check your feeds" replaces the page "Where used" (ADR 0099 item 5). The core compares each feed of the person with the person. It gives each feed a state and a list of findings:
    - A credit of the person has no Nostr key, or a key that is not a key of the person. The finding names the key, and the UI shows its profile name.
    - A credit of the person has a picture or an information address that differs from the value of the person. The core makes this check only when the person has the value.
    - A credit of the person has no role. Apps then show the credit as "Host".
    - A recipient of a wallet of the person does not use the default address. This finding is neutral, because the address pays the same wallet.
    - A recipient has the normal name of the person or of a credit name, but its wallet is not a wallet of the person.
10. A credit of the person is a credit with the identity of a credit name of the person (ADR 0098 item 6). A credit with the normal name of the person or of a credit name is also a credit of the person, whatever its key. Thus Studio finds a credit with a wrong key.
11. The page groups the feeds as "Needs a correction", "Not your default address" and "Correct". A finding of item 9, except the neutral one, shows in the error color. A row opens the feed in the editor.
12. The navigation row on the page of a person shows "N feeds need a correction" in the error color, or "Each feed is correct".
13. The check uses only the data of the collection. It makes no request to a relay or to a wallet. The UI resolves the profile names of the shown keys, as ADR 0103 item 17 gives.

### Correct The Feeds

14. The page has the button "Correct N Feeds" (primary). It opens the sheet "Correct the Feeds: {name}". The sheet lists the changes with check boxes, then each feed with its changes. The primary button reads "Review in N Feeds".
15. The changes are:
    - "Nostr key": each credit of the person gets the key of the person. A person with more than one key gets the menu "Key".
    - "Payment address": each recipient of a wallet of the person uses the default address, as "Use this address" does.
    - "Picture and information address": each credit of the person gets the values of the person.
    - "Role": each credit of the person with no role gets the usual role and its group.
    - "Replace other addresses named "{name}"": each payee of the name gets the default address. The line gives the number of these payments and the default address. It ends with "Select this only when that address is yours too." This change starts not selected, because it changes who gets the payment.
16. The correction is the operation "Correct" of the change set of ADR 0098 item 20. The core applies it, and each feed writes through `pipeline::write_draft` with the ID of its own diff. A feed with no change after the selected corrections is not in the list.
17. The operation keeps the role of each credit that has a role, because one person can have a different role in each release.

## Alternatives Considered

- An edit mode with "Save" and "Cancel", as in iOS Contacts. Not chosen by the operator: each sheet of the page saves its own item already, so a save button would cover only three fields.
- A separate operation "Update the Credits" for the picture, the information address and the key. Not chosen by the operator: the check page compares the same values and gives one correction.
- Remove "Member of". Not chosen by the operator: the membership can be useful, and a clear label and hint make it clear.
- Two buttons on each suggestion row, "Make a Person" and "Add to a Person". Not chosen: two buttons in a row of a list do not follow ADR 0028. One sheet with a menu gives the same choice.
- Correct each feed from its row on the check page. Not chosen: one sheet with all the feeds and one review fits the change sets of ADR 0091.

## Consequences

- The core gets the check of item 9 and the operation "Correct". The API gets their routes. Both are feed rules in the core (ADR 0065).
- The UI gets the new page sequence, the sheet "Check: {name}", the page "Check your feeds" and the sheet "Correct the Feeds".
- ADR 0099 items 3 to 5 and 7, and ADR 0103 items 8 and 9, change as this ADR gives.
- A task file gives the work.

## Acceptance Criteria

Mechanical:

- A core test: the check finds each finding of item 9. These are a credit with a wrong key, found by its name, and a credit with no key. They also include a picture that differs and a credit with no role. The last two are a recipient that does not use the default address, and a payee of the name with a different wallet.
- A core test: the operation "Correct" applies only the selected changes, keeps the role of a credit that has one, and leaves out a feed with no change.
- A core test: the sheet of a suggestion starts on a person with the same normal name, and an add keeps the rules of ADR 0103 item 10.
- A Node test: the page sequence of the person page, and the status "Saved" after a field save.
- The four commit checks pass.

Visual: the screens of the mockups, in the two themes, at 1100 px and at 390 px with touch. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human, on `studio.musicindex.org`:

- The operator opens "Check your feeds" for HeyCitizen. The page shows each credit with the key of City Radio.
- "Correct N Feeds" gives each credit the correct key and each payment the node address.
- After the write, the page shows each feed as correct.
