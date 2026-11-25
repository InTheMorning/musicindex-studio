# ADR 0222: Errors Give A Step, Terms Get A Hint, And The First Run Has Two Stages

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. The operator selected "Step + Try Again", "Try Again in both", "Hint line under it" and "Two stages, clear button". Then the operator selected "Fold, change the shared rule" for the raw text of an error.
- Technical choices: the main session selected items 2, 5, 7 and 10, and the texts of the task files.
- Bar: crosses repositories. Items 1 to 4 apply musicindex ADR 0085 R-MSG-1, and the Studio fold of item 3 is in the app table of that ADR. An agent can also make the opposite choice by mistake: a failed load that shows an empty list.
- Changes: [ADR 0221](../archive/adr/0221-the-welcome-page-and-the-first-steps.md) item 4. The sheet has two stages (item 8).
- Uses: musicindex ADR 0085 R-MSG-1 for the words of an error. [ADR 0018](0018-passphrase-strength-and-error-color.md) item 10 for an error of one field. [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) items 1 to 4 for the rows of step 2. [ADR 0217](0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) item 9 and [ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) item 1 for the payment words.
- Source: [analysis of two journeys for a new independent artist](../research/2026-11-18-user-journeys/analysis.md), themes T5, T8, T9 and T10, and observations O7 and O9.

## Context

- **T5, O7.** `ErrorNotice` (`app.js:441`) shows a title from the error code and then the raw text of the server. A code that the table does not know gets "Something went wrong.". The notice gives no step and no "Try Again". musicindex ADR 0085 R-MSG-1 items 2 to 4 ask for the step, for "Try Again" and for no raw text.
- **T5.** A failed import row has only "Remove" (`editor.js:1250`). The user must remove the row and select the file again.
- **T5.** Some list loads replace a failure with an empty list, for example `() => setMore([])` (`peoplepages.js:881`). The user cannot tell "nothing found" from "the load failed".
- **T8.** Terms from the feed standard show with no explanation, for example "The itunes:author of the feed.", "Send a podping", "Value Routes", "Split (shares)" and "Payees with no person". The component `Field` already has a hint line (`app.js:488`).
- **T9, O9.** The first-run sheet shows three stages and closes after two (`welcome.js:35`, `app.js:3598`). The button that makes the page says "Continue" for some rows. The library result shows counts of zero (`app.js:1774`).
- **T10.** No cross-app rule sets the payment words. The website says "Lightning address" and "Lightning node", and HitIt uses the same words. Studio keeps "LNURL Address" and "Node Address" (ADR 0217 item 9, accepted 2026-11-19) and "Split (shares)" (ADR 0144 item 1). The help line of the type "LNURL Address" says "Some apps call it a Lightning address." This ADR thus changes no payment word.

## Decision

### Errors

1. An error notice has a title, a step and, when another attempt can help, the button "Try Again":
    - The title names the action that failed, for example "Studio could not write the release.". The component that runs the action gives the title.
    - The step is one sentence from the error code. It tells the user what to do. The task file gives the step of each code. A code with no step gets "Try again. If the error stays, restart Studio.".
    - An error of one field stays at its field (ADR 0018).
2. "Try Again" runs the same action again with the same input. It shows for the codes that the task file marks as temporary, for example `offline`, `host_unavailable` and `network`. It does not show for a code that needs a change by the user first, for example `wrong_key` or `invalid_feed`.
3. The raw text of the error is in a fold with the label "Details" below the step. The fold is closed at the start. musicindex ADR 0085 R-MSG-1 item 4 permits the fold (musicindex commit `4025eba`, accepted 2026-11-19 through the orchestrator). A notice with no raw text, or with a raw text that is the same as the step, has no fold.
4. A failed import row has "Try Again" after "Remove". "Remove" stays the first button, because it is destructive (ADR 0193 item 1). "Try Again" sends the same file again.
5. A list that cannot load shows one line in the place of the list, "Studio could not load {the list}.", with "Try Again". An empty list keeps its empty text. This applies to each list load that gives an empty list for a failure. The task file names each load.

### Terms

6. Each term of the task file gets a hint at the place where a new artist first sees it. The term stays, so that the words agree with the feed standard and with the other two apps.
7. The hint uses the hint line of `Field`. A term that is not a field, such as a check box or a fold, gets a line with the same style below it. A hint has at most two sentences. It says what the item does for the user.

### First Run

8. The first-run sheet has two stages:
    - Musician: "Your name", "Where it lives".
    - Label: "Label name", "Where it lives".
    The first-step row of ADR 0260 item 8 is the next step after the sheet. It is not a stage of the sheet.
9. In step 2, the button that makes the page is "Make the Page", and "Making the Page…" while it works. This applies to each row and to the select menu of the connected file hosts.
10. The library result shows only the counts that are more than zero. When each count is zero, it shows "The folder has no releases yet.".

## Alternatives Considered

- A notice with the step and the raw text below it, with no button. Not selected by the operator.
- "Try Again" only on the import row. Not selected by the operator.
- A "?" button with a popover for each term. Not selected by the operator. It is a new component, and a touch screen needs a large target.
- Plain words in the place of each term. Not selected by the operator. The words would differ from the feed standard and from the other two apps.
- Three stages, with the third stage in the editor. Not selected by the operator.
- The payment words of the website. Not selected. ADR 0217 item 9 keeps the Studio words, and its help line names the other word.

## Consequences

- Each error notice of an action gets a title from the component, and the table of steps grows with each new code.
- A list load must give an error state, not an empty list.
- The first-run sheet and its tests change. The steps of ADR 0221 item 4 change.

## How This Decision Is Checked

Mechanical:

- UI tests: the step of each code in the table, and the step of a code that the table does not know (item 1). The fold "Details" (item 3).
- UI tests: "Try Again" for a temporary code, and no "Try Again" for a code that needs a change by the user (item 2).
- UI tests: a failed import row has "Try Again" (item 4). Each load of the task file has an error state (item 5).
- UI tests: each term of the task file has its hint text (items 6 and 7).
- `welcome.test.mjs`: two stages for each path (item 8). The button text of step 2 (item 9). The counts of the library result (item 10).
- `bash tools/gate.sh` passes.

Visual, by screenshots of the main session before each commit, at 1280 px and 390 px:

- An error notice with its step, "Try Again" and the closed fold "Details".
- A failed import row. A list that cannot load.
- The hints of the task file.
- The first-run sheet at step 1 and step 2. The library result.
