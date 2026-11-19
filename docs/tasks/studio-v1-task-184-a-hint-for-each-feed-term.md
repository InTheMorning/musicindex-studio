# Studio v1 Task 184: A Hint For Each Feed Term

Decision: [ADR 0222](../adr/0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) items 6 and 7.

Status: Done.

## Problem

Theme T8 of the [analysis of two journeys](../research/2026-11-18-user-journeys/analysis.md). Terms from the feed standard show with no explanation at the place where a new artist first sees them. Task 185 gives the lines of the first-run rows.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Field hint | reuse | crates/api/ui/app.js | `Field` near line 488 has `hint`. `TextField` in `editor.js` near line 74 passes it. |
| Fold note | reuse | crates/api/ui/pickers.js | `PickFold` near line 206 has `note`. |
| Hint style | reuse | crates/api/ui/styles.css | The class `hint`. Add no new style. |

## Changes

1. UI: give these hints. Use the exact texts.

    | Term | Place | Hint |
    | --- | --- | --- |
    | The author field of a release | `editor.js` near line 2746. Replace "The itunes:author of the feed." | "The artist name that podcast apps show." |
    | "Name" of a publisher feed | `editor.js` near line 3984. Replace "The itunes:author of the feed." | "The name that podcast apps show for this page." |
    | "Send a podping" | `AnnounceFooterChoice` in `editor.js` near line 1856, below the check box | "Tells podcast apps that the feed changed, so that they show the change in minutes." |
    | "Value Routes" | `RecipientsPage` in `editor.js` near line 3307, one line at the top of the page, above the payments | "Value Routes tell podcast apps where to send the payments of listeners." |
    | "Split (shares)" | `values.js` near line 194, the `hint` of the field | "A recipient gets its shares divided by the total shares of the block." |
    | "Payees with no person" | `peoplepages.js` near line 239, the `note` of the fold | "Wallets in your payments that no person in People has." |

2. UI: a hint that is not a `Field` hint uses the class `hint`, as the other hint lines of the same page.
3. Do not change a label or a term. Do not change the API or the core.

## Files

- Change: `crates/api/ui/editor.js`, `crates/api/ui/values.js`, `crates/api/ui/peoplepages.js`.
- New test: `crates/api/ui-tests/hints.test.mjs`. Change tests when a test reads a changed text.

## Checks

Mechanical:

- UI: `hints.test.mjs` reads the source of each file of change 1 and finds each hint text. It also finds no text "The itunes:author of the feed." in `editor.js`.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- The release page with the author hint, the Value Routes page with its line and an open recipient with the split hint.
- The review sheet footer with the podping hint.
- The People page with the open fold "Payees with no person".

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
