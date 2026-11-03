# Studio v1 Task 086: The Default File Host And The Placeholder GUID

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The First Visit".
Decision: [ADR 0122](../adr/0122-a-new-user-starts-with-a-default-file-host.md). It supersedes ADR 0121, and replaces task 085.

Status: 086a and 086b are complete, with the desktop part of the visual check. 086c is next.

## Goal

A new user must pick a file host in step 2. The default works at once: a folder on this computer, or a temporary area on a server. A feed on a file host with no public address gets a placeholder GUID with an explanation. The feed gets its real GUID when it first has a public address. The practice parts of ADR 0121 go.

## Packets

- 086a changes the core and the API. 086b changes the UI. They share no file, and they run at the same time. The planning model commits 086a, 086b and the documents of ADR 0122 together, as one change of direction.
- 086c adds the temporary area of the server, after the commit of 086a and 086b.

Visual check of 086b, by the planning model, before the commit: the desktop part of the visual check of ADR 0122. Visual check of 086c: the server part.

Human test, by the operator, after the deploy: the human test of ADR 0122.

## Packet 086a: The Placeholder GUID In The Core

### Files

Inspect: `crates/core/src/pipeline.rs` (`is_practice`, `is_local_folder`, `new_feed`, `copy_feed`, `write_draft_signed`, `set_feed_host`, `publish_practice`, `plan_practice`, `practice_draft`, `relink`, `swap_op3_prefixes`, `PipelineErrorKind::PracticePage`), `crates/core/src/collection.rs` (`move_practice_pages`, `PracticeMove`, the update of a connector config), `crates/core/src/publisher.rs` (`check_named`), `crates/core/tests/practice.rs`, `crates/core/tests/pipeline.rs`, `crates/api/src/routes.rs` (the `practice` flag of `GET /feeds`, `publish_practice`, the route that changes a connector, `GET /library`), `crates/api/src/error.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/core/src/library.rs` (`default_root`).

Change: those files, and a new test file `crates/core/tests/placeholder_guid.rs` in the place of `crates/core/tests/practice.rs`.

Do not change: the UI, the documents, `crates/server`, the collection schema.

### Architecture

These choices are fixed. Do not change them.

1. `new_feed` and `copy_feed` on a file host with no public address give a feed with no `self` link. Its GUID is `pending-` and a random UUID v4. This applies to each connector kind. The `NoPublicAddress` error of `new_feed` and `copy_feed` goes.
2. `pipeline::has_placeholder_guid(guid: &str) -> bool` is true for a GUID that starts with `pending-`.
3. The write does not refuse a feed with a placeholder GUID. `PipelineErrorKind::PracticePage`, `is_practice`, `publish_practice`, `move_practice_pages` and the route `POST /practice/publish` go. Delete them. Do not keep dead code.
4. `pipeline::assign_real_guids(collection, hosts, ids: &[FeedId]) -> Result<Vec<FeedId>, PipelineError>`: it acts on each feed of `ids` that has a placeholder GUID in its draft, when its file host now gives a public address. It sets the GUID to the UUIDv5 of that address, and the `self` link to that address. In the same transaction, it changes each `feedGuid` that named an old placeholder, and its `feedUrl`, in each draft of the collection. It also changes the OP3 prefixes, with `swap_op3_prefixes`. It gives the IDs of the changed drafts.
5. `set_feed_host` calls `assign_real_guids` for the moved feed, after it sets the new file host of the draft. A change of the public address of a connector, through the API, calls it for each feed of that connector.
6. `GET /feeds` gives `placeholder_guid: true` for a feed with a placeholder GUID, in the place of `practice`.
7. `GET /library` gives `default_host_root` on the desktop: the folder `pages` in the parent folder of `library::default_root()`. The API makes the folder when the UI adds the connector, as the folder form does now.

### Acceptance Criteria

Mechanical:

- A core test: two new feeds on a local folder with no public address get two different `pending-` GUIDs, and no `self` link.
- A core test: a write of such a feed puts the file, and queues no announcement.
- A core test: a move of an artist page and its album to a file host with a public address sets the real GUIDs. It also changes the links in the two drafts.
- A core test: a new public address of the local folder sets the real GUID of each feed with a placeholder.
- An API test: `GET /feeds` gives `placeholder_guid`, the route `POST /practice/publish` is gone, and `GET /library` gives `default_host_root` on the desktop.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0122-a-new-user-starts-with-a-default-file-host.md
- docs/tasks/studio-v1-task-086-default-file-host.md (the Goal and packet 086a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 086a names.

Goal:
- A feed on a file host with no public address gets a placeholder GUID. It gets its real GUID when it first has a public address.
- The practice code goes.

Constraints:
- Follow the Architecture section of packet 086a fully.
- A write to a host occurs only through `pipeline::write_draft`.
- A read and a write with no edit give the same bytes. The corpus tests enforce this.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Clippy runs with `-D warnings`, which includes `too_many_lines` for a test.

Do not touch:
- The UI, the documents, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 086a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs (a UI agent works at the same time; report a failure there, and do not fix the UI)

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 086b: The Required Step 2 And The Explanation

### Files

Inspect: `crates/api/ui/welcome.js` (`HOST_ROWS`, `hostStep`, `PRACTICE`, `publishTargets`), `crates/api/ui/app.js` (`StartSheet`, `AddHostForm`, `LocalHostForm`, `PublishForRealSheet`, the practice badge and its uses), `crates/api/ui/editor.js` (`DraftToolbar` and its practice branch, `FeedFacts`), `crates/api/ui/shell.js` (`practiceFlag`), `crates/api/ui/styles.css` (`.badge-practice`), `crates/api/ui-tests/welcome.test.mjs`, `crates/api/ui-tests/shell.test.mjs`, `docs/user-guide/first-steps.md`.

Change: those files.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. Remove the practice parts: `PRACTICE`, `publishTargets`, `PublishForRealSheet`, the badge, `.badge-practice`, `practiceFlag`, and the practice branch of `DraftToolbar`. Delete them. Do not keep dead code.
2. `hostStep` goes. Step 2 always shows. Its rows, in this sequence, with the first row selected at the start:
    - Desktop: "Folder on this computer" (`local`), "Nostr site (nsite)" (`nsite`), "S3 or Cloudflare R2" (`s3`), "SFTP server" (`sftp`).
    - Server: "Try it here (temporary)" (`temporary`), with the text of ADR 0122 item 3, then "Nostr site (nsite)", "S3 or Cloudflare R2" and "SFTP server".
    - With a connected file host, a select menu of the connected file hosts is above the rows, and it is selected at the start.
3. On the desktop, the folder form of the default row starts with `default_host_root` of `GET /library`. "Continue" adds the folder and makes the page. The public address stays optional.
4. On a server, the row "Try it here (temporary)" sends the connector kind `temporary` with no other field. Packet 086c adds that kind on the server. Until then, the API gives an error for it, and the sheet shows the error.
5. Feed facts shows, for a feed with `placeholder_guid`, the placeholder and the text of ADR 0122 item 6.
6. `docs/user-guide/first-steps.md` tells the default folder, the placeholder GUID and the move to a real file host. It no longer tells practice pages.

### Acceptance Criteria

Mechanical:

- A Node test: the rows of step 2 on the desktop and on a server, the first row selected, and the select menu with a connected file host.
- A Node test: the text of ADR 0122 items 3 and 6.
- A Node test: no UI file has the word "Practice" or "Publish for Real".
- The test of title case and the test of the page rule still pass.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0122-a-new-user-starts-with-a-default-file-host.md
- docs/tasks/studio-v1-task-086-default-file-host.md (the Goal and packet 086b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 086b names.

Goal:
- Step 2 is required and has a ready default. A feed with a placeholder GUID explains it. The practice UI goes.

Constraints:
- Follow the Architecture section of packet 086b fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- Each `setState` that combines the old value uses a functional update.
- Each comment and the user guide page are in ASD-STE100 Simplified Technical English.
- The UI has no build step. To check syntax, copy a changed file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.
- A core agent works at the same time on `crates/core` and `crates/api/src`. The API field `practice` becomes `placeholder_guid`, and `GET /library` gets `default_host_root`. Do not touch those folders.

Do not touch:
- The core, the API, the ADRs.
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 086b.

Test commands:
- tools/node --test crates/api/ui-tests/*.test.mjs
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace (this can fail while the core agent works; report it, and do not fix the core)

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Review Of 086a And 086b

The planning model made these changes before the commit:

- A write removes the draft, so a written feed with a placeholder GUID has no draft. `assign_real_guids` reads the live file of such a feed from its open file host. It saves a draft only when the GUID or a link of the feed changes. A core test writes an artist page and an album, then moves the album. The artist page then has a draft with the new link.
- `GET /feeds` gives the GUID of the draft in `podcast_guid` when the feed has a draft. Before its first write, a new feed has no GUID in its row, and Feed facts must show the placeholder (ADR 0122 item 6).
- With a connected file host, the select menu is selected at the start, and no row is selected. A selected row hides the button of the menu. Thus step 2 shows one primary button at a time.

## Packet 086c: The Temporary Area Of A Server

The planning model writes this packet after the commit of 086a and 086b. It uses `sweep_pending` of `crates/api/src/uploads.rs` for the hourly deletion, and the user folder of ADR 0055.
