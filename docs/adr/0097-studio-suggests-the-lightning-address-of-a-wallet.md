# ADR 0097: Studio Suggests The Lightning Address Of A Wallet

- Status: Accepted
- Date: 2026-10-31
- Owner: operator
- Accepted: 2026-10-31 by the operator.
- Changes: [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) item 12, for a wallet with no Lightning address in the collection.
- Uses: [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) for the Nostr keys of a feed and the lookup relays, [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the network policy, and [ADR 0065](0065-the-core-applies-each-feed-rule.md).

## Context

- ADR 0095 item 11 gives the operation "Use the Lightning address". Item 12 starts the form with the Lightning address of the wallet. When the wallet has none, the user types one, and the core checks it.
- A wallet of a node key frequently has no Lightning address in the collection. On 2026-10-31 the operator found that the user must then find and type an address with no aid.
- No method changes a node key and a custom pair back into a Lightning address. But the core can make guesses, and the keysend lookup of ADR 0095 item 1 checks each guess.
- The provider of a node can be known (task 061). A provider address is frequently the name of the account at the domain of the provider, for example `name@fountain.fm`.
- A kind 0 profile of Nostr can have the field `lud16`, which is a Lightning address. A feed can give the npub of a person or of the feed (ADR 0074).
- `keysend::members` reads each stored lookup. A stored lookup of a guess makes the guess an address of the wallet, also when no feed uses it.
- The operator selected the three sources of item 2 from a questionnaire on 2026-10-31. The musicindex index was the fourth selection. It needs a route that Stophammer does not have.

## Decision

1. The core gives suggestions for the Lightning address of a wallet with a node key. A suggestion is an address with a keysend lookup that gives the node key and the custom pair of the wallet.
2. The core makes guesses from three sources:
   - Collection: each Lightning address of a recipient in a feed of the collection, when the recipient has a name of the wallet.
   - Nostr: the `lud16` of the newest kind 0 of each key of `profiles::feed_keys` in each feed that uses the wallet. A key with a name of the wallet comes first. The core reads at most 8 keys.
   - Provider: each name of the wallet at each domain of `keysend::provider_of`. The core makes the name lowercase and removes each character that a local part cannot have. A name with `@`, such as `ann@fountain.fm`, gives the text before the `@`.
3. The core requests the lookup of each guess, with the time limit of ADR 0095. It does not keep a lookup of a guess in the collection. It checks at most 16 guesses.
4. The core gives each suggestion with its sources, in the sequence Collection, Nostr, Provider. A guess that is an address of the wallet in the collection is not a suggestion.
5. The sheet "Use the Lightning address" asks for the suggestions when the wallet has no Lightning address. Then:
   - With suggestions, the sheet shows a select menu "Lightning address". It starts at the first suggestion. Its last option is "Other address", and that option shows the text field.
   - The menu shows only the address. The hint of the menu gives the sources of the selected address, for example "Studio found this address at the node provider".
   - With no suggestion, the sheet shows the text field and the hint "Studio found no Lightning address for this wallet."
6. The core continues to examine the address of the operation (ADR 0095 item 12), also when the user selects a suggestion.
7. Each relay connection and each request follows the network policy of the shell (ADR 0057).

## Alternatives Considered

### Search the musicindex index

The index has all feeds, and the same wallet can have a Lightning address in a different feed. Stophammer has no route that finds a recipient. This is possible after a decision in Stophammer.

### A text field with a list of suggestions

The browser list of an input shows only when the user types. The user does not see that suggestions are available.

## Consequences

- A user of a provider such as Fountain usually gets the address with one selection.
- The core makes a maximum of 16 lookup requests and one relay request each time the sheet opens. A wallet that has a Lightning address makes no request.
- A guess that pays a different wallet does not show.

## Acceptance Criteria

Mechanical:

- A core test: a provider guess, a collection guess and a Nostr guess with lookups that give the wallet are suggestions, with their sources.
- A core test: a guess with a lookup that gives a different custom value is not a suggestion. A guess that is an address of the wallet is not a suggestion.
- A core test: the collection keeps no lookup of a guess.
- A core test: a name gives a local part in lowercase with no space, and a name with no usable character gives no guess.
- An API test: the route of the suggestions refuses a recipient with no node key.
- A Node test: the model of the sheet gives the select menu with suggestions, and the text field with none.

Visual: the sheet "Use the Lightning address" with suggestions and with none, in the two themes and at 390 px with touch.

Human, on `studio.musicindex.org`: a Fountain node recipient with a known account name gets its Fountain address as a suggestion.
