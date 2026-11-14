# Studio v1 Task 141: An http Image Shows From Its https Address

Decision: [ADR 0202](../adr/0202-an-http-image-shows-from-its-https-address.md).

Status: Complete on 2026-11-14. The human check of ADR 0202 passed on 2026-11-14: the picture of Strange Love shows, and its row keeps the `http` address.

## Result

- `imageSource(url, held)` in `media.js` gives `mediaSource` of the `https` form when `mediaSource` gives `reason: 'http'`. An op3 prefix stays. `useImage` in `app.js` uses it. The audio player still uses `mediaSource`.
- The main session added an assertion to the scan test, so that the test fails when it does not find `AudioPlayer`.
- The walk browser has no network, so no screenshot shows a real `http` picture. The operator check covers it.

## Goal

1. `useImage` shows the `https` form of an `http` image address (ADR 0202 items 1, 2 and 4).
2. `mediaSource` and the player do not change (item 5).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/media.js:51` `mediaSource(url, held)`. Line 61 gives `{ kind: 'none', reason: 'http' }` for an `http` address. `plainAddress` removes a prefix such as op3.
- `crates/api/ui/app.js:318` `useImage(url)`. It calls `mediaSource(url, held)`, then probes a `direct` source or reads a `held` file.
- Each image of the UI uses `useImage`: `PersonArt` (`peoplepages.js:46`), `ImageThumb` and `ArtCell` (`app.js`), `PersonPicture`, `ProfileBody` and `ArtworkRow` (`editor.js`).
- Tests: `crates/api/ui-tests/media.test.mjs:20` (the `http` case of `mediaSource`).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Media source | extend | crates/api/ui/media.js | Add `imageSource(url, held)` next to `mediaSource`. It uses `mediaSource` for the `https` form. |
| Image hook | extend | crates/api/ui/app.js | `useImage` calls `imageSource` in place of `mediaSource`. |

## Rules For The Delegate

- Read ADR 0202 and ADR 0109 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not change `crates/api/ui/peoplepages.js` or `crates/api/ui/recipients.js`. A different task changes them now.
- Do not change the page rule in `crates/api/src/ui.rs`.
- If a step needs an architecture decision that ADR 0202 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `crates/api/ui/media.js`: add `export function imageSource(url, held)`. When `mediaSource(url, held)` gives `reason: 'http'`, it gives `mediaSource` of the same address with `https://` in place of `http://` (keep the rest of the address, also a prefix such as op3). Otherwise it gives the result of `mediaSource`. Add a comment that names ADR 0202.
2. `crates/api/ui/app.js` `useImage`: use `imageSource` in place of `mediaSource`. Update the comment above `useImage`: an `http` address loads from its `https` form.

## Acceptance Criteria

Mechanical:

1. A Node test of `imageSource`: `http://host.example/a.png` gives `{ kind: 'direct', src: 'https://host.example/a.png' }`. A held map with `https://host.example/a.png` gives the `held` source. An `https` address and an empty address give the same result as `mediaSource`. Each failure message starts with "ADR 0202" and gives the fix.
2. The existing test of `mediaSource` for `http` still passes with no change.
3. A Node scan test: `useImage` in `app.js` calls `imageSource`, and the player code does not.
4. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session: the walk browser has no network, so the check of a real `http` picture goes to the operator.

Human, by the operator: the gate of ADR 0202.

## Report

- The changed lines, with file and line.
- The test names and the output of the four checks.
