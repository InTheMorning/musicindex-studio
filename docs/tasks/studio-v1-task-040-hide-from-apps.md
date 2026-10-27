# Studio v1 Task 040: Hide A Feed From Apps

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: one packet, after task 039.

Status: Complete - 2026-10-27. The four commit checks are Green. The screenshot visual check and the human test passed.

## Goal

The user sees that a feed is hidden from apps, and one switch hides it or shows it again. The operator found this problem on 2026-10-27 with `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`.

## Changes

- `crates/core/src/feed/elements.rs`: the element `Block` for `podcast:block`, with its text and its `id`.
- `crates/core/src/feed.rs`: the fields `itunes_block` and `blocks`, and `Feed::hidden` (ADR 0063 items 1 and 2).
- `crates/api/src/routes.rs`: the feed list gives `hidden` for each feed.
- `crates/api/ui/listing.js`: the model module of the switch (ADR 0063 items 2 to 5).
- `crates/api/ui/editor.js`: `ListingField` on the album page, the playlist page and the publisher page, and the notice in the review sheet.
- `crates/api/ui/shell.js` and `crates/api/ui/app.js`: "Hidden from apps" in the release list.
- `crates/api/src/ui.rs`: the server serves `listing.js`.

## Acceptance Criteria

Mechanical:

- The core tests `the_model_reads_each_block_tag_and_a_read_with_no_edit_keeps_the_bytes`, `an_unblock_writes_no_in_place_and_keeps_the_platform_block`, `a_block_of_a_feed_with_no_block_tag_adds_both_tags` and `only_a_block_for_each_app_hides_the_feed` pass.
- The Node tests of `crates/api/ui-tests/listing.test.mjs` pass.
- The Node test "the Releases list marks a feed that apps do not list" passes.
- The corpus round-trip tests pass.

Visual:

- The album page of a hidden feed with a block for one platform: the switch shows "Hidden", and "Advanced" lists the platform. In the dark theme, in the light theme and at 390 px. Passed on 2026-10-27 in a screenshot check.
- The release list shows "Hidden from apps". Passed on 2026-10-27 in a screenshot check.
- The review sheet of a hidden feed with a change shows the notice. Passed on 2026-10-27 in a screenshot check.

Human test, passed on 2026-10-27 with `test.xml` of the operator:

- With the feed of the operator, set the switch to off, examine the diff, and write. The two tags get `no`.
