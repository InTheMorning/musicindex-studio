# Studio V1 Task 097: Collections

Status: Ready, after task 096.

Owner of the rules: [ADR 0149](../adr/0149-the-desktop-app-has-many-collections.md).

## Goal

A desktop user makes, copies, imports, opens and deletes collections. A test needs no scattered data folders.

## Packets

- 097a: the folders, the move at the first start and the file that lists the collections, in `crates/desktop`.
- 097b: the routes on the local API, the copy, the delete and the restart in `crates/desktop-app`.
- 097c: the Settings section, the sheets and the row in the left panel.

## Files To Inspect

`crates/desktop/src/lib.rs` (`default_data_dir`, the lock), `crates/desktop-app/src/main.rs` (`data_dir`, "Quit"), `crates/api/src/lib.rs` (`local_router`), `crates/core/src/account_archive.rs`, `crates/api/ui/shell.js`, `crates/api/ui/pages.js`, `crates/api/ui/move.js`, `docs/user-guide/start-on-desktop.md`.

## Constraints

- No Tauri code in the UI (ADR 0124). No route of this task on a server.
- A vault stays encrypted. No secret goes to a log or an API response (ADR 0003).
- No Tauri command for the page.
- A confirmation is a sheet of `confirms.js`. A new section needs a tier in `pages.js` (ADR 0071).

## Acceptance Criteria

Mechanical, with Rust tests:

- The move gives `collections/main/` with each earlier file, and a second start changes nothing.
- A stop at each step of the move, simulated in a test, completes at the next start with no lost file.
- A folder with a held lock does not move.
- `--data-dir` does not move its folder and has no collection route.
- A new collection has its own vault. A copy opens with the passphrase of its source, and holds no app, no grant and no queued announcement.
- A delete removes only that folder, and the routes refuse a delete of "Main collection" and of the open collection.
- An import makes a new collection, and the first collection does not change.
- The server router has no collection route.

Mechanical, with Node tests: the mark shows only when the open collection is not "Main collection", and the section tier passes.

Visual, by a screenshot at 1100 px and at 390 px with touch: the section, each sheet and the row. Run `$edges()`, `$overflow()` and `$fieldText()` with each screenshot. A window check runs on Xvfb :99, with no input on the operator screen.

Human, by the operator: the upgrade of the real data folder in the AppImage of the next candidate.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

The library has a root outside the data folder, and a copy cannot hold it. A worker of the app cannot stop within the "Quit" rules.
