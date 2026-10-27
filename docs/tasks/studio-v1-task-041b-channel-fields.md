# Studio v1 Task 041b: The Channel Fields

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 3 to 7, [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md) and [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md) item 3.
Sequence: the second packet of task 041. It depends on 041a.

Status: Complete on 2026-10-27. The [visual check](../reviews/2026-10-27-task-041-visual-check.md) passed.

## Goal

The album page, the playlist page and the publisher page get "Explicit" after "Hide from apps". A row of the details section opens the page "Feed settings", with the owner, the keywords, the lock and the two editor fields.

## Files

Inspect: `crates/api/ui/editor.js` (`ChannelFields`, `ListingField`, `AlbumPage`, `PlaylistPage`, `PublisherPage`, `CreditsPage`, `AlbumEditor` and its page selection), `crates/api/ui/shell.js` (the outline pages, `page('credits', …)`), `crates/api/ui/listing.js`, `crates/api/ui-tests/listing.test.mjs`, `crates/api/ui-tests/shell.test.mjs`, `crates/api/src/ui.rs` (the served files).

Change: a new file `crates/api/ui/explicit.js`, a new file `crates/api/ui-tests/explicit.test.mjs`, `crates/api/ui/editor.js`, `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, `crates/api/src/ui.rs`.

Do not change: the core, the API routes, `crates/api/ui/listing.js`. Change `crates/api/ui/styles.css` only for a rule that no current class gives.

## Architecture

These choices are fixed. Do not change them.

### The Model Module `explicit.js`

- `isExplicit(text)` gives true when the trimmed text is `true` or `yes`, in any letter case. It gives false for each other text and for `null`.
- The module has no form rule. A change of the switch sends `true` or `false`. The core keeps the form of the feed at the draft save (ADR 0064 item 4, packet 041a).
- The module has no Preact and no browser state. Packet 041d uses the same module for the track switch.

### "Explicit"

- `ExplicitField` is a check box with the label "Explicit" and the state text "Explicit" or "Clean", as `ListingField` does.
- `ChannelFields` shows it directly after `ListingField`. `PublisherPage` shows it directly after its `ListingField`.
- A feed with no `itunes:explicit` shows the switch in the off position. Studio sets no value until the user changes the switch (ADR 0064 item 3).
- The hint says: "Apple asks for this tag. Studio adds it only when you set the switch."

### The Page "Feed Settings"

- The page has the name `settings`. The album page, the playlist page and the publisher page each get the row "Feed settings" in their details section, as the row "Credits and links" opens `credits`.
- The publisher page has no details section. It gets a new section directly after its first section. The heading is the role label and the word "details", such as "Artist details" or "Label details". The section has only the row "Feed settings".
- The outline of `shell.js` lists the page in these positions:
  - For an album, after "Credits and links".
  - For a playlist, after "Value routes" and before "Files".
  - For a publisher feed, after the entry of the feed and before "Files".
- In `AlbumEditor`, the publisher branch and the playlist branch catch most pages. The page `settings` gets its own branch before these branches.
- The summary line of the row gives the set values in short form, such as "Owner set · Locked". With no value, it says "No settings".
- The page has these groups, in this sequence:
  - "Owner": "Owner name" and "Owner email", for `owner.name` and `owner.email`. When both fields are empty, `owner` becomes `null`.
  - "Keywords": one text field. The hint says: "Words that apps can search for, with a comma between them."
  - "Lock against import": one switch for `locked.value`, and the field "Lock owner email" for `locked.owner`. With no `podcast:locked`, the switch is off and the email field is not shown. When the user sets the switch to on, `locked` gets `{ value: "yes", owner: null }`. When the user sets it to off, `locked.value` gets `no`, and the element stays.
  - "Managing editor" and "Webmaster": one text field each.
- An empty text field gives `null` for its value, so that Studio writes no empty element (ADR 0064 item 3). Each text value has no space at its ends (ADR 0061).
- No field refuses a value that is not an email address (ADR 0064 item 5).

### The Served Files

- `crates/api/src/ui.rs` serves `explicit.js`, as it serves `listing.js`.

## Steps

1. Add `explicit.js` with its Node tests.
2. Add `ExplicitField` to the three pages.
3. Add the page `settings`, its rows and its outline entry.
4. Serve the new module.
5. Run the four checks.
6. Take the screenshots of the visual criteria with `tools/drive-ui.mjs`.

## Constraints

- No feed rule in the UI (ADR 0065). A UI function that changes the model gives only a start value or a display value. The core applies each feed rule.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- Each control follows the durable rules of `AGENTS.md`: the button roles, the target sizes, the three background levels and the keyboard.
- A setting with a few values is a select menu, not a radio group.
- No logic in two places: the form rule of the explicit value is in the core only.

## Acceptance Criteria

Mechanical:

- A Node test: `isExplicit` gives true for `true`, `Yes` and ` yes `, and false for `false`, `no`, `clean` and `null`.
- A Node test of `shell.js`: the outline of an album lists "Feed settings" after "Credits and links". The outline of a playlist and of a publisher feed lists it before "Files".
- The four commit checks pass.

Visual, with screenshots, in the dark theme, in the light theme and at 390 px:

- The album page with "Explicit" after "Hide from apps".
- The page "Feed settings" of a feed with an owner and a lock.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the outline of `shell.js` gives no page entries for a playlist or a publisher feed.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-041b-channel-fields.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- The files that the Files section names.

Goal:
- Add the switch "Explicit" to the album page, the playlist page and the publisher page.
- Add the page "Feed settings" with the owner, the keywords, the lock and the two editor fields.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- A change of the switch sends `true` or `false`. The core keeps the form of the feed.
- Studio writes no field that the user did not set. No field refuses a value that is not an email address.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The core, the API routes and `listing.js`.
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
