# ADR 0132: A Person Can Be Part Of Many Artists And Labels

- Status: Accepted
- Date: 2026-11-05
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-05 by the operator, through the questionnaire, from the mockups in `.studio-dev/mockup-part-of/`. The operator selected the list on the page, the words "Part of", one sheet for each feed and a menu with no hosting feed.
- Technical choices: items 3, 8, 9 and 12 are choices of the planning model. The mockups do not show them. The operator accepted them on 2026-11-05.
- Changes: [ADR 0118](0118-member-of-offers-a-credit-on-the-artist-feed.md) items 1, 2, 3 and 8 to 10, and its non-goal of one membership. [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 3.
- Changed by: [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md). Items 5 and 6: "Link to an Artist or Label Page" opens one sheet with the search of Music Index, and a page that is not in the collection is a read-only link.
- Uses: [ADR 0037](0037-button-roles.md) for the button roles. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the core. [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) for the first role. [ADR 0098](0098-the-people-module.md) item 6 for the link. [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for Save.

## Context

On 2026-11-05 the operator asked what "Member of" does, and if its name and its hint are correct. The code gives these facts:

- The link stays in Studio. Only the credit sheet after Save can change a feed, and the user must accept that sheet and review the write.
- The hint does not say that the link stays in Studio. "Studio can add this person" can read as an operation with no user.
- "Member of" and "is in" suit a band. They do not suit a label or a solo artist.
- The menu keeps one feed, although the core keeps a list. A person cannot be in two bands, or in a band and on a label.
- The menu lists each publisher feed, also a hosting feed, below the heading "Artist or label".

## Decision

### The List

1. A person has zero or more links to publisher feeds. The table `person_members` does not change.
2. The section `member` of the page of a person has the heading "Artists and labels". Its hint is "Add the details of this person to your artist or label page. After Save, you can add a credit to each new feed. The link itself stays in Studio."
3. The section has one group with one row for each link, A to Z. A section with no link has the row "This person is not part of an artist or label yet."
4. Each row shows the type dot, the feed title, the first role and the condition of the link: "In the credits of this feed", "Only in Studio" or "Not saved yet". A row has the normal button "Remove".
5. A second group has the select menu "Add an artist or label", with the placeholder "Select…". A selection adds a row and returns the menu to the placeholder.
6. The menu holds each publisher feed, A to Z, except a feed that the list has and a feed with the first role `hosting`. A feed with no role stays in the menu, and it shows only its title.
7. A link to a feed with the first role `hosting` that exists from before keeps its row, and the user can remove it.
8. "Remove" immediately takes the row out of the page. Discard puts it back. The toolbar counts each added and each removed row as one change (ADR 0116).
9. Save sends the complete list. The core replaces the links in one transaction. When an ID is not a publisher feed, the core refuses the Save and changes nothing.

### The Words

10. The interface uses "Part of" for this link:
    - The list of people shows "Part of {feed}, {feed}" in the place of "Member of {feed}".
    - The head of a person shows each feed with its dot and its link, as before.
    - The section "Members" of a publisher feed has the heading "People". Its note is "The people that are part of this {role word}. A person that is not in the credits is only in Studio."
    - The note of the credit sheet is "{name} is part of {feed} in Studio. The credits of the feed can also show it, so that apps and musicindex.org can find {name}."
    - The menu of a new person has the label "Part of" and the hint "The artist or label that this person is part of. Studio changes no feed." It keeps one feed.

### The Credit Sheets

11. A Save that adds or removes links opens one sheet for each feed with an offer. The conditions of ADR 0118 items 3 and 6 stay. The sheets of a removal come first. With two or more sheets, each sheet starts with its position, for example "1 of 2".
12. "Not Now" and "Review in 1 Feed" each open the next sheet. "Review in 1 Feed" makes its change set first, and a failure stays in that sheet. After the last sheet, one change set opens its review page, and two or more open the list "Changes to write". With no change set, the page of the person stays.

## Alternatives Considered

- One row that opens a sheet with the links. Not chosen by the operator: the condition of each link is then one step away.
- The words "Member of". Not chosen by the operator: they stay incorrect for a label and a solo artist.
- One credit sheet for all feeds. Not chosen by the operator: one role then goes to each feed.
- A menu with only artists and labels. Not chosen by the operator: a feed with no role then has no link until it gets a role.
- A menu with each publisher feed. Not chosen by the operator: a hosting feed stays below an artist or label heading.

## Consequences

- The API body of a Save changes from one member to a list, and the page values change with it.
- `person_members`, the suggestions of ADR 0113 and the export use a list, so they do not change.
- The code keeps the names `member` and `person_members`. Only the interface uses "Part of".
- [Task 093](../tasks/studio-v1-task-093-part-of.md) gives the work.

## Non-goals

- A role for each link. A person keeps one usual role, and a credit sheet can change the role of its credit.
- More than one feed in the menu of a new person.
- The menu "Artist or label" of the operation "Add to an artist or label". It lists each publisher feed.

## Invariants

- A link writes no feed. A feed changes only through a change set that the user reviews (ADR 0007, ADR 0118).
- The core gives the first role, the condition of a link and the refusal of a feed that is not a publisher feed (ADR 0065).

## Acceptance Criteria

Mechanical:

- A core test: a Save keeps two links, replaces the list, and an empty list removes each link. A list with a feed that is not a publisher feed changes nothing.
- An API test: the Save route takes the list and gives the links back.
- A Node test: the menu leaves out a linked feed and a feed with the first role `hosting`, and keeps a feed with no role.
- A Node test: the model gives the rows and the texts of items 2 to 4 and 10, and the UI text has no "Member of".
- A Node test: the credit offers give one sheet for each feed, the removals first, with the position of each sheet.
- A Node test: the model selects the review page for one change set and the list for two or more.
- The four commit checks pass.

Visual: the page of a person with no link and with two links, the People list, the People section of an artist feed and the second credit sheet, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator links one person to an artist and to a label, selects Save, gives a different role in each credit sheet, and writes the two feeds. Each feed page then shows the person with "In the credits of this feed".
