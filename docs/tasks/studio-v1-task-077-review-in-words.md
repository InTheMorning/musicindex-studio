# Studio v1 Task 077: The Review In Words And The Marked Lines

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Review Of A Write".
Decision: [ADR 0114](../adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md). It uses [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md), [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) and [ADR 0103](../adr/0103-one-person-collects-the-items-of-many-suggestions.md) item 18.

Status: In progress. Packet 077a is complete. Notes from the reviews:

- 077a: the parts use the character mode of `similar` with semantic cleanup. The word mode marked parts of the credit text that did not change. A changed attribute covers only the start tag of its element (`ElementSpan::tag_end`). The groups follow the sequence of the table of the model keys. The planning model corrected the count of the uncovered changes: the first count did not count a changed line next to a covered line. With no live file, each line of the draft is covered.

## Goal

Each review shows "What changes" in the words of the editor, and the changed lines with the changed part of each line marked. A guard row counts each changed line that the words do not explain. The user selects "Simplified" or "Detailed", and opens hidden lines or the whole file.

## Packets

The planning model examines each packet before its commit. The packets go in this sequence:

1. 077a: the field changes, the lines with their parts and the cover of the lines (core and API).
2. 077b: the lines of the diff, the two views, the hidden lines and the indent (UI).
3. 077c: "What changes", the group summaries, the profile names and the guard row (UI).

Visual check of 077b and 077c, by the planning model, before each commit. The reference is `.studio-dev/mockup-diff/site/`, screens 1 to 4. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty. The data is a feed with six credits of one name, and a draft that adds one `npub` to each credit. A second draft changes the title and adds a namespace declaration.

Human test, by the operator, on `studio.musicindex.org` after the deploy:

1. Open a person with a credit key that differs, then "Check your feeds", then "Correct N Feeds".
2. In the review, "What changes" shows "6 credits get the Nostr key {profile name}", with the warning of ADR 0103 item 18 when it applies.
3. Select "Detailed". Each changed line shows the sign `−` or `+`, and only ` npub="…"` is marked.
4. Select "Show the Whole File", then "Show Only the Changes".
5. Open the review of one feed in the editor. It shows the same view as the review of the change set, and it starts on the last view that you selected.

## Words

- A **changed line** is a removed line or an added line of the line diff of the live file and the draft.
- A **field change** is one difference between the typed model of the live file and the typed model of the draft (ADR 0114 item 1).
- A **covered line** is a changed line that a field change explains (ADR 0114 item 4). An **uncovered change** is a run of adjacent changed lines that are not covered.
- A **pair** is a removed line and the added line that replaces it. A pair occurs in a replacement of the line diff with the same number of removed and added lines.

## Packet 077a: The Field Changes And The Lines

### Files

Inspect: `crates/core/src/pipeline.rs` (`Diff`, `diff_at`, the `TextDiff` call), `crates/core/src/xml.rs` (`Document`, `Element`, `Document::write`), `crates/core/src/feed.rs` (`Feed`, `Item`, `LiveItem`, `Feed::read`), `crates/core/src/feed/sync.rs` (`Entry`), `crates/core/src/feed/elements.rs`, `crates/core/src/feed/value.rs`, `crates/core/src/lib.rs`, `crates/core/Cargo.toml`, `crates/api/src/routes.rs` (`diff_json`), `crates/api/src/tests.rs` (the diff tests), `~/.cargo/registry/src/*/similar-3.2.0/src/text/inline.rs`.

Change: `crates/core/Cargo.toml`, a new module `crates/core/src/review.rs`, `crates/core/src/lib.rs`, `crates/core/src/xml.rs`, `crates/core/src/pipeline.rs`, `crates/api/src/routes.rs`, the core and API tests.

Do not change: the UI, the ADRs, `crates/server`, the collection schema, the bytes that `Document::write` gives.

### Architecture

These choices are fixed. Do not change them.

1. `crates/core/Cargo.toml` sets `similar = { version = "3", features = ["inline"] }`.
2. The new module `review` owns the field changes, the lines and the cover. `pipeline::diff_at` calls it with `draft.base_xml` and `draft.draft_xml`. `Diff` gets three fields: `fields: Vec<FieldGroup>`, `lines: Vec<DiffLine>` and `uncovered: usize`. `Diff::unified` stays (ADR 0114 item 7).
3. `review` reads each file with `Document::parse_bytes` and `Feed::read`. When a file does not parse, or a model does not read, `fields` is empty and each changed line is uncovered. When there is no live file, the live model is empty: each field of the draft is an added field.
4. The core compares the two models as their `serde_json` values. It ignores the `key` of each `Entry`, because each key is a position in its own file. It matches the entries of a list with these rules:
    - An item and a live item match by `guid`. An item with no `guid` matches by position.
    - The entries of each other list match through a sequence diff of their values, with `similar::capture_diff_slices`. In a replacement with the same number of old and new entries, the k-th old entry matches the k-th new entry. Each other old entry is removed, and each other new entry is added.
    - The same rules apply to a list in an entry, for example the recipients of a value block or the credits of an item.
5. `FieldGroup` has `kind`, `name: Option<String>`, `position: Option<usize>` and `changes: Vec<FieldChange>`. The groups have these kinds, in the sequence of the table of item 8:
    - `field`: one channel field that is not a list, for example `title` or `owner`. `name` is the key of the field.
    - `credits`: the channel credits (`persons`) of one name. `name` is the name of the credit.
    - `track`: one item. `position` is its position in the draft, from 1, or in the live file for a removed item. `name` is its title.
    - `live`: one live item, as `track`.
    - `list`: each other channel list, for example `values` or `fundings`. `name` is the key of the list.
6. `FieldChange` has `op` (`added`, `removed` or `changed`), `path`, `entry: Option<String>`, `old: Option<serde_json::Value>` and `new: Option<serde_json::Value>`. `path` is the JSON path in the group, with the keys of the model, for example `npub`, `enclosure.url` or `persons.npub`. `entry` names the entry in the group: the role of a credit, or the name of a recipient. An added or removed entry is one change with an empty `path` and the whole entry as its value. A value that is the same in the two models gives no change.
7. `Document` gets `write_with_spans(&self) -> (String, Vec<ElementSpan>)`. Its text is the text of `Document::write`. `ElementSpan` gives the path of the element and its byte range in the text. The path is the indexes of the child elements from the root. A test asserts that the text is the same as `Document::write` for each file of the corpus.
8. `review` has one table from each key of the serialized model to the element that `Feed::read` reads it from: a child element by namespace and local name, or the parent element for an attribute. A list key gives the child elements of one name, in their sequence. A test serializes a model with each field set and fails for a key with no entry. Its message is: "ADR 0114 item 5: the model key {key} has no element in the table of review.rs. Add the element that Feed::read reads it from."
9. Each field change covers the lines of the narrowest element of its path: in the live file for a removed line, and in the draft for an added line. A line is covered when the byte range of the element touches it. When `write_with_spans` does not give the bytes of the file, no line is covered.
10. `DiffLine` has `kind` (`same`, `removed` or `added`), `old: Option<usize>` and `new: Option<usize>` (line numbers from 1), `parts: Vec<LinePart>`, `pair: Option<usize>` and `covered: bool`. `LinePart` has `text` and `changed: bool`. A part has no line end. `lines` holds each line of the two files, in the sequence of the file.
11. The core makes the parts with `TextDiff::iter_inline_changes_with_options` and `InlineChangeOptions::semantic_cleanup(true)`. In a pair, the removed line comes first and its added line follows it. `pair` of the removed line is the index of its added line in `lines`. In a replacement with different numbers of lines, the removed lines come first, and no line has a pair.
12. `uncovered` is the number of uncovered changes.
13. The diff route `GET /api/v1/feeds/{id}/diff` gives `fields`, `lines` and `uncovered`. It keeps each field that it gives before this packet.
14. The field changes are a description. They change no draft and no write. The diff ID does not change.

### Acceptance Criteria

Mechanical:

- A core test: a draft that adds `npub` to six credits of one name gives one `credits` group with six `changed` changes of the path `npub`. `uncovered` is 0. Each changed line is covered.
- A core test: a draft that adds a namespace declaration to `rss` and changes the title gives a `field` group `title`, and `uncovered` is 1.
- A core test: the parts of each credit line mark only ` npub="…"` as changed. The parts of a changed title mark only the changed words.
- A core test: each removed line of a pair comes before its added line, and `pair` gives the index of that added line.
- A core test: two items with their `guid` in a different sequence give no field change of their content. A removed item gives one `removed` change in its `track` group.
- A core test: the table of item 8 has each key of a model with each field set.
- A core test: `write_with_spans` gives the text of `Document::write` for each file of the corpus.
- An API test: the diff route gives `fields`, `lines`, `uncovered` and `unified`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md
- docs/tasks/studio-v1-task-077-review-in-words.md (the Goal, the Words section and packet 077a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 077a names.

Goal:
- The diff route gives the field changes of the draft and each line of the file with its marked parts.
- It also gives the number of changes that the field changes do not explain.

Constraints:
- Follow the Architecture section of packet 077a fully.
- `Document::write` gives the same bytes as before. The corpus tests pass with no change.
- The diff ID and the write do not change.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the ADRs, `crates/server`, the collection schema.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 077a.

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

## Packet 077b: The Lines And The Two Views

### Files

Inspect: `crates/api/ui/editor.js` (`DiffView`, the review sheet), `crates/api/ui/app.js` (the review of a change set, `useMedia`, the width hook of 760 px), `crates/api/ui/styles.css` (`.diff`, `--diff-add`, `--diff-remove`, `.change-diff`), `crates/api/ui-tests/casing.test.mjs`, `.studio-dev/mockup-diff/site/review.js` (`xml`, `creditLines`, `viewBar`, screens 2 and 3).

Change: a new model module `crates/api/ui/review.js`, `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css`, a new Node test `crates/api/ui-tests/review.test.mjs`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `review.js` is a model module with no Preact and no browser state (ADR 0042). It gives these functions:
    - `shownLines(lines, view)`: the rows of a view. In "Simplified", a pair shows as one row with the sign `±` when one of its two lines has no changed part. Each other line shows as in "Detailed". In "Detailed", a removed line has the sign `−` and an added line has the sign `+`.
    - `runs(rows, whole)`: the rows with three rows of context around each changed row. Each run of other rows becomes one hidden run with its count. With `whole`, each row shows.
    - `commonIndent(rows)`: the number of leading spaces that each shown row has, with a tab as one space. An empty row does not count.
    - `moreLabel(count)`: "Show 1 more line" or "Show N more lines".
2. `DiffView` replaces its `unified` input with the diff of the route. It shows the view menu and the lines. The review sheet of the editor and the review of a change set use it (ADR 0114 item 13).
3. The select menu "View" has "Simplified" and "Detailed". Its hint is "What changes in words, then the changed lines." for "Simplified", and "The changed lines of the XML, as the file host gets them." for "Detailed". The UI keeps the last choice in `localStorage` with the key `studio.review.view`. Each read and each write is in a `try` block, and the default is "Simplified" (ADR 0114 item 8).
4. Each changed part shows in a `mark`. An added part has the class `ins` and the add color. A removed part has the class `del`, the remove color and a line through it. The sign of each row stays, so that a change is not only a color (ADR 0010).
5. In "Simplified" and in "Detailed", a hidden run shows as a link with the text of `moreLabel`, not as a button. The link is an `a` element with an `href` to its own run. A click or the Enter key opens that run only. The button "Show the Whole File" shows each row, and then reads "Show Only the Changes".
6. At 760 px or less, the view removes `commonIndent` spaces from the start of each shown row (ADR 0114 item 12). It changes no text that the core gave.
7. `diff.changed` false keeps the text "The draft is the same as the hosted file."
8. This packet shows no field view. "What changes" comes in 077c.

Note: the operator selected a link for "Show N more lines" on 2026-11-02. The link keeps the sentence case of ADR 0114 item 11. A button label uses title case (musicindex ADR 0023 R-CASE-1), and this text is not a button label.

### Acceptance Criteria

Mechanical:

- A Node test: `shownLines` gives one `±` row for a pair with only added parts, and two rows for a pair with a removed and an added part.
- A Node test: `runs` keeps three rows of context, gives the count of each hidden run, and with `whole` hides no row.
- A Node test: `commonIndent` gives the shared indent of the shown rows, and ignores an empty row.
- A Node test: `moreLabel` gives "Show 1 more line" and "Show 12 more lines".
- The four commit checks pass.

Visual: screens 2 and 3 of the visual check of this task, and the changed lines of screen 1.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md (items 6 to 13)
- docs/tasks/studio-v1-task-077-review-in-words.md (the Goal, the Words section and packet 077b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 077b names.

Goal:
- Each review shows the lines of the diff with the changed parts marked, in a "Simplified" or a "Detailed" view.
- The user can open the hidden lines.

Constraints:
- Follow the Architecture section of packet 077b fully.
- No feed rule in the UI (ADR 0065). The lines and their parts come from the core.
- Each button label is in title case. Each other text is in sentence case.
- Each comment is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 077b.

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

## Packet 077c: What Changes

### Files

Inspect: `crates/api/ui/review.js` and `DiffView` (from 077b), `crates/api/ui/people.js` (`profileMap`, `profileName`, `keyWarning`, `shortKey`), `crates/api/ui/peoplepages.js` (`useProfiles`), `crates/api/ui/roles.js` or the role words of task 074, `crates/core/src/feed.rs` (the serialized keys of `Feed`, `Item`, `LiveItem` and their entries), `.studio-dev/mockup-diff/site/review.js` (`fieldView`, `titleCase`, screens 1 and 4), `docs/user-guide/review-and-write.md`.

Change: `crates/api/ui/review.js`, `crates/api/ui/editor.js`, `crates/api/ui/peoplepages.js` only to export `useProfiles`, `crates/api/ui/styles.css` only if the visual check needs it, `crates/api/ui-tests/review.test.mjs`, `docs/user-guide/review-and-write.md`.

Do not change: the core, the API, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. In "Simplified", `DiffView` shows the section "What changes", then the section "Changed lines". "Detailed" shows only the lines.
2. `review.js` gives `fieldRows(groups, profiles)`. It gives one row group for each `FieldGroup`:
    - The label of a group: the word of the field for `field`, "Credits · {name}" for `credits`, "Track {position}" for `track`, "Live item" for `live`, and the word of the list for `list`.
    - The line of a change: the word of its path, then the old value, "→", and the new value. The path of a live item starts with `item.`, and the word does not include that prefix. A missing value is "none". An added entry reads "Added", and a removed entry reads "Removed". The label of a credit row is its role.
3. A word table in `review.js` gives a word for each key of the serialized model, in the words of the editor. Examples are `npub` "Nostr key" and `title` "Title". A key with no word shows the key.
4. A group with more than two changes of the same path and the same new value gives one summary line first, for example "6 credits get the Nostr key City Radio". The group then shows two changes, and the others in one collapsed `details` line, for example "4 more credits: songwriter, composer, guitarist, keyboardist" (ADR 0114 item 2).
5. A value of the path `npub` shows the profile name and the short key, for example "City Radio (npub12um…f5ck)". The view reads the profiles with `useProfiles`. Before the answer, or with no profile, it shows the short key only. A change of a credit key also shows `keyWarning(npub, profiles, name)` in the error color, with `name` the name of the credit (ADR 0114 item 3).
6. When `uncovered` is more than 0, the field view ends with the guard row of ADR 0114 item 4, with its words for one change and for more than one.
7. With no field change and `uncovered` more than 0, the field view shows only the guard row.
8. The user guide page "Review and write your changes" tells the two views, "What changes", the guard row, "Show N more lines" and "Show the Whole File".

### Acceptance Criteria

Mechanical:

- A Node test: `fieldRows` gives the summary line "6 credits get the Nostr key City Radio", two change rows and the collapsed line of four credits, for six changes of `npub` with one value.
- A Node test: the line of a changed title, of an added entry and of a removed entry.
- A Node test: a key line with a profile, with no profile, and with the warning of ADR 0103 item 18.
- A Node test: the guard row for 1 change and for 3 changes, and no guard row for 0.
- A Node test: the word table has a word for each key of `Feed`, `Item` and their entries. The test lists the keys from `crates/core/src/feed.rs`.
- The four commit checks pass.

Visual: screens 1 and 4 of the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md (items 1 to 5 and 9)
- docs/adr/0103-one-person-collects-the-items-of-many-suggestions.md (item 18)
- docs/tasks/studio-v1-task-077-review-in-words.md (the Goal, the Words section and packet 077c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 077c names.

Goal:
- The "Simplified" view starts with "What changes": the field changes of the core in words, with the profile name of each key and a guard row for each change that the list does not show.

Constraints:
- Follow the Architecture section of packet 077c fully.
- No feed rule in the UI (ADR 0065). The field changes and the cover come from the core. The words and the warning are display aids.
- An error uses the `--error` token.
- Each button label is in title case. Each other text is in sentence case.
- Each comment and each document sentence is in ASD-STE100 Simplified Technical English.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 077c.

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
