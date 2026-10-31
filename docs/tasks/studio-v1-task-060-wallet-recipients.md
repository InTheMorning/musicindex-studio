# Studio v1 Task 060: Wallet Recipients

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Recipient Batch Edits, File Hosts And Dates".
Decisions: [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0027](../adr/0027-the-recipient-picker.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Planned on 2026-10-31.

## Goal

A `node` recipient and an `lnaddress` recipient of the same wallet are one recipient on the Recipients page. A change set acts on each address of the wallet. A new operation replaces the node key with the Lightning address. The core refuses an operation with no type.

## Packets

The planning model examines each packet before its commit. The packets go in this sequence:

- 060a (the keysend lookup in the core) and 060b (the wallet in the operations) share no file. They can run at the same time.
- 060c (the routes) starts after the commits of 060a and 060b.
- 060d (the UI) starts after the commit of 060c.

Human test, by the operator, on `studio.musicindex.org` after the deploy: the human check of ADR 0095, Verification.

## Words

- A **wallet** is the node key with the custom key and the custom value that receive a payment (ADR 0095 item 1).
- The **key of a wallet** is a `RecipientKey`: the node key with its custom pair, or the Lightning address when the wallet has no node key (ADR 0095 item 5).
- A **member** of a wallet is a `RecipientKey` that names the wallet: the key itself, and each Lightning address whose lookup gives the key.
- A **lookup** is the result of `https://<domain>/.well-known/keysend/<name>` for one Lightning address.

## Packet 060a: The Keysend Lookup In The Core

### Files

Inspect: `crates/core/src/profiles.rs` (`within`, `split_address`, `nip05_key`, `check_addresses`), `crates/core/src/web.rs` (`Web::get`, `WebFile`, `WebError`), `crates/core/src/collection.rs` (`MIGRATIONS`, `saved_recipients`, `SavedRecipientRow`), `crates/core/src/testing.rs` (`TestServer`), `crates/core/src/change_set.rs` (`RecipientKey`, read only), `crates/core/src/lib.rs`.

Change: `crates/core/src/keysend.rs` (new), `crates/core/src/lib.rs` (the module), `crates/core/src/collection.rs` (the migration and the lookup rows), `crates/core/src/profiles.rs` (only `within` becomes `pub(crate)`).

Do not change: `crates/core/src/change_set.rs` (packet 060b changes it at the same time), the API, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `crates/core/src/keysend.rs` has the types `LookupStatus` (`Found`, `None`, `Failed`, with `as_str` and `parse`) and `Lookup`: the lowercase `address`, `pubkey: Option<String>`, `custom_key: Option<String>`, `custom_value: Option<String>`, `status` and `checked_at: i64`. `Lookup::key()` gives the `RecipientKey` of a `Found` lookup: the pubkey with the custom pair.
2. `lookup_url(scheme: &str, address: &str) -> Option<String>`: the address is trimmed and lowercase. It has the form `name@domain`, with a name and a domain that are not empty, and a domain with no `/` and no space. The result is `{scheme}://{domain}/.well-known/keysend/{name}`. Other text gives `None`. A node key has no `@`, so it gives `None`.
3. `parse_lookup(bytes: &[u8]) -> Option<(String, Option<String>, Option<String>)>`: the pubkey, and the `customKey` and `customValue` of the first entry of `customData`. A response with `status` `ERROR`, with no `pubkey`, with an empty `pubkey` or with no JSON gives `None`. The tag is not examined.
4. `request(web: &Web, scheme: &str, address: &str) -> LookupResult`, with `LookupResult::Found { pubkey, custom_key, custom_value }`, `LookupResult::None` and `LookupResult::Failed`. A response with keysend data is `Found`. A response that `parse_lookup` refuses, and an HTTP status from 400 to 499, are `None`. A network error, a time limit and each other status are `Failed`. The limit of the body is 16 KiB.
5. The constants: `LOOKUP_LIMIT` is 5 seconds, `REFRESH_AFTER` is 86 400 seconds (one day), and `RETRY_AFTER` is 3 600 seconds (one hour). `is_due(lookup: Option<&Lookup>, now: i64) -> bool`: no lookup is due. `Found` and `None` are due after `REFRESH_AFTER`. `Failed` is due after `RETRY_AFTER`.
6. `refresh(collection: &Collection, web: &Web, scheme: &str, addresses: &[String], now: i64) -> Result<HashMap<String, Lookup>, CollectionError>`: for each address that `lookup_url` accepts, one time each, it requests the due lookups at the same time, as `check_addresses` of `profiles.rs` does with `within`. A `within` with no result is `Failed`. Then it stores each result:
   - `Found`: the row gets the data, the status `found` and `checked_at = now`.
   - `None`: the row gets no data, the status `none` and `checked_at = now`.
   - `Failed` with a row: the row keeps its data and its status, and gets `checked_at = now`.
   - `Failed` with no row: a row with no data, the status `failed` and `checked_at = now`.

   The result is each lookup of the given addresses that the table has, by lowercase address.
7. `wallet_key(key: &RecipientKey, lookups: &HashMap<String, Lookup>) -> RecipientKey`: when the lowercase address of `key` has a `Found` lookup, the key of that lookup. Else a clone of `key`.
8. `members(key: &RecipientKey, lookups: &HashMap<String, Lookup>) -> Vec<RecipientKey>`: the key itself, then each `Found` lookup with a key equal to `key`. Each of these is a `RecipientKey` with the address of the lookup and no custom pair. The list is A to Z by address after the first entry. Two keys are equal when their address, custom key and custom value are equal, with `None` equal to an empty text.
9. `wallet_of(key, lookups) -> (RecipientKey, Vec<RecipientKey>)` gives `wallet_key(key)` and its `members`.
10. The collection gets a new migration at the end of `MIGRATIONS`:

    ```sql
    CREATE TABLE keysend_lookups (
        address TEXT PRIMARY KEY,
        pubkey TEXT,
        custom_key TEXT NOT NULL DEFAULT '',
        custom_value TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL,
        checked_at INTEGER NOT NULL
    );
    ```

    `Collection::keysend_lookups() -> Vec<Lookup>` reads each row. `Collection::set_keysend_lookup(&Lookup)` inserts or replaces a row. `Collection::touch_keysend_lookup(address, checked_at)` changes only `checked_at`. An empty custom key or custom value in a row is `None` in `Lookup`.
11. No function of this packet makes a request to a host outside the computer in a test. The tests use `TestServer` of `testing.rs` with the scheme `http` and an address such as `ann@127.0.0.1:PORT`.

### Acceptance Criteria

Mechanical:

- A core test: `lookup_url` gives `https://getalby.com/.well-known/keysend/chadf` for `Chadf@GetAlby.com`, and `None` for a node key, for `@x`, for `a@` and for an empty text.
- A core test: `parse_lookup` reads the example of the specification, with `customData` and without it. It gives `None` for `{"status":"ERROR","reason":"x"}`, for a response with no `pubkey` and for text that is not JSON.
- A core test with `TestServer`: `refresh` stores `found` with the data, `none` for a 404 and `failed` for a server that does not answer. A second `refresh` within one day sends no request. A `failed` result after a `found` result keeps the data and the status.
- A core test: `wallet_key` and `members` give one wallet for a node key and a Lightning address with the same lookup. A Lightning address with the status `none` gives a wallet of one member.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md
- docs/tasks/studio-v1-task-060-wallet-recipients.md (the Words section and packet 060a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 060a names.

Goal:
- The core requests and keeps the keysend lookup of each Lightning address, and gives the wallet of a recipient key.

Constraints:
- Follow the Architecture section of packet 060a fully.
- No test sends a request to a host outside the computer (ADR 0045).
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, the API, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 060a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 060b: The Wallet In The Operations

### Files

Inspect: `crates/core/src/change_set.rs` (all), `crates/core/src/collection.rs` (`change_saved_recipient`, `saved_recipient_row`, `restore_saved_recipient`, `set_change_set_saved_before`, `change_set_saved_before`), `crates/core/tests/change_set.rs`, `crates/core/src/feed/value.rs` (`ValueBlock`).

Change: `crates/core/src/change_set.rs`, `crates/core/tests/change_set.rs`.

Do not change: `crates/core/src/collection.rs` (packet 060a changes it at the same time), the API, the UI, the ADRs, an existing migration.

### Architecture

These choices are fixed. Do not change them.

1. `change_set.rs` gets `pub struct Wallet { pub key: RecipientKey, pub members: Vec<RecipientKey> }`. `Wallet::single(key)` has the key as its only member. `Wallet::matches(&Recipient)` is true when a member matches. `Wallet::has(&RecipientKey)` is true when a member is equal to the key. `RecipientKey::parts` becomes `pub`.
2. `apply` and `make_change_set` take `&Wallet` in the place of `&RecipientKey`. The change set row keeps `wallet.key` as its `recipient` JSON, as before. Each match of a recipient in a value block uses `Wallet::matches`.
3. `RecipientOperation` gets `UseLnaddress { address: String }`, with the serde tag `use_lnaddress`. In a block, it changes each recipient that the wallet matches and that has a `recipient_type` other than `lnaddress`. The type becomes `lnaddress`, the address becomes `address`, and the custom key and the custom value become `None`. The name, the split and `fee` stay.
   - A block with no such recipient but with a member gives `Cause::NoNodeRecipient` (new). A block with no member gives `Cause::NoRecipient`. `pick_cause` prefers `NoNodeRecipient` to `NoRecipient`.
4. Change address with a wallet: `HasNewAddress` applies when the block has a recipient with the new key that the wallet does not match. Each matched recipient gets the new address, the custom pair and the type. The core does not merge two recipients of one block.
5. The method rule (ADR 0095 item 13): three operations put an `lnaddress` recipient into a block. These are Change address with the type `lnaddress`, Add with the type `lnaddress`, and Use the Lightning address. After one of them changed a block, the block gets `method = Some("lnaddress")` when its method is `None` or `keysend`. A block that did not change keeps its method.
6. The type rule (ADR 0095 item 14): `make_change_set` gives the `invalid` error "the recipient needs a type" for Change address and Add with a `recipient_type` that is `None` or empty after trim. It gives "the Lightning address is empty" for Use the Lightning address with an empty address. The check comes before `delete_closed_change_sets`.
7. The saved recipients (ADR 0095 item 9): `change_saved_recipient` takes the wallet. For Change address and Rename, it collects the saved row of each member that has one. It stores the list as a JSON array in `saved_before`, and then changes each of these rows with `Collection::change_saved_recipient`, member by member. Use the Lightning address and Remove change no saved recipient. `discard_change_set` reads `saved_before` as a JSON array, or as one object from before this packet (an untagged serde enum), and puts back each row.

### Acceptance Criteria

Mechanical:

- A core test: Rename, Remove and Change address with a wallet of two members change the two recipients of one feed. Remove scales the other splits by the total of the two.
- A core test: in a block with `method="keysend"`, Use the Lightning address writes the attributes of item 3 and the method of item 5. A block that has only the Lightning address of the wallet gives `NoNodeRecipient`, and the feed keeps its bytes. A block with `method="lnaddress"` keeps its method.
- A core test: Change address with the type `lnaddress` sets the method of a keysend block. Change address with the type `node` does not change the method.
- A core test: Change address with no type, and Add with no type, give the `invalid` error, and no change set row exists after it.
- A core test: Change address of a wallet with two saved rows merges them into one saved row, and Discard puts back the two rows.
- A core test: `discard_change_set` reads a `saved_before` with one object.
- The tests of 054a and 054d pass with no change of their expected values, except the signature of `apply` and `make_change_set`.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md
- docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md
- docs/tasks/studio-v1-task-060-wallet-recipients.md (the Words section and packet 060b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 060b names.

Goal:
- Each operation of a change set acts on each member of a wallet, and the new operation replaces the node key with the Lightning address.

Constraints:
- Follow the Architecture section of packet 060b fully.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/collection.rs`, the API, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 060b.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 060c: The Routes

### Files

Inspect: `crates/api/src/routes.rs` (`list_recipients`, `UsedRecipient`, `feed_models`, `recipient_places`, `recipient_uses`, `make_change_set`, `change_set_json`, `nostr_profiles` for the use of `nip05_scheme`), `crates/api/src/workspace.rs` (`nip05_scheme`, `with_plain_nip05`, `web`), `crates/api/src/tests.rs` (`app`, `app_with_index`, `app_with_rules`, the change set tests, the use of `TestServer`), `crates/core/src/keysend.rs`, `crates/core/src/change_set.rs` (`Wallet`).

Change: `crates/api/src/routes.rs`, `crates/api/src/workspace.rs`, `crates/api/src/tests.rs`.

Do not change: the core, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `Workspace` gets `keysend_scheme: Option<&'static str>`. It is `Some("https")` by default. `None` means no lookup. A `#[cfg(test)]` builder `with_keysend(scheme: Option<&'static str>)` sets it. `app`, `app_with_index` and `app_with_rules` of `tests.rs` set `None`, so that no test sends a request outside the computer (ADR 0045). The lookup tests set `Some("http")` and a `TestServer`.
2. `list_recipients` collects the recipients as now. Then, when the scheme is `Some`, it calls `keysend::refresh` with each address of the known recipients and of the used recipients. With `None`, the lookups are the rows of the table. Then it groups the recipients by `keysend::wallet_key`. One entry for each wallet has:
   - `address`, `custom_key`, `custom_value`: the key of the wallet.
   - `recipient_type`, `name`, `saved_id`: from the recipient with the key, else from the first address with a value.
   - `feeds`: the number of different feeds that use an address of the wallet.
   - `addresses`: the list of the recipients of the wallet, each with `address`, `custom_key`, `custom_value`, `recipient_type`, `name`, `feeds`, `saved_id` and `lookup`. `lookup` is `found`, `none`, `failed` or null. The list has the key first, then A to Z by address.

   The list is in the sequence of today: by name, then by address.
3. `recipient_uses` and `make_change_set` make the wallet with `keysend::wallet_of` and the rows of the table, as `Wallet { key, members }`. They send no request, except item 4.
4. For `use_lnaddress`, `make_change_set` checks the address before the core runs:
   - `lookup_url` refuses it: 422 `invalid_recipient`, "the Lightning address needs the form name@domain".
   - The key of the wallet has no node key (its address has `@`): 422 `invalid_recipient`, "the recipient has no node key".
   - The table has no lookup of the address, or it is due, and the scheme is `Some`: the route calls `keysend::refresh` for that one address.
   - The status is not `found`: 422 `lnaddress_no_keysend`, "Studio found no keysend data at {domain}".
   - The key of the lookup is not the key of the wallet: 422 `lnaddress_mismatch`, "the Lightning address {address} pays a different wallet".

   After the check, the route makes the wallet again, so that the address is a member.
5. The type check of ADR 0095 item 14 is in the core (packet 060b). The route maps the `invalid` error as it maps it today.
6. `change_set_json` does not change.

### Acceptance Criteria

Mechanical:

- An API test with `TestServer` and `Some("http")`: one feed has a `node` recipient, and one feed has an `lnaddress` recipient with a lookup that gives the same key. `GET /recipients` gives one entry, with `feeds: 2`, two `addresses`, and `lookup: "found"` on the Lightning address.
- An API test: with `None`, the same two feeds give two entries, and no `TestServer` route is called.
- An API test: `recipients/uses` with the key of the wallet lists the two feeds.
- An API test: a Rename change set with the key of the wallet changes the two feeds.
- An API test: `use_lnaddress` with an address whose lookup gives a different key gives 422 `lnaddress_mismatch`. With an address of status `none`, 422 `lnaddress_no_keysend`. With the address of the wallet, 201, and the draft of the node feed has `type="lnaddress"` and `method="lnaddress"`.
- An API test: Change address with no type gives 422 `invalid_recipient`.
- The tests of 054b and 054d pass with no change of their expected values.
- The four commit checks pass.

### Escalation

- Stop and report when `GET /recipients` with no due lookup is slower than 1 second with the corpus feeds in a test collection.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md
- docs/tasks/studio-v1-task-060-wallet-recipients.md (the Words section and packet 060c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 060c names.

Goal:
- The recipient routes group the recipients by wallet, and a change set acts on each member of the wallet.

Constraints:
- Follow the Architecture section of packet 060c fully.
- No test sends a request to a host outside the computer (ADR 0045).
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The core, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 060c.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 060d: The UI

### Files

Inspect: `crates/api/ui/recipients.js` (all), `crates/api/ui/app.js` (`RecipientsPage`, `RecipientPage`, `OperationForm`, `OperationSheet`, `OPERATION_TITLES`, `ChangeSetPage`, `ChoiceSelect`), `crates/api/ui/values.js` (`Recipients`, `TYPES`, `PickerSheet`, `identityOf`), `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui-tests/casing.test.mjs`, `crates/api/ui-tests/words.test.mjs`.

Change: `crates/api/ui/recipients.js`, `crates/api/ui/app.js`, `crates/api/ui/values.js`, `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui/styles.css` (only when the header of the recipient page needs a rule).

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `recipients.js` gets these functions, with no Preact (ADR 0042):
   - `recipientTitle(entry)`: the name, else the address of the first entry of `addresses`, else `address`.
   - `typesText(entry)`: the different types of `addresses`, `lnaddress` first, joined with " · ", for example "lnaddress · node (keysend)". An entry with no `addresses` gives `typeLabel(entry.recipient_type)`. `recipientRow` uses it as `type`.
   - `addressLines(entry)`: one `{ address, type }` for each entry of `addresses`, with `typeLabel`. An entry with no `addresses` gives one line.
   - `lnaddressOf(entry)`: the first address of the type `lnaddress`, or null. `nodeOf(entry)`: the first address of the type `node`, or null. `hasNode(entry)` is true when `nodeOf` gives an address.
   - `defaultType(address)`: `lnaddress` when the address has "@", else `node`. It is a start value only.
   - `pickerRows(list)`: one row for each entry of `addresses` of each entry, with the `name` of the entry when the address has none.
   - `lookupHint(address)`: "Studio found no keysend data at {domain}. An app that pays only with keysend cannot pay this address.", with the domain after the "@".
   - `operationTitle` gives "Use the Lightning address" for `use_lnaddress`. `operationLine` gives "Studio replaces the node key of {name} with {address} in the selected feeds." `causeText` gives "Each value block has the Lightning address already." for `no_node_recipient`.
2. The Recipients page row shows `recipientRow(entry)` as today. With `typesText`, the row of a wallet reads "TJ Doerfel · lnaddress · node (keysend) · 7 feeds".
3. The header of the recipient page shows one line for each entry of `addressLines`, each with the address in `mono` and its type.
4. The page gets a fifth button, "Use the Lightning Address", with the class `button action`, after "Add to a Level". It shows only when `hasNode(recipient)`. It is disabled when no feed is selected.
5. `OperationForm` with the kind `use_lnaddress` has one field, "Lightning address", with the start value `lnaddressOf(recipient) || ''` and the hint "The address must pay the same wallet. Studio checks it." The button label is "Use in {feedCount}". The operation is `{ kind: 'use_lnaddress', address }`.
6. The Type menu of `OperationForm` has no placeholder. For Change address, the type starts at `recipient.recipient_type || defaultType(recipient.address)`. For Add to a level, the address starts at `lnaddressOf(recipient) || recipient.address`, and the type at `defaultType` of that address. `OPERATION_TITLES` gets `use_lnaddress: 'Use the Lightning address'`.
7. `PickerSheet` of `values.js` uses `pickerRows(list)` in the place of `list`, so that each address is one row (ADR 0027 item 3).
8. The `Recipients` component of `values.js` loads `GET /recipients` one time for the page, through a module-level promise that each `Recipients` shares. It builds a map from the lowercase address to `lookup`. The Address field of a recipient with the type `lnaddress` and the lookup `none` shows `lookupHint(address)` as its hint. A failed load shows nothing. The editor makes no lookup.
9. The UI sends no `itunes:` or `podcast:` text, and makes no feed rule (ADR 0065). `defaultType` and `pickerRows` are edit aids.

### Acceptance Criteria

Mechanical:

- A Node test: `typesText`, `addressLines`, `lnaddressOf`, `nodeOf` and `recipientTitle` for an entry with two addresses, and for an entry with no `addresses`.
- A Node test: `pickerRows` gives one row for each address, with the name of the entry.
- A Node test: `lookupHint("chadf@getalby.com")` names `getalby.com`.
- A Node test: `operationTitle`, `operationLine` and `causeText` for the new operation and the new cause.
- A Node test: the text of `app.js` has no `placeholder` on the element with the id `op-type`. The message names ADR 0095 item 14.
- The casing test and the words test pass with no change of their lists.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- The Recipients page with a wallet of two addresses, and the page of that recipient with its two address lines and five buttons.
- The sheet "Use the Lightning address" with its start value.
- The hint of item 8 on an `lnaddress` recipient with no keysend data.
- `$edges()` finds no content at the edge of a group.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md
- docs/tasks/studio-v1-task-060-wallet-recipients.md (the Words section and packet 060d only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 060d names.

Goal:
- The Recipients pages show one recipient for each wallet, with the fifth operation, and the editor shows the keysend hint.

Constraints:
- Follow the Architecture section of packet 060d fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case (musicindex ADR 0023 R-CASE-1). Each other text is in sentence case.
- Each UI text, comment and test message is in ASD-STE100 Simplified Technical English.
- Check the syntax of each changed UI file with a `.mjs` copy.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 060d.

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
