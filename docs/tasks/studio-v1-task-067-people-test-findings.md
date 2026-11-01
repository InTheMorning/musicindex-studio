# Studio v1 Task 067: The Findings Of The First People Test

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decisions: [ADR 0103](../adr/0103-one-person-collects-the-items-of-many-suggestions.md), [ADR 0102](../adr/0102-delete-a-person-changes-no-feed.md), [ADR 0098](../adr/0098-the-people-module.md) items 6, 7 and 17 to 19, [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) items 3, 4 and 15, [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md) items 12 to 15, [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md) and [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md).

Status: In progress. Packet 067a is complete.

## Goal

The operator tested `0.1.0-rc.9` on `studio.musicindex.org` on 2026-11-01 and found six gaps. This task closes them, and ADR 0103 adds the ways to collect one person:

- a. One artist gave three suggestions "HeyCitizen", and a suggestion could only make a new person. ADR 0103 makes one suggestion of each name, and adds "Add to a Person", "More from your feeds" and "Merge With a Person".
- b. Studio shows a Nostr key and an address in a short form, with no way to copy them.
- c. The suggestion of a `lud16` says "Suggested from the Nostr profile", but not which profile.
- d. The menu "Default address" did not offer the keysend form of an Alby wallet. A wallet lists only the addresses that a feed uses.
- e. A Nostr key of a person shows "Added by you", not its profile. ADR 0099 item 4 asks for the result of its check, and the accepted mockup shows the profile name.
- f. The operator could not delete a person from People. The page has only "Remove", which removes the person from feeds. ADR 0102 adds "Delete Person" and renames the operation to "Remove From Feeds".

## Packets

The planning model examines each packet before its commit. The packets go in this sequence, because 067a to 067c change `people.rs`, and 067d and 067e change the UI:

1. 067a: the suggestions of ADR 0103 items 1 to 7, and the members of each wallet (core and API).
2. 067b: the add to a person, the items that match a person, and the merge of two people (core and API).
3. 067c: the profiles of the keys, and the key of each `lud16` suggestion (core and API).
4. 067d: the UI of items b to f, and the warning of ADR 0103 item 18.
5. 067e: the UI of "Add to a Person", "More from your feeds" and "Merge With a Person".

Visual check of 067d and 067e, by the planning model, before each commit. It uses the method of task 065, and the screens of `.studio-dev/mockup-101/site/merge.html` are the reference.

Human test, by the operator, on `studio.musicindex.org` after the deploy:

1. Open the person "HeyCitizen" that rc.9 made. "More from your feeds" lists the real Nostr key and the credits of the same name. Add them.
2. The key of "City Radio" shows its warning. Remove it. Each key shows its profile name, and a copy button copies the full `npub`.
3. The menu "Default address" offers the keysend form of the Alby wallet. Select it.
4. In People, a suggestion "HeyCitizen" with items that the person does not have goes to the person with "Add to a Person".
5. Make a second person, then merge it into "HeyCitizen" with "Merge With a Person". Then delete a test person with "Delete Person". No feed gets a draft.
6. A `lud16` suggestion names its profile and its key.

## Words

- The **members** of a wallet are the keys of `keysend::members` for the wallet key: the node key with its custom pair, and each Lightning address whose stored lookup gives that key (ADR 0095 item 5).
- To **resolve** a key is to read the newest kind 0 of the key from the lookup relays and to check its NIP-05 address. `profiles::find` does the same for a key query (ADR 0074).

## Packet 067a: The Suggestions Of ADR 0103 And The Members Of A Wallet

### Files

Inspect: `crates/core/src/people.rs` (`suggestions`, `credit_groups`, `join_recipients`, `Identity`, `normal_name`, `dismiss`, the suggestion tests, `default_address`, `is_member`, `set_default_address`), `crates/core/src/keysend.rs` (`wallet_key`, `members`), `crates/api/src/people.rs` (`GET /people/{id}`), `crates/api/src/tests.rs` (the suggestion tests).

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/tests.rs`.

Do not change: `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `suggestions` groups the credits as ADR 0103 items 1 to 3 say. The key of a group is `name:{normal name}`, or `npub:{npub}` for a group of item 2 or item 3. `Suggestion.identity` gives that key, and `dismiss` keeps it.
2. Payees join as ADR 0103 item 4 says. The join through a wallet stays as it is.
3. The exclusions of ADR 0098 item 16 stay. A credit whose `Identity::of` key is a credit name of a person, or whose `npub` is a key of a person, is in no group. A payee whose wallet key is in a person is in no group.
4. The evidence follows ADR 0103 item 6. `same_name` of an item is true when its normal name is the normal name of the group. A credit with no name has `same_name: true`, as now.
5. A suggestion of an `npub` with no name of item 3 uses its `npub` as its name, as now.
6. `people.rs` gets `wallet_members(wallet: &PersonWallet, lookups) -> Vec<WalletMember>`. `WalletMember` has `key: RecipientKey` and `recipient_type: String`. The list is `keysend::members` of the wallet key of `wallet.key`, and it always holds `wallet.key`. The member `wallet.key` has the type of the wallet row. Each other member has `lnaddress` for an address with `@`, else `node`.
7. `GET /people/{id}` gives `members` for each wallet. `default_address` and `set_default_address` stay as they are. They accept each member already, through `is_member`.

### Acceptance Criteria

Mechanical:

- The core tests of the acceptance criteria of ADR 0103.
- The core tests of the suggestions that exist now change only where ADR 0103 changes the result.
- A core test: take a wallet of a Lightning address with a `found` lookup. `wallet_members` gives the Lightning address with the type `lnaddress`, and the node key with its custom pair and the type `node`.
- An API test: `GET /people/{id}` gives `members` for each wallet. `PUT /people/{id}/default-address` with the node key of item 6 gives 204, and `GET` then gives that node key as `default_address`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0103-one-person-collects-the-items-of-many-suggestions.md (items 1 to 7)
- docs/adr/0098-the-people-module.md (items 12 to 16)
- docs/tasks/studio-v1-task-067-people-test-findings.md (the Goal, the Words section and packet 067a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 067a names.

Goal:
- One person gives one suggestion, and the API gives each member of each wallet of a person.

Constraints:
- Follow the Architecture section of packet 067a fully.
- The core makes no link without a request of the user.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 067a.

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

## Packet 067b: Add To A Person, More From Your Feeds And Merge

### Files

Inspect: `crates/core/src/people.rs` (`make_person`, `MakePerson`, `insert_credit_name`, `insert_key`, `insert_wallet`, `uses`, `suggestions`, `remove_person`), `crates/core/src/collection.rs` (the table `change_sets` and its column `person_id`), `crates/api/src/people.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Do not change: `crates/core/src/change_set.rs`, `crates/core/src/collection.rs` (the SQL of this packet goes through `Collection::db` in `people.rs`), the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `add_to_person(collection, id, &PersonItems, lookups) -> AddedItems` (ADR 0103 items 8 to 10). `PersonItems` has `credits`, `recipients` and `keys` in the form of `MakePerson`. In one transaction it adds each item that the person does not have, with the helpers of `make_person`. It changes no field of the person. A conflict of one item adds no item. `AddedItems` gives the number of new links.
2. `more_items(collection, &Person, lookups) -> Vec<MoreItem>` (ADR 0103 items 11 to 13). A `MoreItem` is a credit (`name`, `href`, `npub`, `roles`, `feeds`) or a payee (`key`, `name`, `recipient_type`, `feeds`), with its `cause`: `SameName`, `SameCreditName`, `SameNpub` or `SameWallet`. ADR 0103 item 11 decides the list. An item that is a link of a person is not in the list. The function reads only the collection.
3. `merge_people(collection, keep: PersonId, other: PersonId)` (ADR 0103 items 14 to 16). In one transaction it moves each wallet, key, credit name, support link and membership of `other` to `keep`. A membership that `keep` has already is not copied two times. Then it sets `person_id = keep` on each change set of `other`, and deletes `other`. It keeps each field and the default address of `keep`. `keep` equal to `other` gives `Invalid`.
4. The routes:

    | Route | Result |
    | --- | --- |
    | `POST /people/{id}/items` | 201 with `{ "added": N }`. The body is `PersonItems`, in the array form of `POST /people/suggestions/make` |
    | `GET /people/{id}/more` | `{ "items": [...] }` with the items of item 2 |
    | `POST /people/{id}/merge` | 204. The body is `{ "other": ID }` |

    The routes give the error codes of the people routes: 404 `not_found`, 422 `invalid_person` and 409 `link_in_other_person`.

### Acceptance Criteria

Mechanical:

- A core test: `add_to_person` adds only the new items, keeps the name and the default address, and adds nothing on a conflict.
- A core test: `more_items` gives a credit of the same name, a credit of a credit name, a credit with a key of the person, a payee of the same name and a payee of the same wallet, each with its cause. A link of a person is not in the list.
- A core test: `merge_people` moves each link, keeps the fields of `keep`, gives the change sets of `other` to `keep`, and deletes `other`.
- An API test for each route of item 4, with one error case each.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0103-one-person-collects-the-items-of-many-suggestions.md (items 8 to 16)
- docs/tasks/studio-v1-task-067-people-test-findings.md (the Goal, the Words section and packet 067b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 067b names.

Goal:
- The core and the API add items to a person, give the items of the feeds that match a person, and merge two people.

Constraints:
- Follow the Architecture section of packet 067b fully.
- The core makes no link without a request of the user. No operation changes a feed.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 067b.

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

## Packet 067c: The Profiles Of The Keys

### Files

Inspect: `crates/core/src/profiles.rs` (`find`, `Context`, `Profile`, `newest_profiles`, `newest_metadata`, `meta_of`, `check_addresses`, `lightning_addresses`, `lud16_of`, the tests with the test relay), `crates/core/src/people.rs` (`profile_addresses`, `ProfileAddress`), `crates/api/src/routes.rs` (`nostr_profiles`, `blocking_unlocked`), `crates/api/src/people.rs` (`list_profile_addresses`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs` (the tests of the profile search and of the profile addresses).

Change: `crates/core/src/profiles.rs`, `crates/core/src/people.rs` (only `profile_addresses` and `ProfileAddress`), `crates/api/src/routes.rs` or `crates/api/src/people.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Do not change: `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `profiles::resolve(cx: &Context<'_>, npubs: &[String]) -> Vec<Profile>` resolves each valid `npub`, one time each, in the sequence of the list. It does what `find` does for a key query, for each key: the newest kind 0 from the lookup relays, the NIP-05 check and `nip05_ok`. `domain_match` is false, because no feed is given. `sources` is empty. A key with no kind 0 gives a `Profile` with no name, no picture and no NIP-05. A text that is not an `npub` is ignored. Put the steps that `find` and `resolve` share in one private function. Do not copy them.
2. `POST /nostr/resolve` takes `{ "npubs": [...] }` with a maximum of 50 keys, and gives `{ "profiles": [...] }`. It makes its requests outside the workspace lock, with the network policy of the shell, as `nostr_profiles` does. It stores nothing. More than 50 keys give 422 `invalid_query`.
3. `profiles::lightning_addresses` gets a sibling, `lightning_addresses_by_key(relays, policy, limit, npubs) -> Vec<(String, String)>`, which gives the `npub` and the `lud16` of each key that has one. `lightning_addresses` keeps its result and uses the sibling.
4. `ProfileAddress` gets `npub: String`, the key whose profile gave the address. When two keys give one address, the first key wins. `profile_addresses` uses the sibling of item 3.

### Acceptance Criteria

Mechanical:

- A core test with the test relay: `resolve` gives the name and the picture of the newest kind 0 of each key, a profile with no name for a key with no kind 0, and nothing for a text that is not an `npub`.
- A core test with `TestServer`: `resolve` gives `nip05_ok: true` for a NIP-05 address whose domain gives the key, and `false` for one whose domain gives a different key.
- A core test with the test relay and `TestServer`: `profile_addresses` gives the `npub` of each suggestion.
- An API test: `POST /nostr/resolve` gives the profiles, and 51 keys give 422.
- No test sends a request to a host outside the computer (ADR 0045).
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md (items 12 to 15)
- docs/adr/0098-the-people-module.md (items 17 to 19)
- docs/tasks/studio-v1-task-067-people-test-findings.md (the Goal, the Words section and packet 067c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 067c names.

Goal:
- The API resolves a list of Nostr keys to their profiles, and each `lud16` suggestion names its key.

Constraints:
- Follow the Architecture section of packet 067c fully.
- Each request follows the network policy of the shell (ADR 0057). No test sends a request to a host outside the computer (ADR 0045).
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 067c.

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

## Packet 067d: The UI Of Items b To f

### Files

Inspect: `crates/api/ui/pages.js` (the page `person`), `crates/api/ui/confirms.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`, `crates/api/ui/peoplepages.js` (`PersonPage`, the profile section, `AddressRow`, `MakePersonSheet`, the wallet groups and the `lud16` suggestions), `crates/api/ui/people.js` (`walletGroups`, `addressOptions`, `memberLine`, `keyItemRow`, `keyEvidenceText`), `crates/api/ui/app.js` (the copy of the feed address near `navigator.clipboard`), `crates/api/ui/editor.js` (`NostrRow`), `crates/api/ui/styles.css`, `crates/api/ui-tests/people.test.mjs`.

Change: `crates/api/ui/pages.js`, `crates/api/ui/confirms.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/app.js` (only a shared copy button), `crates/api/ui/editor.js` (only the copy button in `NostrRow`), `crates/api/ui/styles.css`, `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`.

Do not change: `values.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `app.js` exports `CopyButton({ text, label })`: an icon button with the `aria-label` "Copy {label}", which writes `text` to the clipboard and shows "Copied" for 2 seconds. The copy of the feed address uses it. It has the target size of its profile (musicindex ADR 0029 R-SCALE-3 and ADR 0024 R-TARGET-1).
2. Each Nostr key and each address of the People screens has a `CopyButton` with the full value: the keys and the wallet addresses of the page of a person, the key rows of "Make a Person", and the `lud16` suggestions. `NostrRow` of the editor gets one for a key that is set.
3. The page of a person and the sheet "Make a Person" send `POST /nostr/resolve` with their keys. A key row shows the picture and the profile name, the short key, and "NIP-05 checked" when `nip05_ok` is true. While the request runs, and when it fails, the row shows the short key only. A key with no profile shows "No Nostr profile found". A key whose profile name does not contain the normal name of the person or of the suggestion shows the warning of ADR 0103 item 18. In "Make a Person" that key starts not selected.
4. A `lud16` suggestion reads "From the Nostr profile {name} ({short key}) · keysend checked", with the name from item 3. With no name it reads "From the Nostr profile {short key} · keysend checked".
5. A wallet group lists each member of `members` of `GET /people/{id}`, with the address lines of ADR 0099 item 15. The menu "Default address" lists each member of each wallet. `walletGroups` uses `members` and keeps the provider names from `GET /recipients` when an address is there.
6. The page `person` of `pages.js` gets the last section `delete` with the tier `limits`. It holds "Delete Person" (destructive), with the confirmation of ADR 0102 item 2 in `confirms.js`. After the confirmation, the page sends `DELETE /people/{id}` and opens the People page. The operation "Remove" reads "Remove From Feeds" on its button and as its sheet title (ADR 0102 item 4).
7. The model functions of items 3 to 6 are in `people.js`, with Node tests. No feed rule is in the UI (ADR 0065).

### Acceptance Criteria

Mechanical:

- A Node test: `walletGroups` gives each member of `members`, with the node member of an Alby wallet, and `addressOptions` gives it in the menu.
- A Node test: the key row text gives the profile name and "NIP-05 checked", the short key with no profile, and "No Nostr profile found".
- A Node test: the warning of ADR 0103 item 18 for a profile name that differs, no warning for a profile name that contains the name, and no warning with no profile. Such a key starts not selected in `makePersonStart`.
- A Node test: the `lud16` line names the profile and the key.
- The Node tests of the acceptance criteria of ADR 0102.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The page of a person with a resolved key, a copy button on each key and address, a wallet with two members, and a `lud16` suggestion.
- The sheet "Make a Person" with its key rows.
- "Delete Person" at the end of the page of a person, and its confirmation sheet.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 3, 4 and 15)
- docs/adr/0102-delete-a-person-changes-no-feed.md
- docs/tasks/studio-v1-task-067-people-test-findings.md (the Goal, the Words section and packet 067d only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 067d names.

Goal:
- The People screens resolve each Nostr key to its profile, copy each key and address, name the profile of a `lud16` suggestion, and offer each member of a wallet as the default address. The page of a person can delete the person with no change of a feed.

Constraints:
- Follow the Architecture section of packet 067d fully.
- No feed rule in the UI (ADR 0065).
- Logic that a test examines is in `people.js`, not in a component (ADR 0042).
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `values.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 067d. The planning model does the visual check.

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

## Packet 067e: The Screens To Collect One Person

### Files

Inspect: `crates/api/ui/peoplepages.js` (after 067d), `crates/api/ui/people.js`, `crates/api/ui/pages.js` (the page `person`), `crates/api/src/people.rs` (the routes of 067b), `.studio-dev/mockup-101/site/merge.js` (the four screens).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/pages.js`, `crates/api/ui/styles.css` (only new rules that the screens need), `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/pages.test.mjs`.

Do not change: `values.js`, `editor.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The sheet "Make a Person" gets "Add to a Person" (action) between "Not This Person" and "Make the Person" (ADR 0103 item 8). It opens the sheet "Add to a Person: {name}" with the select menu "Person" of each person, A to Z.
2. That sheet lists each item of the suggestion (ADR 0103 item 9). An item that the selected person has already shows "Already in {person}" with no check box. Each other item shows "New", with the start selection of "Make a Person". The button reads "Add 1 Item" or "Add N Items" and sends `POST /people/{id}/items`. Then the page of the person opens.
3. The page `person` of `pages.js` gets the section `more` with the tier `content`, before `payment` (ADR 0103 item 11). It shows `GET /people/{id}/more`. Each row gives the item, its feeds and its cause, with "Add" (action), which sends `POST /people/{id}/items` with that one item. With no item, the section does not show.
4. The last section of the page gets "Merge With a Person" (normal) before "Delete Person" (ADR 0103 item 14). It opens the sheet "Merge With {name}": the select menu "Person to merge", the links of the selected person, the result line, and the button "Merge the People" (primary). The button sends `POST /people/{id}/merge`, then reloads the page.
5. The key rows of these sheets and of the section use the profile row and the warning of packet 067d.
6. The model functions are in `people.js`, with Node tests: the rows of "Add to a Person" with "New" and "Already in", the body of `POST /people/{id}/items`, the rows and the cause texts of "More from your feeds", and the links and the result line of the merge sheet. No feed rule is in the UI (ADR 0065).

### Acceptance Criteria

Mechanical:

- A Node test for each model function of item 6.
- A Node test: `pageSequence('person')` gives `more` with the tier `content` before `payment`.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The four screens of the mockups: "Make a Person" with "Add to a Person", "Add to a Person", the page of a person with "More from your feeds", and "Merge With a Person".

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0103-one-person-collects-the-items-of-many-suggestions.md (items 8 to 18)
- docs/tasks/studio-v1-task-067-people-test-findings.md (the Goal, the Words section and packet 067e only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 067e names.

Goal:
- The user collects one person from many suggestions: "Add to a Person", "More from your feeds" and "Merge With a Person".

Constraints:
- Follow the Architecture section of packet 067e fully.
- No feed rule in the UI (ADR 0065).
- Logic that a test examines is in `people.js`, not in a component (ADR 0042).
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `values.js`, `editor.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 067e. The planning model does the visual check.

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
