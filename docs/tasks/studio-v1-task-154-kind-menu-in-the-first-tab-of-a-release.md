# Studio v1 Task 154: The Menu "Kind" Is In The First Tab Of A Release

Decision: [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md) items 13 and 16. The main session changes the two items in the commit of this task, with a `- Revised:` line. Item 13 then gives the position between "Title" and "Artist". Item 16 then gives the hint of this file.

Status: Complete on 2026-11-15, with the screenshot visual check. The visual gate for the operator is open.

## Result

- The delegate was `gpt-6-luna` with medium effort.
- Corrections of the main session: the delegate put the menu in an extra `div`, and the row did not follow the layout of the other fields. The menu is a direct row of the group. The page table and the page **Settings** do not get the release flag, because no code uses it there.
- The main session changed ADR 0213 items 13 and 16 in the commit of this task. The form of a new release keeps the hint "Studio shows this kind. The feed file has no tag for it."
- The gate passed on the tree with this change.
- Screenshots of the main session, at 1280 px and at 390 px, in the light theme and the dark theme: the first tab of a release has the fields "Title", "Kind" and "Artist" in that sequence, with the hint. A change to "EP" changes the tab name, the type label of the head and the left panel. The top bar shows "No changes". The tab **Settings** has no menu "Kind". `$edges()` and `$overflow()` are empty, and the fit check has no finding.
- A playlist and a publisher feed have no screenshot. Only `AlbumPage` gives the field, and the API gives `kind_shown` only for a feed with the medium `music`. This is a check of the source.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 7.

## Goal

The menu "Kind" of a release moves from the tab **Settings** to the first tab, between "Title" and "Artist". A change saves at once, and a hint says so.

## Operator Choices

- Operator choice (2026-11-15): the menu "Kind" is in the first tab of a release, between "Title" and "Artist". The tab **Settings** has no menu "Kind".
- Operator choice (2026-11-15), from drawings: a change of the menu saves at once, as before. The hint below the menu is "Saves at once. The feed file has no tag for the kind."
- The four values of the menu do not change (ADR 0213 item 12).
- A playlist, a publisher feed and a feed with live shows only get no menu (ADR 0213 item 15).

## Code Locations

- `crates/api/ui/pages.js`, lines 14 to 20: `FEED_FIELDS` and the page `album`. Lines 37 to 46: the page `settings`, with the section `kind` for a release.
- `crates/api/ui/editor.js`, `SettingsPage` and `saveKind`, about lines 2727 to 2757: the section "Kind" and the request that saves the kind.
- `crates/api/ui/editor.js`, `ChannelFields`, about lines 2681 to 2697: the fields of the first section of a feed page.
- `crates/api/ui/app.js`, `ChoiceSelect`, about line 1313: the select menu with a label and a hint.
- `crates/api/ui/addfeeds.js`, `RELEASE_KINDS`: the values and the labels of the menu.
- `crates/api/ui-tests/pages.test.mjs`, the test `the release kind section starts Settings and does not appear for a show`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Select menu with a hint | reuse | crates/api/ui/app.js | `ChoiceSelect` gives the label, the menu and the hint. |
| Kind values | reuse | crates/api/ui/addfeeds.js | `RELEASE_KINDS` gives the four values. |
| Page sections | extend | crates/api/ui/pages.js | The field `kind` goes into the first section of a release. The section `kind` of `settings` goes. |
| Kind save | extend | crates/api/ui/editor.js | `saveKind` moves with the menu. The request does not change. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- The API route that saves the kind, and the core rule of ADR 0213 item 11
- The form that makes a new release
- `crates/api/ui/assets/tokens.json`

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- The kind is not a part of the draft. Do not put it in the draft, in Review or in the state "Saved, not written".
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In `pages.js`, give the first section of the page `album` the field `kind` between `title` and `author`, only for a release that has a kind. Use the option `release` that `settings` uses today. Remove the section `kind` from `settings`.
2. In `editor.js`, show the menu at the field `kind` of the first section, with `ChoiceSelect`, `RELEASE_KINDS` and `saveKind`. Remove the section "Kind" from `SettingsPage`.
3. Set the hint to "Saves at once. The feed file has no tag for the kind."
4. Keep the update of the type label after a save (ADR 0213 item 14): the head of the feed page, the tab name and the left panel show the new kind with no reload.
5. Replace the test of the Settings position with the tests of this file. Change each comment that cites ADR 0213 item 13 for the old position.

## Acceptance Criteria

Mechanical:

1. A Node test in `crates/api/ui-tests/pages.test.mjs`: the first section of the page `album` for a release has the fields `title`, `kind`, `author` in that sequence. With `release: false`, it has no field `kind`. The failure message starts with "ADR 0213 item 13" and gives the fix.
2. A Node test: `pageSequence('settings', { album: true, release: true })` has no section `kind`. The failure message starts with "ADR 0213 item 13".
3. A Node test reads `editor.js`: the hint text is "Saves at once. The feed file has no tag for the kind.", and the old hint text does not occur. The failure message starts with "ADR 0213 item 16".
4. The test `the release kind section starts Settings and does not appear for a show` does not exist.
5. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. The first tab of a release at 1280 px and at 390 px, in Light and in Dark: the menu "Kind" is between "Title" and "Artist", with its hint.
2. A change from "Album" to "EP" changes the tab name, the head of the page and the left panel. The top bar does not show "Saved, not written".
3. The tab **Settings** of a release has no menu "Kind". A playlist and a publisher feed have no menu "Kind".

## Test Commands

- `tools/node --test crates/api/ui-tests/pages.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-154-kind-menu-in-the-first-tab-of-a-release.md` (this file), each section
- `docs/adr/0213-a-release-starts-on-the-page-of-its-artist.md`, items 10 to 16
- `crates/api/ui/pages.js`
- `crates/api/ui/editor.js`
- `crates/api/ui/app.js`
- `crates/api/ui/addfeeds.js`
- `crates/api/ui-tests/pages.test.mjs`

Goal:
- The menu "Kind" of a release moves from the tab **Settings** to the first tab, between "Title" and "Artist". A change saves at once, and a hint says so.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- The API route that saves the kind, and the core rule of ADR 0213 item 11
- The form that makes a new release
- `crates/api/ui/assets/tokens.json`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `tools/node --test crates/api/ui-tests/pages.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
