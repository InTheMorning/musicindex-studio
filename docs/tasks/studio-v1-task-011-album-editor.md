# Studio v1 Task 011: The Album Editor

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md), [ADR 0016](../adr/0016-ui-without-a-build-step.md) and [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md).
Depends on: [task 010](studio-v1-task-010-draft-api.md).

## Goal

Let the user edit the channel and the tracks of an album feed, examine the diff, write the change and roll back from a snapshot.

## Steps

1. Add an editor to the feed page of a feed with a connector. A read-only feed shows no editor.
2. Edit the channel: title, description, author, link, language, image, persons, funding links and the license.
3. Edit the tracks: title, description, enclosure, duration, publication date and image. Show the item GUID. Do not let the user change it.
4. Add a track, remove a track and move a track up or down. A new track needs an enclosure URL (ADR 0009).
5. Save the draft a short time after each change. Send the next save only after the response of the previous save (ADR 0022 items 5 and 9).
6. Show the diff before a write. The write button sends the ID of the diff that the user saw.
7. Show a conflict when the live file changed. Offer to start again from the live file.
8. Show the snapshot list. A rollback makes a draft from a snapshot, and the user then examines its diff.
9. Let the user remove the draft.

## Constraints

- The UI has no build step and loads nothing from other servers (ADR 0016).
- Each control is a minimum of 44 px high, and each function works from the keyboard (ADR 0010).
- An error uses the `--error` token (ADR 0018).

## Acceptance Criteria

Mechanical:

- The API contract tests of task 010 pass.
- The UI files name no other server. The existing test enforces this.

Visual:

- The operator edits each channel field and each track field of a published album feed, and the diff shows each change.
- The operator adds, removes and moves a track. The diff shows the change, and the GUID of each existing track does not change.
- The operator writes the draft, and the hosted file has the change.
- The operator changes the hosted file with a different tool, and a write then shows the conflict.
- The operator rolls back to a snapshot, examines the diff and writes it.
- Each control is a minimum of 44 px high, and the operator can do each step from the keyboard.

## Test Commands

```sh
cargo test --workspace
node tools/drive-ui.mjs <studio-url-with-token> <out-dir> '<steps-json>'
```
