# ADR 0125: Each File Target Shows That It Takes A Drop

- Status: Accepted
- Date: 2026-11-04
- Owner: operator
- Accepted: 2026-11-04 by the operator, through the questionnaire. The operator selected the short hint on a field, a drop area in a sheet, and each target lights up during a file drag.
- Changes: [ADR 0062](0062-a-file-picker-for-each-file-field.md) item 21. A field also shows with no drag.
- Uses: [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) items 22 to 24 for the drop on Linux.

## Context

- A field of ADR 0062 takes a drop, but nothing shows this until a file is over that field.
- The Welcome page says "Drop the songs in", but the sheet "Add tracks" took no drop. The sheets "Choose…", "From MSP 2.0 backups" and "Import an account" also took no drop.
- A file that the user drops outside a field can make a browser open that file.

## Decision

1. With a mouse, each file field of ADR 0062 shows "or drop a file here" after its button. With touch, it does not show (ADR 0062 keeps the button for a screen with no pointer).
2. These locations have a drop area, with a dashed outline and a line of text: "Add tracks", "This computer" in the sheet "Choose…", "From MSP 2.0 backups" and "Import an account". With touch, only the button shows.
3. While a file drag is over the window, each target that the user can reach shows a light outline. The target below the pointer shows the state of ADR 0062 item 21.
4. "Add tracks" and "From MSP 2.0 backups" take many files. Each other target takes one file. A file that a target cannot use shows an error and changes nothing. "Add tracks" adds its other files.
5. A file drag that no target takes changes nothing. The pointer shows a refusal, the browser does not open the file, and a text box does not get its address.
6. "Add the File Again" has no drop, because it takes one exact file for one row.
7. The two shells use the same code. A browser on WebKitGTK gives a page no file ([WebKit bug 323277](https://bugs.webkit.org/show_bug.cgi?id=323277)), so there the button is the only method.

## Alternatives Considered

- A dashed box on each field at all times. Not chosen: a track page has many file fields.
- No sign on a field. Not chosen: a user does not learn about the drop.
- A state on the field below the pointer only. Not chosen: the user must find each target.

## Consequences

- On Linux, the app gives the page the bytes of many files for a target that takes many files. The page holds them in memory until it sends them.

## Invariants

- No drop writes to a host. A drop is an upload, and a write still goes through Review (ADR 0007).

## Non-goals

- A drop on a phone.
- A drop target on "Add the File Again".
- A correction for a browser on WebKitGTK.

## Acceptance Criteria

Mechanical:

- A Node test: a target takes a file drag and gives the files of a drop. A busy target takes nothing. A text drag does not change.
- A Node test: a file drag marks the root, and the mark ends when no `dragover` comes. A drag that no target takes shows a refusal.
- A Node test: the account drop takes one `.tar` file.
- A test: the route of ADR 0124 item 22 gives each file of a drop.

Visual, by the planning model:

- In Chromium, each target of items 1 and 2 shows its hint or its area, and takes a real file drag.
- At 390 px with touch, no hint and no dashed area shows.
- On Xvfb, the Linux app takes a drop of many files on "Add tracks".

Human, by the operator: a drop on each target in the desktop app and in a browser on the server.
