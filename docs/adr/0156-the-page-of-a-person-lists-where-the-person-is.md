# ADR 0156: The Page Of A Person Lists Where The Person Is

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator, from the mockups of the questionnaire. The operator asked to see which tracks pay and credit a person, with the split of each track.
- Uses: [ADR 0143](0143-a-feed-row-names-the-tracks-of-a-person.md) for the places, [ADR 0028](0028-summary-lines-and-one-open-entry.md) for the entries, [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tier.
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 12, the lines of the row "Check your feeds". [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md), the lines of a credit row of "More from your feeds".

## Context

The row "Check your feeds" gave counts, for example "Pays in 1 feed · Credited in 0 feeds". A credit row of "More from your feeds" gave its roles as its title, then the feeds in one line. The two rows did not tell which tracks pay or credit the person, and they gave no split. A credit that is not linked to the person did not count as "Credited", and the page did not say this.

## Decision

1. The page of a person has the section "Where this person is", with the tier identity, before the row "Check your feeds". It has one entry for each feed that pays or credits the person.
2. The summary line of an entry tells in words where the feed pays and credits the person, for example "Paid on the album and 12 tracks" or "Credited as vocalist on 2 tracks".
3. An open entry has one line for each place. The line names "Album" for the channel, or the number and the title of the track. It gives the split of the person in that place, for example "15 % split", and the roles of the credits of the person in that place. One entry is open at a time (ADR 0028).
4. The core gives the split of each place in percent. A fee recipient keeps its share as a percent. Each other recipient gets its part of the rest, as in the share bar of ADR 0133. When many addresses of the person are in one place, the split is their sum.
5. The core keeps the number of each track and the split of each place in the index of ADR 0143. A run that applies the change refills the index from the library copies, as ADR 0143 item 4 does.
6. The row "Check your feeds" gives only the state of the check and the count of feeds. It opens the check page as before.
7. A credit row of "More from your feeds" has the release as its title. A row with many releases has the title "N releases", and each of its lines starts with the release. Each role has one line with its places, for example "Vocalist · track 3 "Night Song", track 5 "Low Tide"". The text of the places is the text of ADR 0120 item 2: the first two places and "and N more". The name in the credit and the cause follow.
8. A credit row with a Nostr key keeps the profile of the key as its title (ADR 0103 item 17). Its lines follow item 7.

## How This Decision Is Checked

Mechanical:

- `studio-core`: the number and the split of each place, a fee, and the sum of two addresses in one place. The places of each role of a "More from your feeds" credit.
- `crates/api/ui-tests/people.test.mjs`: the summary line and the lines of an entry. The lines of "Check your feeds" and of a credit row.
- `crates/api/ui-tests/pages.test.mjs`: the sequence of the sections of the page of a person.

Visual, by the agent: the page of a person with an open entry, at 1280 px and at 390 px with touch. The agent examines the dark and the light theme.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).
