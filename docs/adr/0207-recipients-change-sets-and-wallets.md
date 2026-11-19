# ADR 0207: Recipients, Change Sets And Wallets

- Status: Accepted
- Date: 2026-11-14
- Owner: operator
- Accepted: 2026-11-14. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0027](../archive/adr/0027-the-recipient-picker.md), The Recipient Picker.
- Supersedes: [ADR 0091](../archive/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), A Change Set Edits One Recipient In Many Feeds.
- Supersedes: [ADR 0095](../archive/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), A Lightning Address And Its Node Key Are One Recipient.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the review and the write. [ADR 0014](0014-collection-in-sqlite.md) for the collection. [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the network policy. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) for the test servers.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0030](0030-an-added-recipient-gets-no-split.md) (no split for an added recipient), [ADR 0215](0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) (the levels), [ADR 0096](0096-add-to-an-artist-or-label.md) (the operation of a level), [ADR 0097](0097-studio-suggests-the-lightning-address-of-a-wallet.md) (the suggested address), [ADR 0138](0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) (the names of an address), [ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) (the one picker), [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) (a move of releases), [ADR 0201](0201-people-opens-before-the-lightning-address-lookups.md) (the lookup route), [ADR 0203](0203-people.md) (People), [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) (the announcements), [ADR 0206](0206-the-shell-and-the-editor-pages.md) (the destinations).
- Changed: 2026-11-15 by the operator, through musicindex ADR 0076 (musicindex `a08dc01`). Item 58 gives a node address the short form of R-ADDRESS-1.
- Revised: 2026-11-15 by operator decision. Items 53 to 60 give the menu of an address row. The operator selected items 53 to 59 on 2026-11-14 and item 60 on 2026-11-15, from drawings. Items 53 to 59 were in ADR 0210, which is deleted.

## Context

- ADR 0027 gave the recipient picker of the value editor. ADR 0091 gave the change set: one operation on one recipient in many feeds, with one review. ADR 0095 made a Lightning address and its node key one wallet.
- ADR 0095 changes ADR 0091. ADR 0098 and ADR 0099 changed all three ADRs, and ADR 0203 now restates them. ADR 0027 had four changes, ADR 0091 had six and ADR 0095 had five. Global `CLAUDE.md` §Consolidate A Changed Decision permits three.
- People replaced the Recipients page. A migration made one person from each saved recipient, and `saved_recipients` has no row after it (ADR 0203 item 7). Thus some rules for a saved recipient have no object. They are ADR 0027 item 5, ADR 0091 items 12 and 20 and ADR 0095 item 9. Items 5, 19, 27 and 42 give the current rule.
- The research note of 2026-10-31 ([`docs/research/2026-10-31-keysend-and-lightning-address-recipients.md`](../research/2026-10-31-keysend-and-lightning-address-recipients.md)) gives the evidence for the wallet. The archived ADR 0095 keeps the corpus counts and the defects that the operator found on 2026-10-31.
- The current ADRs in the Uses lines own their rules. This ADR restates the result at each item, and names the owner.

## Decision

### The Recipient Picker

1. The editor offers a recipient picker for the album and for each track. "Add a recipient" and "Change…" open one sheet ([ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) item 2, [ADR 0203](0203-people.md) item 86).
2. The picker lists "Your people", then "Payees with no person" (ADR 0203 item 86). A person row shows the nickname or the name and the address that it adds, which is the default address. A payee is each recipient of each feed and track that Studio read into the collection.
3. A recipient is identified by its address, its custom key and its custom value. The list has each recipient one time. The name and the type come from the most recent read or save. Two addresses of one wallet are one payee (item 37).
4. The collection keeps the recipients of each feed after each read and each write (`feed_recipients`). A recipient of the collection goes out of the list when no feed of the collection has it.
5. The user keeps a payee in a person of People, not as a saved recipient (ADR 0203 item 7). "Add to People" makes a person from a recipient of the editor ([ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) item 3).
6. The picker can also show the `payment_routes` of a feed or a track in musicindex. The row "Copy from a Feed" opens the musicindex search, with check boxes for many recipients (ADR 0144 item 2).
7. Only the musicindex search selects many recipients. A selection of a person or a payee fills the form. Studio adds each selected recipient after the recipients that the editor has. It does not replace or remove a recipient. An added recipient gets no split, and the user gives its split. A recipient from musicindex keeps its fee mark ([ADR 0030](0030-an-added-recipient-gets-no-split.md) items 1 and 2).

### The Destination And The Pages

8. People is the first destination of the bottom group of the left panel, in the place of "Recipients". The sequence is People, File hosts, Settings, then Lock Studio (ADR 0203 item 36, [ADR 0206](0206-the-shell-and-the-editor-pages.md) item 1).
9. The People page lists each wallet that no person holds in the fold "Payees with no person" (ADR 0203 item 37). The number of feeds of a row comes from the same model of each feed as the feed list of item 10: the draft of the feed, or else its library copy. Thus the two lists give the same number.
10. A row opens the page of the person or of the wallet. Each sheet of an operation lists the feeds with check boxes, below its own fields, and each check box starts selected. A feed row shows the type and the places of the recipient: "Channel", the number of tracks and the number of live items (ADR 0203 item 74). The page has no list of feeds with check boxes (ADR 0203 item 49).
11. The page of a wallet with no person has the section "Change your feeds" (ADR 0203 item 80). Its operations are Change Address, Rename, Use the LNURL Address, Add to All Releases and Remove from Feeds. The page of a person has the operations of ADR 0203 item 26.

### The Operations And Their Scope

12. Change address replaces the address, the custom key, the custom value and the type. Each split and each other attribute stays the same.
13. Rename replaces the `name` attribute. Each other attribute stays the same.
14. Remove deletes the recipient from each value block of its places.
15. The scope of Change address, Rename and Remove is each selected feed of item 10.
16. Add to All Releases adds the recipient to each value block of each selected release of a level. A level is a publisher feed and each feed below it ([ADR 0215](0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md)). The scope has only the releases of the level, not its publisher feeds (ADR 0203 item 26). The user selects the artist or label, then the core lists its releases with check boxes, and each check box starts selected. The user gives a percentage from 1 to 99. The operation comes from [ADR 0096](0096-add-to-an-artist-or-label.md), and its name comes from ADR 0203 item 26.
17. An operation changes each value block of the channel, of each item and of each live item. It does not change a value time split.
18. A publisher feed is in a scope as each other feed, but not in the scope of Add to All Releases (item 16). Studio copies no recipient from a publisher feed to its releases, or from a release to its publisher feed ([ADR 0075](0075-the-publisher-page-has-payments-and-credits.md) item 4).
19. A change set does not change a person, and Discard does not change a person. When the change set of Rename changes a feed, the core also changes the name of the person (ADR 0203 item 29).

### The Splits

20. Add to All Releases sets the split of the new recipient to the percentage p. The core then scales the other non-fee splits of the block, so that their total is 100 − p. Each recipient keeps its relative share. The core rounds each split to an integer, and gives the remainder to the largest split.
21. Remove scales the other non-fee splits of the block, so that their total is the non-fee total before the change. Each recipient keeps its relative share. A block with a non-fee total of 100 keeps a total of 100. The core rounds as item 20 says.
22. A fee split does not change.
23. These calculations are feed rules. They are in the core (ADR 0065).

### The Change Set

24. The core applies an operation to each feed of the scope as one change set. For each feed, the result is one of these outcomes:
    - **Changed.** The core put the change in the draft of the feed.
    - **Not changed.** The core gives the cause. For example, the block has the recipient before the change, or the block has the new address.
    - **Not changed** also applies when the feed has no value block, or when each split of the block is a fee.
    - **Not changed** also applies when the feed has a change of an open change set that is not written. Thus a discard of one change set cannot put back a draft over the change of a different change set.
25. The core puts the change in the open draft of the feed, if the feed has one. If the feed has no draft, the core makes a draft.
26. The collection keeps each change set until each changed feed is written or the user discards the change set. The change set keeps the draft of each feed before the change. Thus the core deletes a change set after the write of its last changed feed. A change set is closed when it has no changed feed that is not written. The core deletes each closed change set at the start of the next change set.
27. "Discard" on a change set puts back the draft of each feed from before the change. A feed that had no draft then has no draft. A feed written after the change does not change. When the user discarded the draft of a feed after the change, that feed also does not change.

### The Review And The Write

28. The review of a change set lists each feed with its outcome. Each changed feed opens its diff. The first changed feed starts open, so the page shows a diff before the first write. The diff of a feed with an earlier draft also shows the earlier edits. The primary button of each sheet reads "Review in N Feeds". The review keeps "Write N Feeds" and "Discard" at its top, and the bar keeps them visible at each scroll position (ADR 0203 item 79).
29. The review also lists each changed feed that Studio cannot write now, with the cause. For example, a read-only feed has no host ([ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md)), or the vault is locked.
30. "Write" writes each feed that the user can write, one at a time, through `pipeline::write_draft` with the ID of the diff of that feed. The review shows the status of each feed during the write.
31. When the write of one feed fails, Studio does not stop the write of the other feeds. A conflict of ADR 0007 item 3 stops only that feed. The draft of a feed that is not written stays, and the change set stays open.
32. Each write announces its feed as each other write does ([ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md)).
33. A show grant does not cover a change set. A change set needs the vault of the user.

### The Wallet

34. A wallet is the node key with the custom key and the custom value that receive a payment. A `node` recipient names its wallet in its attributes. An `lnaddress` recipient names its wallet through the keysend lookup of its address.
35. The core makes the lookup, and keeps the result for each Lightning address of the collection in the table `keysend_lookups`. A result has the node key, the custom pair, the status `found`, `none` or `failed`, and the time of the request. The status `none` is a response with no keysend data, for example a 404 or a status `ERROR`.
36. `GET /recipients` sends no lookup, and it gives the list with the lookups that the collection already holds. The route `POST /recipients/lookups` sends the lookups that are due ([ADR 0201](0201-people-opens-before-the-lightning-address-lookups.md) items 1 and 3). An address is due when it has no result. A `found` or `none` result is due after one day, and a `failed` result after one hour. The route sends the requests at the same time, each with a limit of 5 seconds, and a failed request keeps the previous result. Each request goes through the network policy (ADR 0057) with no user data.
37. Two recipients with the same wallet are one recipient. A Lightning address with no `found` result is a recipient of its own.
38. The key of a wallet is a `RecipientKey`: the node key with the custom pair when the wallet has one, else the Lightning address with its custom pair. The routes `recipients/uses` and `change-sets` take the key of a wallet. The core finds each recipient of the wallet: each `node` recipient with the key, and each `lnaddress` recipient whose result gives the key.

### The Rows Of A Wallet

39. `GET /recipients` gives one entry for each wallet, with the name from the most recent read or save and the key of the wallet. An entry also has the number of feeds that use an address of the wallet, and the list of its addresses. Each address has its type. A Lightning address also has the status of its lookup. The recipient picker lists each address of each entry, so its rows do not change (item 3).
40. A row of a wallet shows the name as its title. The number of feeds is at the right of the title. Below the title, the row has one line for each address of the wallet, the Lightning addresses first:
    - A line reads "LNURL address {address}", "{provider} node · account {value}", or "Node address {key} · account {value}" (ADR 0203 item 83, [ADR 0138](0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) item 3). The provider comes from item 49.
    - The key shows its first 8 and its last 4 characters. A node with no custom value has no account part. The accounts of one shared node differ only in the custom value.
    - Thus a name is not an address. An address is only on a line that names its type.
    - The row is a navigation row to the page of the wallet, not an entry of an editor list. The one summary line of ADR 0028 item 1 does not apply to it.
41. The page of a wallet shows the lines of item 40 below the name, with the full node key. It has no list of feeds. Each sheet of an operation lists the feeds (item 10).

### The Operations On A Wallet

42. Change address, Rename and Remove act on each recipient of the wallet in each selected feed. Change address gives the wallet one address.
43. Add to All Releases adds one address. The form starts with the Lightning address of the wallet when it has one, else with the node key. This start value is an edit aid.
44. The page of a wallet with a node key has the operation "Use the LNURL Address" (ADR 0138 item 3). In each selected feed, it replaces each `node` recipient of the wallet with an `lnaddress` recipient: `type="lnaddress"`, the Lightning address as `address`, and no `customKey` and no `customValue`. The `name`, the `split` and `fee` stay. A feed with no `node` recipient of the wallet is "Not changed".
45. The form of item 44 starts with the Lightning address of the wallet. When the wallet has none, the sheet asks the core for suggestions ([ADR 0097](0097-studio-suggests-the-lightning-address-of-a-wallet.md) items 1 to 5). With no suggestion, the user types an address. The core accepts the address only when a fresh lookup gives the node key and the custom pair of the wallet (ADR 0201 item 7). This also applies to a suggestion (ADR 0097 item 6). Otherwise the core refuses the change set with the cause, and the user can use Change address.
46. When an operation puts an `lnaddress` recipient into a value block with `method="keysend"` or with no method, the core writes `method="lnaddress"` on that block. This is the form of the specification example and of MSP 2.0. A block that the operation does not change keeps its bytes. An edit in the editor does not change the method.
47. Change address and Add to All Releases need a type, and the core refuses an operation with no type. The Type menu is the first field of the recipient fields, and it has no empty choice. In Add to All Releases, the artist or label and the percentage come before the recipient fields (ADR 0203 item 80). Its items are "LNURL Address" and "Node Address" (ADR 0203 item 82). The fields below it follow the type. They start at the member of the wallet with that type, not at the key of the wallet:
    - For a node: "Node address", "Custom key" and "Account". "Account" edits the custom value (ADR 0203 item 82).
    - For a Lightning address: the address field alone.
    - Change address starts at the node key when the wallet has one, else at the Lightning address.
    - Add to All Releases starts at the Lightning address when the wallet has one, else at the node key.
    - A change of the type puts the member of the new type in the fields, when the user did not change the address.
    - Each start value is an edit aid.

### The Editor

48. In the value editor, an `lnaddress` recipient with the status `none` shows the hint "Studio found no payment details for this address at {domain}. Some apps cannot pay it." (ADR 0203 item 84). The status comes from `GET /recipients`. The editor makes no lookup.

### The Provider Of A Node

49. The core names the provider of a node by a domain, for the lines of items 40 and 41. A domain names a node when two or more Lightning addresses of that domain in the collection resolve to the key, with different custom values. `GET /recipients` gives the provider of each `node` address, or null.
50. A table of shared nodes in the core also names a key: the Fountain node `03b6f613e88bd874177c28c6ad83b3baba43c4c656f56be1f8df84669556054b79` as `fountain.fm`, and the Alby node `030a58b8653d32b99200a2334cfe913e51dc7d155aa0116c176657a4f1722677a3` from before Alby Hub as `getalby.com`. An entry goes into the table only with the evidence of the research note: a lookup, or the alias of the node in the network graph.
51. One address that resolves to a node does not name the node, because a `getalby.com` address resolves to a personal Alby Hub node. That node and its address are one wallet (item 37), and the row shows the address. The alias of a node in the network graph is not a name: the Fountain node is `klnd0`.

### A Move Of Releases

52. A change set also holds a move of each release on a file host ([ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 16 to 19). Its Discard also removes the host operations of the move.

### The Menu Of An Address Row

53. On the page of a person, each address row of a wallet has one menu button. The row has no other button.
54. The menu has **Make Default**, **Edit** and **Copy**, in that sequence. The default address ([ADR 0203](0203-people.md) item 10) has no **Make Default**.
55. The menu button of a row has two forms. In the narrow layout it is a round icon button with the Lucide icon `ellipsis`. In the wide layout it is the text button **Actions**.
56. The two forms are one control with one accessible name, which names its row, for example "Actions for hey@getalby.com". The layout selects the form. The label is not the text character "…" (musicindex ADR 0026 R-ELLIPSIS-1, musicindex ADR 0080 R-ACT-2 item 2).
57. On Touch, the target of the icon button has the default size of musicindex ADR 0068 R-TARGET-1. The control is the menu control of [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md).
58. An address row shows its label, the address, and one status line. A node address shows in the short form of musicindex ADR 0076 R-ADDRESS-1, and its hover text and **Copy** give the full address. An LNURL address shows in full. The status line has the tag **Default**, when applicable, and the text of the use count.
59. **Remove Wallet** stays a button of the wallet. It is not in a menu.
60. The address row of a recipient, in the editor of a feed and of a track, is the same row component with the same menu control. Its menu has **Change…**, **Copy** and **Add to People**, in that sequence. It has no **Make Default**, because a recipient of a feed has no default address. **Remove** stays a destructive button below the row, because it removes the recipient and not only the address. The editor does not keep a changed copy of the row of People.

## Alternatives Considered

- Only the recipients of the collection. Rejected by the operator: a recipient that no feed uses, such as a new band member, then cannot be in the list.
- Replace the recipients of the editor with the imported recipients. Rejected by the operator: an import usually adds a guest artist to the recipients that exist.
- Select many feeds in a list, then apply one edit. Not selected by the operator for the first version. With it, a search for each place that pays a person is not easy.
- A release takes the value block of its publisher feed. Not selected. Podcasting 2.0 has no inheritance of a value block, and ADR 0075 item 4 does not let Studio copy one.
- Change only the channel value block. Not selected by the operator. A track value block then keeps the previous recipient.
- Skip a feed with an open draft. Not selected by the operator. The user then must write each open draft before a change set.
- One fixed split for an added recipient, or a split of 0 (ADR 0030), in a change set. Not selected by the operator. A fixed split changes the total of each block, and a split of 0 needs an edit of each feed.
- Keep the split values, and add a split of S × p / (100 − p). Not selected by the operator. The total of the block is then not 100.
- A scope of link groups: two-way, one-way, or not linked. Not selected for the first version.
- A second address on one recipient, as a fallback. Not chosen: the specification has no attribute for it, discussion 576 of the podcast namespace did not decide it, and no app reads it.
- Two value blocks in each feed, one for each method. Not chosen: no corpus feed has two blocks at one level, and no source says how an app selects one.
- A link from the name. Not chosen: a name is not proof. The same person has the name "TJ Doerfel" in one feed and the address as the name in another.
- A setting that stops the lookups. Not chosen: each address is public in a feed, and the request carries no user data.
- The UI groups a flat list of recipients. Not chosen: the number of feeds of a wallet is a union over the feed models, and the UI does not have the models (ADR 0042, ADR 0065).
- An operation that replaces a Lightning address with its node key. Not chosen: Change address covers it, and the node key breaks when the provider changes its node.
- The alias of a node from the Lightning network graph, through an explorer API. Not chosen: the alias of the Fountain node is `klnd0`, and it is one more request that goes through the network policy (ADR 0057).
- A domain from one lookup as the name of a node. Not chosen: a `getalby.com` address resolves to a personal Alby Hub node, and "getalby.com node" would name a node of the user.

## Consequences

- The user changes one recipient in many feeds with one review.
- A change set with many feeds makes many writes and many announcements.
- A recipient that a change set adds gets a split. ADR 0030 continues to apply to the recipient picker of the editor.
- A recipient address is public. The tables `feed_recipients` and `keysend_lookups` contain no secret ([ADR 0003](0003-user-unlocked-secret-vault.md)).
- The core makes HTTPS requests to the domains of the Lightning addresses of the collection. On the server, the private network guard applies.
- The provider of a Lightning address can see that a Studio requested its lookup. It gets no other data.
- The import from musicindex needs the musicindex API. The other parts of the picker work with no network.

## Non-Goals

- Batch edits of other fields, for example OP3 or the switch of ADR 0063.
- A recipient of a value time split.
- A repair of the one-way links.
- A link of two addresses by hand, outside a person. A person can hold more than one wallet (ADR 0203 item 9).
- A Lightning address from the `lud16` of a Nostr profile, outside a person. ADR 0203 items 23 to 25 give the `lud16` suggestion of a person.
- Two value blocks in one channel or item.
- The method of a block that the user changes in the editor.
- A payment by Studio. Studio sends no keysend and requests no invoice.

## Invariants

- The picker does not remove or replace a recipient of the editor.
- The list has each recipient, by address and custom pair, one time.
- An operation does not change a recipient outside its places, or a feed outside its scope.
- An operation does not change a fee split.
- Each write of a change set uses the ID of a diff that the review showed.
- Discard puts back the draft of each feed from before the change, byte for byte.
- No test sends a request to a host outside the computer (ADR 0045).

## How This Decision Is Checked

Mechanical. The tests of the superseded ADRs enforce these items, and this ADR adds no test:

- Core tests for items 12 to 14, 17 and 20 to 22. The test feed has a channel block, an item block, a live item block and a value time split.
- Core tests for items 24 to 27: an open draft gets the change, an outcome gives its cause, and Discard puts back each draft byte for byte.
- A core test: Add to All Releases gives each release of the level, through `child_ids`.
- An API test and a UI model test: the write of a change set writes the other feeds after a conflict in one feed.
- An API test: a wallet in the list of item 9 has the number of feeds of item 10. This also applies to a feed that has only a draft.
- A core test: the lookup parser reads the response of the specification example, with and without `customData`. A response with the status `ERROR` or with no `pubkey` gives `none`. A connection error gives `failed`, and keeps a previous result.
- A core test: a `node` recipient and an `lnaddress` recipient with the same result are one entry of the recipients, with the union of their feeds. An `lnaddress` recipient with no result is its own entry.
- A core test: Rename, Remove and Change address act on each recipient of a wallet in one feed.
- A core test: Use the LNURL Address writes the attributes of item 44 and the method of item 46. A block with no `node` recipient of the wallet keeps its bytes. A Lightning address whose result gives a different key is refused.
- A core test: Change address and Add to All Releases with no type are refused.
- A core test: the provider of a node comes from the table, or from two addresses of one domain with different custom values. One address of a domain gives no provider.
- Node tests: the Type menu has no empty choice, and it is before the address field. The start values of a form come from the members of the wallet. The lines of a row of item 40 for a wallet with two addresses, a provider node and a node with no custom value.
- The corpus tests pass with no change.
- The UI has no copy of the rules of items 17 and 20 to 22. A test cannot tell a feed rule from an edit aid. Thus the review of each UI packet examines each new UI function that changes the model (ADR 0065).

Visual: no new visual check. The archived ADR 0027, ADR 0091 and ADR 0095 keep their visual checks.

The human check of item 58 passed on 2026-11-15, on commit `9bd3efd`. The hover text of the short address gives the full address. At about 1200 px, the address row shows the same short form as at 390 px.

Human, on `studio.musicindex.org`: the human test of ADR 0095 is open.

- A Fountain address and an Alby address each join their node recipient in one entry. A Strike address stays its own entry.
- Use the LNURL Address writes one feed. The hosted file has `type="lnaddress"` and `method="lnaddress"`.

## Item Map

The map gives the item of this ADR for each item of a superseded ADR.

| Superseded item | New item |
| --- | --- |
| ADR 0027 items 1 to 7 | 1 to 7 |
| ADR 0091 items 1 to 4 | 8 to 11 |
| ADR 0091 items 5 to 8 | 12 to 15 |
| ADR 0091 item 9 | 16 |
| ADR 0091 items 10 and 11 | 17 and 18 |
| ADR 0091 item 12 | 19 |
| ADR 0091 items 13 to 16 | 20 to 23 |
| ADR 0091 items 17 to 20 | 24 to 27 |
| ADR 0091 items 21 to 26 | 28 to 33 |
| ADR 0095 items 1 and 2 | 34 and 35 |
| ADR 0095 item 3 | 36 |
| ADR 0095 items 4 and 5 | 37 and 38 |
| ADR 0095 items 6 to 8 | 39 to 41 |
| ADR 0095 items 9 to 18 | 42 to 51 |
| ADR 0027, ADR 0091 and ADR 0095 "Invariants" | "Invariants" and "How This Decision Is Checked" |
| ADR 0027, ADR 0091 and ADR 0095 "Verification" | "How This Decision Is Checked" |
