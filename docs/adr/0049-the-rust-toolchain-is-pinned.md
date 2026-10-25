# ADR 0049: The Rust Toolchain Is Pinned

- Status: Accepted
- Date: 2026-10-25
- Owner: operator

## Context

CI installed the newest stable Rust at each run, and the development computer used its own stable toolchain. On 2026-10-25, CI failed on commit `c05cbed`. The new stable Rust 1.99.1 added two Clippy lints. The code passed Clippy on Rust 1.93.1 on the development computer. During the correction, the development computer also changed to Rust 1.99.1.

On 2026-10-25 the operator selected a pinned toolchain.

## Decision

1. `rust-toolchain.toml` at the repository root pins Rust 1.99.1, with the components `rustfmt` and `clippy`.
2. CI installs the same version as `rust-toolchain.toml`.
3. A change of the version is a commit that changes `rust-toolchain.toml` and CI together, and that passes the four commit checks.

## Alternatives Considered

- The newest stable Rust, with no pin. Not chosen by the operator: a new Rust release can break CI with no change of the code.
- Rust 1.93.1. Not chosen by the operator.

## Consequences

- A new Clippy lint has an effect only when a commit changes the version.
- A crate that needs a newer Rust than the pin needs a change of the pin first.

## Invariants

- CI and `rust-toolchain.toml` give the same Rust version.
