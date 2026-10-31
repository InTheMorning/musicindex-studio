# Studio v1 Task 061: Node Providers And Address Lines

- Decision: [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md) items 7, 8 and 16 to 18.
- Research: [keysend and Lightning address recipients](../research/2026-10-31-keysend-and-lightning-address-recipients.md), the section "The Nodes Behind The Addresses".
- Status: Planned on 2026-10-31.

## Goal

A row of the Recipients page tells a node key from an account on it, and a name from an address. Each address of a wallet has its own line that names its type. A node on a shared provider node shows the provider domain in the place of the key. The core names the provider from the lookups of the collection, or from a table of shared nodes with evidence.

## Packets

| Packet | Scope | Model |
| --- | --- | --- |
| 061a | The provider of a node in the core, and in `GET /recipients` | Sonnet |
| 061b | The row lines, the count and the header in the UI | Sonnet |

061b starts after 061a, because its Node tests use the `provider` field of each address.

## Words

- A **provider node** is a node that two or more Lightning addresses of one domain resolve to, with different custom values. A node of the table of shared nodes is also a provider node. The Fountain node is one.
- A **personal node** is a node that one address resolves to, such as an Alby Hub node. It has no provider name.
- An **account** is the custom value of a `node` recipient. The accounts of one shared node differ only in it.

## Packet 061a: The Provider Of A Node

### Architecture

1. `crates/core/src/keysend.rs` gets `pub const SHARED_NODES: &[(&str, &str)]`, a table of node keys and domains. It has the two entries of ADR 0095 item 17: `03b6f613e88bd874177c28c6ad83b3baba43c4c656f56be1f8df84669556054b79` as `fountain.fm`, and `030a58b8653d32b99200a2334cfe913e51dc7d155aa0116c176657a4f1722677a3` as `getalby.com`. The doc comment names the research note as the evidence, and says that an entry needs a lookup or a graph alias as evidence.
2. `pub fn provider_of<S: BuildHasher>(pubkey: &str, lookups: &HashMap<String, Lookup, S>) -> Option<String>`. It compares keys in lowercase:
   - When the table has the key, the result is its domain.
   - Else, for each `Found` lookup with that key, the function takes the domain of the lookup address (the text after the last `@`) and its custom pair. A domain with two or more different custom pairs names the node.
   - With two or more such domains, the result is the domains A to Z, with `, ` between them. Else `None`.
3. `crates/api/src/routes.rs`, `group_wallets`: each entry of `addresses` gets `"provider"`: `provider_of(&m.address, lookups)` when `m.recipient_type` is `node`, else `null`.
4. Core tests in `keysend.rs`: the table names the Fountain key, also in upper case. Two lookups of `a@fountain.example` and `b@fountain.example` with the same key and different custom values give `fountain.example`. One lookup gives `None`. Two lookups with the same custom pair give `None`. A key that no lookup names gives `None`.
5. An API test in `crates/api/src/tests.rs`, with the fixture `wallet_app` and a third test address on the same key with a different custom value: `GET /recipients` gives the test domain as the `provider` of the node address, and `null` for the Lightning address. With one address on the key, the `provider` is `null`.

### Acceptance Criteria

Mechanical:
- The core tests of item 4.
- The API test of item 5.
- No test sends a request to a host outside the computer (ADR 0045).
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md, items 16 to 18
- docs/tasks/studio-v1-task-061-node-providers.md, packet 061a
- crates/core/src/keysend.rs
- crates/api/src/routes.rs, the functions `list_recipients` and `group_wallets`
- crates/api/src/tests.rs, the fixtures `app_with_keysend`, `wallet_app` and `wallet_feeds`, and the test `recipient_uses_and_a_rename_act_on_each_member_of_the_wallet`
- ~/.agents/skills/asd-ste100/SKILL.md

Goal:
- The core names the provider of a node, and `GET /recipients` gives it for each node address.

Constraints:
- Follow the Architecture section of packet 061a fully.
- Each comment, doc comment and test message is in ASD-STE100 Simplified Technical English.
- No test sends a request to a host outside the computer (ADR 0045).

Do not touch:
- The UI files, the ADRs, the other core modules.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 061a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 061b: The Row Lines, The Count And The Header

### Architecture

1. `crates/api/ui/recipients.js` (no Preact, ADR 0042):
   - `addressLine(member, full)` gives `{ label, address, account }` for one member of a wallet. For the type `lnaddress`, the label is "Lightning address" and `address` is the address. For a node with a `provider`, the label is "{provider} node" and `address` is `null` when `full` is false, else the full key. For a node with no provider, the label is "Node" and `address` is the key: the full key when `full` is true, else `shortKey` of `nostrkey.js`. `account` is the custom value, or `null`.
   - `rowLines(entry)` gives the texts of the lines of a row (ADR 0095 item 7): the Lightning addresses first, then each node. Each text starts with the label. The address follows after one space, when it is not null. Then " · account {custom value}" follows, when the account is not null. Examples:
     - "Lightning address boostbot@fountain.fm"
     - "fountain.fm node · account 01IMQkt4BFzAiSynxcQQqd"
     - "Node 03d55f4d…aa67"
   - `addressLines(entry)` gives `addressLine(member, true)` for each member, in the same sequence, for the header of the page (item 8).
   - `recipientRow(entry)` gives `{ title, count, lines }`: `recipientTitle`, the text "N feeds" or "Saved, not used", and `rowLines(entry)`. The function `shortAddress` goes away.
   - An entry with no `addresses` is its own single member, as today.
2. `crates/api/ui/app.js`:
   - The list row: the title in `<span class="label">`, then one `<span class="summary">` for each line, in `<span class="text">`. After it, `<span class="count">{count}</span>` and the `go` chevron.
   - The header of the recipient page has one `<p class="hint">` for each line of `addressLines`. It shows the label, then the address in a `<span class="mono">` when it is not null. Then it shows " · account {account}" when the account is not null.
3. `crates/api/ui/styles.css`: `.link-row .count { color: var(--muted); font-size: .85rem; white-space: nowrap; font-variant-numeric: tabular-nums; }`, next to `.link-row .go`.
4. `crates/api/ui-tests/recipients.test.mjs`: the tests of `recipientRow` and `addressLines` change to the new shapes. The test of the type menu stays. Three fixtures:
   - a wallet with a Lightning address and a node with a `provider` and a custom value
   - a node with no provider and no custom value
   - a node with no provider and a custom value
5. No feed rule in the UI (ADR 0065). The lines are display values.

### Acceptance Criteria

Mechanical:
- A Node test: `rowLines` for the three fixtures of item 4 gives the texts of item 1.
- A Node test: `addressLines` gives the full key and the account, and the provider label.
- A Node test: `recipientRow` gives `title`, `count` and `lines`, and `count` is "Saved, not used" for a saved recipient with no feed.
- The four commit checks pass.

Visual (the planning model takes the screenshots, ADR 0095 Verification):
- The Recipients page with a wallet of two addresses, a provider node and a node with no provider. The page of that wallet. In the two themes, and at 390 px with touch. `$edges()` and `$small()` are empty.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md, items 7, 8 and 16 to 18
- docs/tasks/studio-v1-task-061-node-providers.md, packet 061b
- crates/api/ui/recipients.js
- crates/api/ui/nostrkey.js, the function `shortKey`
- crates/api/ui/app.js, the components `RecipientsPage` and `RecipientPage` (search for `recipientRow(` and `walletLines(`)
- crates/api/ui/styles.css, the rules of `.link-row`
- crates/api/ui-tests/recipients.test.mjs
- ~/.agents/skills/asd-ste100/SKILL.md

Goal:
- Each address of a wallet has its own line in the row and in the header. The feed count is at the right of the title, and a provider node shows its domain.

Constraints:
- Follow the Architecture section of packet 061b fully.
- No feed rule in the UI (ADR 0065).
- Each UI text is in sentence case, except a button label (musicindex ADR 0023 R-CASE-1).
- Each UI text, comment and test message is in ASD-STE100 Simplified Technical English.
- Check the syntax of each changed UI file with a `.mjs` copy in `.studio-dev/`.
- Example: `cp crates/api/ui/app.js .studio-dev/app-check.mjs && tools/node --check .studio-dev/app-check.mjs`.

Do not touch:
- The core, the API, the ADRs, `values.js`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 061b.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
