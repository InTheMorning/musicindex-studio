# ADR 0099: The People Destination And The Page Of A Person

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Changes: [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 1 to 4 and 21. [ADR 0092](0092-file-hosts-and-the-destination-icons.md), the name and the icon of the first destination of the bottom group. [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) items 6 to 8, 10 to 12, 14 and 15. [ADR 0096](0096-add-to-an-artist-or-label.md), the name of the operation. [ADR 0027](0027-the-recipient-picker.md) items 1 to 3 and 7, for the rows of the picker.
- Changed by: [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md). Items 3 and 4: "Add to a Person", "More from your feeds", "Merge With a Person", and the profile of each key.
- Changed by: [ADR 0102](0102-delete-a-person-changes-no-feed.md). Items 4 and 11: the page of a person gets "Delete Person", and "Remove" reads "Remove From Feeds".
- Uses: [ADR 0098](0098-the-people-module.md) for the data and the operations. Also [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tiers, [ADR 0028](0028-summary-lines-and-one-open-entry.md) and [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) for the rows and the levels, and musicindex ADR 0023 for the words.
- Changed by: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md). Items 3 to 5 and 7: the sheet "Check: {name}", the sequence of the page of a person, "Check your feeds" in the place of "Where used", and "Add to a Person" for a wallet.

## Context

- The operator accepted the mockups in `.studio-dev/mockup-098/site/people.html` on 2026-11-01. The mockups in `versions.html` of the same folder show the current screens and their proposed words.
- A UI review of 2026-11-01 examined the Recipients area. It found these problems:
    - A sheet button such as "Change in 2 feeds" only makes a review.
    - The list says "2 feeds changed" for a change that is not written.
    - The words "node", "keysend", "lnaddress", "custom value", "change set" and `method="lnaddress"` are not words of a musician.
    - The operation buttons are disabled, with no cause, when no feed is selected.
- On 2026-11-01 the operator did not understand the sheet "Add to an Artist or Label". The sheet did not say what it does, and it asked about the recipient first.
- On 2026-11-01 the operator found that the feed list was out of place on the page of a person. It pushed the profile down.
- v4vmm uses the group heading "People" for the artist, the contributors and the label of a track (`src/view_models/track_metadata_grid.rs` and `src/metadata.rs` of v4vmm). musicindex.org shows "Contributors" for the credits of one release. The operator selected "People" on 2026-11-01.

## Decision

### The Destination

1. The bottom group of the left panel starts with "People", in the place of "Recipients". Its icon is the Lucide icon `users`. The sequence is People, File hosts, Settings, then Lock Studio.
2. The People page shows, in this sequence:
    - A search field. It finds a person, an address or a key.
    - The buttons "Add a Person" (action) and "Export People" (normal).
    - "Changes to write", with each change set that is not closed. A row shows the operation, the person or the wallet, and "N feeds to write".
    - "Suggestions from your feeds" (ADR 0098 items 12 to 16). A row shows the name, the counts, for example "Credit in 3 feeds · Payee in 6 feeds", and one line with the evidence. On the first visit, this section comes before "People".
    - "People". A row shows the picture, the name, the usual role with the publisher feed, and the default address. The number of feeds is at the right. With no person, the section shows "No people yet. Check a suggestion above, or add a person."
    - "Payees with no person". A row is a wallet of ADR 0095, with the lines of item 15.
3. A suggestion row opens the sheet "Make a Person: {name}". The sheet lists the credits, the payment addresses and the Nostr keys of the suggestion with check boxes, and the select menu "Member of". An item with a different name in its feed starts not selected. The buttons are "Not This Person" (normal), which dismisses the suggestion, and "Make the Person" (primary). The sheet changes no feed.

### The Page Of A Person

4. The page of a person shows, in the sequence of the tiers:
    - Identity: the picture, the name, the usual role, "Member of" with a link to each publisher feed, and the Nostr key with the result of its check.
    - The buttons "Use This Address", "Rename" and "Add to All Releases" (action), then "Remove" (destructive).
    - One navigation row "Where used", with "Pays in N feeds · Credited in N feeds" and the number of feeds.
    - Content: "Payment", with the default address on its own row ([ADR 0139](0139-the-default-address-is-on-its-row.md)), one group for each wallet, a `lud16` suggestion with its lookup result, and "Add a Wallet".
    - Details: "Credits", with the fields "Usual role", "Picture address" and "Information address".
    - Details: "Support links", with each link and "Add a Support Link".
5. The page "Where used" lists each feed of the person. A row shows the type and the places, the line "Pays {address}", and the line "Credited as {role}". A row opens the feed in the editor.
6. The page of a person has no feed list with check boxes.

### The Sheets Of The Operations

7. Each sheet of an operation of ADR 0098 item 20 lists the feeds of the person with check boxes, below its own fields. Each check box starts selected. Each feed row shows the lines of item 5. Thus no operation button is disabled because no feed is selected.
8. "Use This Address" has the select menu "Address", with each address of each wallet of the person.
9. "Rename" has the field "Name", and the check boxes "Payment name" and "Credit name". Both start selected.
10. "Add to All Releases" starts with one line: "Studio adds {name} to each release of the artist or label that you select." Then it has:
    - The select menu "Artist or label".
    - The check boxes "As a payee", with the field "Percentage", "As a credit" and "Support link".
    - One line with the result, for example "10 releases · 7 get the payee · 7 get the credit · 3 have both already".
    - On the page of a wallet with no person, the recipient details are in one collapsed entry.
11. "Remove" has the check boxes "Payee" and "Credit".
12. The primary button of each sheet reads "Review in N Feeds". The review keeps "Write N Feeds" and "Discard" at its top, and the bar keeps them visible at each scroll position. This changes ADR 0091 item 21.

### The Words

13. A UI text says "changes" for a change set. Examples: "Changes to write", "Discard These Changes?" and "Discard the Changes".
14. A type menu has the items "Lightning Node" and "Lightning Address". A field that edits the custom value of a node is "Account", and the custom key keeps its name "Custom key" with a hint.
15. A line of an address reads "Lightning address {address}", "{provider} node · account {value}", or "Lightning node {key}". This changes the word "Node" of ADR 0095 item 7.
16. The hint of ADR 0095 item 15 reads: "Studio found no payment details for this address at {domain}. Some apps cannot pay it."
17. No UI text shows an XML attribute, such as `method="lnaddress"`.
18. The operation "Add to an artist or label" of ADR 0096 is "Add to All Releases".

### The Pickers

19. The recipient picker of the value editor shows "People" first. A person row shows the address that it adds, which is the default address. Then the picker shows "Payees with no person". An added recipient gets no split (ADR 0030).
20. The credits editor gets "Pick From Your People". A person row shows the usual role and the Nostr key. A pick adds one credit with the name, the usual role, the picture, the information address and the key of the person.
21. The support links editor gets "Pick From Your People", with the support links of each person.

## Alternatives Considered

- The feed list with check boxes on the page of a person. Not chosen by the operator: it pushed the profile down, and on a phone the payment details were below the first screen.
- The feed list as the last section of the page. Not chosen: the buttons at the top and their check boxes at the bottom are far apart.
- Two segments "Profile" and "Feeds". Not chosen: Studio has no segment control on a page.
- The default address as a check box in the sheet "Use This Address". Not chosen: the default changes the picker, not the feeds.
- "Contributors" or "Contacts" as the name. Not chosen: a payee such as a fee is not a contributor, and "Contacts" sounds like the address book of a phone.
- A summary line for each person in place of the address lines. Not chosen: the page of a person shows the addresses, and the row shows the default address.

## Consequences

- The Recipients page goes. Its operations move to the page of a person and to the page of a wallet with no person.
- musicindex ADR 0023 R-WORD-2 item 3 renames the Studio page "Recipients" to "Value routes". With this decision, that page does not exist. The operator decides the change of that rule later, in `../musicindex`.
- The sheets of the operations are longer. On a phone, a sheet fills the screen in all cases.
- The Node tests of the Recipients page change with the screens.

## Acceptance Criteria

Mechanical:

- A Node test: the bottom group gives People, File hosts, Settings, then Lock Studio, and People has the icon `users`.
- A Node test: the model of the People page gives the sections of item 2 in their sequence. With no person, the suggestions come before the people.
- A Node test: the model of the page of a person gives the sections of item 4 in the sequence of the tiers, and no feed check boxes.
- A Node test: the model of each sheet of an operation gives the feed check boxes, all selected, and the button text "Review in N Feeds".
- A Node test: the model of the sheet "Make a Person" starts an item with a different name not selected.
- A Node test: the type menu gives "Lightning Node" and "Lightning Address". The address lines of item 15 and the hint of item 16.
- A Node test: no UI string of the People area has `method=` or "change set".
- A Node test: the recipient picker gives a person row with its default address, and the credits picker gives a credit with the usual role.

Visual, before each commit of a UI packet. The check takes screenshots in the two themes. It runs `$edges()` at 1100 px, and `$fieldText()` and `$overflow()` at 390 px with touch:

- The People page on the first visit and with people.
- The sheet "Make a Person".
- The page of a person, the page "Where used", and the sheets "Use This Address", "Rename", "Add to All Releases" and "Remove".
- The review with its buttons at the top.
- The recipient picker and the credits picker.

Human, on `studio.musicindex.org`: the operator makes a person from a suggestion, and uses each sheet of item 7 one time.
