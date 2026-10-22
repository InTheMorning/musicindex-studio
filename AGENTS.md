# musicindex-studio

## What This Project Is

musicindex-studio is a part of the musicindex.org project. It replaces MSP 2.0.
An artist uses it to edit hosted Podcasting 2.0 music feeds directly.
Studio keeps the full collection of feeds for each user, and the method that writes each feed.

Studio has one core and two shells ([ADR 0001](docs/adr/0001-one-core-with-desktop-and-server-shells.md)):

- A desktop shell runs on the artist's machine.
- A server shell lets a host operator run a public service for web users.

The license is AGPL-3.0 ([ADR 0011](docs/adr/0011-agpl-license.md)).

## Where The Work Stands

On 2026-10-22, the repository contains decisions and a plan. It contains no code.

- Decisions: `docs/adr/0001` to `docs/adr/0011`.
- Plan: [Studio v1 phase plan](docs/plans/studio-v1-phase-plan.md). Phase 0 is next.
- Research: [baseline research note](docs/research/2026-10-22-studio-baseline-research.md).
- Open questions: the plan lists them. [ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md) has open questions about the self-hosted file host and file moves.

## Design Philosophy

- Studio connects to hosts. It does not host files. The user can move files between hosts at any time ([ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- The user controls the secrets. Only the user can open the vault ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md)).
- Feed identity does not change. Studio keeps the feed GUID and each item GUID ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md)).
- No write occurs without a diff that the user has seen ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md)).
- Studio uses the musicindex.org "Afterglow" visual direction ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).

## Working Rules

Each rule below states its owner. This file makes no rule.

Durable rules:

- The core crate has no HTTP code and no UI code. Each feed rule is in the core ([ADR 0002](docs/adr/0002-rust-core-json-api-browser-ui.md)).
- A connector gets its secrets from the vault only. No secret is in a log, an API response or the collection database ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- A read and a write with no edit keep the meaning of the feed ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md)).
- An entity color is not a general accent. Each control is a minimum of 44 px high, and all functions work from the keyboard ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).

Situational rules:

- The v1 connectors are local folder, SFTP, S3 and R2, Git pages, Blossom and nsite, and the optional self-hosted file host ([ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- Do not copy code from `../mi-studio`. Use it as reference only ([ADR 0001](docs/adr/0001-one-core-with-desktop-and-server-shells.md)).

Documentation:

- Put no Markdown file in the repository root other than `AGENTS.md` and a future `README.md`. The global repository-docs rules own this.
- Add each new document to [docs/README.md](docs/README.md) in the same change.
- Write prose in ASD-STE100 Simplified Technical English. Check it with `~/.agents/skills/asd-ste100/scripts/ste_lint.py`. The operator's global instructions own this rule.
- A change of a decision changes the ADR, `docs/README.md`, the tests and this file in the same commit.

## Reference Material

- Research: `../musicindex-research`. It is private. Do not copy text that has a private notice.
- Visual direction: `../musicindex`, musicindex ADR 0003 and ADR 0012, and `search.html`.
- Feed API: Stophammer, through `../musicindex/api.json`.
- MSP 2.0: `../MSP-2.0`.
