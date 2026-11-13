# Studio v1 Task 131: The Nickname Saves At Once, And The Update Sheet Selects Each Value

Decision: [ADR 0194](../adr/0194-the-nickname-saves-at-once-and-the-update-sheet-selects-each-value.md).

Status: Complete on 2026-11-12. The mechanical checks passed.

The screenshot check of "Update from Linked Pages" passed at 1280 px light and 390 px dark, with two people and six values. All six check boxes start selected. A click clears one check box, and the other five stay selected. `$edges()` and `$overflow()` are empty. On the page of a person, the picture and the website suggestions show **Use**, and the wallet suggestion shows **Add**. **Change** in the sheet "Change the Nickname" closed the sheet, the head showed the new nickname, the bar showed "No changes", and the database had the new nickname. Human check step 1, the nickname, passed on 2026-11-12. Step 2, the update sheet, found that two linked pages replace the values of each other. ADR 0196 and task 133 correct it. Step 2 runs again after task 133.

## Goal

**Change** in the sheet "Change the Nickname" saves the nickname at once. The sheet "Update from Linked Pages" has one check box for each value. The page suggestion of the picture and of the website shows **Use**.

## Rules For The Delegate

- Read ADR 0194 and the code below first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change the API or the core. `PATCH /people/{id}` and `POST /people/linked-page-changes` already accept the bodies of this task.

## Work

Put the logic in pure functions of `crates/api/ui/people.js`, so that a Node test can reach it.

1. The nickname (ADR 0194 items 1 and 2).
   - Read `PersonPage` in `crates/api/ui/peoplepages.js` (about line 830 to 1095), and find how it keeps the saved values and the shown values. Read `NicknameSheet` (about line 1095). Read `savedValues`, `pendingKeys` and `saveBody` in `people.js` (about line 910 to 1000).
   - Add `nicknameBody(saved, nickname)`. It gives the body of `PATCH /people/{id}`: the saved `name`, `usual_role`, `usual_group`, `img` and `href`, and the new nickname. It uses the saved values, not the pending values.
   - Add `afterNicknameSave(saved, shown, nickname)`. It gives `{ saved, shown }` with the new nickname in both, and keeps each other value of `shown`.
   - `NicknameSheet`: **Change** calls an async `onChange`. The sheet closes after a success. After a failure, the sheet stays open and shows the error with `ErrorNotice`. The button is not available during the request.
   - `PersonPage`: the `onChange` of the sheet sends `PATCH /people/{id}` with `nicknameBody`, then applies `afterNicknameSave`. The bar of the page does not count the nickname.
2. The update sheet (ADR 0194 items 3 to 5).
   - Read `LinkedPageReview` in `peoplepages.js` (about line 217) and `linkedPageSelection` and `linkedPageChangeLines` in `people.js` (about line 1163).
   - Replace `linkedPageSelection` with a function that gives one key for each value line of each person, for example `"{person_id}:{kind}:{value}"`. Each key starts selected.
   - Add `linkedPageBody(people, selected)`. It gives the `people` list of the request with only the selected values: `img` and `href` become `null` when not selected, and `keys` and `wallets` keep only the selected entries. It leaves out a person with no selected value.
   - `LinkedPageReview`: the name of each person is a label line with no check box. Each value line has its own check box. Keep the small picture and the address of each value. **Update** sends `linkedPageBody`. It is not available when no value is selected.
3. The suggestion button (ADR 0194 items 6 and 7).
   - `pageSuggestions` in `people.js` (about line 1117): the picture row and the website row get `button: 'Use'` and `ariaLabel: 'Use {value}'`. The key and wallet rows keep **Add**.
   - Update each existing test that expects **Add** for a picture or a website.
4. `AGENTS.md` is owned by the main session. Do not change it.

## Checks

Mechanical, in `crates/api/ui-tests/people.test.mjs`:

- `nicknameBody` gives the saved fields with the new nickname, and not a pending name.
- `afterNicknameSave` keeps a pending change of the name, and `pendingKeys` of the result does not include the nickname.
- The selection of the update sheet gives one key for each value, all selected at the start.
- `linkedPageBody` keeps only the selected values, and leaves out a person with no selected value.
- `pageSuggestions` gives **Use** for the picture and the website, and **Add** for a key and a wallet.
- The four checks of `AGENTS.md` pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.
- For each changed file in `crates/api/ui/`, copy it to `$TMPDIR/x.mjs` and run `tools/node --check $TMPDIR/x.mjs`.

Visual, by the main session: ADR 0194 "How This Decision Is Checked".

Human, by the operator: ADR 0194 "How This Decision Is Checked".

## Stop And Ask When

- The nickname save needs an API change.
- The page keeps the saved values in a form that `afterNicknameSave` cannot change without a change of other callers.
- A test of ADR 0170 asserts one check box for each person. Report it. Do not delete it without the main session.

## Report

- The changed functions and components, with file and line.
- The test names.
- The output of the four checks.
