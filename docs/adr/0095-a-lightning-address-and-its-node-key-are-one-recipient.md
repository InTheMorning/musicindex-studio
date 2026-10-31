# ADR 0095: A Lightning Address And Its Node Key Are One Recipient

- Status: Accepted
- Date: 2026-10-31
- Owner: operator
- Accepted: 2026-10-31 by the operator.
- Changes: [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 2 to 8 and 12. An operation acts on each address of one wallet, and the page of a recipient has a fifth operation.
- Uses: [ADR 0027](0027-the-recipient-picker.md) for the identity of a recipient, and [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the network policy. Also [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules, and [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) for the test servers.

## Context

- The [research note of 2026-10-31](../research/2026-10-31-keysend-and-lightning-address-recipients.md) gives the evidence for this record.
- In the podcast namespace, a `podcast:valueRecipient` has the type `node` or `lnaddress`. For `lnaddress`, an app gets the node key and the custom data from `https://<domain>/.well-known/keysend/<name>` before it pays. The specification has no attribute that gives one recipient two addresses.
- Fountain and Alby publish the keysend lookup. Of 30 Lightning addresses in the corpus, 23 gave keysend data. Strike, Wallet of Satoshi and a custom domain gave none.
- Four persons of the corpus are a `node` recipient in one feed and an `lnaddress` recipient in a different feed. The node key and the custom data are the same. Studio shows each of them as two recipients, because ADR 0027 item 3 identifies a recipient by its address with its custom pair. The user then must make two change sets.
- Five corpus feeds mix the two recipient types in one value block, in both directions of `method`. Thus an app cannot select the recipients by the method alone.
- The operation form of task 054c has the choice "Keep the type" in its Type menu. The core replaces the type (ADR 0091 item 5), so the empty choice removes the `type` attribute. The operator found this on 2026-10-31.
- The operator asked for this decision on 2026-10-31, after the research.
- On 2026-10-31, on `0.1.0-rc.6`, the operator found two defects. A row that showed the name and the types did not tell a node recipient with a name such as "deltaog@getalby.com" from a Lightning address. The form of Change address started at the key of the wallet. For a Lightning address with a `found` lookup, that key is a node key that no feed has. The form showed that key with the type `lnaddress`. Items 7 and 14 changed on that day.

## Decision

### The Wallet

1. A wallet is the node key with the custom key and the custom value that receive a payment. A `node` recipient names its wallet in its attributes. An `lnaddress` recipient names its wallet through the keysend lookup of its address.
2. The core makes the lookup, and keeps the result for each Lightning address of the collection in the table `keysend_lookups`. A result has the node key, the custom pair, the status `found`, `none` or `failed`, and the time of the request. The status `none` is a response with no keysend data, for example a 404 or a status `ERROR`.
3. The core requests the lookups when it lists the recipients. It requests each address that has no result, or a result older than one day. It sends the requests at the same time, with a limit of 5 seconds for each. A request that fails keeps the previous result, when there is one. Each request goes through the network policy (ADR 0057), and it carries no user data.
4. Two recipients with the same wallet are one recipient on the Recipients page. A Lightning address with no `found` result is a recipient of its own, as before this record.
5. The key of a wallet is a `RecipientKey`: the node key with the custom pair when the wallet has one, else the Lightning address with its custom pair. The routes `recipients/uses` and `change-sets` take the key of a wallet. The core finds each recipient of the wallet: each `node` recipient with the key, and each `lnaddress` recipient whose result gives the key.

### The Recipients Page

6. `GET /recipients` gives one entry for each wallet, with the name from the most recent read or save and the key of the wallet. An entry also has the number of feeds that use an address of the wallet, and the list of its addresses. Each address has its type, and the ID of its saved recipient when the user saved it. A Lightning address also has the status of its lookup. This changes ADR 0091 item 2. The recipient picker lists each address of each entry, so its rows do not change (ADR 0027 item 3).
7. A row of the Recipients page shows the name, each address of the wallet and the number of feeds. A Lightning address is before a node key, and it shows in full. A node key shows its first 8 and its last 4 characters, then its custom value when it has one, for example "TJ Doerfel · tj@getalby.com · 02ab12cd…24bf · 01abc · 7 feeds". The accounts of one shared node differ only in the custom value. The row shows no type word, because the shape of each address shows its type.
8. The page of a recipient shows each address below the name, with its type and its custom value. It lists each feed that uses an address of the wallet, with its places (ADR 0091 item 3).

### The Operations

9. Change address, Rename and Remove act on each recipient of the wallet in each selected feed. This changes ADR 0091 items 5 to 8. Change address gives the wallet one address. The saved recipients of the wallet then merge into one saved recipient, and Discard puts back each of them (ADR 0091 items 12 and 20).
10. Add to a level adds one address. The form starts with the Lightning address of the wallet when it has one, else with the node key. This start value is an edit aid.
11. The page of a recipient gets a fifth operation, "Use the Lightning address". In each selected feed, it replaces each `node` recipient of the wallet with an `lnaddress` recipient: `type="lnaddress"`, the Lightning address as `address`, and no `customKey` and no `customValue`. The `name`, the `split` and `fee` stay. A feed with no `node` recipient of the wallet is "Not changed".
12. The form of item 11 starts with the Lightning address of the wallet. When the wallet has none, the user types one. The core accepts the address only when its lookup gives the node key and the custom pair of the wallet. Otherwise the core refuses the change set with the cause, and the user can use Change address.
13. When an operation puts an `lnaddress` recipient into a value block with `method="keysend"` or with no method, the core writes `method="lnaddress"` on that block. This is the form of the specification example and of MSP 2.0. A block that the operation does not change keeps its bytes. An edit in the editor does not change the method.
14. Change address and Add to a level need a type. The core refuses an operation with no type. The Type menu is the first field of the form, and it has no empty choice. The fields below it follow the type. They start at the member of the wallet with that type, not at the key of the wallet:
    - For `node`: "Node address", "Custom key" and "Custom value".
    - For `lnaddress`: "Lightning address" alone.
    - Change address starts at the node key when the wallet has one, else at the Lightning address.
    - Add to a level starts at the Lightning address when the wallet has one, else at the node key.
    - A change of the type puts the member of the new type in the fields, when the user did not change the address.
    - Each start value is an edit aid.

### The Editor

15. In the value editor, an `lnaddress` recipient with the status `none` shows the hint "Studio found no keysend data at {domain}. An app that pays only with keysend cannot pay this address." The status comes from `GET /recipients`. The editor makes no lookup.

## Alternatives Considered

- A second address on one recipient, as a fallback. Not chosen: the specification has no attribute for it, discussion 576 of the podcast namespace did not decide it, and no app reads it.
- Two value blocks in each feed, one for each method. Not chosen: no corpus feed has two blocks at one level, and no source says how an app selects one.
- A link of two addresses by hand. Not chosen for the first version: the lookup proves the link. A later decision can add a hand link for a provider with no lookup.
- A link from the name. Not chosen: a name is not proof. The same person has the name "TJ Doerfel" in one feed and the address as the name in another.
- A setting that stops the lookups. Not chosen: each address is public in a feed, and the request carries no user data.
- The UI groups a flat list of recipients. Not chosen: the number of feeds of a wallet is a union over the feed models, and the UI does not have the models (ADR 0042, ADR 0065).
- An operation that replaces a Lightning address with its node key. Not chosen: Change address covers it, and the node key breaks when the provider changes its node.
- A row with the type words, "lnaddress · node (keysend)". Replaced on 2026-10-31: two wallets with the same types showed the same row, and the shape of an address shows its type.
- An address field that starts at the key of the wallet. Replaced on 2026-10-31: for a Lightning address with a `found` lookup, the key is a node key that no feed has.

## Consequences

- The core makes HTTPS requests to the domains of the Lightning addresses of the collection. On the server, the private network guard applies. A listing of the recipients can take a maximum of 5 seconds when a provider is slow.
- A new migration adds the table `keysend_lookups` ([ADR 0014](0014-collection-in-sqlite.md)). The table holds no secret.
- The provider of a Lightning address can see that a Studio requested its lookup. It gets no other data.
- The Recipients page shows fewer rows for a collection with both forms of an address, and one change set changes both forms.
- No test sends a request to a provider (ADR 0045). The tests use a local test server, and the check against Fountain and Alby is a human test.

## Invariants

- A core test: the lookup parser reads the response of the specification example, with and without `customData`. A response with the status `ERROR` or with no `pubkey` gives `none`. A connection error gives `failed`, and keeps a previous result.
- A core test: a `node` recipient and an `lnaddress` recipient with the same result are one entry of the recipients, with the union of their feeds. An `lnaddress` recipient with no result is its own entry.
- A core test: Rename, Remove and Change address act on each recipient of a wallet in one feed.
- A core test: Use the Lightning address writes the attributes of item 11 and the method of item 13. A block with no `node` recipient of the wallet keeps its bytes. A Lightning address whose result gives a different key is refused.
- A core test: Change address and Add to a level with no type are refused.
- A Node test: the Type menu has no empty choice, and it is before the address field.
- A Node test: the start values of a form come from the members of the wallet.
- A Node test: the row of a wallet with two addresses gives the text of item 7.
- The corpus tests pass with no change.
- No test sends a request to a host outside the computer (ADR 0045).

## Non-Goals

- A link of two addresses by hand.
- A Lightning address from the `lud16` of a Nostr profile (ADR 0074).
- Two value blocks in one channel or item.
- The method of a block that the user changes in the editor.
- A payment by Studio. Studio sends no keysend and requests no invoice.

## Verification

Mechanical: the invariants above, and the four commit checks.

Visual: the Recipients page with a wallet of two addresses, and the page of that recipient. Also the sheet "Use the Lightning address", the sheet "Change address" with each type, and the hint of item 15. In the two themes, and at 390 px.

Human, on `studio.musicindex.org`: a Fountain address and an Alby address each join their node recipient in one entry. A Strike address stays its own entry. Use the Lightning address writes one feed, and the feed on its file host has `type="lnaddress"` and `method="lnaddress"`.
