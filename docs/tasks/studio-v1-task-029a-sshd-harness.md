# Studio v1 Task 029a: The sshd Test Harness

Parent: [task 029](studio-v1-task-029-sftp-connector.md). Decisions: [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) items 1 and 3, [ADR 0047](../adr/0047-the-sftp-connector.md).
Sequence: the first packet of task 029.

## Goal

A test can start the OpenSSH `sshd` of the computer as a normal user, on a free loopback port. It has a temporary configuration, a temporary host key and a temporary user key. `studio-core` depends on `ssh2`, and a test logs in to the server with it. CI installs `openssh-server`.

## Files

Inspect: `crates/core/src/testing.rs`, `crates/core/src/testing/fake_s3.rs` (the form of a test server), `crates/core/Cargo.toml`, the root `Cargo.toml`, `.github/workflows/ci.yml`.

Change: add `crates/core/src/testing/sshd.rs`. Change `testing.rs` (the module line), `crates/core/Cargo.toml`, the root `Cargo.toml`, `Cargo.lock` and `.github/workflows/ci.yml`.

Do not change: the connectors, the pipeline, the API, the UI.

## Architecture

These choices are fixed. Do not change them.

- Add `ssh2` 0.9 to the workspace dependencies and to `studio-core`, with the feature `vendored-openssl`.
- `pub struct TestSshd` starts with `TestSshd::start() -> TestSshd`. It does these steps in a new temporary folder:
  1. It finds `sshd`: the first of `/usr/sbin/sshd`, `/usr/bin/sshd` and `/usr/local/sbin/sshd` that exists. When none exists, it panics with the message "the SFTP tests need the OpenSSH sshd program. Install openssh-server." ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 3).
  2. It makes an Ed25519 host key and an Ed25519 user key with `ssh-keygen -t ed25519 -N "" -q -f <file>`. It makes a second user key with the passphrase `test passphrase`, for the test of an encrypted key.
  3. It writes `authorized_keys` with the two public user keys.
  4. It selects a free loopback port: it binds `127.0.0.1:0`, reads the port and closes the listener.
  5. It writes `sshd_config` with these lines: `ListenAddress 127.0.0.1`, `Port <port>`, `HostKey <host key>`, `AuthorizedKeysFile <authorized_keys>`, `PasswordAuthentication no`, `KbdInteractiveAuthentication no`, `PubkeyAuthentication yes`, `UsePAM no`, `StrictModes no`, `PidFile none`, `Subsystem sftp internal-sftp` and `LogLevel ERROR`.
  6. It starts `<sshd> -D -e -f <sshd_config>` as a child process, with its error output in a file of the folder.
  7. It waits until a TCP connection to the port works, for a maximum of 10 seconds. After that time, it panics with the text of the error output file.
- `TestSshd` gives `port()`, `user()` (the name of the user that runs the test, from the `USER` variable or else from `whoami`), `key_path()`, `encrypted_key_path()`, `root()` (a new empty folder in the temporary folder for the files of a test) and `host_key_fingerprint()`. The fingerprint is the OpenSSH form `SHA256:<base64 with no padding>` of the host key, from `ssh-keygen -l -E sha256 -f <host key>.pub`.
- `TestSshd::with_host_key_change(&mut self)` replaces the host key with a new key and starts `sshd` again on the same port, for the test of a changed host key.
- When `TestSshd` is dropped, it stops the child process and removes the folder.
- CI gets a step before the Rust checks: `sudo apt-get update && sudo apt-get install -y openssh-server`.

## Steps

1. Add `ssh2` with `vendored-openssl`, and check that the workspace builds.
2. Add `TestSshd`.
3. Add tests that start `TestSshd` and connect with `ssh2::Session`:
   - Read the host key fingerprint with `host_key_hash(HashType::Sha256)`, and compare it with `host_key_fingerprint()`.
   - Log in with `userauth_pubkey_file`, with the plain key, and with the encrypted key and its passphrase.
4. Add a test of `with_host_key_change`: the fingerprint is different after the change.
5. Add the CI step.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- One new crate only: `ssh2` (with its own dependencies).
- No test sends a request to a host outside the computer.
- Two tests can run at the same time, each with its own `TestSshd`.

## Acceptance Criteria

Mechanical:

- A test logs in to `TestSshd` with the plain user key, and a second test logs in with the encrypted key and its passphrase.
- The fingerprint from `ssh2` is `host_key_fingerprint()`.
- After `with_host_key_change`, the fingerprint is different, and a connection still works.
- After a `TestSshd` is dropped, no `sshd` child process of the test remains.
- CI installs `openssh-server` before `cargo test`.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report in these cases:

- `ssh2` with `vendored-openssl` does not build with the pinned Rust.
- `sshd` does not start as a normal user with this configuration.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-029a-sshd-harness.md
- docs/adr/0047-the-sftp-connector.md
- The files that the Files section names.

Goal:
- crates/core/src/testing/sshd.rs has TestSshd, which starts the system sshd as a normal user on a free loopback port with temporary keys. studio-core depends on ssh2, and tests log in with it. CI installs openssh-server.

Constraints:
- Follow the Architecture section of the task file fully.
- One new crate only: ssh2.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connectors, the pipeline, the API, the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
