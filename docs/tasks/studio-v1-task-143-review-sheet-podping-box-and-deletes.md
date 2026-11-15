# Studio v1 Task 143: The Review Sheet Keeps The Podping Box In View And Names Each Delete

Decision: [ADR 0209](../adr/0209-the-review-sheet-keeps-the-podping-box-in-view-and-names-each-delete.md).

Status: Complete on 2026-11-14, with its screenshot visual check. The human gate of ADR 0209 is open.

## Result

- ChatGPT did the build. The main session read the diff, ran the five checks and took the screenshots.
- `announceLines(diff, enabled, wait)` gives each row with `text` and `hint`. `deleteWrite` gives **Delete 1 File** or **Delete N Files** for a write of only deletions. `refusedDelete` gives the warning row.
- The main session made these corrections:
    - The delegate changed `AnnounceChoice` for each use. The review page of a change set then had the short label, no hint and a call of a function that it did not give. Now `useAnnounceChoice` holds the state, `AnnounceChoice` keeps its form, and `AnnounceFooterChoice` is the footer form.
    - The delegate removed the styles `.host-op-row .title`, `.host-op-row .sub` and `.host-op-row .kind.warning`. The page **Check** uses them. They are back.
    - The delegate removed one assertion of ADR 0198 and changed one comment of ADR 0126. Both are back.
    - The row of a move had each label on its own line, and the file host after the notes. Now a label and its values are one pair.
    - The addresses of the new rows broke inside a word at 390 px. They now have the mark of musicindex ADR 0065 R-FIT-3 item 3.
- The diff does not give the file host of the address before the move. The row uses the file host of the feed row, which the review sheet already has. A feed with no file host shows no name there.
- Walk: `steps-143-usual.json` with `setup-none.py`, `steps-143-move.json` with `setup-142-move.py`, and `steps-143-deletes.json` with `setup-index-follows-later.py`. Each view at 1280 px light and 390 px dark gave no edge, overflow or fit finding.
- At 390 px the footer has three rows: the check box, **Discard the Draft** and **Write to the File Host**. Each control is fully in view.
- The walk `steps-143-refused.json` with `setup-143-refused.py` showed a refused deletion of a pointer file at 1280 px light and 390 px dark, with no finding. The kind "Not deleted" is on one line.

## Goal

1. The podping check box is in the footer of the review sheet, and the row **Announce** has its hint (ADR 0209 items 1 to 5).
2. A value in a row of the review has the normal text color (items 6 to 8).
3. A refused deletion is a warning row with the kind "Not deleted" (items 9 to 12).
4. A write of only deletions has a destructive write button with the count (items 13 to 16).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/editor.js`, `AnnounceChoice` (about line 1826): the check box, its hint and its error notice. The review sheet uses it at the end of its body (about line 2204). `crates/api/ui/app.js` (about line 3065) uses it on the review page of a change set.
- `crates/api/ui/editor.js`, `AnnounceLines` (about line 1847): the rows **Announce** from `announceLines(diff)`.
- `crates/api/ui/announce.js`: `announceHint(enabled, wait)` and `announceLines(diff)`.
- `crates/api/ui/editor.js`, the footer of the review sheet (about line 2206): the destructive write button of a feed delete from `deleteWrite(diff)`, the discard button, and the primary write button.
- `crates/api/ui/feeddelete.js`: `deleteWrite(diff)`.
- `crates/api/ui/editor.js`: `HostOperationRow` (about line 1686), `FeedMoveRow`, `BlockedDeleteRow` (about line 1797) and `ReturnWarning` (about line 1766) with the warning markup.
- `crates/api/ui/styles.css`: `.host-op-row` and its `.sub` rules (about line 2000), `.announce-choice` (about line 2139), `.sheet footer` (about lines 308 and 842).
- The diff of the API: `changed`, `operations` with `kind` and `deletes_feed_file`, `blocked` with `pointer` and `feed_titles`, `feed_move`, `announcements` and `announce_skip`.
- Tests: `crates/api/ui-tests/announce.test.mjs`, `crates/api/ui-tests/feeddelete.test.mjs`, `crates/api/ui-tests/review.test.mjs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Announce choice | extend | crates/api/ui/editor.js | The check box gets a footer form with the short label and no hint. |
| Announce lines | extend | crates/api/ui/announce.js | The rows get the hint as a second line, and a line for the check box off. |
| Delete write view | extend | crates/api/ui/feeddelete.js | `deleteWrite` also gives a view for a write of only deletions. |
| Refused deletion view | extend | crates/api/ui/feeddelete.js | A pure function gives the kind, the tone and the text of the row. |
| Warning markup | reuse | crates/api/ui/editor.js | The icon `alert` and the Warning color of `ReturnWarning`. |
| Review rows | extend | crates/api/ui/editor.js | The rows mark each value and each label. |
| Row styles | extend | crates/api/ui/styles.css | A value has the text color. A label and a note have the muted color. |

## Rules For The Delegate

- Read ADR 0209 first, then ADR 0198, ADR 0193 and ADR 0205 items 19 to 26.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not change a route, the core or the schema. If a step needs a fact that the diff does not give, stop and report.
- If a step needs an architecture decision that ADR 0209 does not give, stop and report.
- `node` is not on the PATH. Use `tools/node`. `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.
- Use the exact words of ADR 0209 for each text. Use tokens only in the styles.

## Changes

1. `crates/api/ui/announce.js`:
    - `announceLines(diff, enabled, wait)` gives one entry for each row, with `text` and an optional `hint`. An announcement row has the hint of `announceHint(true, wait)`.
    - When `enabled` is false and the diff has no skip reason, it gives one row with the text of ADR 0209 item 3. The skip rows do not change.
    - Add `export const ANNOUNCE_FOOTER_LABEL = 'Send a podping'`.
2. `crates/api/ui/editor.js`, the announcement:
    - The review sheet owns the state of the choice (the read and the write of `/settings/announcements`), so that the row at the top and the footer use one value. `AnnounceChoice` keeps its current form for the review page of a change set.
    - `AnnounceLines` shows `text`, and `hint` as a second line in the muted color.
    - The footer of the review sheet has the check box in its own row above the buttons, with `ANNOUNCE_FOOTER_LABEL`. It shows in the same conditions as the check box of today. An error of the save shows in the body of the sheet.
3. `crates/api/ui/feeddelete.js`:
    - `deleteWrite(diff)` keeps its result for a feed delete.
    - A diff of only deletions has one or more operations, each of the kind `delete`. It has `changed` false, no `feed_move` and no `deletes_feed_file`. For it, the function gives `{ label, destructive: true, cancel: null }`. The label is `Delete 1 File` or `Delete N Files`. In each other case it gives null.
    - Add `export function refusedDelete(blocked)`. It gives `{ kind: 'Not deleted', tone: 'warning', text }` with the text of ADR 0209 item 10 or item 11.
4. `crates/api/ui/editor.js`, the footer: a view with `cancel: null` keeps the label **Discard the Draft** in the normal style and its usual disabled rule. The destructive write button is the first button.
5. `crates/api/ui/editor.js`, the rows:
    - `BlockedDeleteRow` uses `refusedDelete`. It has the warning style, the icon `alert` and `role="status"`.
    - `FeedMoveRow` shows "From" and "To" as labels, each address as a value, and the name of the file host as a value below the address that it belongs to. Show the file host of the address before the move only if `feed_move` gives it.
    - `HostOperationRow` shows the file host and the path as values, and the note in the muted color.
6. `crates/api/ui/styles.css`: add the rules for a value, a label and a note of a row, for the warning row, and for the footer row of the check box. Remove each rule that no markup uses. At 390 px, the check box row and each button are fully in view.
7. Update each test that reads a changed text or function. Delete each test of a removed text.

## Acceptance Criteria

Mechanical:

1. A Node test of `announceLines`: a row with its hint for the check box on, the hint with a wait, the row of ADR 0209 item 3 for the check box off, and the two skip rows. Each failure message starts with "ADR 0209" and gives the fix.
2. A Node scan test of `editor.js`: the footer of the review sheet has the check box with `ANNOUNCE_FOOTER_LABEL`, and the body of the review sheet has no `AnnounceChoice`.
3. A Node test of `refusedDelete`: the kind, the tone and the two texts.
4. A Node test of `deleteWrite`: `Delete 1 File` and `Delete 3 Files` for only deletions. Null for a changed feed, a feed move, an upload with a deletion, and no operation. The result of a feed delete does not change.
5. A Node scan test of `editor.js`: `BlockedDeleteRow` has no `role="alert"` and no class `error`.
6. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The review of a usual write, with the check box in the footer, at 1280 px light and 390 px dark.
2. The review of a move, with the labels and the values.
3. The review with a refused deletion of a pointer file.
4. The review of only deletions, with the red button at the left.

Human, by the operator: the gate of ADR 0209.

## Report

- The changed lines, with file and line.
- The test names, and the output of the four checks.
- Each point where the code did not agree with this task.
