# Studio v1 Task 204: Remove The Empty Folders After A Delete

Decision: [ADR 0229](../adr/0229-the-media-host-after-a-follow-a-podping-again-and-empty-folders.md) items 8 to 11.

Status: Done.

## Problem

Finding 3 of the [rc.29 human check](../reviews/2026-11-20-rc29-human-check.md). The delete of a leftover copy deleted the files and kept the empty folder `Songs/Albums/nathan-abbott-everything/` on the SFTP host. No connector removes a folder.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Connector delete | extend | crates/core/src/connector.rs | `Connector::delete` near line 399. Its doc comment gets the folder rule. |
| SFTP connector | extend | crates/core/src/connector/sftp.rs | `delete` near line 830 calls `wire.remove`. The wire is in `sftp_wire.rs`. |
| SFTP wire | extend | crates/core/src/connector/sftp_wire.rs | Add the SFTP request RMDIR if the wire does not have it. |
| Local connector | extend | crates/core/src/connector/local.rs | `delete` near line 286 calls `fs::remove_file`. `crates/core/src/library.rs:402` has a `fs::remove_dir` pattern. |
| Connector suite | extend | crates/core/src/connector/suite.rs | `run_all` near line 16 and the delete tests near line 234. |
| Test SFTP server | reuse | crates/core/src/testing/sshd.rs | The SFTP tests use it. |

## Changes

1. After `delete` of the local and the SFTP connector removes the file, the connector removes the folder of the file when it is empty. Then it does the same for each parent folder, up to the root of the host. It stops at the first folder that is not empty (items 8 and 9).
2. The connector never removes the root folder of the host (item 9).
3. A failure to remove a folder does not change the result of `delete`. Log it with the logger that the core uses (item 11).
4. S3, Blossom and nsite do not change (item 10).

## Checks

Mechanical:

- Suite test: write `a/b/one.mp3` and `a/two.mp3`, then delete `a/b/one.mp3`. `list` still gives `a/two.mp3`. Then delete `a/two.mp3`. `list` gives no file. The suite runs on each connector.
- Local test and SFTP test: after the two deletes, the folders `a/b` and `a` do not exist, and the root folder exists. After the first delete only, `a` exists.
- `bash tools/gate.sh` passes.

Visual: none. The human check deletes a test file on the real SFTP host.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

The local and the SFTP connector now remove each empty folder after `delete`, from the folder of the file up to the root. They stop at the first folder that has content. They never remove the root. A failure only goes to `eprintln!`, the logger of the core, and `delete` still succeeds. The SFTP wire has a new `rmdir` request (RMDIR). The doc comment of `Connector::delete` gives the rule. S3, Blossom and nsite are unchanged.

Tests added: the suite check `delete_of_the_last_file_in_a_folder_keeps_the_other_files` (in `run_all` and as one test each in the local and SFTP tests), and a local test and an SFTP test that check the folders `a/b`, `a` and the root.

Checks: `cargo test -p` of the core, filter `connector`, 231 passed. `bash tools/gate.sh` passed.
