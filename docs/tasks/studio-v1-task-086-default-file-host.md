# Studio v1 Task 086: The Default File Host And The Placeholder GUID

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The First Visit".
Decision: [ADR 0122](../adr/0122-a-new-user-starts-with-a-default-file-host.md). It supersedes ADR 0121, and replaces task 085.

Status: Complete, with the screenshot visual check of the desktop and of the server. The human test is open.

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

### Files

Inspect: `crates/core/src/connector.rs` (`HostEnv`, `from_row_in`, `LocalFolder`), `crates/api/src/workspace.rs` (`data_dir`, `rules`, `connector_of`, `hosts_for`, `private_dir`), `crates/api/src/routes.rs` (`NewConnector`, `add_connector`, `not_on_server`, `list_feeds`, `write_feed`, `ensure_draft`, the update of a connector), `crates/api/src/uploads.rs` (`sweep_pending`, `spawn_cleaner`), `crates/core/src/pipeline.rs` (`start_draft`, `set_feed_host`), `crates/core/src/account_archive.rs` (the import of the connectors), `crates/api/src/tests.rs`, `crates/api/ui/editor.js` (the notice "The file host has no file for this feed yet"), `crates/api/ui/app.js` (the File hosts page and its rows), `crates/api/ui/welcome.js` (`HOST_STEP.expired`), `crates/api/ui-tests/`, `docs/user-guide/first-steps.md`.

Change: those files, and a new module `crates/api/src/temporary.rs`.

Do not change: the collection schema, the ADRs, `crates/server` other than a test.

### Architecture

These choices are fixed. Do not change them.

1. The core gets `connector::TEMPORARY_KIND = "temporary"` and the field `HostEnv::temporary_root: Option<PathBuf>`. `from_row_in` makes a `LocalFolder` at `temporary_root` with no public address for this kind. With no `temporary_root`, it gives a connector error. The connector config holds no path: it is `{}` or `{"expired_at": SECONDS}`. Thus an account archive cannot point the area at a different folder.
2. The workspace sets `temporary_root` to `data_dir/temporary` on a server (`rules` is set), and to `None` on the desktop. It makes the folder with `private_dir` (mode 0700) before the first use.
3. `NewConnector` gets the variant `Temporary {}`. On the desktop, the route gives 403 `not_on_desktop` (ADR 0122 non-goal). When the account has a temporary area, the route gives 409 `temporary_exists`. The name of the connector is "Temporary area".
4. `temporary::sweep(w, now)` runs in the hourly loop of `spawn_cleaner`, after `sweep_pending`, with its own error handling. For each temporary area with no `expired_at`: the last write is the newest modification time of a file in the folder. When the folder has a file and the last write is 7 days or more before `now`:
    - First, each written feed of the area with no draft gets a draft from its live file, through `pipeline::start_draft`. Thus the feed keeps its content.
    - Then the function removes each file and each subfolder in the folder. The folder stays.
    - Then it sets `expired_at` in the connector config.
    - It does not read or touch the vault (ADR 0055 item 16).
5. `GET /feeds` gives `temporary_expired: true` for a feed whose connector, or whose draft feed move source, is an expired temporary area.
6. `write_feed` gives 409 `temporary_expired` for a feed whose target file host is an expired temporary area. A move to a different file host, and its write, work. The removal of an expired area works. A user can then add a new area.
7. The update route of a connector refuses a temporary area, as it refuses each kind with no form.
8. The UI: the File hosts page shows a temporary area with its name and the text `HOST_STEP.temporary`, and with no edit form. The page of a feed with `temporary_expired` shows the notice `HOST_STEP.expired` in the place of the notice "The file host has no file for this feed yet". `docs/user-guide/first-steps.md` tells the temporary area of a server.

### Acceptance Criteria

Mechanical:

- An API test with server rules: `POST /connectors {kind: "temporary"}` makes the connector, its folder is `data_dir/temporary` with mode 0700, and its config holds no path. A second one gives 409 `temporary_exists`.
- An API test on the desktop: the same request gives 403 `not_on_desktop`.
- A test: a new album on the area gets a placeholder GUID, and its write puts the file in the folder.
- A test of `temporary::sweep`: with a file 6 days old, nothing changes. With the newest file 7 days old, the written feed gets a draft, the folder is empty, `expired_at` is set, and `GET /feeds` gives `temporary_expired`. A write then gives 409 `temporary_expired`. Use `File::set_modified` for the times.
- A test: a feed of an expired area moves to a local folder with a public address, and its write works. It gets its real GUID.
- A core test: `from_row_in` for the kind `temporary` with no `temporary_root` gives an error, and a config with a path does not change the folder.
- A Node test: the File hosts row of a temporary area and the expired notice use the texts of ADR 0122 items 3 and 10.
- The four commit checks pass.

Visual, by the planning model: the server part of the visual check of ADR 0122.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0122-a-new-user-starts-with-a-default-file-host.md
- docs/tasks/studio-v1-task-086-default-file-host.md (the Goal, the review of 086a and 086b, and packet 086c)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 086c names.

Goal:
- A server user can try Studio in a temporary area in the user folder. The server removes its files 7 days after the last write, and each feed keeps its draft and can move to a real file host.

Constraints:
- Follow the Architecture section of packet 086c fully.
- A write to a host occurs only through `pipeline::write_draft`.
- No feed rule in the UI (ADR 0065). Each button label is in title case. Each `setState` that combines the old value uses a functional update.
- Each comment, each test message and the user guide page are in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Clippy runs with `-D warnings`, which includes `too_many_lines` for a test.
- The UI has no build step. To check syntax, copy a changed UI file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.

Do not touch:
- The collection schema, the ADRs, `crates/server` other than a test.
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 086c.

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

### Review Of 086c

The planning model found these items in the operator's test and in the visual check:

- The route that adds a folder did not make the default folder, so a new computer gave "Studio did not find this folder on this computer." `add_connector` now makes the folder when its path is the `default_host_root`. It makes no other folder. The visual check ran the route with a home folder in `.studio-dev`.
- In Feed facts, the explanation of the placeholder had no term, so it went into the first column of the grid. At 390 px this pushed the values off the screen. The explanation now stays in the value column, and a long value wraps.
- A feed of an expired area that moves needs a write with no old file. The core skips the conflict check, the snapshot and the pointer only when the old file host is an expired area and the file is gone.
- Open: a write under a show grant does not check the expiry of a temporary area. Only `write_feed` refuses it.
