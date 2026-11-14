# Studio v1 Task 140a: The Lightning Address Lookups Get Their Own Route

Decision: [ADR 0201](../adr/0201-people-opens-before-the-lightning-address-lookups.md).

Status: Open.

## Goal

1. `GET /recipients` sends no lookup (ADR 0201 item 1).
2. A new route `POST /recipients/lookups` sends the due lookups with no lock of the workspace, keeps the results, and gives `{ "checked": n }` (items 3 to 5 and 9).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/src/routes.rs:6339` `list_recipients`. It builds `known` from `known_recipients()` and from `feed_models(w)`, then calls `keysend::refresh` at about line 6395 with the lock, then `keysend_map(w)` and `group_wallets`.
- `crates/core/src/keysend.rs:195` `is_due`, `:212` `refresh` (finds the due addresses, sends the requests in threads with `LOOKUP_LIMIT`, keeps each result in the collection).
- `crates/api/src/lib.rs:337`: the routes of `/recipients`.
- `crates/api/src/feed_retire.rs` `retire_state` and `delete_draft`: the pattern `blocking` (facts with the lock), `blocking_unlocked` (network), then `blocking` (keep the result).
- Callers of `keysend::refresh` that do not change: `crates/api/src/people.rs:231` and `:292` (ADR 0201 item 8), `crates/api/src/routes.rs` `check_lnaddress` (item 7).
- Tests: `crates/api/src/tests.rs:11983` `profile_address_app` (an app with `.with_keysend(Some("http"))`), the tests of `/recipients` at about lines 2944, 10806, 10853, 11037 and 12574.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Keysend lookups | extend | crates/core/src/keysend.rs | Divide `refresh` into the due addresses, the requests (no collection) and the store. `refresh` stays as these three steps for its other callers. |
| Recipient list | extend | crates/api/src/routes.rs | Move the build of the list out of `list_recipients` into one function, so that the list and the lookups route use the same addresses. |
| Unlocked network read | reuse | crates/api/src/feed_retire.rs | The same pattern as `retire_state`: `blocking`, `blocking_unlocked`, `blocking`. |
| Keysend test app | reuse | crates/api/src/tests.rs | `profile_address_app` gives an app with a keysend scheme and a loopback server. |

## Rules For The Delegate

- Read ADR 0201 and ADR 0095 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change `AGENTS.md`, an ADR or a task file. Do not change the UI. Task 140b changes the UI.
- Do not change the schema of the collection.
- If a step needs an architecture decision that ADR 0201 does not give, stop and report.

## Changes

1. `crates/core/src/keysend.rs`: add `pub fn due_addresses(collection, scheme, addresses, now) -> Result<Vec<String>, CollectionError>`, `pub fn request_lookups(web, scheme, due: &[String]) -> Vec<(String, LookupResult)>` (no collection; the same threads and `LOOKUP_LIMIT` as today), and `pub fn store_lookups(collection, results, now) -> Result<HashMap<String, Lookup>, CollectionError>` (a failed request keeps the previous result, as today). `refresh` calls the three functions. Its behavior does not change.
2. `crates/api/src/routes.rs`: move the build of `known`, `feed_sets` and `fee_identities` out of `list_recipients` into one function. `list_recipients` uses it and does not call `keysend::refresh`.
3. Add `POST /recipients/lookups` in `crates/api/src/lib.rs` and its handler. With the lock: get the addresses of the list from change 2, the scheme, a clone of `w.web`, and `due_addresses`. With no scheme or no due address, answer `{ "checked": 0 }`. With no lock (`blocking_unlocked`): `request_lookups`. With the lock: `store_lookups`. Answer `{ "checked": n }`.
4. Change the existing tests that expect a lookup from `GET /recipients`: they call `POST /recipients/lookups` first. Do not weaken an assertion about the list.

## Acceptance Criteria

Mechanical:

1. An API test with `profile_address_app` and a loopback lookup server: `GET /recipients` sends no request to the server. `POST /recipients/lookups` sends one request for each due address and gives `checked` with that count. A second call gives `{ "checked": 0 }` and sends no request. Then `GET /recipients` shows the found wallet. Each failure message starts with "ADR 0201" and gives the fix.
2. An API test: while the loopback server holds a lookup (for example 2 seconds), a `GET /people` answers before the lookup ends. Prove that this test fails when change 3 holds the lock during the requests.
3. A core test of `due_addresses` and `store_lookups`: an address with a failed request keeps its previous `found` result.
4. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual: none. Task 140b has the screenshots.

Human: none. Task 140b has the gate of ADR 0201.

## Report

- The changed lines, with file and line.
- The test names, the proof of criterion 2, and the output of the four checks.
