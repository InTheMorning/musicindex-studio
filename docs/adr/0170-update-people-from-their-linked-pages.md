# ADR 0170: Update People From Their Linked Pages

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, during the rc.21 human test. The operator asked for one button that updates the linked people from their publisher feeds in a batch.
- Technical choices: items 1 to 6 are choices of the planning model. The screenshots of the task show the button and the review to the operator before the commit.
- Uses: [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md) items 8, 9 and 13 to 15, [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md), [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md).
- Changed by: [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md). Items 1, 2 and 5: the button is **Add All Suggestions**, and each row of the update sheet shows its value.
- Changed by: [ADR 0194](0194-the-nickname-saves-at-once-and-the-update-sheet-selects-each-value.md). Item 5: the review sheet has one check box for each value.

## Context

A person with a linked page gets suggestions from that page (ADR 0159). They are a picture, a website, a Nostr key and the wallets with a matching name. The user must select **Use** on each suggestion of each person. With many people, that is slow.

## Decision

1. The page of a person with suggestions from its linked pages has the button **Use All Suggestions**. It applies each suggestion of the linked pages as a pending change. The user then saves or discards them (ADR 0116).
2. **Use All Suggestions** uses only the wallets that match (ADR 0159 item 13). The other recipients in the fold stay suggestions.
3. A page suggestion has no hidden state. A value that the person has already is not a suggestion (ADR 0159 item 8), so the button does not change it.
4. The People page has **Update from Linked Pages** when one or more people have a linked page.
5. **Update from Linked Pages** reads the linked pages of each person when the user selects it, with no request before (ADR 0159 item 7). It then opens a review sheet. The sheet lists each person with the values that change. Each person has a check box, and all start selected.
6. **Update** in the sheet saves the selected people. The change of a person changes no feed. The feeds change only through the operations of the person, as before.

## How This Decision Is Checked

Mechanical: a Rust test gives the changes of the batch for two people. It leaves out a value that the person has already and a wallet that does not match. It saves only the selected people.

Visual, by the agent: the button on the page of a person, the button on the People page and the review sheet. The screenshots use the light and the dark theme, at the desktop width and at 390 px.

Human, by the operator: a batch update of people that link to real pages.
