# Studio v1 Task 001: Workspace

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 0.

## Goal

Make the Cargo workspace and the checks that each subsequent phase uses.

## Files To Inspect

- `docs/adr/0001-one-core-with-desktop-and-server-shells.md`
- `docs/adr/0002-rust-core-json-api-browser-ui.md`
- `docs/adr/0011-agpl-license.md`

## Files Likely To Change

- `Cargo.toml`, `Cargo.lock`, `rustfmt.toml`, `.gitignore`
- `crates/core/`, `crates/api/`, `crates/server/`, `crates/desktop/`
- `.github/workflows/ci.yml`
- `AGENTS.md`, `docs/README.md`, the phase plan

## Do Not Touch

- The ADRs. This task makes no decision.

## Constraints

- Rust edition 2024.
- Workspace members:
  - `studio-core`: a library crate. It has no dependency on the other members.
  - `studio-api`: a library crate. It can depend on `studio-core` only.
  - `studio-server`: a binary crate. It can depend on `studio-api` and `studio-core`.
  - `studio-desktop`: a binary crate. Tauri comes in Phase 10, not in this task.
- Add no external dependency in this task.
- The license field of each crate is `AGPL-3.0-only`.
- Workspace lints forbid `unsafe_code` and require documentation on each public item.

## Implementation Steps

1. Write the workspace `Cargo.toml` with the members, the shared package fields and the lints.
2. Write each crate with crate-level documentation.
3. Write the CI workflow with the three checks below.
4. Add the commands to `AGENTS.md`.

## Acceptance Criteria

Mechanical:

- `cargo fmt --all --check` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo test --workspace` passes.
- `cargo tree -p studio-core` shows no other workspace member.

Visual: none.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo tree -p studio-core
```

## Final Report

Give the result of each test command, the files created and each deviation from this task.

## Escalation Triggers

- A check fails because of a cause outside this repository, such as a toolchain defect.
- A step needs an external dependency.
