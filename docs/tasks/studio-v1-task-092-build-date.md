# Studio v1 Task 092: The Build Date

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0127](../adr/0127-a-changed-feed-gets-a-new-build-date.md).
Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F2.

Status: Open. Packets 092a and 092b are complete.

## Goal

A change of a feed sets `lastBuildDate` from a checked time. An incorrect clock cannot put a date in the future, and it never stops a write.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- Each write stays in `pipeline::write_draft_signed` and its variants (ADR 0007, ADR 0015).
- No test sends a request outside the computer (ADR 0045).
- A read and a write with no edit give the same bytes (ADR 0008, ADR 0012). The corpus tests do not change.
- A clock difference and a failed time request never stop a write.
- Do not add the index view or the feed check. These are F3 and F1 of the request.

## Files To Read First

- ADR 0127, and Stophammer ADR 0053 section 3.
- `crates/core/src/web.rs`, `crates/core/src/datetime.rs` and `crates/core/src/feed.rs`.
- `crates/core/src/pipeline.rs`: `revision`, `edit_draft`, `changed_draft`, `replace_model`, `apply_repair`, `draft_from_snapshot`, `rebase_linked_draft`, `copy_feed`, `diff_at`, `write_granted_with` and each call of `save_draft`.
- `crates/core/src/grant.rs`: `covered`.
- `crates/api/src/routes.rs`: `blocking`, `blocking_unlocked`, `diff_json` and each of its callers. `crates/api/src/workspace.rs`: `link_reader` and `wall_clock`.
- `crates/api/ui/review.js`, and `ReviewSheet` in `crates/api/ui/editor.js`.

## Packets

Do the packets in this sequence. Each packet is one commit, with the four commit checks.

### 092a: The Checked Time

ADR 0127 items 7, 9, 10, 13 and 14.

- The typed model reads and writes `lastBuildDate`. An unchanged value keeps its bytes.
- A new core module gives the checked time, the kept differences and the rule for a new date. It has no knowledge of a draft.
- A new trait gives the time of a server. Its real form sends a `HEAD` request with the policy `Public`, a limit of 5 seconds and no redirect. It reads the `Date` and `Age` headers at each status. Its test form serves a time for each URL and records each request.

### 092b: The Date In The Draft

ADR 0127 items 1 to 6, 8, 11, 12, 16 to 22.

- Each path that saves a changed draft gives it the date of its base.
- A new core function sets the date before a diff, with the rule of 092a. It sets the date one time for each change.
- `revision` ignores `lastBuildDate`, so that a date does not make the revision of the editor stale.
- The editor cannot change this value through `replace_model`.
- Each route that gives a diff gets the time outside the lock of the workspace, and then sets the date under the lock.
- The diff JSON gives the state of the date and each notice as data.
- A granted write sets the date, `grant::covered` ignores it, and a failed request does not stop the write.

### 092c: The Review

ADR 0127 items 23 and 24.

- The field view names the date "Build date".
- A model module holds the text of each notice, with a Node test. The UI calculates no clock difference.
- Update `docs/user-guide/review-and-write.md`.

No feed rule in the UI (ADR 0065). Review each new UI function that changes the model.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0127.
- The four commit checks.

Visual, by the planning model. Use `tools/drive-ui.mjs` with a test time source:

- The visual criterion of ADR 0127.
- `$edges()` gives no result. At 390 px, `$fieldText()` and `$overflow()` give no result.

Human, by the operator. This gate stays open until it runs:

- The human criteria of ADR 0127.

## Stop And Ask When

- A corpus feed changes its bytes with no edit.
- A diff ID changes with no edit.
- A granted write fails only because of this date.
- A `Date` header of a real form cannot be read by `datetime::parse_rfc2822`, and the correction is not small.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
