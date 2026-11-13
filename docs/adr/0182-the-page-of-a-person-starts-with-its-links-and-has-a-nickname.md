# ADR 0182: The Page Of A Person Starts With Its Links And Has A Nickname

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, from the mockups in `.studio-dev/mockup-people-3/site/`. The operator selected **Add**, **Add All Suggestions** and the search in "Add a Person". The operator also selected the section sequence, the separate buttons of variant A and the two names. The values in the update sheet and the line of a fill are also choices of the operator.
- Technical choices: items 4, 9, 12, 14, 15 and 16 are choices of the planning model.
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 1, the sequence. [ADR 0156](0156-the-page-of-a-person-lists-where-the-person-is.md) item 1, the tier of "Where this person is". [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md) items 8 and 15, the button of a suggestion.
- Changes: [ADR 0170](0170-update-people-from-their-linked-pages.md) items 1, 2 and 5. [ADR 0169](0169-an-open-credit-can-find-on-music-index.md) item 2. [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 4, the name in the head.
- Uses: [ADR 0163](0163-one-sheet-finds-and-links-a-page-on-music-index.md) items 4 to 11 for the sheet "From Music Index". [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for the pending changes. [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tiers. [ADR 0108](0108-people-data-at-rest-stays-plain.md) item 2 for the data at rest.
- Source: findings 12, 13, 14, 15 and 17 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).
- Changed by: [ADR 0192](0192-add-a-person-has-the-layout-of-the-other-pickers.md). Items 4 to 8: "Add a Person" has the layout of the other pickers, and Studio makes the person only at **Add the Person**.
- Changed by: [ADR 0194](0194-the-nickname-saves-at-once-and-the-update-sheet-selects-each-value.md). Items 1 and 3: the picture and the website have **Use**. Item 13: **Change** saves the nickname at once.

## Context

On 2026-11-12, the operator found these defects on the People pages:

- A page suggestion has **Use**. The button adds a wallet to the person, and the wallet then shows "Not used". Thus "Use" and "Not used" tell the user two different things.
- The sheet "Add a Person" has no search. To link a page, the user adds the person first. The sheet does not show a person that is in People already.
- The section "Linked artist and label pages" is at the bottom of the page. It gives the suggestions in the sections above it, so the user sees the cause last.
- The head shows the name, and the field "Name" in "Credits" edits the same name. The operator wants a short name in Studio and the official name in a credit.
- The sheet "Update from Linked Pages" shows "Picture" and "Website" with no value.
- **Find on Music Index** in an open credit fills the empty fields with no sign.

## Decision

### The Button Of A Suggestion

1. Each page suggestion has **Add**: the picture, the website, the Nostr key, each wallet that matches and each row of the fold "N other recipients". This changes the button **Use** of ADR 0159 items 8 and 15.
2. The button of ADR 0170 item 1 is **Add All Suggestions**. It stays below the rows of the linked pages.
3. A suggestion for a field with a value keeps the rule of ADR 0159 item 8. It is a different value, and **Add** replaces the value of the field as a pending change.

### Add A Person

4. The field "Name" of the sheet "Add a Person" searches People and Music Index while the user types. The search starts after two characters.
5. The sheet shows the group "Already in People". Each row shows the person and has **Open**. **Open** closes the sheet and opens the page of that person.
6. The sheet shows the group "On Music Index", with the artist and label pages, and the hint "Select a page to add the person with a link to it."
7. A result of Music Index adds the person with the name of the field. It then opens the sheet "From Music Index: {title}" of ADR 0163 items 4 to 11.
8. The button at the bottom is **Add "{name}" Without a Page**. With an empty field, it is **Add the Person**, and it is not available.
9. The group "Already in People" finds the name, the nickname and the credit names of each person.

### The Page Of A Person

10. The page of a person shows its sections in this sequence:
    - Identity: the head, then **Find on Music Index**.
    - Identity: "Linked artist and label pages".
    - Identity: "Credits".
    - Identity: "Where this person is", then the row "Check your feeds".
    - Content: "More from your feeds", then "Payment".
    - Details: "Support links".
    - Setup: "Change your feeds".
    - Limits: **Merge with a Person** and **Delete Person**.

    This changes ADR 0111 item 1 and the tier of ADR 0156 item 1.
11. **Find on Music Index** is alone below the head. **Merge with a Person** and **Delete Person** stay at the bottom of the page.

### The Nickname

12. A person has a nickname. It is empty at the start.
13. The head shows the nickname. With no nickname, it shows the name. A pencil button adjacent to it has the accessible name "Change the Nickname". The button opens the sheet "Change the Nickname", with the field "Nickname" and the hint "Only Studio shows the nickname. Studio does not put it in a feed. A new credit uses the field Name in Credits."
14. The People list and the title bar of the page show the nickname. The pickers of ADR 0099 items 19 to 21 also show it. With no nickname, they show the name.
15. The field "Name" in "Credits" is the official name. Its hint is "Studio puts this name in a new credit." Studio uses the name in each credit and in each payment name. The match of ADR 0159 item 13 also uses the name. Studio does not use the nickname for these.
16. **Rename** in "Change your feeds" changes the name in the feeds (ADR 0099 item 9). It does not change the nickname.
17. A merge keeps the nickname of the person that stays. When that person has no nickname, it gets the nickname of the other person.
18. The export of the people (ADR 0098 item 24) gives the nickname. The nickname holds no secret, so it stays plain at rest (ADR 0108 item 2).

### The Update Sheet

19. Each row of the sheet "Update from Linked Pages" shows the value that it adds. A picture shows a small picture and its address. A website and a wallet show their address. This applies ADR 0170 item 5.

### The Fill Of An Open Credit

20. After **Find on Music Index** fills fields of an open credit, a line below the row names the filled fields, for example "Filled from Music Index: Picture address". The line stays until the next edit of the credit. This adds to ADR 0169 item 2.

## Alternatives Considered

- Keep **Use** and change "Not used". Not chosen by the operator: **Add** tells what the button does.
- One row of buttons below the head, with **Merge with a Person** and **Delete Person** (variant B of the mockup). Not chosen by the operator: a destructive button at the top is easy to click by mistake.
- One name only, with the pencil on the field "Name". Not chosen by the operator: the operator wants a short name in Studio and the official name in a credit.

## Consequences

- The core keeps a nickname for each person. A migration adds it, empty.
- The search of "Add a Person" sends a request to Music Index while the user types. Without an answer from Music Index, the group "On Music Index" does not show. The sheet continues to add a person.
- The user sees the linked pages before the suggestions that they give.

## Acceptance Criteria

Mechanical:

- A Node test: the model of a page suggestion gives **Add** for each type of value, and the page gives **Add All Suggestions**.
- A Node test: `pageSequence('person')` gives the sequence and the tiers of item 10.
- A Node test: the model of "Add a Person" gives the groups "Already in People" and "On Music Index", and the button text of item 8. A search finds a person by the nickname and by a credit name.
- A Node test: the head and the People list give the nickname, or the name with no nickname. A new credit from a picker gets the name.
- A Rust test: a person saves and loads with its nickname. The migration gives each person an empty nickname. A merge keeps the nickname of item 17. The export gives the nickname.
- A Node test: each row of the update sheet gives its value.
- A Node test: the fill model gives the names of the filled fields, and an edit of the credit clears them.

Visual, by the planning model:

- The sheet "Add a Person" with results, the page of a person, the sheet "Change the Nickname", the update sheet and an open credit after a fill. The screenshots use the light and the dark theme, at 1280 px and at 390 px.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, the operator adds a person from a real page on Music Index, then gives the person a nickname. A new credit from the picker gets the official name.
