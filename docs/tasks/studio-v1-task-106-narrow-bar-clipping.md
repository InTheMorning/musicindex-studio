# Studio V1 Task 106: Narrow Bar Clipping

Status: Ready.

Owner of the rules: none new. The existing layout rules own the bar.

- [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) item 10a gives the two rows of the bar at 760 px or less.
- [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) item 1 gives the draft toolbar.
- [ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md) item 2 gives the toolbar of the page of a person.

Evidence: finding 9 of the [rc.21 findings](../reviews/2026-11-10-rc21-human-test-findings.md).

## Goal

No control is outside the window at a width from 320 px to 760 px. No part of the bar is hidden (ADR 0039 item 10a).

## Evidence

On a credit page of a feed ("The Heycitizen Experience" in the data of the operator), in a narrow window and on the phone, the right edge cut off the bar. It cut off "Saved, not written", **Versions** and "Revi…". It also cut off a red button and the Nostr row.

## Files To Inspect

- `crates/api/ui/styles.css`: `.bar`, `.bar-title`, `.draft-toolbar` and `.draft-status` (around lines 697 to 727), and the media block at 760 px (around line 899) with `.bar { flex-wrap: wrap }`.
- `crates/api/ui/editor.js`: `DraftToolbar` (line 2105) and `toolbarStatus`. The status text can be long.
- `crates/api/ui/editor.js`: the credit rows and `NostrRow` (around lines 225 to 270 and line 618), and the red remove button of a credit.
- `crates/api/ui/app.js`: the bar of a screen and the page tabs.
- `crates/api/ui/styles.css`: `.field`, `.field-body` and the entry row styles.
- `tools/drive-ui.mjs`: `$overflow()` and `$edges()`.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The bar keeps two rows at 760 px or less. The status stays in view, and **Review** keeps its place (ADR 0039 item 10a, ADR 0029 item 1).
- Find the cause first with a drive at 390 px and 320 px on the credit page. Fix the element that goes outside, not the symptom.
- A fix keeps the look of the bar at the desktop width.

## Acceptance Criteria

Mechanical: a browser drive reads `$overflow()` on each editor page of a feed at 390 px and at 320 px. The result is empty on each page. The pages are the album page, Credits, Recipients, Files, Settings of the feed, and a track page. The feed has a long status text, such as "Saved, not written", and a Nostr key.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs`. Use 1100 px, then 390 px and 320 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- The bar of the credit page with the status, **Versions** and **Review** all in view.
- The Nostr row and the red button of a credit in view.
- The page of a person, because it uses the same draft toolbar markup.

Human, by the operator: the credit page of a real feed on the iPhone.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- The cause is a long text with no break, such as an address. A rule for how to cut or wrap such text is not stated in an ADR.
- The bar needs a third row at 320 px, and ADR 0039 item 10a allows two rows.
- The drive at 320 px finds a defect in a page that this task does not cover.
