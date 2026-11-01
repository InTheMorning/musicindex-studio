# ADR 0098: The People Module

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Changes: [ADR 0027](0027-the-recipient-picker.md) items 2 and 5 (the saved recipients become people). [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md), its non-goal for `lud16`. [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md), its non-goal "A link of two addresses by hand", for the wallets of one person.
- Uses: [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) for the change sets, [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules, [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the network policy, and [ADR 0014](0014-collection-in-sqlite.md) for the collection.
- Screens: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md).

## Context

- The [People module design note](../research/2026-11-01-people-module-design-note.md) gives the evidence, the jobs J1 to J11 and the answers of the operator of 2026-11-01.
- Studio keeps no record of a person. A payee, a credit, a Nostr key and a support link exist only in the feeds. Only `saved_recipients` and `keysend_lookups` exist outside the feeds.
- One person can have more than one wallet. On 2026-11-01 the operator gave an example: one artist has a Fountain wallet and a wallet on a home node, with four addresses in total. No lookup proves that two nodes are of one person.
- Stophammer [ADR 0082](../../../stophammer/docs/adr/0082-a-funding-link-is-a-source-link.md) keeps each `podcast:funding` tag. The operator asked that each app of the suite includes the support links.
- The contributor database research of 2026-10-31 in `../musicindex-research` gives three recommendations. They are an identity rule for a known person, the raw role text, and the `lud16` of a signed Nostr profile.
- The operator accepted the mockups in `.studio-dev/mockup-098/site/people.html` on 2026-11-01.

## Decision

### The Module

1. The core gets the module `people`. It keeps the people of the collection of one user. It has no feed rule and no HTTP server code ([ADR 0002](0002-rust-core-json-api-browser-ui.md)).
2. `people` writes no feed. A change of a feed from a person is a change set of ADR 0091. Each feed writes through `pipeline::write_draft` with the ID of its own diff.
3. A value from a person is a start value in a feed. The feed keeps its own copy, so a change of a person does not change a feed. This is an edit aid (ADR 0065).
4. The module serves the desktop shell and the server shell. On the server, each user folder has its own collection, and thus its own people ([ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).

### A Person

5. A person has a name, a usual role, a picture address and an information address. The usual role is raw text, for example "Vocals".
6. A person has these links. The user makes each link:
    - Wallets. A wallet keeps the identity of ADR 0095 item 1, and the keysend lookup links its addresses. A wallet is in a maximum of one person.
    - Nostr keys, in the `npub` form.
    - Credit names. A credit name is an identity of item 12: an `npub`, else a name with an information address, else a name. A credit of a feed with that identity is a credit of the person.
    - Support links, each with a label and a URL.
    - Publisher feeds of which the person is a member.
7. A person has a default address. It is one address of one of its wallets. The first address of the first wallet is the default until the user selects a different address.
8. An artist or a label is its publisher feed, not a person. A person can be a member of a publisher feed (item 6).

### The Sources Of A Person

9. The collection keeps the people and their links in new tables. A new migration adds them. The tables hold no secret.
10. The core keeps an index of the credits of each feed, as `feed_recipients` keeps the recipients (ADR 0027 item 4). It has the name, the role, the group, the information address and the `npub` of each credit. The core refreshes it after each read and each write.
11. The migration makes one person from each saved recipient, with its name and its wallet. After the migration, `saved_recipients` has no row.

### Suggestions

12. The core suggests people from the index of the recipients and the index of the credits. A suggestion groups the items with one identity: the `npub` first, else the name with the information address, else the name. A recipient joins a suggestion when it has the identity name. A recipient also joins when its wallet is the wallet of a recipient of the suggestion.
13. A suggestion gives the number of feeds with a credit and with a payee, and the evidence of each item: the same name, the same `npub`, or the same wallet through the keysend lookup.
14. The core makes no link without the user. The user selects the items of a suggestion that are the person, and the core then makes the person with those links.
15. The user can dismiss a suggestion. The core keeps the dismissal, and the suggestion does not come back. A new item with the same identity gives a new suggestion.
16. A suggestion is not given for an item that is a link of a person already.

### The Lightning Address Of A Nostr Profile

17. For each Nostr key of a person, the core reads the `lud16` of the newest kind 0. It uses the lookup relays of ADR 0074 item 12. It then makes the keysend lookup of ADR 0095 item 2.
18. The address is a suggestion on the page of the person, with the result of the lookup. The user adds it to a wallet. The collection keeps no lookup of a suggestion that the user did not add, as ADR 0097 item 3 says for a guess.
19. Each request follows the network policy of the shell (ADR 0057). This changes the non-goal of ADR 0074 for `lud16`.

### Operations Of A Person

20. The change set of ADR 0091 gets the operations of a person. The core applies each one to the selected feeds:
    - Use this address: each recipient of each wallet of the person becomes the selected address. The `name`, the `split` and `fee` stay. Two recipients of the person in one block then merge into one recipient, with the sum of the two splits. A fee recipient and a recipient with no fee do not merge.
    - Rename: the new name replaces the `name` of each recipient of the person, the name of each credit of the person, or both. The user selects them.
    - Add to all releases: the core adds items to each release of a publisher feed. An item is the default address with a percentage, a credit with the usual role, or a support link. The user selects one or more items. A release that has the item already is "Not changed" for that item.
    - Remove: the core removes each recipient of the person, each credit of the person, or both. The splits follow ADR 0091 item 14.
21. When an operation puts an `lnaddress` recipient into a block, the core writes the method of ADR 0095 item 13.
22. These operations are feed rules. They are in the core (ADR 0065).
23. A change set does not change a person. Discard does not change a person. When the change set of Rename changes a feed, the core also changes the name of the person. ADR 0091 item 12 says the same for a saved recipient.

### The Export

24. The route `GET /people/export` gives a JSON file of the people. Each link has its evidence: the statement of the user, the keysend lookup or a Nostr signature, and the time of the link.
25. The export holds no secret. An import is not a part of this decision.

## Alternatives Considered

- Contacts on the Recipients page. Not chosen: the page then is a list of payees and a tool for batch edits, and credits, keys and support links stay separate. The mockups of 2026-11-01 showed the crowded screens.
- A shared service of the suite. Not chosen now: it needs accounts, sync and a data license. The export keeps it possible.
- An artist or a label as a person. Not chosen: a publisher feed proves its releases, and the stophammer index proves that link from both sides.
- An automatic link by `npub`. Not chosen: the user confirms each link, also for a strong sign.
- A link by name with no confirmation. Not chosen: a name is not proof (ADR 0095).
- One credit entry with many roles. Not chosen for the first version: the feed output is the same, and one entry for each role exists now.

## Consequences

- Each feed edit that starts from a person needs only one change set, also for a person with more than one wallet.
- A new migration adds the tables of the people and the index of the credits.
- The core makes relay requests and keysend lookups for the `lud16` of the keys of each person.
- The recipient picker and the credits editor read from `people` (ADR 0099).
- A support link from a person goes into a feed only through an action of the user.

## Non-Goals

- A link of a person across the collections of two users.
- An import of people from a file or from a different application.
- One credit entry with many roles.
- A display rule for the support links in musicindex.org and v4vmm. Each of those apps needs its own decision, and a shared rule needs a record in `../musicindex`.

## Acceptance Criteria

Mechanical:

- A core test: the migration makes one person from each saved recipient, with its wallet.
- A core test: a wallet is in a maximum of one person.
- A core test: a suggestion groups a credit and a recipient by `npub`, else by name with information address, else by name. It gives the counts and the evidence. A dismissed suggestion does not come back. An item of a person is not in a suggestion.
- A core test: the core makes no person and no link from a suggestion without a request of the user.
- A core test: Use this address changes each recipient of each wallet of the person. It keeps `name`, `split` and `fee`. It merges two recipients of one block, with the sum of their splits. It writes the method of ADR 0095 item 13.
- A core test: Rename changes only the selected names. Remove removes only the selected items, and scales the splits.
- A core test: Add to all releases adds each selected item to each release that does not have it.
- A core test: a change set and its Discard do not change a person. Rename changes the person when a feed changed.
- A core test: the `lud16` of a profile is a suggestion only when its lookup gives `found`, and the collection keeps no lookup of it.
- An API test: the export gives each person with the evidence of each link, and no secret.
- The corpus tests pass with no change.
- No test sends a request to a host outside the computer ([ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md)).

Human, on `studio.musicindex.org`: the operator makes a person from a suggestion. The person has a Fountain wallet and a second wallet. Use this address changes one feed from the Fountain node to the second wallet. The feed on its file host has the expected recipient and method.
