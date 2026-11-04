# Studio v1 Task 090: Bad Move Links

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0126](../adr/0126-studio-refuses-a-bad-move-link.md).
Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F0.

Status: Open. Packet 090a is complete.

## Goal

A legitimate move always works, also a return to an earlier host. Studio refuses a loop or another bad link before that link is on a host, and it gives the reason.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- Each write stays in `pipeline::write_draft` and its variants (ADR 0007, ADR 0015).
- No test sends a request outside the computer (ADR 0045).
- Do not change the bytes of a write with no followed URL and no move (ADR 0008, ADR 0012).
- Do not implement condition 5 of ADR 0126. It waits for a route in Stophammer (item 12).
- Do not add the feed check, the build date or the index view. These are F1 to F3 of the request, and they have no decision.

## Files To Read First

- ADR 0126.
- ADR 0038 items 16 to 20 and ADR 0043 items 7 to 9.
- ADR 0054 items 13 to 16 and ADR 0057 items 1 and 7.
- `crates/core/src/pipeline.rs`: `set_feed_host`, `write_draft_signed`, `put_moved_feed_file`, `put_public_feed_file`, `put_feed_from_expired`, `write_to_new_host`, `pointer_bytes`, `confirm_new_addresses` and `keep_draft_after_failed_confirmation`.
- `crates/core/src/discover.rs`: `read_feed`. `crates/core/src/web.rs`: `Web::with_policy`. `crates/core/src/net.rs`: `PublicRules::with_test_exceptions`.
- `crates/core/tests/pipeline.rs`: `move_setup` and the feed move tests.
- `crates/api/src/routes.rs`: `write_feed` and `set_feed_host`. `crates/api/ui/editor.js`: the Review sheet and `FeedMoveRow`.

## Packets

Do the packets in this sequence. Each packet is one commit, with the four commit checks.

### 090a: The Names And Stage 1

ADR 0126 items 1 to 11, 13, 17 and 24 to 27.

- A new core module `move_link` holds the names of a body, the exact comparison and the walk of a chain. It has no knowledge of a host.
- A new trait `FeedReader` gives the read of a public feed. Its real form uses `read_feed` through a `Web` with `NetPolicy::Public` in both shells. Its test form serves bodies from a map of URLs, with a sequence of results for each URL.
- A new `PipelineErrorKind::BadMoveLink` carries the condition and each hop with its URL and its result. The API gives it as status 422 with the code `bad_move_link`.
- `write_draft_signed` runs stage 1 before any file changes, after the check of the diff ID.
- For a move, stage 1 sends one read to the destination and acts only on `address_not_public`.
- A new `pipeline::check_links` gives the refusals and the notices for one draft, for Review.
- A granted write skips stage 1. The caller states this with an explicit argument. Do not infer it from the diff. The write that records a grant is not a granted write.
- A change set gets the refusal of each feed, and goes on with the other feeds.

Tests: the mechanical criteria of ADR 0126 for stage 1, the self-reference, the chains, the loops, the unchanged self link and the granted write.

### 090b: Stage 2 And The Placed File

ADR 0126 items 14 to 16, 18 and 19.

- `write_draft_signed` gets the sequence of item 16. Stage 2 runs in `put_moved_feed_file` after `write_to_new_host` and before `add_snapshot`. It also runs after `put_public_feed_file` and `put_feed_from_expired`.
- Stage 2 makes 3 reads, with 10 seconds between them. The wait goes through `FeedReader`, so that a test does not wait.
- A new migration adds the table `placed_feed_files`, with the connector, the path and the feed.
- A refusal in stage 2 records the placed file and keeps the draft move. The uploads and the copies get a confirmation, as `keep_draft_after_failed_confirmation` does. Do not use that function directly, because it removes the move and changes the base of the draft.
- `write_to_new_host` replaces a placed file of this feed when the file has the `podcast:guid` of the draft. Other bytes with a different GUID give `FileExists`. A successful move removes the row.
- `set_feed_host` accepts a placed file of this feed.

Tests: the mechanical criteria of ADR 0126 for a refused destination, the next write after a repair, and a slow destination.

### 090c: A Return To An Earlier Host

ADR 0126 items 21 to 23.

- `set_feed_host` accepts a file at the path when `feed_pointers` has that path for this feed. The current `is_feed_pointer` does not compare the feed, so add a comparison.
- `write_to_new_host` replaces that file when it has the `podcast:guid` of the draft.
- The write removes the row of the path of the new host from `feed_pointers`, and the host from before gets its row.
- The diff adds a value that says that the pointer file becomes the feed. The value is a part of `FeedMove`, and it goes into `feed_move_bytes`.

Tests: the mechanical criteria of ADR 0126 for a return and for a file with a different GUID. Also two returns in sequence: A to B, B to A, A to B.

### 090d: The API And Review

ADR 0126 items 6, 10 and 20.

- Before this packet, show the operator a mockup of a refusal and of a notice in Review. The operator selects the form. Then write the UI.
- A route gives the result of `pipeline::check_links`. The read runs outside the lock of the workspace.
- Two core routes give the repairs "Use the Public Address" and "Remove the Tag". Each edits the draft, so the diff shows the change.
- A model module holds the text of each condition, with a Node test. The UI calculates no condition.
- `FeedMoveRow` shows the value of 090c.
- Update `docs/user-guide/` for a refused move and for a return.

No feed rule in the UI (ADR 0065). Review each new UI function that changes the model.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0126.
- An API test: a refused write gives status 422 and `bad_move_link`, with each hop. No file changes on a host.
- An API test: each repair route changes only its one value in the draft.
- A Node test: the model module gives a text for each condition that the core can give.
- The four commit checks.

Visual, by the planning model. Use `tools/drive-ui.mjs` with the local test servers:

- Review shows a refusal with each hop, and the notice, at 1280 px and at 390 px with touch.
- The row of a return says that the pointer file becomes the feed.
- `$edges()` gives no result. At 390 px, `$fieldText()` and `$overflow()` give no result.

Human, by the operator. This gate stays open until it runs:

- A move between two real file hosts, and a return. A second app reads the feed at the new address after each move.
- A refused move on a real file host, then a repair, then the next write.

## Open Items That This Task Does Not Close

- Condition 5 needs a lookup of a feed by URL in Stophammer. That needs a request in `../stophammer/docs/plans/` and a decision there.
- A later connector with `set_redirect` needs a record for the removal of a redirect (ADR 0126 item 23).

## Stop And Ask When

- A correct move in a corpus feed or an existing test gets a refusal.
- A connector cannot replace a file at a known version, so a return cannot use it.
- The 20 seconds of stage 2 stop other work of the workspace in a way that a user can see.
- The Review form needs a shared rule that no musicindex ADR owns.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
