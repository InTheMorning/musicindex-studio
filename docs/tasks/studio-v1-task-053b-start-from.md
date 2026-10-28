# Studio v1 Task 053b: "Start From" In The Form

Parent: [task 053](studio-v1-task-053-album-from-album.md). Decision: [ADR 0077](../adr/0077-a-new-album-can-start-from-an-album.md) items 1 to 3.

Status: Complete - 2026-10-28. The four commit checks pass.

## Goal

The form that makes a new feed shows the select menu "Start from" for the type Album. A selected album gives the start value of "Artist", and the form sends `from`.

## Files

Inspect: `crates/api/ui/addfeeds.js`, `crates/api/ui-tests/addfeeds.test.mjs`, `crates/api/ui/app.js` (`MakeFeedForm`, `AddFeedSheet`, `ChoiceSelect`, `useAction`), `crates/api/ui/shell.js` (`mediumKind`), `docs/user-guide/new-release.md`.

Change: `crates/api/ui/addfeeds.js`, `crates/api/ui-tests/addfeeds.test.mjs`, `crates/api/ui/app.js`, `docs/user-guide/new-release.md`.

Do not change: the Rust crates, `editor.js`, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `addfeeds.js` exports `startChoices(feeds)`: `[['', 'Empty album'], ...]`, then one pair `[String(id), title]` for each feed that the Releases list shows as an album (`mediumKind`), in the sequence of the title with `localeCompare`. A feed with no title uses its address.
- `MakeFeedForm` gets the props `feeds`. With the type `album` and more than one value in `startChoices(feeds)`, it shows the `ChoiceSelect` "Start from" after "Type", with the ID `make-from`.
- A selection of an album reads `GET /api/feeds/{id}/draft`. When the user did not type in "Artist", the `author` of the model becomes the value of "Artist". "Album title" does not change.
- The form sends `from` with the ID of the selected album. "Empty album" sends no `from`.
- A change of the type to a type other than `album` clears "Start from".

## Steps

1. Write `startChoices` and its tests in `addfeeds.test.mjs`.
2. Change `MakeFeedForm` and `AddFeedSheet` in `app.js`.
3. Add a section to `new-release.md`: "Start from an album". It says that the new album gets the tracks of the source with new GUIDs, and that the source does not change.
4. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the changed guide lines.
5. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). The start value of "Artist" is an edit aid. The core makes the copy. Name each new function that changes a value in your report.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- `addfeeds.test.mjs` tests `startChoices` with no album, with two albums in a different sequence, with a publisher feed and with a playlist.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when `mediumKind` cannot tell an album from a show feed.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0077-a-new-album-can-start-from-an-album.md
- docs/tasks/studio-v1-task-053-album-from-album.md
- docs/tasks/studio-v1-task-053b-start-from.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The new feed form shows "Start from" for the type Album, gives the start value of "Artist", and sends from.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- The Rust crates, editor.js, the ADRs.
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
