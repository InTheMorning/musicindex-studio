# ADR 0218: A Page Link In Review, And A Suggestion Starts With No Person

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. For T4, the operator selected "Link in Review". For T6, the operator selected "Select first".
- Technical choices: the main session selected items 4, 6, 7, 9, 12 and 13.
- Bar: opposite choice by mistake. An agent can send the user out of Review to link a page, or start a suggestion on "A new person". Items 1 to 7 also cross the editor, the Check tab and the result panel of ADR 0217.
- Changed by: [ADR 0251](0251-review-writes-the-two-sides-of-a-changed-link.md) item 13, on 2026-11-23. Items 3 and 6: the menu always shows, and its last choice "New artist page…" opens the form of item 6. The button **Make an Artist Page…** is removed.
- Changes: [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) items 18, 19 and 21. Review links the page itself, and the write button shows disabled.
- Changes: [ADR 0203](0203-people.md) item 39. The menu "Person" starts on "Select a person" when no person has the name of the suggestion.
- Changes: [ADR 0217](0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) item 2. The result panel tells a page that the link changed.
- Uses: [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) items 5 to 8 for the links of a publisher feed. [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) for the findings. [ADR 0007](0007-draft-diff-write-and-announce.md) for Review.
- Source: [analysis of two journeys for a new independent artist](../research/2026-11-18-user-journeys/analysis.md), themes T4 and T6, and its section "Operator choices".

## Context

- **T4.** Review stops the write of a release with no artist page (ADR 0213 item 21). The button **Link a Page** closes Review and goes to the section "Linked artist and label pages". The user then links the page, opens Review again and writes.
- **T4, more facts.** The link also changes the draft of the artist page, and the user must write that page separately. Walk J04 counts 8 steps, and 11 steps with the write of the artist page. While the refusal shows, the write button is not there. The Check tab shows the same condition as a "Notice". With no artist page in the collection, the section shows only "Studio has no other artist or label page to link to." (`editor.js:2614`).
- **T6.** A suggestion opens the sheet "Check: {name}". Its menu "Person" starts on "A new person", unless a person has the name of the suggestion (ADR 0203 item 39). The note of the fold says "Check each one, then make a person." The fastest path makes a new person, also when the name belongs to a person that exists.

A release that starts on the page of its artist has the link (ADR 0213 item 1). A new artist meets T4 mainly with a release from an import.

## Decision

### T4: The Link In Review

1. When Review shows the page refusal of ADR 0213 item 21, the refusal panel holds the link. The panel has the title of ADR 0213 item 18 and one line of explanation. It also has the select menu of the pages and the button **Link and Continue**.
2. The texts:

   | Refusal | Line | Menu label | Write line |
   |---|---|---|---|
   | No artist page | "Music Index lists each release under its artist." | "Artist page" | "Link an artist page first." |
   | No artist, label or hosting page | "Music Index lists each feed under its page." | "Page" | "Link a page first." |

3. The menu has the placeholder "Select a page" and the same pages as the menu "Link to an artist or label page" of the feed. **Link and Continue** is disabled until a selection.
4. **Link and Continue** makes the same link as **Link to This Page** of the section. Then Review reads the diff again. When the refusal goes, the write button works.
5. While the refusal shows, the write button shows disabled. The write line of item 2 shows above the footer.
6. When the collection has no page for the menu, the panel has the button **Make an Artist Page…**, not the menu. It opens the form that makes an artist page, with the name from the field "Artist" of the release. After the page is made, Studio links the release to it, and the user is on the release with Review open.
7. A link in Review can also change the draft of a page. Then the result panel of ADR 0217 has the line "{page title} also changed." under its first line, with the button **Open {page title}**.
8. The finding of ADR 0213 item 18 on the tab **Check** gets the line "Studio cannot write it before the link." The level and the button **Link a Page** stay.
9. The section "Linked artist and label pages" does not change. Its button **Link to This Page** stays.

### T6: A Suggestion Starts With No Person

10. The menu "Person" of the sheet "Check: {name}" starts on "Select a person" when no person has the normal name of the suggestion. A credit name of a person counts as its name. When a person has it, the menu starts on that person, as today.
11. The menu choices are "Select a person", then "A new person", then each person from A to Z.
12. With "Select a person", the sheet shows the menu and **Hide This Suggestion**. The primary button reads **Add the Items** and is disabled. The other fields show after a selection.
13. The sheet "Add to a Person: {name}" of a payee follows items 10 to 12. Its primary button with no selection reads **Add the Wallet** and is disabled.
14. The note of the fold "Suggestions from your feeds" is "Each name is a credit in your feeds. Add it to a person that you have. Make a new person only for a person that is not in the list."

## Alternatives Considered

- T4: a link in Review and one button that writes the release and the page. Not selected. One button makes two writes, and a failed second write gives a mixed result.
- T4: a better explanation, with the trip to the section. Not selected. The journey stays at 8 to 11 steps.
- T4: a warning in place of the stop. Not offered. The operator selected the stop on 2026-11-14 (ADR 0213).
- T6: Studio starts on a person that shares a wallet, a key or a website. Not selected. It needs new matching in the core, and a wrong guess adds items to the wrong person with no undo.
- T6: a better note only. Not selected. The fastest path still makes a new person.

## Consequences

- A user with no artist page can make one from Review. The release then has a link, and the artist page has a draft to write.
- The write of the artist page stays a separate write. The result panel tells it.
- Each new person from a suggestion needs one more click: the selection of "A new person".

## How This Decision Is Checked

Mechanical:

- A UI test examines the view of the page refusal: the texts of item 2 and the state of **Link and Continue** with and without a selection.
- A UI test examines the line of item 7 in the result panel.
- A core test examines the line of item 8 in the finding.
- A UI test examines the start state of the sheet "Check: {name}" with and without a person of the same name. It also examines the choices of item 11 and the button of item 12. A second test examines the payee sheet of item 13.

Visual, by screenshots of the main session before each commit:

- Review with the refusal panel before and after a selection, and after **Link and Continue**, at 1280 px and 390 px.
- The panel with **Make an Artist Page…**, and the result panel with the line of item 7.
- The sheet "Check: {name}" on "Select a person", and after the selection of "A new person".
