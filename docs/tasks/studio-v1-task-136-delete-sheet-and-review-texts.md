# Studio v1 Task 136: The Delete Sheet Stays Still And The Review Texts Are Correct

Decision: [ADR 0131](../adr/0131-delete-a-feed-from-its-file-host.md) items 10 to 12. This task corrects defects. It makes no new rule.

Status: Open.

## Goal

Findings 3 and 4 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md):

1. A click on **Also delete the audio and image files** does not hide the title, the file host row or the line of the linking feed. The sheet keeps its height while the new preview loads.
2. In the review of a delete, the row of the feed file does not say "Studio deletes it after the feed is written."
3. In the review of a delete, the row **Announce** does not say "this feed has no public address".

## Code Locations

These come from a search on 2026-11-13. Make sure that each one is correct before a change.

- `crates/api/ui/editor.js:2377` `loadPreview` in `FeedDelete`: it calls `setPreview(null)` before the request.
- `crates/api/ui/editor.js:1683` `OPERATION_NOTE`: `delete: 'Studio deletes it after the feed is written.'`. `HostOperationRow` uses it.
- `crates/api/ui/announce.js:28` `announceLines`: `announce_skip === 'no_address'` gives "No announcement: this feed has no public address."
- `crates/core/src/announce.rs:114` `AnnounceSkip::NoAddress` gives `no_address`. `crates/core/src/pipeline.rs:406` has `announce_skip` in the diff. Find where the diff of a delete draft sets it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Delete sheet | extend | crates/api/ui/editor.js | `FeedDelete` keeps the old preview until the new one arrives. |
| Operation row note | extend | crates/api/ui/editor.js | `OPERATION_NOTE` gets a note for the delete of the feed file. |
| Announce lines | extend | crates/api/ui/announce.js | `announceLines` gets the line of a delete. |
| Diff announce skip | extend | crates/core/src/announce.rs | `AnnounceSkip` gets the value `feed_deleted` for a delete draft. |

## Rules For The Delegate

- Read ADR 0131 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `loadPreview` keeps the current preview while the request runs. It sets the new preview when the answer arrives. During the request, the check box and **Delete** are off, and the counts show the old values. On an error, the sheet shows the error, as it does now.
2. In the review of a delete draft, the row that deletes the feed file has the note "This write deletes the feed file." The other delete rows keep their note. Find the path of the feed file from the diff or the feed. Do not compare file names in the UI.
3. For a delete draft, the core gives `announce_skip: "feed_deleted"`, not `no_address`. `announceLines` gives "No announcement: this write deletes the feed file."

## Acceptance Criteria

Mechanical:

1. A Node test of `announceLines` with `announce_skip: "feed_deleted"` gives the line of change 3.
2. A core test: the diff of a delete draft has `announce_skip` `feed_deleted`.
3. A Node test of the note of change 2, if the note comes from a pure function. Otherwise, the screenshot below is the check.
4. The four commit checks pass.

Visual, by the main session, with screenshots and the fit walk:

- The final sheet before and after a click on the check box. The title, the file host row and the line of the linking feed stay. The sheet has the same height.
- The review of a delete: the note of the feed file row and the line of **Announce**.

Human, by the operator: none. The screenshots prove these changes.

## Stop And Ask When

- ADR 0131 says that a delete sends an announcement.
- The diff gives no way to know the feed file path.

## Report

- The changed functions, with file and line.
- The test names.
- The output of the commit checks.
