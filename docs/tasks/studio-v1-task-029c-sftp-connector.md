# Studio v1 Task 029c: The SFTP Connector

Parent: [task 029](studio-v1-task-029-sftp-connector.md). Decisions: [ADR 0047](../adr/0047-the-sftp-connector.md) items 1, 2, 4 and 7 to 12, and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md).
Sequence: the third packet of task 029. It depends on 029b.

## Goal

`studio-core` has the connector kind `sftp`. It logs in with a private key, a password or ssh-agent, and only to the host key that its row names. It reads, writes, lists and deletes the files below its root folder with the sequence of ADR 0047. It passes the test module of the trait against `TestSshd`.

## Files

Inspect: `crates/core/src/connector.rs` (the trait, `from_row`, `secret_of`), `crates/core/src/connector/s3.rs` (the form of a remote connector), `crates/core/src/connector/sftp_wire.rs`, `crates/core/src/connector/suite.rs`, `crates/core/src/testing/sshd.rs`, `crates/core/src/digest.rs`.

Change: add `crates/core/src/connector/sftp.rs`. Change `connector.rs` (the module and `from_row`).

Do not change: the pipeline, the API, the UI, the other connectors, the suite.

## Architecture

These choices are fixed. Do not change them.

### The Host

- `pub struct SftpHost` has `pub const KIND: &'static str = "sftp"`. Its configuration JSON has `host`, `port`, `user`, `root`, `base_url` (optional), `login` (`"key"`, `"password"` or `"agent"`) and `host_key` (the fingerprint `SHA256:…`).
- The secret is JSON. For `key`: `{"private_key": "<OpenSSH private key text>", "passphrase": "…"}`, with `passphrase` optional. For `password`: `{"password": "…"}`. For `agent`: no secret.
- A row of kind `sftp` with the login `key` or `password` and no secret gives `InvalidConfig`. The `Debug` form shows no secret.
- `SftpHost` connects when it needs the server, and keeps one session while it is in use. A connection does these steps:
  1. A TCP connection to the host and port, with a timeout of 20 seconds. A failure gives `Unavailable`.
  2. The SSH handshake.
  3. The fingerprint of the host key, in the form `SHA256:<base64 with no padding>`, must be `host_key`. A different key gives `Unauthorized`, with the message "the host key changed: <new fingerprint>". Studio then does not log in.
  4. The login: `userauth_pubkey_memory` with the key and the passphrase, `userauth_password`, or the agent with each of its identities. A refused login gives `Unauthorized`.
  5. `SftpWire::open`. A server with no `posix-rename@openssh.com` gives `NoConditions` (packet 028c), with the message "the server has no posix-rename, so Studio cannot write to it safely".
- `SftpHost::fingerprint_of(host, port) -> Result<String, ConnectorError>` does steps 1 and 2, and gives the fingerprint with no login. Packet 029d uses it.

### The Operations

- A path is below `root`. The check of `relative_path` applies. Each operation does `stat` on each segment of the path below `root`, and a symbolic link gives `InvalidPath`.
- `read`: reads the file, and gives the SHA-256 version (`digest::content_version`).
- `stat`: the size and the stamp `<size>-<mtime>`. A symbolic link or a folder gives `None`.
- `list`: `read_dir` from `root`, folder after folder, with the limits of `LocalFolder::list`. It leaves out names that start with `.`, and symbolic links.
- `write` and `write_from`, with the sequence of ADR 0047 item 8:
  1. For `Expected::Version`, read the current file and calculate its SHA-256. A different version or no file gives `Conflict`. For `Expected::Absent`, `stat` of the path must give no file, or the write gives `Conflict`.
  2. Make each missing folder of the path with `mkdir`.
  3. Write the bytes to `.<name>.<uuid v4>.studio-tmp` in the same folder, with `open_create`, with a SHA-256 of the bytes. `write_from` reads the reader in parts of 32 KiB, and a reader with a different length gives `Io`.
  4. `fsync`, when the server has it. Then `close`.
  5. For `Expected::Absent`: `hardlink` to the target, then `remove` of the temporary file. Without `hardlink`: `rename`. A `Conflict` there removes the temporary file and gives `Conflict`. For `Expected::Version`: `posix_rename` to the target.
  6. When a step after step 3 fails, remove the temporary file, and give the error.
- `delete`: `stat`, then a compare of the stamp, then `remove`.
- `public_url`: `base_url` and the path, as `LocalFolder` does.
- `sha256` and `can_redirect` use the trait defaults.
- `from_row` makes an `SftpHost` for the kind `sftp`.

## Steps

1. Add the configuration, the secret and the connection, with tests of the host key check and the login against `TestSshd`: the plain key, the encrypted key, a wrong key and a changed host key.
2. Add the operations, with tests against `TestSshd`.
3. Add one test for each check of the suite against `TestSshd`, as `connector/local.rs` does. Each test uses its own folder below `TestSshd::root()` as `root`.
4. Add a test of `fingerprint_of`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- `write_from` reads the reader in parts. It does not read the full file into memory.
- No test sends a request to a host outside the computer.
- The password login and the agent login have no test against `TestSshd`. An `sshd` that runs as a normal user cannot check a password. A test cannot change the agent of the process safely. Each login is thus a human test of task 029.

## Acceptance Criteria

Mechanical:

- `SftpHost` passes each check of the suite against `TestSshd`.
- A login with the plain key and with the encrypted key works. A wrong key gives `Unauthorized`.
- After `TestSshd::with_host_key_change`, a connection gives `Unauthorized` with the new fingerprint in the message, and no file changes.
- A write leaves no `.studio-tmp` file after a success and after a `Conflict`.
- A write through a symbolic link gives `InvalidPath`, and the list leaves out symbolic links and hidden names.
- `fingerprint_of` gives `TestSshd::host_key_fingerprint()`.
- The `Debug` text of an `SftpHost` contains no secret.
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

Stop and report when `SftpHost` fails a check of the suite. Do not change the suite.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-029c-sftp-connector.md
- docs/adr/0047-the-sftp-connector.md
- The files that the Files section names.

Goal:
- studio-core gets the connector kind sftp in crates/core/src/connector/sftp.rs. It logs in only to the host key of its row, and it writes with the temporary file sequence of ADR 0047. It passes the trait suite against TestSshd.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The pipeline, the API, the UI, the other connectors, the suite.
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
