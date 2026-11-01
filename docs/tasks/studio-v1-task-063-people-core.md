# Studio v1 Task 063: The People Core

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decisions: [ADR 0098](../adr/0098-the-people-module.md), [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md), [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0027](../adr/0027-the-recipient-picker.md), [ADR 0014](../adr/0014-collection-in-sqlite.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: In progress. Packets 063a and 063b are complete. Notes from the review of 063b:

- The SQL of the people is in `people.rs` through `Collection::db`, as in `apps.rs` and `grant.rs`. Only the migration, `known_recipients` and the refill pass are in `collection.rs`.
- The functions that change a person take `&Collection`, because `make_change_set` has only a `&Collection`. Each `add_*` function gives the row ID, and a second add of the same link gives the same ID.
- The planning model changed `known_recipients`: a wallet of a person always gives the name of the person, also after a newer read of a feed.

## Goal

The core keeps the people of one user, with their links. It keeps an index of the credits of each feed. It makes suggestions from the feeds, and it makes a person from a suggestion only on a request of the user. The API gives the people and the suggestions. The UI does not change in this task: the Recipients page of ADR 0091 stays until task 065.

## The Tasks Of The People Module

| Task | Contents | ADR items |
| --- | --- | --- |
| 063 (this task) | The credit index, the tables of the people, the migration of the saved recipients, the suggestions and the API routes | ADR 0098 items 1 to 16 |
| 064 | The operations of a person, the export and the `lud16` of a Nostr profile | ADR 0098 items 17 to 25 |
| 065 | The People destination, the page of a person, "Where used", the sheets and the words | ADR 0099 items 1 to 18 |
| 066 | The recipient picker, the credits picker and the support links picker | ADR 0099 items 19 to 21 |

The planning model writes the packets of each task after the commit of the task before it.

## Packets

The planning model examines each packet before its commit. The packets go in this sequence, because each packet adds a migration or uses the code of the packet before it:

1. 063a: the credit index.
2. 063b: the tables of the people, the migration and the bridge of the saved recipients.
3. 063c: the suggestions.
4. 063d: the API routes.

No packet changes the UI. Thus no packet needs a screenshot visual check. Task 065 has the visual checks of ADR 0099.

Human test: none for this task. The human tests of ADR 0098 and ADR 0099 need the screens of task 065.

## Words

- A **credit** is one `podcast:person` element of a channel, of an item or of a live item.
- An **identity** is the key of a credit (ADR 0098 item 12): the `npub` when the credit has one, else the name with the information address (`href`), else the name.
- A **normal name** is a name with no spaces at its two ends. Each series of spaces in it is one space, and each letter is lowercase. Two names are the same when their normal names are equal.
- A **wallet key** is the `RecipientKey` that `keysend::wallet_key` gives for a key and the stored lookups (ADR 0095 item 5).
- A **link** of a person is a wallet, a Nostr key, a credit name, a support link or a membership of a publisher feed (ADR 0098 item 6).

## Packet 063a: The Credit Index

### Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `record_state`, `put_recipients`, `RecipientRow`, `known_recipients`), `crates/core/src/feed.rs` (`Feed::recipients`, `push_recipients`, the `persons` fields of `Feed`, `Item` and `LiveItem`), `crates/core/src/feed/elements.rs` (`Person`).

Change: `crates/core/src/collection.rs`, `crates/core/src/feed.rs` (only the new `Feed::credits`).

Do not change: `crates/core/src/change_set.rs`, `crates/core/src/pipeline.rs`, the API, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `Feed::credits(&self) -> Vec<&Person>` gives each `podcast:person` of the channel, of each item and of each live item, in that sequence. A person with no name and no `npub` is not in the list. A name of only spaces is no name.
2. A new migration at the end of `MIGRATIONS`:

    ```sql
    CREATE TABLE feed_credits (
        feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
        name TEXT NOT NULL DEFAULT '',
        role TEXT NOT NULL DEFAULT '',
        grp TEXT NOT NULL DEFAULT '',
        img TEXT NOT NULL DEFAULT '',
        href TEXT NOT NULL DEFAULT '',
        npub TEXT NOT NULL DEFAULT '',
        seen_at INTEGER NOT NULL,
        PRIMARY KEY (feed_id, name, role, grp, href, npub)
    );
    ```

3. `put_credits(db, id, credits: Vec<&Person>)` replaces the rows of the feed, as `put_recipients` does. Each text is trimmed. A missing attribute is empty text. The table keeps the raw role text, not the default `host` of the specification. Two credits with the same primary key give one row, and the last one gives `img`.
4. `record_state` calls `put_credits` after `put_recipients`, in the same transaction.
5. A collection with feeds from before the migration has no credit rows. The migration fills no row, because SQL cannot read a feed. The rows come with the next read or write of each feed. Do not add a refill pass in this packet. Packet 063b adds it.
6. `CreditRow` has the public fields `feed_id: FeedId`, `name`, `role`, `group`, `img`, `href` and `npub`, each a `String`. `Collection::feed_credits() -> Result<Vec<CreditRow>, CollectionError>` gives each row, by feed ID and then by name.

### Acceptance Criteria

Mechanical:

- A core test: `Feed::credits` gives the persons of the channel, of an item and of a live item. It leaves out a person with no name and no `npub`.
- A core test: `record_state` of a feed with three credits, two of them equal, gives two `feed_credits` rows. A second `record_state` with one credit gives one row.
- A core test: the delete of a feed deletes its credit rows.
- A core test: an empty role stays empty text, not `host`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md (items 9 and 10)
- docs/tasks/studio-v1-task-063-people-core.md (the Words section and packet 063a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 063a names.

Goal:
- The core keeps an index of the credits of each feed, as `feed_recipients` keeps the recipients.

Constraints:
- Follow the Architecture section of packet 063a fully.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, `crates/core/src/pipeline.rs`, the API, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 063a.

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

## Packet 063b: The People And The Bridge Of The Saved Recipients

### Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `known_recipients`, `KnownRecipient`, `RecipientAgg`, `save_recipient`, `saved_recipient_row`, `change_saved_recipient`, `restore_saved_recipient`, `remove_saved_recipient`, `SavedRecipientRow`, the change set rows), `crates/core/src/change_set.rs` (`make_change_set`, `change_saved_recipient`, `SavedBefore`, `discard_change_set`, `RecipientKey`, `Wallet`), `crates/core/src/keysend.rs` (`wallet_key`, `members`), `crates/core/src/library.rs` (the read of a library copy), `crates/api/src/routes.rs` (`list_recipients`, `group_wallets`, `save_recipient`, `remove_saved_recipient`), `crates/api/src/tests.rs` (the tests of the saved recipients).

Change: `crates/core/src/people.rs` (new), `crates/core/src/lib.rs` (the module), `crates/core/src/collection.rs`, `crates/core/src/change_set.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`.

Do not change: the UI, the ADRs, `crates/core/src/pipeline.rs`, the JSON names of `GET /recipients`.

### Architecture

These choices are fixed. Do not change them.

1. A new migration at the end of `MIGRATIONS` makes these tables. No table holds a secret (ADR 0098 item 9).

    ```sql
    CREATE TABLE people (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        usual_role TEXT NOT NULL DEFAULT '',
        img TEXT NOT NULL DEFAULT '',
        href TEXT NOT NULL DEFAULT '',
        default_address TEXT,
        default_custom_key TEXT NOT NULL DEFAULT '',
        default_custom_value TEXT NOT NULL DEFAULT '',
        created_at INTEGER NOT NULL
    );
    CREATE TABLE person_wallets (
        id INTEGER PRIMARY KEY,
        person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
        address TEXT NOT NULL,
        custom_key TEXT NOT NULL DEFAULT '',
        custom_value TEXT NOT NULL DEFAULT '',
        recipient_type TEXT NOT NULL DEFAULT '',
        position INTEGER NOT NULL,
        evidence TEXT NOT NULL,
        linked_at INTEGER NOT NULL,
        UNIQUE (address, custom_key, custom_value)
    );
    CREATE TABLE person_keys (
        id INTEGER PRIMARY KEY,
        person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
        npub TEXT NOT NULL UNIQUE,
        evidence TEXT NOT NULL,
        linked_at INTEGER NOT NULL
    );
    CREATE TABLE person_credit_names (
        id INTEGER PRIMARY KEY,
        person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
        identity TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL DEFAULT '',
        href TEXT NOT NULL DEFAULT '',
        npub TEXT NOT NULL DEFAULT '',
        evidence TEXT NOT NULL,
        linked_at INTEGER NOT NULL
    );
    CREATE TABLE person_support_links (
        id INTEGER PRIMARY KEY,
        person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
        label TEXT NOT NULL DEFAULT '',
        url TEXT NOT NULL,
        position INTEGER NOT NULL
    );
    CREATE TABLE person_members (
        person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
        feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
        PRIMARY KEY (person_id, feed_id)
    );
    CREATE TABLE person_dismissals (
        identity TEXT PRIMARY KEY,
        items TEXT NOT NULL,
        dismissed_at INTEGER NOT NULL
    );
    ```

    A row of `person_wallets` holds the key that the user linked. The keysend lookup gives the other addresses of that wallet (ADR 0098 item 6). The `evidence` text is `user`, `keysend` or `nostr`. In this task, each link has `user`. Task 064 uses the other two values for the export.
2. The same migration moves the saved recipients (ADR 0098 item 11):
    - Each row of `saved_recipients` gives one person with the same ID. The name of the person is the trimmed name of the row, else the address. `created_at` is `saved_at`.
    - Each person gets one wallet with the key and the type of its row, `position` 0, `evidence` `user` and `linked_at` `saved_at`.
    - An exception keeps a wallet in a maximum of one person. A saved row S with a `found` lookup whose key is the key of a different saved row T makes no person. Its key becomes the second wallet of the person of T, with `position` 1. When the name of T is empty, the person gets the name of S.
    - Then the migration deletes each row of `saved_recipients`. The table stays, with no row, and no code reads it after this packet.
3. After the migrations, `Collection::open` fills `feed_credits` for each feed that has no credit row and has a library copy. It reads the library copy and calls `put_credits`. A feed with no library copy, or with a copy that does not parse, gets no row and causes no error. This is the refill pass that packet 063a did not add. It runs only one time, because the migration version records it: put it in a function `refill_credits` that `migrate` calls when it applies the migration of this packet.
4. `crates/core/src/people.rs` has the types and the functions of the people. The SQL is in `collection.rs`, as for `keysend.rs`. The types:
    - `PersonId(pub i64)`.
    - `Identity` with the variants `Npub(String)`, `NameHref { name: String, href: String }` and `Name(String)`. `Identity::of(name: &str, href: &str, npub: &str) -> Option<Identity>`: a trimmed, lowercase `npub` that is not empty gives `Npub`. Else a normal name and a trimmed `href`, both not empty, give `NameHref`. Else a normal name that is not empty gives `Name`, and other text gives `None`.
    - `Identity::key(&self) -> String` gives `npub:{npub}`, `href:{name}\u{1f}{href}` or `name:{name}`, with the normal name.
    - `normal_name(name: &str) -> String`, as the Words section says.
    - `Person`: `id`, `name`, `usual_role`, `img`, `href`, `default_address: Option<RecipientKey>`, `wallets: Vec<PersonWallet>`, `keys: Vec<PersonKey>`, `credit_names: Vec<CreditName>`, `support_links: Vec<SupportLink>` and `members: Vec<FeedId>`. Each link type has its row ID, its fields and, for a wallet, a key and a credit name, its `evidence` and `linked_at`.
    - `PersonFields`: `name`, `usual_role`, `img`, `href`. A name of only spaces is refused with `CollectionErrorKind::Invalid`.
5. The functions of `people.rs` take a `&Collection` or `&mut Collection`. Each change is one transaction:
    - `people(collection) -> Vec<Person>`, A to Z by normal name, then by ID. `person(collection, id) -> Person`, with `CollectionErrorKind::NotFound` for an ID that does not exist.
    - `add_person(collection, &PersonFields) -> PersonId`, `change_person(collection, id, &PersonFields)`, `remove_person(collection, id)`.
    - `add_wallet(collection, id, &RecipientKey, recipient_type: &str)`. It calculates the wallet key of the new key with the stored lookups. It refuses with the new kind `CollectionErrorKind::Conflict` when a wallet of a different person has the same wallet key. A second key of the same wallet in the same person is permitted. The `position` is the next number of the person.
    - `remove_wallet(collection, id, wallet_id)`. When the default address of the person is an address of that wallet, the default goes back to `None`.
    - `add_key(collection, id, npub)`: the `npub` is trimmed and lowercase, and it must start with `npub1`. A key of a different person gives `Conflict`. `remove_key`.
    - `add_credit_name(collection, id, name, href, npub)`: the identity of `Identity::of`. No identity gives `Invalid`. An identity of a different person gives `Conflict`. `remove_credit_name`.
    - `add_support_link(collection, id, label, url)`: a URL that does not start with `https://` or `http://` gives `Invalid`. `remove_support_link`.
    - `add_member(collection, id, feed_id)`: a feed that is not a publisher feed gives `Invalid`. `remove_member`.
    - `set_default_address(collection, id, Option<&RecipientKey>)`: the key must be a member of a wallet of the person, with the stored lookups. Else `Invalid`.
    - `default_address(person, lookups) -> Option<RecipientKey>`: the stored default when it is still a member of a wallet. Else the key of the first wallet by `position` (ADR 0098 item 7).
    - `person_of_wallet(collection, &RecipientKey, lookups) -> Option<PersonId>`: the person that has a wallet with the same wallet key.
6. The bridge of the saved recipients. The JSON of `GET /recipients` keeps its names until task 065:
    - `known_recipients` reads `person_wallets` in the place of `saved_recipients`. `KnownRecipient.saved_id` keeps its name and holds the person ID. The name of a wallet row is the name of its person.
    - `POST /recipients/saved` calls the new function `people::save_recipient(collection, &Recipient) -> PersonId`. When the wallet key of the recipient is in a person, it changes nothing and gives that person. Else it makes a person with the name of the recipient, or the address when the name is empty, and one wallet.
    - `DELETE /recipients/saved/{id}` removes the person with that ID.
    - Remove from `collection.rs`: `save_recipient`, `saved_recipient_row`, `change_saved_recipient`, `restore_saved_recipient`, `remove_saved_recipient` and `SavedRecipientRow`. Code that no entry point reaches is deleted.
7. The change sets follow ADR 0098 item 23:
    - Remove `change_saved_recipient` and `SavedBefore` from `change_set.rs`. A change set does not keep a person, and Discard does not change a person.
    - When a Rename change set changes one or more feeds, `make_change_set` sets the name of each person with a wallet of the change set. Discard does not set the name back.
    - Change address, Remove, Add and Use the Lightning address do not change a person.
    - The column `saved_before` of `change_sets` stays. No code writes it or reads it. Remove the functions of `collection.rs` that only read or write it.
8. Change the API test `rename_and_change_address_change_the_saved_recipient` to the rules of item 7: Rename changes the name of the person, Discard keeps the new name, and Change address does not change the wallet of the person. Give the test a name that tells this.

### Acceptance Criteria

Mechanical:

- A core test: the migration makes one person from each saved recipient, with its name and its wallet, and `saved_recipients` has no row after it.
- A core test: the migration puts a saved Lightning address with a `found` lookup into the person of the saved node key of that lookup. Thus it makes one person with two wallets.
- A core test: a wallet is in a maximum of one person. `add_wallet` with a Lightning address whose lookup gives the wallet key of a different person gives `Conflict`. The same call on the same person succeeds.
- A core test: `add_key`, `add_credit_name` and `add_member` refuse the cases of item 5.
- A core test: `default_address` gives the first wallet when no default is set, and the stored default when it is set. `remove_wallet` of the wallet of the default gives the first wallet again.
- A core test: `remove_person` removes each link of the person.
- A core test: the refill pass fills `feed_credits` for a feed with a library copy and no credit row.
- A core test: a Rename change set sets the name of the person of the wallet. Discard keeps the new name. Change address does not change the wallets of the person.
- An API test: `POST /recipients/saved` makes a person, and `GET /recipients` gives `saved_id` with that person ID. A second save of the same key gives the same ID. `DELETE /recipients/saved/{id}` removes the person.
- The tests of 054, 060 and 061 pass, with the changes of item 8 only.
- The corpus tests pass with no change.
- The four commit checks pass.

### Escalation

- Stop and report when a Node test of the UI fails. This packet must not change the JSON that the UI reads.
- Stop and report when the migration of item 2 cannot be written in SQL alone.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md (items 1 to 5)
- docs/tasks/studio-v1-task-063-people-core.md (the Words section and packet 063b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 063b names.

Goal:
- The core keeps the people of one user and their links. The saved recipients become people, and the routes of the saved recipients use the people.

Constraints:
- Follow the Architecture section of packet 063b fully.
- The JSON of `GET /recipients` keeps its names.
- No test sends a request to a host outside the computer (ADR 0045).
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/core/src/pipeline.rs`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 063b.

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

## Packet 063c: The Suggestions

### Files

Inspect: `crates/core/src/people.rs`, `crates/core/src/collection.rs` (`feed_credits`, `known_recipients`, `keysend_lookups`, the tables of packet 063b), `crates/core/src/keysend.rs` (`wallet_key`).

Change: `crates/core/src/people.rs`, `crates/core/src/collection.rs` (only the SQL of the dismissals and the reads that the suggestions need).

Do not change: `crates/core/src/change_set.rs`, the API, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The sources are the index of the credits, the index of the recipients (`feed_recipients`) and the stored lookups. The suggestions do not read a feed model or a draft (ADR 0098 items 10 and 12).
2. A suggestion starts from the credits. The accepted mockup of 2026-11-01 shows a credit in each suggestion, and a payee with no credit is in "Payees with no person" (ADR 0099 item 2). A group of recipients with no credit is not a suggestion.
3. `suggestions(collection, lookups) -> Vec<Suggestion>` does these steps:
    1. It gives each credit row its identity with `Identity::of`. A row with no identity is ignored. A row whose identity is a credit name of a person, or whose `npub` is a key of a person, is ignored (ADR 0098 item 16).
    2. It groups the rows by `Identity::key`. The name of a group is its most frequent trimmed name. With a tie, the first name A to Z.
    3. A recipient joins a group when its normal name is the normal name of the group. A recipient can join more than one group. The user selects the items in the sheet.
    4. Then a recipient also joins a group when its wallet key is the wallet key of a recipient of that group.
    5. A recipient whose wallet key is in a person is ignored.
4. An **item** of a suggestion is a distinct credit `(name, href, npub)` with the raw texts, or a distinct recipient key. A credit in two feeds is one item.
5. `Suggestion` has these fields:
    - `identity: String`, the key of the group.
    - `name: String`.
    - `credits: Vec<SuggestedCredit>`: `name`, `href`, `npub`, `img`, `roles: Vec<String>` (distinct, A to Z), `feeds: Vec<FeedId>` and `same_name: bool`.
    - `recipients: Vec<SuggestedRecipient>`: `key: RecipientKey`, `name`, `recipient_type`, `feeds: Vec<FeedId>`, `same_name: bool` and `joined_by: JoinedBy` (`Name` or `Wallet`).
    - `credit_feeds: usize` and `payee_feeds: usize`: the number of distinct feeds of the credits and of the recipients.
    - `evidence: Vec<Evidence>`, with `SameNpub` when the identity is an `npub`, `SameName` when a recipient joined by name, and `SameWallet` when a recipient joined by wallet. The sequence is `SameNpub`, `SameName`, `SameWallet`.

    `same_name` is false when the normal name of the item is not the normal name of the group. The sheet of ADR 0099 item 3 starts such an item not selected.
6. The sequence of the list: the most feeds first (`credit_feeds + payee_feeds`), then by normal name.
7. The dismissals (ADR 0098 item 15):
    - `dismiss(collection, identity, items: &[String])` inserts or replaces a row of `person_dismissals`. `items` is a JSON array of the item keys. An item key is `credit:{name}\u{1f}{href}\u{1f}{npub}` or `recipient:{address}\u{1f}{custom_key}\u{1f}{custom_value}`, with the raw texts.
    - `suggestions` leaves out a group with a dismissal row when each item key of the group is in the row. A group with a new item key comes back.
    - `Suggestion` gets `item_keys: Vec<String>`, so that the API can send them back with a dismissal.
8. `make_person(collection, &MakePerson, lookups) -> PersonId` (ADR 0098 item 14). `MakePerson` has `name`, `credits: Vec<(String, String, String)>` (name, href, npub), `recipients: Vec<RecipientKey>` with the type of each, `keys: Vec<String>` and `member_of: Option<FeedId>`. In one transaction, it:
    - makes the person with the name;
    - adds a credit name for each credit, and a key for each `npub` of the credits and of `keys`, with no duplicate;
    - adds a wallet for each recipient. Two keys with the same wallet key are two rows of the same person;
    - sets the usual role to the most frequent role of the selected credits in `feed_credits`, and `img` and `href` to the first selected credit that has one;
    - adds the membership when `member_of` is set.

    A `Conflict` of one link cancels the full transaction. The core makes no person and no link in `suggestions` or in `dismiss`.

### Acceptance Criteria

Mechanical:

- A core test: two credits with the same `npub` and different names make one suggestion with `SameNpub`. The credit with the name that is not the most frequent has `same_name: false`.
- A core test: a credit "Mary Kate Ultra" and a recipient "mary kate  ultra" make one suggestion with `SameName`. A second recipient with a different name and the same wallet key joins with `SameWallet` and `same_name: false`.
- A core test: a credit with a name and an `href`, and a credit with the same name and no `href`, make two suggestions.
- A core test: a group of recipients with no credit is not a suggestion.
- A core test: `credit_feeds` and `payee_feeds` count distinct feeds. A credit in two feeds is one item.
- A core test: a dismissed suggestion does not come back. A new recipient with the same name brings it back.
- A core test: a credit name or a key of a person, and a wallet of a person, are not in a suggestion.
- A core test: `suggestions` and `dismiss` make no person and no link. `make_person` makes the person with the selected links only, and a `Conflict` makes no person.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md (items 12 to 16)
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 2 and 3)
- docs/tasks/studio-v1-task-063-people-core.md (the Words section and packet 063c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 063c names.

Goal:
- The core suggests people from the credits and the recipients of the feeds, keeps the dismissals, and makes a person from the items that the user selects.

Constraints:
- Follow the Architecture section of packet 063c fully.
- The core makes no link without a request of the user.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, the API, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 063c.

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

## Packet 063d: The API Routes

### Files

Inspect: `crates/api/src/lib.rs` (the router), `crates/api/src/routes.rs` (`list_recipients`, `keysend_map`, `blocking`, `ApiError`, the mapping of `CollectionErrorKind`), `crates/api/src/tests.rs` (the helpers of the recipient tests), `crates/core/src/people.rs`.

Change: `crates/api/src/lib.rs`, `crates/api/src/routes.rs` or a new `crates/api/src/people.rs`, `crates/api/src/error.rs` (the map of `CollectionErrorKind::Conflict`), `crates/api/src/tests.rs`.

Do not change: the core, the UI, the ADRs. The one exception is item 3, and `serde` derives on the types of `people.rs` that the routes send or receive.

### Architecture

These choices are fixed. Do not change them.

1. The routes are in the router of `studio-api`. Thus the desktop shell and the server shell serve them, with the access rules of each shell (ADR 0017, ADR 0055).
2. The routes:

    | Route | Result |
    | --- | --- |
    | `GET /people` | Each person, with `default_address` and the counts `pays` and `credited` |
    | `POST /people` | 201 with `{ "id" }`. The body is `PersonFields` |
    | `GET /people/{id}` | One person with each link, `default_address`, and the lists `pays` and `credited` of feed IDs |
    | `PATCH /people/{id}` | 204. The body is `PersonFields` |
    | `DELETE /people/{id}` | 204 |
    | `PUT /people/{id}/default-address` | 204. The body is a recipient key or `null` |
    | `POST /people/{id}/{kind}` | 201 with `{ "id" }`. `kind` is `wallets`, `keys`, `credit-names`, `support-links` or `members` |
    | `DELETE /people/{id}/{kind}/{link_id}` | 204. For `members`, `link_id` is the feed ID |
    | `GET /people/suggestions` | Each suggestion of packet 063c |
    | `POST /people/suggestions/make` | 201 with `{ "id" }`. The body is `MakePerson` |
    | `POST /people/suggestions/dismiss` | 204. The body is `{ "identity", "items" }` |

3. `pays` is each feed whose `feed_recipients` has a member of a wallet of the person. `credited` is each feed whose `feed_credits` has an identity of a credit name, or an `npub` of a key, of the person. A core function `people::uses(collection, &Person, lookups) -> PersonUses` gives them. Put it in `people.rs`: this is the one change of the core in this packet.
4. The errors: `NotFound` gives 404 `not_found`, `Invalid` gives 422 `invalid_person`, and `Conflict` gives 409 `link_in_other_person`. The body of a 409 names the person that has the link.
5. No route makes a keysend lookup or a relay request. Task 064 adds the `lud16` requests. The routes use the stored lookups.
6. The JSON uses snake case names. A recipient key has `address`, `custom_key` and `custom_value`, as in `GET /recipients`.

### Acceptance Criteria

Mechanical:

- An API test: `POST /people`, then `GET /people/{id}`, `PATCH` and `DELETE` give the expected results.
- An API test: each `kind` of `POST /people/{id}/{kind}` adds a link, and its `DELETE` removes it.
- An API test: a wallet of a different person gives 409 `link_in_other_person`.
- An API test: two feeds of the collection, one with a credit and one with a recipient of the same name, give one suggestion with `credit_feeds: 1` and `payee_feeds: 1`. `make` makes the person, and the suggestion is not in the next list. `GET /people/{id}` gives `pays` and `credited` with one feed each.
- An API test: `dismiss` removes the suggestion from the list.
- An API test in server mode: `GET /people` with no session gives 401, as each other `/api/` route.
- No route test calls a `TestServer` route for a keysend lookup.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0098-the-people-module.md
- docs/tasks/studio-v1-task-063-people-core.md (the Words section and packet 063d only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 063d names.

Goal:
- The API gives the people, their links, their uses and the suggestions, in the two shells.

Constraints:
- Follow the Architecture section of packet 063d fully.
- The only change of the core is `people::uses`.
- No route makes a request to a host.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, the core outside `people::uses`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 063d.

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
