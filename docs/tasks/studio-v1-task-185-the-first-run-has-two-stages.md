# Studio v1 Task 185: The First Run Has Two Stages

Decision: [ADR 0222](../adr/0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) items 6 to 10, and [ADR 0221](../adr/0221-the-welcome-page-and-the-first-steps.md) item 6 for the lines of the rows.

Status: Open.

## Problem

Theme T9, observation O9 and the first-run part of theme T8 of the [analysis of two journeys](../research/2026-11-18-user-journeys/analysis.md):

- The first-run sheet shows three stages and closes after two (`welcome.js:35`).
- The button that makes the page says "Continue" for the temporary row and for the folder form (`app.js:3618`, `app.js:3620`). The S3, SFTP and nsite forms show "Add the Bucket", "Connect" and "Add the Site" (`app.js:902`, `app.js:1037`, `app.js:1271`).
- The library result shows counts of zero (`app.js:1774`).
- The lines of the rows "Nostr site (nsite)", "S3 or Cloudflare R2" and "SFTP server" use "Blossom servers", "storage bucket" and "SFTP" with no explanation (`welcome.js:93`).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Paths of the first run | extend | crates/api/ui/welcome.js | `PATHS` near line 33 and `HOST_ROWS` near line 88. |
| First-run sheet | extend | crates/api/ui/app.js | The sheet near line 3590 and its step line near line 3628. |
| File host forms | extend | crates/api/ui/app.js | `AddHostForm` near line 1387 and `LocalHostForm` near line 759 have `submitLabel`. The SFTP form (near line 1037) and the nsite form (near line 1271) do not. |
| Library result | extend | crates/api/ui/app.js | The result near line 1770. |
| Welcome tests | extend | crates/api/ui-tests/welcome.test.mjs | The tests of the paths and the texts. |

## Changes

1. UI: `PATHS` has two steps for each path: musician "Your name", "Where it lives". Label "Label name", "Where it lives".
2. UI: in step 2 of the sheet, each button that makes the page has the text "Make the Page", and "Making the Page…" while it works:
    - The button of the row "Try it here (temporary)".
    - The submit button of each file host form in the sheet: the folder, SFTP, nsite and S3 forms. Give `AddHostForm` the properties `submitLabel` and `busyLabel`, and pass them to each form. Each form keeps its own texts when it opens outside the sheet ("Add the Folder", "Add the Bucket", "Connect" and "Add the Site").
    - The submit button of the select menu of the connected file hosts already has these texts. Do not change it.
3. UI: the library result shows a row of "Release folders", "Feed files" and "Files moved" only when its count is more than zero. When each count is zero, it shows the line "The folder has no releases yet." in the place of the list. The errors and "Continue" do not change.
4. UI: the lines of three rows of `HOST_ROWS` change. Use the exact texts:

    | Row | Line |
    | --- | --- |
    | "Nostr site (nsite)" | "A site on Nostr. Its pages and songs go to Blossom servers, the file servers of Nostr, signed with your Nostr key." |
    | "S3 or Cloudflare R2" | "A storage bucket, an online folder at Amazon S3, Cloudflare R2 or a similar provider." |
    | "SFTP server" | "Your own server or web hosting account, which you reach with SFTP." |

5. Do not change the API or the core.

## Files

- Change: `crates/api/ui/welcome.js`, `crates/api/ui/app.js`.
- Change tests: `crates/api/ui-tests/welcome.test.mjs`, and each test that reads a changed text.

## Checks

Mechanical:

- UI: `welcome.test.mjs` examines the two steps of each path, the three row lines of change 4 and no word "feed" in them.
- UI: a source test finds "Make the Page" and "Making the Page…" for the temporary row and for the forms in the sheet.
- UI: a function of the library result gives only the counts that are more than zero, and the line of change 3 when each count is zero. Put this function in `welcome.js`, so that the Node test can import it.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- The sheet "Your artist page" at step 1 and at step 2, with two stages and "Make the Page".
- The library result of an empty folder.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
