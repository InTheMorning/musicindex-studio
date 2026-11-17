# Studio v1 Task 157: Research, A Keyboard Shortcut For The Left Panel At A Narrow Width

Decision: none. This is Note 3 of the walk. [ADR 0088](../archive/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 7 and [ADR 0085](../adr/0085-each-artist-and-label-is-a-destination.md) item 7 own the narrow layout. musicindex ADR 0069 R-SHELL-2 owns the rule of the three apps. This task changes no rule and no code. Its result gives the options for a decision of the operator.

Status: Complete on 2026-11-15. The [research note](../research/2026-11-15-keyboard-shortcut-for-the-narrow-shell.md) is ready for the operator decision.

## Result

- The delegate was `gpt-6-luna`. The first run, with medium effort, left most cells of the key table empty. The second run, with high effort, gave the note.
- Gate of the main session: met for the code locations and the ADR items. The main session compared them with the code and with musicindex ADR 0069.
- The main session did not read the web sources. Examine the row of the messaging app in the table of sidebar shortcuts before a decision uses it.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 3.

## Goal

A research document gives the options for a keyboard shortcut that opens and closes the left panel at 760 px or less. The shortcut is for a device with no touch screen. The document first says what the shortcut can show, because the shell has no panel at that width.

## Facts From The Code

- `crates/api/ui/app.js`, `useNarrow`, lines 1832 to 1846: the narrow layout applies at `(max-width: 760px)`.
- `crates/api/ui/app.js`, `Shell`, lines 3973 to 3978: the shell makes the sidebar only when the layout is not narrow.
- `crates/api/ui/shell.js`, `narrowStack` and `levelPages`, lines 802 to 830: at a narrow width, each level is a page of the stack. `backNav`, lines 744 to 760, opens the page above.
- `crates/api/ui/app.js`, `Shell.back`, lines 3721 to 3734, and the history effects, lines 3748 to 3799: Back uses the history of the browser.
- `crates/api/ui/app.js`, `RootPage`, lines 3326 to 3335: the root page has the destinations and the footer.
- `crates/api/ui/shell.js`, `showsEdgeIndicator`, lines 858 to 862: the edge indicator shows only with a coarse pointer.
- `crates/api/ui` has no `keydown` handler on `window` or `document`. Each key handler is local: `onSegmentKey`, `listkeys.js`, `menu.js` and `pickers.js`.

## Questions

1. What can the shortcut show at a narrow width? Give each option: the root page of the stack, the page one level above, or a panel that opens above the page.
2. Which rule does each option change? A panel above the page changes ADR 0088 item 7, and musicindex ADR 0069 R-SHELL-2 can also apply.
3. Which key combinations are free in Chromium, Firefox, Safari and WebKitGTK, and in the Tauri shell on Linux, macOS and Windows? Name each combination that a browser or a screen reader already uses.
4. Which combinations do the usual apps with a sidebar use? Give the source for each one.
5. Do musicindex.org and HitIt have a shortcut for the same panel? musicindex ADR 0045 R-SYSTEM-1 says that the three apps share the system.
6. What does WCAG 2.1.4 (Character Key Shortcuts) require for a shortcut, and how does each option obey it?
7. How does a user learn the shortcut? Give the options, for example a tooltip of the Back button or a line in Settings.
8. Must the shortcut also work at a wide width, where the left panel always shows?

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page stack | reuse | crates/api/ui/shell.js | The research examines `narrowStack` and `backNav`. It changes no code. |
| Shell | reuse | crates/api/ui/app.js | The research examines `Shell` and `RootPage`. It changes no code. |
| Local key handlers | reuse | crates/api/ui/listkeys.js | The research lists the keys that the handlers use. It changes no code. |

## Do Not Touch

- Each file outside `docs/research/`
- Each file of a sibling repository

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write. Write the document in Simplified Technical English.
- Do not commit. Leave the document in the working tree and report it.
- Read `AGENTS.md` of `/home/citizen/build/musicindex-research` first. Read the sibling repositories `/home/citizen/build/musicindex` and `/home/citizen/build/HitIt` only, and change nothing there.
- Give the source for each claim: a file with its lines, or a document with its address and the date of the read. Mark each inference as an inference.
- Give one recommendation in its own section, with its conditions.
- Write no ADR text and no drawing. The main session shows drawings to the operator before a decision.

## Output

One file: `docs/research/2026-11-15-keyboard-shortcut-for-the-narrow-shell.md`, with these sections:

1. Summary, at most ten lines.
2. One section for each option of question 1: what the user sees, the evidence, the good points, the bad points and the rules that it changes.
3. A table of the key combinations against the browsers and the operating systems.
4. The answers to the eight questions.
5. Recommendation, with its conditions.
6. Each item that the research could not find.

## Acceptance Criteria

Mechanical:

1. The file of the section "Output" exists and has the six sections.
2. `git status --short` shows no other changed file in this repository, and no changed file in a sibling repository.
3. `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics` on the file reports no sentence above the word limit.

By the main session, which reads the document. This gate is open until the main session records it:

1. Each claim has a source, and each code location is correct.
2. Each option names the rule that it changes and the repository that owns that rule.

## Test Commands

- `git status --short`
- `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/research/2026-11-15-keyboard-shortcut-for-the-narrow-shell.md`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-157-research-keyboard-shortcut-for-the-narrow-shell.md` (this file), each section
- `docs/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md`, item 7
- `docs/adr/0085-each-artist-and-label-is-a-destination.md`, item 7
- `/home/citizen/build/musicindex/docs/README.md`, then musicindex ADR 0069 and ADR 0071
- The code locations of the section "Facts From The Code"

Goal:
- A research document gives the options for a keyboard shortcut that opens and closes the left panel at 760 px or less, on a device with no touch screen.

Constraints:
- The sections "Questions", "Rules For The Delegate" and "Output" of this file.

Do not touch:
- Each file outside `docs/research/`
- Each file of a sibling repository

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `git status --short`
- `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/research/2026-11-15-keyboard-shortcut-for-the-narrow-shell.md`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
