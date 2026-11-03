# Studio v1 Task 085: Practice Pages

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The First Visit".
Decision: [ADR 0121](../adr/0121-a-folder-with-no-public-address-holds-practice-pages.md).

Status: Complete. Packets 085a and 085b are committed. The screenshot visual check passed on 2026-11-02. The human test is open.

## Goal

A user can try Studio in a folder with no public address. The pages there are practice pages: Studio makes them and edits them, but writes nothing. "Publish for Real" moves all practice pages of the folder to a file host with a public address. Each page gets its real GUID, and the links between them change.

## Packets

The planning model examines each packet before its commit. 085a changes the core and the API. 085b changes the UI.

Visual check of 085b, by the planning model, before its commit: the visual check of ADR 0121.

Human test, by the operator, after the deploy: the human test of ADR 0121.

## Packet 085a: Practice Pages In The Core

### Files

Inspect: `crates/core/src/pipeline.rs` (`new_feed`, `write_draft`, `set_feed_host`, `writable`, `edit_draft`, `current_draft`, `PipelineErrorKind`), `crates/core/src/guid.rs` (`podcast_guid_for_url`), `crates/core/src/publisher.rs` (`link` and the code of the `feedGuid` and the `feedUrl` of a link), `crates/core/src/feed.rs` (`set_self_url`, the `podcast:publisher` and `remoteItem` model), `crates/core/src/collection.rs` (the feed row, `connector_id`, `path`, `public_url`, the draft table), `crates/core/src/connector.rs` (`public_url`), `crates/api/src/routes.rs` (`add_feed`, `make_feed` of `POST /feeds/new`, the feed row JSON of `GET /feeds`, `write_feed`), `crates/api/src/error.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Change: `crates/core/src/pipeline.rs`, `crates/api/src/routes.rs`, `crates/api/src/error.rs`, `crates/api/src/lib.rs`, and the core and API tests. Change `crates/core/src/publisher.rs` only if item 5 needs a function there. Change `crates/core/src/collection.rs` only for a new update of the connector and the path of a feed.

Do not change: the UI, the ADRs, `crates/server`, the move of ADR 0038 for a written feed, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `pipeline::is_practice(hosts, row) -> bool`: true when the feed has a connector and the connector gives no `public_url` for its path.
2. `new_feed` gives no `NoPublicAddress` error. With no public address, the GUID is `podcast_guid_for_url(&format!("studio-practice://{}/{}", connector_id.0, path))`, and the draft has no `self` link. The `FileExists` check does not change.
3. `write_draft` and `write_draft_signed` give the new kind `PipelineErrorKind::PracticePage` for a practice page, before each other step. The API gives 409 `practice_page`.
4. A link of `publisher.rs` to a practice page writes the temporary GUID in `feedGuid` and no `feedUrl`.
5. `pipeline::publish_practice(collection, hosts, from: ConnectorId, to: ConnectorId) -> Result<Vec<FeedId>, PipelineError>`:
    - It refuses a `to` with no public address (`NoPublicAddress`), and a `from` that is not a file host with practice pages.
    - It collects each practice page of `from`. For each one, a host file of `to` at the same path stops it with `FileExists` and no change.
    - In one transaction, it sets the connector of each page to `to`. It sets the `podcast:guid` of each draft to the UUIDv5 of the new public address, and the `self` link to that address.
    - In the same transaction, it changes each `remoteItem` in the drafts of these pages that has an old temporary GUID of the operation in `feedGuid`. The new `feedGuid` is the new GUID, and the new `feedUrl` is the new public address.
    - It keeps the release folder and the library files of each page. It writes no file to a host.
6. `GET /feeds` gives `practice: true` for a practice page. `POST /api/v1/practice/publish` takes `{ from, to }`, and gives `{ feed_ids }`. The route is in the list of the token test.

### Acceptance Criteria

Mechanical:

- A core test: a new feed on a local folder with no public address has the temporary GUID and no `self` link.
- A core test: `write_draft` of a practice page gives `PracticePage`, and the folder has no file.
- A core test: `publish_practice` of an artist page and an album that link to each other gives each the real GUID and the `self` link. It also changes the link in each draft.
- A core test: a file at the same path on `to` stops `publish_practice`, and no draft changes.
- An API test: `GET /feeds` gives `practice`, `POST /practice/publish` moves the pages, and a write of a practice page gives 409 `practice_page`.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0121-a-folder-with-no-public-address-holds-practice-pages.md
- docs/tasks/studio-v1-task-085-practice-pages.md (the Goal and packet 085a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 085a names.

Goal:
- The core makes practice pages on a file host with no public address, and refuses their write.
- The core publishes them for real on a file host with a public address.

Constraints:
- Follow the Architecture section of packet 085a fully.
- A write to a host occurs only through `pipeline::write_draft`. `publish_practice` writes no file.
- A read and a write with no edit give the same bytes. The corpus tests enforce this.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Clippy runs with `-D warnings`, which includes `too_many_lines` for a test. Put a long setup in a helper function.

Do not touch:
- The UI, the ADRs, `crates/server`, the move of ADR 0038, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 085a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 085b: The Practice Badge And "Publish For Real"

### Files

Inspect: `crates/api/ui/app.js` (`StartSheet`, `AddHostForm`, `LocalHostForm`, the Releases rows, the left panel rows), `crates/api/ui/editor.js` (`DraftToolbar`), `crates/api/ui/welcome.js` (`HOST_ROWS`), `crates/api/ui/shell.js`, `crates/api/ui/release-list.js` or the module that gives the Releases rows, `crates/api/ui-tests/welcome.test.mjs`, `docs/user-guide/first-steps.md`.

Change: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/welcome.js`, `crates/api/ui/styles.css` only for the badge, `crates/api/ui-tests/welcome.test.mjs`, `docs/user-guide/first-steps.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `welcome.js` gets `PRACTICE` texts: the badge "Practice", the toolbar button "Publish for Real", the folder hint of ADR 0121 item 8, and the sheet texts of item 3 below.
2. A feed with `practice: true` shows the badge "Practice" after its type line in Releases and in the left panel. The badge uses an existing badge class, or one new class with no new color.
3. `DraftToolbar` of a practice page shows the badge and "Publish for Real" (`button primary`) in the place of "Review". "Versions" does not show. The button opens the sheet "Publish for Real". It has the text "Studio moves each practice page of this folder to the file host that you select, and gives each page its real address. Then you review and write each page." It shows the host rows of the Welcome step 2. It also shows a select menu of each connected file host with a public address, when there is one or more. "Move the Pages" sends `POST /practice/publish`, and the draft reads again.
4. In the Welcome step 2, the folder form of "My own server" keeps the public address optional, with the hint of ADR 0121 item 8.
5. `docs/user-guide/first-steps.md` tells the practice pages and "Publish for Real".

### Acceptance Criteria

Mechanical:

- A Node test: the `PRACTICE` texts, with title case on the two button labels.
- A Node test: the model that gives the target file hosts of the sheet keeps only a file host with a public address.
- The test of title case and the test of the page rule still pass.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0121-a-folder-with-no-public-address-holds-practice-pages.md
- docs/tasks/studio-v1-task-085-practice-pages.md (the Goal and packet 085b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 085b names.

Goal:
- A practice page shows its badge, and "Publish for Real" moves the practice pages to a real file host.

Constraints:
- Follow the Architecture section of packet 085b fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- Each `setState` that combines the old value uses a functional update.
- Each comment and the user guide page are in ASD-STE100 Simplified Technical English.
- The UI has no build step. To check syntax, copy a changed file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.

Do not touch:
- The core, the API, the ADRs.
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 085b.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
