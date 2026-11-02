# ADR 0113: The Page Of A Person Suggests A Picture And An Information Address

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screen 9 of the mockups in `.studio-dev/mockup-people-2/site/`.
- Uses: [ADR 0098](0098-the-people-module.md) items 5, 6, 10 and 19. [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 10 for the credits of a person. [ADR 0109](0109-images-and-audio-load-from-their-hosts.md) for the preview. [ADR 0028](0028-summary-lines-and-one-open-entry.md) for the collapsed line. [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the requests.

## Context

- The fields "Picture address" and "Information address" of a person are empty text fields. The operator must type or paste each value.
- Studio has the values already in three places: the credits of the person in the feeds, the Nostr profile of each key of the person, and the artist or label feed of "Member of".
- The operator accepted screen 9 of the mockups on 2026-11-02. The operator selected the three sources, an open list below an empty field, and a collapsed line below a filled field.

## Decision

1. The core gives the suggestions for the picture and for the information address of a person. The route is `GET /people/{id}/detail-suggestions`. The answer has `img` and `href`, each a list of `{ value, sources }`.
2. The sources are:
    - `profile`: the `picture` and the `website` of the newest kind 0 of each key of the person, from the lookup relays of ADR 0074 item 12. The source names the key.
    - `credit`: the `img` and the `href` of each credit of the person in the index of the credits. A credit of the person follows ADR 0111 item 10. The source names the feeds.
    - `member`: the image and the `link` of each publisher feed in "Member of". The source names the feed.
3. A value is an `http` or `https` address. Equal values join one suggestion with each of their sources. A value that is equal to the current value of the person is not a suggestion.
4. The sequence is `profile`, then `credit` by the number of feeds, most first, then `member`.
5. The requests to the relays follow the network policy of the shell (ADR 0057) and the time limit of the profile lookups. They run with no lock of the workspace. A relay that does not answer gives no `profile` source, and the other sources stay.
6. Below an empty field, the page shows the list open, with the heading "Suggestions for the picture" or "Suggestions for the information address". Below a filled field, the page shows one collapsed line, for example "2 other values in your feeds and profiles", that opens the same list.
7. Each row shows the address, a line that names its sources, and "Use" (action). A picture row shows a preview from its `https` address (ADR 0109).
8. "Use" puts the value in the field and saves it, as a field save does. The field then shows "Saved". No feed changes. "Correct N Feeds" of ADR 0111 copies the value to the credits.

## Alternatives Considered

- Suggestions only below an empty field. Not chosen by the operator: a different value in the feeds then stays hidden until "Check your feeds".
- An open list below each field at all times. Not chosen by the operator: the page gets long for a person in many feeds.
- A free text field with the values as a `datalist`. Not chosen: it shows no source and no preview.

## Consequences

- The core reads the `website` of a profile. It reads only the `picture` now.
- The core gets the route of item 1. The UI gets the lists and the collapsed line.
- The suggestions are a display aid. The value goes to the person, not to a feed (ADR 0065).
- A task file gives the work.

## Non-goals

- A dismissal of a suggestion. A suggestion that the user does not want stays in the list. The list is closed when the field has a value.
- Suggestions for the name or for the usual role.

## Acceptance Criteria

Mechanical:

- A core test: the suggestions join equal values and leave out the current value. They follow the sequence of item 4, and refuse a value that is not an `http` or `https` address.
- A core test: a profile with `picture` and `website` gives `profile` sources, and a relay that does not answer gives the other sources.
- An API test: `GET /people/{id}/detail-suggestions` gives `img` and `href`.
- A Node test: the list is open for an empty field and collapsed for a filled field, with the text of item 6.
- The four commit checks pass.

Visual: screen 9 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator opens HeyCitizen, selects "Use" on the picture of the Nostr profile, and the field shows "Saved".
