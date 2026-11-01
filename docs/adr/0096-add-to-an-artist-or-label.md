# ADR 0096: Add To An Artist Or Label

- Status: Accepted
- Date: 2026-10-31
- Owner: operator
- Accepted: 2026-10-31 by the operator.
- Changed by: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md). Item 18: the operation is "Add to All Releases".
- Changes: [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 4 and 9, and [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) items 10 and 14, for the name of the operation "Add to a level".
- Uses: musicindex ADR 0023 R-CASE-1 (title case on a button) and musicindex ADR 0033 R-WORD-3 (the role words).

## Context

- ADR 0091 item 9 gives the operation "Add to a level". It adds a recipient to each value block of a publisher feed and of each feed below it.
- "Level" is a word of the left panel of [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md). The UI does not show it as a name. On 2026-10-31 the operator asked what "Add to a level" means.
- The sheet has the placeholder "Select an artist or label", and the hint "Studio found no artist or label feed in your collection". Its menu has the label "Level".
- The menu lists each publisher feed. A hosting feed is also in it.
- The operator selected this name from a questionnaire on 2026-10-31. The other name was "Add to Feeds".

## Decision

1. The operation has the name "Add to an artist or label". The button shows it in title case: "Add to an Artist or Label". The sheet title, the title of a change set and each other text show it in sentence case.
2. The menu of the sheet has the label "Artist or label".
3. The operation, its feeds and its splits do not change. The menu continues to show each publisher feed, also a hosting feed.
4. The code name `add`, the route `feeds/{id}/level` and the API fields do not change.

## Alternatives Considered

### Add to Feeds

The name is shorter. It does not tell the user what to select first.

### Keep "Add to a level"

The user does not know the meaning of the word "level" on this sheet.

## Consequences

- The name tells the user to select an artist or a label.
- A hosting feed is in the menu, but the name does not say "hosting". The menu shows the title of each feed.

## Acceptance Criteria

Mechanical:

- A Node test: `operationTitle({ kind: 'add' })` gives "Add to an artist or label".
- The test of musicindex ADR 0023 R-CASE-1 accepts the button label "Add to an Artist or Label".

Visual: the recipient page and the sheet "Add to an artist or label", at 390 px with touch.
