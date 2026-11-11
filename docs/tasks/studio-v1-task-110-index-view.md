# Studio v1 Task 110: The Index View In The Check

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0175](../adr/0175-the-check-shows-what-music-index-holds.md).
Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F3.

Status: Not started. ADR 0175 is Proposed. Packets 110a and 110b can start. Packet 110c stops at its mockup until the operator confirms it.

## Goal

The check of ADR 0174 also reads what Music Index holds for the feed. It gives a row for a different address and a row for a copy that pays other people.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- The requests to the index go out only with the reads of ADR 0174 item 16.
- No test sends a request outside the computer (ADR 0045). Use a scripted index.
- A finding never stops a write. Do not change the refusals of ADR 0126.
- Studio sends no report and resolves no copy.
- Do not add a change set of repairs (F4), a public check (F5) or a published list of rules (F6).

## Files To Read First

- ADR 0175, ADR 0174, ADR 0038 items 17 to 20 and ADR 0126 items 1 to 12.
- `crates/core/src/musicindex.rs`: `IndexFeed` and `feed`.
- `crates/core/src/feed_check.rs`, `crates/core/src/feed_reads.rs` and `crates/core/src/pipeline/check_repair.rs`.
- `crates/api/src/feed_check.rs`, `crates/api/ui/feedcheck.js` and `CheckPage` in `crates/api/ui/editor.js`.
- In `../stophammer`: `src/openapi.rs`, the paths `/v1/feeds/{guid}` and `/v1/feeds/{guid}/copies`. Read only. Do not change that repository.

## Packets

Do the packets in this sequence. Each packet is one commit, with the four commit checks.

### 110a: The Index Facts And The Findings In The Core

ADR 0175 items 2 to 4, 6, 10 and 11.

- `musicindex.rs` reads `last_build_date` and the copies of a feed.
- The facts of the check get the index facts. The check takes them as data, and it sends no request.
- The level Warning, and the findings `index_address_differs` and `copy_other_recipients`.
- The fact of item 10 for the line under the rows.

### 110b: The Reads, The Repair And The Routes

ADR 0175 items 1, 5 and 8.

- The reads of ADR 0174 item 16 also read the index, outside the lock of the workspace.
- The repair **Point the Old Address Here**, through the draft and the checks of ADR 0126.
- The route of the check gives the new findings, the line of item 10 and the text of the report of item 8.

### 110c: The Rows And The Sheets

ADR 0175 items 5 and 7 to 10.

- First a mockup of the two rows, the two sheets and the line, at 1280 px and at 390 px. Stop until the operator confirms it.
- The UI model gives the order of item 7, the button of item 9 and the report. Add a Node test.
- The page shows the rows, the sheets and the line. **Copy the Report** puts the report on the clipboard.
- Update `docs/user-guide/feed-check.md`.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0175.
- The four commit checks.

Visual, by the planning model. Use `tools/drive-ui.mjs` with a scripted index:

- The visual criteria of ADR 0175.

Human, by the operator. This gate stays open until it runs:

- The human criteria of ADR 0175.

## Stop And Ask When

- The index API does not give a field that ADR 0175 names.
- A repair would change a file that is not the pointer file of item 5.
- A finding of this task and a refusal of ADR 0126 apply to the same address.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
