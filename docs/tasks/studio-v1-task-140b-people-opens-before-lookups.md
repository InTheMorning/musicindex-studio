# Studio v1 Task 140b: The People Page Opens Before The Recipients And The Lookups

Decision: [ADR 0201](../adr/0201-people-opens-before-the-lightning-address-lookups.md).

Status: Open. Task 140a must be complete first, because this task uses its route.

## Goal

1. The People page shows the list of people before the answer of `GET /recipients` (ADR 0201 item 2).
2. The People page and the page of a person send `POST /recipients/lookups` after the answer of `GET /recipients`. They read `GET /recipients` again only when `checked` is more than 0 (item 6).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/peoplepages.js:137` `PeoplePage` `load`: one `Promise.all` of `/people`, `/people/suggestions`, `/change-sets`, `/recipients` and `/people/suggestions/hidden`, then one `setData`.
- `crates/api/ui/peoplepages.js:148`: `payees` uses `data.recipients`. `:166`: `changeRow` uses `payeesWithNoPerson(data.recipients)`.
- `crates/api/ui/peoplepages.js:644` `usePerson`: one `Promise.all` of `/people/${id}` and `/recipients`. `:967` and `:969` use `recipients` for `walletGroups`.
- `crates/api/ui/peoplepages.js:502` to `:506`: the sheet Add a Wallet reads `/recipients`. It does not change.
- `crates/api/ui/recipients.js`: `payeesWithNoPerson` and the other helpers.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| People page load | extend | crates/api/ui/peoplepages.js | The load reads the recipients after the people. |
| Person load | extend | crates/api/ui/peoplepages.js | `usePerson` sends the lookups after its read. |
| Lookups request | new | | Searched crates/api/ui/*.js and ~/build/musicindex-lib: no client of a lookups route. Add one pure function in `crates/api/ui/recipients.js` that the two pages use. |

## Rules For The Delegate

- Read ADR 0201 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file. Do not change the API.
- If a step needs an architecture decision that ADR 0201 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `crates/api/ui/recipients.js`: add `export async function refreshLookups(api)`. It sends `POST /recipients/lookups`. It gives `true` when the answer has `checked` more than 0. A failed request gives `false` and throws no error.
2. `PeoplePage` `load`: read the four other requests first, and call `setData` with `recipients: null`. Then read `/recipients` and set it. Then call `refreshLookups`. When it gives `true`, read `/recipients` again and set it. Only the answers of the last load set the state.
3. With `recipients: null`, the payees section is not on the page, and `changeRow` gets an empty list. Do not show an error or a "Loading" line for the recipients.
4. `usePerson`: keep the current read. After it, call `refreshLookups`. When it gives `true`, read `/recipients` again and set it. A failed lookups request changes nothing.

## Acceptance Criteria

Mechanical:

1. A Node test of `refreshLookups` with a stub `api`: `{ checked: 2 }` gives `true`, `{ checked: 0 }` gives `false`, and a rejected request gives `false`. Each failure message starts with "ADR 0201" and gives the fix.
2. A Node scan test of `peoplepages.js`: the `Promise.all` of `PeoplePage` does not contain `api('/recipients')`, and `PeoplePage` and `usePerson` call `refreshLookups`.
3. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The People page and the page of a person after the lookups end, at 1280 px light and 390 px dark. Walk: `setup-138.py` with new steps `steps-140-{light,dark}.json`. The steps also record the time of each `/api/` request, as `steps-people-timing.json` does. The list must show before `/recipients` ends.

Human, by the operator: the gate of ADR 0201.

## Report

- The changed lines, with file and line.
- The test names and the output of the four checks.
