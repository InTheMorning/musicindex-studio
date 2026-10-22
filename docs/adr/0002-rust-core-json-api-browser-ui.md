# ADR 0002: Rust Core, JSON API And One Browser UI

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0001](0001-one-core-with-desktop-and-server-shells.md) requires one core in two shells.
The main work in Studio is interactive editing: track order, value splits, recipients and a diff before each write.
The other musicindex services, such as Stophammer, use Rust.

## Decision

1. The core is a Rust library crate. It contains the feed model, the RSS reader and writer, the connectors, the vault and the write pipeline. It contains no HTTP code and no UI code.
2. An axum service gives the core functions as a JSON API.
3. One browser UI uses that API.
4. The desktop shell is a Tauri app. It shows the same UI in a native window.
5. The server shell serves the same API and UI behind sign-in.
6. The UI build method (a framework and a build step, or no framework) is not decided. A subsequent ADR selects it after the core design exists.

## Alternatives Considered

- Server-rendered pages with Askama. Rejected: interactive editing needs a server request for each change, and the desktop shell must open an external browser.
- A TypeScript stack, as in MSP 2.0. Rejected: it is different from the other musicindex services.

## Consequences

- The project uses two languages: Rust and JavaScript or TypeScript.
- The JSON API is a contract between two layers. A change to it changes the UI in the same commit.
- Other tools can use the API.

## Invariants

- Each feed rule (identity, value blocks, round trip) is in the core, not in the UI.
- The UI does not write to a host directly. It asks the API to do the write.
