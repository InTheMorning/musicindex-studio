# Studio v1 Task 049a: The Add Feeds Page And One Form To Make A Feed

Parent: [task 049](studio-v1-task-049-add-pages.md). Decision: [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md) items 1 to 10.

Status: Open.

## Goal

The Add feeds page shows two groups of rows. Each row opens a sheet with its form. One form makes each of the four new feed types, and it gives a start value for "Feed file".

## Files

Inspect: `crates/api/ui/app.js` (`AddFeeds`, `NewReleaseForm`, `NewPublisherForm`, `NewPlaylistForm`, `FindFeeds`, `FoundFeeds`, `AddFeedForm`, `MspImport`, `Field`, `ChoiceSelect`, `useAction`, `splitError`), `crates/api/ui/editor.js` (`useSheet`, `AddTrackSheet` for the sheet markup, the "Properties" `details` near line 198), `crates/api/ui/audio.js` (a model module example), `crates/api/ui/styles.css` (`.sheet`, `.group`, `.link-row`, `.disclosure`), `crates/api/ui-tests/audio.test.mjs`, the guide pages below.

Change: `crates/api/ui/addfeeds.js` (new), `crates/api/ui-tests/addfeeds.test.mjs` (new), `crates/api/ui/app.js`, `docs/user-guide/add-feeds.md`, `docs/user-guide/new-release.md`, `docs/user-guide/artist-and-label-feeds.md`, `docs/user-guide/playlists.md`, `docs/user-guide/live-shows.md`.

Do not change: `crates/api/ui/editor.js`, `crates/api/ui/shell.js`, `crates/api/ui/pages.js`, the forms `FindFeeds`, `FoundFeeds`, `AddFeedForm` and `MspImport` (only their position), the Rust crates, the ADRs. Change `styles.css` only when the visual check needs it, and then say so.

## Architecture

These choices are fixed. Do not change them.

- `addfeeds.js` has no Preact and no browser state (ADR 0042). It exports:
  - `addChoices({ hasHost })`: `[{ id: 'make', label: 'Make a new feed', rows }, { id: 'have', label: 'Add a feed you have', rows }]`. Each row is `{ id, title, text }`, with the IDs `album`, `publisher`, `show`, `playlist`, `find`, `folder`, `msp` and the texts of the parent task. With `hasHost` false, the `make` group has `rows: []` and `connect: true`, and the `have` group has no `folder` row.
  - `MAKE_TYPES`: `[['album', 'Album'], ['publisher', 'Artist or label'], ['show', 'Show'], ['playlist', 'Playlist']]`, for the Type select menu.
  - `makeFields(type)`: `{ medium, titleLabel, authorLabel, role, button, busyLabel }`:
    - `album`: `music`, "Album title", "Artist", `false`, "Make the release", "Making the release…".
    - `publisher`: `publisher`, "Name", `null`, `true`, "Make the feed", "Making the feed…".
    - `show`: `music`, "Show feed title", "Artist", `false`, "Make the show feed", "Making the show feed…".
    - `playlist`: `musicL`, "Playlist name", "Author", `false`, "Make the playlist", "Making the playlist…".
  - `feedFileStart(title)`: ADR 0073 item 8. Use `title.normalize('NFKD')`, and remove the characters U+0300 to U+036F (`/[\u0300-\u036f]/g`). Make the text lower case. Replace each run of characters other than `a-z0-9` with one `-`, and remove a `-` at the two ends. An empty result gives `''`. Else add `.xml`.
- `AddFeeds` shows a `Bar` with "Add feeds", then for each group of `addChoices({ hasHost: connectors.length > 0 })` an `h2` with the label and a `div class="group"` of rows. Each row is a `button class="link-row"` with `span.label` (the title), `span.summary` (the text) and `span.go` (`›`). With `connect: true`, the group shows the hint "Studio puts the files of a release on a host. Connect a host first." and the button "Connect a Host" of the current page.
- A row sets the state `open` to its ID. One sheet component `AddFeedSheet` shows a `dialog class="sheet small"` with `useSheet(open !== null)`, a header with the title of the row and the close button ✕, and a body:
  - `album`, `publisher`, `show`, `playlist`: `MakeFeedForm` with the start type of the row.
  - `find`: `FindFeeds`, then `FoundFeeds` and "Search again", as in the panel of the current page.
  - `folder`: `AddFeedForm`. `msp`: `MspImport`.
  - The close of the dialog sets `open` to `null`.
- `MakeFeedForm({ connectors, startType, onAdded })` replaces `NewReleaseForm`, `NewPublisherForm` and `NewPlaylistForm`. Delete those three. It has:
  1. "Type": a `ChoiceSelect` of `MAKE_TYPES`.
  2. "Host": only when `connectors.length > 1`, as in the current forms.
  3. The title field with `titleLabel`, and the author field with `authorLabel` when it is not `null`.
  4. "Role" (`ChoiceSelect`, Artist or Label, "Select a role") only when `role` is `true`.
  5. A `details class="disclosure"` row: `summary class="link-row"` with `span.label` "Feed file" and `span.summary` the path, or "From the title" when it is empty. In the body, the `Field` "Feed file" with the hint "The path of the new feed file in the folder of the host."
  - The path follows `feedFileStart(title)` until the user types in the field. After that the title does not change it.
  - The `details` opens when the API gives a field error (`no_public_address` or `file_exists`).
  - The body of the request is the body of the current form of the type. For `publisher`, `title` and `author` are the name.
  - The submit button is `button primary`, with `button` and `busyLabel` of the type.
- The ID prefix of the fields is `make` (`make-type`, `make-host`, `make-title`, `make-author`, `make-role`, `make-path`).

## Steps

1. Write `addfeeds.js` and `addfeeds.test.mjs`. Test `feedFileStart` with "Pixel Palace" (`pixel-palace.xml`), "Ça va — Live!" (`ca-va-live.xml`), "  " (`''`) and "2024: Best Of" (`2024-best-of.xml`). Test `addChoices` with and without a host, and `makeFields` for each type.
2. Change `AddFeeds` and add `AddFeedSheet` and `MakeFeedForm` in `app.js`. Delete the three old forms.
3. Change the guide pages. Each one tells the user to select the row on the Add feeds page, then to fill the sheet. `new-release.md` says that Studio gives the feed file from the title, and that "Feed file" opens to change it.
4. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the changed guide lines.
5. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). `feedFileStart` gives a start value only, and the core checks the path. Name each new function that changes a value in your report.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.
- Each labeled button has its role of ADR 0037. A `link-row` is not a labeled button of ADR 0037.

## Acceptance Criteria

Mechanical:

- The tests of `addfeeds.test.mjs` pass, with the cases of step 1.
- `grep -n "NewReleaseForm\|NewPublisherForm\|NewPlaylistForm" crates/api/ui/app.js` finds nothing.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when a form of "Add a feed you have" needs a change to work in a sheet.
- Stop and report when the request body of a type differs from the current form of that type.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0073-the-add-pages-ask-one-question-at-a-time.md
- docs/tasks/studio-v1-task-049-add-pages.md
- docs/tasks/studio-v1-task-049a-add-feeds-page.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The Add feeds page shows two groups of rows, each row opens its form in a sheet, and one form makes each new feed with a start value for "Feed file".

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- editor.js, shell.js, pages.js, the four forms of "Add a feed you have" (only their position), the Rust crates, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
