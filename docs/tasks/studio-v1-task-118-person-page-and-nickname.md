# Studio v1 Task 118: The Page Of A Person Starts With Its Links And Has A Nickname

Decision: [ADR 0182](../adr/0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md).

Status: Not started.

## Goal

A page suggestion has **Add**. "Add a Person" searches People and Music Index. The page of a person starts with its links and "Credits". A person has a nickname that only Studio shows. The update sheet shows each value, and a fill of an open credit names the filled fields.

## Mockups

The operator selected the mockup in `.studio-dev/mockup-people-3/` (ignored by Git, on the computer of the operator). Variant A is the selected variant:

- `site/index.html` and `site/people3.js`: the pages and the sheets. Copy the layout, not the code.
- `shots/person-*.png`: the page of a person. It shows the head, the pencil, **Find on Music Index** and the linked pages. It also shows **Add All Suggestions**, "Credits" and the rows with **Add**.
- `shots/person-buttons-*.png`: variant B, with a row of buttons below the head. The operator did not select it. Do not make it.
- `shots/add-*.png` and `shots/add-empty-*.png`: the sheet "Add a Person" with results and with an empty field.
- `shots/rename-*.png`: the sheet "Change the Nickname".
- `shots/update-*.png`: the sheet "Update from Linked Pages" with the values.
- `shots/fill-*.png`: an open credit after a fill, with the line "Filled from Music Index: …".

The code uses the tokens and the components of `crates/api/ui/styles.css`.

## Constraints

- No feed rule in the UI (ADR 0065). The core gives the name for a credit and a payment name. The nickname does not go into a feed (ADR 0182 item 15).
- The nickname stays plain at rest (ADR 0108 item 2).
- The pending changes use Save and Discard of ADR 0116.
- No test sends a request outside the computer (ADR 0045). Use the test server of Music Index for the search.
- Without an answer from Music Index, the group "On Music Index" does not show, and the sheet continues to add a person.

## Files To Read First

- ADR 0182, ADR 0159 items 8, 13 and 15, ADR 0163 items 4 to 11, ADR 0169 item 2 and ADR 0170.
- `Person` and the merge and the export in `crates/core/src/people.rs`. The People migrations in `crates/core/src/`.
- `crates/api/ui/people.js`: the suggestion models, `useAllPageSuggestions` and the fill values of an open credit.
- `crates/api/ui/peoplepages.js`: the People list, the sheet "Add a Person", the page of a person and the buttons **Use**.
- `crates/api/ui/pages.js`: `pageSequence('person')`.
- `crates/api/ui/pickers.js`: the pickers of ADR 0099 items 19 to 21.
- `crates/api/ui/editor.js`: the fill of an open credit (ADR 0169).
- `crates/api/ui-tests/people.test.mjs` and `crates/api/ui-tests/pages.test.mjs`.

## Packets

### 118a: The Nickname In The Core

ADR 0182 items 12, 15 to 18.

- Add `nickname` to `Person`, empty at the start. Add a migration that gives each person an empty nickname.
- The API routes of a person read and save the nickname.
- A merge keeps the nickname of the person that stays. When it is empty, the person gets the nickname of the other person.
- The export of the people gives the nickname.
- Credits, payment names and the match of ADR 0159 item 13 use the name only.
- Add the Rust test of the ADR.

### 118b: The Nickname In The UI

ADR 0182 items 13 to 16.

- The head shows the nickname, or the name with no nickname. A pencil button with the accessible name "Change the Nickname" opens the sheet "Change the Nickname". The sheet has the field "Nickname" and the hint of item 13.
- The People list, the title bar and the pickers show the nickname, or the name with no nickname.
- The field "Name" in "Credits" gets the hint "Studio puts this name in a new credit."
- **Rename** in "Change your feeds" does not change the nickname.
- Add the Node test of the head, the list and a new credit from a picker.

### 118c: The Section Sequence And Add

ADR 0182 items 1 to 3, 10, 11 and 19.

- Change `pageSequence('person')` to the sequence and the tiers of item 10. **Find on Music Index** is alone below the head.
- Change each **Use** of a page suggestion to **Add**, with the accessible name "Add {value}". This includes each row of the fold "N other recipients".
- Change "Use All Suggestions" to **Add All Suggestions**, below the rows of the linked pages.
- Each row of the sheet "Update from Linked Pages" shows its value. A picture shows a small picture and its address. A website and a wallet show their address.
- Change the Node tests that the ADR changes. Add the Node tests of the ADR.

### 118d: The Search Of Add A Person

ADR 0182 items 4 to 9.

- After two characters, the field "Name" searches People and Music Index. Use the search of Music Index that the sheet of ADR 0163 uses.
- The group "Already in People" finds the name, the nickname and the credit names. Each row has **Open**, which closes the sheet and opens the page.
- The group "On Music Index" shows the artist and label pages with the hint of item 6. A selected page adds the person with the name of the field. It then opens the sheet "From Music Index: {title}".
- The bottom button is **Add "{name}" Without a Page**. With an empty field it is **Add the Person**, and it is not available.
- Add the Node test of the ADR.

### 118e: The Line Of A Fill

ADR 0182 item 20.

- After **Find on Music Index** fills fields of an open credit, a line below the row names the filled fields.
- The next edit of the credit clears the line.
- Add the Node test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0182.

Visual, by the planning model:

- The visual criterion of ADR 0182, at 1280 px and at 390 px, in the light and dark themes. Compare each screenshot with the mockup.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0182, in the desktop app.

## Stop And Ask When

- A credit or a payment name must use the nickname for a current rule.
- The search of Music Index cannot run while the user types without a new route.
