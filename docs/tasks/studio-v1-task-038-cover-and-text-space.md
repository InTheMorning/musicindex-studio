# Studio v1 Task 038: The Cover Form And The Space Around Text

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md), [ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: one packet. It comes before [task 039](studio-v1-task-039-file-picker.md), because task 039 item 22 uses the cover rule.

Status: Ready - 2026-10-27.

## Goal

A feed in the DeMu form shows its cover, and an edit keeps that form. A text value shows no space at its ends, and a save with no edit keeps the source bytes.

The operator found both defects on 2026-10-27 with `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`.

## Files

Inspect: `crates/core/src/xml.rs` (`Element::text`, `Element::set_text`), `crates/core/src/feed/sync.rs` (`read_text`, `write_text`, `Children::text`), `crates/core/src/feed/elements.rs` (the element macro, lines 30 to 40), `crates/core/src/feed.rs` (`Feed`, `Feed::read`, `Feed::write_hosted`, `Feed::adopt`, `CHANNEL_TAIL`), `crates/core/src/pipeline.rs` (`feed_addresses`), the address fields of ADR 0023 in the core, `crates/core/tests/corpus/manifest.json` and its round-trip test, `crates/api/ui/editor.js` (`ChannelFields`, the publisher page, the track page and the live item page where they use `model.image`).

Change: `crates/core/src/feed/sync.rs`, `crates/core/src/feed/elements.rs`, `crates/core/src/feed.rs`, `crates/core/src/pipeline.rs`, the address field list of ADR 0023, the corpus files and `manifest.json`, the new model module `crates/api/ui/cover.js`, the new test file `crates/api/ui-tests/cover.test.mjs`, `crates/api/ui/editor.js`.

Do not change: `Element::text` and `Element::set_text` in `xml.rs`. They stay generic XML functions. Do not change the write pipeline, the diff or the connectors.

## Architecture

These choices are fixed. Do not change them.

### The Space Around Text (ADR 0061)

- `feed/sync.rs` gets one function `model_text(element: &Element) -> String`. It gives `element.text()` with no XML white space at its ends: space, tab, carriage return and line feed. Space in the text stays.
- `feed/sync.rs` gets one function `set_model_text(element: &mut Element, value: &str)`. It removes the same space from `value`. When the result is equal to `model_text(element)` and the element has no child element, it returns and changes nothing. Otherwise it calls `element.set_text` with the result.
- `read_text` and the text field of the element macro use `model_text`. `write_text` and the text field of the element macro use `set_model_text`. No other code removes this space.
- An element that holds only space gives `Some("")`, as the read gives today for an empty element.

### The Channel Image (ADR 0060)

- `feed/elements.rs` gets a modeled type `RssImage` for the channel `<image>` with no namespace. It holds four optional text children: `url`, `title`, `link` and `description`. The children use `model_text` and `set_model_text`. A child with no value is not written. A child of `<image>` that the type does not model stays in the tree.
- `Feed` gets the field `rss_image: Option<Entry<RssImage>>` or `Option<RssImage>`. Use the form that `Feed` uses for its other single modeled children. `Feed::read` reads it, `Feed::write_hosted` writes it after `language` and before `podcast:guid`, and `Feed::adopt` keeps its source element.
- `feed_addresses` in `pipeline.rs` also gives the `url` of `rss_image`. The address check of ADR 0023 also checks that `url`.
- The core has no cover rule. The core models the two tags and writes what the model holds.

### The Cover Rule In The UI (ADR 0060 items 2 to 6)

- `cover.js` has no Preact and no browser state. It works on the model JSON that the editor sends. It has these functions:
  - `coverOf(model)`: the `href` of `itunes:image`, else the `url` of the RSS image, else an empty text.
  - `coverForm(model)`: `rss-only`, `itunes-only`, `both` or `none`.
  - `coverConflict(model)`: the address of the RSS image when both tags exist with different addresses, else `null`.
  - `withCover(model, address)`: a new model. It sets the address in each tag that the model has. With neither tag, it sets `image` (the `itunes:image`). With an empty address, it removes `image` and the full `rss_image`.
- The cover field of `ChannelFields` and of the publisher page shows `coverOf(model)` and calls `withCover` on a change.
- When `coverConflict(model)` gives an address, the field shows it below the field with the action button "Use this address" ([ADR 0037](../adr/0037-button-roles.md)). The action calls `withCover(model, address)`.
- Each place that shows the album cover for a track or a live item with no image uses `coverOf(model)`. No code writes the album cover into an item.

### The Corpus

- Add `demu-feed-template.xml` and `demu-feed-template-with-comments.xml` from `https://raw.githubusercontent.com/de-mu/demu-feed-template/4e2a1dc/feed.xml` and `.../feed-with-comments.xml`. Use the commit address, not `master`. Add one entry to `manifest.json` for each file, in the form of the other entries.

## Steps

1. Add `model_text` and `set_model_text`, and use them in the four places. Add the core tests of ADR 0061.
2. Add the two corpus files and their manifest entries. Run the round-trip test.
3. Add `RssImage` and the `Feed` field. Add the core tests of ADR 0060.
4. Add `rss_image` to `feed_addresses` and to the address check.
5. Add `cover.js` and its Node tests. Change the cover fields and the item fallback in `editor.js`.
6. Run the four checks. Give the operator the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No logic in two places: one function removes the space, and one module holds the cover rule.

## Acceptance Criteria

Mechanical:

- A core test reads a `description` with a line break and spaces at its ends. `read_text` gives the text with no space at its ends.
- A core test writes the same value with no edit. The document has the same bytes.
- A core test writes a new value. The element holds the value with no added space.
- A core test writes a value with space at its ends. The element holds the value with no space at its ends.
- A core test writes a value with space at its ends to an element whose source text is equal after the removal. The document has the same bytes.
- A core test reads an element that holds only space. The value is empty.
- The same tests pass for a CDATA section, and the CDATA section stays.
- The round-trip test passes for both DeMu files: a read and a write with no edit give the same bytes.
- A core test reads the DeMu `feed.xml`. `rss_image.url` is the address of `<image><url>`, and `image` is empty.
- A core test changes only `rss_image.url` of a feed with only `<image>`. The diff shows one changed element, and the document has no channel `itunes:image`.
- A core test removes `rss_image` and `image`. The channel has no `<image>` and no `itunes:image`.
- A core test confirms that `feed_addresses` gives the `url` of `rss_image`.
- The Node tests examine `coverOf`, `coverForm` and `coverConflict` for the four forms, and `withCover` for the four forms and for an empty address.
- The four commit checks pass.

Visual, for the operator:

- The album editor with `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`. "Cover image address" shows `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.png`, and the editor shows that image. "Description" starts with "testing".
- A feed with two different cover addresses shows the second address and "Use this address" below the field.
- A track with no image shows the cover of the DeMu feed.
- Each one in the dark theme, in the light theme and at 390 px.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a caller outside `feed/` depends on the space at the ends of a modeled text value.
- Stop and report when a corpus file of the existing corpus changes its bytes after step 1.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-038-cover-and-text-space.md
- docs/adr/0060-the-album-cover-keeps-the-form-of-its-feed.md
- docs/adr/0061-a-text-value-has-no-space-at-its-ends.md
- The files that the Files section names.

Goal:
- Remove the space at the ends of each modeled text value, on read and on write.
- Keep the source bytes of a value with no edit.
- Model the channel RSS `<image>`, and put the cover rule of ADR 0060 in the UI model module `cover.js`.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No logic in two places.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `Element::text` and `Element::set_text`, the write pipeline, the diff and the connectors.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The operator does the visual check.

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
