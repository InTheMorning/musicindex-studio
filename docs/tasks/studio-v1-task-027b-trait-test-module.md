# Studio v1 Task 027b: The Test Module Of The Trait

Parent: [task 027](studio-v1-task-027-connector-interface.md). Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) item 2, [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md) items 2 to 4 and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 6 and 16.
Sequence: the second packet of task 027. It depends on 027a.

## Goal

`studio-core` has one set of checks that examines a connector through the `Connector` trait only. `LocalFolder` passes each check. Each Phase 9 connector then uses the same checks against its local test server.

## Files

Inspect: `crates/core/src/connector.rs`, `crates/core/src/connector/local.rs` (its tests), `crates/core/src/testing.rs`, `crates/core/src/lib.rs`, `crates/core/Cargo.toml`.

Change: add `crates/core/src/connector/suite.rs`. Change `connector.rs` (the module line) and the tests of `connector/local.rs`.

Do not change: the trait, the `LocalFolder` code other than its tests, the pipeline, the API, the UI.

## Architecture

These choices are fixed. Do not change them.

- `connector.rs` has `#[cfg(any(test, feature = "testing"))] pub mod suite;`. The module is thus available to the tests of `studio-core` and to the crates that enable the `testing` feature.
- Each check is a public function with this signature:

  ```rust
  pub fn check_name(make: &dyn Fn() -> Box<dyn Connector>);
  ```

  `make` gives a connector on an empty host. A check calls `make` one time or more, and each call gives a different empty host. A check panics with a message that names the trait function and the expected result.
- `suite.rs` also has `pub fn run_all(make: &dyn Fn() -> Box<dyn Connector>)`, which calls each check.
- The checks are these. Use these names:

  | Check | What it examines |
  | --- | --- |
  | `read_of_an_absent_file_gives_none` | `read` of a path with no file gives `Ok(None)`. |
  | `write_then_read_gives_the_bytes_and_the_version` | `write` with `Expected::Absent` gives a version. `read` then gives the same bytes and the same version. |
  | `write_over_a_file_with_absent_is_a_conflict` | `write` with `Expected::Absent` on a path with a file gives `Conflict`. The file does not change. |
  | `write_with_an_old_version_is_a_conflict` | `write` with the version before the last write gives `Conflict`. The file does not change. |
  | `write_with_the_current_version_replaces_the_file` | `write` with the current version gives a different version, and `read` gives the new bytes. |
  | `write_from_gives_the_same_file_as_write` | `write_from` and `write` of the same bytes on two paths give the same bytes on `read`. |
  | `write_from_with_a_short_reader_writes_nothing` | `write_from` with a reader that gives fewer bytes than `length` gives `Io`, and `read` of the path gives `None`. |
  | `stat_gives_the_size_and_a_new_stamp_after_a_change` | `stat` gives the size of the file. After a write of different bytes, the stamp is different. `stat` of an absent path gives `None`. |
  | `list_gives_each_file_in_path_sequence` | After writes to `b.txt`, `a/c.txt` and `a.txt`, `list` gives the three paths in sorted sequence, each with its size, and `truncated` false. |
  | `delete_with_the_current_stamp_removes_the_file` | `delete` with the stamp from `stat` removes the file. |
  | `delete_with_an_old_stamp_is_a_conflict` | `delete` with a stamp from before a change gives `Conflict`, and the file stays. |
  | `delete_of_an_absent_file_is_not_found` | `delete` of a path with no file gives `NotFound`. |
  | `a_path_out_of_the_host_is_invalid` | `read`, `write`, `stat` and `delete` of `../x` and of `/x` each give `InvalidPath`. |
  | `sha256_when_given_is_the_sha256_of_the_bytes` | When `sha256` gives a value, it is the lowercase hexadecimal SHA-256 of the written bytes. |
  | `set_redirect_without_the_ability_is_unsupported` | When `can_redirect` gives false, `set_redirect` gives `Unsupported`. |

- The tests of `connector/local.rs` have one `#[test]` for each check. Each test calls the check with a `LocalFolder` on a new temporary folder and no base address.
- The current tests of `LocalFolder` stay.

## Steps

1. Add `suite.rs` with the checks and `run_all`.
2. Add one test for each check to the tests of `connector/local.rs`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. `suite.rs` uses `tempfile` only in the tests of `local.rs`, not in the suite.
- A check uses only the functions of the `Connector` trait. It does not read the file system.

## Acceptance Criteria

Mechanical:

- `suite.rs` has each check of the table, and `run_all` calls each one.
- `LocalFolder` passes each check, with one test for each check.
- A check uses no type of `connector/local.rs`. The review checklist names this manual check.
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

Stop and report when `LocalFolder` fails a check. Do not change the check or `LocalFolder` to make it pass.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-027b-trait-test-module.md
- docs/adr/0043-phase-9-connectors-and-the-interface-change.md
- The files that the Files section names.

Goal:
- studio-core gets crates/core/src/connector/suite.rs, a set of checks that examines a connector through the Connector trait only. LocalFolder passes each check.

Constraints:
- Follow the Architecture section of the task file fully. Use the check names of its table.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The trait, the LocalFolder code other than its tests, the pipeline, the API, the UI.
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
