# Studio v1 Task 201: Overview, Issues And The Person Head

Decision: [ADR 0228](../adr/0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 11 to 13 and 17 to 19.

Status: Open.

## Problem

Findings 3, 5, 6, 7 and 8 of the [rc.28 human check](../reviews/2026-11-20-rc28-human-check.md).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Sections button | extend | crates/api/ui/app.js | Near line 2342 in `EditableFeed`. |
| Tab menu | extend | crates/api/ui/app.js | `PageTabs` near line 2239. The select is near line 2256. |
| Tab label | extend | crates/api/ui/feedcheck.js | `tabLabel` near line 108. Also `PAGE_NAMES` near app.js:2094 and the crumb near shell.js:843. |
| Key evidence | extend | crates/api/ui/people.js | `keyEvidenceText` near line 683. Callers near people.js:660 and peoplepages.js:1093. |
| Find button | extend | crates/api/ui/peoplepages.js | Near line 1096. The role line near line 1091 shows the members. |
| Artist page rows | extend | crates/api/ui/editor.js | `PublisherPage` near line 3947. Release rows near line 4081. |
| Release row style | extend | crates/api/ui/styles.css | `.release-row` near line 2129. |

## Changes

1. The narrow button "Sections" moves out of the bar. It has the label "‹ Overview" and is at the left of the select menu of the tabs, in the same row (item 11). It goes to the outline as before.
2. The tab label is "Issues ({n})", or "Issues" with no finding. The crumb and `PAGE_NAMES` use "Issues" (item 13). The person page "Check your feeds" does not change.
3. `keyEvidenceText` gives no text for a key that the user added. The callers show no separator when the text is empty (item 17).
4. A person with members shows "Find Another Page" as a quiet text button in the role line. A person with no member keeps the button "Find on Music Index" below the role line (item 18).
5. On the artist page, the release rows have an indent below the artist row (item 19). Use a spacing token.

## Checks

Mechanical:

- UI tests: `tabLabel`, `keyEvidenceText`, and the head model for a person with and with no member.
- `bash tools/gate.sh` passes.

Visual (the main session):

- Screenshots at 390 px of a release (the row of "‹ Overview" and the menu) and at 1280 px and 390 px of a person head and an artist page.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
