# Studio v1 Task 035f: The Home Screen App

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md) items 8 to 11, and [ADR 0016](../adr/0016-ui-without-a-build-step.md).
Sequence: the sixth packet of task 035. It depends on task 034.

## Goal

A phone user adds Studio to the home screen, and it opens with no browser bar.

## Files

Inspect: `crates/api/src/ui.rs` (the file list and the headers), `crates/api/ui/index.html`, `crates/api/ui/assets/favicon.svg`, `crates/api/ui/theme.js` (the colors of the dark theme), `../musicindex-assets/brand/` (`apple-touch-icon.png`, `favicon.svg`), `../musicindex-assets/AGENTS.md`.

Change: `crates/api/src/ui.rs`, `crates/api/ui/index.html`, the new files `crates/api/ui/manifest.webmanifest`, `crates/api/ui/assets/icon-192.png`, `crates/api/ui/assets/icon-512.png`, `crates/api/ui/assets/icon-maskable-512.png`, `crates/api/ui/assets/apple-touch-icon.png`, `crates/api/src/tests.rs`.

Do not change: the security headers, other than the media type of the manifest.

## Architecture

These choices are fixed. Do not change them.

- `/manifest.webmanifest` has `name` "Music Index Studio", `short_name` "Studio", `start_url` `/`, `scope` `/`, `display` `standalone`, and `background_color` and `theme_color` from the page color of the dark theme. Its icons are the three PNG files.
- The media type is `application/manifest+json`.
- The icons come from the brand files of `musicindex-assets`. The coding model makes the PNG files of 192 px and 512 px from `favicon.svg` with a tool on the computer, such as headless Chromium. The maskable icon keeps the mark in the safe area of the Web App Manifest specification. The report names the source and the tool.
- `index.html` gets `<link rel="manifest" href="/manifest.webmanifest">`, `<link rel="apple-touch-icon" href="/ui/assets/apple-touch-icon.png">` and `<meta name="theme-color">`.
- The manifest and the icons need no token and no session. In the server, `ui_router()` serves them.
- Studio has no service worker.

## Steps

1. Make the icon files and record their source.
2. Add the manifest and its route.
3. Add the links to `index.html`.
4. Add the tests.

## Constraints

- No new crate. The UI loads nothing from other servers.
- The icon files have the license of `musicindex-assets`.

## Acceptance Criteria

Mechanical:

- An API test: `/manifest.webmanifest` gives 200 with `application/manifest+json` and no token in local mode. Each icon of the manifest gives 200.
- A server test: the same with no session.
- The test of ADR 0016 that finds no other server in the UI passes.
- The four commit checks pass.

Visual:

- The planning model opens the manifest in headless Chromium and confirms that the DevTools protocol reports no manifest error (`Page.getAppManifest`).

Human test, in task 037:

- "Add to Home Screen" on an iPhone and on an Android phone opens Studio with no browser bar.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the brand files have no license that permits this use.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035f-home-screen-app.md
- docs/adr/0059-uploads-in-parts-and-the-home-screen-app.md
- The files that the Files section names.

Goal:
- Add the web app manifest, its icons and the links in `index.html`.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No service worker.

Do not touch:
- The security headers, other than the media type of the manifest.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

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
