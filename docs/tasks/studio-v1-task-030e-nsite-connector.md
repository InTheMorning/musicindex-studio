# Studio v1 Task 030e: The nsite Connector

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 1, 4 to 11 and 13, [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 12 to 17, [ADR 0044](../adr/0044-the-nsite-signer.md) items 4 to 6.
Sequence: the fifth packet of task 030. It depends on 030d.

## Goal

`studio-core` has the connector kind `nsite`. It holds the feed XML and the media of one named site. A write uploads the blobs, and `finish` publishes one manifest. It passes the trait suite against `TestRelay` and `TestBlossom`.

## Files

Inspect: `crates/core/src/connector.rs`, `crates/core/src/connector/blossom.rs` and `blossom_http.rs` (the uploads, the signing and the plan), `crates/core/src/connector/relay.rs`, `crates/core/src/nostr_events.rs`, `crates/core/src/connector/suite.rs`, `crates/core/src/testing/relay.rs`, `crates/core/src/testing/blossom.rs`, `crates/core/tests/pipeline.rs`.

Change: add `crates/core/src/connector/nsite.rs`. Change `connector.rs` (the module line, the export and `from_row_in`) and `crates/core/tests/pipeline.rs` (one test).

Do not change: the pipeline code, the API, the UI, the other connectors, the suite, the test servers.

## Architecture

These choices are fixed. Do not change them.

### The Host

- `pub struct NsiteHost` with `pub const KIND: &'static str = "nsite"`. Its configuration JSON has `public_key`, `d`, `relays` (one or more `ws` or `wss` addresses), `servers` (one or more `http` or `https` addresses), `gateway` (a host name, `nsite.lol` when it is absent), `signing` (`"vault"` or `"signer"`) and `title` (optional).
- `d` must pass `nostr_events::valid_site_name`, or the row gives `InvalidConfig`.
- The secret, the key check and the `Debug` text are as for `BlossomHost`.

### The View

- The connector reads the newest manifest of the site at its first need: `relay::newest` over `relays`, with the filter kind 35128, the author `public_key` and `#d` = `d`. No manifest on the relays is an empty site. No answer from each relay gives `Unavailable`.
- The view is that manifest with the changes of this write. A write adds or replaces a path. A delete removes a path. A Studio path `a/b.mp3` is the manifest path `/a/b.mp3`.

### The Operations

- The version and the stamp of a path are `sha256:<hex>`.
- `read`: the SHA-256 of the path in the view, then `get` from each server in sequence until one gives the bytes. A path that is not in the view gives `None`.
- `write` and `write_from`: compare `Expected` with the view, as `BlossomHost` compares it with its records. Then spool, upload to the first server, and mirror to the others, as `BlossomHost` does. The view gets the path, the SHA-256 and the size.
- `stat` and `list`: the paths of the view, with the limits and the hidden names of `LocalFolder::list`. The size comes from the write, or from `head` on the servers in sequence. The connector keeps each size that it found.
- `delete(path, stamp)`: a path that is not in the view gives `NotFound`. A different stamp gives `Conflict`. The path goes out of the view. The connector sends no `DELETE` to a server ([ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) item 11).
- `sha256`: the SHA-256 of the path in the view, with no request.
- `public_url`: `https://<base36 public key><d>.<gateway>/<path>`.

### `finish`

With no change since the last `finish`, the function does nothing. If not, it does these steps:

1. Read the newest manifest from each relay. When the newest manifest is not the manifest that the view started from, give `Conflict` and publish nothing.
2. Make the manifest of the view with `Manifest::unsigned`. The `servers` of the configuration are its `server` tags, and `title` is its title.
   - With a plan, use the planned manifest of this step. When its paths are not the paths of the view, give `Io` with "the write is different from its plan".
   - With no plan, the time is `max(time of the computer, newest manifest time + 1)`.
3. Sign the manifest as `BlossomHost` signs an event: with the key in the vault, or with the signed event of `env.signing`.
4. Publish it to each relay. When no relay accepts it, give `Unauthorized` when each refusal message starts with `auth-required`, `blocked` or `restricted`. Otherwise give `Unavailable`.
5. Read the newest manifest from each relay again. A newer manifest with a different ID on a relay gives `Conflict`.
6. The published manifest becomes the start of the view. The connector keeps the newest manifest ID of each relay.

### The Plan

- `plan_events(changes, plan_time)` reads the newest manifest, and gives:
  - the manifest of the view with `changes.writes`, with the time `max(plan_time, newest manifest time + 1)`.
  - when `changes.deletes` is not empty, the manifest after the deletions, with one second more.
  - one upload event for each server, with an `x` tag for each SHA-256 of `changes.writes`, when the list is not empty.
- The connector keeps the plan for the write that follows, as `BlossomHost` does. `user_signs` is true for `"signer"`.

### The Site State

- `pub fn site_state(&self) -> Result<SiteState, ConnectorError>` reads each relay. `SiteState` has, for each relay, its address and the ID and time of its newest manifest, or its error. `consistent` is true when each relay answered with the same ID. Packet 030g shows it.

## Steps

1. Add `NsiteHost`, the configuration and `from_row_in`, with tests of the configuration, `d`, the secret and the `Debug` text.
2. Add the view and the operations, with one test for each check of the suite. Each connector of a suite check uses a new `d` on two `TestRelay` and two `TestBlossom` servers.
3. Add `finish`, the plan and `site_state`, with the tests below.

## Tests

- Two connectors on one site: connector A writes `feed.xml` and calls `finish`. A new connector B reads the same bytes.
- A different client publishes a newer manifest (`TestRelay::insert`) after connector A read the view. Then the `finish` of A gives `Conflict`, and the relays get no event from A.
- A relay with `drop_events(true)`: `finish` gives `Ok`, and `site_state` gives `consistent` false.
- A delete sends no `DELETE` to a server.
- `public_url` with the public key of the first base36 test vector of 030b and `d` = `abc` gives `https://68ukf094zwffw6sq71wn17fst37v9rcdz4uimbb2s5gor83u4yabc.nsite.lol/a/b.mp3` for `a/b.mp3`.
- With `"signer"` and no plan, a write gives `SignatureMissing`, and no server has a blob.
- A plan with an old `plan_time` gives a manifest time that is larger than the newest manifest time.
- In `crates/core/tests/pipeline.rs`: a feed on an nsite host, with one upload and one deletion, writes. The relays get two manifests, and no server gets a `DELETE`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. No `unsafe` code.
- No secret key is in a log, an error message or the `Debug` text.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- `NsiteHost` passes each check of the suite against the test servers.
- The tests of this packet pass.
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

- `NsiteHost` fails a check of the suite. Do not change the suite.
- The pipeline test needs a change of the pipeline code.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030e-nsite-connector.md
- docs/adr/0048-the-nsite-and-blossom-connectors.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- crates/core/src/connector/nsite.rs has NsiteHost, the connector kind nsite. Its view is the newest manifest with the changes of the write. finish publishes one manifest, with a conflict check before and after. It passes the trait suite against TestRelay and TestBlossom.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No unsafe code. A delete sends no DELETE to a Blossom server.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The pipeline code, the API, the UI, the other connectors, the suite, the test servers.
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
