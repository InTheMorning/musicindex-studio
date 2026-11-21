# Studio v1 Task 222: The User Guide Follows The Current UI

Decision: no new rule. The pages describe the UI of 0.1.0-rc.30. [ADR 0188](../adr/0188-the-readme-is-a-landing-page-for-an-artist.md) item 7 gives the link test. [ADR 0237](../adr/0237-the-readme-shows-a-few-screenshots-in-the-dark-theme.md) gives the README images.

Status: Done. The main session checked each changed page against the code on 2026-11-21. It removed the section "Start from another release", because the UI has no such action.

## Problem

Many pages of `docs/user-guide/` were last changed in October or early November 2026. The UI changed after that. Examples: "＋ Add a Feed" is now "＋ Add", the tab "Album" is now "Release", the tab "Check" is now "Issues", and "Delete Anyway" does not exist. An audit on 2026-11-21 compared each page with the code of `HEAD` (`f82eef9`). Its results are in `.studio-dev/docs-audit/findings-a.md` and `.studio-dev/docs-audit/findings-b.md`. Git ignores that folder.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| User guide pages | extend | docs/user-guide | The pages stay. Each change corrects the text of a page. |
| Guide index | reuse | docs/user-guide/README.md | It lists each page. Change it only when a page title changes. |
| Link test | reuse | crates/api/ui-tests | The test of ADR 0188 item 7 refuses a broken relative link. |

## Changes

1. For each finding in the two audit files, examine the code at the given line. Use only a finding that the code confirms. Ignore a finding that the code does not confirm, and list it in the report.
2. Correct the page text. Use the UI strings exactly as the code gives them. Keep the structure of each page. Do not add a page.
3. For a finding marked "large omission", add the necessary steps in the section of the page that owns the task. Keep each addition short.
4. Do not change a page that is not in the findings. Do not change `README.md`, `docs/adr/` or code.

## Constraints

- Write in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Keep RSS words out of the guide text where the app does not show them.
- The agent does not commit and does not stage.

## Acceptance Criteria

Mechanical:

- `tools/node --test crates/api/ui-tests/` passes. It includes the link test of ADR 0188 item 7.
- `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics` gives no new error in a changed paragraph, except for UI names and established feed terms.
- Each bold UI name in a changed sentence is a string in `crates/api/ui/`. The main session checks this with `grep`.

Visual: none. The guide has no new image.

## Report

Give the list of changed pages, each finding that the code did not confirm, and `git status --short`.
