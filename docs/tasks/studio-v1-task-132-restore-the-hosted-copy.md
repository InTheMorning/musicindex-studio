# Studio v1 Task 132: Restore The Hosted Copy Is Destructive

Decision: [ADR 0195](../adr/0195-restore-the-hosted-copy-is-destructive.md).

Status: Complete on 2026-11-12. The mechanical checks passed. The test fails on the code before the change, because `CONFIRMS.restoreHostedCopy` does not exist. The walk data has no changed file, so the test is the visual check. The human check is open.

## Goal

**Restore** on a changed file row becomes a destructive button **Restore the Hosted Copy…** with a confirmation. It is the first button of its row.

## Rules For The Delegate

- Read ADR 0195, ADR 0193 and the code below first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change the API or the core.
- Do not change `AGENTS.md`, an ADR or a task file.

## Work

1. `crates/api/ui/confirms.js`: add `restoreHostedCopy: (name) => destructive(...)`. Title: `Restore the hosted copy of “${name}”?`. Text: `Studio downloads the copy on the file host and writes it over the changed file in your folder. The changes in the local file are lost.` Action: `Restore the Hosted Copy`.
2. `crates/api/ui/editor.js`, the changed file branch of the library change row (about line 4040):
   - The order is **Restore the Hosted Copy…** (class `button destructive`), then **Use the New File**.
   - Keep `disabled=${off || !hasCopy}` and the text "No copy on a file host".
   - The click calls `askConfirm(CONFIRMS.restoreHostedCopy(name))` and calls `act({ action: 'restore', ... })` only after a true result. Find how `editor.js` already imports and calls `askConfirm` (for example near line 2053), and use the same form.
3. Search `crates/api/ui/*.js` for each other button that writes over local data or deletes local data with no confirmation. Examples to examine: "Download Again", "Forget the File", "Delete" of a hosted copy, "Use the New File", "Read Again". Do not change them. For each, report the file, the line, what it removes, and if a confirmation already exists.
4. `crates/api/ui-tests/confirms.test.mjs`: add a test `restore the hosted copy is destructive and asks first (ADR 0195)`:
   - `CONFIRMS.restoreHostedCopy('a.mp3')` has `role: 'destructive'` and the action `Restore the Hosted Copy`.
   - A scan of `editor.js` finds the button label `Restore the Hosted Copy…`, finds no button with the label `Restore</button>`, and finds `CONFIRMS.restoreHostedCopy` before the `action: 'restore'` call.
   - Each failure message starts with "ADR 0195" and gives the fix.
   - Prove that the test fails on the code before the change. Report how.

## Checks

- The four checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.
- For each changed file in `crates/api/ui/`, copy it to `$TMPDIR/x.mjs` and run `tools/node --check $TMPDIR/x.mjs`.

## Report

- The changed lines, with file and line.
- The list of item 3.
- The test name, the proof, and the output of the four checks.
