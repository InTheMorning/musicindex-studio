# Studio v1 Task 172: The Result Panel Of A Write

Decision: [ADR 0217](../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 1 to 6.

Status: Open.

## Problem

After a write from Review, Review closes, and no message tells the result (`editor.js:2059`). A feed on a folder with no public address is written, but no listener can get it. The artist does not know this.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Review sheet and write handler | extend | crates/api/ui/editor.js | The write handler near line 2040 closes Review. It shows the panel in place of the diff. |
| Warning notice | reuse | crates/api/ui/editor.js | The form `notice warning` with the `alert` icon near line 1773 (ADR 0191 item 1). |
| Copy button | reuse | crates/api/ui/app.js | `CopyButton` near line 1927. |
| Icon | reuse | crates/api/ui/app.js | `Icon` near line 1899. The icon `check` exists in the icon set. Verify it. If it is not there, use the success icon of the set. |
| Host edit form | extend | crates/api/ui/app.js | `HostCard` near line 2584 has `startEdit`. The hosts view near line 3971 opens a card in edit mode from a view value, for example `{ name: 'hosts', edit: id }`. |

## Changes

1. Add a pure function `writeResult(feed, connector)` to a UI module that has tests. An example is `crates/api/ui/editor.js`, or a new small module in the same folder. It gives `{ title, line, address }` for a feed with `public_url`, or `{ title, line, warning: { title, text } }` for a feed with no public address. The texts are in ADR 0217 items 2 to 4. The local host kind gives "This folder has no public address." Each other kind gives "This file host gives no public address."
2. After a write succeeds, Review stays open. It does the current steps, but it does not close. The steps are `setDiff(null)`, `draft.load()`, `onOpsCleared()`, `onWritten(feed)` and the `ANNOUNCE_RELOAD` event. It shows the result panel in place of the diff. The footer has only **Close**. Focus goes to the panel title.
3. **Put It Online…** closes Review and opens File hosts with the edit form of the host of the write open.
4. Items 2 and 3 do not apply to a change set write, a delete write or a show grant write (ADR 0217 item 6). Verify which code paths these writes use, and leave them as they are.

## Files

- Change: `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css` if a style is necessary.
- Add tests: a file in `crates/api/ui-tests/`, for example `writeresult.test.mjs`.

## Checks

Mechanical:

- The test of `writeResult` examines three cases. A feed with `public_url` gives the line of item 3. A local host with no public address gives the warning of item 4. An S3 host with no public address gives the warning of item 4 for each other host.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots (`.studio-dev/journeys/run.sh`, data `fresh` and `base`):

- The panel after the write of a new album on the default folder: the warning and **Put It Online…**. 1280 px and 390 px.
- **Put It Online…** opens File hosts with the edit form of that host open.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.
