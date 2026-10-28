# Studio v1 Task 051c: The Rows, The Sheet And The Setting In The UI

Parent: [task 051](studio-v1-task-051-nostr-keys.md). Decision: [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md) items 16 to 26.

Status: Open.

## Goal

A person entry has the row "Nostr". The first section of the album page, the publisher page and the playlist page has the row "Nostr profile". Each row opens the sheet "Find a Nostr profile". The Settings page has the field "Nostr search relay".

## Files

Inspect: `crates/api/ui/editor.js` (`Persons`, `BLANK_PERSON`, `ChannelFields`, `useSheet`, `AddTrackSheet` for the sheet markup, `TextField`), `crates/api/ui/credits.js`, `crates/api/ui-tests/credits.test.mjs`, `crates/api/ui/pages.js` (`FEED_FIELDS`, `studio-settings`), `crates/api/ui/app.js` (`AnnouncePanel`, the `studio-settings` map near line 2151, `useAction`), `crates/api/ui/account.js` (`accountName`), `crates/api/ui/signing.js` (`npubOf`), `crates/api/ui/draft.js` (`bad`), `crates/api/ui/styles.css` (`.sheet`, `.group`, `.link-row`, `.choice`), `docs/user-guide/edit-album.md`, `docs/user-guide/artist-and-label-feeds.md`.

Change: a new model module `crates/api/ui/nostrkey.js`, a new test `crates/api/ui-tests/nostrkey.test.mjs`, `crates/api/ui/editor.js`, `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/app.js`, `crates/api/ui/draft.js`, `crates/api/ui/credits.js` (only when the key test of step 1 fails), `crates/api/src/ui.rs` (the list of served files), a new guide page `docs/user-guide/nostr-keys.md`, `README.md` and `docs/user-guide/README.md` (the link to the new page). Change `styles.css` only when the visual check needs it, and then say so.

Do not change: the Rust crates other than the list of served files, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `nostrkey.js` has no Preact and no browser state (ADR 0042). It exports:
  - `shortKey(npub)`: the first 8 characters, `…`, and the last 4 characters. An empty value gives `''`.
  - `keySummary(name, npub)`: `"NAME · SHORT"`, or `SHORT` with no name, or "Not set" with no key.
  - `feedKey(model)`: the `value` of the first `txts` entry with `purpose` `npub`, or `null`.
  - `withFeedKey(model, npub)`: a new model. It sets the value of that entry, adds the entry `{ purpose: 'npub', value }` when there is none, or removes the entry when `npub` is `null`.
  - `sourceText(source)`: "In your feeds", "In 1 musicindex feed", "In N musicindex feeds" or "Nostr search".
- `BLANK_PERSON` gets `npub: null`. `Persons` adds, after the "Information address" field, a `button class="link-row"` with `span.label` "Nostr", `span.summary` `keySummary(person.name, person.npub)` and `span.go` `›`.
- `FEED_FIELDS` gets `nostr` after `author`. `ChannelFields` renders it as the same `link-row`, with the label "Nostr profile" and `feedKey(model)`.
- One component `NostrSheet({ open, value, feedId, personName, onSet, onClose })` gives a `dialog class="sheet small"` with `useSheet`:
  - With a key, it shows the profile card of `GET /api/nostr/profiles?q=KEY`, and the buttons "Change" (normal) and "Remove" (destructive). "Change" shows the search.
  - The search is a `form role="search"` with the field "npub, NIP-05 address or name" and the button "Search", as `FindFeeds` has. With a session that has a key, the button "Use my key" sets the key of the session.
  - Each result is a `button class="link-row"`. It shows the picture (40 px), the name and the NIP-05 address with "✓" when `nip05_ok`. `picture` is the address from the profile. `useImage` of `app.js` loads it through the image route. It also shows the short key and the source texts. A result with `domain_match` shows "Confirmed at DOMAIN".
  - `failed` gives one hint, for example "musicindex did not answer."
  - In a person entry with no name or no picture, the sheet shows two check boxes before the results: "Use the profile name" and "Use the profile picture". Their start value is checked when the field is empty.
  - A selection calls `onSet` with the key, and the name and the picture when the boxes are checked. The sheet then closes.
- `draft.js` keeps `bad_keys` of the draft and the save responses. A row with a bad key shows the summary "Not a Nostr key" with the class of a field error. The sheet shows the text as it is.
- The Settings page gets the section `nostr`, tier `setup`, after `announcements`. `NostrPanel` reads and sets `/settings/nostr` as `AnnouncePanel` does. Its field is "Nostr search relay", with the hint "Studio sends the text of a name search to this relay. Leave the field empty to search only your feeds and musicindex."
- `withTrackCredits` keeps `npub`. A test shows it.

## Steps

1. Add a test to `credits.test.mjs`: `withTrackCredits` keeps the `npub` of each person.
2. Write `nostrkey.js` and `nostrkey.test.mjs`.
3. Change `pages.js` and `pages.test.mjs`.
4. Change `editor.js`, `app.js` and `draft.js`.
5. Write the guide page `nostr-keys.md`: what a key is for, how to find a key, the evidence on each result, and the search relay setting. Add the links.
6. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the new guide page.
7. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). The UI sends the key text as the user selected or typed it. The core gives the `npub` form. Name each new function that changes the model in your report.
- The UI connects to no relay and to no other server (ADR 0016). Each picture comes from the image route.
- Each labeled button has its role of ADR 0037.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- `credits.test.mjs` shows that `withTrackCredits` keeps the key.
- `nostrkey.test.mjs` tests `shortKey`, `keySummary` with and without a name and a key, `feedKey`, `withFeedKey` (set, add and remove) and `sourceText` with 1 and 3 feeds.
- `pages.test.mjs` gives `nostr` after `author` in the first section of the three feed pages, and `nostr` after `announcements` on the Settings page.
- The test of ADR 0016 finds no other server in the UI files.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when the session answer has no key for a user who signed in with Nostr.
- Stop and report when `ChannelFields` cannot open a sheet.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md
- docs/tasks/studio-v1-task-051-nostr-keys.md
- docs/tasks/studio-v1-task-051c-nostr-ui.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The rows "Nostr" and "Nostr profile" open the sheet "Find a Nostr profile". The Settings page has "Nostr search relay".

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065). The UI connects to no other server (ADR 0016).

Do not touch:
- The Rust crates other than the list of served files, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
