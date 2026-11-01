# Studio v1 Task 065: The People Screens

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decisions: [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) items 1 to 18, [ADR 0098](../adr/0098-the-people-module.md), [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0092](../adr/0092-file-hosts-and-the-destination-icons.md), [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md), [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: In progress. Packet 065a is complete. Notes from the reviews:

- 065a: `GET /people` and `GET /people/{id}` also give `feeds`, the number of distinct feeds of `pays` and `credited`, because `personRow` needs it. `roles` keeps an empty role as `""`, so that a credit with no role gives "Credited". `crates/api/src/ui.rs` serves `people.js`. An address line of `people.js` gives no provider name, because `places` has no lookup data. Packet 065d gives the address lines with the provider.

## Goal

The People destination replaces the Recipients page. The user makes a person from a suggestion, edits a person, and applies the four operations of a person from sheets that list the feeds. A payee with no person keeps its page with the operations of a wallet. The words of the screens change to the words of ADR 0099.

## Packets

The planning model examines each packet before its commit. The packets go in this sequence, because 065b, 065c and 065d each change `app.js`:

1. 065a: the model module, the page of a person in `pages.js`, and the places of each feed in the API. No visible change.
2. 065b: the People destination, the People page, the sheets "Make a Person" and "Add a Person", the export and the review.
3. 065c: the page of a person, "Where used" and the four sheets of the operations.
4. 065d: the page of a wallet with no person, the words, and the removal of the old Recipients code.

Visual check, by the planning model, before each commit of 065b, 065c and 065d. The check serves Studio with test data, through a temporary test in `crates/api` that Git ignores, as the earlier UI tasks did. It takes screenshots in the two themes at 1100 px, and at 390 px with touch. It runs `$edges()` at 1100 px, and `$fieldText()` and `$overflow()` at 390 px with touch. A packet that fails the check is not committed.

Human test, by the operator, on `studio.musicindex.org` after the deploy of task 065: the human tests of ADR 0098 and ADR 0099. The operator makes a person from a suggestion. Then the operator uses each sheet of ADR 0099 item 7 one time. One Use This Address changes a feed from a Fountain node to a second wallet, and the feed on its file host has the expected recipient and method.

## Words

- The **mockups** are `.studio-dev/mockup-098/site/people.html` and `people.js`, which the operator accepted on 2026-11-01. A screen follows its mockup, with the components and the tokens of the UI.
- A **sheet** is a `dialog` with the class `sheet`, opened through `useSheet` of `editor.js`.
- The **model module** is `crates/api/ui/people.js`. It has no Preact and no browser state (ADR 0042).
- The **screens module** is `crates/api/ui/peoplepages.js`. It holds the Preact components of the People area. It imports the shared components from `app.js`, as `values.js` does.

## Packet 065a: The Model And The Places Of Each Feed

### Files

Inspect: `crates/api/ui/recipients.js` (as a pattern), `crates/api/ui/pages.js`, `crates/api/ui-tests/recipients.test.mjs` and `crates/api/ui-tests/pages.test.mjs` (as patterns), `crates/api/src/people.rs` (`GET /people/{id}`), `crates/core/src/people.rs` (`uses`, `PersonUses`), `.studio-dev/mockup-098/site/people.js` (the texts of the rows).

Change: `crates/api/ui/people.js` (new), `crates/api/ui/pages.js`, `crates/api/ui-tests/people.test.mjs` (new), `crates/api/ui-tests/pages.test.mjs`, `crates/core/src/people.rs` (only `uses`), `crates/api/src/people.rs`, `crates/api/src/tests.rs`.

Do not change: `app.js`, `shell.js`, `recipients.js`, `values.js`, `editor.js`, `styles.css`, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `PersonUses` gets `places: Vec<FeedPlaces>`. `FeedPlaces` has `feed_id`, `pays: Vec<RecipientKey>` (each address of the person in the value blocks of the feed, each one time) and `roles: Vec<String>` (each role of a credit of the person in the feed, each one time, A to Z, with an empty role as no entry). The lists `pays` and `credited` stay. `GET /people/{id}` gives `places`. `GET /people` does not give it.
2. `pages.js` gets the page `person` (ADR 0099 item 4):

    ```js
    person: () => [
      { id: 'profile', tier: 'identity' },
      { id: 'actions', tier: 'identity' },
      { id: 'where', tier: 'identity' },
      { id: 'payment', tier: 'content' },
      { id: 'credits', tier: 'details' },
      { id: 'support', tier: 'details' },
    ],
    ```

3. `people.js` exports these functions. Each one takes plain data from the API and gives plain data:
    - `peopleSections({ people, suggestions, changes, payees })`: the IDs of the sections of the People page, in the sequence of ADR 0099 item 2: `search`, `actions`, `changes`, `suggestions`, `people`, `payees`. `changes` is left out with no change set. `suggestions` is left out with no suggestion. `payees` is left out with no payee. With no person, `suggestions` comes before `people`. With one or more people, it comes after `people`. `people` is always in the list, because it gives the empty text.
    - `PEOPLE_EMPTY`: "No people yet. Check a suggestion above, or add a person."
    - `suggestionRow(suggestion)`: `{ title, lines }`. The first line gives the counts, for example "Credit in 3 feeds · Payee in 6 feeds". A count of 0 is left out. The second line gives the evidence: "The same Nostr key is in the credits" for `SameNpub`, "The same name is in the credits and in the value blocks" for `SameName`, and "The same wallet is in the value blocks" for `SameWallet`. With two or more kinds, it gives the first. With no recipient, it gives "No payment address found".
    - `personRow(person, feedTitles)`: `{ title, lines, count }`. The first line is the usual role and "Member of {title}" of each membership, joined with " · ". The line is left out when both are empty. The second line is "Pays {line}" with the address line of the default address, and " · N wallets" when the person has two or more wallets. With no wallet, it is "No payment address". `count` is "N feeds" with the number of distinct feeds of `pays` and `credited`.
    - `makePersonStart(suggestion)`: `{ name, credits, recipients, keys, memberOf }`. Each credit and each recipient is `{ item, checked }`, and `checked` is `same_name` (ADR 0099 item 3). `keys` lists each distinct `npub` of the credits, each with `checked: true`. `memberOf` is empty text.
    - `makePersonBody(state)`: the body of `POST /people/suggestions/make`, in the form of the task 063 notes. Only checked items go in. `member_of` is a number or `null`.
    - `sheetFeeds(places, titles)`: one row for each feed of `places`: `{ id, title, lines, checked: true }`. The lines are "Pays {address}" for each address and "Credited as {role}" for each role (ADR 0099 item 5). A credit with no role gives "Credited".
    - `checkedIds(rows)` and `toggledRow(rows, id)`.
    - `reviewLabel(n)`: "Review in 1 Feed", "Review in N Feeds" (ADR 0099 item 12).
    - `operationBody(kind, fields)`: the `PersonOperation` of task 064 for `use_address`, `rename`, `add_to_releases` and `remove`.
    - `addResultLine(outcomes)`: the line of ADR 0099 item 10, from the outcomes of the preview. It gives "N releases", then "N get the payee", "N get the credit" and "N get the support link" for each selected part with one or more changes. Then it gives "N have it already" with one selected part, "N have both already" with two, or "N have all already" with three. N counts the feeds where no selected part changed and each cause starts with `has_`. A count of 0 is left out.
4. The model has no feed rule (ADR 0065). It does not calculate a split, and it does not decide which feed changes. The core gives each outcome.

### Acceptance Criteria

Mechanical:

- A Node test: `peopleSections` gives the sequence of ADR 0099 item 2. With no person, `suggestions` comes before `people`.
- A Node test: `pageSequence('person')` gives the sections of ADR 0099 item 4 in the sequence of the tiers, and no section has feed check boxes. The tier test of `pages.test.mjs` covers the new page.
- A Node test: `makePersonStart` starts an item with a different name not selected, and `makePersonBody` sends only the checked items.
- A Node test: `sheetFeeds` gives each row selected, with its lines. `reviewLabel` gives "Review in 1 Feed" and "Review in 3 Feeds".
- A Node test: `suggestionRow`, `personRow` and `addResultLine` give the texts of the mockup for the data of the mockup.
- An API test: `GET /people/{id}` gives `places` with the address and the role of the person in each feed.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md
- docs/tasks/studio-v1-task-065-people-screens.md (the Words section and packet 065a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 065a names.

Goal:
- The UI has a tested model module for the People area, and the API gives the places of a person in each feed.

Constraints:
- Follow the Architecture section of packet 065a fully.
- No feed rule in the UI (ADR 0065).
- `people.js` has no Preact and no browser state (ADR 0042).
- Each comment, each UI text and each test message is in ASD-STE100 Simplified Technical English. A button label uses title case (musicindex ADR 0023 R-CASE-1).
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill for the Rust part.

Do not touch:
- `app.js`, `shell.js`, `recipients.js`, `values.js`, `editor.js`, `styles.css`, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 065a.

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

## Packet 065b: The People Destination And The People Page

### Files

Inspect: `crates/api/ui/shell.js` (`leftPanel`, `narrowStack`, the view names), `crates/api/ui/app.js` (the icon table near `'hard-drive'`, `destinationRows`, `RecipientsPage`, `RecipientPage`, `ChangeSetPage`, the view switch near `view.name === 'recipients'`, `Bar`, `ChoiceSelect`, `FeedCheckRow`), `crates/api/ui/people.js`, `crates/api/ui/confirms.js`, `crates/api/ui/styles.css` (the `link-row`, `group` and `sheet` rules), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui-tests/destinations.test.mjs`, `.studio-dev/mockup-098/site/people.js` (the screens `first`, `people` and `confirm`).

Change: `crates/api/ui/peoplepages.js` (new), `crates/api/ui/app.js`, `crates/api/ui/shell.js`, `crates/api/ui/styles.css` (only new rules that the mockup needs), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui-tests/destinations.test.mjs`, other Node tests only where they name the destination `recipients`.

Do not change: `recipients.js` (packet 065d changes it), `values.js`, `editor.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The bottom group is People, File hosts, Settings, then Lock Studio or Unlock Studio (ADR 0099 item 1). The row `people` has the label "People" and the Lucide icon `users`. Add the icon to the icon table of `app.js` with the Lucide paths: `<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M22 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" />`.
2. The views of the People area are `people`, `person` (with `id`), `where-used` (with `id`), `wallet` (with the recipient entry, the view `recipient` of today renamed) and `change-set`. People is current on each of them. `narrowStack` gives `[root, people, view]` for `person`, `wallet` and `change-set`, and `[root, people, person, view]` for `where-used`. The view `recipients` goes.
3. The People page follows `peopleSections` of `people.js` and the screens `first` and `people` of the mockup:
    - The search field filters the rows of People and of "Payees with no person" in the browser. It matches the name, an address and a key, with no regard to capital letters. It is a display filter only.
    - "Add a Person" (action) opens the sheet "Add a Person" with the field "Name". It sends `POST /people`, then opens the page of the new person.
    - "Export People" (normal) downloads `GET /people/export`.
    - "Changes to write" lists each change set of `GET /change-sets` that is open. A row shows the operation, the name of the person or the wallet, and "N feeds to write". A row opens the review.
    - "Suggestions from your feeds" lists `GET /people/suggestions` with `suggestionRow`. The button "Check" on a row opens the sheet "Make a Person: {name}".
    - "People" lists `GET /people` with `personRow`. A row opens the page of the person. Packet 065c gives that page. Until then, the view `person` shows the name of the person and the text "Studio shows this page in the next version." Packet 065c removes that text.
    - "Payees with no person" lists each wallet of `GET /recipients` with `saved_id` `null`, with the row lines of today. A row opens the view `wallet`, which shows `RecipientPage` of today until packet 065d.
4. The sheet "Make a Person: {name}" follows the screen `confirm` of the mockup and `makePersonStart`. It lists the credits, the payment addresses and the Nostr keys with check boxes, and the select menu "Member of" with each publisher feed of the collection and the choice "None". The hint is "Studio found these items with the name "{name}". Select the items that are this person. Studio changes no feed." The buttons are "Not This Person" (normal), which sends the dismissal, and "Make the Person" (primary). After the make, the page opens the new person. A 409 shows its message in the sheet.
5. The review of a change set (`ChangeSetPage`) keeps "Write N Feeds" and "Discard" at its top, in the bar, at each scroll position (ADR 0099 item 12). Discard opens the page of the person for a change set with `person_id`, else the People page. The title of the page is "Review the changes".
6. Remove `RecipientsPage` and each call of it. Code that no entry point reaches is deleted.
7. The button roles follow ADR 0037, and each button label is in title case.

### Acceptance Criteria

Mechanical:

- A Node test: the bottom group gives People, File hosts, Settings, then Lock Studio, and People has the icon `users`. People is current on each view of item 2.
- A Node test: `narrowStack` gives the stacks of item 2.
- The tests of `shell.test.mjs` and `destinations.test.mjs` that named Recipients name People, with the same checks.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The People page on the first visit, with suggestions and no person, and with people.
- The sheet "Make a Person".
- The sheet "Add a Person".
- The review with "Write N Feeds" and "Discard" at its top after a scroll.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 1 to 3, 12 and 13)
- docs/tasks/studio-v1-task-065-people-screens.md (the Words section and packet 065b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 065b names.

Goal:
- The People destination replaces Recipients, with the People page, the sheets "Make a Person" and "Add a Person", the export and the review.

Constraints:
- Follow the Architecture section of packet 065b fully.
- No feed rule in the UI (ADR 0065).
- Logic that a test examines is in `people.js` or `shell.js`, not in a component (ADR 0042).
- Use the components and the tokens that exist. A new CSS rule uses `--text-*` tokens and no free `font-size`.
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case. Each other text uses sentence case.

Do not touch:
- `recipients.js`, `values.js`, `editor.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 065b. The planning model does the visual check.

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

## Packet 065c: The Page Of A Person And The Sheets

### Files

Inspect: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/pages.js` (the page `person`), `crates/api/ui/app.js` (`ChoiceSelect`, `Field`, `FeedCheckRow`, the old `OperationForm` as a pattern), `crates/api/ui/recipients.js` (`addressLine`), `crates/api/src/people.rs` (the routes of a person, the changes, the preview and the profile addresses), `.studio-dev/mockup-098/site/people.js` (the screens `person`, `rename`, `use`, `where` and `add`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/app.js` (only the view switch), `crates/api/ui/styles.css` (only new rules that the mockup needs), `crates/api/ui/people.js` and `crates/api/ui-tests/people.test.mjs` (only for a model function that the sheets need).

Do not change: `recipients.js`, `values.js`, `editor.js`, `shell.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The page of a person renders the sections of `pageSequence('person')` in their sequence (ADR 0099 item 4):
    - `profile`: the picture, the name, the usual role, "Member of" with a link to each publisher feed, and each Nostr key with the result of its check.
    - `actions`: "Use This Address", "Rename" and "Add to All Releases" (action), then "Remove" (destructive). No button is disabled because no feed is selected (ADR 0099 item 7).
    - `where`: one navigation row "Where used", with "Pays in N feeds · Credited in N feeds" and the number of feeds. It opens the view `where-used`.
    - `payment`: the select menu "Default address" with each address of each wallet, one group for each wallet with its address lines, each `lud16` suggestion of `GET /people/{id}/profile-addresses` with "Add to Wallet", and "Add a Wallet".
    - `credits`: the fields "Name", "Usual role", "Picture address" and "Information address". A change sends `PATCH /people/{id}`. Also the credit names and the Nostr keys, each with "Remove", and "Add a Credit Name" and "Add a Nostr Key".
    - `support`: each support link with "Remove", and "Add a Support Link".
2. The page "Where used" lists `sheetFeeds` rows with no check box (ADR 0099 item 5). A row opens the feed in the editor.
3. The four sheets (ADR 0099 items 7 to 11) put their own fields first, then the feeds of `sheetFeeds` with check boxes, each selected. The primary button reads `reviewLabel(n)`. It sends `POST /people/{id}/changes`, then opens the review.
    - "Use This Address": the select menu "Address" with each address of each wallet of the person.
    - "Rename": the field "Name", and the check boxes "Payment name" and "Credit name", both selected.
    - "Add to All Releases": the line "Studio adds {name} to each release of the artist or label that you select.", the select menu "Artist or label", the check boxes "As a payee" with the field "Percentage", "As a credit" and "Support link" with a menu of the support links of the person. Its feeds are the feeds of `GET /feeds/{id}/level` of the selected artist or label, not the feeds of the person. After each change of a field, the sheet sends `POST /people/{id}/changes/preview` and shows `addResultLine`.
    - "Remove": the check boxes "Payee" and "Credit".
4. A sheet with no selected feed, or with no selected part, has its primary button disabled. This is not the case of item 1, which is about the buttons of the page.
5. The page reloads `GET /people/{id}` after each change of a link.

### Acceptance Criteria

Mechanical:

- The Node tests of 065a pass. A new model function has its own Node test.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The page of a person with two wallets, a `lud16` suggestion, credit names, a key and a support link.
- The page "Where used".
- The sheets "Use This Address", "Rename", "Add to All Releases" with its result line, and "Remove".

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 4 to 12)
- docs/tasks/studio-v1-task-065-people-screens.md (the Words section and packet 065c only)
- docs/tasks/studio-v1-task-064-person-operations.md (packet 064b, the routes)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 065c names.

Goal:
- The page of a person, the page "Where used" and the four sheets of the operations work, with a preview for Add to All Releases.

Constraints:
- Follow the Architecture section of packet 065c fully.
- No feed rule in the UI (ADR 0065). The core gives each outcome and each split.
- Logic that a test examines is in `people.js`, not in a component (ADR 0042).
- Use the components and the tokens that exist. A new CSS rule uses `--text-*` tokens and no free `font-size`.
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `recipients.js`, `values.js`, `editor.js`, `shell.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 065c. The planning model does the visual check.

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

## Packet 065d: The Page Of A Wallet And The Words

### Files

Inspect: `crates/api/ui/app.js` (`RecipientPage`, `OperationForm`, `OperationSheet`, `OPERATION_TITLES`), `crates/api/ui/recipients.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/confirms.js`, `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui-tests/words.test.mjs` (the scan of the UI text).

Change: `crates/api/ui/app.js`, `crates/api/ui/recipients.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/confirms.js`, `crates/api/ui/styles.css` (only to remove rules that no element uses), `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui-tests/people.test.mjs` (new test of item 5).

Do not change: `values.js` and `editor.js` (task 066 changes the pickers), `shell.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The page of a wallet with no person (the view `wallet`) has no feed list with check boxes. Its operations are Change Address, Rename, Use the Lightning Address and Add to All Releases (action), then Remove (destructive). Each sheet lists the feeds of the wallet with check boxes below its fields, each selected, and its primary button reads `reviewLabel(n)` (ADR 0099 items 7 and 12). The feeds come from `POST /recipients/uses`.
2. In the sheet "Add to All Releases" of a wallet, the recipient details (type, name, address, custom key and account) are in one collapsed entry (ADR 0099 item 10).
3. The words (ADR 0099 items 13 to 18):
    - Each UI text says "changes" for a change set, for example "Changes to write", "Discard These Changes?" and "Discard the Changes".
    - The type menu has the items "Lightning Node" and "Lightning Address". The field of a custom value is "Account". The field of a custom key keeps "Custom key", with the hint "Most wallets do not need it. Copy it from your wallet when it gives one."
    - An address line reads "Lightning address {address}", "{provider} node · account {value}", or "Lightning node {key}".
    - The hint of ADR 0095 item 15 reads "Studio found no payment details for this address at {domain}. Some apps cannot pay it."
    - No UI text shows an XML attribute such as `method="lnaddress"`.
    - The operation "Add to an artist or label" is "Add to All Releases".
4. Remove `RecipientPage`, `OperationForm` and `OperationSheet` from `app.js` when their parts move to `peoplepages.js`, and each function of `recipients.js` that no entry point reaches after this packet. Code that no entry point reaches is deleted.
5. A new Node test reads `people.js`, `peoplepages.js` and `recipients.js`. It refuses each UI text with `method=` or the words "change set" (ADR 0099 item 17 and the acceptance criteria of ADR 0099). Its message names ADR 0099 and the fix.

### Acceptance Criteria

Mechanical:

- A Node test: the type menu gives "Lightning Node" and "Lightning Address", and the address lines and the hint give the texts of item 3.
- A Node test: no UI text of the People area has `method=` or "change set".
- A Node test: the model of each sheet of a wallet gives the feed check boxes, each selected, and the button text "Review in N Feeds".
- The tests of `recipients.test.mjs` change only for the new words and the removed functions.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The page of a wallet with no person, and each of its five sheets.
- The review of a change set of a wallet.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 7, 10 and 12 to 18)
- docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md (items 7, 14 and 15)
- docs/tasks/studio-v1-task-065-people-screens.md (the Words section and packet 065d only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 065d names.

Goal:
- The page of a wallet with no person uses sheets that list the feeds, and each text of the People area uses the words of ADR 0099.

Constraints:
- Follow the Architecture section of packet 065d fully.
- No feed rule in the UI (ADR 0065).
- Logic that a test examines is in a model module, not in a component (ADR 0042).
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `values.js`, `editor.js`, `shell.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 065d. The planning model does the visual check.

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
