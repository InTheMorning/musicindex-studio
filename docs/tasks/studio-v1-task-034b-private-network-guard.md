# Studio v1 Task 034b: The Private-Network Guard

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md).
Sequence: the second packet of task 034. It depends on 034a.

## Goal

The core has the network policies `Open` and `Public`. With `Public`, no outbound connection to an address that a user gives goes to a private, loopback, link-local or reserved address. The check uses the resolved address, so DNS rebinding cannot go around it.

## Files

Inspect: `crates/core/src/web.rs` (`Web::new`, the two `ureq` agents), `crates/core/src/connector/s3.rs` (the agent near line 148), `crates/core/src/connector/blossom_http.rs` (the agent near line 144), `crates/core/src/connector/relay.rs` (the connection near line 120), `crates/core/src/connector/sftp.rs` (the connection near line 544), `crates/core/src/connector.rs` (`HostEnv`, `from_row_in`), `crates/api/src/workspace.rs` (`Workspace::open`, `hosts_with`), `crates/api/src/routes.rs` (`connector_fingerprint`, `discover`, `get_image`, `media_facts`), the `ureq` 3.5 module `unversioned::resolver`.

Change: the new file `crates/core/src/net.rs`, `crates/core/src/lib.rs`, the five client files above, `crates/core/src/connector.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/error.rs`, the tests of each.

Do not change: the musicindex client and the hub client. The operator gives their addresses, so they keep the policy `Open`.

## Architecture

These choices are fixed. Do not change them.

### The Core

- `net.rs` has `pub enum NetPolicy { Open, Public(PublicRules) }`. `PublicRules` has a private list of permitted socket addresses. Only `PublicRules::with_test_exceptions(Vec<SocketAddr>)` sets it, under `#[cfg(any(test, feature = "testing"))]`. `PublicRules::default()` has an empty list.
- `pub fn is_global(ip: IpAddr) -> bool` refuses each range of ADR 0057 item 2. An IPv4-mapped IPv6 address and a `64:ff9b::/96` address use the check of the IPv4 address in it.
- `pub fn connect(host: &str, port: u16, policy: &NetPolicy, timeout: Duration) -> Result<TcpStream, NetError>` resolves the name, removes each refused address, and connects to the first permitted address. The relay client and the SFTP client use it.
- `pub(crate) struct GuardResolver` implements the `ureq` `Resolver` trait. It wraps the default resolver and removes each refused address. A name with no permitted address gives an error that Studio maps to `NetError::NotPublic`.
- `Web::with_policy(policy)`, the S3 agent and the Blossom agent build their agents with `GuardResolver` for `Public`. With `Public`, the agents use no proxy from the environment.
- `NetError::NotPublic { host }` names the host, not the resolved address. `WebErrorKind` and `ConnectorErrorKind` each get a `NotPublic` kind.
- `HostEnv` gets `policy: NetPolicy`, and each connector that connects passes it to its client.

### The API

- `Workspace` gets `net_policy: NetPolicy`, `Open` by default, and `with_net_policy(policy)`. `Workspace::open` keeps `Open`. The `web` field, `hosts_with` and the fingerprint route use the policy.
- The `NotPublic` kinds map to 422 `address_not_public`, with the message "Studio on a server cannot connect to <host>, because it is not a public address."

## Steps

1. Add `net.rs` with `is_global`, `connect` and `GuardResolver`, with unit tests for each range of ADR 0057 item 2.
2. Use the policy in `Web`, S3, Blossom, the relay client and SFTP.
3. Add the policy to `HostEnv` and to `Workspace`.
4. Add the error mapping.
5. Add the tests of the acceptance criteria.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)). A test of a public address uses a resolver test double, not DNS.

## Acceptance Criteria

Mechanical:

- Unit tests: `is_global` refuses one address of each range of ADR 0057 item 2, an IPv4-mapped loopback address and a `64:ff9b::` address of `10.0.0.1`. It permits `1.1.1.1` and `2606:4700::1111`.
- With `Public` and no exception, API tests get 422 `address_not_public` for each of these:
  - A discovery of a feed, an image and a media facts request on a local test server.
  - An S3 host on the fake S3 server, and an SFTP host on the sshd harness.
  - An nsite host with a relay on the test relay, and the fingerprint route.
- A local test server sends a redirect to `127.0.0.1`. With `Public`, the first address is an exception and the redirect target is not. The test gets the refusal.
- With `Open`, the existing tests pass with no change.
- A test of `connect` with a resolver test double that gives a loopback address and a public address connects only to the public address.
- The four commit checks pass.

Visual: none.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the `ureq` resolver interface cannot remove addresses.
- Stop and report when a redirect does not go through the resolver.
- Stop and report when a client above needs a change of the connector trait for the policy.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034b-private-network-guard.md
- docs/adr/0057-the-private-network-guard-of-the-server.md
- The files that the Files section names.

Goal:
- Add `net.rs` with the policies `Open` and `Public`, the address check, the guarded connection and the `ureq` resolver.
- Use the policy in each client that connects to an address that a user gives.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No test sends a request outside the computer.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The musicindex client and the hub client.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
