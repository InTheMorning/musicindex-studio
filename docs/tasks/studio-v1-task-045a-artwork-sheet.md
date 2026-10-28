# Studio v1 Task 045a: The Artwork Sheet

Parent: [task 045](studio-v1-task-045-page-sequence-and-artwork.md). Decisions: [ADR 0070](../adr/0070-a-cover-opens-the-artwork-sheet.md), [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md) item 4, [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md), [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 10 to 12, [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the first packet of task 045. It depends on task 044.

Status: Ready.

## Goal

Each cover in the header of an editor page is a button that opens the Artwork sheet of its item. The sheet holds the image field, the RSS image note and the "More artwork" list. Each page shows one row "Artwork" in the position of its image field.

## Files

Inspect: `crates/api/ui/app.js` (`ImageThumb`, `EditableFeed`, the `feed-head` header), `crates/api/ui/editor.js` (`CoverField`, `PickedImageField`, `ArtworkList`, `ArtworkEntry`, `imagesSummary`, `ChannelFields`, `PublisherPage`, `CreditsPage`, `TrackPage`, `useSheet`, `VersionsSheet`, `AlbumEditor`), `crates/api/ui/artwork.js`, `crates/api/ui/cover.js` (`coverOf`, `coverConflict`), `crates/api/ui/styles.css` (the sheet styles, `.head .cover`, `.link-row`), `crates/api/ui-tests/artwork.test.mjs`, `crates/api/ui-tests/feed-rules.test.mjs`.

Change: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/artwork.js`, `crates/api/ui/styles.css`, `crates/api/ui-tests/artwork.test.mjs`.

Do not change:

- `crates/api/ui/cover.js` and the cover rule of ADR 0060. The core keeps the form of the cover (ADR 0065).
- The file picker and `useFilePick`.
- The outline, `shell.js` and the cover in the outline head (ADR 0069).
- The live item page and its image.
- Each Rust file.

## Architecture

These choices are fixed. Do not change them.

### The Model

- Add `export function artworkSummary(subject, kind)` to `artwork.js`. `kind` is `'feed'` or `'track'`. `subject` is the model of the feed or of the track.
  - For a feed: `Cover` when `coverOf(subject)` gives an address, else `No cover`. Then ` · 1 more image` or ` · N more images` when `subject.images` has entries.
  - For a track: `Own image` when `subject.image` has a value, else `Uses the album cover`. Then the same text for `subject.images`.
- Remove `imagesSummary` from `editor.js`.

### The Cover Button

- Add the prop `onOpen` to `ImageThumb`. With `onOpen`, the component gives a `button` with `type="button"`, the class `cover-button`, `aria-label` from the prop `label` and `title="Change the artwork"`. The image or the mark is in the button. With no `onOpen`, the component does not change.
- In `EditableFeed`, the cover of the `feed-head` header gets `onOpen` and `label` `Artwork of` and the title of the feed. It sets the sheet state to `{ kind: 'feed' }`.
- In `TrackPage`, the cover of the track header gets `onOpen` and `label` `Artwork of` and the title of the track. It sets the sheet state to `{ kind: 'track', key: trackKey(item) }`.

### The Sheet

- `EditableFeed` keeps the state `artwork`: `null`, `{ kind: 'feed' }` or `{ kind: 'track', key }`. It gives `artwork` and `onArtwork` to `AlbumEditor`. `AlbumEditor` gives `onArtwork` to `AlbumPage`, `PlaylistPage`, `PublisherPage` and `TrackPage`.
- `AlbumEditor` shows `<${ArtworkSheet}>` when `artwork` is not `null`. A track key that no track has closes the sheet.
- `ArtworkSheet` uses `useSheet`. Its title is "Artwork" and the title of the item. Its parts, in this sequence:
  1. A large `ImageThumb` of the image: `coverOf(model)` for a feed. For a track, `item.image`, else `coverOf(model)` with the text "Uses the album cover".
  2. For a feed, `CoverField`, with its note and "Use this address". For a track, the `PickedImageField` that the track page has now, with the same `field`, `apply`, `address` and `glow`.
  3. For a track with an image, the button "Use the Album Cover" with the role normal ([ADR 0037](../adr/0037-button-roles.md)). It sets `image` of the track to `null`.
  4. For an album and a track, `ArtworkList` with the props that it has now. A playlist and a publisher feed have no list.
  5. The button "Done", with the role primary. It closes the sheet. Escape also closes it.
- Each change in the sheet calls `update`, as the fields on the pages do now.

### The Pages

- `ChannelFields` and the first section of `PublisherPage`: replace `CoverField` with one `link-row` button "Artwork". Its summary is `artworkSummary(model, 'feed')`. A click calls `onArtwork({ kind: 'feed' })`.
- `TrackPage`: remove the disclosures "Track image" and "More artwork" from "More". Add one `link-row` button "Artwork" in the group "Track", after "Explicit". Its summary is `artworkSummary(item, 'track')`. A click calls `onArtwork({ kind: 'track', key: trackKey(item) })`. Packet 045b moves it.
- `CreditsPage`: remove the section "More artwork".

### The Styles

- `.cover-button` has no border, no background and no padding. Its corner radius is the radius of the cover in it. It has the hover style and the focus ring of the other buttons, and `cursor: pointer`.
- The preview of the sheet is at most 240 px wide, in the shape of musicindex ADR 0021 R-ART-4.

## Steps

1. Add `artworkSummary` and its Node tests.
2. Add `onOpen` to `ImageThumb`, and the sheet state to `EditableFeed`.
3. Add `ArtworkSheet`.
4. Change `ChannelFields`, `PublisherPage`, `TrackPage` and `CreditsPage`.
5. Add the styles.
6. Run the four checks.
7. Make the screenshots of the visual check with `tools/drive-ui.mjs`, and run `$edges()`.

## Constraints

- No feed rule in the UI (ADR 0065). The sheet sends the same values that the fields send now. Examine each new function that changes the model.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- The UI loads nothing from other servers (ADR 0016).
- All functions work from the keyboard.
- A labeled button has a role (ADR 0037). No labeled button is a `button quiet`.

## Acceptance Criteria

Mechanical:

- A Node test: `artworkSummary` gives `Cover` for a feed with a cover and no other image, and `Cover · 2 more images` for a feed with two other images.
- A Node test: `artworkSummary` gives `No cover` for a feed with no cover.
- A Node test: `artworkSummary` gives `Uses the album cover` for a track with no image, and `Own image · 1 more image` for a track with an image and one other image.
- A Node test: `editor.js` has no `imagesSummary`, and `CreditsPage` has no text "More artwork". Read the file as text, as `feed-rules.test.mjs` does.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`, in the dark theme, the light theme and at 390 px:

- A click on the album cover opens the sheet of the album. A click on the track cover opens the sheet of the track.
- Tab reaches each cover, and the focus ring shows. Enter opens the sheet. Escape closes it.
- The sheet of a track with no image shows "Uses the album cover" and no button "Use the Album Cover".
- A change of the cover address in the sheet shows in the draft status of the toolbar.
- `$edges()` finds no content at the edge of a group.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a file pick in the sheet loses its upload after the sheet closes.
- Stop and report when `useSheet` cannot open a second sheet over the page while the Review sheet is closed.
- Stop and report when a different module uses `imagesSummary`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-045a-artwork-sheet.md
- docs/adr/0070-a-cover-opens-the-artwork-sheet.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- The files that the Files section names.

Goal:
- Each cover in the header of an editor page opens the Artwork sheet of its item.
- Each artwork field is only in the sheet. Each page shows one row "Artwork".

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `cover.js`, the file picker, `shell.js`, the outline head, the live item page and each Rust file.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
