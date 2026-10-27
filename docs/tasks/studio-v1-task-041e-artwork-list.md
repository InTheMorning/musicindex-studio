# Studio v1 Task 041e: The Artwork List

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 10 to 12, [ADR 0016](../adr/0016-ui-without-a-build-step.md), [ADR 0025](../adr/0025-studio-serves-cover-art.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md).
Sequence: the last packet of task 041. It depends on 041d.

Status: Not started.

## Goal

The album page and the track page get a list of `podcast:image` entries. When the user selects an image, Studio fills the dimensions, the type and a near ratio. The user can change each value.

## Files

Inspect: `crates/api/ui/editor.js` (`AlbumPage`, `CreditsPage`, `TrackPage`, `Persons`, `EntryRow`, `EntryActions`, `addOpen`, `PickedImageField`, `useFilePick`, `FIELD_NOUN`), `crates/api/ui/app.js` (`useImage`, `ImageThumb`), `crates/api/src/routes.rs` (`get_image`), `crates/api/src/ui.rs`.

Change: a new file `crates/api/ui/artwork.js`, a new file `crates/api/ui-tests/artwork.test.mjs`, `crates/api/ui/editor.js`, `crates/api/src/ui.rs`.

Do not change: the core, the API routes, `crates/api/ui/picker.js`, `crates/api/ui/cover.js`.

## Architecture

These choices are fixed. Do not change them.

### The Model Module `artwork.js`

- `PURPOSES` is the list of the tokens of the specification: `artwork`, `social`, `canvas`, `banner`, `publisher`, `circular` and `poster`.
- `purposeTokens(text)` gives the tokens of a `purpose` value. The separator is one or more spaces. `null` gives an empty list.
- `withPurpose(text, token, on)` gives the new `purpose` value. It adds the token at the end, or removes it. Each other token stays in its position, also a token that is not in `PURPOSES` (ADR 0064 item 12). An empty result is `null`.
- `nearRatio(width, height)` gives the ratio as text, such as `16/9`:
  - The near ratios are `1/1`, `5/4`, `4/5`, `4/3`, `3/4`, `3/2`, `2/3`, `16/10`, `10/16`, `16/9`, `9/16`, `21/9`, `9/21`, `2/1`, `1/2`, `3/1`, `1/3`, `4/1` and `1/4`.
  - When the ratio of the image is within 2 % of a ratio of the list, the result is the nearest ratio of the list. The difference is relative to the ratio of the list.
  - In each other case, the result is the width and the height divided by their greatest common divisor.
  - A width or a height of 0 or of no value gives `null`.
- `imageFacts(width, height, mediaType)` gives `{ width, height, media_type, aspect_ratio }` as text values, for the fields of an entry.
- The module has no Preact and no browser state.

### The Facts Of An Image

- The user can select an image with the picker, by an upload, by a drop or by a typed address. After each selection, the UI loads the image through `/images` (ADR 0025), as `useImage` does. The UI loads nothing from other servers (ADR 0016).
- The width and the height come from `naturalWidth` and `naturalHeight` of the loaded image. The type comes from the type of the response. The UI applies `imageFacts` to the entry.
- Studio fills the facts only after a new address. A load does not change a value that the user typed for the same address.
- When the load fails, the facts stay as they are. The entry shows the hint: "Studio cannot read this image. Type the dimensions."

### The Artwork List

- `ArtworkList` lists the `podcast:image` entries of one parent, with `EntryRow`, `EntryActions` and `addOpen`, as `Persons` does. One entry is open at a time (ADR 0028).
- The summary line of an entry gives the purpose tokens and the dimensions, such as "banner · 3000 × 1000". With no purpose, it gives "No purpose".
- An entry has these fields:
  - "Image address", for `href`. It uses `PickedImageField`.
  - "Description", for `alt`. The hint says: "A short text for people who cannot see the image."
  - "Purpose": one check box for each token of `PURPOSES`. A token of the feed that is not in `PURPOSES` shows as a note with its text, and stays.
  - "Width", "Height", "Ratio" and "Type", in one `grid2`, for `width`, `height`, `aspect-ratio` and `type`.
- A new entry has `href` empty and each other value `null`.
- The album: the page "Credits and links" gets the section "More artwork" after "License". The summary of its row in the album page does not change.
- The track: the section "More" gets the disclosure "More artwork" after "Track image". Its summary gives the number of images, such as "2 images", or "None".

### The Served Files

- `crates/api/src/ui.rs` serves `artwork.js`.

## Steps

1. Add `artwork.js` with its Node tests.
2. Add `ArtworkList` and the facts of an image.
3. Add the list to the album and to the track.
4. Serve the new module.
5. Run the four checks.
6. Take the screenshots of the visual criteria with `tools/drive-ui.mjs`.

## Constraints

- No feed rule in the UI (ADR 0065). A UI function that changes the model gives only a start value or a display value. The core applies each feed rule.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- Each control follows the durable rules of `AGENTS.md`.
- No logic in two places: the ratio rule and the purpose rule are in `artwork.js` only.
- Do not copy code from MSP 2.0 or from `../mi-studio`.

## Acceptance Criteria

Mechanical:

- A Node test: `nearRatio` gives `16/9` for 1376 × 768, `1/1` for 3000 × 3000, `3/1` for 3000 × 1000 and `9/16` for 1080 × 1920.
- A Node test: `nearRatio` gives `7/5` for 1400 × 1000, and `null` for a width of 0.
- A Node test: `withPurpose` adds `banner` to `artwork`, and gives `artwork banner`.
- A Node test: `withPurpose` removes `artwork` from `artwork hero banner`, and gives `hero banner`.
- A Node test: `withPurpose` removes the last token and gives `null`.
- A Node test: `purposeTokens` reads `artwork  social` with two spaces as two tokens.
- An API test saves a draft with a channel image whose `purpose` has an unknown token, and changes its `alt`. The diff changes only the `alt` attribute.
- The four commit checks pass.

Visual, with screenshots, in the dark theme, in the light theme and at 390 px:

- The artwork list of the album, with one entry open, after the user selects an image. The dimensions, the type and the ratio show.
- The disclosure "More artwork" of a track with two images.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `/images` does not give the type of the image in its response.
- Stop and report when `PickedImageField` cannot take the address path of an entry of a list.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-041e-artwork-list.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- The files that the Files section names.

Goal:
- Add the list of `podcast:image` entries to the album and to the track.
- Fill the dimensions, the type and a near ratio when the user selects an image.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- The ratio rule and the purpose rule are in `artwork.js` only, with Node tests.
- Keep each `purpose` token that the list does not offer.
- The UI loads the image through `/images` only.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The core, the API routes, `picker.js` and `cover.js`.
- Do not copy code from MSP 2.0 or from `../mi-studio`.
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
