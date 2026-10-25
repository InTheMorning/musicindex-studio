# Studio v1 Task 025c: The Publisher Page

Parent: [task 025](studio-v1-task-025-publisher-feeds.md). Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) items 1 to 6, 8, 10, 11 and 13, [ADR 0037](../adr/0037-button-roles.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md).
Sequence: the third packet of task 025. It depends on 025b.

## Goal

The user makes a publisher feed in "Add feeds", and edits it on its own page: the fields, the role and the list of releases.

## Files

Inspect: `crates/api/ui/app.js` (`isEditable`, `EditableFeed`, `FeedTree`, `AddFeeds`, `NewReleaseForm`, `Shell`, `mediumKind`), `crates/api/ui/editor.js` (`AlbumEditor`, `AlbumPage`, `ReplaceAudio` for the file button, `EntryActions`, `FilesPage`), `crates/api/ui/draft.js`, `crates/api/ui/styles.css`, `crates/api/src/tests.rs` (the UI tests: `each_details_element_is_a_named_entry_or_a_section`, `each_quiet_button_is_an_icon_button`, `each_control_height_uses_the_target_token`).

Change: `app.js`, `editor.js`, `styles.css` when a new rule is necessary.

Do not change: `crates/core/`, `crates/api/src/`, `crates/api/ui/values.js`, `crates/api/ui/draft.js`, the album page and the track page.

## Architecture

These choices are fixed. Do not change them.

### The Editable Publisher Feed

- `isEditable` also gives true for a feed with the medium `publisher` and a connector.
- `AlbumEditor` shows the new `PublisherPage` for a publisher feed on the page `album`, and the Files page on the page `files`. A publisher feed has no other page. The shell gives `AlbumEditor` the feed list and a function that loads the feed list again.
- `FeedTree` shows two pages for an open publisher feed: "Publisher" and "Files". It shows no tracks, no "Add a track" and no "Import files" for it.

### The Publisher Page

A new component `PublisherPage` in `editor.js`, with these sections in this sequence:

1. "Publisher", one group: Title (`title`), Name (`author`, with the hint "The itunes:author of the feed."), Description, the cover image address with its preview, and Website (`link`). Below the image field, a file button "Import Image", with the pattern of `ReplaceAudio`. It sends one image to the upload route with no `replace`, and puts the address of the result into `image`. A result that is not an image goes out of the library with the delete route, and the field shows "Select an image file.".
2. "Role", one group: a `select` with "Artist" and "Label". Its value is `publisher_role` of the feed row. With no recorded role, the select shows the role that ADR 0041 item 4 gives from the draft model. It then shows the hint "Studio found this role in the feed." With no role from the two, the select shows "Select a role". A change calls `PUT /feeds/{id}/role`.
3. "Releases", one group: one row for each channel `remote_items` entry with `medium` `music`, in model sequence. Each row shows the title, or the `feed_url` when it has no title. Its second line is "In the collection" or "Not in the collection".

   The row has "Move up" and "Move down" with the normal role, and "Remove" with the destructive role (ADR 0037). Below the rows is a `select` of the candidate releases with the action button "Add the Release". With no row, the group shows "No releases yet.".

### The Releases

- A row matches a feed of the feed list in two cases. The `podcast_guid` of the feed is the `feed_guid` of the entry, or the `public_url` of the feed is the `feed_url` of the entry.
- A candidate is a feed with the medium `music`, a connector, and no matching row.
- "Move up" and "Move down" change the sequence of `remote_items` in the model, as `EntryActions` does. They change no other feed (ADR 0041 item 10).
- "Remove" of a matched row calls `DELETE /feeds/{id}/links/{release}`. "Remove" of a row with no match takes the entry out of the model only (ADR 0041 item 13).
- "Add the Release" calls `POST /feeds/{id}/links`.

### A Change Of Two Drafts

Each call of a link route or of the role route does these steps:

1. `draft.flush()`. Stop when it gives false.
2. The call.
3. `draft.load()`, and the load of the feed list.
4. A notice with `role="status"` for each feed of `changed`, but not for the open feed: "Studio also changed the draft of {title}. Open it to examine and write it." Each notice has a normal button "Open {title}" that opens that feed (ADR 0041 item 8).

An error of a call shows in the section of the call, with the `ErrorNotice` of the page. For `role_required`, the text is "Select the role of this publisher feed first."

### Add Feeds

`AddFeeds` gets a panel "Make a publisher feed" after "Make a new release", when the user has a host. The form has the host (as `NewReleaseForm` does), the feed file with the placeholder `my-label.xml` and the name. It has the role as two radio buttons, "Artist" and "Label", in a `fieldset`. The submit button "Make the Publisher Feed" has the action role. It calls `POST /feeds/new` with `medium` `publisher`, the name in `title` and in `author`, and `role`. Then it opens the new feed.

### Labels

Each new button label is in title case (musicindex ADR 0023 R-CASE-1).

## Steps

1. Make a publisher feed editable, and give it its tree pages.
2. Add `PublisherPage`, with its sections and its calls.
3. Add the panel of "Add feeds".
4. Run the `drive-ui.mjs` check below, and give the screenshots to the planning model.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step and no new UI file ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- The page uses the three background levels of [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), and the existing classes for a group and a row.
- All functions work from the keyboard.

## Acceptance Criteria

Mechanical:

- The UI tests of `crates/api/src/tests.rs` pass.
- The three commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`:

- "Add feeds" shows "Make a publisher feed", and the form makes a label feed that opens on its page.
- The tree of a publisher feed shows "Publisher" and "Files" only.
- The publisher page shows its three sections. "Add the Release" adds a release, and the notice names the release. The release opens from the notice, and its review shows its `podcast:publisher`.
- A change of the role gives the notice for each linked release.
- "Remove" takes the release out, and the notice names it.
- The page in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run Studio with a data folder in `.studio-dev/`, as `AGENTS.md` says. Drive the UI with `tools/drive-ui.mjs`. When Node.js is not on the PATH, run the script with Bun.

## Escalation

Stop and report when a step needs a change of `crates/api/src/` or of the core.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-025c-publisher-page.md
- docs/adr/0041-publisher-feeds-and-two-way-links.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The user makes a publisher feed in "Add feeds", and edits it on its own page: the fields, the role and the list of releases.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step and no new UI file. All functions work from the keyboard.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core/, crates/api/src/, crates/api/ui/values.js, crates/api/ui/draft.js, the album page and the track page.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. Give the screenshots of the visual criteria to the planning model.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
