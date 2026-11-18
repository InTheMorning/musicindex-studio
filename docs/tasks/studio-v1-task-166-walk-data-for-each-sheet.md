# Studio v1 Task 166: Walk Data For Each Sheet

Decision: ADR 0214 items 5 to 10, and musicindex ADR 0077 R-SHEET-2. This task makes walk data for an existing rule, so it has no new decision.

Status: Complete on 2026-11-17 for 16 of the 22 entries. The 6 entries of `collections` stay open.

## Problem

`tools/sheets.json` lists 58 sheets. The step `{"sheets": "tools/sheets.json", "data": "walk"}` opens the 36 entries of the condition `walk`. The other 22 entries name a condition that the walk data does not have (ADR 0214, section "Consequences"). Thus no check examines the fit of those 22 sheets.

The 13 conditions and their entries:

| Condition | Entries |
| --- | --- |
| `two-hosts` | Move This Release, Move All Releases |
| `library` | Release Folder |
| `collections` | Collection Actions menu, New Collection, Import an Account File, Copy a Collection, Rename a Collection, Export a Collection |
| `empty` | Start Sheet, Import an Account |
| `locked` | Unlock Studio |
| `removal-done` | Delete the Feed File |
| `label-no-release` | Add a Release |
| `playlist` | Playlist: Add Tracks |
| `findings` | Check: A Copy Pays Other People, Check: Music Index Has Another Address |
| `linked-page` | Update from Linked Pages, Credit Offer |
| `suggestion` | Check a Suggestion |
| `network` | From Music Index (person) |
| `wrong-feeds` | Correct the Feeds |

## Work

1. For each condition, write a setup file `.studio-dev/walk/setup-sheets-CONDITION.py`. It receives the run data folder as its first argument, as the other setup files do (`.studio-dev/walk/README.md`). Start from an existing setup file when one gives the condition. Use it through `runpy`, as `setup-131-removed.py` does, and do not write its code again. These setup files are examples:
   - `setup-142-move.py` has a second file host.
   - `setup-131-removed.py` has a done removal.
   - `setup-127b.py` and `setup-120c-*.py` write `index.json` for the local Music Index responder.
   - `setup-0076.py` has the person of the ADR 0076 walk.
2. For each condition, write a steps file `.studio-dev/walk/steps-sheets-CONDITION.json` with the light theme, the size `[1280, 800]` and the step `{"sheets": "tools/sheets.json", "data": "CONDITION"}`.
3. Write `.studio-dev/walk/sheets-all.sh`. It runs `run.sh` for `walk` and for each condition, one after the other. It writes one line for each condition with the exit status and each finding line of `drive-ui.mjs`.
4. Run `.studio-dev/walk/build.sh` one time, then `sheets-all.sh`.
5. When an entry of `tools/sheets.json` cannot open with any data, because its `open` steps are wrong, correct the `open` steps of that entry. Do not change its `data` value, its name or its source.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Walk harness | reuse | .studio-dev/walk/run.sh | It copies the data, runs the setup and starts the local responders. |
| Sheet walk step | reuse | tools/drive-ui.mjs | `walkSheets` filters by `data`. |
| Sheet list | extend | tools/sheets.json | Only corrections of `open` steps, if an entry cannot open. |
| Setup files | reuse | .studio-dev/walk/setup-142-move.py | Called with `runpy` for `two-hosts`. Other existing setup files the same. |
| Local Music Index responder | reuse | .studio-dev/walk/index.py | `run.sh` starts it when the data has `index.json`. |

## Do Not Touch

- Each file in `crates/`. A fit fault of a sheet is a finding for the main session. Do not correct the UI.
- `tools/drive-ui.mjs` and `.studio-dev/walk/run.sh`. If one of them blocks a condition, stop and report.
- `.studio-dev/v-116a-117/data`. Each run copies it.

## Constraints

- Each Bash call has its own network. Start Studio and Chromium in one command, through `run.sh`.
- `.studio-dev/` is ignored by Git. The setup files and the steps files stay there.
- Start a long run in the foreground of one call, or with the background option of your tool. Do not use `&` at the end of a command, because the sandbox stops the process when the call ends.
- The condition `empty` needs a data folder with no feed. The condition `locked` needs a vault with a secret, in the locked state at the start. If a condition needs a change to `run.sh`, stop and report.

## Acceptance Criteria

Mechanical:

1. `sheets-all.sh` runs each of the 14 conditions (`walk` and the 13 in the table).
2. For each of the 22 entries, the walk gives no `no-sheet` finding. The entry opens at 390 px and at 761 px.
3. The report gives each fit finding of the 58 entries with its entry name and width. A fit finding is not a failure of this task. The main session decides each one.
4. `git status` shows a change only in `tools/sheets.json`, or no change.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-166-walk-data-for-each-sheet.md
- docs/adr/0214-each-sheet-fits-the-window-and-the-walk-opens-it.md
- tools/sheets.json and tools/drive-ui.mjs (`walkSheets`, `runStep`, the helpers `$go` and `$tab`)
- .studio-dev/walk/README.md, .studio-dev/walk/run.sh, .studio-dev/walk/build.sh
- the existing setup files in .studio-dev/walk/ that the Work section names
- /home/citizen/.agents/skills/asd-ste100/SKILL.md, for comments and the report

Goal:
- Each of the 58 entries of tools/sheets.json opens in the sheet walk, with walk data for its condition.

Constraints:
- The constraints and the "Do Not Touch" list of the task file.
- Do not commit. Keep the change in the working tree. The main session reviews it and commits.

Do not touch:
- crates/, tools/drive-ui.mjs, .studio-dev/walk/run.sh and .studio-dev/v-116a-117/data.

Acceptance criteria:
- Mechanical criteria 1 to 4 of the task file.

Test commands:
- .studio-dev/walk/build.sh
- .studio-dev/walk/sheets-all.sh

At the end, report:
1. files changed or added
2. the line of each condition from sheets-all.sh
3. each fit finding, with entry and width
4. deviations from task
5. unresolved concerns, and each condition that you could not make

## Findings Of Run 1

Run 1 made the files in `.studio-dev/walk`. Keep them and correct them. The main session examined `runs/sheets-*/report.log`:

- `two-hosts` and `removal-done` gave no R-SHEET finding.
- `walk`: the setup file did not exist when the run started. Run it again.
- `findings` and `network`: `setup-127b.py` stopped at `sys.argv[2]`. Find why the mode did not arrive, and correct the setup file of this task.
- `collections`, `library`, `label-no-release`, `suggestion` and `wrong-feeds` only call `setup-0076.py`. That data does not have the condition, so the control that the `open` steps click does not exist. For each condition, read the component in the `source` field of the entry. Find the condition that shows the control, and write that condition into the data with SQL or files.
- `empty` calls `setup-none.py`, which does nothing. The data needs no feed and no collection content, so that Studio shows the start screen.
- `locked` calls `setup-139-held.py`, which has no vault. Find how Studio stores a locked vault (search `crates/core` for the vault and the lock). If only the UI can make the secret of the vault, add steps before the `sheets` step that make it and lock it, and report that.
- `playlist` calls `setup-164.py`, which has no playlist feed.
- `linked-page`: "Credit Offer" opens at 1280 px but not at 390 px and 761 px. Find if the `open` steps fail at those widths or the sheet does not fit. Report which.

Do not stop at the first fault. Make each condition, run `sheets-all.sh` again, and repeat until each of the 22 entries opens or you can name the block.

## Result

ChatGPT made the setup files and the `open` steps in two runs. The main session ran `sheets-all.sh` again, examined each change of `tools/sheets.json`, and corrected the two faults below.

- 16 of the 22 entries open at 390 px and at 761 px with the data of their condition, and give no finding.
- `collections`: the 6 entries do not open. Only the desktop app shell sets `CollectionsPlace`, and `studio-desktop` of `run.sh` cannot set it. Walk data for this condition needs a harness that starts with `CollectionsPlace`. It stays open work.
- `tools/sheets.json` changes only the `open` steps of 6 entries: Unlock Studio, Add a Release, Credit Offer, Check a Suggestion, From Music Index (person) and Correct the Feeds. The steps of "Unlock Studio" send `POST /vault/signer-key`, which opens the unlock sheet through the lock check of the API.
- Run 2 found that "Credit Offer" did not open because Save had no change to save. It was a fault of the `open` steps, not of the sheet.

Faults that the walk found, corrected by the main session:

1. "Review" and "Versions" closed at 390 px. `Bar` (`crates/api/ui/app.js`) put the children after the title in a different place when the narrow root buttons show. Preact then mounted the draft toolbar again, and its open sheet closed. Commit `55ef457` added this after ADR 0214. The children now keep one place at each width.
2. "Release Folder": a folder name with no break point broke inside a word (musicindex ADR 0065 R-FIT-3). The selected folder and each folder row of the sheet now have the break attribute, as the folder path of a file host has (ADR 0214, section "Consequences").

Checks by the main session:

- `sheets-all.sh`: each condition other than `collections` exits with status 0 after the corrections.
- Screenshots: the review sheet stays open after a change from 1280 px to 390 px. At 390 px Dark, the long folder name wraps in its row. `$edges()` and `$overflow()` give no result.
- `tools/gate.sh` passes.
