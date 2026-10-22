# ADR 0022: The Editor Sends The Model With Entry Keys

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

Phase 5 makes the album editor. The UI edits the typed model of a draft ([ADR 0012](0012-lossless-tree-with-a-typed-model.md)), and the core writes the change into the draft ([ADR 0015](0015-connector-interface-and-write-pipeline.md)).

Each list entry of the typed model keeps the source element that it came from. The writer uses that element to keep the source bytes of the entry and each element in it that the core does not model. JSON cannot transmit the source element to the UI and back. The core must thus find the source element of each entry again when the UI sends a change.

ADR 0008 does not let a GUID change without a user action. An editor that can add a track must give the new track a GUID.

ADR 0007 leaves the snapshot retention limit open. Phase 5 shows the snapshot list.

## Decision

### The Model And The Keys

1. The API gives the typed model of a draft as JSON. Each entry of each list has a `key`: its position in that list in the draft, from 0.
2. The UI sends the full changed model back. The core reads the draft again and finds, for each sent entry with a key, the entry with the same key in the same list. The core keeps the source element of that entry and writes the sent values into it.
3. A sent entry with no key is a new entry. A draft entry with a key that the UI did not send is removed. The sequence of a sent list is the new sequence.
4. The core refuses a change when a key is not in the draft, or when a key occurs two times in one list. The draft then does not change.
5. Each draft has a revision: a hash of the draft bytes. The UI sends the revision that it edited. The core refuses a change when the revision is not the current revision. The response gives the new model, with new keys, and the new revision.
6. A field that the user did not change goes back with the value that the UI got. When the user clears a text field, the UI sends `null`, and the writer removes the element.

### Identity

7. When a sent item has no key and no GUID, the core gives it a new random UUID (version 4) as its item GUID.
8. The core refuses a change of the feed `podcast:guid` when the draft has one. It also refuses a change of the GUID of an item with a key when that item has a GUID. The editor shows these GUIDs and does not let the user change them.

### Saving And Snapshots

9. The UI saves the draft a short time after each change. A save changes only the draft. The hosted file changes only through a write with an approved diff ([ADR 0007](0007-draft-diff-write-and-announce.md)).
10. Studio keeps each snapshot. The retention limit of ADR 0007 stays open. A subsequent ADR sets a limit if the database becomes too large.

## Alternatives Considered

- Change operations, such as set, insert, remove and move, with a path in the model. Rejected: more routes and more code in the core for the same result. A full model is one route, and the UI can send all its values.
- Keys from a stable ID that the core keeps for each entry. Rejected: the core must then keep a store for the IDs. A position in the draft is sufficient, because the revision check stops a change from a previous model.
- Edit existing tracks only. Rejected by the operator: an album editor must add, remove and move tracks.
- Keep the last 50 snapshots. Rejected at this time: a feed file is small, and a removal changes the ADR 0014 rule that a snapshot row does not change.
- A save button. Rejected by the operator: with an automatic save, the user keeps each edit, and a save does not change the hosted file.

## Consequences

- The model types of the core get JSON forms. An entry with a key carries its key, and a new entry carries none.
- A new track in an album needs an enclosure URL that the user types, because Studio does not host files ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
- The UI must wait for the response of a save before it sends the next save. Each response gives new keys and a new revision.
- The collection database grows with each write. No limit applies.

## Invariants

- An entry that the user did not change keeps its source bytes.
- The feed GUID and the GUID of an existing item do not change through the editor.
- A change from a previous revision of a draft does not change the draft.
