# Studio v1 Task 093: A Person Can Be Part Of Many Artists And Labels

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0132](../adr/0132-a-person-can-be-part-of-many-artists-and-labels.md). It changes [ADR 0118](../adr/0118-member-of-offers-a-credit-on-the-artist-feed.md) and [ADR 0111](../adr/0111-the-page-of-a-person-checks-the-feeds.md) item 3. It uses [ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md).

Status: Complete, with packets 093a and 093b. The screenshot visual check passed on 2026-11-05. The human test is open.

## Goal

The page of a person holds a list of links to artists and labels, in the words "Part of". A Save sends the list. One credit sheet opens for each feed with an offer.

## Packets

The planning model examines each packet before its commit. 093a changes the core and the API. 093b changes the UI and its documents.

Visual check of 093b, by the planning model, before its commit: the page of a person with no link and with two links, the People list, the People section of an artist feed and the second credit sheet. The mockups are in `.studio-dev/mockup-part-of/`. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` must be empty.

Human test, by the operator, after the deploy: the human test of ADR 0132.

## Packet 093a: The Save Takes A List

### Files

Inspect: `crates/core/src/people.rs` (`PersonSave`, `save_person`, `insert_member`), `crates/api/src/tests.rs` (the save test).

Change: `crates/core/src/people.rs`, `crates/api/src/people.rs`, `crates/api/src/tests.rs`.

Do not change: the UI, the ADRs, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `PersonSave::member` becomes `members: Option<Vec<FeedId>>`. `None` changes no link. A list replaces each link, in the transaction of `save_person`. An empty list removes each link.
2. `insert_member` refuses a feed that is not a publisher feed, and the transaction then changes nothing. A hosting feed is a publisher feed, so the core accepts a link that exists from before. The menu of packet 093b leaves out a new one.
3. `Person::members`, `person_members`, `add_member`, `remove_member`, `feed_members` and the request field `member_of` of a new person do not change.
4. `people::linked_credits` gives the links with a credit of the person, with the test of `feed_members`, and `GET /people/{id}` gives them as `linked_credits`. The field `credited` does not fit, because it does not hold a credit that only the name matches. The two pages can then disagree.

### Acceptance Criteria

Mechanical:

- A core test: a Save keeps two links, replaces the list, and an empty list removes each link.
- A core test: a list with an album changes nothing, also for a valid feed earlier in the list.
- A core test: `linked_credits` gives a link with a credit that only the name matches, and no other link.
- An API test: the Save route takes `members`, and `GET /people/{id}` gives `members` and `linked_credits`.
- The four commit checks pass.

## Packet 093b: The List, The Words And The Sheets

### Files

Inspect: `crates/api/ui/people.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/editor.js` (the section `members`), `crates/api/ui-tests/people.test.mjs`, `docs/user-guide/people.md`.

Change: these files, `AGENTS.md`, this task, and the status lines of ADR 0111, ADR 0118 and ADR 0132.

Do not change: `memberChoices` for the operation "Add to an artist or label", the core.

Constraint: No feed rule in the UI (ADR 0065). The UI gets the first role from `publisher_roles` and the condition of a link from `linked_credits`.

### Architecture

These choices are fixed. Do not change them.

1. `savedValues` gives `members` as a list of text IDs. `pendingKeys` gives one entry `member` for each added and each removed ID. `saveBody` gives `members` as numbers.
2. A new function `linkChoices(feeds, linked)` gives the menu of ADR 0132 item 6. The menu "Part of" of a new person also uses it, with an empty list.
3. A new function `linkRows(values, saved, feeds, linkedCredits)` gives the rows of ADR 0132 items 3 and 4, A to Z.
4. `creditOffers(before, after, feeds, members, personId)` takes two lists. Each removal comes first, in the sequence of the list.
5. `creditPosition(index, total)` gives "1 of 2." with two or more sheets, else an empty text.
6. `afterCredits(made)` gives the view after the last sheet: `null` with no change set, the review page with one, and `{ name: 'people' }` with two or more. The People page holds "Changes to write".
7. A sheet that makes a change set gives its ID to the page, and then closes. The close opens the next sheet. A failure keeps the sheet open.

### Acceptance Criteria

Mechanical:

- A Node test: the menu leaves out a linked feed and a feed with the first role `hosting`, and keeps a feed with no role.
- A Node test: the model gives the rows and the texts of ADR 0132 items 2 to 4 and 10.
- A Node test: `crates/api/ui/` has no text "Member of".
- A Node test: each added and each removed link is one pending change, and `saveBody` sends the complete list.
- A Node test: the credit offers give one sheet for each feed, the removals first, with the position of each sheet.
- A Node test: `afterCredits` selects the review page for one change set and the list for two or more.
- The four commit checks pass.

Visual: the list in "Visual check of 093b" above.
