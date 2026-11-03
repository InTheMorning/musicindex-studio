# ADR 0123: One Type Of Value Has One Picker

- Status: Accepted
- Date: 2026-11-03
- Owner: operator
- Accepted: 2026-11-03 by the operator: "reuse our rich pickers instead of having different input code for the same data".
- Changes: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 4, the forms "Add a Nostr Key" and "Add a Credit Name" of the page of a person.
- Uses: [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) for the profile search.

## Context

- On 2026-11-03 the operator could not find a Nostr key by name on the page of a person. The editor has the sheet "Find a Nostr profile" (ADR 0074), with a search by name and the profile of each result. The page of a person had a plain text field for the same key, in "Add a Nostr Key" and in "Add a Credit Name".
- The text of "Find on musicindex" (ADR 0117 item 9) tells the user to use "Add a Nostr Key" to search the Nostr profiles. That sheet had no search.
- Two inputs for one type of value give two behaviors. A fix or a feature in one does not go to the other.

## Decision

### The Rule

1. When Studio has a picker for a type of value, each location that edits that type of value uses that picker. A new location does not have its own input for that value. This is a durable rule.
2. A picker is the component of the UI that edits a type of value with more than a text field: a search, a lookup or a list of choices. The current pickers are:
    - A Nostr key: `NostrRow` and the sheet "Find a Nostr profile" (`NostrSheet`) of `crates/api/ui/editor.js`.
    - A credit role: `RoleMenu` of `crates/api/ui/app.js`, from the role list of the core (ADR 0112).
    - A person of the user: "Pick From Your People" (ADR 0099 item 20).
    - A date and a time: `DateTimeField` (ADR 0093). A duration: `DurationField` (ADR 0094).
3. A picker can need a context that a location does not have, such as the release folder of a feed for "Choose…" (ADR 0062). It is then not a picker for that location. The location keeps its field, and the record of the location says why.

### The Page Of A Person

4. "Add a Nostr Key" opens the sheet "Find a Nostr profile". A selection adds the key to the person, with the route of the form from before. The sheet does not show the choices that put the profile name and the profile picture in a feed.
5. In "Add a Credit Name", the field "Nostr key" is a `NostrRow`. Its sheet is "Find a Nostr profile", and the selected key goes in the form.

## Alternatives Considered

- Add a search to the form of the page of a person. Not chosen: it is a second input for one type of value, which this decision stops.
- A rule for the Nostr key only. Not chosen: the operator asked for the reuse of each picker.

## Consequences

- `editor.js` exports `NostrRow` and `NostrSheet`, and `peoplepages.js` uses them.
- The link form of a person has no plain field for a Nostr key.
- A Node test refuses a plain field for a Nostr key in the link forms of a person.

## Invariants

- The UI has no copy of a feed rule (ADR 0065). A picker gives a value, and the core keeps each rule.

## Non-goals

- A picker for a wallet address. The editor and the page of a person each have a text field for it.
- A picker for the picture of a person. "Choose…" needs the release folder of a feed (item 3).

## Acceptance Criteria

Mechanical:

- A Node test: `LINK_FORMS` has no field with the id `npub`.
- The four commit checks pass.

Visual: on the page of a person, "Add a Nostr Key" searches a name and adds the selected key. "Add a Credit Name" sets its key through the same sheet. The check uses 1100 px, and 390 px with touch, in the two themes.

Human: the operator finds the key of a person by name on a phone, and adds it.
