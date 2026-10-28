# Studio v1 Task 045b: The Page Sequence

Parent: [task 045](studio-v1-task-045-page-sequence-and-artwork.md). Decisions: [ADR 0068](../adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md) item 5 and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the second packet of task 045. It depends on 045a.

Status: Ready.

## Goal

One model module gives the sections and the fields of each editor page in the sequence of ADR 0068. The album page, the playlist page, the publisher page, the settings page and the track page show them from that list. "Language" moves to the page "Feed settings".

## Files

Inspect: `crates/api/ui/editor.js` (`ChannelFields`, `AlbumPage`, `PlaylistPage`, `PublisherPage`, `SettingsPage`, `settingsSummary`, `SettingsRow`, `FeedFacts`, `TrackPage`, `TrackPayments`, `AlbumEditor`), `crates/api/ui/styles.css` (`.page-section`, `.group`), `crates/api/ui-tests/feed-rules.test.mjs`, `crates/api/ui-tests/words.test.mjs`.

Change: `crates/api/ui/pages.js` (new), `crates/api/ui/editor.js`, `crates/api/ui-tests/pages.test.mjs` (new).

Do not change:

- The fields themselves: their labels, their hints, their values and their `onChange` functions. Only their position changes.
- The outline and `shell.js`.
- The Credits page, the Value routes page, the Files page and the live item page.
- Each Rust file.

## Architecture

These choices are fixed. Do not change them.

### The Model Module

- Add `crates/api/ui/pages.js` with no Preact and no browser state (ADR 0042). It exports `pageSequence(name)`. It gives a new copy of the list at each call.
- The field IDs of the first section are the same for the three feed pages: `['title', 'author', 'artwork', 'description', 'link', 'listing', 'explicit']`.
- `pageSequence('album')`:
  `[{ id: 'album', fields: FEED_FIELDS }, { id: 'tracks' }, { id: 'details', rows: ['recipients', 'credits'] }, { id: 'publishers' }, { id: 'feed', rows: ['settings', 'facts'] }]`.
- `pageSequence('publisher')`:
  `[{ id: 'channel', fields: FEED_FIELDS }, { id: 'releases' }, { id: 'role' }, { id: 'feed', rows: ['settings', 'facts'] }]`.
- `pageSequence('playlist')`:
  `[{ id: 'channel', fields: FEED_FIELDS }, { id: 'feed', rows: ['settings', 'facts'] }]`.
- `pageSequence('settings')`:
  `[{ id: 'language' }, { id: 'keywords' }, { id: 'owner' }, { id: 'locked' }, { id: 'editors' }]`.
- `pageSequence('track')`:
  `[{ id: 'track', fields: ['title', 'audio', 'duration', 'alternates', 'artwork', 'description'] }, { id: 'credits' }, { id: 'lyrics' }, { id: 'payments' }, { id: 'more', rows: ['pub_date', 'explicit', 'identity'] }]`.
- Give the module a header comment that names ADR 0068. The comment says that the list is the sequence of the pages.

### The Pages

- Each page gets its list from `pageSequence`.
- A page shows each section through an object that maps the section ID to a render function. It shows each field or row through an object that maps the field ID to a render function.
- The section headings are these:
  - Album page: `Album`, `Tracks`, `Album details`, `Artists and labels`, `Feed`
  - Publisher page: the type label, `Releases`, `Role`, `Feed`
  - Playlist page: `Playlist`, `Feed`
  - Track page: `Track`, `Credits`, `Lyrics`, the heading of `TrackPayments`, `More`
- `ChannelFields` has no field "Language". The album page, the playlist page and the publisher page use `ChannelFields` for their first section. The second field has the label `Artist`, `Author` or `Name`, as now.
- `SettingsPage` gets the section "Language" first, with the field of `ChannelFields` from before: the same ID `ch-language`, the label, the hint and `set(m, 'language', value)`.
- `settingsSummary` starts with `Language` and the language code when the model has a language.
- `PublisherPage` gets `onRemoved` from `AlbumEditor`, and the row `FeedFacts`.
- `TrackPage`: the field "Duration" follows the audio file. The group "More" holds the field "Publication date", then the field "Explicit", then the row "Identity". The publication date keeps its hint (ADR 0024 item 5).
- The buttons "Move up", "Move down" and "Remove the track" stay at the end of the track page.

## Steps

1. Add `pages.js` and `pages.test.mjs`.
2. Change `ChannelFields`, `SettingsPage` and `settingsSummary`.
3. Change `AlbumPage`, `PlaylistPage`, `PublisherPage` and `TrackPage` to show their lists.
4. Run the four checks.
5. Make the screenshots of the visual check with `tools/drive-ui.mjs`, and run `$edges()`.

## Constraints

- No feed rule in the UI (ADR 0065). This packet moves fields and changes no value.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- Each section of a page is a direct row of `.page`, as now (review of task 014c, defect 2).
- The editor keeps three background levels (ADR 0029).

## Acceptance Criteria

Mechanical:

- A Node test in `pages.test.mjs` compares `pageSequence` for each of the five pages with the lists of this packet. Each assertion message starts with `ADR 0068:`, names the page, and says "Change the list in crates/api/ui/pages.js and the ADR together."
- A Node test: two calls of `pageSequence('album')` give two different objects.
- A Node test: `editor.js` has no field "Language" in `ChannelFields`. Read the file as text, as `feed-rules.test.mjs` does.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`, in the dark theme, the light theme and at 390 px:

- The album page, the publisher page, the playlist page, the settings page and a track page show the sequence of this packet.
- The field "Language" on the settings page changes the language in the diff.
- `$edges()` finds no content at the edge of a group.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a field cannot move without a change of its `onChange` function.
- Stop and report when a Node test or the words test examines the position of a field.
- Stop and report when the publisher page cannot get `onRemoved` from `AlbumEditor`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-045b-page-sequence.md
- docs/adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md
- The files that the Files section names.

Goal:
- Add `crates/api/ui/pages.js`, which gives the sequence of each editor page.
- The five pages show their sections and fields from that list. "Language" moves to "Feed settings".

Constraints:
- Follow the Architecture section of the task file fully.
- Move the fields. Do not change their labels, hints, values or `onChange` functions.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The outline, `shell.js`, the Credits page, the Value routes page, the Files page, the live item page and each Rust file.
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
