# Studio v1 Task 120: After A Move Studio Shows The Podping Of The Move

Decision: [ADR 0184](../adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md).

Status: In progress. Packets 120a and 120b are complete on 2026-11-12. 120a has its screenshot visual check. Packet 120c (the notice) is not started.

## Goal

The **Files** page shows the line of the new feed address only for a new value. After a move by Studio, the release page shows a notice with the state of the podping of the move. The user can send a podping that failed or that was not sent.

## Constraints

- No feed rule in the UI (ADR 0065). The core gives the last written move and the podping state. The UI calculates no time.
- A send uses the queue of ADR 0052, with its wait and its tries. No other page has a send button (ADR 0184 item 10).
- No test sends a request outside the computer (ADR 0045). Use the test hub of the announce tests.
- Task 116 also changes the Files page. Do task 116 first.

## Files To Read First

- ADR 0184, ADR 0150 item 5, ADR 0052 items 8, 9, 13 to 21, and ADR 0128 item 6.
- `crates/api/ui/editor.js`: the line "The feed file names the new feed address".
- `crates/api/ui/feedfacts.js`: the tag `itunes:new-feed-url` of the feed file.
- `crates/api/ui/outsidecheck.js` and `crates/api/ui-tests/outsidecheck.test.mjs`: the notice of a different address.
- `crates/core/src/outside.rs` and `crates/core/src/moves.rs`.
- `crates/core/src/announce.rs`, and `list_announcements` in `crates/api/src/routes.rs`.

## Packets

### 120a: The Line Of The Feed Address

ADR 0184 items 1 and 2.

- The Files page model gives the line only when the tag of the draft differs from the tag of the hosted file.
- Add the Node test of the ADR.

### 120b: The Last Move And The Send

ADR 0184 items 3, 10 and 11.

- The check of ADR 0150 item 5 also gives the last written move of the feed in this collection. It gives the source address and the time of that move.
- Add a route that puts one announcement of the current public address into the queue. A feed with no public address gives a refusal.
- Add the Rust test of the ADR.

### 120c: The Notice

ADR 0184 items 3 to 9.

- The address at Music Index can be the source of the last move. Then the notice model gives the notice of a move by Studio, in the information style.
- The notice gives one podping line for each announced address, with the address first. Use the four lines of item 5.
- A failed podping has **Send the Podping Again**. A podping that was not sent has **Send the Podping**. The buttons use the route of 120b.
- For each other address, the model gives the notice of ADR 0150 item 5 with its text.
- Add the Node tests of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0184.

Visual, by the planning model:

- The notice with each podping state, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0184, with a real move between two file hosts.

## Stop And Ask When

- The announcement queue cannot give the state of the announcements of one write.
- A send with no write needs a change of a rule of ADR 0052 other than item 21.
