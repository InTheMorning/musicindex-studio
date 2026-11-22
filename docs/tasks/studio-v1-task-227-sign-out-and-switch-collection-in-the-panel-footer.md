# Studio v1 Task 227: Sign Out And Switch Collection In The Panel Footer

Decision: [ADR 0241](../adr/0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md) items 1 to 8.

Status: Done.

## Problem

On a server, "Sign Out" is only in Settings, tab Account. On the desktop, a different collection opens only from Settings, section Collections. ADR 0241 puts both in the footer of the left panel.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Panel footer model | extend | crates/api/ui/shell.js | `leftPanel` pushes the rows `signout` and `switch` into `bottom`. |
| Panel footer rows | extend | crates/api/ui/app.js | `panelRows` gets a branch for each new row id. |
| Sign out call | reuse | crates/api/ui/app.js | `authCall('/signout')` and `reload()`, as `AccountPanel` does. |
| Collection list | reuse | crates/api/ui/collectionlist.js | `collectionRow` and the list of `GET /collections` give the rows of the sheet. |
| Open a collection | reuse | crates/api/ui/app.js | The "Open" action of the section Collections (ADR 0149 item 14). The sheet calls the same function. Do not copy it. |
| Icons | extend | crates/api/ui/app.js | `ICONS` gets `user` and `folder-open` from `~/build/musicindex-assets/icons/lucide/`, as the other icons. |

## Changes

1. `shell.js`, `leftPanel`: add the parameters `serverMode` and `collectionCount`. On a server, push `{ id: 'signout', label: 'Sign Out', icon: 'user' }` after the lock row. On the desktop with `collectionCount >= 2`, push `{ id: 'switch', label: 'Switch Collection…', icon: 'folder-open' }` after the row `collection`.
2. The root page of a narrow window uses the same `bottom`, so it gets the sequence of ADR 0241 item 3. Make sure of this with a test.
3. `app.js`, `panelRows`: `signout` calls the sign-out of `AccountPanel`. Move that call into one function that both use. `switch` opens the sheet of step 4.
4. A sheet "Switch Collection" in `app.js`. A pure model in `collectionlist.js` gives one row for each collection, in the sequence of Settings. The open collection has the text "Open now" and no button. Each other row has **Open**. The sheet has **Cancel**. **Open** calls the existing open action of the section Collections.
5. The caller of `leftPanel` gives `serverMode` and the count of collections. `useCollectionMark` already reads `GET /collections`. Extend it to also give the count. Do not add a second read.
6. Tests in `crates/api/ui-tests/`:
   - `shell.test.mjs`: "Sign Out" on a server and not on the desktop. "Switch Collection…" on the desktop with two collections, not with one, and not on a server. The sequence of ADR 0241 item 3.
   - A test of the sheet model: one row for each collection, and no button on the open collection.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0241 and an item. A test failure message names ADR 0241, the item and the function to fix.
- Button labels use title case.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The Node tests of step 6 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- The footer of a server with "Sign Out", in the dark theme, at 1280 px and at 390 px. "Sign Out" shows the sign-in page.
- The footer of a desktop with two collections, and the sheet "Switch Collection".

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

## Results

- `tools/gate.sh` passes.
- Server, dark theme, 1280 px: the footer is People, File hosts, Settings, Lock Studio, Sign Out. The root page at 390 px has the same sequence. Sign Out shows the sign-in page.
- Desktop: the `studio-desktop` binary opens one folder and has no collection list. The check used the release Tauri binary with a data copy that has two collections (`.studio-dev/t224/tauri227.sh`). The footer shows "Switch Collection…" after "Collection: HeyCitizen". The sheet shows "Open now" on the open collection and **Open** on the other. **Open** opened the second collection.
- Operator choice: the text "Open now" was in the middle of its row. A CSS rule puts it at the right end, in the column of the **Open** button.
