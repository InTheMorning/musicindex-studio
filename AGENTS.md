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

Phase 0 to Phase 11 are complete. `studio-core` reads a feed into a lossless XML tree and a typed model, and writes each edit back. It has the encrypted secret vault, the SQLite collection, the release library, the connectors, the write pipeline and the announcement queue. The connectors are local folder, S3 and R2, SFTP, Blossom and nsite. `studio-api` serves the local JSON API and the UI. The UI has the album editor, a new release, the import of music files, the publisher feeds, the playlists and the live shows. A connected app can change a live show under a show grant of the user, through `/app/v1/`. A lossless import gives an MP3 enclosure and a FLAC alternative enclosure. `studio-desktop` runs them on a loopback address. `studio-server` serves them to many users on a public origin. It has five sign-in methods, a user folder for each account and a short vault window. Each shell sends a music file in parts, and the server is a home screen app. A connected app writes under a show grant through a grant vault, also while the user vault is locked.

The UI starts on Releases and shows the outline of an open feed. [ADR 0263](docs/adr/0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) replaces Releases with Overview, [ADR 0264](docs/adr/0264-overview-groups-the-releases-below-label-artist-and-curator-headings.md) groups Overview below label, artist and curator headings, ADR 0263 puts each playlist below its curator page, and gives the release list of a page one sequence with three moves. [Task 256](docs/tasks/studio-v1-task-256-overview-and-the-sequence-of-a-page.md) gives its build, and no packet is started. The left panel is the Catalog tree of the publisher feeds ([ADR 0254](docs/adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md)), and each link has its own roles ([ADR 0253](docs/adr/0253-the-roles-of-a-feed-and-of-each-link.md)). A release has one artist page, one label page and one publisher, in the fields of its first tab ([ADR 0257](docs/adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md)). Tasks 250 to 254 built these three decisions. The operator checks of task 254 are open.

- Decisions: [docs/README.md](docs/README.md) is the index. Its section "ADRs" lists each current decision, in `docs/adr/`. Each current decision is accepted. Its section "Archive" lists each superseded decision, in `docs/archive/adr/`, with the decision that replaces it. [ADR 0203](docs/adr/0203-people.md) gives the People area.
- Plan: [Studio v1 phase plan](docs/plans/studio-v1-phase-plan.md). Phase 0 to Phase 11 are complete. Phase 12 (the server shell) is in progress. The plan section "Status Of Each Area" gives the state of each area, each task and each human check.
- The current candidate is `0.1.0-rc.32`, with its [notes](docs/releases/0.1.0-rc.32.md). Its tag `v0.1.0-rc.32` is on `66a1a64`. The [checks of rc.32](docs/reviews/2026-11-22-rc32-checks.md), the [checks of rc.31](docs/reviews/2026-11-21-rc31-checks.md) and the [checks of rc.30](docs/reviews/2026-11-21-rc30-checks.md) give the open steps. The [walk of the older human tests](docs/reviews/2026-11-14-open-older-human-tests.md) gives the open steps of earlier candidates.
- `Cargo.toml` gives the workspace version. `tools/release-candidate.sh VERSION` makes a release candidate and deploys it. After 0.1.0-rc.9, it stops when `docs/releases/VERSION.md` is not in the commit of the tag. The draft GitHub Release gets that file as its notes ([ADR 0100](docs/adr/0100-each-release-candidate-has-a-notes-file.md)). The notes are for testers who are not the operator. The checks of the operator are in `docs/reviews/DATE-rcN-checks.md` ([ADR 0236](docs/adr/0236-the-release-notes-are-for-testers-and-the-operator-checks-are-in-a-review-file.md)). The [distribution review](docs/reviews/2026-10-26-distribution-review.md) gives the release gates.
- Research: [baseline research note](docs/research/2026-10-22-studio-baseline-research.md) and [Phase 12 authentication research](docs/research/2026-10-26-phase-12-auth-research.md).
- Open questions: the plan lists them. [ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md) has open questions about the self-hosted file host and file moves.

## Layout

| Path | Contents |
| --- | --- |
| `crates/core` | `studio-core`. The feed model is in `feed`, `xml` and `guid`. The other modules are `vault`, `collection`, `collections` (the collections of the desktop app), `outside` (the check for a change from outside), `people` (the people of a user and the suggestions), `address` (feed addresses), `connector`, `media`, `library`, `inventory`, `migrate`, `pipeline` (the write), `merge` (the hosted file into a draft), `move_link` (the links that a write follows), `feed_check` (the check of a feed), `medium_problem` (a bad or missing medium tag), `feed_reads` (the reads of the feed check), `checked_time` (the time for a build date), `announce` (the announcement queue), `playlist` (the track sources of a playlist), `live` (the live item rules), `roles` (the credit role list), `apps` (connected apps) and `grant` (show grants). `net` has the network policy, `nip46` has the NIP-46 client, and `grant_vault` has the grant vaults. No HTTP server code and no UI code. |
| `crates/core/tests/corpus` | Published feeds for the round-trip tests. `manifest.json` gives the source URL, the fetch time and the SHA-256 of each file. |
| `crates/api` | `studio-api`: the JSON API over the core, and the UI files in `crates/api/ui/` |
| `crates/api/ui-tests` | The Node tests of the UI model modules ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)) |
| `crates/server` | `studio-server`: the server shell. It keeps `server.sqlite` and one user folder for each account. |
| `crates/app-client` | `studio-app-client`: the client of a connected app (ADR 0110). It signs each request with the key of the app. It does not depend on `studio-core`. |
| `crates/desktop` | `studio-desktop`: the desktop shell. It runs the local API and prints the UI address. Its default data folder is the local data folder of the operating system. Its library has the start of the shell, which the app also uses. |
| `crates/desktop-app` | `studio-desktop-app`: the Tauri app "Music Index Studio" ([ADR 0124](docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md)). It opens one window on the local API, and it has a tray. |
| `.github/workflows` | `ci.yml` runs the four checks. `desktop-release.yml` builds the desktop archives and installers, and makes a draft GitHub Release from a `v*` tag. |
| `packaging/arch` | The PKGBUILD of the desktop app and its desktop file ([ADR 0216](docs/adr/0216-the-linux-packages-of-the-desktop-app.md), [Arch package runbook](docs/runbooks/arch-package.md)). |
| `docs/` | ADRs, plans, tasks, reviews, runbooks, release notes (`docs/releases/`) and research. The index is `docs/README.md`. |
| `tools/drive-ui.mjs` | Drives the UI in headless Chromium for a check. Its header gives the usage. |
| `tools/fit-check.mjs` | The fit check helpers of musicindex ADR 0065 R-FIT-5. It is a copy from musicindex-lib. Do not change it in this repository. |
| `tools/release_desktop.py` | Gives the version, packages a desktop archive, runs its smoke test and writes `SHA256SUMS.txt`. The release workflow uses it ([desktop release runbook](docs/runbooks/desktop-release.md)). |
| `tools/release-candidate.sh` | Sets the version, pushes `main`, waits for CI, pushes the tag, waits for the server image and runs `tools/deploy-server.sh`. Its header gives the steps. A second run continues after the last completed step. |
| `tools/gate.sh` | The commit gate: the four checks of CI and the ADR link guard. |
| `tools/node` | Runs a pinned Node.js from `.studio-dev/tools/` ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)). |

## Commands

`tools/gate.sh` is the commit gate. It must pass before each commit. It runs `cargo fmt`, `cargo clippy`, the Rust tests, the Node tests of the UI models ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)) and the ADR link guard of the `project-baseline` skill. The guard runs its bar check from ADR 0214 (`--bar-from 0214`). CI runs the first four checks, because the guard script is not in this repository. `rust-toolchain.toml` pins the Rust version, and CI uses the same version ([ADR 0049](docs/adr/0049-the-rust-toolchain-is-pinned.md)).

```sh
tools/gate.sh
```

`tools/node` gets a pinned Node.js into `.studio-dev/tools/` at its first run. It runs the Node tests and `tools/drive-ui.mjs`.

Run Studio locally. The command prints the UI address with its access token:

```sh
cargo run -p studio-desktop -- --data-dir .studio-dev/data
```

`--musicindex URL` selects a different musicindex API, such as a local test server.

Keep development and check data in `.studio-dev/` at the repository root. Git ignores this folder. Do not use `/tmp` for it.

## Design Philosophy

- Studio connects to hosts. It does not host files. The user can move files between hosts at any time ([ADR 0009](docs/adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- The user controls the secrets. Only the user can open the vault. Under a show grant, the key of a connected app that the grant names opens a grant vault with one host secret ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0058](docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md), [ADR 0110](docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md)).
- Feed identity does not change. Studio keeps the feed GUID and each item GUID ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md)). Studio changes a feed GUID on its own at one time only: a placeholder GUID gets the real GUID ([ADR 0122](docs/adr/0122-a-new-user-starts-with-a-default-file-host.md) item 8). Each other GUID change stops at the write until the user types the feed title. An outside GUID change gives a Warning with **Put Back** and **Keep the New GUID** ([ADR 0249](docs/adr/0249-studio-strongly-discourages-a-feed-guid-change.md)).
- No write occurs without a diff that the user has seen, or a change that a show grant of the user covers ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md), [ADR 0255](docs/adr/0255-publisher-feeds-announcements-and-show-grants.md) item 53).
- Studio uses the musicindex.org "Afterglow" visual direction ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).
- musicindex ADR 0021, ADR 0022, ADR 0026, ADR 0028, ADR 0068, ADR 0080 and ADR 0085 own the interface rules that musicindex.org, Studio and HitIt share. They replace parts of ADR 0010, ADR 0029 and ADR 0037, and ADR 0010 names each part. A change to a shared rule needs a record in `../musicindex`.
- The words that Studio shows for feed data come from `../musicindex/docs/glossary.md` (musicindex ADR 0087, [ADR 0244](docs/adr/0244-studio-reads-the-fee-value-of-musicindex-adr-0088.md) item 4). Studio keeps its own feed reader, and it reads `fee` as musicindex ADR 0093 R-FEED-6 gives.
- From 2026-11-11 the app v4vmm has the name HitIt (HitIt ADR 0096). In archived records, finished tasks, reviews, dated records and accepted ADRs, v4vmm means HitIt. The checkout of HitIt is `../HitIt`, and `../v4vmm` in these records means `../HitIt`.

## Working Rules

Each rule below names its owner, and this file makes no rule. Read the owner before a change in its area.

Durable rules:

- The core holds each feed rule, and it has no HTTP server code and no UI code ([ADR 0002](docs/adr/0002-rust-core-json-api-browser-ui.md)).
- The UI has no copy of a feed rule. An edit aid gives only a start value or a display value ([ADR 0065](docs/adr/0065-the-core-applies-each-feed-rule.md)). No test can tell a feed rule from an edit aid. Thus each UI task packet has the constraint "No feed rule in the UI (ADR 0065)". Its review examines each new UI function that changes the model.
- A connector gets its secrets from the vault only. No secret goes into a log, an API answer or the collection database ([ADR 0003](docs/adr/0003-user-unlocked-secret-vault.md), [ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md)).
- A write to a host goes only through `pipeline::write_draft`, with the ID of an approved diff or of a diff that a show grant covers ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md), [ADR 0015](docs/adr/0015-connector-interface-and-write-pipeline.md), [ADR 0255](docs/adr/0255-publisher-feeds-announcements-and-show-grants.md) item 53).
- A read and a write with no edit give the same bytes, and the writer changes only the elements that the user changed ([ADR 0008](docs/adr/0008-feed-types-and-feed-identity.md), [ADR 0012](docs/adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0127](docs/adr/0127-a-changed-feed-gets-a-new-build-date.md)).
- The UI has no build step and loads no script, style, font or frame from another server ([ADR 0016](docs/adr/0016-ui-without-a-build-step.md), [ADR 0109](docs/adr/0109-images-and-audio-load-from-their-hosts.md)).
- UI logic that a test examines is in a model module with no Preact and no browser state ([ADR 0042](docs/adr/0042-ui-models-with-node-tests.md)).
- Each local `/api/` request needs the access token and a loopback `Host`. Only `/app/v1/` takes the signed event of a connected app ([ADR 0017](docs/adr/0017-local-api-access.md), [ADR 0110](docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md)).
- A secret value stays hidden until the user selects its eye button ([ADR 0078](docs/adr/0078-a-secret-value-stays-hidden-until-the-user-shows-it.md)).
- Each place that edits one type of value uses the picker of that type ([ADR 0123](docs/adr/0123-one-type-of-value-has-one-picker.md)).
- Each function works from the keyboard ([ADR 0010](docs/adr/0010-studio-uses-the-afterglow-direction.md)).
- An error uses the `--error` token, not an entity color ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)).
- Each control size and each text size is a token of the shared rules (musicindex ADR 0068 R-SCALE-4, R-TARGET-1 and R-TYPE-1).
- The type dot and the entity colors follow musicindex ADR 0032 R-DOT-1. `crates/api/ui/assets/tokens.json` changes only as a copy from `../musicindex-assets` (R-DOT-2).
- A confirmation is a sheet of Studio, never the browser box `confirm()` (musicindex ADR 0085 R-CONFIRM-1).
- A destructive button is the first button of its row (musicindex ADR 0064 R-ACT-8 to R-ACT-12, [ADR 0193](docs/adr/0193-a-destructive-button-is-the-first-button-of-its-row.md)).
- A button label uses title case, and each other text uses sentence case (musicindex ADR 0085 R-CASE-1).
- Each labeled button has one of the roles of [ADR 0037](docs/adr/0037-button-roles.md).
- A list shows each entry as one summary line, and one entry is open at a time ([ADR 0028](docs/adr/0028-summary-lines-and-one-open-entry.md)).
- The editor has a maximum of three background levels ([ADR 0029](docs/adr/0029-editor-pages-toolbar-and-three-levels.md)). CI has no browser. Thus the visual check of each UI packet runs `$edges()` of `tools/drive-ui.mjs` before the commit.
- Each section of a page has a tier, and `crates/api/ui/pages.js` gives the sections in the sequence of their tiers ([ADR 0071](docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)).

Situational rules:

- Connectors: the v1 set ([ADR 0006](docs/adr/0006-connectors-and-the-v1-set.md), [ADR 0043](docs/adr/0043-phase-9-connectors-and-the-interface-change.md)). No test sends a request to a host outside the computer ([ADR 0045](docs/adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- No deletion on a host occurs before each new file passes the confirmation ([ADR 0043](docs/adr/0043-phase-9-connectors-and-the-interface-change.md)). A move keeps each old copy until the user reviews its deletion ([ADR 0148](docs/adr/0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md)).
- A read of a feed moves only a draft with no edits to the new file. The user selects each merge of a hosted file ([ADR 0007](docs/adr/0007-draft-diff-write-and-announce.md) item 6, [ADR 0129](docs/adr/0129-a-draft-takes-a-change-of-the-hosted-file.md)).
- The vault refuses a weak passphrase, and the UI gives no advice on a passphrase ([ADR 0018](docs/adr/0018-passphrase-strength-and-error-color.md)).
- A publisher feed shows each of its roles. The screen word of the role `hosting` is "Publisher", and no screen uses "host" or "Hosting" for it. Each link has its own roles, a subset of the set of its feed. The dot is filled when the two sides of a link give the first role ([ADR 0253](docs/adr/0253-the-roles-of-a-feed-and-of-each-link.md), musicindex ADR 0097 R-ROLE-7 and R-WORD-15, musicindex ADR 0033 R-WORD-3 and R-WORD-4).
- The role `hosting` ("Publisher") has the highest level rank ([ADR 0253](docs/adr/0253-the-roles-of-a-feed-and-of-each-link.md) item 15). Only the page of an artist has the add row for a release ([ADR 0164](docs/adr/0164-the-releases-of-a-publisher-feed-have-an-add-row.md), [ADR 0256](docs/adr/0256-a-release-starts-on-the-page-of-its-artist.md) item 4, [ADR 0254](docs/adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) item 15).
- Sort menus name both directions in words ([ADR 0155](docs/adr/0155-the-sort-menu-names-both-directions.md), musicindex ADR 0054 R-LIST-13). The People list sorts as [ADR 0153](docs/adr/0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) gives.
- The results of a write, the refusal panel and the release checklist follow [ADR 0217](docs/adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md), [ADR 0218](docs/adr/0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) and [ADR 0219](docs/adr/0219-a-release-checklist-and-a-check-of-the-address-type.md).
- The header of a release page links Music Index only when the outside check finds the feed there ([ADR 0220](docs/adr/0220-a-release-page-links-its-page-on-music-index.md)).
- An error notice names the action and gives one step, and the raw error is in a closed fold ([ADR 0222](docs/adr/0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md), musicindex ADR 0085 R-MSG-1).
- Splits, the release search, the phone layout, the root order and the person tabs follow [ADR 0223](docs/adr/0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) and [ADR 0226](docs/adr/0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md).
- Undo covers the edits before Write or Save. A write to a host, a podping and a Nostr publish get no undo ([ADR 0227](docs/adr/0227-undo-and-redo.md), [ADR 0228](docs/adr/0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md)).
- The shell, the panel footer and the narrow layout follow [ADR 0258](docs/adr/0258-the-shell-and-the-editor-pages.md), [ADR 0203](docs/adr/0203-people.md) item 36, [ADR 0241](docs/adr/0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md) and [ADR 0242](docs/adr/0242-the-collection-row-of-the-footer-opens-switch-collection.md).
- The left panel is the Catalog: a tree of the publisher feeds. Each change of a page or of a release asks one question in music words, and waits for Review ([ADR 0254](docs/adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md)). The art and the panel marks follow [ADR 0089](docs/adr/0089-the-left-panel-shows-the-art-of-each-publisher-feed.md), [ADR 0090](docs/adr/0090-studio-names-the-feeds-that-it-does-not-edit.md), [ADR 0092](docs/adr/0092-file-hosts-and-the-destination-icons.md), [ADR 0259](docs/adr/0259-studio-checks-each-feed-and-the-left-panel-marks-its-state.md) and [ADR 0254](docs/adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md). A UI text names a connector "file host", because "host" alone is the role `host` ([ADR 0092](docs/adr/0092-file-hosts-and-the-destination-icons.md)).
- A change set applies one operation to many feeds. Each feed writes with its own diff, and Discard puts back each draft ([ADR 0207](docs/adr/0207-recipients-change-sets-and-wallets.md) items 24 to 33).
- Wallet addresses, their types and their lookups follow [ADR 0207](docs/adr/0207-recipients-change-sets-and-wallets.md) items 34 to 47, [ADR 0096](docs/adr/0096-add-to-an-artist-or-label.md), [ADR 0097](docs/adr/0097-studio-suggests-the-lightning-address-of-a-wallet.md), [ADR 0138](docs/adr/0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) and [ADR 0217](docs/adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 7 to 11.
- The module `people` writes no feed, and Studio makes no link of a person without the user ([ADR 0203](docs/adr/0203-people.md) items 1 to 3). The core calculates each difference between a person and a feed, and the UI calculates none ([ADR 0203](docs/adr/0203-people.md) item 70).
- Studio applies no People value and adds no `https://` without a button of the user ([ADR 0245](docs/adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md) item 14).
- The page of a person follows [ADR 0116](docs/adr/0116-the-page-of-a-person-has-save-and-discard.md), [ADR 0117](docs/adr/0117-the-page-of-a-person-finds-values-on-musicindex.md), [ADR 0118](docs/adr/0118-member-of-offers-a-credit-on-the-artist-feed.md), [ADR 0132](docs/adr/0132-a-person-can-be-part-of-many-artists-and-labels.md), [ADR 0139](docs/adr/0139-the-default-address-is-on-its-row.md), [ADR 0140](docs/adr/0140-a-credit-name-shows-where-it-is-used.md), [ADR 0143](docs/adr/0143-a-feed-row-names-the-tracks-of-a-person.md), [ADR 0146](docs/adr/0146-link-words-already-marks-and-credit-edits.md), [ADR 0159](docs/adr/0159-a-person-can-link-to-a-page-on-music-index.md), [ADR 0163](docs/adr/0163-one-sheet-finds-and-links-a-page-on-music-index.md), [ADR 0194](docs/adr/0194-the-nickname-saves-at-once-and-the-update-sheet-selects-each-value.md), [ADR 0196](docs/adr/0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md) and [ADR 0203](docs/adr/0203-people.md).
- The sheets that add a value have one layout ([ADR 0145](docs/adr/0145-each-sheet-that-adds-a-value-has-one-layout.md)). Their parts follow [ADR 0137](docs/adr/0137-the-people-list-fills-nostr-keys-and-open-entries.md), [ADR 0141](docs/adr/0141-the-form-of-add-a-wallet-comes-first.md), [ADR 0144](docs/adr/0144-recipients-use-one-picker-and-sort-by-split.md), [ADR 0147](docs/adr/0147-the-nostr-and-artwork-rows-show-a-preview.md) and [ADR 0112](docs/adr/0112-a-credit-role-comes-from-a-list.md).
- Text names the product "Music Index". Code names, routes and domains keep their form ([ADR 0152](docs/adr/0152-text-names-the-product-music-index.md), musicindex ADR 0085 R-WORD-8).
- A search field for musicindex reads the shared query text, and the UI has no parser ([ADR 0151](docs/adr/0151-the-search-fields-read-the-shared-query-text.md), [ADR 0154](docs/adr/0154-the-search-fields-have-a-field-list.md), [ADR 0160](docs/adr/0160-the-field-list-shows-the-placeholder-of-a-text-field.md)).
- A release is an album or a playlist ([ADR 0086](docs/adr/0086-a-playlist-links-to-its-artist-or-label.md)).
- The Welcome page, the first steps and the Add pages follow [ADR 0260](docs/adr/0260-the-welcome-page-and-the-first-steps.md), [ADR 0122](docs/adr/0122-a-new-user-starts-with-a-default-file-host.md), [ADR 0072](docs/adr/0072-add-a-feed-opens-from-releases.md) and [ADR 0212](docs/adr/0212-the-add-pages.md).
- The outline, the page tabs and the field rows follow [ADR 0069](docs/adr/0069-the-outline-head-opens-the-feed-page.md), [ADR 0157](docs/adr/0157-the-page-tabs-are-a-segmented-control.md), [ADR 0197](docs/adr/0197-the-tabs-and-the-field-rows-follow-the-editor-column.md), [ADR 0239](docs/adr/0239-the-panel-holds-the-content-and-the-page-has-tabs.md) and [ADR 0254](docs/adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md).
- `pages.js` owns the sequence of each editor page ([ADR 0258](docs/adr/0258-the-shell-and-the-editor-pages.md) items 30 to 41, [ADR 0071](docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)).
- Each artwork field is only in the Artwork sheet ([ADR 0070](docs/adr/0070-a-cover-opens-the-artwork-sheet.md)).
- A read-only feed opens in the editor, and its write needs a host first ([ADR 0067](docs/adr/0067-a-read-only-feed-opens-in-the-editor.md)).
- On the server, each `/api/` request needs a session cookie and the `X-Studio-CSRF` header ([ADR 0204](docs/adr/0204-feed-files-moves-and-the-server.md) items 36 to 39).
- Vault lock time, recent unlock and sign-in limits follow [ADR 0142](docs/adr/0142-a-user-selects-the-time-before-the-vault-locks.md) and [ADR 0106](docs/adr/0106-a-sensitive-account-change-needs-a-recent-unlock.md).
- With the network policy `Public`, no outbound connection goes to a private, loopback or reserved address ([ADR 0057](docs/adr/0057-the-private-network-guard-of-the-server.md)).
- A granted write uses only the secrets of its grant vault ([ADR 0058](docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md)).
- A connected app pairs with a one-time text, and apps and the API tests use the crate `studio-app-client` ([ADR 0110](docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md)).
- A server with `--hold-assets off` keeps an asset only until its write ([ADR 0105](docs/adr/0105-a-public-server-holds-no-assets.md)).
- An account archive holds the vault files as they are, and an import changes no feed ([ADR 0262](docs/adr/0262-a-user-can-export-and-import-an-account.md)).
- The People data stays plain at rest and holds no secret ([ADR 0108](docs/adr/0108-people-data-at-rest-stays-plain.md)). A new field of a person states that it holds no secret.
- Each shell process is not dumpable, and the data folders and vault files have private modes ([ADR 0104](docs/adr/0104-key-material-stays-off-the-disk.md), [ADR 0235](docs/adr/0235-the-desktop-app-can-open-the-vault-from-the-keyring-of-the-computer.md) item 15).
- No server image comes from a commit that did not pass CI ([ADR 0066](docs/adr/0066-the-server-ships-as-a-container-image.md)).
- Studio writes a Nostr key only in the two forms of [ADR 0074](docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md), and only after a selection by the user.
- A comparison of a feed address with a file address uses the plain address, with no OP3 prefix ([ADR 0076](docs/adr/0076-an-op3-switch-on-feed-settings.md)).
- Web addresses, encoded spaces and short addresses follow [ADR 0023](docs/adr/0023-web-addresses-and-audio-file-facts.md) and [ADR 0245](docs/adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md).
- Track order, disc, dates, duration and audio facts follow [ADR 0081](docs/adr/0081-a-position-menu-sets-the-sequence-of-a-list.md), [ADR 0080](docs/adr/0080-the-season-of-a-track-is-an-experimental-disc-setting.md), [ADR 0093](docs/adr/0093-dates-and-times-use-a-picker-in-local-time.md), [ADR 0094](docs/adr/0094-a-duration-is-edited-as-minutes-and-seconds.md) and [ADR 0130](docs/adr/0130-the-facts-of-an-audio-file-stay-locked.md).
- A changed value keeps the escape form of its source ([ADR 0115](docs/adr/0115-a-changed-value-keeps-the-escape-form-of-its-source.md)).
- The UI scale, the announcement choice and the file drop follow [ADR 0134](docs/adr/0134-a-ui-scale-and-larger-release-tiles.md), [ADR 0136](docs/adr/0136-one-announcement-choice-at-the-write.md) and [ADR 0125](docs/adr/0125-each-file-target-shows-that-it-takes-a-drop.md).
- The desktop app is a Tauri window on the local API, with no Tauri command for the page ([ADR 0124](docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md)). Its collections follow [ADR 0149](docs/adr/0149-the-desktop-app-has-many-collections.md).
- The core refuses a write with a bad move link, and the user has no override ([ADR 0126](docs/adr/0126-studio-refuses-a-bad-move-link.md)). A moved feed names its own address ([ADR 0128](docs/adr/0128-a-moved-feed-names-its-own-address.md)).
- A check for a change from outside writes nothing and needs no vault ([ADR 0259](docs/adr/0259-studio-checks-each-feed-and-the-left-panel-marks-its-state.md)). The public address comparison follows [ADR 0021](docs/adr/0021-public-address-of-a-feed.md) item 4.
- Each screenshot walk checks the fit at 390 px and 761 px (musicindex ADR 0065, [ADR 0214](docs/adr/0214-each-sheet-fits-the-window-and-the-walk-opens-it.md)). A new sheet goes into `tools/sheets.json` in the commit that adds it, because no check finds a missing entry.
- Do not copy code from `../mi-studio` ([ADR 0001](docs/adr/0001-one-core-with-desktop-and-server-shells.md)).

Documentation:

- `README.md` is a landing page for an artist ([ADR 0188](docs/adr/0188-the-readme-is-a-landing-page-for-an-artist.md), [ADR 0237](docs/adr/0237-the-readme-shows-a-few-screenshots-in-the-dark-theme.md)). Only `docs/user-guide/README.md` lists the guide pages.
- `docs/README.md` is the index of the documents and the current ADRs. Add each new document to it in the same change.

## Reference Material

- Research: `../musicindex-research`. It is private. Do not copy text that has a private notice.
- Visual direction: `../musicindex`, musicindex ADR 0067 items 19 to 49, and `search.html`.
- Shared interface rules: musicindex ADR 0021, ADR 0022, ADR 0026, ADR 0028, ADR 0068, ADR 0080, ADR 0085, ADR 0087 and ADR 0093. Shown words: `../musicindex/docs/glossary.md`. Shared assets and tokens: `../musicindex-assets` (musicindex ADR 0022).
- Feed API: Stophammer, through `../musicindex/api.json`.
- MSP 2.0: `../MSP-2.0`.
