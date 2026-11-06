# ADR 0146: Link Words, The Mark Of A Value That Is Already There And Edits Of A Person

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, from screenshots of the local build. The operator gave the words "link to Artist or Label page", and asked that a row that is already there keeps its content with a mark.
- Technical choices: item 5 is a choice of the planning model.
- Changes: the words of [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md), "Part of" and "Add an artist or label". [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) item 10, the look of a row that the list has already.
- Source: [findings of local use, findings 33 to 35](../reviews/2026-11-06-local-use-findings.md)

## Context

The section "Artists and labels" did not tell what it does. A row that a list had already lost its line, and only a text told its state. A Nostr result that came from the feeds of the user could also have no picture, when a person holds that key and has one.

## Decision

### The Words

1. A control that adds this link has the label "Link to an artist or label page". The section has the heading "Linked artist and label pages". This applies to an album, a playlist and a person.
2. A line of a person that names its links starts with "Linked to".

### A Value That Is Already There

3. A row that is already there shows the same content as each other row. It is gray, and it has a tag with a check mark and its state, for example "Already in this list". It cannot be selected.
4. This applies to the rows of each sheet of ADR 0145, to the fill of "Find on musicindex" of a person, and to the check of a suggestion.
5. A result of "Find a Nostr profile" that is in the feeds of the user is also gray, with the tag "In your feeds". It can be selected, because a selection is the only way to use a key that no person holds. When a person holds the key, the row takes a missing name or picture from that person.

### Edit

6. A credit name and a support link of a person have the button "Edit", which opens the sheet that adds them, with the saved values. A save replaces the values and changes no feed. A Nostr key that the person holds with the old value of the credit name goes with that value.
7. A Nostr key with no credit name has no "Edit", because it has one value. The user removes it and adds a different key.

## Alternatives Considered

- A Nostr result in the feeds that cannot be selected. Not chosen by the planning model: the user could then not use that key.

## Consequences

- A gray row that can be selected is a different meaning from a gray row that cannot. The tag and the pointer show the difference.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines the words, the merge of a result with a person and the start of an edit sheet. `crates/core/src/people.rs` examines each change.

Visual: the planning model looks at each sheet at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
