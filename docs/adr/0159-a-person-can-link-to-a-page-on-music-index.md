# ADR 0159: A Person Can Link To A Page On Music Index

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator, from the mockups of the questionnaire.
- Amended: 2026-11-08 by the operator, through the orchestrator session. Item 12 adds the pages to the export of the people.
- Changed by: [ADR 0163](0163-one-sheet-finds-and-links-a-page-on-music-index.md). Items 4 and 5: "Link to an Artist or Label Page" opens the sheet of **Find on Music Index**, and a result makes the link in the sheet "From Music Index".
- Uses: [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md), [ADR 0123](0123-one-type-of-value-has-one-picker.md), [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md), [ADR 0154](0154-the-search-fields-have-a-field-list.md), [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md), [ADR 0147](0147-the-nostr-and-artwork-rows-show-a-preview.md).
- Changes: [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md) items 5 and 6, the select menu of a link. [ADR 0108](0108-people-data-at-rest-stays-plain.md) item 6, which this record examines.

## Context

A person could link only to an artist or label feed of the collection. A collaborator frequently has a page that is not in the collection: an artist feed on Music Index that a different person writes. That page gives a picture, a website, a Nostr key and wallets, and they change over time. Studio did not show these changes on the person.

## Decision

### Two Kinds Of Link

1. A link to a feed of the collection stays as ADR 0132 gives it. The details of the person go to that page, through the credit sheets of ADR 0118.
2. A read-only link goes to an artist or label feed on Music Index that is not in the collection. The details of that page come to the person as suggestions. Studio writes nothing to that feed.
3. A read-only link keeps the feed GUID and the title of the page with the person. Studio does not add the feed to the collection. Releases and the left panel do not change.

### The Picker

4. "Link to an Artist or Label Page" opens one sheet in the layout of ADR 0145. The sheet has the search field of Music Index, with the field list of ADR 0154, and the results that are artist and label feeds. A fold below it holds the artist and label pages of the collection.
5. A result that is a feed of the collection makes a link of item 1. Each other result makes a read-only link. A row of the person shows "read only" for a read-only link.
6. A selection is a pending change until Save (ADR 0116).

### The Suggestions

7. When the page of a person opens, Studio reads each read-only link on Music Index, with no request before the page opens (ADR 0147). It stores no result.
8. A value of the page that is different from the person shows as a suggestion at its field. An example is "New picture on "Ann Bell Music"" with "Use", the button of the suggestions of ADR 0113 at the same fields. "Use" is a pending change until Save.
9. The values are the picture, the website, the Nostr key and the wallets of the page. A wallet suggestion uses the sheet "Add a Wallet" (ADR 0141).
10. With no answer from Music Index, the page shows no suggestion and no error. A link to a feed that Music Index no longer knows shows the note "Music Index does not know this page now".

### The Examination Of ADR 0108

11. ADR 0108 item 6 asks for this examination, because a read-only link keeps data that the feeds of the user do not give. The result: the People data stays plain at rest. The new data is a public feed GUID and its title, and it holds no secret (ADR 0108 item 2). The text of ADR 0108 item 3 stays.

### The Export

12. The export of the people (ADR 0098 item 24) gives each read-only link of a person with its feed GUID, its title and the time of the link. A save keeps the time of each link that stays. The account archive of ADR 0107 holds the pages because it holds the collection.

## How This Decision Is Checked

Mechanical:

- `studio-core`: a read-only link saves and loads with its GUID and title. A link to a feed of the collection stays a link of ADR 0132. The export gives each page, and a save keeps the time of its link.
- `crates/api/ui-tests/`: the model gives the suggestions of a page that differs, and none for a page that agrees.

Visual, by the agent, at 1280 px and at 390 px with touch:

- The sheet with results and the fold.
- A row "read only".
- A suggestion on the page of a person.

Human: the operator links a person to the page of a collaborator. After the collaborator changes a picture, the operator opens the person again.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)). The human test is open.
