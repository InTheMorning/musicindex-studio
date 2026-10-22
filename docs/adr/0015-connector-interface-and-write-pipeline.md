# ADR 0015: The Connector Interface And The Write Pipeline

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0006](0006-connectors-and-the-v1-set.md) gives each connector three functions: read with a version marker, write only if the marker did not change, and give the public URL.
[ADR 0007](0007-draft-diff-write-and-announce.md) requires a draft, a diff that the user has seen, a version check and a snapshot before each write.
The core is a library for two shells. The server uses an async runtime. The desktop shell uses Tauri.

## Decision

### Connector Interface

1. A connector is a Rust trait with blocking functions. The server calls it from a blocking task. The core does not depend on an async runtime.
2. A version marker is an opaque string. Each connector selects its form: a content hash, an ETag or a commit ID.
3. A write gives the expected version marker, or "no file" for a new file. The connector refuses the write with a conflict when the file on the host has a different version.
4. The local folder connector uses the SHA-256 of the file content as its version marker. It writes a temporary file and then renames it.

### Write Pipeline

5. A draft keeps the version and the bytes of the live file that it started from, and the bytes of the draft.
6. A diff is a line diff between the live file and the draft. Each diff has an ID: a hash of the base version and the draft bytes.
7. A write takes the ID of the diff that the user approved. The write stops when the ID is not the ID of the current diff. A changed draft thus needs a new approval.
8. Before a write, the pipeline reads the live file again. The write stops with a conflict when its version is not the base version of the draft.
9. Before a write, the pipeline keeps a snapshot of the live file.
10. A rollback makes a draft from a snapshot. The user approves its diff like each other draft.

## Alternatives Considered

- An async connector trait. Rejected for now: a trait object with async functions needs boxed futures, and the SFTP libraries are blocking. A subsequent ADR can change this if a connector needs async.
- A version marker from the modification time of a file. Rejected: a copy or a restore can keep the time and change the content.

## Consequences

- The local folder connector cannot stop a change that occurs between its version check and its rename. The window is short. A remote connector with a conditional write, such as an S3 `If-Match`, has no such window.
- The pipeline makes no announcement yet. Phase 7 adds the podping and the Podcast Index notification after a write.

## Invariants

- No write occurs without the ID of the current diff.
- No write occurs when the live file changed after the draft started.
- Each write keeps a snapshot of the file that it replaces.
