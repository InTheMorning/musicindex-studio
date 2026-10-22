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

On 2026-10-22, Phase 0 is complete and Phase 1 is partly complete. `studio-core` reads a feed into a lossless XML tree and a typed model, and writes each edit back. The other crates contain no function yet.

- Decisions: `docs/adr/0001` to `docs/adr/0012`.
- Plan: [Studio v1 phase plan](docs/plans/studio-v1-phase-plan.md). Its current state lists the Phase 1 work that remains.
- Research: [baseline research note](docs/research/2026-10-22-studio-baseline-research.md).
- Open questions: the plan lists them. [ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md) has open questions about the self-hosted file host and file moves.

## Layout

| Path | Contents |
| --- | --- |
| `crates/core` | `studio-core`: feed model, connectors, vault and write pipeline. No HTTP and no UI code. |
| `crates/core/tests/corpus` | Published feeds for the round-trip tests. `manifest.json` gives the source URL, the fetch time and the SHA-256 of each file. |
| `crates/api` | `studio-api`: the JSON API over the core |
| `crates/server` | `studio-server`: the server shell |
| `crates/desktop` | `studio-desktop`: the desktop shell. Tauri comes in Phase 10. |
| `docs/` | ADRs, plans, tasks and research. The index is `docs/README.md`. |

## Commands

These three checks must pass before each commit. CI runs the same checks.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

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
- A read and a write with no edit give the same bytes. The writer changes only the elements that the user changed. The corpus tests enforce this ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md), [ADR 0012](docs/adr/0012-lossless-tree-with-a-typed-model.md)).
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
