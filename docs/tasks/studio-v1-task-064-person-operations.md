# Studio v1 Task 064: The Operations Of A Person

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decisions: [ADR 0098](../adr/0098-the-people-module.md) items 17 to 25, [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) items 7 to 12, [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0097](../adr/0097-studio-suggests-the-lightning-address-of-a-wallet.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Not started.

## Goal

The core applies the four operations of a person to the selected feeds, through a change set. The API gives a preview of an operation, so that a sheet can show its result before the review. The core suggests the `lud16` of each Nostr key of a person, and the API gives an export of the people. The UI does not change in this task.

## Packets

The planning model examines each packet before its commit.

- 064a (the operations in `change_set.rs`) and 064c (the `lud16` and the export) share no file. They can run at the same time.
- 064b (the change sets of a person and the routes) starts after the commits of 064a and 064c.

No packet changes the UI, so no packet needs a screenshot visual check. Human test: none for this task. The human tests of ADR 0098 and ADR 0099 need the screens of task 065.

## Words

- A **person operation** is one of the four operations of ADR 0098 item 20: Use this address, Rename, Add to all releases and Remove.
- The **wallet of a person** is a `Wallet` whose members are the members of each wallet of the person (ADR 0095 item 5). Its key is the default address of the person, else an empty key.
- A **credit of a person** is a `podcast:person` that matches the person (ADR 0098 item 6). Its identity is a credit name of the person, or its `npub` is a key of the person.
- A **part** is one of the items of Add to all releases: the payee, the credit or the support link.

## Packet 064a: The Person Operations In The Change Set

### Files

Inspect: `crates/core/src/change_set.rs` (`RecipientOperation`, `Cause`, `Outcome`, `apply`, `apply_block`, `pick_cause`, `scale`, `set_lnaddress_method`, `check_operation`, the tests), `crates/core/src/people.rs` (`Identity`), `crates/core/src/feed.rs` and `crates/core/src/feed/elements.rs` (`Person`, `Funding`, `Recipient`, `ValueBlock`).

Change: `crates/core/src/change_set.rs`, and `crates/core/tests/change_set.rs` only to add `parts: Vec::new()` to each struct literal of `Outcome`.

Do not change: `crates/core/src/people.rs`, `crates/core/src/collection.rs`, the API, the UI, the ADRs. Keep the behavior of the current operations. Do not change an assertion of a current test.

### Architecture

These choices are fixed. Do not change them.

1. Each person operation is a new variant of `RecipientOperation`. The variant holds each value of the person that the operation needs. Thus `apply(feed, wallet, operation)` stays a pure function, and a change set does not change when the person changes after it.
2. The new types, with `Serialize` and `Deserialize`:
    - `CreditMatch { identities: Vec<String>, npubs: Vec<String> }`. `CreditMatch::matches(&Person) -> bool` is true when `Identity::of(name, href, npub)` gives a key in `identities`, or when the trimmed, lowercase `npub` of the person is in `npubs`.
    - `NewPayee { name: Option<String>, recipient_type: String, address: String, custom_key: Option<String>, custom_value: Option<String>, percent: u64 }`.
    - `NewCredit { name: String, role: String, img: String, href: String, npub: String }`. An empty text gives no attribute.
    - `NewSupportLink { label: String, url: String }`.
    - `Part` with the values `Payee`, `Credit` and `SupportLink`.
    - `PartOutcome { part: Part, changed: bool, cause: Option<Cause> }`.
3. The new variants:
    - `UseAddress { address: String, custom_key: Option<String>, custom_value: Option<String>, recipient_type: String }`.
    - `RenamePerson { name: String, payment: bool, credit: bool, credits: CreditMatch }`.
    - `AddToReleases { payee: Option<NewPayee>, credit: Option<NewCredit>, support_link: Option<NewSupportLink>, credits: CreditMatch }`.
    - `RemovePerson { payee: bool, credit: bool, credits: CreditMatch }`.
4. `Outcome::Changed` and `Outcome::NotChanged` get the field `parts: Vec<PartOutcome>`, with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. Only `AddToReleases` fills it. A stored outcome from before this packet reads with an empty list.
5. New causes: `NoCredit` (no credit of the person), `HasCredit` (the channel has a credit of the person), `HasSupportLink` (the channel has a support link with the same URL) and `NothingSelected`.
6. `UseAddress`, for each value block of the channel, of each item and of each live item:
    - Each recipient that the wallet matches gets the new `address`, `customKey`, `customValue` and `type`. Its `name`, `split` and `fee` stay.
    - Then two or more recipients of the block with the new key and the same `fee` merge into the first of them. Its split is the sum of their splits. When a split is not a non-negative integer, the recipients do not merge.
    - A block with an `lnaddress` result gets the method of ADR 0095 item 13, through `set_lnaddress_method`.
    - A block with no recipient of the wallet gives `NoRecipient`. A block whose recipients of the wallet all have the new key already, with nothing to merge, gives `HasNewAddress`.
7. `RenamePerson`:
    - With `payment`, the `name` of each recipient that the wallet matches becomes `name`.
    - With `credit`, the name text of each credit that `credits` matches becomes `name`, in the channel, in each item and in each live item.
    - A feed with no change gives `NoRecipient` when only `payment` is set, `NoCredit` when only `credit` is set, else `NoRecipient`.
8. `AddToReleases` acts on the channel only:
    - The payee part does what `Add` does with the same values, for the channel value blocks only. The wallet gives the check "has the recipient already".
    - The credit part adds a `podcast:person` after the last channel credit, or at the end of the channel credits when it has none. A channel credit that `credits` matches gives `HasCredit`.
    - The support link part adds a `podcast:funding` to the channel. A channel funding with the same trimmed URL gives `HasSupportLink`.
    - Each selected part gives one `PartOutcome`. The outcome is `Changed` when one or more parts changed. Else it is `NotChanged`, with the cause of the first part.
9. `RemovePerson`:
    - With `payee`, it does what `Remove` does with the wallet, in each place.
    - With `credit`, it removes each credit that `credits` matches, in each place.
    - A feed with no change gives the cause of the payee part when it was selected, else `NoCredit`.
10. `check_operation` refuses `RenamePerson` and `RemovePerson` with no selected part, and `AddToReleases` with no part, with `NothingSelected` as an invalid operation. It refuses `RenamePerson` with an empty name, and `UseAddress` and a `NewPayee` with no type or no address.
11. The counts of `Outcome::Changed` include the changed credits and support links: `channel` is true for a change in the channel, and `items` and `live_items` count the places with a change.

### Acceptance Criteria

Mechanical:

- A core test: `UseAddress` changes each recipient of each wallet member. It keeps `name`, `split` and `fee`. Two recipients of the person in one block merge, with the sum of their splits. A fee recipient and a recipient with no fee do not merge. An `lnaddress` result writes `method="lnaddress"`.
- A core test: `RenamePerson` changes only the selected names: the payment names, the credit names, or both. A credit of a different person keeps its name.
- A core test: `AddToReleases` adds the payee, the credit and the support link to a channel that has none, with three changed parts. A channel that has all three gives `NotChanged` with three parts and the causes `HasRecipient`, `HasCredit` and `HasSupportLink`.
- A core test: `RemovePerson` removes the selected items only, and scales the splits as `Remove` does.
- A core test: an outcome JSON from before this packet reads, with an empty `parts`.
- A core test: `check_operation` refuses an operation with no part, and a rename with an empty name.
- The tests of the operations that exist now pass with no change.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md (items 20 to 23)
- docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md (items 10 to 17)
- docs/tasks/studio-v1-task-064-person-operations.md (the Words section and packet 064a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 064a names.

Goal:
- The change set can apply the four operations of a person to a feed model.

Constraints:
- Follow the Architecture section of packet 064a fully.
- The operations that exist now keep their behavior.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- Each file other than `crates/core/src/change_set.rs`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 064a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 064b: The Change Sets Of A Person

### Files

Inspect: `crates/core/src/change_set.rs` (after 064a: `make_change_set`, `ChangeSet`, `change_sets`, `find_change_set`, `discard_change_set`), `crates/core/src/collection.rs` (`MIGRATIONS`, the change set rows), `crates/core/src/people.rs` (`person`, `default_address`, `rename_with_wallets`, `change_person`), `crates/core/src/keysend.rs` (`wallet_key`, `members`), `crates/api/src/people.rs`, `crates/api/src/routes.rs` (`make_change_set`, `change_set_json`, `files_feed`, `ensure_draft`), `crates/api/src/lib.rs`.

Change: `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, `crates/api/src/people.rs`, `crates/api/src/routes.rs` (only `change_set_json`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/core/tests/change_set.rs`.

Do not change: the UI, the ADRs, `crates/core/src/pipeline.rs`.

### Architecture

These choices are fixed. Do not change them.

1. A new migration at the end of `MIGRATIONS`: `ALTER TABLE change_sets ADD COLUMN person_id INTEGER REFERENCES people(id) ON DELETE SET NULL;`. `ChangeSet` gets `person: Option<PersonId>`.
2. `PersonOperation` is the request of the user, with `Deserialize`:
    - `UseAddress { key: RecipientKey }`
    - `Rename { name: String, payment: bool, credit: bool }`
    - `AddToReleases { payee_percent: Option<u64>, credit: bool, support_link: Option<i64> }`. `support_link` is the ID of a support link of the person.
    - `Remove { payee: bool, credit: bool }`
3. `person_operation(collection, id, &PersonOperation, lookups) -> Result<(Wallet, RecipientOperation), PipelineError>` makes the snapshot of 064a item 1:
    - The wallet of the person, as the Words section says.
    - The `CreditMatch` from the credit names and the keys of the person.
    - `UseAddress`: the key must be a member of a wallet of the person. Else the request is invalid. The type is the type of the wallet row with that key, else `lnaddress` for an address with `@`, else `node`.
    - `AddToReleases`: the payee is the default address of the person, with its type and the name of the person. A person with no default address and a `payee_percent` gives an invalid request. The credit is the name, the usual role, `img`, `href` and the first key of the person. The support link must be a link of the person.
4. `make_person_change_set(collection, start, id, &PersonOperation, feeds) -> Result<ChangeSet, PipelineError>` calls `person_operation`, then the same code as `make_change_set`, and stores `person_id`. Put the shared code in one private function. Do not copy it.
5. When a `RenamePerson` change set changes one or more feeds, the core sets the name of the person (ADR 0098 item 23). Discard does not set it back. No other person operation changes a person.
6. The routes, in `crates/api/src/people.rs`:

    | Route | Result |
    | --- | --- |
    | `POST /people/{id}/changes` | 201 with the change set JSON. The body is `{ "operation", "feed_ids" }` |
    | `POST /people/{id}/changes/preview` | 200 with `{ "feeds": [{ "feed_id", "outcome" }] }`. The same body |

    The preview applies the snapshot operation to a copy of the model of each feed: the draft, else the library copy, as `files_feed` gives it. It saves nothing and makes no draft. A feed with no model gives `cannot_start` as its outcome.
7. `change_set_json` gives `person_id`. The routes of `/change-sets` stay as they are, and a person change set is in their list.
8. An empty `feed_ids` gives 422 `invalid_change_set`, as for `POST /change-sets`. An invalid operation gives 422 `invalid_person_operation`.

### Acceptance Criteria

Mechanical:

- A core test: `person_operation` makes the wallet of a person with two wallets, and the `CreditMatch` of its credit names and keys.
- A core test: `make_person_change_set` with Use this address changes a feed that pays the first wallet and a feed that pays the second wallet, and stores `person_id`.
- A core test: a Rename change set sets the name of the person when a feed changed, and Discard keeps it. A Rename that changes no feed keeps the name.
- A core test: a person with no default address and a payee part gives an invalid request.
- An API test: `POST /people/{id}/changes/preview` gives the outcomes and changes no draft. `POST /people/{id}/changes` then makes the change set, and `GET /change-sets` lists it with `person_id`.
- An API test: Add to all releases with the payee and the credit gives two parts for each feed.
- The tests of 054, 060, 061 and 063 pass with no change.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md (items 20 to 23)
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 7 to 12)
- docs/tasks/studio-v1-task-064-person-operations.md (the Words section and packet 064b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 064b names.

Goal:
- A person operation makes a change set for the selected feeds, and the API gives a preview of its result.

Constraints:
- Follow the Architecture section of packet 064b fully.
- `make_change_set` and `make_person_change_set` share one private function.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/core/src/pipeline.rs`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 064b.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 064c: The lud16 Of A Profile And The Export

### Files

Inspect: `crates/core/src/people.rs`, `crates/core/src/profiles.rs` (`lightning_addresses`, `SOURCE_LIMIT`), `crates/core/src/keysend.rs` (`request`, `refresh`, `wallet_key`, `members`), `crates/api/src/people.rs`, `crates/api/src/routes.rs` (`lnaddress_suggestions`, `blocking_unlocked`, `keysend_map`), `crates/api/src/workspace.rs` (`lookup_relays`, `keysend_scheme`, `net_policy`), `crates/api/src/tests.rs` (the tests of ADR 0097, with the test relay and `TestServer`).

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Do not change: `crates/core/src/change_set.rs` (packet 064a changes it at the same time), `crates/core/src/collection.rs`, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `LinkEvidence` with the values `User`, `Keysend` and `Nostr`, stored as `user`, `keysend` and `nostr`. `add_wallet` gets the parameter `evidence: LinkEvidence`. Each caller that exists now gives `User`.
2. `profile_addresses(web, policy, relays, scheme, limit, &Person, lookups) -> Vec<ProfileAddress>` (ADR 0098 items 17 to 19):
    - It reads the `lud16` of the newest kind 0 of each key of the person, through `profiles::lightning_addresses`.
    - It leaves out an address that is a member of a wallet of the person already.
    - It makes the keysend lookup of each other address with `keysend::request`. It does not call `refresh`, so the collection keeps no lookup of a suggestion (ADR 0097 item 3).
    - It gives only an address with a `Found` result. `ProfileAddress` has `address`, `wallet: RecipientKey` (the key of the lookup) and `in_person_wallet: bool`, true when that key is the wallet key of a wallet of the person.
    - Each request follows the network policy of the shell (ADR 0057).
3. `add_profile_address(collection, web, scheme, id, address)`: it calls `keysend::refresh` for the address, which stores the lookup. With a `Found` result, it calls `add_wallet` with the key of the address, the type `lnaddress` and `LinkEvidence::Nostr`. Another result gives an invalid request. A conflict of the wallet gives `Conflict`.
4. `export(collection, lookups) -> PeopleExport` (ADR 0098 items 24 and 25), with `Serialize`:
    - `format: "studio-people"`, `version: 1` and `exported_at`.
    - Each person, with its fields, its default address and its links. Each wallet, key and credit name has its `evidence` and `linked_at`.
    - Each wallet lists the other addresses of the wallet from the stored lookups, each with `evidence: "keysend"` and the `checked_at` of the lookup.
    - Each membership has the feed ID, the title and the `podcast:guid`.
    - The export holds no secret. The tables of the people hold none.
5. The routes:

    | Route | Result |
    | --- | --- |
    | `GET /people/{id}/profile-addresses` | `{ "suggestions": [...] }`. A shell with no keysend scheme gives an empty list |
    | `POST /people/{id}/profile-addresses` | 201 with `{ "id" }`. The body is `{ "address" }` |
    | `GET /people/export` | The export JSON, with `Content-Disposition: attachment; filename="studio-people.json"` |

    The GET of the profile addresses makes its requests outside the workspace lock, as `lnaddress_suggestions` does. The static route `/people/export` must win over `/people/{id}`.

### Acceptance Criteria

Mechanical:

- A core test with the test relay and `TestServer`: the `lud16` of a profile is a suggestion only when its lookup gives `found`. After `profile_addresses`, the collection has no lookup of it.
- A core test: an address that is a member of a wallet of the person already is not a suggestion.
- An API test: `POST /people/{id}/profile-addresses` adds the wallet with the evidence `nostr`, and the collection then has the lookup.
- An API test: the export gives each person with the evidence of each link, the keysend addresses of each wallet, and no secret. The answer has the `Content-Disposition` header.
- No test sends a request to a host outside the computer (ADR 0045).
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md (items 17 to 19, 24 and 25)
- docs/adr/0097-studio-suggests-the-lightning-address-of-a-wallet.md
- docs/tasks/studio-v1-task-064-person-operations.md (the Words section and packet 064c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 064c names.

Goal:
- The core suggests the Lightning address of each Nostr key of a person, the user can add it to a wallet, and the API gives an export of the people.

Constraints:
- Follow the Architecture section of packet 064c fully.
- The collection keeps no lookup of a suggestion that the user did not add.
- No test sends a request to a host outside the computer (ADR 0045).
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 064c.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
