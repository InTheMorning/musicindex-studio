# Studio V1 Task 099: Settings Tabs And One Save Bar

Status: Complete, with its screenshot visual checks on 2026-11-11 on the desktop. The human tests are open. The server shell had no screenshot check: the Node tests cover its five tabs. Each section has the section heading of the app above one group (musicindex ADR 0059 R-SET-2 item 2). The hint of the lock setting has more than one line, and R-SET-2 item 1 asks for one line.

Owner of the rules: [ADR 0166](../adr/0166-settings-has-tabs-and-one-save-bar.md), all items including 17 and 18. The shared layout rules are musicindex ADR 0059 (`../musicindex/docs/adr/0059-settings-has-a-tab-row-and-one-save-bar.md`, read only).

## Goal

A user changes the settings of Studio in four or five tabs. One bar holds the state, Discard and Save. No long scroll remains.

## Packets

- 099a: the model of the tabs, and the tabs in `pages.js` and the Settings page (ADR 0166 items 1 to 5).
- 099b: the draft and the Save bar, with the pencil marks and the leave question (items 7 to 11, 13 to 16).
- 099c: the rows (item 6) and the removal of the Save buttons of Nostr and Library (item 14).
- 099d: the save failure and the conditions that stop Save (items 12, 17 and 18).

Do the packets in this sequence. Each packet leaves the tests green.

## Files To Inspect

- `crates/api/ui/app.js`: `SettingsPage` (around line 2658), `ScalePanel`, `LockPanel`, `WindowPanel`, `AccountPanel`, `ConnectedAppsPanel`, `NostrPanel`, `OutsidePanel`, `MovePanel`, `CollectionsPanel`, `LibraryPanel`, `FilesOnServerPanel`.
- `crates/api/ui/app.js`: `useLeaveGuard` (around line 1945) and `PageTabs` with the class `page-tab-menu` (around line 2028).
- `crates/api/ui/pages.js`: the page `studio-settings`, `pageSequence` and `tierError`.
- `crates/api/ui/peoplepages.js`: the person page bar, the class `draft-toolbar` (around line 969).
- `crates/api/ui/people.js`: `pendingStatus`, which gives the texts "No changes" and "N changes not saved".
- `crates/api/ui/uiscale.js` and `crates/api/ui/outsidecheck.js`: the model of the scale and of the outside check.
- `crates/api/ui/styles.css`: `.draft-toolbar`, `.draft-status`, `.segmented`, `.page-tab-menu`.
- Tests that name `studio-settings`: `crates/api/ui-tests/pages.test.mjs`, `collectionlist.test.mjs`, `outsidecheck.test.mjs`.
- Mockups: `.studio-dev/settings-mockup/d5-tabs-dirty.png` and `.studio-dev/settings-mockup/p5-tabs-dirty.png`. The pencil replaces the dot of these mockups (musicindex ADR 0059 R-SET-3 item 4).
- The routes of each setting. Search `crates/api/src/lib.rs` for the routes of the lock, the window, the outside check, the Nostr relay and the library limits.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The page and the tabs come from `pages.js`. The tier order of ADR 0071 item 1 holds in each tab.
- Save sends each pending setting to its current route. This task adds no route.
- An action keeps its button and its sheet, and Save does not run it (ADR 0166 item 15).
- The pencil of ADR 0165 item 2 is the one mark for a change that is not saved. Do not add a new mark.

## Acceptance Criteria

Mechanical, with Node tests (a new test file for the draft model, and `pages.test.mjs`):

- `pages.js` gives four tabs on the desktop and five on a server, with the labels and the sequence of ADR 0166 item 1.
- `pages.js` gives the sections of each tab, and `tierError` is empty for each tab.
- The model gives the Data tab with Library on a server that holds assets, and "Files on this server" on a server that holds none.
- The draft model gives "No changes", "1 change not saved" and "N changes not saved". Discard and Save are off with no change.
- The draft model marks each of the seven settings of item 7 as pending, and the tab of each pending setting.
- The model keeps the draft when the tab changes, and asks no leave question for a tab change.
- The model asks the leave question when the user leaves with a pending change.
- The model gives the saved UI scale back after Discard.
- A refusal of one route keeps that setting pending with its cause. The other settings are saved.
- The model gives the cause and turns Save off for a library limit that is not a number greater than 0.
- The model gives the cause and turns Save off for a Nostr relay that is not empty and does not start with `wss://` or `ws://`. Discard stays on.
- A Rust test in `crates/api` keeps the existing refusal `invalid_relay` for a relay value with a wrong scheme.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- Each tab, in a clean state and in a state with pending changes.
- The pencil on a changed row and on its tab.
- The select menu of the tabs at 390 px.
- A drive that changes the UI scale and the lock time, goes to a different tab, selects Discard, then does the same with Save.
- The bar with a cause, when Save is off.
- The row of a field with its cause.

Human, by the operator:

- The select menu of the tabs on the iPhone.
- A Save and a Discard on the desktop app with the real lock and window settings.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- A setting of item 7 has no route that accepts a draft value, or its route saves in a way that Save cannot repeat.
- The UI scale cannot show a preview without a reload.
- The tab for a server holds more than five groups, or a section has no tier in `pages.js`.
- A condition that stops Save needs a rule that ADR 0166 item 17 does not state.
