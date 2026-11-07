# Studio V1 Task 098: Outside Changes

Status: Ready, after task 097.

Owner of the rules: [ADR 0150](../adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md).

## Goal

A user sees a change from outside when a page opens, and no write occurs because of the check.

## Files To Inspect

`crates/core/src/remote.rs`, `crates/core/src/merge.rs`, `crates/core/src/musicindex.rs` (`feed_url`), `crates/core/src/net.rs`, `crates/api/src/routes.rs` (`refresh_feed`), `crates/api/ui/app.js` (the Releases page, the page head), `crates/api/ui/feedfacts.js`.

## Constraints

- No feed rule in the UI (ADR 0065).
- No background worker. No vault access.
- No test sends a request to a host outside the computer (ADR 0045).

## Acceptance Criteria

Mechanical, with Rust tests against a local test server:

- A hosted file with the bytes of the last read gives no result.
- A hosted file with other bytes gives `changed`.
- A hosted file with a new `itunes:new-feed-url` gives `moved`, with the address.
- A failed read gives no result, and the earlier result stays.
- A different musicindex address gives the two addresses. No answer and no entry give nothing.
- With the setting off, the route sends no request.
- The check changes no draft and no row of the collection, except its own result.
- "Keep This Host" hides the result until the bytes change again.

Mechanical, with Node tests: the words of each result come from a model module, and the setting is a check box.

Visual, by a screenshot at 1100 px and at 390 px with touch, in the dark and the light theme, with `$edges()`, `$overflow()` and `$fieldText()`: the marks on a tile and each notice.

Human, by the operator: the feed that two collections hold, on the real file hosts.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

A connector has no public read that gives the same bytes as its own read.
