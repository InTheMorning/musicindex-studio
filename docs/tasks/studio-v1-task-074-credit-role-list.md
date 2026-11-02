# Studio v1 Task 074: The Credit Role List

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0112](../adr/0112-a-credit-role-comes-from-a-list.md). It uses [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md) and [ADR 0098](../adr/0098-the-people-module.md) items 5 and 20.

Status: Planned.

## Goal

The role of a credit comes from a menu: the Music group of MSP 2.0, then the groups of the Podcast Taxonomy Project, then "Other role…". The core writes the group of a role of the list. The usual role of a person uses the same menu.

## Packets

The planning model examines each packet before its commit. 074a changes the core and the API. 074b changes the UI.

Visual check of 074b, by the planning model, before its commit: screens 8a and 8b of `.studio-dev/mockup-people-2/site/`. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, on `studio.musicindex.org` after the deploy: set the role "Vocalist" on a credit and write the feed. The feed on the file host has `role="vocalist" group="music"`.

## Words

- The **list** is the role list of the core, from 074a item 1.
- A role **of the list** has the text of a role of the list. The comparison ignores case and the space at each end.

## Packet 074a: The List, The Group Rule And The Usual Group

### Files

Inspect: `crates/core/src/feed/elements.rs` (`Person`), `crates/core/src/pipeline.rs` (`save_draft` and its steps, such as `apply_duration_seconds` and `keep_key_form`), `crates/core/src/people.rs` (`Person`, the people table code, `update_person`), `crates/core/src/change_set.rs` (`NewCredit`, `PersonOperation::AddToReleases`), `crates/core/src/collection.rs` (the migrations, `APP_KEYS_MIGRATION`), `crates/api/src/people.rs` (`PATCH /people/{id}`), `crates/api/src/lib.rs` (the routes), `../musicindex-research/sources/podcast-namespace/taxonomy.json` (commit `f593c5da3f6ef680cd80b92d96078795744dac0b`, CC0 1.0).

Change: add `crates/core/src/roles.rs` and its tests. Change `crates/core/src/lib.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/people.rs`, `crates/core/src/change_set.rs`, `crates/core/src/collection.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the API tests, the AGENTS.md layout row of `crates/core`.

Do not change: the UI, the ADRs, `crates/server`.

### Architecture

These choices are fixed. Do not change them.

1. `roles.rs` has the list as a constant: groups in sequence, each with its roles in sequence. The first group is "Music" with Band, Vocalist, Guitarist, Bassist, Drummer, Keyboardist and Musician. Then come the 11 groups of `taxonomy.json` in its sequence, with each of its roles in its sequence. A comment names the source file, its commit and its license. Copy only the group and role names, not the descriptions.
2. `roles.rs` gives `groups() -> &[RoleGroup]`, `groups_of(role: &str) -> Vec<&str>` (with no regard to case and to the space at each end), and `wire(text: &str) -> String` (the text in lowercase, trimmed). "Editor" is in "Writing" and in "Video Post-Production".
3. `GET /api/v1/roles` gives `{ groups: [{ name, roles: [name] }] }`, with the display names. It needs no open vault.
4. A new step of `save_draft`, `apply_role_group(current, sent)`, acts on each credit of the channel and of each item, before `write_hosted`. It acts only on a new credit, or on a credit with a `role` or `group` that changed. The step compares the credit with the credit of the same entry key in `current`. For such a credit with a role of the list:
    - When the sent group is one of `groups_of(role)`, with no regard to case, the step keeps that group.
    - Else the group becomes the first group of `groups_of(role)`.
    - The role and the group become `wire` of their text.
   A credit with a role that is not of the list keeps its sent role and group. A credit that does not change keeps its bytes (ADR 0012).
5. A migration (the next number after `APP_KEYS_MIGRATION`) adds `usual_group TEXT NOT NULL DEFAULT ''` to `people`. `Person` gets `usual_group`. `PATCH /people/{id}` accepts it. The export gives it.
6. "Pick from your people" and `AddToReleases` write the usual role and the usual group. When the usual role is of the list and the usual group is empty, the credit gets the first group of the role. `NewCredit` gets `group`.

### Acceptance Criteria

Mechanical:

- A core test: the list has each role of `taxonomy.json` at the named commit and the Music group. "Editor" is in two groups. The test fails with a message that names ADR 0112 and the fix.
- A core test: a new credit with the role "Vocalist" saves as `role="vocalist" group="music"`. A changed credit with the role "Editor" and the group "Video Post-Production" keeps that group.
- A core test: a credit that the user does not change keeps its bytes, also with the role "Vocalist" and no group.
- A core test: a credit with the role "Field Recordist" keeps its sent role and group.
- A core test: the migration gives each person an empty `usual_group`. `AddToReleases` writes the usual group.
- An API test: `GET /roles` gives the groups, the Music group first.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0112-a-credit-role-comes-from-a-list.md
- docs/tasks/studio-v1-task-074-credit-role-list.md (the Goal, the Words section and packet 074a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 074a names.

Goal:
- The core keeps the role list and gives it through the API. It writes the group of a role of the list at the draft save. Each person gets a usual group.

Constraints:
- Follow the Architecture section of packet 074a fully.
- A credit that the user does not change keeps its bytes.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 074a.

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

## Packet 074b: The Role Menu

### Files

Inspect: `crates/api/ui/editor.js` (the credits entry, `BLANK_PERSON`, the sheet "Pick From Your People"), `crates/api/ui/peoplepages.js` (`PersonFields`), `crates/api/ui/people.js` (`personFieldsBody`, `creditFromPerson`), `crates/api/ui/app.js` (`ChoiceSelect`, `Field`), `.studio-dev/mockup-people-2/site/people.js` (`roleSelect`, `creditEntry`).

Change: add `crates/api/ui/roles.js` and `crates/api/ui-tests/roles.test.mjs`. Change `crates/api/ui/editor.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/app.js` if a shared component needs a group of options, `crates/api/ui/styles.css` only if the visual check needs it, `docs/user-guide/` (the page that explains the credits).

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The UI reads `GET /roles` one time for each page load, and keeps the result in memory.
2. `roles.js` has no Preact and no browser state. It gives:
    - `roleChoice(role, group, list)`: the value of the menu. It is `group|role` of the list entry when the role is of the list. The group is the sent group when it is one of the groups of the role, else the first group. It is `other` for a role that is not of the list, and empty text for no role.
    - `fromChoice(value, list)`: `{ role, group }` with the display names, or `null` for `other`.
    - `ROLE_PLACEHOLDER` "Select a role", `OTHER_ROLE` "Other role…", `OTHER_HINT` "Use a role from the menu when one fits. Apps know those roles." and `NO_ROLE_HINT` "Apps show a credit with no role as Host."
3. The role field of a credit is a `select` with one `optgroup` for each group, then the option "Other role…". With `other`, the entry shows the text field "Other role" with `OTHER_HINT`, and the select menu "Group" with each group of the list.
4. The UI sends the role and the group with their display names. The core writes the form of the wire (074a item 4). This is not a feed rule in the UI: the UI gives only the selected values (ADR 0065).
5. The field "Usual role" of the page of a person uses the same menu, with the same "Other role…" fields. It sends `usual_role` and `usual_group` with `personFieldsBody`.
6. The summary line of a credit entry shows the role and the group of the menu.

### Acceptance Criteria

Mechanical:

- A Node test: `roleChoice` gives the entry of the list with no regard to case. It gives the first group for a group that is not a group of the role. It gives `other` for "Field Recordist", and empty text for no role.
- A Node test: `fromChoice` gives the display names, and `null` for `other`.
- The Node test of ADR 0078 and the page rule tests pass with no change.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0112-a-credit-role-comes-from-a-list.md
- docs/tasks/studio-v1-task-074-credit-role-list.md (the Goal, the Words section and packet 074b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 074b names.

Goal:
- The role of a credit and the usual role of a person are a menu of the role list, then "Other role…".

Constraints:
- Follow the Architecture section of packet 074b fully.
- No feed rule in the UI (ADR 0065).
- Each select menu and each text field follows the size rules of AGENTS.md.
- Each button label is in title case. Each other text is in sentence case.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 074b.

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
