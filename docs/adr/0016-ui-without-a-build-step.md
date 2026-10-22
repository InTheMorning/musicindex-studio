# ADR 0016: The UI Has No Build Step

- Status: Accepted
- Date: 2026-10-22
- Owner: operator. The operator gave this choice to the agent on 2026-10-22.

## Context

[ADR 0002](0002-rust-core-json-api-browser-ui.md) gives one browser UI for the desktop shell and the server shell, and leaves its build method open.
The public site `musicindex.org` has no framework and no build step.
The editor is interactive: track lists, value splits, recipients and diffs. Plain DOM code for this is long and makes defects easy.

## Decision

1. The UI is plain HTML, CSS and JavaScript modules. It has no build step, no package manager and no `node_modules`.
2. The UI uses one vendored file for its components: `htm/preact/standalone.module.js` from htm 3.1.1. It contains Preact 10, the Preact hooks and htm.
   - The file is in `crates/api/ui/vendor/`, with the license texts of Preact (MIT) and htm (Apache-2.0).
   - The npm integrity value of the htm 3.1.1 package is `sha512-983Vyg8NwUE7JkZ6NmOqpCZ+sh1bKv2iYTlUkzlWmA5JD2acKoxd4KVxbMmxX/85mtfdnDmTFoNKcg5DGAvxNQ==`.
3. The UI files are in the Rust binary. The API crate includes them at compile time and serves them. The desktop app and the server need no separate UI files.
4. The UI loads nothing from a CDN or a third-party server.

## Alternatives Considered

- No library at all, as on `musicindex.org`. Rejected: the editor has much more state than the search site.
- A framework with a build step, such as Svelte or Solid with Vite. Rejected: it adds Node.js to each build and many dependencies to examine.

## Consequences

- A change of the vendored file is a change of this ADR, with the new version and integrity value.
- The UI uses htm templates. It does not use JSX.

## Invariants

- The UI makes no request to a server other than its own API.
