# Studio v1 Task 080: Find On musicindex

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0117](../adr/0117-the-page-of-a-person-finds-values-on-musicindex.md). It uses [ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md) and [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md).

Status: In progress. [Task 079](studio-v1-task-079-person-save-and-discard.md) is committed. Packet 080a extends the save of packet 079a, and packet 080b uses the pending changes of packet 079b. The planning model examined the packets again on 2026-11-02, and changed item 2 of packet 080b.

## Goal

"Find on musicindex" on the page of a person searches the artist and label feeds of musicindex. It shows them in three levels with the dots of musicindex.org. The user selects a feed and the values to fill. The values become pending changes, and Save keeps them.

## Packets

The planning model examines each packet before its commit. 080a changes the core and the API. 080b changes the UI.

Visual check of 080b, by the planning model, before its commit: screens 2 to 5 of `.studio-dev/mockup-lookup/site/`. The check data comes from a local test server of the musicindex API. It has one row in each level, and one feed with a Lightning address route. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0117.

## Packet 080a: The Search And The Values In The Core

### Files

Inspect: `crates/core/src/musicindex.rs` (`MusicIndex::search`, `feed_routes`, `RouteData`, `Envelope`, `filter_guid`, `MAX_RESPONSE_BYTES`, the tests with `TestServer`), `crates/core/src/people.rs` (`Person`, `PersonSave` of task 079, `save_person`, `add_wallet`, `add_key`, `add_profile_address`, `LinkEvidence`), `crates/core/src/keysend.rs` (`request`, `wallet_of`), `crates/core/src/lib.rs`, `crates/api/src/people.rs` (`list_profile_addresses` and `add_profile_address`, the pattern with no workspace lock and `keysend_scheme`), `crates/api/src/workspace.rs` (`index`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs` (the route list of the token test), `../musicindex/search.html` (`normName`, `publisherNames`, `creditMatch`, `publisherKindFromRow`, `publisherRowKind`).

Change: `crates/core/src/musicindex.rs`, a new module `crates/core/src/lookup.rs`, `crates/core/src/lib.rs`, `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `MusicIndex::search_publishers(query) -> Result<Vec<PublisherHit>, WebError>` sends `GET /v1/search` with `q`, `type=publisher`, `include=link_facts` and `limit=25`. It keeps the rows with `entity_type` `publisher`. `PublisherHit` has `feed_guid`, `title`, `image_url` (from `feed_image_url`), `release_artist`, `release_artist_source`, `two_way_link_count: Option<i64>`, `agreed_roles: Vec<String>` and `confirmed_release_artists: Vec<String>`. A missing list is empty.
2. `MusicIndex::publisher_values(guid) -> Result<Option<PublisherValues>, WebError>` sends `GET /v1/feeds/{guid}` with `include=source_links,source_ids,payment_routes`. `PublisherValues` has `title`, `feed_url`, `image_url`, `website` (the `url` of the first `source_links` row with `link_type` `website`), `npub` (the `value` of the first `source_ids` row with `scheme` `nostr_npub`) and `routes: Vec<Recipient>` (from `payment_routes`, as `feed_routes` does). A 404 gives `None`.
3. The new module `lookup` has no HTTP server code. It gives:
    - `Level` with `Role`, `TwoWay` and `NameOnly`, in that sequence, and `Kind` with `Role { roles }`, `InferredArtist`, `NoRole` and `Unverified`.
    - `classify(hit) -> (Level, Kind)` with the conditions of ADR 0117 item 4. A row with no `two_way_link_count` is `NameOnly` and `Unverified`.
    - `name_match(credit, names) -> bool` with the rule of ADR 0117 item 5, as `creditMatch` and `normName` of musicindex.org do. The names are the title, and the release artist when its source is not `placeholder`.
    - `LookupRow` with `feed_guid`, `title`, `image_url`, `level`, `kind`, `albums` (the `two_way_link_count`, or `None` for `Unverified`), `host: Option<String>` and `has: Has { picture, website, wallet, npub }`. Each `has` value is `false` until item 4 reads the detail.
4. `lookup::search(index, query)` sorts the rows by level with a stable sort, so that a level keeps the sequence of the search. It reads `publisher_values` for the first 10 rows in that sequence, and sets `host` (the host of `feed_url`) and `has`. A failed detail read leaves the row with no host and no `has`. A picture or a website that is not an `http` or `https` address counts as missing.
5. `lookup::fetch_values(index, web, scheme, guid)` makes the requests, and `lookup::compose_values(collection, person, values)` reads the collection. Together they give the rows of ADR 0117 item 9. `LookupValues` has `title`, `host`, `picture`, `website`, `wallets` and `npub`. The values sheet takes the kind and the number of albums from the search row that the user selected. Each value row has `value` and `state`: `new`, `current` (equal to the value of the person after a trim), `has_wallet`, or `other_person { id, name }`.
6. A wallet row also has `recipient_type`, `split`, `key` and, for an `lnaddress`, the result of `keysend::request` with the scheme of the shell.
    - The `key` of a Lightning address is the address, as the wallet sheet links it. The node of the lookup gives the state of the row only.
    - Before the save, the save route stores the lookup of each Lightning address with `keysend::refresh`, as `add_profile_address` does. A shell with no keysend scheme gives no lookup result. Use the wallet rules of `add_profile_address` to find the person of a wallet. Do not copy them.
7. `PersonSave` of task 079 gets `wallets: Vec<WalletAdd>` (`key` and `recipient_type`) and `keys: Vec<String>`. `save_person` adds them in the same transaction, with the rules of `add_wallet` and `add_key` and the evidence `LinkEvidence::User`. Move the body of each into a private function that takes a `&Connection`, as task 079 did. A wallet of a different person fails the save.
8. `GET /api/v1/people/{id}/musicindex?q=` gives `{ rows }`. `GET /api/v1/people/{id}/musicindex/{guid}` gives the `LookupValues`, or 404. Each route reads the person under the workspace, and makes the requests with no workspace lock, as `list_profile_addresses` does. The musicindex requests use the client of the workspace, as each other musicindex request does. That client uses `NetPolicy::Open`, because the operator sets its address with `--musicindex`. The keysend requests use `w.web`, with the policy of the shell.
9. The core adds nothing to the collection, except through `save_person`.

### Acceptance Criteria

Mechanical:

- A core test, with `TestServer`: `search_publishers` sends `type=publisher` and `include=link_facts`, and reads the link facts.
- A core test: `classify` gives each level and each kind of ADR 0117 item 4.
- A core test: `name_match` gives `true` for an equal name and a name in whole words, and `false` for a part of a word.
- A core test: `search` keeps the sequence of the search in a level, and reads the detail of 10 rows only.
- A core test: `values` gives the states `new`, `current`, `has_wallet` and `other_person`, and refuses a picture that is not an `http` or `https` address.
- A core test: `save_person` adds a wallet and a key with the fields in one transaction, and a wallet of a different person changes nothing.
- An API test: the two routes give the rows and the values from a local test server, and they are in the list of the token test.
- No test sends a request to a host outside the computer (ADR 0045).
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0117-the-page-of-a-person-finds-values-on-musicindex.md
- docs/tasks/studio-v1-task-080-find-on-musicindex.md (the Goal and packet 080a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 080a names.

Goal:
- The core searches the artist and label feeds of musicindex in three levels, and gives the values of one feed.
- Save adds the selected wallets and keys. The API serves the search and the values.

Constraints:
- Follow the Architecture section of packet 080a fully.
- No test sends a request to a host outside the computer. Use `TestServer`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 080a.

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

## Packet 080b: The Two Sheets

### Files

Inspect: `crates/api/ui/peoplepages.js` (`PersonPage` and its `profile` section, the pending values of task 079, `PersonArt`, `ListRow`, `LinkSheet` for the pattern of a sheet), `crates/api/ui/people.js` (`savedValues`, `pendingKeys`, `saveBody`, `pendingStatus` of task 079), `crates/api/ui/shell.js` (`roleWord`, `feedKind`), `crates/api/ui/app.js` (`useSheet` or the sheet component of `LinkSheet`, `Field`), `crates/api/ui/styles.css` (`.dot`, `.dot.hollow`), `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/type-dot.test.mjs`, `.studio-dev/mockup-lookup/site/lookup.js` (`kind`, `lookupSearch`, `lookupPick`, `lookupWallet`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/styles.css` only if the visual check needs it, `crates/api/ui-tests/people.test.mjs`, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `people.js` gets the model of the lookup, with no Preact and no browser state:
    - `LOOKUP_LEVELS`: the heading and the note of each level of ADR 0117 items 4 and 6, in sequence.
    - `lookupKind(row) -> { dot, text }`. `Role` gives the filled dot of the first role and "Artist · N albums" or "Label · N releases", with `roleWord` of `shell.js`. `InferredArtist` gives `artist hollow` and "Inferred artist · N albums". `NoRole` gives `feed hollow` and "Feed · two-way links, no role". `Unverified` gives `feed hollow` and "Unverified link". One album or one release uses the singular.
    - `hasLine(row)`: the host, then the values, for example "wavlake.com · a picture, a website and a Lightning address". A row with no detail gives the host only, or `null`.
    - `fillRows(values, pending)`: one row for each value of ADR 0117 item 9, with `selectable`, `selected` (the start values of item 11), the text of item 12 for a row that cannot be selected, and the warning line of a wallet. A wallet row with no `key` cannot be selected.
    - `fillLabel(count)`: "Fill 1 Field" or "Fill N Fields".
    - `fillSource(row)`: "From musicindex: the {word} feed {title}", with the lowercase word of the kind of the search row that the user selected.
2. `savedValues` of task 079 gets `wallets: []` and `keys: []`. A fill adds to them. `isPending` compares these two lists by their length, not with `!==`. `pendingKeys` counts each added wallet and key, and `pendingStatus` gets that count. `saveBody(saved, values)` sends them. `LEAVE_NAMES` gets a name for an added wallet and an added key. The page changes its values only through the `update` function of task 079, which uses the current state.
3. The `profile` section gets "Find on musicindex" (`button action`) below the role line and the key lines.
4. The search sheet has the title "Find on musicindex: {name}" and a search field that starts with the name. Below the field are the hint of the mockup and one `page-section` for each level with rows. A level with no row does not show. It sends the search when it opens and 300 ms after the last change of the text. A row is a `ListRow` with the picture, the title, the type line with its dot, and `hasLine`. The only button is "Cancel".
5. A row opens the values sheet "From musicindex: {title}". It has the feed row with the type line of the search row and the link "Open on musicindex.org" to `https://musicindex.org/#feed={guid}` (`target="_blank"`, `rel="noopener"`). The section "Fill" has the check box rows of `fillRows`, then the note of ADR 0117 item 13. The buttons are "Back" (normal) and the label of `fillLabel` (primary), which is disabled with no selected row.
6. "Fill" closes the sheet and makes the pending changes. A filled field shows `fillSource` as its hint until Save or Discard.
7. The dot is a `span` with the classes of `lookupKind`. It is before the type word. Use the classes of `styles.css` and add no color.
8. A failed request shows its error in the sheet with `ErrorNotice`. The page does not change.
9. `docs/user-guide/people.md` gets a section on "Find on musicindex": the levels, the fill and Save.

### Acceptance Criteria

Mechanical:

- A Node test: `lookupKind` gives the dot and the text of each kind, with the singular and the plural.
- A Node test: `fillRows` gives the start values of ADR 0117 item 11 and the rows of item 12. `fillLabel` gives the singular and the plural.
- A Node test: a fill makes pending changes, and `saveBody` sends the added wallets and keys.
- The test of the type dot and the test of title case still pass.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0117-the-page-of-a-person-finds-values-on-musicindex.md
- docs/tasks/studio-v1-task-080-find-on-musicindex.md (the Goal and packet 080b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 080b names.

Goal:
- The page of a person has "Find on musicindex", with the search sheet in three levels and the values sheet that fills pending changes.

Constraints:
- Follow the Architecture section of packet 080b fully.
- No feed rule in the UI (ADR 0065). The levels come from the core.
- Each button label is in title case. Each other text is in sentence case.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 080b.

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
