# Studio v1 Task 183: An Error Gives A Step And Try Again

Decision: [ADR 0222](../adr/0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) items 1, 2, 4 and 5.

Status: Done.

## Problem

Theme T5 and observation O7 of the [analysis of two journeys](../research/2026-11-18-user-journeys/analysis.md):

- `ErrorNotice` (`crates/api/ui/app.js:441`) shows a title from the error code and the raw server text. It gives no step and no "Try Again".
- A failed import row has only "Remove" (`editor.js:1250`).
- Eight list loads give an empty list when the load fails. The user cannot tell "nothing found" from "the load failed".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Error notice | extend | crates/api/ui/app.js | `ErrorNotice` near line 441 and `title` near line 468. Add the action title, the step and "Try Again". |
| New attempt rule | reuse | crates/api/ui/app.js | `canTryAgain` near line 208: status 0 or a status of 500 or more. |
| Action runner | extend | crates/api/ui/app.js | `useAction` near line 616. Keep the last work, so that "Try Again" can run it again. |
| Import row | extend | crates/api/ui/editor.js | `ImportRow` near line 1244 and the send near line 1333. Add "Try Again". |
| Words of a failure | reuse | crates/api/ui/failures.js | `FAIL_TEXT` gives the words of a feed read. Do not copy them. |

## Changes

1. UI: `ErrorNotice` gets two optional properties, `action` and `onRetry`.
    - The title is `action` when the caller gives it. Else it is the title of the code, as now. The title of a code that the table does not know changes from "Something went wrong." to "Studio could not finish this.".
    - Below the title, the notice shows the step of the code from the table of change 2. A code with no row gets "Try again. If the error stays, restart Studio.".
    - Below the step, the notice has a `<details>` fold with the summary "Details" and the raw text `error.message`. The fold is closed at the start (ADR 0222 item 3). A notice has no fold when `error.message` is empty or the same as the step. Use the fold style of the other `disclosure` folds of Studio.
    - The notice shows the button "Try Again" when `onRetry` is given and `retryHelps(error)` is true. `retryHelps` is true when `canTryAgain(error)` is true and the code is not in the list "needs the user" of change 2.
2. UI: put the table of steps and the function `retryHelps` in a new module `crates/api/ui/errors.js` with no Preact, so that the Node tests can import it. Move `canTryAgain` to that module and import it in `app.js`. The steps:

    | Code | Step | Needs the user |
    | --- | --- | --- |
    | `offline` | "Make sure that Studio is still running, then try again." | no |
    | `network`, `not_reachable` | "Check the internet connection of this computer, then try again." | no |
    | `host_unavailable` | "The file host did not answer. Check the connection, then try again." | no |
    | `host_unauthorized` | "The file host refused the login. Check the login of the file host in File hosts." | yes |
    | `vault_locked` | "Unlock the vault, then try again." | yes |
    | `signature_required` | "Connect your Nostr signer, then try again." | yes |
    | `stale_revision`, `conflict` | "The page changed in a different place. Open the page again, then do the change again." | yes |
    | `read_only` | "Studio cannot change this feed. It is read-only." | yes |
    | `no_public_address`, `address_not_public` | "Give this file host a public address in File hosts." | yes |
    | `too_large`, `download_limit` | "The file is too large. Use a smaller file, or change the limit in Settings." | yes |

3. UI: `useAction` keeps the last work function. It returns `retry()`, which runs that work again. Give `action` and `onRetry=${retry}` to the notices of these actions:
    - The write in the review sheet: "Studio could not write the release." For a publisher feed: "Studio could not write the page."
    - The save of a draft: "Studio could not save the change." The draft save does not use `useAction`. Its notice gets `onRetry` that calls `flush` of `useDraft` (`draft.js` near line 245).
    - The connection of a file host: "Studio could not connect the file host."
    - The add of a feed by address: "Studio could not add the feed."
    - The library folder setup: "Studio could not set up the library."
    Find each notice in `app.js` and `editor.js`. When one of them does not use `useAction`, report it and do not change it.
4. UI: a failed import row (`row.state === 'failed'`) has the button "Try Again" after "Remove". "Remove" stays the first button of the row (ADR 0193 item 1). It sends the same file again with the same options, through the same send code as the first send. Its accessible name is `Try again: {file name}`.
5. UI: these loads get an error state in the place of an empty list. The state shows one line "Studio could not load {list}." and the button "Try Again", which runs the load again:

    | Load | Place | {list} |
    | --- | --- | --- |
    | Wallet recipients | `peoplepages.js` near line 535 | "the wallets" |
    | More suggestions | `peoplepages.js` near line 881 | "the suggestions" |
    | Profile address suggestions | `peoplepages.js` near line 889 | "the suggestions" |
    | Snapshots | `editor.js` near line 2290 | "the snapshots" |
    | Moves | `editor.js` near line 4628 | "the moves" |
    | Feeds of a folder | `app.js` near line 2365 | "the feeds of this folder" |
    | Page suggestions of a person | `peoplepages.js` near line 906 | "the suggestions" |
    | Members of a publisher feed | `editor.js` near line 3914 | "the members" |

    Do not change `loadRecipients` in `values.js` near line 72. It gives an empty list on purpose, and the editor then makes no lookup (ADR 0207 item 48).
6. Do not change the API or the core.

## Files

- New: `crates/api/ui/errors.js`, `crates/api/ui-tests/errors.test.mjs`.
- Change: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/peoplepages.js`.
- Change tests when a test reads a changed text: the tests in `crates/api/ui-tests/`.

## Checks

Mechanical:

- UI: `errors.test.mjs` examines the step of each code of the table, and the step of a code that the table does not know.
- UI: `retryHelps` is true for status 0, for status 503 with `host_unavailable` and for status 500 with `internal`. It is false for `host_unauthorized`, for `vault_locked` and for status 422.
- UI: a source test finds the fold "Details" in `ErrorNotice`, closed at the start.
- UI: a source test finds "Try Again" in `ImportRow`, and an error state for each load of change 5.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- A write error with its title, its step, "Try Again" and the closed fold "Details". The same notice with the fold open.
- A failed import row with "Try Again" and "Remove".
- A list that cannot load, with its line and "Try Again".

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
