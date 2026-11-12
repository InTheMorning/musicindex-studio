# Studio v1 Task 091: The Own Address Tag

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0128](../adr/0128-a-moved-feed-names-its-own-address.md).

Status: Complete, with its screenshot visual check. The human test is open.

## Goal

A feed that Studio moves has the tag that the index needs for a later return, and Review warns the user at each return.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- Each write stays in `pipeline::write_draft_signed` and its variants (ADR 0007, ADR 0015).
- No test sends a request outside the computer (ADR 0045).
- A feed that never moved keeps its bytes (ADR 0008, ADR 0012).
- Do not add the index view, condition 5 or an update of an older pointer.

## Files To Read First

- ADR 0128, and ADR 0126 items 5, 16 and 21.
- `../stophammer/docs/adr/0083-a-feed-move-refuses-a-loop.md`, sections 2 and 5. Examine its status first.
- `crates/core/src/pipeline.rs`: `set_feed_host`, `restore_self_link`, `pointer_bytes` and `feed_move`.
- `crates/core/tests/move_link.rs`: the tests of a return.
- `crates/api/ui/movelink.js` and `FeedMoveRow` in `crates/api/ui/editor.js`.

## Packets

Each packet is one commit, with the four commit checks.

### 091a: The Tag

ADR 0128 items 1 to 6.

- `set_feed_host` sets the tag of the draft to the URL of the destination.
- Undo of a move puts back the tag of the base.
- Each test of a return examines the tag at the destination. The failure message names Stophammer ADR 0083 section 2 and ADR 0128.

### 091b: The Warning

ADR 0128 items 7 to 9.

- A text in `movelink.js`, with a Node test. `FeedMoveRow` or its neighbor shows it when `replaces_pointer` is true.
- Update `docs/user-guide/review-and-write.md`.

No feed rule in the UI (ADR 0065). Review each new UI function that changes the model.

## Acceptance Criteria

Mechanical: the mechanical criteria of ADR 0128, and the four commit checks.

Visual, by the planning model: the visual criterion of ADR 0128. `$edges()` gives no result. At 390 px, `$fieldText()` and `$overflow()` give no result.

Human, by the operator: the human criterion of ADR 0128. Stophammer release 0.18.1 holds ADR 0083, and the live node runs `v0.19.0-rc.7`.

Agent check, 2026-11-08: a scratch copy of Stophammer at `398d10f` parsed the bytes that Studio writes. It sent them to `POST /ingest/feed`. The live node runs that revision. The parser was `stophammer-parser` at `77a4ed8`.

- A move from A to B gave B. The return gave A, with no wait.
- A second move in the first hour stayed at A, with the refusal of ADR 0083. After the wait, it gave B.
- A return with the tag removed stayed at B, with the refusal of ADR 0083.
- A crawl of A before the B pointer got a refusal. The next crawl of A gave A. The crawler follows the tag of the B pointer to A.

Human check, 2026-11-12, in the release build of local `main`, with the test album `7ea30e6c-c7ba-5b81-a7be-6465c1591b7a`:

- The move from bastone to heycitizen gave heycitizen in the index at 3:50:03 PM.
- Review of the move back to bastone showed the return warning.
- After the move back, the index crawled at 4:06:17 PM and kept the heycitizen address. The file at heycitizen names the bastone address in its tag. Inference: this is the second return of the day, and Stophammer ADR 0083 section 5 gives it a wait of 1 hour from the last return. The check of the index after the wait is open.

The screenshot check of the warning did not run again at `c02910d`, because the sandbox of that session stops Chromium.

## Stop And Ask When

- The status or the rule of ADR 0083 changed.
- An existing test of a move fails for a cause other than the new tag.
- A corpus feed gets a refusal from the new tag.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
