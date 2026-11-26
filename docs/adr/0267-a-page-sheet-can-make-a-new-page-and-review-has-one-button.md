# ADR 0267: A Page Sheet Can Make A New Page, And Review Has One Button

- Status: Accepted
- Date: 2026-11-26
- Owner: operator
- Accepted: 2026-11-26 by the operator, from the findings of the [walk of 2026-11-26](../reviews/2026-11-26-human-check-walk.md). The operator selected the words "self-published" (finding 1), the warning in Review with the write still available (finding 11), and the two mockups in the questionnaire: the group "New" at the top of a page sheet (finding 3) and one **Review** button in the top bar (finding 12).
- Technical choices: the main session selected the text of the warning of item 9, the sequence of item 6 and the count of item 7. The operator did not receive them as a question.
- Bar: opposite choice by mistake. Without this record, an agent can keep the words and the three buttons of ADR 0254.
- Changes: [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 11, 24, 25, 27, 31, 32, 35 and 36.
- Changes: [ADR 0257](0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) item 5. The value "Not published" becomes "Self-published" (item 1).
- Uses: [ADR 0261](0261-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1 to 3 for the form of a new page. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1 for the warning style. [ADR 0243](0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) for the change set.

## Context

- ADR 0254 uses "Not published" for a page with no publisher page. An artist who puts the files on a personal file host publishes the music. The words read as "not released".
- The sheet "Sign to a label…" lists only the label pages of the collection. With no label page, the sheet says "No label matches." and gives no next step.
- One change set shows three **Review** buttons: in the notice, at the top of the Catalog panel and in the top bar. The top bar says "No changes" when the open page has no draft of its own, while the change set changes its releases.
- Music Index refuses a release with no item and no channel `podcast:value` (the verifier `v4v_payment` of Stophammer). Studio writes the release and sends the podping, and the user does not learn of the refusal.

## Decision

### The Words For No Publisher Page

1. A page or a release with no publisher page is "Self-published" or "self-published" in each text. These texts said "Not published". The words of a page with a publisher do not change. Examples:
   - The head of a page: "Independent, self-published" and "On Northwind Records, self-published" (ADR 0254 item 11).
   - The first row "Self-published" of a publisher sheet (ADR 0254 item 25).
   - The value "Self-published" of the menu **Publisher** (ADR 0257 item 5).
   - The notice "{name} is now independent, self-published."

### The Group "New" Of A Page Sheet

2. Each sheet of ADR 0254 item 24 with a list of pages starts the list with the group "New". The group has one row: "＋ Make a new label page" for a label question, and "＋ Make a new publisher page" for a publisher question. The group shows also when the collection has pages of that role. The group "Labels" or "Publishers" follows, with the rows of ADR 0254 item 25.
3. The row opens the form of a new page of that role (ADR 0261 items 1 to 3). After the user makes the page, the sheet shows again, with the new page selected and the other choices of the sheet kept. When the user cancels the form, the sheet shows again with no change.
4. The sheet "Go independent…" has no list, and thus no group "New".
5. The button of the sheet "Sign to a label…" is **Sign**. The other buttons of ADR 0254 item 24 do not change.

### One Review Button

6. The notice of a change shows "{name} is now {place}. Not written yet." with the button **Undo** and a close button. It has no **Review** button.
7. While a change set is open, the top bar shows "{N} changes not written" and the button **Review**. This applies to each open page. N is the number of feeds that Review writes. **Review** opens the Review of the change set (ADR 0254 item 36). With no open change set, the top bar shows the draft state of the open page, as before.
8. The top of the Catalog panel has no **Review** button and no count.

### A Warning Before A Write That Music Index Refuses

9. When Review writes a release with no item and no channel `podcast:value`, Review shows a warning for that release. The warning has the style of ADR 0191 item 1. Its text is: "Music Index does not list {title}: it has no track and no payment split. Add a track or a split. You can write it now and add them later." The button **Write** stays available.
10. The rule of item 9 belongs to Stophammer. Studio does not copy other Stophammer rules into Review. When Stophammer changes the rule, a subsequent decision changes item 9.

## Alternatives Considered

- The row "＋ Make a new label page" only when the collection has no label page. Not selected by the operator. With this rule, a user with one label cannot make a second one from the sheet.
- **Review** in the notice only, with the count in the top bar. Not selected by the operator. After the user closes the notice, no button opens Review until the next change.
- A block of the write for a release that Music Index refuses. Not selected by the operator: a release can be written before its tracks exist.

## Consequences

- The tests and the core texts that say "not published" change in the same build.
- The top bar gets the count from the change set, not only from the draft of the open page.
- Review reads the item count and the channel `podcast:value` of each release that it writes.

## How This Decision Is Checked

Mechanical:

- An API test: the place texts of a page and of the notice give "self-published" (item 1).
- A Node test of the sheet model: a label sheet and a publisher sheet start with the group "New", also with pages of that role (item 2). The sign sheet has the button **Sign** (item 5).
- A Node test of the top bar model: with an open change set, the text is "{N} changes not written" and Review opens the change set (item 7).
- An API test: the Review of a release with no item and no channel `podcast:value` gives the warning. A release with one of the two gives no warning (item 9).

Visual (the main session with screenshots):

- The sheet "Sign to a label…" with the group "New", in Light and Dark.
- The notice with **Undo** and the close button, the top bar with the count and **Review**, and the Catalog panel with no **Review**.
- The warning of item 9 in Review.

Human check (the operator):

- In the release binary, make a new label from the sheet "Sign to a label…", sign the artist and write the change.
