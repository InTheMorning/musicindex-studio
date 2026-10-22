# ADR 0001: One Core With A Desktop Shell And A Server Shell

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

musicindex-studio replaces MSP 2.0.
An artist uses it to edit hosted Podcasting 2.0 music feeds directly.
The mission of the project is to encourage self-hosting and to give artists the power to do it.

MSP 2.0 keeps one draft in the browser and writes only to its own host, Blossom and nsite.
It keeps no collection of feeds.
The research is in [the baseline research note](../research/2026-10-22-studio-baseline-research.md).

An earlier project, `../mi-studio`, had a related purpose.
Its design forbids stored credentials and hosting automation.

## Decision

1. Studio has one core and two shells.
   - The desktop shell runs on the artist's machine. It needs no hosting service.
   - The server shell lets a host operator run a public service for web users. That service keeps secrets for its users.
2. The two shells use the same core, the same API and the same UI.
3. Studio keeps the full collection of feeds for each user, and the method that writes each feed.
4. This repository starts new. It copies no code from `../mi-studio`. That project is reference material only.

## Alternatives Considered

- A hosted service only. Rejected: it does not support the self-hosting mission.
- A desktop app only. Rejected: some artists cannot run a desktop app.
- Continue `../mi-studio`. Rejected by the operator.

## Consequences

- Each core function must work without a network service from the musicindex project.
- Each server-only function, such as sign-in, stays in the server shell.

## Invariants

- The core has no dependency on a shell.
- A function that the desktop shell needs does not need a public server.

## Follow-Up Work

- [ADR 0002](0002-rust-core-json-api-browser-ui.md) gives the stack.
