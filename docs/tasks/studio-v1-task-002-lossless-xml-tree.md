# Studio v1 Task 002: Lossless XML Tree

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 1.
Decision: [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md).

## Goal

Read a feed into a lossless XML tree and write the same bytes again.

## Files Likely To Change

- `crates/core/Cargo.toml`
- `crates/core/src/lib.rs`, `crates/core/src/xml.rs` and its submodules
- `crates/core/tests/`

## Constraints

- Use `quick-xml` for the tokens. Keep the source bytes of each tag.
- Resolve the namespace URI of each element and attribute from the `xmlns` declarations.
- Accept UTF-8 only.

## Acceptance Criteria

Mechanical:

- For each file in `crates/core/tests/corpus/`, a read and a write give the same bytes.
- A file that is not well-formed XML gives an error, not a panic.
- A test resolves the podcast namespace of an element with the prefix `podcast` and of an element with a different prefix.

Visual: none.

## Test Commands

```sh
cargo test -p studio-core
```
