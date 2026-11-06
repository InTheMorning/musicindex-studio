# ADR 0138: The Names Of A Wallet Address And The Correction Of A Wrong Key

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire, from drawings of the rows. The operator selected "Correct the Feeds" with "Add Anyway". The operator also selected the names "LNURL address" and "Node address".
- Changes: the words of [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) and [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 15 in the UI, and the button of [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md) item 18 for a key with a warning.
- Uses: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) for the correction of a key.
- Source: [findings of local use, findings 19 and 20](../reviews/2026-11-06-local-use-findings.md)

## Context

The two kinds of wallet address had names that start with the same word, so a list of them was not easy to read.

"More from your feeds" showed a Nostr key of a different account with a warning, and its only button was "Add". "Check your feeds" can correct that key, but the row had no link to it.

## Decision

1. A row of "More from your feeds" with a key warning has two buttons. "Correct the Feeds" opens "Check your feeds". "Add Anyway" adds the key. Both have the role action of ADR 0037.
2. A row with no warning keeps the button "Add".
3. The UI names an address with `@` "LNURL address", and it names a node key "Node address". The type menus use "LNURL Address" and "Node Address".
4. A node of a shared provider keeps its name, for example "Alby node" (ADR 0095).
5. The feed values do not change. A feed keeps `type="node"`, `type="lnaddress"` and `method="lnaddress"`.

## Alternatives Considered

- A third button, "Not This Person". Not chosen: it needs new stored data, and the feed keeps the wrong key.
- Only "Correct the Feeds". Not chosen: a band or a label can have a different profile name (ADR 0103).
- The name "Lightning address". The planning model recommended it, because it is the name in other wallets. The operator selected "LNURL address".

## Consequences

- A user who knows the name "Lightning address" from a wallet reads a different name in Studio.
- The earlier ADRs keep their words for this concept. This ADR gives the words of the UI.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` and `crates/api/ui-tests/recipients.test.mjs` examine the labels and the texts.

Visual: the planning model looks at a row with a warning at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
