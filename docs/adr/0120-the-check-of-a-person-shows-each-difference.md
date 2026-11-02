# ADR 0120: The Check Of A Person Shows Each Difference

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screen 8 of the mockups in `.studio-dev/mockup-start/site/`.
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) items 9 to 13, the rows of "Check your feeds".
- Uses: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) items 14 to 17 for the operation "Correct". [ADR 0098](0098-the-people-module.md) item 10 for the credit index. [ADR 0028](0028-summary-lines-and-one-open-entry.md) for the open entry.

## Context

- On 2026-11-02 a row of "Check your feeds" showed the feed, its type and one line for each finding, for example "The information address in the credit differs from DuhLaurien". A select of the row opened the editor page of the feed.
- The operator found the row confusing. The row did not show the old value, the value of the person, or the place of the credit. Two findings gave the same line, because they came from two credits with a different role.
- The credit index `feed_credits` keeps one row for each different credit of a feed, with its role, its group, its picture and its information address. It does not keep the place of a credit: the channel, or the title of an item.

## Decision

1. A row of the check is a `details` entry (ADR 0028). Its summary line gives the title, the type and the count, for example "Album · 2 credits differ". One entry is open at a time.
2. The open entry shows one block for each finding. A block gives:
    - The place: "Channel", or the title of each item with the credit, for example "Track 3 · Neon Diner". More than three places give the first two and "and N more".
    - The kind of the value and the role of the credit, for example "Credit · Vocals · information address".
    - Two lines: "In the {release word}" with the value of the feed, and the name of the person with the value of the person. A missing value is "None".
3. The entry ends with "Use {name}'s Values" (primary) and "Open the {release word}". The first button opens the sheet "Correct" of ADR 0111 item 14 for that one feed. The second button opens the editor page of the feed.
4. The page title is "Check your releases". The release word is "album", "playlist", "artist page" or "label page".
5. The credit index keeps the places of each credit: the channel, and the GUID and the title of each item. A migration fills the places from the last version of each feed in the collection.
6. The core gives each finding with its values and its places. The UI calculates no difference.

## Alternatives Considered

- A select of the row opens a sheet with the differences. Not chosen by the operator.
- The values with no places, and no change of the index. Not chosen: the user cannot find the credit in a long album.

## Consequences

- `feed_credits` gets the places, with a migration. The check reads them, and the other users of the index do not change.
- The findings of the API get values and places. The check page shows the entries of item 1.
- A task file gives the work.

## Non-goals

- A change of the operation "Correct".
- A check of fields that ADR 0111 does not compare.

## Acceptance Criteria

Mechanical:

- A core test: the index keeps the channel and each item of a credit, and the migration fills the places of a stored feed.
- A core test: a finding gives the value of the feed, the value of the person, the role and the places.
- A Node test: the summary line, the blocks of item 2 with more than three places, and the texts of items 3 and 4.
- The four commit checks pass.

Visual: screen 8 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator opens "Check your releases" of DuhLaurien, opens the album entry, and selects "Use DuhLaurien's Values".
