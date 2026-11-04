# Studio v1 Task 089: Drop Targets

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Phase 13: Desktop Shell".
Decision: [ADR 0125](../adr/0125-each-file-target-shows-that-it-takes-a-drop.md).

Status: Complete, with its visual check. The human test is open.

## Goal

A user with a mouse can see where a file can drop, in the desktop app and in a browser.

## Work

- `crates/api/ui/filedrop.js` is a new model module. It has the handlers of a target, the mark of a file drag and the account drop rule. `crates/api/ui-tests/filedrop.test.mjs` tests it.
- `useFileDrop` and `dropAreaClass` in `app.js` connect a target to the module.
- The file fields show "or drop a file here". "Add tracks", "This Computer", "From MSP 2.0 backups" and "Import an account" have a drop area.
- `addsAsTrackFile` and `refusedText` in `picker.js` give the rule and the text of "Add tracks". The kind `lyrics` includes `.txt`, so `kindOfFile` alone is not sufficient.
- On Linux, the route of ADR 0124 item 22 gives each file of a drop. `linux_page.js` reads the bytes only for a target that takes them. A target with `data-drop-many` takes many files.

No feed rule in the UI (ADR 0065). The new functions change no feed model. A drop is an upload through the existing routes.

## Acceptance Criteria

Mechanical: the criteria of ADR 0125, and the four commit checks.

Visual, by the planning model. Each result was a pass on 2026-11-04.

In Chromium at 1280 px, with `Input.dispatchDragEvent` and real files:

- A track page shows the hint. During a drag, the field has the light outline, and the field below the pointer has the strong state.
- A drop on Title changes no text, and the page stays.
- "This Computer" refuses a text file with "Select an audio file", and it sends a WAV file.
- "Add tracks" adds two WAV files. A drop of `notes.txt` alone shows "Studio cannot use notes.txt. Drop music and image files." and adds no row. A drop of audio and text together ran before the correction of the `.txt` rule, and it did not run after it. A Node test examines the rule.
- "From MSP 2.0 backups" takes many files and names a file that is not JSON.
- "Import an account" refuses a file of a different kind and two files. It sends a `.tar` file, and the check of the archive answers.

At 390 px with touch, each `.drop-text` has `display: none`. `$fieldText()`, `$overflow()` and `$edges()` give no result. `$edges()` at 1280 px gives no result.

On Xvfb, in the Linux debug build, with a drag from Thunar:

- "Add tracks" gets six files of 100 MB. The rows show after about 3 seconds. The network process of WebKitGTK uses about 750 MB.
- The light outline stays while the pointer does not move, and it ends when the drag leaves the window.

Human, by the operator: a drop on each target in the desktop app, and in a browser on the server. A second browser (Firefox) and Safari are not tested.
