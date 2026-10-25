# Studio v1 Task 029b: The SFTP Requests

Parent: [task 029](studio-v1-task-029-sftp-connector.md). Decisions: [ADR 0047](../adr/0047-the-sftp-connector.md) items 5, 6 and 8.
Sequence: the second packet of task 029. It depends on 029a.

## Goal

`studio-core` sends the SFTP version 3 requests that the connector needs over an `ssh2` channel, with the OpenSSH extensions `posix-rename@openssh.com`, `fsync@openssh.com` and `hardlink@openssh.com`. A connector of 029c uses this session.

## Files

Inspect: `crates/core/src/testing/sshd.rs`, `crates/core/src/connector.rs` (`ConnectorError`, `ConnectorErrorKind`), the `ssh2` documentation for `Session::channel_session` and `Channel::subsystem`.

Change: add `crates/core/src/connector/sftp_wire.rs`. Change `connector.rs` (the module line only).

Do not change: the other connectors, the pipeline, the API, the UI.

## Architecture

These choices are fixed. Do not change them.

- The source is draft-ietf-secsh-filexfer-02 (SFTP version 3) and the OpenSSH file `PROTOCOL`, section 4. Each packet is a `u32` length and a `u8` type, and each request has a `u32` ID.
- `pub(crate) struct SftpWire` has `SftpWire::open(session: &ssh2::Session) -> Result<SftpWire, ConnectorError>`. It opens a session channel, starts the subsystem `sftp`, sends `SSH_FXP_INIT` with version 3, and reads `SSH_FXP_VERSION` with its extension pairs.
- `extensions()` gives the names and versions that the server gave. `has(name)` tells if the server gave a name.
- The functions are these. Each one sends one request and reads its response. A response with a different ID is a protocol error.

  | Function | Request | Result |
  | --- | --- | --- |
  | `stat(path)` | `SSH_FXP_LSTAT` | `Option<Attrs>`: `None` for `SSH_FX_NO_SUCH_FILE` |
  | `open_read(path)` | `SSH_FXP_OPEN` with `READ` | a handle |
  | `open_create(path)` | `SSH_FXP_OPEN` with `WRITE`, `CREAT` and `EXCL` | a handle. An existing file gives `Conflict`. |
  | `read(handle, offset, len)` | `SSH_FXP_READ` | the bytes, or an empty result at the end of the file |
  | `write(handle, offset, bytes)` | `SSH_FXP_WRITE` | nothing |
  | `close(handle)` | `SSH_FXP_CLOSE` | nothing |
  | `read_dir(path)` | `SSH_FXP_OPENDIR`, then `SSH_FXP_READDIR` until `SSH_FX_EOF`, then `SSH_FXP_CLOSE` | the name and the `Attrs` of each entry, with no `.` and no `..` |
  | `remove(path)` | `SSH_FXP_REMOVE` | nothing |
  | `rename(from, to)` | `SSH_FXP_RENAME` | nothing. An existing target gives `Conflict`. |
  | `mkdir(path)` | `SSH_FXP_MKDIR` | nothing |
  | `posix_rename(from, to)` | `SSH_FXP_EXTENDED` `posix-rename@openssh.com` | nothing. It replaces the target. |
  | `hardlink(from, to)` | `SSH_FXP_EXTENDED` `hardlink@openssh.com` | nothing. An existing target gives `Conflict`. |
  | `fsync(handle)` | `SSH_FXP_EXTENDED` `fsync@openssh.com` | nothing |

- `Attrs` has `size: Option<u64>`, `mtime: Option<u32>` and a kind: file, folder, symbolic link or other, from the permission bits.
- A write sends parts of 32 KiB. A read asks for parts of 32 KiB.
- The status mapping: `SSH_FX_NO_SUCH_FILE` gives `NotFound`, except where the table says differently. `SSH_FX_PERMISSION_DENIED` gives `Unauthorized`. `SSH_FX_FAILURE` on an exclusive create, a rename or a hardlink to an existing target gives `Conflict`. Each other status gives `Io` with the status code and the message of the server. A channel error gives `Unavailable`.
- An extension function on a server that did not give the extension gives `Unsupported`, with no request.

## Steps

1. Add the packet encoding and decoding, with unit tests on bytes that need no server.
2. Add `open` and the functions of the table.
3. Add tests against `TestSshd` for each function, in a folder below `TestSshd::root()`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- No `unsafe` code.

## Acceptance Criteria

Mechanical:

- A unit test decodes a `SSH_FXP_VERSION` packet with two extension pairs, and a `SSH_FXP_NAME` packet with two entries.
- Against `TestSshd`, `extensions()` contains `posix-rename@openssh.com`, `fsync@openssh.com` and `hardlink@openssh.com`.
- A file of 100 KiB written with `open_create`, `write`, `fsync` and `close` reads back with the same bytes.
- `open_create` on an existing file gives `Conflict`. `hardlink` to an existing target gives `Conflict`. `rename` to an existing target gives `Conflict`. `posix_rename` to an existing target replaces it.
- `stat` of a missing path gives `None`. `stat` of a symbolic link gives the kind symbolic link.
- `read_dir` gives the entries of a folder with no `.` and no `..`.
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

- `ssh2` cannot open the `sftp` subsystem on a channel.
- The server status for an existing target is not `SSH_FX_FAILURE`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-029b-sftp-requests.md
- docs/adr/0047-the-sftp-connector.md
- The files that the Files section names.

Goal:
- crates/core/src/connector/sftp_wire.rs has SftpWire. It sends the SFTP version 3 requests of the task table over an ssh2 channel, with the OpenSSH extensions posix-rename, fsync and hardlink.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No unsafe code.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The other connectors, the pipeline, the API, the UI.
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
