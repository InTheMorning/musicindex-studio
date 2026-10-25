# ADR 0042: UI Models With Node Tests

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

The UI has no build step ([ADR 0016](0016-ui-without-a-build-step.md)). Its tests are Rust tests that read the UI files as text, for example the test of the `details` elements ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). A text test cannot examine the result of UI logic.

[ADR 0039](0039-the-shell-releases-and-the-outline.md) asks for mechanical criteria at the owning layer. These are the destinations of the left panel, the sequence of the outline, and the marked row after "Releases". musicindex ADR 0027 asks for the same type of criteria for the back control. Each one is UI logic.

On 2026-10-25 the operator selected a plain JavaScript model with the test runner of Node.js. The operator then asked for no system installation of Node.js or Bun. `tools/node` gets a pinned Node.js into a folder that Git ignores.

## Decision

1. UI logic that a test must examine is in a model module in `crates/api/ui/`. A model module imports no Preact. It reads no DOM, no `window`, no `history`, no storage and no network. It gives values from its arguments only. A component calls it.
2. The tests are files `crates/api/ui-tests/*.test.mjs`. They use only `node:test` and `node:assert/strict`. The repository has no `package.json` and no JavaScript package.
3. The commit check gets a fourth command: `node --test crates/api/ui-tests/*.test.mjs`. On a computer with no Node.js, `tools/node` runs the same command with a pinned Node.js in `.studio-dev/tools/`. CI runs the Node.js command.
4. Studio does not serve a test file. Each model module is a UI file, so `ui.rs` serves it ([ADR 0016](0016-ui-without-a-build-step.md)).

## Alternatives Considered

- Keep only the Rust text tests. Not chosen by the operator: the sequence of the outline and the marked row are then visual checks only.
- A test framework from a package, such as Vitest. Rejected: it needs `package.json`, packages and a lock file. The UI then has a dependency that ADR 0016 does not let the UI have.
- A browser test with `tools/drive-ui.mjs` in CI. Rejected: it needs Chromium in CI, and a check of the model does not use a browser.

## Consequences

- CI installs Node.js.
- `tools/node` downloads the Node.js archive for Linux x64 at its first run, and refuses an archive with a different SHA-256. Another platform needs Node.js 22 on the PATH.
- A component that holds logic for a test moves the logic into a model module.
- The rule of ADR 0016 does not change: the browser loads the same files as before, with no build step.

## Non-Goals

- A test of a component with a DOM.
- A test of styles. The Rust text tests and the visual checks stay for them.

## Invariants

- A model module imports no Preact and reads no data from the browser.
- The commit check runs each test file of `crates/api/ui-tests/`.
