# Studio v1 Task 004: Remaining Feed Elements

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 1.
Decisions: [ADR 0008](../adr/0008-feed-types-and-feed-identity.md) and [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md).

## Goal

Model the feed elements that task 003 did not model.

## Modeled Elements In This Task

- Channel: `itunes:new-feed-url`, `atom:link`, `podcast:image`, `podcast:person`, `podcast:funding`, `podcast:txt`, `podcast:license` and `podcast:podroll`.
- Item and live item: `description`, `itunes:image`, `podcast:image`, `podcast:person`, `podcast:funding`, `podcast:transcript`, `podcast:txt` and `podcast:license`.

## Acceptance Criteria

Mechanical:

- For each corpus file, a read, a projection and a write with no edit give the same bytes.
- The corpus has a feed with each of these: a channel person, a channel funding, an `atom:link` with `rel="self"`, and an item transcript.
- A host move changes the `self` link, adds `itunes:new-feed-url` before the first item, and changes no other line.
- An edit of one item keeps the feed GUID and each item GUID.

Visual: none.

## Test Commands

```sh
cargo test -p studio-core
```
