# Studio v1 Task 030g: The nsite And Blossom Host Routes

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 1 to 5, [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 2 and 3, [ADR 0044](../adr/0044-the-nsite-signer.md) items 1, 2 and 6, [ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0021](../adr/0021-public-address-of-a-feed.md).
Sequence: the seventh packet of task 030. It depends on 030f.

## Goal

The API reads the relay list and the server list of a key. `POST /api/v1/connectors` adds an nsite site or a Blossom host, with "Key in the vault" or "My signer". The secret key goes into the vault only. The API gives the state of each relay of a site, and the user can change the relays, the servers and the gateway.

## Files

Inspect: `crates/core/src/connector/relay.rs`, `crates/core/src/connector/nsite.rs`, `crates/core/src/connector/blossom.rs`, `crates/core/src/nostr_events.rs`, `crates/core/src/address.rs` (`folder_url`), `crates/api/src/lib.rs`, `crates/api/src/routes.rs` (`NewConnector`, `add_s3_connector`, `add_sftp_connector`, `check_http_address`, `update_connector`, `set_connector_base_url`, `match_connector`), `crates/api/src/workspace.rs`, `crates/api/src/error.rs`, `crates/api/src/tests.rs`.

Change: `relay.rs` (one public function), `nsite.rs` (one public function), `nostr_events.rs` (one constant and one function), `connector.rs` (the exports), `address.rs`, `lib.rs`, `routes.rs`, `workspace.rs`, `error.rs`, `tests.rs`.

Do not change: the UI, the pipeline, the connector operations.

## Architecture

These choices are fixed. Do not change them.

### The Core Additions

- `nostr_events::LOOKUP_RELAYS`: `["wss://purplepag.es", "wss://user.kindpag.es"]`.
- `nostr_events::site_name_from_guid(guid) -> String`: the text before the first `-` of the GUID, in lowercase, with a maximum of 8 characters ([ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) item 4).
- `connector::key_lists(public_key, lookup_relays) -> Result<KeyLists, ConnectorError>`: the newest kind 10002 and kind 10063 event of the key from the lookup relays. `KeyLists.relays` has each `r` tag with no marker or with the marker `write`. `KeyLists.servers` has each `server` tag. No answer from each relay gives `Unavailable`. No event gives empty lists.
- `connector::nsite_site_state(row, vault) -> Result<SiteState, ConnectorError>` makes the `NsiteHost` of the row and gives its `site_state`.
- `address::folder_url` gives `https://<base36 public key><d>.<gateway>` for an nsite row, and `None` for a Blossom row.

### The Routes

- `POST /api/v1/connectors/nostr-lists` with `{"key": "<npub, nsec or hex>"}` gives `{"public_key": "<hex>", "npub": "...", "relays": [...], "servers": [...]}`. It needs no vault. A 64-character hexadecimal key is a public key. With an `nsec`, the route derives the public key and keeps no part of the secret key. A key that does not parse gives 422 `invalid_config`. No answer from the lookup relays gives 503 `host_unavailable`.
- `Workspace` gets `lookup_relays: Vec<String>`, with `LOOKUP_RELAYS` as the default. A test sets the address of a `TestRelay`.
- `NewConnector` gets the variants `Nsite` and `Blossom`:
  - `Nsite`: `name`, `public_key` (optional with `"vault"`), `d` (optional), `feed_id` (optional), `relays`, `servers`, `gateway` (optional), `title` (optional), `signing` and `secret_key` (for `"vault"` only).
  - `Blossom`: `name`, `public_key` (optional with `"vault"`), `servers`, `signing` and `secret_key` (for `"vault"` only).
- The add route does these steps, in this sequence:
  1. With `"vault"` and no open vault, it gives 409 `vault_required` or 423 `vault_locked`, as for S3. `"signer"` needs no vault.
  2. It checks the fields:
     - With `"vault"`, `secret_key` must parse, or the route gives 422 `invalid_config`. A `public_key` that is not the key of `secret_key` gives 422 `key_mismatch`.
     - With `"signer"`, `public_key` must parse, and `secret_key` must be absent. If not, the route gives 422 `invalid_config`.
     - Each server must be an `http` or `https` address, or the route gives 422 `invalid_address`. An empty server list gives 422 `invalid_config`.
     - For nsite: each relay must be a `ws` or `wss` address, and the list must not be empty. With no `d`, the route uses `site_name_from_guid` of the GUID of `feed_id`. `d` must pass `valid_site_name`. The gateway is a host name, with `nsite.lol` as the default. A field that is not correct gives 422 `invalid_config`.
  3. It connects:
     - For nsite, it reads the newest manifest of the site. No answer from each relay gives 503 `host_unavailable`. No manifest is not an error.
     - It sends `HEAD /<64 zeros>` to each server. A connection failure gives 503 `host_unavailable`. A `404` is the correct answer.
  4. With `"vault"`, it puts `{"secret_key": "<hex>"}` into the vault with the ID `connector-<uuid v4>`, and saves the vault.
  5. It adds the collection row. When step 4 or step 5 fails, it removes the secret from the vault again, as `add_s3_connector` does.
- When step 3 fails, nothing goes into the vault or the collection. The response is the connector row, with no secret.
- `GET /api/v1/connectors/{id}/site` gives `{"consistent": bool, "relays": [{"url", "id", "created_at", "error"}]}` for an nsite row. A different kind gives 422 `invalid_config`. A locked vault with `"vault"` gives 423 `vault_locked`.
- `PUT /api/v1/connectors/{id}` accepts `relays` and `gateway` for an nsite row, and `servers` for an nsite row and a Blossom row. The checks of step 2 apply. A field for a different kind gives 422 `invalid_config`.
- The match route works for an nsite host as for an S3 host. A feed whose address is the gateway address of a path in the manifest matches.

## Steps

1. Add the core additions, with tests against `TestRelay` for `key_lists` and `nsite_site_state`, and a test of `folder_url`.
2. Add the lookup route, with API tests.
3. Add the `Nsite` and `Blossom` variants of the add route, with an API test for each step and each error.
4. Add the site route and the changes of `PUT`, with API tests.
5. Add an API test of the match route for an nsite host.

## Tests

- The lookup route gives the relays and the servers of a key from a `TestRelay`, and empty lists for a key with no events.
- An nsite host with `"vault"` is added. The vault holds the key under the `secret_ref` of the row. The secret key is not in the row, the response, the data files or the vault file as plain text.
- An nsite host with `"signer"` is added with no vault.
- A secret key of a different public key gives 422 `key_mismatch`, and the vault and the collection do not change.
- A relay address with no server gives 503 `host_unavailable`, and the vault and the collection do not change.
- With no `d` and a `feed_id`, the row has the `d` of the GUID of the feed.
- The site route gives `consistent` false for a site with a relay that has `drop_events(true)`.
- `PUT` with new servers changes the row.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- No secret key is in a log, an error message, an API response, the collection row or the collection database file.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- The tests of this packet pass.
- A failed connection changes neither the vault nor the collection.
- The four commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report in these cases:

- The match route needs more than a small change for the kind `nsite`.
- A route needs a change of a connector operation.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030g-host-routes.md
- docs/adr/0048-the-nsite-and-blossom-connectors.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- The API reads the relay list and the server list of a key. POST /api/v1/connectors adds an nsite site or a Blossom host with "Key in the vault" or "My signer". GET /connectors/{id}/site gives the state of each relay. PUT /connectors/{id} changes the relays, the servers and the gateway.

Constraints:
- Follow the Architecture section of the task file fully, including the error codes and the step sequence of the add route.
- No new crate. No secret key in a log, an error message, an API response or the collection.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the pipeline, the connector operations.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
