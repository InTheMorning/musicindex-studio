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

On 2026-10-24, Phase 0 to Phase 7 are complete. `studio-core` reads a feed into a lossless XML tree and a typed model, and writes each edit back. It has the encrypted secret vault, the SQLite collection, the release library, the local folder connector and the write pipeline. `studio-api` serves the local JSON API and the UI, with the album editor, a new release and the import of music files. A lossless import gives an MP3 enclosure and a FLAC alternative enclosure. `studio-desktop` runs them on a loopback address, and `studio-server` has no function yet.

- Decisions: `docs/adr/0001` to `docs/adr/0040`. ADR 0017 is proposed.
- Plan: [Studio v1 phase plan](docs/plans/studio-v1-phase-plan.md). Phase 5 (album editor), Phase 6 (new releases) and Phase 7 (the release library of [ADR 0035](docs/adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](docs/adr/0036-library-records-host-inventory-and-host-operations.md) and [ADR 0038](docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)) are complete, with their visual checks. Phase 8 (publisher feeds) is next. The plan and the visual checks of milestone 7a, task 022 and milestone 7b list the open items.
- Research: [baseline research note](docs/research/2026-10-22-studio-baseline-research.md).
- Open questions: the plan lists them. [ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md) has open questions about the self-hosted file host and file moves.

## Layout

| Path | Contents |
| --- | --- |
| `crates/core` | `studio-core`. The feed model is in `feed`, `xml` and `guid`. The other modules are `vault`, `collection`, `address` (feed addresses), `connector`, `media`, `library`, `inventory`, `migrate` and `pipeline` (the write). No HTTP server code and no UI code. |
| `crates/core/tests/corpus` | Published feeds for the round-trip tests. `manifest.json` gives the source URL, the fetch time and the SHA-256 of each file. |
| `crates/api` | `studio-api`: the JSON API over the core, and the UI files in `crates/api/ui/` |
| `crates/server` | `studio-server`: the server shell |
| `crates/desktop` | `studio-desktop`: the desktop shell. It runs the local API and prints the UI address. Tauri comes in the Desktop Shell phase. |
| `docs/` | ADRs, plans, tasks and research. The index is `docs/README.md`. |
| `tools/drive-ui.mjs` | Drives the UI in headless Chromium for a check. Its header gives the usage. |

## Commands

These three checks must pass before each commit. CI runs the same checks.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run Studio locally. The command prints the UI address with its access token:

```sh
cargo run -p studio-desktop -- --data-dir .studio-dev/data
```

`--musicindex URL` selects a different musicindex API, such as a local test server.

Keep development and check data in `.studio-dev/` at the repository root. Git ignores this folder. Do not use `/tmp` for it.

## Design Philosophy

- Studio connects to hosts. It does not host files. The user can move files between hosts at any time ([ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- The user controls the secrets. Only the user can open the vault ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md)).
- Feed identity does not change. Studio keeps the feed GUID and each item GUID ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md)).
- No write occurs without a diff that the user has seen ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md)).
- Studio uses the musicindex.org "Afterglow" visual direction ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).
- musicindex ADRs 0020 to 0029 own the interface rules that musicindex.org, Studio and v4vmm share. They replace parts of ADR 0010, ADR 0029 and ADR 0037, and ADR 0010 names each part. A change to a shared rule needs a record in `../musicindex`.

## Working Rules

Each rule below states its owner. This file makes no rule.

Durable rules:

- The core crate has no HTTP server code and no UI code. It can make HTTP requests. Each feed rule is in the core ([ADR 0002](docs/adr/0002-rust-core-json-api-browser-ui.md)).
- A connector gets its secrets from the vault only. No secret is in a log, an API response or the collection database ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- A write to a host occurs only through `pipeline::write_draft`, with the ID of the diff that the user approved ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md), [ADR 0015](docs/adr/0015-connector-interface-and-write-pipeline.md)).
- A read and a write with no edit give the same bytes. The writer changes only the elements that the user changed. The corpus tests enforce this ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md), [ADR 0012](docs/adr/0012-lossless-tree-with-a-typed-model.md)).
- The UI loads nothing from other servers, and it has no build step ([ADR 0016](docs/adr/0016-ui-without-a-build-step.md)). A test enforces the first part.
- In local mode, each `/api/` request needs the access token, and each request needs a loopback `Host` ([ADR 0017](docs/adr/0017-local-api-access.md)). Tests enforce this.
- The vault refuses a passphrase with fewer than 12 characters or a zxcvbn score below 3. The UI gives no advice on how to make a passphrase ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)).
- An entity color is not a general accent. An error uses the `--error` token, not an entity color ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)). All functions work from the keyboard ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)). A control has the target size of its profile: 32 px with a mouse and 44 px with touch (musicindex ADR 0029 R-SCALE-3 and ADR 0024 R-TARGET-1). A test enforces the token.
- Each labeled button has a role: primary, action, normal or destructive. A button with a text label is not a `button quiet` ([ADR 0037](docs/adr/0037-button-roles.md)).
- Each entry of a list shows as one summary line, and one entry of a list is open at a time. A section that the user does not change in each visit starts collapsed, with a summary line ([ADR 0028](docs/adr/0028-summary-lines-and-one-open-entry.md)). A test enforces the `details` elements.
- The editor uses a maximum of three background levels: the page, a group and an open entry. A level changes the fill only, and a border does not identify a level ([ADR 0029](docs/adr/0029-editor-pages-toolbar-and-three-levels.md)).

Situational rules:

- The v1 connectors are local folder, SFTP, S3 and R2, Git pages, Blossom and nsite, and the optional self-hosted file host ([ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- The draft status, "Versions" and "Review" stay in the toolbar at all scroll positions and widths. The left panel holds Releases and Add a feed, or the outline of the open feed. Hosts and Lock Studio always show at its bottom ([ADR 0039](docs/adr/0039-the-shell-releases-and-the-outline.md)).
- Do not copy code from `../mi-studio`. Use it as reference only ([ADR 0001](docs/adr/0001-one-core-with-desktop-and-server-shells.md)).

Documentation:

- Put no Markdown file in the repository root other than `AGENTS.md` and a future `README.md`. The global repository-docs rules own this.
- Add each new document to [docs/README.md](docs/README.md) in the same change.
- Write prose in ASD-STE100 Simplified Technical English. Check it with `~/.agents/skills/asd-ste100/scripts/ste_lint.py`. The operator's global instructions own this rule.
- A change of a decision changes the ADR, `docs/README.md`, the tests and this file in the same commit.

## Reference Material

- Research: `../musicindex-research`. It is private. Do not copy text that has a private notice.
- Visual direction: `../musicindex`, musicindex ADR 0003 and ADR 0012, and `search.html`.
- Shared interface rules: musicindex ADRs 0020 to 0029. Shared assets and tokens: `../musicindex-assets` (musicindex ADR 0022).
- Feed API: Stophammer, through `../musicindex/api.json`.
- MSP 2.0: `../MSP-2.0`.
