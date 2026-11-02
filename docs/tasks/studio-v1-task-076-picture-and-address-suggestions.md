# Studio v1 Task 076: Picture And Address Suggestions

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0113](../adr/0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md). It uses [ADR 0111](../adr/0111-the-page-of-a-person-checks-the-feeds.md) item 10 and [ADR 0098](../adr/0098-the-people-module.md) item 19.

Status: Complete, in two packets, with the screenshot visual check of 076b on 2026-11-02. The human test is open. Notes from the reviews:

- 076a: one helper finds the credits of a person for the check of ADR 0111 and for the suggestions. The image of an artist or label feed is the cover of the feed list.
- 076b: the visual check found the heading above a collapsed line. The line names the values already, so the planning model shows the heading only above an open list (ADR 0113 item 6).

## Goal

The fields "Picture address" and "Information address" of a person show suggestions from the Nostr profiles, the credits in the feeds and the artist or label feed. "Use" fills the field and saves it.

## Packets

The planning model examines each packet before its commit. 076a changes the core and the API. 076b changes the UI.

Visual check of 076b, by the planning model, before its commit: screen 9 of `.studio-dev/mockup-people-2/site/`. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0113.

## Packet 076a: The Suggestions In The Core

### Files

Inspect: `crates/core/src/profiles.rs` (`Profile`, the parse of kind 0 near `text("picture")`, `lightning_addresses_by_key`), `crates/core/src/people.rs` (`check`, `credit_findings`, `normal_name`, `Identity`, `profile_addresses` and its tests with `TestRelay`), `crates/core/src/collection.rs` (`feed_credits`, the feed list with the cover and the `link`), `crates/api/src/people.rs` (`list_profile_addresses`, the pattern with no workspace lock), `crates/api/src/lib.rs`, `crates/api/src/tests.rs` (the route list of the token test).

Change: `crates/core/src/profiles.rs`, `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `profiles.rs` reads `website` of a kind 0 as it reads `picture`: only an `http` or `https` address. Add a function. It gives the `picture` and the `website` of the newest kind 0 of each key in a list. It uses the relays, the policy and the time limit, as `lightning_addresses_by_key` does.
2. `people.rs` gets `detail_suggestions(collection, person, feeds, profiles) -> DetailSuggestions`. It makes no request: the caller gives the profile values. `DetailSuggestions` has `img` and `href`, each `Vec<DetailSuggestion>`. `DetailSuggestion` has `value` and `sources: Vec<DetailSource>`.
3. `DetailSource` is an enum serialized with `kind`: `profile { npub }`, `credit { feed_ids }` and `member { feed_id }`.
4. The credits of the person follow ADR 0111 item 10, with the same rule as `credit_findings`. Use one shared helper, not a copy.
5. Equal values, after a trim, join one suggestion. A value equal to the current value of the person, after a trim, is left out. Only `http` and `https` values count. The sequence follows ADR 0113 item 4. A `credit` source lists its feed IDs from low to high.
6. `GET /api/v1/people/{id}/detail-suggestions` reads the person, the keys and the feed list under the workspace. It reads the profiles with no workspace lock, as `list_profile_addresses` does. Then it gives the suggestions. A shell with no lookup relays gives no `profile` source.

### Acceptance Criteria

Mechanical:

- The core tests of the acceptance criteria of ADR 0113.
- An API test: the route gives `img` and `href`, and the route is in the list of the token test.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md
- docs/tasks/studio-v1-task-076-picture-and-address-suggestions.md (the Goal and packet 076a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 076a names.

Goal:
- The core gives the suggestions for the picture and the information address of a person, and the API serves them.

Constraints:
- Follow the Architecture section of packet 076a fully.
- No test sends a request to a host outside the computer. Use `TestRelay`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 076a.

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

## Packet 076b: The Suggestions On The Page

### Files

Inspect: `crates/api/ui/peoplepages.js` (`PersonFields`, `PersonPage`, `AddressRow`), `crates/api/ui/people.js` (`personFieldsBody`, `saveStatus`), `crates/api/ui/app.js` (`useImage`, `ImageThumb`), `.studio-dev/mockup-people-2/site/people.js` (`detailsPage`).

Change: `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/styles.css` only if the visual check needs it, `crates/api/ui-tests/people.test.mjs`, `docs/user-guide/people.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The page reads `GET /people/{id}/detail-suggestions` after it reads the person, and again after each field save.
2. A model function of `people.js` gives, for one field, the state of the list: `open` for an empty field with suggestions, `collapsed` for a filled field with suggestions, and `none` with no suggestion. It also gives the heading, the collapsed line and the source line of each row. A Node test examines it.
3. The headings are "Suggestions for the picture" and "Suggestions for the information address". The collapsed line is "1 other value in your feeds and profiles" or "N other values in your feeds and profiles". It is a `details` element with a summary line (ADR 0028).
4. A source line names each source: "From the Nostr profile {profile name or short key}", "In the credits of N feeds: {titles}" (one feed: "In the credit of 1 feed: {title}"), and "The image of the artist feed {title}" or "The link of the artist feed {title}". The word of the feed is its first role in lowercase. Two or more sources join with " · ".
5. Each row shows the address, the source line and "Use" (action). A picture row shows a preview with `ImageThumb` or the same image component of the page. "Use" saves the field with `personFieldsBody`, as a field save does, and the field shows "Saved".
6. The UI changes no value. It shows the values of the core.

### Acceptance Criteria

Mechanical:

- A Node test: the state of the list for an empty field, a filled field and no suggestion.
- A Node test: the source lines of each source kind, and the join of two sources.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md
- docs/tasks/studio-v1-task-076-picture-and-address-suggestions.md (the Goal and packet 076b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 076b names.

Goal:
- The picture field and the information address field of a person show the suggestions of the core, and "Use" fills and saves the field.

Constraints:
- Follow the Architecture section of packet 076b fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 076b.

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
