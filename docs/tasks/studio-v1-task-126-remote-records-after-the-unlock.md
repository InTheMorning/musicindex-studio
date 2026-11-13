# Studio v1 Task 126: The Remote File Records Wait For The Unlock

Decision: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 3.

Status: Complete on 2026-11-12. The mechanical checks passed. The API tests prove the wait and its single clear. They do not prove the rows of a real host, because the test has no feed. The human check passed on 2026-11-12.

## Goal

At the start, Studio records the remote files of each feed with a release folder (ADR 0038 item 3). When the vault is locked at the start, that pass now fails for each feed with a host that needs the vault, and the log gives one line for each feed: "the remote file records of feed N failed: ... vault is locked". Nothing runs the pass again after the unlock. The records thus stay old until a read or a write of each feed.

After this task, a locked vault does not cause a failure line. The pass waits, and it runs one time after the next unlock.

## Work

- `crates/api/src/workspace.rs`, `record_all_remote_files`: when the vault state is `Locked`, do not run the pass. Set a flag on the workspace that the pass waits for the unlock, and write one log line: "studio: the remote file records wait for the vault unlock". With no vault file, or with an open vault, the pass runs as now.
- After each unlock (passphrase, key, recovery key and passkey), when the flag is set, clear it and run `record_all_remote_files` in a background task. The task must not hold the unlock response: the pass can read the file hosts over the network. Use the pattern of `Workspace::run` or `blocking` in `crates/api/src/routes.rs`.
- The callers in `crates/desktop/src/lib.rs`, `crates/server/src/users.rs` and `routes.rs` do not change.

## Acceptance Criteria

Mechanical:

- An API test: with a locked vault, `record_all_remote_files` sets the flag and adds no remote row.
- An API test: after the unlock, the pass runs one time and clears the flag. A second unlock does not run it again.
- An API test: with an open vault, the pass runs at once, and it sets no flag.
- Each new assertion message names ADR 0038 item 3 and this task.
- The four checks pass.

Human, by the operator. This gate passed on 2026-11-12:

- Start the release build with a locked vault. The log has no line "remote file records of feed ... failed ... vault is locked". After the unlock, the Files page of a feed with a remote host shows its remote files.

## Stop And Ask When

- A host with no secret needs the pass before the unlock.
- The unlock response must wait for the pass.
