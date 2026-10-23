# ADR 0027: The Recipient Picker

- Status: Accepted
- Date: 2026-10-22
- Owner: operator
- Changed by: [ADR 0030](0030-an-added-recipient-gets-no-split.md). Item 7: an added recipient gets no split.

## Context

On 2026-10-22 the operator checked the value editor ([task 012](../tasks/studio-v1-task-012-value-editor.md)). In that editor, an artist types the same recipients into many feeds and tracks. The operator asked for a list of the recipients that the artist used. The operator also asked for an import of recipients from the feeds and tracks in musicindex.

The musicindex API gives `payment_routes` with each feed and each track. Each route has a name, a route type, an address, a split, `fee`, and a custom key and value. Musicindex identifies a recipient by its address, its custom key and its custom value. A keysend route to a shared node names the account in the custom pair.

## Decision

1. The editor offers a recipient picker for the album and for each track.
2. The picker lists recipients from two sources:
   - each recipient of each feed and track that Studio read into the collection, and
   - each recipient that the user saved.
3. A recipient is identified by its address, its custom key and its custom value. The list has each recipient one time. The name and the type come from the most recent read or save.
4. The collection keeps the recipients of each feed after each read and each write. A recipient of the collection goes out of the list when no feed of the collection has it.
5. The user can save a recipient, and remove a saved recipient. A saved recipient stays until the user removes it.
6. The picker can also show the `payment_routes` of a feed or a track in musicindex. The user finds the feed or the track with the musicindex search.
7. The picker shows recipients with checkboxes. Studio adds each selected recipient after the recipients that the editor has. It does not replace or remove a recipient. Each added recipient keeps the split of its source, and the user then changes the splits.

## Alternatives Considered

- Only the recipients of the collection. Rejected by the operator: a recipient that no feed uses, such as a new band member, then cannot be in the list.
- Only a list that the user keeps. Rejected by the operator: the user then saves each recipient by hand.
- Replace the recipients of the editor with the imported recipients. Rejected by the operator: an import usually adds a guest artist to the recipients that exist.

## Consequences

- The collection schema gets a table for the recipients of each feed and a table for the saved recipients. This is a new migration ([ADR 0014](0014-collection-in-sqlite.md)).
- A recipient address is public. These tables contain no secret ([ADR 0003](0003-user-unlocked-secret-vault.md)).
- The import from musicindex needs the musicindex API. The other parts of the picker work with no network.

## Invariants

- The picker does not remove or replace a recipient of the editor.
- The list has each recipient, by address and custom pair, one time.
