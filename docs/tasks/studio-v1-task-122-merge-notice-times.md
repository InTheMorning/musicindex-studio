# Studio v1 Task 122: The Merge Notice Gives The Times Of The Change

Decision: [ADR 0186](../adr/0186-the-merge-notice-gives-the-times-of-the-change.md).

Status: Not started.

## Goal

The notice of a hosted file that changed after the draft started has a neutral text. It gives the time that the draft started and the time of the change.

## Constraints

- The core gives the two times. The UI calculates no time (ADR 0186 item 3).
- **Apply My Edits to the New File** does not change.
- No test sends a request outside the computer (ADR 0045). Use the test connectors.

## Files To Read First

- ADR 0186 and ADR 0129.
- The conflict notice in `crates/api/ui/editor.js`.
- The `Conflict` error kind in `crates/core/src/pipeline.rs`.
- The read of a feed file in `crates/core/src/connector.rs`. The connectors give no `Last-Modified` time at this time.

## Work

- The conflict error of the core gives the start time of the draft and the time of the change.
- The time of the change is the `lastBuildDate` of the hosted file. With no `lastBuildDate`, it is the `Last-Modified` time of the file host. With no time from the file or the file host, the error gives no time of the change.
- Add the `Last-Modified` time to the read of a feed file for each connector that can give it. A connector with no such time gives none.
- The notice model gives the text of ADR 0186 item 1. It gives no line "File changed" with no time.
- Add the Node test and the Rust test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0186.

Visual, by the planning model:

- The notice at 1280 px and at 390 px, in the light and dark themes.

Human: none (ADR 0186).

## Stop And Ask When

- The `Last-Modified` time needs a change of the connector interface of ADR 0015 for each connector.
