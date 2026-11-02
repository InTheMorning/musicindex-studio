# Studio v1 Task 084: The Check Of A Person Shows Each Difference

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0120](../adr/0120-the-check-of-a-person-shows-each-difference.md).

Status: Not started.

## Goal

Each row of the check of a person opens in place. For each difference, it shows the place, the role, the value of the feed and the value of the person. A button corrects that one release.

## Packets

The planning model examines each packet before its commit. 084a changes the core and the API. 084b changes the UI.

Visual check of 084b, by the planning model, before its commit: screen 8 of `.studio-dev/mockup-start/site/`, with a person whose credits differ on two tracks. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0120.

## Packet 084a: The Places And The Values In The Core

### Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `PEOPLE_MIGRATION` and the refill of the credit index after it, `migrate`, `put_credits`, `record_state`, the table `feed_credits`), `crates/core/src/people.rs` (`person_credits`, `CreditRow`, `credit_findings`, `Finding`, `FeedCheck`, `check_person`), `crates/core/src/feed/` (the `Person` of a channel and of an item, the item `guid` and `title`), `crates/api/src/people.rs` (`check_person`), the core and API tests of the check.

Change: `crates/core/src/collection.rs`, `crates/core/src/people.rs`, the core and API tests. `crates/api/src/people.rs` only if the JSON shape needs it.

Do not change: the UI, the ADRs, `crates/server`, the other tables, the operation "Correct".

### Architecture

These choices are fixed. Do not change them.

1. A new migration adds the column `places TEXT NOT NULL DEFAULT '[]'` to `feed_credits`. The primary key does not change.
2. `places` holds a JSON list. Each value is `{"kind":"channel"}`, or `{"kind":"item","guid":…,"title":…}`. The list keeps the sequence of the feed and has each place one time.
3. `put_credits` gets the place of each credit. Two credits with the same key give one row, and the row gets the places of the two.
4. After the new migration, Studio fills the credit index again from the library copies, as it does after `PEOPLE_MIGRATION`. Use the same code. Do not copy it.
5. `CreditRow` gets `places`. `Finding::CreditKey` and `Finding::CreditDetails` get `places`. `CreditDetails` gets `feed_img`, `feed_href`, `person_img` and `person_href` as text. `CreditKey` gets `group`. The serde form of each existing field does not change.
6. The UI calculates no difference (ADR 0120 item 6). Each value of item 5 comes from the core.

### Acceptance Criteria

Mechanical:

- A core test: `put_credits` keeps the channel and two items of one credit in one row, in the sequence of the feed.
- A core test: an old collection at the schema version before the migration opens, and the index has the places of its feed after the refill.
- A core test: a `CreditDetails` finding gives the four values and the places.
- An API test: `GET /people/{id}/check` gives the places and the values.
- The tests of the check and of "Correct" still pass.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0120-the-check-of-a-person-shows-each-difference.md
- docs/tasks/studio-v1-task-084-check-differences.md (the Goal and packet 084a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 084a names.

Goal:
- The credit index keeps the places of each credit, and each finding of the check gives its places and its values.

Constraints:
- Follow the Architecture section of packet 084a fully.
- A read and a write with no edit give the same bytes. The corpus tests enforce this.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Clippy runs with `-D warnings`, which includes `too_many_lines` for a test. Put a long setup in a helper function.

Do not touch:
- The UI, the ADRs, `crates/server`, the other tables, the operation "Correct".
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 084a.

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

## Packet 084b: The Entries Of The Check

### Files

Inspect: `crates/api/ui/peoplepages.js` (`CheckPage`, `CorrectSheet`, `ListRow`), `crates/api/ui/people.js` (`findingLine`, `checkFeedLines`, `checkGroups`, `checkIntro`, `correctFeedIds`, `feedRows`), `crates/api/ui/shell.js` (`feedKind`), `crates/api/ui-tests/people.test.mjs`, `.studio-dev/mockup-start/site/start.js` (`checkNew`, `diffRow`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/styles.css` only for the block of a difference, `crates/api/ui-tests/people.test.mjs`, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs, the operation "Correct".

### Architecture

These choices are fixed. Do not change them.

1. `people.js` gets:
    - `releaseWord(feed)`: "album", "playlist", "artist page" or "label page", from `feedKind`.
    - `checkSummary(entry, feed)`: the type and the count, for example "Album · 2 credits differ".
    - `placeText(places)`: "Channel", or the item titles, and "and N more" after the first two of more than three.
    - `differenceBlocks(entry, name, feed, profiles)`: one block for each finding, with `place`, `what` (for example "Credit · Vocals · information address") and `lines` (`[{ label, value, old }]`). The value of the feed has `old: true`. A missing value is "None".
2. `CheckPage` shows each feed of a group as a `details` entry with the summary of `checkSummary`. One entry is open at a time (ADR 0028). The open entry shows the blocks, then "Use {name}'s Values" (primary) and "Open the {Release Word}".
3. "Use {name}'s Values" opens `CorrectSheet` with the feed IDs of that one feed. `CorrectSheet` gets an optional `feedIds`, and with no value it uses `correctFeedIds(check)` as before.
4. The page title and the heading are "Check your releases". The row of the page of a person keeps its title.
5. A feed with the state `correct` keeps the row of today, with no entry.

### Acceptance Criteria

Mechanical:

- A Node test: `releaseWord` and `checkSummary` for an album, a playlist and an artist page.
- A Node test: `placeText` for the channel, two items and five items.
- A Node test: `differenceBlocks` for a `CreditDetails` and a `CreditKey` finding, with "None" for a missing value.
- The test of title case and the test of the `details` elements still pass.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0120-the-check-of-a-person-shows-each-difference.md
- docs/tasks/studio-v1-task-084-check-differences.md (the Goal and packet 084b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 084b names.

Goal:
- Each row of the check opens in place and shows each difference, with a button that corrects that one release.

Constraints:
- Follow the Architecture section of packet 084b fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- Each `setState` that combines the old value uses a functional update.
- Each comment is in ASD-STE100 Simplified Technical English.
- The UI has no build step. To check syntax, copy a changed file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.

Do not touch:
- The core, the API, the ADRs, the operation "Correct".
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 084b.

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
