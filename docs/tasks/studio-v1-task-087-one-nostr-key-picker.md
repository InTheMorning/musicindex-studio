# Studio v1 Task 087: One Nostr Key Picker

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The People Module".
Decision: [ADR 0123](../adr/0123-one-type-of-value-has-one-picker.md).

Status: Complete, with its screenshot visual check. The human test is open.

## Goal

On the page of a person, the user finds a Nostr key by name, as in the editor. "Add a Nostr Key" opens the sheet "Find a Nostr profile". The form "Add a Credit Name" sets its key through the same sheet. The link forms of a person have no plain field for a Nostr key.

## Packets

One packet, 087a. It changes the UI only.

Visual check of 087a, by the planning model, before the commit: the visual check of ADR 0123.

Human test, by the operator, after the deploy: the human test of ADR 0123.

## Packet 087a: The Nostr Key Picker On The Page Of A Person

### Files

Inspect: `crates/api/ui/editor.js` (`NostrRow`, `NostrSheet`, `ProfileBody`), `crates/api/ui/peoplepages.js` (`LinkSheet`, `LinkForm`, the section of the credit names and the keys, `setLinking`), `crates/api/ui/people.js` (`LINK_FORMS`, `linkStart`, `linkFields`, `linkBody`, `linkReady`, the text `NO_KEY`), `crates/api/ui/nostrkey.js` (`keySummary`), `crates/api/ui-tests/people.test.mjs`, `docs/user-guide/` (the page that tells the Nostr keys of a person).

Change: `crates/api/ui/editor.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui-tests/people.test.mjs`, and the user guide page that tells "Add a Nostr Key".

Do not change: the core, the API, the ADRs, `crates/api/ui/styles.css`.

### Architecture

These choices are fixed. Do not change them.

1. `editor.js` exports `NostrRow` and `NostrSheet`. Their behavior in the editor does not change.
2. The entry `keys` of `LINK_FORMS` goes. "Add a Nostr Key" opens `NostrSheet` directly, with `value` null and no `person`, so the sheet shows no choices for the profile name and picture. A selection sends `POST /people/{id}/keys` with `{ npub }`, the body from before, then reads the person again. An error of the request shows in the page with `ErrorNotice`.
3. The entry `credit-names` of `LINK_FORMS` loses its field `npub`, and gets `nostrKey: true`. `LinkForm` shows a `NostrRow` with the label "Nostr key" below the fields of a form with `nostrKey`. The summary is `keySummary` of the name of the form and the key. The selected key goes in the values of the form as `npub`. `linkBody` adds `npub` for a form with `nostrKey`: the key, or `null` with no key.
4. `linkReady` does not need the key for a form with `nostrKey`.
5. The text `NO_KEY` of `people.js` does not change. With this change, the text is correct.
6. The user guide tells that "Add a Nostr Key" searches the profiles by name.

### Acceptance Criteria

Mechanical:

- A Node test: no entry of `LINK_FORMS` has a field with the id `npub` (ADR 0123). Its failure message names ADR 0123 and the fix: use `NostrRow` or `NostrSheet` of `editor.js`.
- A Node test: `linkBody('credit-names', …)` gives `npub` from the values, and `null` with no key.
- The test of `Object.keys(LINK_FORMS)` lists the entries with no `keys`.
- The test of title case and the test of the secret fields still pass.
- The four commit checks pass.

Visual: the visual check of ADR 0123.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0123-one-type-of-value-has-one-picker.md
- docs/tasks/studio-v1-task-087-one-nostr-key-picker.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 087a names.

Goal:
- On the page of a person, "Add a Nostr Key" and the key of "Add a Credit Name" use the Nostr key picker of the editor.

Constraints:
- Follow the Architecture section of packet 087a fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- Each `setState` that combines the old value uses a functional update.
- Each comment and the user guide text are in ASD-STE100 Simplified Technical English.
- The UI has no build step. To check syntax, copy a changed file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.

Do not touch:
- The core, the API, the ADRs, `styles.css`.
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 087a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Review Of 087a

- The first version put the sheet "Find a Nostr profile" in the form of "Add a Credit Name". A form in a form is not correct HTML, and the submit of the search went to the outer form. The code stopped that with a check of the event target. The planning model divided `NostrRow` into the row `NostrKeyRow` and the sheet. The form keeps the row in its group, and the sheet is after the end of the form.
- Visual check: on the page of a person, "Add a Nostr Key" found "Sir HeyCitizen" and "City Radio" by name, and the selection added the key of City Radio. In "Add a Credit Name", a submit of the search did not submit the credit name. The selected key showed in the row "Nostr key", and the credit name got that key. The check used 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` gave no result.
