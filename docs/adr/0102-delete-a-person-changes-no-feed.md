# ADR 0102: Delete A Person Changes No Feed

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Changes: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) items 4 and 11, the name of the operation "Remove".
- Uses: [ADR 0098](0098-the-people-module.md) items 2, 3 and 15, and [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tier of a section.

## Context

- In the human test of `0.1.0-rc.9` on 2026-11-01, the operator could not delete a person from People. The page of a person has only "Remove", the operation of ADR 0098 item 20 that removes the person from feeds.
- The route `DELETE /people/{id}` deletes a person and its links, and changes no feed (ADR 0098 item 2). No button uses it.
- "Remove" and a delete of the person are two different actions with similar names. The user can select the wrong one.

## Decision

1. The page of a person gets the button "Delete Person" (destructive) in the last section, with the tier `limits`.
2. The button opens a confirmation sheet with the title "Delete {name} from People?" and the text "Studio deletes this person and its links. Studio changes no feed." The action button reads "Delete the Person". The text is in `confirms.js`.
3. After the delete, Studio opens the People page. The items of the person can come back as a suggestion, because they are no longer a link of a person (ADR 0098 item 16).
4. The operation "Remove" of ADR 0099 items 4 and 11 reads "Remove From Feeds", on its button and as the title of its sheet. Its behavior does not change.

## Alternatives Considered

- Keep "Remove" and add "Delete Person". Not chosen: two destructive buttons with similar names on one page.
- Put the delete in the menu of a row of the People page. Not chosen: Studio has no row menus, and the page of a person holds each action of a person.

## Consequences

- The page of a person has two destructive buttons with different names: "Remove From Feeds" at the top and "Delete Person" at the end.
- The Node tests of the sheet titles change for item 4.

## Acceptance Criteria

Mechanical:

- A Node test: the page sequence of a person ends with a section of the tier `limits` that holds "Delete Person".
- A Node test: the confirmation of item 2 is in `confirms.js` with its title, its text and its action.
- A Node test: the operation title of item 4 is "Remove From Feeds".

Visual: the page of a person with "Delete Person" at its end, and the confirmation sheet.

Human, on `studio.musicindex.org`: the operator deletes a person, and the feeds of that person have no draft.
