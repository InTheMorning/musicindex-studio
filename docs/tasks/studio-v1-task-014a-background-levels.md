# Studio v1 Task 014a: The Background Levels

Parent: [task 014](studio-v1-task-014-editor-layout.md). Decision: [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 11 to 14.
Sequence: the first packet of task 014. It changes only the styles and adds one test.

## Goal

Give the UI three background levels with no borders on the levels. Add a test that keeps each background on a token.

## Files

Inspect: `crates/api/ui/styles.css`, `crates/api/src/tests.rs` (the ADR 0028 test at the end), `crates/api/src/ui.rs` (`text_files`), and `docs/notes/2026-10-22-editor-layout-mockup.html` (its `<style>` element).

Change: `crates/api/ui/styles.css` and `crates/api/src/tests.rs`.

Do not change: the JavaScript files, `index.html`, `crates/core` and the API routes. Do not change the tokens that `styles.css` copies from `search.html` (the first two `:root` blocks).

## Steps

1. After the Studio tokens, add `--level-1`, `--level-2` and `--level-3`. Dark: `var(--bg)`, `var(--surface)`, `var(--surface2)`. Light: `#f2f2f7`, `#ffffff`, `#f2f2f7`.
2. Set the `body` background and the `.main` background to `--level-1`.
3. Use `--level-2` for `.panel` and `.card`, and `--level-3` for `.recipient`, `.entry-row` and an open `.track`. Remove the border of `.panel`, `.entry`, `.track`, `.choice` (keep its selected border), `.notice` (keep `.notice.error`), and each text field, `textarea` and `select`.
4. Give each text field, `textarea` and `select` the `--fill-soft` background, and `--fill` on focus.
5. Replace each other background that names `--bg`, `--surface` or `--surface2` with a level token.
6. Add the test `each_background_uses_a_token` to `crates/api/src/tests.rs`. It reads `styles.css` from `crate::ui::text_files()`. Examine each declaration that starts with `background:` or `background-color:`. Its value must contain no `#`, `rgb(`, `rgba(` or `hsl(`. It must not contain `var(--bg)`, `var(--surface)` or `var(--surface2)`. The test does not examine the custom properties of the `:root` blocks.
7. Give the test a doc comment that names the incident: on 2026-10-22 the operator found six shades of gray hard to read.

## Constraints

- Keep each control a minimum of 44 px high (ADR 0010).
- Keep the `.diff` background. Change it from `var(--bg)` to `var(--level-1)`.
- Do not change a layout, a width or a font.

## Acceptance Criteria

Mechanical:

- `each_background_uses_a_token` passes. It fails when a test edit adds `background: #123456;` to `styles.css`, and its message names ADR 0029.
- The test message is: `ADR 0029 §Background Levels: styles.css sets a background that is not a level, fill or named token: {declaration}. Use --level-1, --level-2, --level-3 or a fill token.`
- The three commit checks pass.

Visual:

- None in this packet. Task 014 has the visual check.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when a background needs a color literal. Stop and report when you cannot see the edge of a control in the light theme after you remove its border.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-014a-background-levels.md
- docs/adr/0029-editor-pages-toolbar-and-three-levels.md
- crates/api/ui/styles.css
- crates/api/src/tests.rs (the last test)
- crates/api/src/ui.rs

Goal:
- Three background levels with no borders on the levels, and a test that keeps each background on a token.

Constraints:
- Follow the steps of the task file in sequence.
- Keep each control 44 px high or more. Change no layout, width or font.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the file do.

Do not touch:
- The JavaScript files, index.html, crates/core, the API routes, the first two :root blocks of styles.css.
- Do not commit.

Acceptance criteria:
- The criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
