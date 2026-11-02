# Studio v1 Task 082: The Credit Name And The Title Of A Member Credit

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0118](../adr/0118-member-of-offers-a-credit-on-the-artist-feed.md) items 11 and 12. They come from the visual check of [task 081](studio-v1-task-081-member-credit.md).

Status: Complete. Packets 082a and 082b are committed. The screenshot visual check passed on 2026-11-02. The human test is open.

## Goal

The credit of a member links its credit name to the person, so that the remove sheet of ADR 0118 item 6 removes that credit. The change set of a member credit keeps its own operation, and the review page gives it the title "Add to the Credits".

## Packets

The planning model examines each packet before its commit. 082a changes the core. 082b changes two texts of the UI, and the planning model writes it.

Visual check of 082b, by the planning model, before its commit: the cycle of the task 081 check. Set "Member of" of a person with no credit name, select Save, "Review in 1 Feed" and write. Then set "None", select Save, "Review in 1 Feed" and write. The first review has the title "Add to the Credits", and the second write changes the feed. The check uses 1100 px, and 390 px with touch, in the dark theme.

Human test, by the operator, after the deploy: the human test of ADR 0118, then the remove of that credit.

## Packet 082a: The Credit Name And The Operation In The Core

### Files

Inspect: `crates/core/src/change_set.rs` (`RecipientOperation`, `apply`, `add_to_releases`, each `match` on `RecipientOperation::AddToReleases`, `PersonOperation::CreditFeed`, `person_operation`, `make_person_change_set`, `check_credit_feed`, `credit_of`), `crates/core/src/people.rs` (`add_credit_name`, `insert_credit_name`, `Identity`), `crates/core/tests/change_set.rs` (the tests of task 081a).

Change: `crates/core/src/change_set.rs`, `crates/core/tests/change_set.rs`, and `crates/core/src/people.rs` only if a function must become `pub(crate)`.

Do not change: the UI, the API, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `RecipientOperation` gets `CreditFeed { credit: NewCredit, credits: CreditMatch }`, serialized as `credit_feed`. `person_operation` maps `PersonOperation::CreditFeed` to it, with the credit of `credit_of`.
2. `apply` gives `CreditFeed` the result of `add_to_releases` for an `AddToReleases` with no payee, the same credit, no support link and the same `credits`. Do not copy the code of `add_to_releases`. Each other `match` on `RecipientOperation` treats `CreditFeed` as that `AddToReleases`.
3. A stored change set of `add_to_releases` keeps its form and its behavior.
4. `make_person_change_set` with `PersonOperation::CreditFeed` runs `check_credit_feed` first. Then it links the credit name with `people::add_credit_name(collection, id, name, href, npub)`, with the values of the new credit. Then it builds the change set. A `Conflict` of the credit name stops it, with a `PipelineError` and no draft.
5. The person that `person_operation` reads after the link has the new credit name. Thus the `credits` of the operation hold the identity of the new credit.

### Acceptance Criteria

Mechanical:

- A core test: a `credit_feed` change set of a person with no credit name links the credit name. After the write of its draft, a `Remove { payee: false, credit: true }` change set of the same feed changes the channel and removes the credit.
- A core test: a credit name of a different person with the same identity stops the change set, and the core makes no draft.
- A core test: the stored operation of a `credit_feed` change set has the kind `credit_feed`.
- The tests of task 081a still pass. A read and a write with no other edit keep the other bytes.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0118-member-of-offers-a-credit-on-the-artist-feed.md (items 5, 6, 11 and 12)
- docs/tasks/studio-v1-task-082-credit-name-and-title.md (the Goal and packet 082a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 082a names.

Goal:
- The credit of a member links its credit name to the person, and keeps its own operation `credit_feed`.

Constraints:
- Follow the Architecture section of packet 082a fully.
- A write to a host occurs only through `pipeline::write_draft`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Clippy runs with `-D warnings`, which includes `too_many_lines` for a test. Put a long setup in a helper function.

Do not touch:
- The UI, the API, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 082a.

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

## Packet 082b: The Title And The Line Of A Member Credit

### Files

Change: `crates/api/ui/people.js` (`changeTitle`, the `switch` of the change line near the case `add_to_releases`), `crates/api/ui-tests/people.test.mjs`.

### Architecture

1. `changeTitle` gives "Add to the Credits" for the kind `credit_feed`.
2. The change line of `credit_feed` is the line of `add_to_releases` with only a credit: "Add {name} as a credit in 1 feed".

### Acceptance Criteria

Mechanical:

- A Node test: `changeTitle({ kind: 'credit_feed' })` gives "Add to the Credits", and the change line names the credit.
- The test of title case still passes.
- The four commit checks pass.

Visual: the visual check of this task.
