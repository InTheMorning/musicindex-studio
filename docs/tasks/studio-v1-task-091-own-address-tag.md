# Studio v1 Task 091: The Own Address Tag

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0128](../adr/0128-a-moved-feed-names-its-own-address.md).

Status: Complete, with its screenshot visual check. The human test waits for a release of Stophammer that holds ADR 0083.

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

Human, by the operator: the human criterion of ADR 0128. This gate stays open until a release of Stophammer holds ADR 0083.

## Stop And Ask When

- The status or the rule of ADR 0083 changed.
- An existing test of a move fails for a cause other than the new tag.
- A corpus feed gets a refusal from the new tag.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
