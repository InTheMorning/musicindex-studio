#!/usr/bin/env bash
#
# The commit gate of Studio. Each check must pass before each commit.
#
# Usage:
#   tools/gate.sh
#
# CI runs the first four checks. The fifth check runs only here, because its
# script is in the project-baseline skill and not in this repository.
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# ADR 0042: the UI models have Node tests. tools/node runs the pinned Node.js.
tools/node --test crates/api/ui-tests/*.test.mjs
# project-baseline "A Binding Rule Lives In A Decision Record": the link lines
# of each ADR, the limit of global CLAUDE.md §Consolidate A Changed Decision
# and the status rule of AGENTS.md. No check is skipped.
python3 "${HOME}/.agents/skills/project-baseline/scripts/check_adr_links.py" "${ROOT}"

echo "gate: each check passed"
