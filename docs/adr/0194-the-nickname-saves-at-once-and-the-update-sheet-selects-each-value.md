# ADR 0194: The Nickname Saves At Once, And The Update Sheet Selects Each Value

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-12 by the operator, in the human checks of task 118 and task 128. The operator gave items 1, 3 and 5 in the chat. The operator selected **Use** in item 6 through the questionnaire, from a drawing.
- Technical choices: items 2, 4 and 7 are choices of the planning model.
- Changes: [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md) items 1 and 3, the button of the picture and the website. ADR 0182 item 13, the save of the nickname. [ADR 0170](0170-update-people-from-their-linked-pages.md) item 5, the check boxes of the review sheet. [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) item 1, for the nickname only.
- Source: the human checks of task 118 and task 128 on 2026-11-12.

## Context

The sheet "Change the Nickname" (ADR 0182 item 13) makes a pending change. The user must then select **Save** in the bar of the page. The nickname shows only in Studio and changes no feed, so the second step gives no protection.

The sheet "Update from Linked Pages" (ADR 0170 item 5) has one check box for each person. The user cannot keep the picture of a person and select the wallet.

A person has one picture and one information address. The page suggestion for these fields has **Add** (ADR 0182 item 1), but the button replaces the value of the field (ADR 0182 item 3).

## Decision

### The Nickname

1. **Change** in the sheet "Change the Nickname" saves the nickname at once. The nickname is not a pending change, and the bar of the page does not count it.
2. The save changes only the nickname. Each other pending change of the page stays pending. If the save fails, the sheet stays open and shows the error.

### The Update Sheet

3. The sheet "Update from Linked Pages" has one check box for each value: the picture, the website, each Nostr key and each wallet. The values of one person stay together below the name of the person.
4. Each check box starts selected. **Update** saves only the selected values. A person with no selected value does not change. **Update** is not available when no value is selected.
5. The sheet shows each value as ADR 0182 item 19 gives: a picture shows a small picture and its address, and a website and a wallet show their address.

### The Button Of A Suggestion

6. The page suggestion of the picture and of the website has **Use**. It replaces the value of the field as a pending change (ADR 0182 item 3). The suggestions of a Nostr key and of a wallet keep **Add**, because a person can have many of them.
7. The accessible name of the button follows its label, for example "Use https://wavlake.com/prosperous-soul".

## Alternatives Considered

- **Replace** for the picture and the website. Not chosen by the operator.
- Keep **Add**. Not chosen: the button replaces a value and does not add one.

## Consequences

- The nickname saves with one click, as a rename does in many apps.
- The user can select part of the suggestions of a linked page.

## How This Decision Is Checked

Mechanical:

- A Node test: the model of the nickname save sends the saved fields of the person with the new nickname, and keeps the pending changes of the page.
- A Node test: the pending keys of the page do not include the nickname.
- A Node test: the selection model of the update sheet gives one entry for each value, all selected at the start. The body of **Update** holds only the selected values, and leaves out a person with no selected value.
- A Node test: `pageSuggestions` gives **Use** for the picture and the website, and **Add** for a key and a wallet.
- The four commit checks pass.

Visual, by the planning model:

- The sheet "Update from Linked Pages" with two people and four values, at 1280 px and at 390 px, in the light and dark themes. `$edges()` and `$overflow()` are empty.
- The page of a person with a picture suggestion and a website suggestion: the buttons show **Use**.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, change the nickname of a person. The head shows the new nickname at once, and the bar shows "No changes".
- In "Update from Linked Pages", clear the check box of one value and select **Update**. Only the selected values change.
