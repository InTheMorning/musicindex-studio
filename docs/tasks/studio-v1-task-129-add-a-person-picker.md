# Studio v1 Task 129: "Add A Person" Has The Layout Of The Other Pickers

Decision: [ADR 0192](../adr/0192-add-a-person-has-the-layout-of-the-other-pickers.md).

Status: Complete on 2026-11-12. The mechanical checks and the screenshot visual check passed. The walk added a person from a page on Music Index, and the database had one new person, with the fields and the page link. A selection and **Cancel** made no person. A selection of a release gives no values, because a release is not the page of a person. The human check passed on 2026-11-12.

## Goal

The sheet "Add a Person" makes the person at the first click on a Music Index result, and the user cannot go back to a person with no page. After this task, the sheet has the layout of ADR 0145 (`PickerLayout`). A selection only fills the form and a pending link. Studio makes the person only at **Add the Person**. The sheet "Find on Music Index" of a person shows the pages of the user before the results.

## Work

Read ADR 0192, ADR 0145 and the code first:

- `crates/api/ui/peoplepages.js`: `AddPersonSheet` and `AddPersonForm` (about line 300 to 365), `FindForm` (about line 647), `PersonPage` and its `pick` value (about line 761 to 780 and line 1076).
- `crates/api/ui/pickers.js`: `PickerLayout`, `PickFold`, `FindOneSheet`, `SearchResults`.
- `crates/api/ui/people.js`: `addPersonBody`, `ADD_PAGE_HINT`, the view of the add sheet (`view.already`, `view.button`, `view.disabled`), `linkChoices` and `pickLink`.
- The other users of `PickerLayout` and `FindOneSheet` in `crates/api/ui/values.js` and `crates/api/ui/editor.js`, as examples.

Changes:

1. `AddPersonForm` uses `PickerLayout`. The fields are "Name", "Picture address" and "Information address". Below the fields, a pending link is one row with the title of the page and a remove button (ADR 0192 item 2).
2. The row "Find on Music Index" opens `FindOneSheet`. A selection of an artist or label feed puts its title, its picture and its website in the fields, and sets the pending link to that feed (ADR 0192 item 3). It makes no request that makes a person.
3. The lists are `PickFold` folds: "Your artist and label pages" (the rows of `linkChoices`), then "Already in People" (the current name match, with **Open**). A row of "Your artist and label pages" sets the pending link.
4. The footer has **Cancel** and **Add the Person**. **Add the Person** is not available with an empty name. It sends `POST /people` with the fields, then makes the link with the existing request of the page of a person, then opens the page of the person. Remove the `pick` value of the route and its sheet start in `PersonPage`, because no caller uses it after this change.
5. Remove the inline search results of the add sheet, the hint `ADD_PAGE_HINT` and the label **Add "{name}" Without a Page**, with their tests. Keep the name match of "Already in People" (ADR 0182 item 9).
6. `FindForm`: put the fold "Your artist and label pages" above the results (ADR 0192 item 8).

Put the logic in pure functions of `crates/api/ui/people.js`, so that a Node test can reach it.

## Checks

Mechanical, in `crates/api/ui-tests/people.test.mjs`:

- The model of "Add a Person" gives the label **Add the Person**, and the button is not available with an empty name.
- A selection of an artist feed gives the name, the picture, the website and the pending link. A selection sends no request.
- The add plan gives the body of `POST /people` with the fields, and then the link of the page.
- The model of "Find on Music Index" puts the pages of the user before the results.
- The four checks of `AGENTS.md` pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session: ADR 0192 "How This Decision Is Checked".

Human, by the operator: ADR 0192 "How This Decision Is Checked".

## Stop And Ask When

- The link needs a new API route.
- `FindOneSheet` cannot give the feed GUID of a selection without a change of its other users.
- A selection of a feed that is not an artist or label feed needs a different rule.
