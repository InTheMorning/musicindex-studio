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

On 2026-10-26, Phase 0 to Phase 11 are complete. The human tests of Phase 9 and milestone 11b are open. `studio-core` reads a feed into a lossless XML tree and a typed model, and writes each edit back. It has the encrypted secret vault, the SQLite collection, the release library, the connectors, the write pipeline and the announcement queue. The connectors are local folder, S3 and R2, SFTP, Blossom and nsite. `studio-api` serves the local JSON API and the UI. The UI has the album editor, a new release, the import of music files, the publisher feeds, the playlists and the live shows. A connected app can change a live show under a show grant of the user, through `/app/v1/`. A lossless import gives an MP3 enclosure and a FLAC alternative enclosure. `studio-desktop` runs them on a loopback address. `studio-server` serves them to many users on a public origin. It has five sign-in methods, a user folder for each account and a short vault window. Each shell sends a music file in parts, and the server is a home screen app. A connected app writes under a show grant through a grant vault, also while the user vault is locked.

The UI starts on Releases and shows the outline of an open feed. Below Releases, the left panel shows the levels of the publisher feeds: hosting, labels, then artists. At narrow width it is one stack.

- Decisions: `docs/adr/0001` to `docs/adr/0095`. All are accepted.
- Plan: [Studio v1 phase plan](docs/plans/studio-v1-phase-plan.md). Phase 5 (album editor), Phase 6 (new releases) and Phase 7 (the release library of [ADR 0035](docs/adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](docs/adr/0036-library-records-host-inventory-and-host-operations.md) and [ADR 0038](docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)) are complete, with their visual checks. Phase 8 (publisher feeds, [ADR 0041](docs/adr/0041-publisher-feeds-and-two-way-links.md)) is complete, with its visual check, and task 026 built the shell of ADR 0039. Phase 9 (remaining connectors, [ADR 0043](docs/adr/0043-phase-9-connectors-and-the-interface-change.md) and [ADR 0044](docs/adr/0044-the-nsite-signer.md)) is complete. The plan and the visual checks of milestone 7a, task 022, milestone 7b and Phase 8 list the open items.
- Phase 10 (announce, [ADR 0052](docs/adr/0052-announcements-through-the-podcast-index-hub.md)) is complete with [task 031](docs/tasks/studio-v1-task-031-announce.md), and its human test passed on 2026-10-27. After each write, Studio tells the Podcast Index hub, with a podping. Phase 11 (other feed types) is complete.
- Phase 11 has two milestones. 11a, the playlists of [ADR 0053](docs/adr/0053-playlists-are-musicl-feeds-with-remote-items.md), is [task 032](docs/tasks/studio-v1-task-032-playlists.md). 11b, the live items, connected apps and show grants of [ADR 0054](docs/adr/0054-live-items-connected-apps-and-show-grants.md), is [task 033](docs/tasks/studio-v1-task-033-live-items.md). Both milestones are complete, with their visual checks. The human test of 11b with a real host and the other app is open.
- [Task 027](docs/tasks/studio-v1-task-027-connector-interface.md) (the connector interface change) is complete. [Task 028](docs/tasks/studio-v1-task-028-s3-connector.md) (S3 and R2, [ADR 0046](docs/adr/0046-the-s3-and-r2-connector.md)) is complete, and its human test passed on 2026-10-31. [Task 029](docs/tasks/studio-v1-task-029-sftp-connector.md) (SFTP, [ADR 0047](docs/adr/0047-the-sftp-connector.md)) is complete, with its human tests open. [Task 030](docs/tasks/studio-v1-task-030-nsite-and-blossom.md) (nsite with Blossom, in eight packets) is complete, with its human tests open ([ADR 0045](docs/adr/0045-phase-9-crates-test-servers-and-host-forms.md) to [ADR 0048](docs/adr/0048-the-nsite-and-blossom-connectors.md), [ADR 0050](docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md) and [ADR 0051](docs/adr/0051-the-core-signs-nostr-events-with-secp256k1.md)).
- Phase 12 (the server shell) is in progress. Its goal is a change of a feed from a phone. [ADR 0055](docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) to [ADR 0059](docs/adr/0059-uploads-in-parts-and-the-home-screen-app.md) give the decisions. It has four milestones. [Task 034](docs/tasks/studio-v1-task-034-server-core.md) (the server core), [task 035](docs/tasks/studio-v1-task-035-phone.md) (the phone) and [task 036](docs/tasks/studio-v1-task-036-grant-vaults.md) (grant vaults) are complete, with their visual checks. Their CI results and their human tests are open. In [task 037](docs/tasks/studio-v1-task-037-runbook-and-human-tests.md), Studio `0.1.0-rc.5` runs on `studio.musicindex.org`, from the image of [task 043](docs/tasks/studio-v1-task-043-server-image.md). The human tests are next.
- The editor files and covers: [ADR 0060](docs/adr/0060-the-album-cover-keeps-the-form-of-its-feed.md) to [ADR 0062](docs/adr/0062-a-file-picker-for-each-file-field.md) decide the cover of a DeMu feed, the space around text, and a file picker with an audio player. [Task 038](docs/tasks/studio-v1-task-038-cover-and-text-space.md) and [task 039](docs/tasks/studio-v1-task-039-file-picker.md) are complete. Their screenshot visual checks and their human tests passed. [ADR 0063](docs/adr/0063-one-switch-hides-a-feed-from-apps.md) gives one switch that hides a feed from apps. [Task 040](docs/tasks/studio-v1-task-040-hide-from-apps.md) is complete, with its screenshot visual check and its human test passed.
- The fields that MSP 2.0 edits: [ADR 0064](docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) adds the fields that MSP 2.0 edits. [Task 041](docs/tasks/studio-v1-task-041-msp-fields.md) is complete in five packets. Its screenshot visual check and its human test passed. The [UI feed rule audit](docs/reviews/2026-10-27-ui-feed-rule-audit.md) found four feed rules in the UI. [ADR 0065](docs/adr/0065-the-core-applies-each-feed-rule.md) moves them to the core. [Task 042](docs/tasks/studio-v1-task-042-core-feed-rules.md) is complete in three packets, with its screenshot visual checks passed.
- The editor pages and the outline: [ADR 0067](docs/adr/0067-a-read-only-feed-opens-in-the-editor.md) opens a read-only feed in the editor. [ADR 0068](docs/adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) gives the sequence of each editor page. [ADR 0069](docs/adr/0069-the-outline-head-opens-the-feed-page.md) makes the outline head the control of the feed page. [ADR 0070](docs/adr/0070-a-cover-opens-the-artwork-sheet.md) opens the Artwork sheet from a cover. [ADR 0071](docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) adds a Settings page, and gives each section of a page a tier. [Task 047](docs/tasks/studio-v1-task-047-settings-page-and-tiers.md) is complete, with its screenshot visual check and its human test passed. [ADR 0072](docs/adr/0072-add-a-feed-opens-from-releases.md) opens Add a feed from Releases. [Task 048](docs/tasks/studio-v1-task-048-add-a-feed-from-releases.md) is complete, with its screenshot visual check. [ADR 0073](docs/adr/0073-the-add-pages-ask-one-question-at-a-time.md) makes the Add feeds page a list of choices, with one form to make a feed, and gives one row "Add tracks". [Task 049](docs/tasks/studio-v1-task-049-add-pages.md) is complete, with its screenshot visual check. Its human test on a phone is open.
- The tasks of ADR 0067 to ADR 0070: [task 044](docs/tasks/studio-v1-task-044-outline-head.md), [task 045](docs/tasks/studio-v1-task-045-page-sequence-and-artwork.md) and [task 046](docs/tasks/studio-v1-task-046-read-only-editor.md) are complete, with their screenshot visual checks. The human test of task 046 passed.
- MSP 2.0 parity: the [parity research](docs/research/2026-10-28-msp-parity-and-nostr-identities.md) gives the gaps. [ADR 0074](docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md) adds the Nostr keys of artists, labels and contributors, with a profile search. [ADR 0075](docs/adr/0075-the-publisher-page-has-payments-and-credits.md) gives the publisher page payments and credits. [ADR 0076](docs/adr/0076-an-op3-switch-on-feed-settings.md) adds an OP3 switch. [ADR 0077](docs/adr/0077-a-new-album-can-start-from-an-album.md) starts a new album from an album. [Task 050](docs/tasks/studio-v1-task-050-publisher-payments-and-credits.md) to [task 053](docs/tasks/studio-v1-task-053-album-from-album.md) give the work, in that sequence. Tasks 050 to 053 are complete, with their screenshot visual checks. Their human tests are open.
- Batch edits and the bottom group: [ADR 0091](docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) gives a change set that edits one recipient in many feeds, from a Recipients page. [ADR 0092](docs/adr/0092-file-hosts-and-the-destination-icons.md) renames Hosts to "File hosts", and gives each fixed destination an icon. [Task 054](docs/tasks/studio-v1-task-054-recipient-change-sets.md) and [task 056](docs/tasks/studio-v1-task-056-file-hosts-and-icons.md) give the work. [Task 055](docs/tasks/studio-v1-task-055-date-time-fields.md) gives the date and time fields of [ADR 0093](docs/adr/0093-dates-and-times-use-a-picker-in-local-time.md). Tasks 054, 055 and 056 are complete, with their screenshot visual checks. Their human tests passed on 2026-10-31. [Task 057](docs/tasks/studio-v1-task-057-confirmation-sheets.md) (confirmation sheets) is complete. [Task 058](docs/tasks/studio-v1-task-058-title-case-buttons.md) (title case on buttons) is complete. [Task 059](docs/tasks/studio-v1-task-059-duration-field.md) (the duration field of [ADR 0094](docs/adr/0094-a-duration-is-edited-as-minutes-and-seconds.md)) is complete. The plan section "Recipient Batch Edits, File Hosts And Dates" gives their sequence. [ADR 0095](docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md) makes a Lightning address and its node key one recipient, from the [keysend research](docs/research/2026-10-31-keysend-and-lightning-address-recipients.md). [Task 060](docs/tasks/studio-v1-task-060-wallet-recipients.md) is complete in four packets, with its screenshot visual check. Its human test is open.
- Desktop releases: the [distribution plan](docs/plans/distribution-phase-plan.md) gives the steps to the desktop release drafts. [ADR 0066](docs/adr/0066-the-server-ships-as-a-container-image.md) gives the server image: a candidate tag builds `ghcr.io/inthemorning/studio-server` after a passed CI run, and `tools/deploy-server.sh --release VERSION` deploys it. In milestone 1, `.github/workflows/desktop-release.yml` makes and tests an archive on five native runners. Milestone 2 (the Tauri installers) starts after Phase 12.
- A manual run of the release workflow on `81542b8` passed on the five native runners. The first release candidate was `0.1.0-rc.1`. `0.1.0-rc.2` was the first candidate with the server image of ADR 0066. The workspace version is `0.1.0-rc.6`. It adds tasks 057 to 060 to `0.1.0-rc.5`: the confirmation sheets, title case on buttons, the duration field and the wallet recipients. `0.1.0-rc.5` runs on `studio.musicindex.org`. The [distribution review](docs/reviews/2026-10-26-distribution-review.md) gives the release gates.
- Research: [baseline research note](docs/research/2026-10-22-studio-baseline-research.md) and [Phase 12 authentication research](docs/research/2026-10-26-phase-12-auth-research.md).
- Open questions: the plan lists them. [ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md) has open questions about the self-hosted file host and file moves.

## Layout

| Path | Contents |
| --- | --- |
| `crates/core` | `studio-core`. The feed model is in `feed`, `xml` and `guid`. The other modules are `vault`, `collection`, `address` (feed addresses), `connector`, `media`, `library`, `inventory`, `migrate`, `pipeline` (the write), `announce` (the announcement queue), `playlist` (the track sources of a playlist), `live` (the live item rules), `apps` (connected apps) and `grant` (show grants). `net` has the network policy, `nip46` has the NIP-46 client, and `grant_vault` has the grant vaults. No HTTP server code and no UI code. |
| `crates/core/tests/corpus` | Published feeds for the round-trip tests. `manifest.json` gives the source URL, the fetch time and the SHA-256 of each file. |
| `crates/api` | `studio-api`: the JSON API over the core, and the UI files in `crates/api/ui/` |
| `crates/api/ui-tests` | The Node tests of the UI model modules ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)) |
| `crates/server` | `studio-server`: the server shell. It keeps `server.sqlite` and one user folder for each account. |
| `crates/desktop` | `studio-desktop`: the desktop shell. It runs the local API and prints the UI address. Its default data folder is the local data folder of the operating system. Tauri comes in the Desktop Shell phase. |
| `.github/workflows` | `ci.yml` runs the four checks. `desktop-release.yml` builds the desktop archives and makes a draft GitHub Release from a `v*` tag. |
| `docs/` | ADRs, plans, tasks, reviews, runbooks and research. The index is `docs/README.md`. |
| `tools/drive-ui.mjs` | Drives the UI in headless Chromium for a check. Its header gives the usage. |
| `tools/release_desktop.py` | Gives the version, packages a desktop archive, runs its smoke test and writes `SHA256SUMS.txt`. The release workflow uses it ([desktop release runbook](docs/runbooks/desktop-release.md)). |
| `tools/node` | Runs a pinned Node.js from `.studio-dev/tools/` ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)). |

## Commands

These four checks must pass before each commit. CI runs the same checks ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md) adds the fourth). `rust-toolchain.toml` pins the Rust version, and CI uses the same version ([ADR 0049](docs/adr/0049-the-rust-toolchain-is-pinned.md)).

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

On a computer with no Node.js, run `tools/node --test crates/api/ui-tests/*.test.mjs`. `tools/node` gets a pinned Node.js into `.studio-dev/tools/` at its first run. It also runs `tools/drive-ui.mjs`.

Run Studio locally. The command prints the UI address with its access token:

```sh
cargo run -p studio-desktop -- --data-dir .studio-dev/data
```

`--musicindex URL` selects a different musicindex API, such as a local test server.

Keep development and check data in `.studio-dev/` at the repository root. Git ignores this folder. Do not use `/tmp` for it.

## Design Philosophy

- Studio connects to hosts. It does not host files. The user can move files between hosts at any time ([ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- The user controls the secrets. Only the user can open the vault. Under a show grant, the token of a connected app of the user opens a grant vault with one host secret ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0058](docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md)).
- Feed identity does not change. Studio keeps the feed GUID and each item GUID ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md)).
- No write occurs without a diff that the user has seen, or a change that a show grant of the user covers ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md), [ADR 0054](docs/adr/0054-live-items-connected-apps-and-show-grants.md)).
- Studio uses the musicindex.org "Afterglow" visual direction ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).
- musicindex ADRs 0020 to 0029 own the interface rules that musicindex.org, Studio and v4vmm share. They replace parts of ADR 0010, ADR 0029 and ADR 0037, and ADR 0010 names each part. A change to a shared rule needs a record in `../musicindex`.

## Working Rules

Each rule below states its owner. This file makes no rule.

Durable rules:

- The core crate has no HTTP server code and no UI code. It can make HTTP requests. Each feed rule is in the core ([ADR 0002](docs/adr/0002-rust-core-json-api-browser-ui.md)).
- The UI has no copy of a feed rule. An edit aid in the UI gives only a start value or a display value. The core can change a sent value at the draft save, or through a core route for one user action. Each task packet that changes the UI has the constraint "No feed rule in the UI (ADR 0065)", and its review examines each new UI function that changes the model. A test cannot tell a feed rule from an edit aid ([ADR 0065](docs/adr/0065-the-core-applies-each-feed-rule.md)).
- A connector gets its secrets from the vault only. No secret is in a log, an API response or the collection database ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- A write to a host occurs only through `pipeline::write_draft`. It uses the ID of the diff that the user approved, or of a diff that a show grant covers ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md), [ADR 0015](docs/adr/0015-connector-interface-and-write-pipeline.md), [ADR 0054](docs/adr/0054-live-items-connected-apps-and-show-grants.md)).
- A read and a write with no edit give the same bytes. The writer changes only the elements that the user changed. The corpus tests enforce this ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md), [ADR 0012](docs/adr/0012-lossless-tree-with-a-typed-model.md)).
- The UI loads nothing from other servers, and it has no build step ([ADR 0016](docs/adr/0016-ui-without-a-build-step.md)). A test enforces the first part.
- UI logic that a test examines is in a model module with no Preact and no browser state. The Node tests in `crates/api/ui-tests/` examine it ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)).
- In local mode, each `/api/` request needs the access token, and each request needs a loopback `Host` ([ADR 0017](docs/adr/0017-local-api-access.md)). Tests enforce this. Each `/app/v1/` request needs the token of a connected app, and no other route accepts that token ([ADR 0054](docs/adr/0054-live-items-connected-apps-and-show-grants.md)).
- A secret value stays hidden until the user selects its eye button. A secret value is in a vault, or Studio shows it one time only. A Node test finds a secret field that does not use the component of `app.js` ([ADR 0078](docs/adr/0078-a-secret-value-stays-hidden-until-the-user-shows-it.md)).
- The vault refuses a passphrase with fewer than 12 characters or a zxcvbn score below 3. The UI gives no advice on how to make a passphrase ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)).
- An entity color is not a general accent. An error uses the `--error` token, not an entity color ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)). All functions work from the keyboard ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)). A control has the target size of its profile: 32 px with a mouse and 44 px with touch (musicindex ADR 0029 R-SCALE-3 and ADR 0024 R-TARGET-1). A test enforces the token.
- The type dot and the entity colors follow musicindex ADR 0032 R-DOT-1. A test compares `styles.css` with `crates/api/ui/assets/tokens.json`, the copy from `../musicindex-assets` (musicindex ADR 0032 R-DOT-2).
- A publisher feed shows each of its roles, for example "Artist" or "Artist · Label". The interface shows the word "publisher" only as an RSS name (musicindex ADR 0033 R-WORD-3 and R-WORD-4, musicindex ADR 0037 R-WORD-7, [ADR 0084](docs/adr/0084-the-label-of-a-feed-lists-each-role.md)). A Node test examines the UI text.
- A publisher feed has a set of the eight roles of musicindex ADR 0012, in the sequence of musicindex ADR 0036. A role that Studio infers from the feed has a hollow dot until the user confirms it. A release in the collection that gives the first role in its link also fills the dot. Artist, label and hosting are the choices of the Role menu and of the form of a new publisher feed ([ADR 0082](docs/adr/0082-a-publisher-feed-has-a-set-of-roles.md), [ADR 0083](docs/adr/0083-hosting-is-a-first-class-role.md), [ADR 0087](docs/adr/0087-the-two-sides-of-a-link-confirm-a-role.md)).
- A confirmation is a sheet of Studio, never the browser box `confirm()`. Its texts are in `confirms.js`. A Node test finds each call of `confirm(` (musicindex ADR 0023 R-CONFIRM-1, [task 057](docs/tasks/studio-v1-task-057-confirmation-sheets.md)).
- A button label uses title case. Each other text uses sentence case. A Node test finds a button label that is not in title case (musicindex ADR 0023 R-CASE-1, [task 058](docs/tasks/studio-v1-task-058-title-case-buttons.md)).
- Each labeled button has a role: primary, action, normal or destructive. A button with a text label is not a `button quiet` ([ADR 0037](docs/adr/0037-button-roles.md)).
- Each entry of a list shows as one summary line, and one entry of a list is open at a time. A section that the user does not change in each visit starts collapsed, with a summary line ([ADR 0028](docs/adr/0028-summary-lines-and-one-open-entry.md)). A test enforces the `details` elements.
- Each section of a page has a tier: identity, content, details, setup, then limits. A page shows its sections in the sequence of their tiers. `crates/api/ui/pages.js` gives the sections, and a Node test enforces the sequence. A new section needs a tier, not an ADR ([ADR 0071](docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)).
- The editor uses a maximum of three background levels: the page, a group and an open entry. A level changes the fill only, and a border does not identify a level. A group holds rows, and thin separators divide them ([ADR 0029](docs/adr/0029-editor-pages-toolbar-and-three-levels.md)). Before each commit of a UI packet, the visual check runs `$edges()` of `tools/drive-ui.mjs` to find content at the edge of a group. CI has no browser, so no commit check runs it.

Situational rules:

- The v1 connectors are local folder, SFTP, S3 and R2, Blossom and nsite, and the optional self-hosted file host. The Git pages connector comes after v1 ([ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md), [ADR 0043](docs/adr/0043-phase-9-connectors-and-the-interface-change.md)).
- No deletion on a host occurs before each new file passes the confirmation ([ADR 0043](docs/adr/0043-phase-9-connectors-and-the-interface-change.md)).
- No test sends a request to a host outside the computer. Each connector passes the trait suite against a local test server ([ADR 0045](docs/adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- The draft status, "Versions" and "Review" stay in the toolbar at all scroll positions and widths. The left panel holds the destinations, or the outline of the open feed. Recipients, File hosts, Settings and Lock Studio always show at its bottom. On a server with a locked vault, "Unlock Studio" is in the place of "Lock Studio". At 760 px or less, Studio is one stack with a root page and an outline page, and no menu button ([ADR 0039](docs/adr/0039-the-shell-releases-and-the-outline.md), musicindex ADR 0030, [ADR 0091](docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md)).
- Each fixed destination shows its Lucide icon, and no destination row shows a type dot. A UI text names a connector "file host". The word "host" alone is the role `host` ([ADR 0092](docs/adr/0092-file-hosts-and-the-destination-icons.md)).
- A change set applies one recipient operation to many feeds. The core calculates each split, and Discard puts back each draft from before the change. Each feed of a change set writes through `pipeline::write_draft` with the ID of its own diff ([ADR 0091](docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md)).
- A `node` recipient and an `lnaddress` recipient with the same keysend lookup result are one recipient, and an operation acts on each address of that recipient. The core makes the lookup and keeps the result in the collection. An operation that puts an `lnaddress` recipient into a value block writes `method="lnaddress"`. Change address and Add to a level need a type ([ADR 0095](docs/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md)).
- Below Releases, the left panel shows the top publisher feeds, then "Not linked". The core gives the links of each feed in the feed list. Releases lists only the albums and the playlists. "Other feeds" holds each feed of a type that the editor does not open, and its page gives the cause ([ADR 0085](docs/adr/0085-each-artist-and-label-is-a-destination.md), [ADR 0090](docs/adr/0090-studio-names-the-feeds-that-it-does-not-edit.md)).
- A publisher feed has the rank hosting, label or artist, from its first role. A feed of a lower rank that shares a release is below it. Each level of the panel has "‹" to the level above, and a feed that opens from a level goes back to it. The core gives `child_ids` and `parent_ids` ([ADR 0088](docs/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md)).
- A row of a publisher feed in the left panel shows its art: its cover, or a mosaic of its release covers, or the logo shape. The type dot is on the bottom right corner of the art (musicindex ADR 0038 R-DOT-3, [ADR 0089](docs/adr/0089-the-left-panel-shows-the-art-of-each-publisher-feed.md)).
- A release is an album or a playlist. A playlist links to a publisher feed as an album does ([ADR 0086](docs/adr/0086-a-playlist-links-to-its-artist-or-label.md)).
- Add a feed opens from a button on Releases, not from the left panel. At narrow width, Back from Add feeds opens Releases ([ADR 0072](docs/adr/0072-add-a-feed-opens-from-releases.md)).
- The Add feeds page holds rows, and each row opens its form in a sheet. One form makes each new feed. The outline has one row "Add tracks" ([ADR 0073](docs/adr/0073-the-add-pages-ask-one-question-at-a-time.md)).
- The outline head opens the first page of the feed, and the outline has no row for that page. The link "‹ Releases" opens Releases, not the history entry before it. From a level, the link names that level and opens it ([ADR 0069](docs/adr/0069-the-outline-head-opens-the-feed-page.md), [ADR 0088](docs/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md)).
- Each editor page starts from the sequence of [ADR 0068](docs/adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md). `pages.js` owns the current sequence. The Hosts page holds only hosts, and the Studio settings are on the Settings page ([ADR 0071](docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)).
- A cover in the header of an editor page opens the Artwork sheet of its item. Each artwork field is only in that sheet ([ADR 0070](docs/adr/0070-a-cover-opens-the-artwork-sheet.md)).
- A read-only album, publisher feed or playlist opens in the editor. The write of its draft needs a host first. A move from a read-only feed sets no redirect and writes no pointer file ([ADR 0067](docs/adr/0067-a-read-only-feed-opens-in-the-editor.md)).
- On the server, each `/api/` request needs a session cookie and the `X-Studio-CSRF` header. The server keeps the vault key only in the vault window ([ADR 0055](docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).
- With the network policy `Public` of the server, no outbound connection goes to a private, loopback or reserved address ([ADR 0057](docs/adr/0057-the-private-network-guard-of-the-server.md)).
- A granted write uses only the secrets of its grant vault, in both shells ([ADR 0058](docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md)).
- No server image comes from a commit that did not pass CI. The VPS publishes the port of the container only on a loopback address ([ADR 0066](docs/adr/0066-the-server-ships-as-a-container-image.md)).
- Studio writes a Nostr key only as the channel `podcast:txt` with `purpose="npub"` or as the `npub` attribute of `podcast:person`. It sets no key without a selection or an entry by the user ([ADR 0074](docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md)).
- Each comparison of a feed address with a file address uses the plain address, with no OP3 prefix ([ADR 0076](docs/adr/0076-an-op3-switch-on-feed-settings.md)).
- A position menu sets the sequence of the album tracks, the playlist tracks and the releases of a publisher feed. A move changes no track number in the UI ([ADR 0081](docs/adr/0081-a-position-menu-sets-the-sequence-of-a-list.md)).
- The track page shows `podcast:season` as "Disc", in a collapsed entry with the Experimental badge of musicindex ADR 0034 ([ADR 0080](docs/adr/0080-the-season-of-a-track-is-an-experimental-disc-setting.md)).
- A copy of an album keeps no feed GUID and no item GUID of its source ([ADR 0077](docs/adr/0077-a-new-album-can-start-from-an-album.md)).
- Each date and time field is a `DateTimeField` of `app.js`, in the local time of the browser. The UI sends RFC 3339 text with the local offset. The core writes `pubDate` from `pub_date_time`, and a `pubDate` that the user does not change keeps its bytes ([ADR 0093](docs/adr/0093-dates-and-times-use-a-picker-in-local-time.md)).
- The track field "Duration" edits `duration_seconds` as minutes and seconds. The core writes a changed `itunes:duration` as `HH:MM:SS`, and a duration that the user does not change keeps its bytes ([ADR 0094](docs/adr/0094-a-duration-is-edited-as-minutes-and-seconds.md)).
- Do not copy code from `../mi-studio`. Use it as reference only ([ADR 0001](docs/adr/0001-one-core-with-desktop-and-server-shells.md)).

Documentation:

- Put no Markdown file in the repository root other than `AGENTS.md` and `README.md`. `README.md` is the short user guide: it holds only links to `docs/user-guide/`. The global repository-docs rules own this.
- Add each new document to [docs/README.md](docs/README.md) in the same change. Add each new user guide page to `README.md`.
- Write prose in ASD-STE100 Simplified Technical English. Check it with `~/.agents/skills/asd-ste100/scripts/ste_lint.py`. The operator's global instructions own this rule.
- A change of a decision changes the ADR, `docs/README.md`, the tests and this file in the same commit.

## Reference Material

- Research: `../musicindex-research`. It is private. Do not copy text that has a private notice.
- Visual direction: `../musicindex`, musicindex ADR 0003 and ADR 0012, and `search.html`.
- Shared interface rules: musicindex ADRs 0020 to 0029. Shared assets and tokens: `../musicindex-assets` (musicindex ADR 0022).
- Feed API: Stophammer, through `../musicindex/api.json`.
- MSP 2.0: `../MSP-2.0`.
