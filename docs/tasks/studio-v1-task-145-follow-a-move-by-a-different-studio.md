# Studio v1 Task 145: Studio Follows A Move That A Different Studio Made

Decision: [ADR 0211](../adr/0211-studio-follows-a-move-that-a-different-studio-made.md).

Status: Open. The code is not built.

## Goal

1. The outside check gives the result "moved" for a pointer file at the folder address (ADR 0211 items 1 to 5).
2. The core follows a move, with or with no connected file host for the new address (items 9 to 16).
3. The feed page shows one warning with its buttons, and the read-only notice (items 6 to 8, 15 to 18).
4. Each mark of an outside change has the Warning color (items 19 to 21).

The work has two packets. Packet 145a is the core and the API. Packet 145b is the UI. Do 145a first. One delegate does one packet.

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/core/src/outside.rs`: `OutsideResult`, `address_of`, `read_hosted`, `record_read`, `apply_read`, `stored_result`, `keep` and `compare`. `compare` gives `Moved` only when the new address is not the public address of the feed row.
- `crates/core/src/address.rs`: `AddressSource` with `Read`, `SelfLink` and `Folder`, and the differences of a feed.
- `crates/core/src/collection.rs`: `set_feed_location` (about line 2800), `add_feed_pointer` (about line 2628), `clear_feed_location` (about line 2263), `set_address` (about line 1803) and `record_state`.
- `crates/core/src/connector.rs:333`: `public_url(path)` of a connector. No function gives the connector and the path of an address.
- `crates/core/src/pipeline.rs`: `refresh` (about line 606) reads the file of a feed with a connector. The function below it reads a feed with no connector from its public address with `discover::read_public`. `move_unedited_draft` moves a draft with no edits.
- `crates/api/src/routes.rs`: `outside_check` (about line 1661) and `outside_keep`. The answer has `state`, `new_url`, `own_url` and `connected`.
- `crates/api/ui/outsidecheck.js`: `outsideMark`, `outsideNotice`, `indexNotice` and `stateMark` with `STATE_MARKS`.
- `crates/api/ui/app.js`: the notice of the outside check on the feed page (search `Keep This File Host`), `AddressDifferences`, and the marks of a tile and of the left panel.
- `crates/api/ui/editor.js`: `ReturnWarning` with the warning markup, and `OPEN_REVIEW`.
- Tests: `crates/core/src/outside.rs` (module `tests`), `crates/api/src/tests.rs` (search `the_outside_check_finds_a_change_and_a_move`), `crates/api/ui-tests/outsidecheck.test.mjs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Outside check | extend | crates/core/src/outside.rs | The check also reads the folder address, and it gets the follow of a move. |
| Feed location | reuse | crates/core/src/collection.rs | `set_feed_location`, `add_feed_pointer` and `clear_feed_location` change the feed row. |
| Read of a feed | reuse | crates/core/src/pipeline.rs | `refresh` and the public read record the file after the follow. |
| Connector address | reuse | crates/core/src/connector.rs | `public_url` gives the address of a path on a file host. |
| Outside check route | extend | crates/api/src/routes.rs | The answer gets the file host of the new address, and a new route starts the follow. |
| Outside words | extend | crates/api/ui/outsidecheck.js | The warning view, the read-only notice and the tones of the marks. |
| Feed page notices | extend | crates/api/ui/app.js | The page shows the warning and its buttons. |
| Warning markup | reuse | crates/api/ui/editor.js | The icon `alert` and the Warning color of `ReturnWarning`. |

## Rules For The Delegate

- Read ADR 0211 first, then ADR 0150, ADR 0021 and ADR 0067.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file. Do not change the schema of the collection.
- Change a shared function only as this task says. Search each caller before you change a signature.
- If a step needs an architecture decision that ADR 0211 does not give, stop and report.
- `node` is not on the PATH. Use `tools/node`. `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.
- Use the exact words of ADR 0211 for each text. Use tokens only in the styles.

## Changes Of Packet 145a: The Core And The API

1. `crates/core/src/outside.rs`, the result "moved":
    - Add a function that gives the folder address of a feed row: the address of the source `Folder` of the feed, when it is not the public address that `address_of` gives. A feed with no connector has none.
    - The check reads the folder address also, as a public address. When the file there names a different address in `itunes:new-feed-url`, the result is `Moved` with that address. This result comes before the result of the public address.
    - `compare` keeps its rule for the file at the public address.
    - A failed read of the folder address changes nothing.
2. `crates/core/src/outside.rs`, the connected file host of an address: add a function that gives the connector and the path for an address, or none. It examines each connected file host with `public_url`. Find the path from the address and the address of the root of the file host. The result must give the same address again through `public_url`.
3. `crates/core/src/outside.rs`, `follow_move`:
    - It reads the feed at the new address with the public read of the core. It gives an error when the read fails, and an error when the `podcast:guid` is different. Add an error kind for each case only if no existing kind fits.
    - With a connected file host for the address: add the pointer record for the old connector and path, then set the feed location, then record the file as the last read. Use the functions of the table above.
    - With no connected file host: add the pointer record, clear the feed location, set the public address from the read, and record the file as the last read.
    - A draft with no edits moves to the new file. A draft with edits stays.
    - After the follow, the stored result of the outside check is "same".
4. `crates/api/src/routes.rs`:
    - `outside_check` reads the folder address with no lock of the workspace, as it reads the public address. The answer gets `new_host`: the name of the connected file host that gives `new_url`, or null.
    - Add `POST /feeds/{id}/outside/follow`. It needs the rights of a writer. It reads the network with no lock of the workspace. The answer has `read_only`: true when no connected file host gives the address. Map the two errors to `409` with the codes `move_read_failed` and `move_other_guid`.
    - Add `POST /feeds/{id}/outside/use-host` for ADR 0211 item 16: it gives a read-only feed the connected file host that gives its public address. It gives `409` with the code `no_host_for_address` when none does.
    - The feed row of the API gets `host_for_address`: true for a read-only feed whose public address a connected file host gives.
5. Tests of 145a: the criteria 1 to 5 below.

## Changes Of Packet 145b: The UI

1. `crates/api/ui/outsidecheck.js`:
    - `outsideMark` gives the tone `warning` for a change and for a move.
    - `outsideNotice` for "moved" gives the warning of ADR 0211 items 6 to 8: the title, the label "New address", the address, `new_host`, the line about the pointer file, and the keys of its buttons in their sequence.
    - `outsideNotice` for "changed" has the tone `warning`.
    - Add `readOnlyHostNotice(feed)` for items 15 and 16. It gives null for a feed with a connector.
    - `STATE_MARKS.outside` has the tone `warning`.
2. `crates/api/ui/app.js`:
    - The notice of the outside check uses the warning markup for the tone `warning`.
    - **Follow the Move** calls the follow route, then loads the feed list and the draft, then runs the outside check again. An error shows in the notice.
    - **Move It Back…** opens the review sheet with `OPEN_REVIEW`. Examine first that the write of the current draft puts the feed file at the old address and gives the return warning. If it does not, stop and report.
    - A feed with the result "moved" shows no `AddressDifferences` for the new address.
    - The read-only notice shows on the page of a read-only feed that followed a move, with **Connect a File Host…** or **Use This File Host**.
    - The mark of a tile and the mark of the left panel use the Warning color for the tone `warning`.
3. `crates/api/ui/styles.css`: the Warning tone of a tile mark and of the left panel mark. Do not remove a rule that a different mark uses.
4. Tests of 145b: the criteria 6 to 8 below.

## Acceptance Criteria

Mechanical:

1. A core test: a pointer file at the folder address gives `Moved` with the new address. This is also true when the public address of the feed row is already the new address. A feed file at the folder address gives the result of the public address.
2. A core test: `follow_move` with a connected file host gives the feed that connector and path, adds the pointer record, and the stored result is "same".
3. A core test: `follow_move` with no connected file host gives a feed with no connector and the new public address. Its draft with edits and its snapshots stay.
4. A core test: `follow_move` gives an error for a failed read, and an error for a different `podcast:guid`. The feed row does not change.
5. An API test: the follow route gives `read_only` false and true for the two cases, and `409` with each code. The use-host route gives the feed its connector, and `409` with no host.
6. A Node test of `outsideMark`, `outsideNotice` and `STATE_MARKS`: the tone `warning`, the texts of ADR 0211 items 6 and 21, and the button keys. Each failure message starts with "ADR 0211" and gives the fix.
7. A Node test of `readOnlyHostNotice`: the text of item 15, the button of item 15 or of item 16, and null for a feed with a connector.
8. A Node scan test of `app.js`: the notice calls `outside/follow`, and the text "moved outside Studio" is not in the UI files.
9. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The warning of a moved feed, the page after the follow with a connected file host, and the page after the follow with no connected file host, at 1280 px light and 390 px dark.
2. A tile and a row of the left panel with the mark of a change and of a move.
3. The review after **Move It Back…**, with the return warning.

Human, by the operator: the gate of ADR 0211.

## Report

- The changed lines, with file and line.
- The test names, and the output of the four checks.
- Each point where the code did not agree with this task.
