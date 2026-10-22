# Studio v1 Task 010: The Draft API

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0008](../adr/0008-feed-types-and-feed-identity.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md) and [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md).

## Goal

Give the draft, the diff, the write and the snapshots of a feed through the local API. The UI edits the typed model as JSON with entry keys.

## Steps

1. Give the model types of `studio-core` a JSON form. Each list entry carries its key. A new entry has no key (ADR 0022 items 1 and 3).
2. Add a core function that applies a sent model to a draft. It finds each entry by its key and keeps its source element (ADR 0022 items 2 to 4).
3. Add the draft revision, and refuse a change from a different revision (ADR 0022 item 5).
4. Give a new item a random UUID as its GUID. Refuse a change of the feed GUID and of the GUID of an existing item (ADR 0022 items 7 and 8).
5. Add API routes:
   - Give the draft model and revision. Start a draft from the live file when the feed has none.
   - Replace the draft model.
   - Remove the draft.
   - Give the diff and its ID.
   - Write the draft with the ID of the approved diff.
   - Show the snapshots, and make a draft from a snapshot.

## Constraints

- A write occurs only through `pipeline::write_draft`, with the approved diff ID (ADR 0007, ADR 0015).
- A read-only feed has no draft. Its routes give the `read_only` error.
- Tests use the local folder connector and need no network.

## Acceptance Criteria

Mechanical:

- The model of a corpus feed goes to JSON and back with no change. The draft then gives the same bytes as the live file.
- A sent model with one changed field changes only that element. The other entries keep their source bytes.
- A sent list in a different sequence moves the entries, and each moved entry keeps the elements that the core does not model.
- An entry that the UI does not send is removed. A sent entry with no key is added.
- A key that the draft does not have, or a key that occurs two times in one list, gives 422. The draft does not change.
- A model from a previous revision gives 409. The draft does not change.
- A new item with no GUID gets a version 4 UUID. A change of the feed GUID or of an existing item GUID gives 422.
- A cleared text field (`null`) removes its element.
- The write route writes only with the current diff ID, and gives 409 when the live file changed after the draft started.
- A write keeps a snapshot. A draft from a snapshot gives the snapshot bytes as the draft.
- Each draft route of a read-only feed gives the `read_only` error.

Visual: none.

## Test Commands

```sh
cargo test --workspace
```
