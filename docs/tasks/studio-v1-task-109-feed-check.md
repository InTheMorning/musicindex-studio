# Studio v1 Task 109: The Feed Check

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0174](../adr/0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md).
Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F1.

Status: Not started. The operator accepted ADR 0174 on 2026-11-11. Packet 109c waits for the operator to confirm the tab form of ADR 0174 item 17 on its mockup.

## Goal

The core examines a feed and gives a list of findings. Each finding has a level, a reason, and a repair that goes to the draft when one exists. The page of a feed shows the findings in the tab **Check**.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- The check changes no feed. A repair changes the draft only when the user selects its button (ADR 0007).
- A read and a write with no edit give the same bytes (ADR 0008, ADR 0012). The corpus tests do not change.
- No test sends a request outside the computer (ADR 0045).
- A finding never stops a write. Do not change the refusals of ADR 0126.
- Do not add the index view (F3) or a change set of repairs (F4).
- Do not add a public check (F5) or a published list of rules (F6).

## Files To Read First

- ADR 0174, ADR 0126 items 1 to 12, ADR 0021, ADR 0023 item 5, ADR 0130 and ADR 0135.
- `crates/core/src/move_link.rs`: `check`, `Repair`, `applicable_repairs` and `repair_applies`.
- `crates/core/src/pipeline.rs`: `new_feed`, `apply_feed_change`, `set_self_link`, `copy_feed`, `changed_draft` and the repair code of `move_link`.
- `crates/core/src/address.rs`, `crates/core/src/web.rs` and `crates/core/src/live.rs` (`problems` gives a list with levels).
- `crates/api/ui/shell.js`: `pageTabs`. `crates/api/ui/editor.js`: the page of a feed.
- `.studio-dev/mockup-feedcheck/steps.py` and `.studio-dev/mockup-feedcheck/shots/a-1280.png`.

## Packets

Do the packets in this sequence. Each packet is one commit, with the four commit checks.

### 109a: The Check In The Core

ADR 0174 items 1 to 12.

- A new core module `feed_check` gives the finding type, the levels and the check function of item 1. The function takes the facts as data, and it sends no request.
- The local findings of item 7: `no_self_link`, `self_link_differs`, `self_link_spelling`, `no_file_size`, `no_duration` and `no_language`.
- The spelling rule of item 12.
- No finding for a link that `move_link::check` refuses (item 5).
- `pipeline::new_feed` gives a new feed with a public address its self link (item 11).
- The field targets of **Open the Track** and **Select a Language**. These buttons need no read.

### 109b: The Reads, The Repairs And The Routes

ADR 0174 items 10, 13 to 16 and 18.

- The facts step of item 4, with `NetPolicy::Public`. It reads the chain of the tag (item 13), the `https` address of each `http` file (item 14) and the `Content-Length` of each enclosure.
- The findings `http_file`, `wrong_file_size` and `move_not_confirmed`.
- The repairs **Add the Link**, **Use the Public Address**, **Use the https Address** and **Read the Size**. **Add the Link** and **Use the Public Address** first confirm the body at the public address (item 10).
- A route gives the findings of a feed. A second route applies one repair and gives the new findings. Run each read outside the lock of the workspace.
- The API keeps the last read facts of each feed in memory (item 18).

### 109c: The Tab And The Page

ADR 0174 items 17 and 19 to 21.

- First make a mockup of the tab **Check** at 1280 px and at 390 px. Give it to the operator. Do not build the page until the operator confirms the tab form.
- `pageTabs` gets the tab **Check** with its count, before **Settings**.
- A model module gives the rows, the button labels and the texts of items 19 to 21, with a Node test.
- The page shows each row, **Check Again**, the text for a read-only feed and the text for a feed with a grant.
- Update `docs/user-guide/` with a page for the feed check.

No feed rule in the UI (ADR 0065). Review each new UI function that changes the model.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0174.
- The four commit checks.

Visual, by the planning model. Use `tools/drive-ui.mjs` with test servers for the reads:

- The visual criteria of ADR 0174.

Human, by the operator. This gate stays open until it runs:

- The human criteria of ADR 0174.

## Stop And Ask When

- A corpus feed changes its bytes with no edit.
- A corpus feed gives a finding that the research names as not a defect (ADR 0174 item 8).
- A repair would change an element that its finding does not name.
- A finding and a refusal of ADR 0126 apply to the same link.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
