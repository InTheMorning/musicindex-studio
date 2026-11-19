# Studio v1 Task 194: The Person Page Has Four Tabs

Decision: [ADR 0226](../adr/0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) items 9 to 12.

Status: Open.

## Problem

Finding 13 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). The person page is a long page with no tabs.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Section sequence | extend | crates/api/ui/pages.js | `PAGES.person` near line 56. The sections of `studio-settings` already name a `tab`. |
| Section tabs | extend | crates/api/ui/pages.js | `settingsTabs` near line 104 groups the sections by tab. Make it a general function, and keep `settingsTabs` as a call of it. |
| Tab control | reuse | crates/api/ui/app.js | `PageTabs` near line 2222 takes `tabs`, `current`, `onSelect` and `label`, with `pending` on a tab. It gives the select menu in a narrow window. Export it if it is not exported. |
| Tab wrapper style | reuse | crates/api/ui/styles.css | `.settings-tabs` near line 1638. |
| Person page | extend | crates/api/ui/peoplepages.js | `PersonPage` near line 855 renders all of `pageSequence('person')` near line 1173. |
| Pending keys | reuse | crates/api/ui/people.js | `pendingKeys` near line 1027 gives one entry for each pending change. |
| Settings page | reuse | crates/api/ui/app.js | `SettingsPage` near line 2952 keeps the open tab in `view.tab`. Use the same pattern. |

## Changes

1. In `PAGES.person`, each section other than `profile` gets a `tab`:
   - `member` and `check`: `profile`.
   - `credits`, `where` and `more`: `feeds`.
   - `payment` and `support`: `payment`.
   - `change` and `delete`: `settings`.
   The section `profile` (the head) has no `tab`. Do not change the sequence or the tiers of the sections.
2. Write one function in `pages.js` that gives `[{ id, label, sections }]` for a page, a tab order and the labels. `settingsTabs` calls it. Add `personTabs()`: Profile, Feeds, Payment, Settings, with the ids `profile`, `feeds`, `payment` and `settings`.
3. `PersonPage` renders these parts in this order:
   - The sections with no `tab`.
   - The error notice and the merge notice, as now.
   - The tabs, in a `div` with the class `settings-tabs`. Use the label "Person tabs" for `PageTabs`.
   - The sections of the open tab.
4. The open tab is `view.tab`. A missing or unknown value opens `profile`. A tab selection calls `go({ ...view, tab })`, as Settings does. Find how `app.js` gives the view to `PersonPage` (near line 4071) and pass `view.tab`. A change of tab keeps the draft.
5. A tab with a pending change gets `pending: true`. Add a function in `people.js` that maps each entry of `pendingKeys` to its tab. Read `CONTROLS`, `PERSON_VALUE_LISTS` and `LIST_NAMES` for the entries. If an entry has no clear section, stop and report it.
6. Change the person tests in `crates/api/ui-tests/pages.test.mjs` near line 148 only if they fail. Add the tests of the Checks.

## Files

- Change: `crates/api/ui/pages.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, and `crates/api/ui/app.js` only for the export of `PageTabs` and the `tab` of the person view.
- Tests: `crates/api/ui-tests/pages.test.mjs`, and a people test file for the map of change 5.

## Checks

Mechanical:

- UI test: `personTabs()` gives Profile, Feeds, Payment and Settings with the sections of change 1. Each tab passes `tierError`.
- UI test: `settingsTabs` gives the same result as before, for a desktop and for a server.
- UI test: the map of change 5 gives a tab for each entry that `pendingKeys` can give.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- Each tab of a person page, with the head above the tabs. At 390 px, the tabs are a select menu.
- A tab with a change that is not saved shows the pencil.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
