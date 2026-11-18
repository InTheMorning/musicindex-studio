# Studio user journeys at commit a64637d

Date: 2026-11-18. Status: research record. This record is advisory. It states no rule.

This record gives the user journeys of Studio as they are in the code at commit `a64637d`.
It is the data for a subsequent UX analysis. It does not give design proposals.

The previous study is in the research repository, at
`musicindex-research/research_notes/ui-design/journeys/studio/`. That study used Studio commit `563cce6`
(2026-10-24). ADR 0071 cites it. This record does not replace it. The two records use the same step method,
so you can compare the step counts.

## Scope

- The scope is Studio only: the UI in `crates/api/ui/`, the desktop app and the server shell.
- The record covers the first run, the shell and navigation, the file hosts and Settings.
- It also covers the collections, the account, the feed editor, live shows, People, the confirmations
  and the errors.
- The record does not cover the other apps of the suite.

## Method

Three steps made the record.

1. **Code reading.** Three ChatGPT runs (`gpt-6.1-sol`, effort `medium`) read the code and wrote one
   inventory each. Each inventory gives the entry points and the steps with the exact labels and `file:line`.
   It also gives the branches, the confirmations, the errors, the end state and a step count.
2. **Examination of the inventories.** The main session compared each quoted label of the three inventories
   with the sources. All 1,098 labels exist in the code, or the code makes them from a template. The main
   session also read a sample of the `file:line` references. They are correct, or one or two lines off.
3. **Walks.** The main session operated the desktop binary of commit `a64637d` in headless Chromium and
   made screenshots. The walks examine the most frequent journeys. A walk result is stronger evidence than
   the code reading. Where the two disagree, `walks.md` says so.

## Step method

The method is the same as in the previous study.

- One click, one key, one typed field or one selection from a list is one step.
- A wait or an automatic action is zero steps.
- A native file selection is one step.
- A count starts at the start point that the journey names.

## Evidence marks

| Mark | Meaning |
|---|---|
| **Code** | The fact comes from the code, with a `file:line` reference. |
| **Shot** | A walk screenshot shows the fact. The file is in `shots/`. |
| **Inference** | A conclusion from the code that no walk examined. |
| **Unsure** | The code reading could not find the fact, for example the text of a native dialog. |

## Shells

- **Desktop local mode.** The Tauri app with a local library folder. The walks used this shell.
- **Server mode.** The server shell with a sign-in and an account. The walks did not examine it.
- **Narrow layout.** The CSS condition is `max-width: 760px` (`app.js:1844`). The walks examined it at 390 px.
- **Touch device.** The condition is `pointer: coarse` (`app.js:1971`). The walks did not examine it.

## Files

| File | Contents | Language |
|---|---|---|
| `README.md` | This page. | STE |
| `walks.md` | Each walk, its steps, its screenshots and its results. | STE |
| `step-counts.md` | The step counts of the main journeys, with a comparison to the previous study. | STE |
| `observations.md` | The observed facts and defects for the analysis. | STE |
| `inventory/A-shell-and-settings.md` | Run A: first run, sign-in, vault, navigation, file hosts, Settings, collections, apps, live shows, notices, errors, surfaces and unused controls. | Tool output |
| `inventory/B-feed-editor.md` | Run B: each journey in the feed editor, Review, the write, versions, conflicts, files, moves and deletion. | Tool output |
| `inventory/C-people.md` | Run C: People, the person pages, merges, and the use of a person in a feed. | Tool output |
| `shots/` | The walk screenshots. The name starts with the walk ID. | Images |

The three inventories are the recorded output of the ChatGPT runs. They are kept without a change, except
that absolute paths are relative. They are not written in STE, because an STE rewrite of 300 KB can lose
facts. Use their labels and references as data. Use their step counts as **Inference**, unless `walks.md`
gives a walk result.

## Known limits

- The walks used one local data set and no network. Steps that need a relay, a signer, a remote host or
  Music Index are code reading only.
- The record does not measure time, errors of users or the difficulty of a step. A step count is not a
  measure of difficulty.
- One correction to inventory B is in `walks.md`, walk J04.
