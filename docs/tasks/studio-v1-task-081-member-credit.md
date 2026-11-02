# Studio v1 Task 081: The Credit Of A Member And The Members Of A Feed

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0118](../adr/0118-member-of-offers-a-credit-on-the-artist-feed.md). It uses [ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md), [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0111](../adr/0111-the-page-of-a-person-checks-the-feeds.md) item 10 and [ADR 0112](../adr/0112-a-credit-role-comes-from-a-list.md).

Status: Not started. [Task 079](studio-v1-task-079-person-save-and-discard.md) is committed. The sheets open after the Save of packet 079b. The task can start without task 080. It starts after task 080, because the two tasks change `people.rs` and `peoplepages.js`. The planning model examined the packets again on 2026-11-02, and changed items 2 and 3 of packet 081b.

## Goal

After a Save that adds a membership, the page of a person offers a channel credit on the artist feed through a change set. After a Save that removes one, it offers to remove the credit. The page of a publisher feed lists its members.

## Packets

The planning model examines each packet before its commit. 081a changes the core and the API. 081b changes the UI.

Visual check of 081b, by the planning model, before its commit: screens 7 and 8 of `.studio-dev/mockup-lookup/site/`. Also take the sheet "Remove {name} from the Credits of {feed}?". The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0118.

## Packet 081a: The Credit Operation And The Members In The Core

### Files

Inspect: `crates/core/src/change_set.rs` (`PersonOperation`, `person_operation`, `add_snapshot`, `add_to_releases`, `RecipientOperation::AddToReleases`, `RecipientOperation::RemovePerson`, `make_person_change_set`), `crates/core/src/people.rs` (`Person`, `person_credits`, `person_normal_names`, `CreditRow`, `add_member`), `crates/core/src/collection.rs` (`feed_credits`, the table `feed_credits`, the table `person_members`), `crates/core/src/roles.rs`, `crates/api/src/people.rs` (`make_changes`, `person_snapshot`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs` (the route list of the token test).

Change: `crates/core/src/change_set.rs`, `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `PersonOperation` gets `CreditFeed { role: String, group: String }`, serialized as `credit_feed`. `person_operation` maps it to `RecipientOperation::AddToReleases` with no payee and no support link. The credit has the values of the person, as `add_snapshot` makes it, with `role` and `group` from the request. A role of the role list gets its group from `roles.rs`, as ADR 0112 item 4 says. Do not copy the credit code of `add_to_releases`.
2. `CreditFeed` needs a feed list of one publisher feed. A different list fails with `CollectionError::invalid`, before the core makes a draft.
3. The removal uses `PersonOperation::Remove { payee: false, credit: true }` with the one feed. It needs no new operation.
4. `people::feed_members(collection, feed_id) -> Result<Vec<FeedMember>, CollectionError>` gives the members of ADR 0118 item 9. `FeedMember` has `person_id`, `name`, `img`, `usual_role`, `member: bool` (a row in `person_members`) and `in_credits: bool` (a credit of the person in the feed, with the identity rule of `person_credits`). The list holds each person with `member` or `in_credits`, by name. Use `person_credits`. Do not copy its rule.
5. `feed_credits` has no column for the place of a credit. The 7 publisher feeds of the corpus have one item, and it has no credit. Thus `feed_members` counts each indexed credit of a publisher feed as a channel credit. Write this fact in a comment.
6. `GET /api/v1/feeds/{id}/members` gives `{ members }`. A feed that is not a publisher feed gives an empty list. The credit operation uses the route `POST /people/{id}/changes` that exists.

### Acceptance Criteria

Mechanical:

- A core test: `CreditFeed` adds one `podcast:person` to the channel of a publisher feed with the role and its group. A read and a write with no other edit keep the other bytes.
- A core test: `CreditFeed` with two feeds, or with an album, fails and makes no draft.
- A core test: `Remove { payee: false, credit: true }` with the feed removes the credit.
- A core test: `feed_members` gives a person with a membership only, a person with a credit only, and a person with both, with `member` and `in_credits`.
- An API test: `GET /feeds/{id}/members` gives the members, and the route is in the list of the token test.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0118-member-of-offers-a-credit-on-the-artist-feed.md
- docs/tasks/studio-v1-task-081-member-credit.md (the Goal and packet 081a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 081a names.

Goal:
- The core adds or removes the channel credit of a person in one publisher feed through a change set.
- The core gives the members of a publisher feed, and the API serves them.

Constraints:
- Follow the Architecture section of packet 081a fully.
- A write to a host occurs only through `pipeline::write_draft`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 081a.

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

## Packet 081b: The Credit Sheets And The Members Section

### Files

Inspect: `crates/api/ui/peoplepages.js` (`PersonPage`, the Save of task 079, `OperationForm` and its `go({ name: 'change-set', id })`, `RoleMenu`, `PersonArt`, `ListRow`), `crates/api/ui/people.js` (`saveBody`, `MEMBER_HINT`, `memberChoices`, `profileLine`), `crates/api/ui/shell.js` (`isEditable`, `isReadOnly`, `feedKind`), `crates/api/ui/editor.js` (`PublisherPage`, its sections and `showSections`), `crates/api/ui/pages.js` (the publisher page), `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/pages.test.mjs`, `.studio-dev/mockup-lookup/site/lookup.js` (`memberOffer`, `members`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/editor.js`, `crates/api/ui/pages.js`, `crates/api/ui/styles.css` only if the visual check needs it, `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/pages.test.mjs`, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `MEMBER_HINT` becomes "The artist or label that this person is in. After Save, Studio can add this person to the credits of that feed." `NEW_MEMBER_HINT` of the sheet "Make a Person" does not change, because that sheet has no Save.
2. `people.js` gets `creditOffers(before, after, feeds, members, personId)`. `before` is the `member` of `savedValues` before the Save, and `after` is the `member` of the values that the Save sent (task 079). `feeds` is the feed list of the page. `members` maps each feed ID of `before` and `after` to its `GET /feeds/{id}/members` list. It gives a list of zero, one or two offers, with `remove` first:
    - `{ kind: 'remove', feedId }`: `before` is a feed, it differs from `after`, and `members` of that feed has a row of the person with `in_credits`.
    - `{ kind: 'add', feedId }`: `after` is a feed, it differs from `before`, and `isEditable` of that feed is `true`. Also, `members` of that feed has no row of the person with `in_credits`.
3. After a successful Save that changed the membership, the page reads `GET /feeds/{id}/members` for each feed of item 2. It opens the sheet of each offer of `creditOffers`, one after the other. The next sheet opens when the user closes the sheet before it with "Not Now". "Review in 1 Feed" opens the change set, and the page shows no other sheet.
4. The add sheet has these parts:
    - The title "Add {name} to the Credits of {feed}?" and the note of ADR 0118 item 4.
    - `RoleMenu`, with the usual role and group as start values.
    - The section "The credit", with one row, and the public-data note.
    - The buttons "Not Now" (normal) and "Review in 1 Feed" (primary).
    - The row and the note name the Nostr key only when the person has one.
5. "Review in 1 Feed" sends `POST /people/{id}/changes` with `{ operation: { kind: 'credit_feed', role, group }, feeds: [feedId] }`, and opens the change set as `OperationForm` does.
6. The remove sheet has the title "Remove {name} from the Credits of {feed}?", the text "The artist feed then does not name {name}. Studio shows the review first.", and the buttons "Not Now" (normal) and "Review in 1 Feed" (primary). It sends the operation `{ kind: 'remove', payee: false, credit: true }` with the one feed.
7. "Not Now" closes the sheet and changes nothing.
8. `pages.js` gives the publisher page `{ id: 'members', tier: 'details' }` before `{ id: 'details', tier: 'details' }`.
9. `PublisherPage` gets the section `members`. It reads `GET /feeds/{id}/members` when the page opens. With no member, the section shows nothing. Each member is a `ListRow` with `PersonArt`, the name, and the line "{usual role} · in the credits of this feed" or "{usual role} · only in Studio". A member with no usual role shows the state only. A row opens `{ name: 'person', id }`.
10. The heading is "Members". The note is "The people that are members of this {word}. A member that is not in the credits is only in Studio." The word is the first role of `feedKind(feed)` in lowercase, or "feed" for a feed with no role.
11. `docs/user-guide/people.md` tells how "Member of" offers the credit, and that the review comes before the write.

### Acceptance Criteria

Mechanical:

- A Node test: `creditOffers` gives `add`, `remove`, and the two offers with `remove` first.
- A Node test: `creditOffers` gives an empty list for a read-only feed, for a feed with the credit, and for no change.
- A Node test: the texts of `MEMBER_HINT`, the titles of the two sheets and the note of item 10.
- A Node test: `pages.js` puts `members` before `details` on the publisher page, and the tier test passes.
- The test of title case still passes.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0118-member-of-offers-a-credit-on-the-artist-feed.md
- docs/tasks/studio-v1-task-081-member-credit.md (the Goal and packet 081b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 081b names.

Goal:
- After a Save that changes "Member of", the page of a person offers a change set.
- The change set adds or removes the credit on the artist feed. The page of a publisher feed lists its members.

Constraints:
- Follow the Architecture section of packet 081b fully.
- No feed rule in the UI (ADR 0065). The core makes the credit.
- Each button label is in title case. Each other text is in sentence case.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 081b.

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
