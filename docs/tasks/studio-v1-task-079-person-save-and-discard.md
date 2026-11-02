# Studio v1 Task 079: Save And Discard On The Page Of A Person

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md). It changes [ADR 0111](../adr/0111-the-page-of-a-person-checks-the-feeds.md) item 2 and [ADR 0113](../adr/0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md) item 8.

Status: Complete. Packets 079a and 079b are committed. The screenshot visual check passed on 2026-11-02. The human test is open.

## Goal

The page of a person keeps the changes of its fields and menus as pending changes. The toolbar shows the status, "Discard" and "Save". Save applies all the changes in one transaction. The page asks before the user leaves it with a pending change.

## Packets

The planning model examines each packet before its commit. 079a changes the core and the API. 079b changes the UI.

Visual check of 079b, by the planning model, before its commit: screens 1, 5 and 6 of `.studio-dev/mockup-lookup/site/`. Screen 5 shows two pending fields with no lookup source line, because task 080 adds that line. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0116.

## Packet 079a: One Save Of A Person

### Files

Inspect: `crates/core/src/people.rs` (`PersonFields`, `change_person`, `clean_fields`, `set_default_address`, `add_member`, `remove_member`, the use of `unchecked_transaction` in `merge_people`), `crates/api/src/people.rs` (`change_person`, `set_default_address`, `add_link`, `remove_link`), `crates/api/src/lib.rs` (the people routes), `crates/api/src/tests.rs` (the route list of the token test).

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `people.rs` gets `PersonSave`, deserialized with `#[serde(default)]`. It has `fields: Option<PersonFields>`, `default_address: Option<Option<RecipientKey>>` and `member: Option<Option<FeedId>>`. `None` means "no change". `Some(None)` removes the default address or each membership.
2. `save_person(collection, id, &PersonSave) -> Result<(), CollectionError>` applies the parts in one `unchecked_transaction`. A failure of one part changes nothing.
3. To share the rules, move the body of `change_person`, `set_default_address`, `add_member` and `remove_member` into private functions that take a `&Connection`. The public functions keep their signatures and call the private functions. Do not copy a rule.
4. `member: Some(Some(feed))` keeps one membership: it adds `feed` and removes each other membership, as `memberChange` of `people.js` does now. `add_member` keeps its check that the feed is a publisher feed.
5. `POST /api/v1/people/{id}/save` takes a `PersonSave` and gives 204. The errors use `person_error`, as `change_person` does. The routes of each field stay.

### Acceptance Criteria

Mechanical:

- A core test: `save_person` applies the fields, the default address and the membership together.
- A core test: a membership of a feed that is not a publisher feed fails, and the fields of the same save do not change.
- A core test: `member: Some(None)` removes each membership, and `None` changes no membership.
- An API test: `POST /people/{id}/save` gives 204 and changes the person. The route is in the list of the token test.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0116-the-page-of-a-person-has-save-and-discard.md
- docs/tasks/studio-v1-task-079-person-save-and-discard.md (the Goal and packet 079a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 079a names.

Goal:
- The core applies a save of a person in one transaction, and the API serves it at `POST /people/{id}/save`.

Constraints:
- Follow the Architecture section of packet 079a fully.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 079a.

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

## Packet 079b: Pending Changes, The Toolbar And The Leave Sheet

### Files

Inspect: `crates/api/ui/peoplepages.js` (`PersonFields`, `PersonPage`, `saveFields`, `changeMember`, the default address menu, the `delete` section, `MergeSheet`, `OperationSheet`), `crates/api/ui/people.js` (`personFieldsBody`, `saveStatus`, `memberChange`, `FIELDS_HINT`, `MEMBER_HINT`, `detailList`), `crates/api/ui/editor.js` (`DraftToolbar`, the markup of `draft-toolbar`), `crates/api/ui/app.js` (`Bar`, `NavContext`, `navValue`, the `go`, `back` and `forward` of the shell, the `popstate` handler of the shell, `askConfirm`, `ConfirmHost`), `crates/api/ui/confirms.js`, `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`, `.studio-dev/mockup-lookup/site/lookup.js` (`personTools`, `person`, `leave`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/app.js`, `crates/api/ui/confirms.js`, `crates/api/ui/styles.css` only for the mark of a pending control, `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs, `crates/api/ui/editor.js`.

### Architecture

These choices are fixed. Do not change them.

1. `people.js` gets a model of the pending changes, with no Preact and no browser state:
    - `savedValues(person)` gives `{ name, usual_role, usual_group, img, href, default_address, member }`. `default_address` is the `keyValue` text or `''`. `member` is the first membership as text, or `''`.
    - `pendingKeys(saved, values)` gives the keys that differ, in the sequence of `savedValues`. `usual_role` and `usual_group` count as one change, with the key `usual_role`.
    - `pendingStatus(count)` gives "No changes", "1 change not saved" or "N changes not saved".
    - `saveBody(person, saved, values)` gives the `PersonSave` body of packet 079a. It sends `fields` when a field differs, `default_address` when it differs, and `member` when it differs. It gives `null` with no change.
    - `leaveText(name, keys)` gives the text of the leave sheet. It names each pending control with its label, for example "DuhLaurien has 2 changes that are not saved: the picture address and the information address."
2. `PersonPage` keeps `values` in its state, from `savedValues(person)`. Each field, the role menu, the default address menu and the "Member of" menu change `values` only. They send no request. `saveFields`, `changeMember` and the `PUT` of the default address on change are removed from these controls.
3. The `Bar` of the page has the markup of `DraftToolbar` after the title: `div.draft-toolbar`, then `span.draft-status` with `role="status"`, then "Discard" (`button`), then "Save" (`button primary`). Both buttons are disabled with no pending change. Do not import or change `DraftToolbar`.
4. Save sends `POST /people/{id}/save` with `saveBody`, and then reads the person again. A failure keeps `values`, and the status shows the error with the `field-error` class.
5. Discard sets `values` to `savedValues(person)`. It sends no request.
6. A control with a pending change gets the class `pending`. `styles.css` gives `.pending` a mark of 3 px at the start edge in the `--accent` token, as screen 5 shows. It uses no new color.
7. "Use" of a suggestion of ADR 0113 sets `values` only. Remove `saveStatus`, the status "Saved" below a field, and `FIELDS_HINT`. The suggestions compare with `values`, not with `person`.
8. `MEMBER_HINT` and `NEW_MEMBER_HINT` do not change in this packet. Task 081 changes them.
9. `confirms.js` gets `leavePerson(text)`: the title "Leave without saving?", the text from `leaveText`, the action "Discard Changes" with the role `destructive`, and `cancel: 'Keep Editing'`. `ConfirmHost` of `app.js` shows `entry.cancel` in the place of "Cancel" when the entry has it. Each other entry keeps "Cancel".
10. The shell gets one leave guard. `navValue` gets `setLeaveGuard(fn)`. The guard is an `async () => boolean`, and `true` means "leave". `go`, `back` and `forward` call the guard first. Each row of the left panel also calls it first. They stop when it gives `false`. A `popstate` with a guard first puts the history entry back with `history.pushState` of the current index, then calls the guard, and on `true` goes back again. The page of a person sets the guard while it has a pending change, and removes it when it has none or when it unmounts.
11. The guard of the page opens `leavePerson`. On "Discard Changes" it removes the pending changes and gives `true`.
12. The page adds a `beforeunload` listener while it has a pending change, and removes it after. The listener calls `event.preventDefault()`. It is not a call of `confirm(`.
13. With a pending change, the rows of "Change your feeds", "Check your feeds", "Merge with a Person", "Delete Person", "Add a Wallet", "Add a Credit Name", "Add a Nostr Key", "Add a Support Link", each "Remove" and each "Add" of "More from your feeds" first call the guard of item 11. They continue only on `true`.
14. `docs/user-guide/people.md` says that the page keeps the changes until Save, and that Save changes no feed.

### Acceptance Criteria

Mechanical:

- A Node test: `pendingKeys`, `pendingStatus` and `saveBody` for no change, one field, the role with its group, the default address and the membership.
- A Node test: `saveBody` gives `member: null` for the choice "None", and gives no `member` when the membership does not change.
- A Node test: `leaveText` names the pending controls.
- A Node test: `CONFIRMS.leavePerson` has the title, the action, the role `destructive` and `cancel: 'Keep Editing'`.
- The test of `confirm(` and the test of title case still pass.
- The four commit checks pass.

Visual: the visual check of this task. Also select Save with two changes, and Discard with two changes, and take a screenshot after each.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0116-the-page-of-a-person-has-save-and-discard.md
- docs/tasks/studio-v1-task-079-person-save-and-discard.md (the Goal and packet 079b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 079b names.

Goal:
- The page of a person keeps pending changes, with Save and Discard in its toolbar.
- The page asks before the user leaves it with a pending change.

Constraints:
- Follow the Architecture section of packet 079b fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- A confirmation is a sheet of Studio, never `confirm()`.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs, `crates/api/ui/editor.js`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 079b.

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
