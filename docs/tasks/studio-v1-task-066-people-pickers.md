# Studio v1 Task 066: The People Pickers

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decisions: [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) items 14 and 19 to 21, [ADR 0098](../adr/0098-the-people-module.md) item 3, [ADR 0027](../adr/0027-the-recipient-picker.md), [ADR 0030](../adr/0030-an-added-recipient-gets-no-split.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Not started.

## Goal

The editor gives the people of the user as start values. The recipient picker of the value editor shows the people first, then the payees with no person. The credits editor and the support links editor get "Pick From Your People". A value from a person is a start value, and the feed keeps its own copy (ADR 0098 item 3).

## Packets

The planning model examines each packet before its commit. The packets go in this sequence, because both add functions to `people.js`:

1. 066a: the recipient picker and the words of the value editor.
2. 066b: the pickers of the credits and of the support links.

Visual check, by the planning model, before each commit. It uses the method of task 065. It takes screenshots in the two themes at 1100 px, and at 390 px with touch. It runs `$edges()` at 1100 px, and `$fieldText()` and `$overflow()` at 390 px with touch.

Human test, by the operator, on `studio.musicindex.org` after the deploy. The operator adds a person from each of the three pickers. Then the draft has the values of the person.

## Words

- A **picker** is a sheet that lists rows with check boxes and adds the selected rows to the editor. It only adds. It does not remove or replace an entry (ADR 0027 invariant).
- A **start value** is a value that the editor puts in a new entry. The user can change it, and the core applies the feed rules at the draft save (ADR 0065).
- The list of the people comes from `GET /people`. Each person has `default_address`, `wallets`, `keys`, `credit_names`, `support_links`, `usual_role`, `img` and `href`.

## Packet 066a: The Recipient Picker And The Words Of The Value Editor

### Files

Inspect: `crates/api/ui/values.js` (`TYPES`, `Recipients`, `SaveRecipientButton`, `PickerSheet`, `PickRow`, `identityOf`, the "Your recipients" section), `crates/api/ui/people.js` (`memberText`, `personRow`, `addressText`), `crates/api/ui/recipients.js` (`pickerRows`), `crates/api/ui-tests/picker.test.mjs`, `.studio-dev/mockup-098/site/people.js` (the screen `vpicker`).

Change: `crates/api/ui/values.js`, `crates/api/ui/people.js`, `crates/api/ui/recipients.js` (only `pickerRows` and its callers), `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/picker.test.mjs` and other Node tests only for the new words.

Do not change: `editor.js`, `peoplepages.js`, `app.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The section "Your recipients" of the picker becomes two sections (ADR 0099 item 19):
    - "People": one row for each person with a default address. The row shows the picture, the name and the line "Adds {line}", with `memberText` of the default address. A person with no wallet is not in the section.
    - "Payees with no person": one row for each address of `GET /recipients` with `saved_id` `null`, with its name and its address line.
    - The filter field filters the two sections by name and address. It is a display filter.
2. A selected person adds one recipient: the name of the person, the type, the address, the custom key and the custom value of the default address. A selected payee adds its own values. A new recipient gets no split (ADR 0030), as the picker does now. A row whose identity is in the list already shows "Already in this list" and has no check box to select.
3. The section "From musicindex" stays as it is.
4. `people.js` gets `recipientPickRows(people, recipients)`, which gives `{ people, payees }` with the rows of item 1, and `recipientFromPerson(person)`, which gives the new recipient of item 2. The component uses them. A Node test examines them.
5. The button "Save to Your Recipients" of an open recipient reads "Add to People". It sends `POST /recipients/saved` as now, and that route makes a person (task 063b). After the add it shows "Added" for 2 seconds. The "Remove" button of the picker rows goes, because a picker does not remove a person.
6. The words of ADR 0099 item 14 in the value editor: the type menu gives "Lightning Node" and "Lightning Address". The field of a custom value is "Account". The field of a custom key keeps "Custom key", with the hint "Most wallets do not need it. Copy it from your wallet when it gives one."
7. The footer button reads "Add 1 Recipient" or "Add N Recipients". The hint below the sections reads "A new recipient gets no split. You give the splits in the editor."

### Acceptance Criteria

Mechanical:

- A Node test: `recipientPickRows` gives a person row with its default address, and leaves out a person with no wallet. A payee row comes only from an address with no person.
- A Node test: `recipientFromPerson` gives the name and the address values of the default address, and no split.
- A Node test: the type menu of the value editor gives "Lightning Node" and "Lightning Address".
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The recipient picker with people and payees, and an open recipient with "Add to People".

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 14 and 19)
- docs/tasks/studio-v1-task-066-people-pickers.md (the Words section and packet 066a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 066a names.

Goal:
- The recipient picker shows the people first, then the payees with no person, and the value editor uses the words of ADR 0099.

Constraints:
- Follow the Architecture section of packet 066a fully.
- No feed rule in the UI (ADR 0065). A value from a person is a start value.
- Logic that a test examines is in `people.js`, not in a component (ADR 0042).
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `editor.js`, `peoplepages.js`, `app.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 066a. The planning model does the visual check.

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

## Packet 066b: The Pickers Of The Credits And The Support Links

### Files

Inspect: `crates/api/ui/editor.js` (`Persons`, `Fundings`, `CreditsPage`, `BLANK_PERSON`, `addOpen`, `useSheet`, the track credits near `withTrackCredits`), `crates/api/ui/values.js` (`PickerSheet` and `PickRow` as a pattern), `crates/api/ui/people.js`, `crates/api/ui/credits.js`, `.studio-dev/mockup-098/site/people.js` (the screen `cpicker`).

Change: `crates/api/ui/editor.js`, `crates/api/ui/people.js`, `crates/api/ui/styles.css` (only new rules that the sheet needs), `crates/api/ui-tests/people.test.mjs`.

Do not change: `values.js`, `peoplepages.js`, `app.js`, the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `Persons` gets the row "Pick From Your People" below "Add a person" (ADR 0099 item 20). It opens the sheet "Pick From Your People", with a filter field and one row for each person: the picture, the name and the line of the usual role and the Nostr key, joined with " · ". The hint reads "Each person comes with the usual role, the picture and the Nostr key. You can change them in the credit." The footer button reads "Add 1 Person" or "Add N People".
2. A selected person adds one credit with the name, the usual role, `img`, `href` and the first Nostr key of the person. An empty value gives no attribute. The credit gets no group. A person that the list has already, with the same Nostr key or the same name, shows "Already in this list".
3. `Fundings` gets the row "Pick From Your People" below "Add a support link" (ADR 0099 item 21). It opens a sheet with one row for each support link of each person: the label, and the line with the name of the person and the address. A selected row adds one support link with its label and its address. A row whose address the list has already shows "Already in this list". The footer button reads "Add 1 Support Link" or "Add N Support Links".
4. The pickers work in each place that uses `Persons` or `Fundings`: the album or publisher page, and a track with its own credits.
5. `people.js` gets `creditPickRows(people, persons)`, `creditFromPerson(person)`, `supportPickRows(people, fundings)` and `fundingFromLink(link)`. The components use them. A Node test examines them.
6. A new entry gets no `_cid` and no `key` from the picker. The editor gives it a new client ID, as `addOpen` does (ADR 0022).

### Acceptance Criteria

Mechanical:

- A Node test: `creditPickRows` gives each person with the line of its role and its key, and marks a person that the list has already. `creditFromPerson` gives the name, the usual role, the picture, the information address and the first key.
- A Node test: `supportPickRows` gives each support link of each person, and marks an address that the list has already. `fundingFromLink` gives the label and the address.
- The four commit checks pass.

Visual, by the planning model, before the commit:

- The credits picker and the support links picker, on the album page and on a track with its own credits.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0099-the-people-destination-and-the-page-of-a-person.md (items 20 and 21)
- docs/tasks/studio-v1-task-066-people-pickers.md (the Words section and packet 066b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 066b names.

Goal:
- The credits editor and the support links editor give the people of the user as start values.

Constraints:
- Follow the Architecture section of packet 066b fully.
- No feed rule in the UI (ADR 0065). A value from a person is a start value.
- Logic that a test examines is in `people.js`, not in a component (ADR 0042).
- Each UI text is in ASD-STE100 Simplified Technical English. A button label uses title case.

Do not touch:
- `values.js`, `peoplepages.js`, `app.js`, the core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 066b. The planning model does the visual check.

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
