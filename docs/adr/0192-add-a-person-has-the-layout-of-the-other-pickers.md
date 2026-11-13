# ADR 0192: "Add A Person" Has The Layout Of The Other Pickers

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-12 by the operator, through the questionnaire, in the human check of task 118. The operator selected the layout of the other pickers (ADR 0145) from a drawing.
- Technical choices: items 3, 5 and 7 are choices of the planning model.
- Changes: [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md) items 4 to 8, the sheet "Add a Person". [ADR 0163](0163-one-sheet-finds-and-links-a-page-on-music-index.md) item 2, the sequence of the parts of the sheet "Find on Music Index".
- Uses: [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) items 1 to 3, 5 and 7 for the layout and the search. [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md) items 1, 4 and 8 to 15 for the link and the suggestions. ADR 0182 item 9 for the name match of "Already in People".
- Source: the human check of task 118 on 2026-11-12.

## Context

The sheet "Add a Person" of ADR 0182 shows the results of Music Index in the sheet. A click on a result makes the person at once, with the text of the field as the name. Then the sheet "From Music Index" opens. **Back** in that sheet goes to "Find on Music Index" of the new person, which has only **Cancel**. The user cannot go back to a person with no page, and the person is already made.

The sheet "Find on Music Index" puts the fold "Your artist and label pages" below the results. A long list of results pushes the pages of the user out of view.

The sheets "Add a Credit", "Add a Wallet", "Add a Recipient" and "Add a Support Link" have one layout (ADR 0145). A selection fills the form, and nothing is made before the button at the bottom.

## Decision

### Add A Person

1. The sheet "Add a Person" has the layout of ADR 0145 item 1: the form, then the row "Find on Music Index" under the heading "From Music Index", then the lists.
2. The form has the fields "Name", "Picture address" and "Information address". Below the fields, a pending link shows as a row with the title of the page and a button that removes the link.
3. The row "Find on Music Index" opens the search of ADR 0145 item 3. A selection of an artist or label feed puts its title in "Name", and its picture and its website in their fields (ADR 0145 item 5). It also makes the pending link of item 2. A selection makes no person.
4. The lists are folds of ADR 0145 item 2: "Your artist and label pages" (ADR 0159 item 4), then "Already in People" (ADR 0182 item 9). A row of "Your artist and label pages" makes the pending link of item 2. A row of "Already in People" has **Open**, which closes the sheet and opens the page of that person.
5. The button at the bottom is **Add the Person**. It is not available with an empty name. It makes the person with the fields, then the link, and opens the page of the person. Studio makes no person before this button.
6. **Cancel** closes the sheet and makes nothing.
7. The Nostr key and the wallets of the page come after the add, as the suggestions of the linked page (ADR 0159 items 8 to 15). The wallets of other recipients stay in their fold.

### Find On Music Index

8. The sheet "Find on Music Index" of a person shows the fold "Your artist and label pages" above the results. The search field stays at the top (ADR 0163 item 13).

## Alternatives Considered

- Keep the sheets of ADR 0182 and ADR 0163, move the fold of the pages of the user above the results, and make the person only after the sheet "From Music Index". Not chosen by the operator: the sheets stay different from the other pickers.

## Consequences

- "Add a Person" and the other pickers use one layout and one search. The user learns one pattern.
- A person from Music Index gets the Nostr key and the wallets in a second step, on the page of the person. The page suggestions already show them.
- The hint "Select a page to add the person with a link to it." and the button **Add "{name}" Without a Page** go away.

## How This Decision Is Checked

Mechanical:

- A Node test: the model of "Add a Person" gives the fields, the pending link and the label of the button. A selection of a feed fills the fields and the link, and sends no request to make a person.
- A Node test: the button is not available with an empty name.
- An API test or a Node test: **Add the Person** sends the fields, and then the link of the page.
- A Node test: the model of "Find on Music Index" puts the fold of the pages of the user before the results.
- The four commit checks pass.

Visual, by the planning model:

- "Add a Person" with a filled form and a pending link, and "Find on Music Index" with results, at 1280 px and at 390 px, in the light and dark themes. `$edges()` and `$overflow()` are empty.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, the operator adds a person from a real page on Music Index, and also adds a person with no page. Each person is made only at **Add the Person**.
- In "Add a Person", the operator selects a page on Music Index, then goes back, then selects **Cancel**. People has no new person.
