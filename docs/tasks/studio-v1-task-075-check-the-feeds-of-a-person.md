# Studio v1 Task 075: Check The Feeds Of A Person

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0111](../adr/0111-the-page-of-a-person-checks-the-feeds.md). It uses [ADR 0098](../adr/0098-the-people-module.md) items 6, 10 and 20, [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0103](../adr/0103-one-person-collects-the-items-of-many-suggestions.md) items 9 to 11 and [ADR 0112](../adr/0112-a-credit-role-comes-from-a-list.md).

Status: Complete, in three packets, with the screenshot visual checks of 075b and 075c on 2026-11-02. The human test is open. Notes from the reviews:

- 075a: the check also counts a credit with a key of the person, as `uses` does. The payee findings need a default address. A feed that neither pays nor credits the person is not in the check.
- 075b: no user guide page covered People. Packet 075c added `docs/user-guide/people.md`.
- 075c: the core preview gives only the parts of a feed that change. The planning model added the lines of the findings that the selected changes correct to each feed row of "Correct the Feeds". The lines come from the check of the core, so they are a display aid (ADR 0065).

## Goal

The page of a person shows the edits first and the feed operations lower. "Check your feeds" compares each feed with the person, and "Correct N Feeds" corrects the selected feeds in one change set. One sheet adds a suggestion to a new person or to a person that exists. A payee with no person can go to a person.

## Packets

The planning model examines each packet before its commit. The packets go in this sequence:

1. 075a: the check and the operation "Correct" (core and API).
2. 075b: the page of a person and the page "Check your feeds" (UI).
3. 075c: the sheet "Check: {name}", the payee page, "Add a Wallet" and the sheet "Correct the Feeds" (core field and UI).

Visual check of 075b and 075c, by the planning model, before each commit. The reference is `.studio-dev/mockup-people-2/site/`, screens 1 to 7. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, on `studio.musicindex.org` after the deploy:

1. Open HeyCitizen, then "Check your feeds". Each credit with the key of City Radio is in "Needs a correction".
2. Select "Correct N Feeds", keep "Nostr key" and "Payment address", review and write.
3. "Check your feeds" then shows each feed as correct, and each payment uses the node address.
4. Open a suggestion with the name of a person. The sheet starts on that person, and "Add N Items" adds the items.
5. Open a payee with no person, and add it to a person with "Add to a Person".

## Words

- A **credit of the person** follows ADR 0111 item 10. It has the identity of a credit name of the person (ADR 0098 item 6), or the normal name of the person or of a credit name. Use `normal_name` of `people.rs` for the normal name.
- A **recipient of the person** has a key that is a member of a wallet of the person, through `keysend::wallet_key`. `people::uses` finds them now.
- A **payee of the name** has the normal name of the person or of a credit name. Its wallet is not a wallet of the person.

## Packet 075a: The Check And The Operation "Correct"

### Files

Inspect: `crates/core/src/people.rs` (`uses`, `PersonUses`, `FeedPlaces`, `normal_name`, `Identity`, `wallet_members`, `default_address`), `crates/core/src/change_set.rs` (`PersonOperation`, `person_operation`, `CreditMatch`, `RecipientOperation`, `apply`, `Outcome`), `crates/core/src/collection.rs` (`feed_credits`, `CreditRow`, the index of the recipients), `crates/core/src/roles.rs` (from task 074), `crates/api/src/people.rs` (`GET /people/{id}`, `POST /people/{id}/changes` and its preview), `crates/api/src/lib.rs`.

Change: `crates/core/src/people.rs`, `crates/core/src/change_set.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `people.rs` gets `check(collection, person, lookups) -> Result<PersonCheck, CollectionError>`. It reads only the indexes of the collection and the stored lookups. It makes no request.
2. `PersonCheck` has `feeds: Vec<FeedCheck>`. `FeedCheck` has `feed_id`, `state` and `findings: Vec<Finding>`. `state` is `needs_correction`, `not_default` or `correct`. A feed with an error finding is `needs_correction`. A feed with only the neutral finding is `not_default`.
3. `Finding` is an enum, serialized with `kind`, with these variants:
    - `credit_key { role, npub }`: a credit of the person with no key (`npub` empty), or with a key that is not a key of the person. The core makes this check only when the person has a key.
    - `credit_details { role, img, href }`: a credit of the person with a picture or an information address that differs from a value of the person. Each field is true when it differs. The core compares a field only when the person has that value.
    - `credit_role`: a credit of the person with no role.
    - `not_default { key }`: a recipient of the person that does not use the default address. This is the neutral finding.
    - `other_wallet { key }`: a payee of the name.
4. `GET /api/v1/people/{id}/check` gives the `PersonCheck`. `GET /people/{id}` also gives `needs_correction`, the number of feeds with that state.
5. `PersonOperation` gets `Correct { key: Option<String>, payment: bool, details: bool, role: bool, other_payees: bool }`:
    - `key`: each credit of the person gets this `npub`. It must be a key of the person.
    - `payment`: each recipient of the person uses the default address, as `UseAddress` does.
    - `details`: each credit of the person gets the picture and the information address of the person, when the person has them.
    - `role`: each credit of the person with no role gets the usual role and the usual group, as task 074 gives.
    - `other_payees`: each payee of the name becomes the default address. The `name`, the `split` and `fee` stay.
6. The operation keeps the role of each credit that has a role (ADR 0111 item 17). The core applies it as the other person operations, and the change set writes each feed through `pipeline::write_draft` with the ID of its own diff.
7. The preview of `POST /people/{id}/changes/preview` gives the changes of each feed for `Correct`. A feed with no change is "Not changed", and the UI leaves it out.
8. The credit match of `Correct` uses the rule of a credit of the person. The other operations keep `CreditMatch` as it is.

### Acceptance Criteria

Mechanical:

- A core test: `check` finds a credit with the key of a different account by its name. It also finds a credit with no key, a picture that differs, and a credit with no role.
- A core test: `check` gives `not_default` for a recipient of a different member of a wallet of the person, and `other_wallet` for a payee of the name.
- A core test: a feed that agrees with the person is `correct`. A person with no key gives no `credit_key` finding.
- A core test: `Correct` applies only the selected changes. It keeps the role of a credit that has one, and it gives "Not changed" for a feed with no change.
- A core test: `Correct` with `other_payees` keeps the split and the fee of each payee.
- An API test: `GET /people/{id}/check` gives the states, and `GET /people/{id}` gives `needs_correction`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0111-the-page-of-a-person-checks-the-feeds.md (items 9 to 17)
- docs/adr/0098-the-people-module.md (items 6, 10 and 20)
- docs/tasks/studio-v1-task-075-check-the-feeds-of-a-person.md (the Goal, the Words section and packet 075a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 075a names.

Goal:
- The core compares each feed of a person with the person, and the operation "Correct" corrects the selected feeds.

Constraints:
- Follow the Architecture section of packet 075a fully.
- The check makes no network request.
- Each write goes through a change set and `pipeline::write_draft`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 075a.

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

## Packet 075b: The Page Of A Person And "Check Your Feeds"

### Files

Inspect: `crates/api/ui/peoplepages.js` (`PersonPage`, `PersonFields`, `WhereUsedPage`, `OperationSheet`), `crates/api/ui/people.js` (`profileLine`, `whereRow`, `feedRows`, `memberChoices`, `personFieldsBody`), `crates/api/ui/pages.js` (the page `person`), `crates/api/ui/shell.js` (`PEOPLE_VIEWS`, the view `where-used`), `crates/api/ui/app.js` (the view `where-used`, `ListRow`), `.studio-dev/mockup-people-2/site/people.js` (`person2`, `changeFeeds`, `checkPage`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/pages.js`, `crates/api/ui/shell.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css` only if the visual check needs it, the Node tests, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `pages.js` gives the page `person` the sequence of ADR 0111 item 1: `profile` and `check` (identity), `more` and `payment` (content), `credits`, `support` and `member` (details), `change` (setup), then `delete` (limits).
2. The header shows the usual role, then each membership as the type dot of the feed, a link with its title, and its first role. The Nostr key line stays.
3. Each field of `PersonFields` shows the status "Saved" after a save that succeeded. The hint of the first field is "Studio saves each field when you leave it." A model function of `people.js` gives the status text from the save state. A Node test examines it.
4. The section `member` ("Artist or label") has the select menu "Member of" with `memberChoices`, each label with the first role of the feed. Its hint is "The artist or label that this person is in. Studio changes no feed." A change adds the new membership and removes the old one, through the routes that exist.
5. The section `change` ("Change your feeds") has one navigation row for each operation, with the lines of the mockup. It has the note "Each operation shows the feeds and a review before Studio writes." Each row opens the sheet of its operation. The page has no operation button at its top.
6. The view `where-used` becomes the view `check`, and `WhereUsedPage` becomes `CheckPage`. The navigation row on the person page shows "N feeds need a correction" in the error color, or "Each feed is correct", and the counts of the feeds.
7. `CheckPage` reads `GET /people/{id}/check`. It groups the feeds as "Needs a correction", "Not your default address" and "Correct". A model function of `people.js` gives the lines of each finding. Each error finding shows in the error color, with the warning icon of the mockup. A key line names the profile name of the key, through the profile lookup that exists. A row opens the feed in the editor. The button "Correct N Feeds" comes in 075c, and this packet leaves its place empty.

### Acceptance Criteria

Mechanical:

- A Node test: the page sequence of the page `person` follows the tiers.
- A Node test: the status text after a save that succeeded, after a failure, and before a save.
- A Node test: the lines of each finding kind, and the group of each state.
- The four commit checks pass.

Visual: screens 1 and 6 of the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0111-the-page-of-a-person-checks-the-feeds.md (items 1 to 4 and 9 to 13)
- docs/tasks/studio-v1-task-075-check-the-feeds-of-a-person.md (the Goal, the Words section and packet 075b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 075b names.

Goal:
- The page of a person shows the edits first, "Change your feeds" lower, and the row "Check your feeds" that opens the page of the check.

Constraints:
- Follow the Architecture section of packet 075b fully.
- No feed rule in the UI (ADR 0065). The findings come from the core.
- Each button label is in title case. Each other text is in sentence case.
- An error uses the `--error` token.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 075b.

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

## Packet 075c: The Sheets

### Files

Inspect: `crates/core/src/people.rs` (`Suggestion`, `suggestions`, `make_person`, `add_items`), `crates/api/src/people.rs` (`GET /people/suggestions`), `crates/api/ui/peoplepages.js` (the sheets "Make a Person" and "Add to a Person", `WalletPage`, `LinkSheet`, `OperationSheet`, `CheckPage`), `crates/api/ui/people.js` (`makePersonStart`, `addToRows`, `personChoices`, `payeesWithNoPerson`, `sheetBody`), `crates/api/ui/recipients.js` (`walletActions`), `.studio-dev/mockup-people-2/site/people.js` (screens 2 to 5 and 7).

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/recipients.js`, `crates/api/ui/styles.css` only if the visual check needs it, the core, API and Node tests, `docs/user-guide/people.md`.

Do not change: `crates/core/src/change_set.rs`, the ADRs, `crates/server`.

### Architecture

These choices are fixed. Do not change them.

1. `Suggestion` gets `person_id: Option<PersonId>`: the person whose normal name, or one of whose credit names, has the normal name of the suggestion. When two people match, the core gives the person with the lowest ID.
2. The sheet "Check: {name}" replaces the sheets "Make a Person" and "Add to a Person". Its first field is the select menu "Person": "A new person", then each person, A to Z. It starts on `person_id`, else on "A new person". The hint for a match is "You have a person with this name. Studio adds the selected items to it."
3. With a person, the sheet lists the items as ADR 0103 item 9 gives, and its primary button reads "Add 1 Item" or "Add N Items". It uses `POST /people/{id}/items`. With "A new person", it shows "Name" and "Member of", and its primary button reads "Make the Person". It uses the route of the make that exists. "Not This Person" stays.
4. `WalletPage` gets the section "Person" with the navigation row "Add to a Person". Its sheet "Add to a Person: {name}" has the menu "Person" of item 2, and "Name" for a new person. A new person uses the route that makes a person, then the route that adds the wallet. The operations of the wallet move to a section "Change your feeds", as rows with one line each.
5. The sheet "Add a Wallet" lists `payeesWithNoPerson` first, each as a row. A row adds its wallet to the person. The form for a different address follows.
6. `CheckPage` gets the button "Correct N Feeds" (primary). It opens the sheet "Correct the Feeds: {name}", with the check boxes "Nostr key", "Payment address", "Picture and information address", "Role", and "Payees with this name that pay a different wallet". The last one starts not selected. A person with more than one key gets the menu "Key".
7. The sheet reads the preview of the operation `Correct` for the selected changes. It lists each feed with its changes and a check box, and it leaves out each feed with no change. The primary button reads "Review in N Feeds", and the review is the review of the change sets.

### Acceptance Criteria

Mechanical:

- A core test: a suggestion with the normal name of a person, or of one of its credit names, gives that `person_id`.
- A Node test: the start state of the sheet "Check: {name}" with and without `person_id`, and the label of its primary button.
- A Node test: the start state of the sheet "Correct the Feeds", with "Payees with this name that pay a different wallet" not selected.
- A Node test: the rows of "Add a Wallet" list the payees with no person first.
- The four commit checks pass.

Visual: screens 2 to 5 and 7 of the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0111-the-page-of-a-person-checks-the-feeds.md (items 5 to 8 and 14 to 17)
- docs/adr/0103-one-person-collects-the-items-of-many-suggestions.md (items 9 and 10)
- docs/tasks/studio-v1-task-075-check-the-feeds-of-a-person.md (the Goal, the Words section and packet 075c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 075c names.

Goal:
- One sheet adds a suggestion to a new person or to a person that exists, a payee can go to a person, and "Correct the Feeds" corrects the selected feeds.

Constraints:
- Follow the Architecture section of packet 075c fully.
- No feed rule in the UI (ADR 0065). The changes of each feed come from the preview of the core.
- Each button label is in title case. Each other text is in sentence case.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.

Do not touch:
- `crates/core/src/change_set.rs`, the ADRs, `crates/server`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 075c.

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
