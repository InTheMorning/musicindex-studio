# Studio v1 Task 003: Feed Model And Writer

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 1.
Decisions: [ADR 0008](../adr/0008-feed-types-and-feed-identity.md) and [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md).

## Goal

Give a typed model of the feed elements that Studio edits first. Write each change back to the tree.

## Modeled Elements In This Task

- Channel: `title`, `description`, `link`, `language`, `podcast:guid`, `podcast:medium`, `podcast:locked`, `itunes:author`, `itunes:image`, `podcast:publisher`, channel `podcast:remoteItem`, `podcast:value`.
- Item and live item: `title`, `guid`, `enclosure`, `pubDate`, `itunes:duration`, `podcast:value`. Live items also have `status`, `start` and `end`.
- Value: `podcast:value`, `podcast:valueRecipient` and `podcast:valueTimeSplit`, with `podcast:remoteItem` in a split.
- Each modeled element keeps the attributes that the core does not model.

Other elements stay in the tree without a model. Subsequent tasks add them.

## Acceptance Criteria

Mechanical:

- For each corpus file, a read, a projection and a write with no edit give the same bytes.
- A change of one recipient split changes only that element in the output.
- A new value recipient appears after the last recipient of its value block.
- A removed item is not in the output. Each other item keeps its bytes.
- The projection of the namespace examples for `podcast:valueTimeSplit` gives the documented values.
- `podcast_guid_for_url` gives the two GUIDs in the namespace example for `podcast:guid`.

Visual: none.

## Test Commands

```sh
cargo test -p studio-core
```
