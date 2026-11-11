# Research Request: The Stream And The Content Link Of A Live Item

## Status

Answered - 2026-11-11. Each section has an answer. Received - 2026-11-09.

This request comes from the research repository at
`../musicindex-research`, commit `82f56fe`. That repository has no Git remote,
and it is private. The note is
`research_notes/podcast-apps/live-item-stream-and-content-link.md`. The
evidence is in `sources/captures/2026-11-09-live-item-stream/`.

This document binds nothing in Studio. Studio records its decision in
[ADR 0161](../adr/0161-a-live-item-needs-a-stream-and-a-content-link-is-recommended.md).

The research examined Studio at commit `915cd5e`.

## Purpose

Answered. ADR 0161 items 1 to 4 give L1 to L3, and commit `d577def` puts them in
the core. ADR 0162 and task 100 fill and check the stream type.

| # | Change | Why |
|---|---|---|
| L1 | Refuse a live item with no stream URL | Partytime drops a live item with no enclosure. Podverse and Boost Me Bitch play the enclosure. |
| L2 | Refuse a stream type that does not agree with the stream URL | Two Pixel Palace snapshots had an HLS manifest URL with the type `audio/mpeg`. |
| L3 | Make the content link a warning, not a refusal | 8 of 14 published live items have no content link. The search found no app that plays a content link. |

## Findings At Studio `915cd5e`

Answered by ADR 0161. These findings describe the code before that record.

- `crates/core/src/live.rs:255` gives "This show has no content link." ADR 0054 item 4 makes it a refusal.
- `live::problems` checks `start`, `end`, the content link and `status`. The search found no check of the stream URL or the stream type.
- Thus Studio refuses 8 of the 14 published live items, and it accepts a live item with no stream.

## The Decision Of Studio

Answered by ADR 0161.

ADR 0161 agrees with L1 to L3. It adds a table of extensions and types for L2. A URL with no extension in that table accepts each type that is not blank.
