# Studio V1 Task 100: Stream Type Fixes

Status: Complete, with its screenshot visual checks on 2026-11-11. For packet 100c, the operator selected [ADR 0173](../adr/0173-a-live-show-problem-shows-at-its-field.md): each problem of a live show shows at its field. The human test of a paste on the iPhone is open. The human test of **Check Stream** with a real Icecast mount is open.

Owner of the rules: [ADR 0162](../adr/0162-studio-fills-and-checks-the-stream-of-a-live-show.md) items 5 and 10. Evidence: [findings of rc.21](../reviews/2026-11-10-rc21-human-test-findings.md). Step 9: [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md).

## Goal

A check fills an empty stream type. The result line and the type warning name the type with the label of the menu. The stream address field does not scroll the page, and a space before the address stops the write.

## Packets

- 100a: an empty type is not a selection (ADR 0162 item 5, finding 4).
- 100b: the menu label in the result line and in the type warning (item 10, finding 2).
- 100c: a paste or a delete in the stream address field scrolls the page (finding 3).
- 100d: a space before the stream address (open step 9 of rc.21).

## Result Of 100c

A drive reproduced finding 3 on 2026-11-11. The cause is the refusal line "This show has no stream address." in the Status section, above the field. The line goes away when the draft saves after a paste, and it comes back after a delete. The page above the field then changes by 35 px at 1100 px and by 43 px at 390 px. Chromium keeps the field still with scroll anchoring. WebKit possibly does not, but no drive examined WebKit. The fix needed a rule for the position of the refusal line (escalation trigger 1). ADR 0173 gives it.

## Files To Inspect

- `crates/api/ui/editor.js`: `runCheck`, `fillType`, `selectedType`, `pickType`, `typeText` and `changeAddress` (around lines 3160 to 3220), and the Stream section (around line 3300).
- `crates/api/ui/live.js`: `typeIsSelected`, `filledType`, `typeMenuState`, `checkLine` and `checkTypeWarning`.
- `crates/api/src/routes.rs`: `check_stream` and `suggest_stream_type` (around line 2888). The answer gives `fill_type`, `fill_label` and `type_warning`.
- `crates/core/src/live.rs`: `STREAM_TYPE_MENU`, `filled_stream_type`, `stream_type_label` and `stream_type_warning` (around line 357). Its refusals of a stream are near line 430.
- `crates/api/src/tests.rs`: the tests of `/api/v1/stream/check` (around line 7117).
- `crates/api/ui-tests/live.test.mjs`: the tests of the type and the check line.
- `docs/releases/0.1.0-rc.21.md`: step 9.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The core gives the warning text and the label. The UI has no table of types (ADR 0162 item 4).
- The warning does not stop Go Live or a write (ADR 0162 item 10).
- Finding 3 has no verified cause. First reproduce it with a screenshot drive. Then fix the cause that the drive shows.
- A fix of item 5 keeps the rule that a type the user selected stays.

## Acceptance Criteria

Mechanical, with Node tests (`live.test.mjs`):

- `filledType` gives the candidate when the current type is empty, also after an earlier selection on the page.
- `filledType` keeps a type that is not empty and was selected.
- `typeIsSelected` does not count an empty type as a selection.
- `checkLine` names the type of the server with the menu label for a type in the menu.
- `checkLine` keeps the server name for a type with no entry. The model takes the label from the answer of the core.

Mechanical, with Rust tests:

- The route `check_stream` gives the label of the server type in its answer.
- A server type with a menu entry gives the same label as the menu.
- `stream_type_warning` names the server type with the menu label, and keeps the name that the server sends for a type with no entry.
- Two names of one type give no warning.

Mechanical, for 100d: a Rust test and a Node test show what step 9 expects. A stream address with a space before it gives a marked field and a refused write. If the check shows that the behavior is already correct, add the test only.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- A check with an empty type: the menu shows the filled type, and the line shows the menu label.
- The type warning with **Use** and the menu label.
- A paste in the stream address field, then a delete. Read the scroll position of the page before and after. The two values are the same.
- A space before the address: the field is marked, and the write stops.

Human, by the operator: **Check Stream** with a real Icecast mount on the desktop and on the server.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- The scroll of finding 3 comes from a notice that appears and goes away. The fix then needs a rule for the place of a notice.
- The first check with no line (finding 4, last sentences) does not reproduce.
