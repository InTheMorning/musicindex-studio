# ADR 0203: People

- Status: Accepted
- Date: 2026-11-14
- Owner: operator
- Accepted: 2026-11-14. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0098](../archive/adr/0098-the-people-module.md), The People Module.
- Supersedes: [ADR 0099](../archive/adr/0099-the-people-destination-and-the-page-of-a-person.md), The People Destination And The Page Of A Person.
- Supersedes: [ADR 0102](../archive/adr/0102-delete-a-person-changes-no-feed.md), Delete A Person Changes No Feed.
- Supersedes: [ADR 0103](../archive/adr/0103-one-person-collects-the-items-of-many-suggestions.md), One Person Collects The Items Of Many Suggestions.
- Supersedes: [ADR 0111](../archive/adr/0111-the-page-of-a-person-checks-the-feeds.md), The Page Of A Person Checks The Feeds.
- Supersedes: [ADR 0120](../archive/adr/0120-the-check-of-a-person-shows-each-difference.md), The Check Of A Person Shows Each Difference.
- Supersedes: [ADR 0156](../archive/adr/0156-the-page-of-a-person-lists-where-the-person-is.md), The Page Of A Person Lists Where The Person Is.
- Supersedes: [ADR 0158](../archive/adr/0158-a-hidden-suggestion-can-come-back.md), A Hidden Suggestion Can Come Back.
- Changed: 2026-11-15 by the operator, through musicindex ADR 0075 (musicindex `bf58554`). Item 62 uses the shares method of R-VALUE-5. A fee share is a share in the total, not a percent.
- Changed by: [ADR 0217](0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 12 to 17, on 2026-11-18. Item 52: the menu "Person to merge" has no first selection, and a merge has an undo.
- Changed by: [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 10 to 14, on 2026-11-19. Item 39: the menu "Person" starts on "Select a person" when no person has the name of the suggestion.
- Uses: [ADR 0091](../archive/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) for the change sets. [ADR 0095](../archive/adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md) for the wallets. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules. [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the network policy. [ADR 0028](0028-summary-lines-and-one-open-entry.md) for the entries. [ADR 0143](0143-a-feed-row-names-the-tracks-of-a-person.md) for the places.
- Uses: these current ADRs own a part of the People area. This ADR names each one at its item. [ADR 0112](0112-a-credit-role-comes-from-a-list.md) (the usual role), [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) (Save and Discard), [ADR 0123](0123-one-type-of-value-has-one-picker.md) (the pickers), [ADR 0132](0132-a-person-can-be-part-of-many-artists-and-labels.md) (Part of), [ADR 0138](0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) (the address words), [ADR 0139](0139-the-default-address-is-on-its-row.md) (the default address), [ADR 0140](0140-a-credit-name-shows-where-it-is-used.md) (the credit names), [ADR 0141](0141-the-form-of-add-a-wallet-comes-first.md) (Add a Wallet), [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) (the sheet layout), [ADR 0146](0146-link-words-already-marks-and-credit-edits.md) (the link words), [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md) (the linked pages), [ADR 0170](0170-update-people-from-their-linked-pages.md) (Update from Linked Pages), [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md) (the sequence and the nickname), [ADR 0192](0192-add-a-person-has-the-layout-of-the-other-pickers.md) (Add a Person), [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md) (the button order), [ADR 0196](0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md) (the pictures and the websites), [ADR 0201](0201-people-opens-before-the-lightning-address-lookups.md) (the lookups).
- Changed by: [ADR 0241](0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md). Item 36: the bottom group also has "Switch Collection…" on a desktop with two collections, and "Sign Out" on a server.
- Revised: 2026-11-22. Item 6 states that equal credits of a feed give one row of the index. The rule was in the Context of ADR 0120 and had no item. The behavior does not change.

## Context

- People exists so that a user can join the scattered identities of one person: the payees, the credits, the Nostr keys and the support links of the feeds. The [People module design note](../research/2026-11-01-people-module-design-note.md) gives the jobs J1 to J11 and the answers of the operator of 2026-11-01.
- Eight ADRs from 2026-11-01 to 2026-11-08 made the People area. Each subsequent ADR changed some items of an earlier ADR. On 2026-11-14 a reader had to merge these ADRs item by item to find one rule.
- The global decision of 2026-11-14 (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR is that ADR for People.
- The superseded ADRs keep their context, their alternatives and their checks. They are in `docs/archive/adr/` for research. The item map at the end of this ADR gives the new item of each old item.

## Decision

### The Module

1. The core has the module `people`. It keeps the people of the collection of one user. It has no feed rule and no HTTP server code ([ADR 0002](0002-rust-core-json-api-browser-ui.md)).
2. `people` writes no feed. A change of a feed from a person is a change set of ADR 0091. Each feed writes through `pipeline::write_draft` with the ID of its own diff.
3. A value from a person is a start value in a feed. The feed keeps its own copy, so a change of a person does not change a feed. This is an edit aid (ADR 0065).
4. The module serves the desktop shell and the server shell. On the server, each user folder has its own collection, and thus its own people ([ADR 0055](../archive/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).
5. The collection keeps the people and their links in tables of their own. The tables hold no secret.
6. The core keeps an index of the credits of each feed, as `feed_recipients` keeps the recipients ([ADR 0027](../archive/adr/0027-the-recipient-picker.md) item 4). It has the name, the role, the group, the information address and the `npub` of each credit. Equal credits of a feed give one row of the index. It also has the places of each credit: the channel, and the GUID, the number and the title of each item. The index of ADR 0143 keeps the split of each place. The core refreshes the indexes after each read and each write. A run that applies a change refills them from the library copies (ADR 0143 item 4).
7. A migration made one person from each saved recipient, with its name and its wallet. After the migration, `saved_recipients` has no row.

### A Person

8. A person has a name, a nickname (ADR 0182 items 12 to 18), a usual role with a usual group (ADR 0112 item 7), a list of pictures and a list of websites. Each list has one default value (ADR 0196).
9. A person has these links. The user makes each link:
    - Wallets. A wallet keeps the identity of ADR 0095 item 1, and the keysend lookup links its addresses. A wallet is in a maximum of one person.
    - Nostr keys, in the `npub` form.
    - Credit names. A credit name is an identity: the `npub`, else the normal name with the information address, else the normal name. A credit of a feed with that identity is a credit of the person.
    - Support links, each with a label and a URL.
    - Links to publisher feeds that the person is part of. A person can have many (ADR 0132).
    - Links to artist and label pages (ADR 0159).
10. A person has a default address. It is one address of one of its wallets. The first address of the first wallet is the default until the user selects **Make Default** (ADR 0139).
11. An artist or a label is its publisher feed, not a person.
12. The core gives a wallet of a person its type from its address, at each write and at each read. An address with `@` is a Lightning address, and each other address is a node key. A form cannot store a node key as a Lightning address (the [local use findings](../reviews/2026-11-06-local-use-findings.md), finding 19).

### Suggestions

13. The core suggests people from the index of the recipients and the index of the credits. A suggestion groups the credits with one normal name (task 063, Words). Credits with equal normal names join one suggestion. This applies to a credit with an information address, a credit with an `npub` and a credit with neither.
14. When two or more different `npub` values have one normal name, each `npub` makes its own suggestion, because they can be two people. The credits of that name with no `npub` then make one more suggestion.
15. A credit with an `npub` and no name makes a suggestion of its `npub`.
16. A payee joins the suggestion of its normal name. When a name has more than one suggestion (item 14), the payee joins the suggestion with no `npub`. When that suggestion does not exist, the payee joins each suggestion of its name. A payee also joins through the wallet of a payee that joined by name.
17. The identity of a suggestion is `name:{normal name}`, or `npub:{npub}` for a suggestion of item 14 or item 15. A dismissal from before 2026-11-01 with a different identity does not hide the new suggestion.
18. A suggestion gives the number of feeds with a credit and with a payee. Its evidence lists `SameNpub` when the suggestion has one `npub`. It lists `SameName` when a payee joined by name, and `SameWallet` when a payee joined through a wallet.
19. The core makes no person and no link without the user. The user selects the items of a suggestion in the sheet of items 38 to 42.
20. The core gives no suggestion for an item that is already a link of a person.
21. **Hide This Suggestion** makes the core keep a dismissal, and the suggestion goes from the list. A suggestion that gets a new item comes back by itself. The core gives the hidden suggestions: each dismissed suggestion with no new item. **Show Again** removes the dismissal at once. It changes no person and no feed.
22. The core gives the places of each role of a suggested credit.

### The Lightning Address Of A Nostr Profile

23. For each Nostr key of a person, the core reads the `lud16` of the newest kind 0. It uses the lookup relays of [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) item 12. It then makes the keysend lookup of ADR 0095 item 2.
24. The address is a suggestion on the page of the person, with the result of the lookup. The user adds it to a wallet, or hides it with **Dismiss**. The collection keeps each dismissed address for that person, and the address does not come back. The collection keeps no lookup of a suggestion that the user did not add, as [ADR 0097](0097-studio-suggests-the-lightning-address-of-a-wallet.md) item 3 says for a guess.
25. Each request follows the network policy of the shell (ADR 0057).

### The Operations Of A Person

26. The change set of ADR 0091 has the operations of a person. The core applies each one to the selected feeds:
    - **Use This Address**: each recipient of each wallet of the person becomes the selected address. The `name`, the `split` and `fee` stay. Two recipients of the person in one block then merge into one recipient, with the sum of the two splits. A fee recipient and a recipient with no fee do not merge.
    - **Rename**: the new name replaces the `name` of each recipient of the person, the name of each credit of the person, or both. The user selects them.
    - **Add to All Releases**: the core adds items to each release of a publisher feed. An item is the default address with a percentage, a credit with the usual role and the usual group, or a support link. The user selects one or more items. A release that has the item already is "Not changed" for that item.
    - **Remove from Feeds**: the core removes each recipient of the person, each credit of the person, or both. The splits follow ADR 0091 item 14.
    - **Correct**: the changes of item 71.
27. When an operation puts an `lnaddress` recipient into a block, the core writes the method of ADR 0095 item 13.
28. These operations are feed rules. They are in the core (ADR 0065).
29. A change set does not change a person. Discard does not change a person. When the change set of **Rename** changes a feed, the core also changes the name of the person. **Rename** does not change the nickname.

### Add, Merge And Delete

30. The core adds the selected items of a suggestion to a person in one transaction. A conflict of one item adds no item. The core changes no name, picture or default address of the person.
31. A merge moves each link of the other person to this person, then deletes the other person, in one transaction. The name, the usual role, the default picture, the default website and the default address of this person stay. The other values go into the lists of this person, after its values (ADR 0196 item 15). The nickname follows ADR 0182 item 17. The dismissed addresses of item 24 move to this person. A change set of the other person gets this person.
32. A merge changes no feed.
33. A delete removes the person and its links. It changes no feed. The items of the person can then come back as a suggestion, because they are no longer a link of a person (item 20).

### The Export

34. The route `GET /people/export` gives a JSON file of the people. Each link has its evidence and the time of the link. The evidence is the statement of the user, the keysend lookup or a Nostr signature. The file gives the nickname (ADR 0182 item 18). It gives the defaults in `img` and `href`, and the lists in `pictures` and `websites` with their sources (ADR 0196 item 16).
35. The export holds no secret. An import of people is not a part of this ADR. [ADR 0107](0107-a-user-can-export-and-import-an-account.md) imports a full account.

### The People Destination

36. The bottom group of the left panel starts with **People**, with the Lucide icon `users`. The sequence is People, File hosts, Settings, then Lock Studio.
37. The People page shows these sections in this sequence. A section with no data does not show, but "People" always shows:
    - On a server, the line of [ADR 0108](0108-people-data-at-rest-stays-plain.md) item 3.
    - A search field. It finds a person, an address or a key.
    - The buttons **Add a Person** (action), **Update from Linked Pages** (normal, ADR 0170) and **Export People** (normal). **Add a Person** opens the sheet of ADR 0192.
    - "Changes to write", with each change set that is not closed. A row shows the operation, the person or the wallet, and "N feeds to write".
    - The fold "Suggestions from your feeds". A row shows the name, the counts, for example "Credit in 3 feeds · Payee in 6 feeds", and one line with the evidence. With no person, this fold comes before "People". Else it comes after "People".
    - "People". A row shows the picture, the nickname or the name, the usual role, "Linked to {feed}, {feed}" (ADR 0146 item 2) and the default address. The number of feeds is at the right. With no person, the section shows "No people yet. Check a suggestion above, or add a person."
    - The fold "Payees with no person". A row is a wallet of ADR 0095, with the address line of item 83.
    - The fold "Hidden suggestions", with its count. It starts closed. Each row shows the suggestion as the list of suggestions shows it, with the normal button **Show Again**.

### The Sheet "Check: {name}"

38. A suggestion row opens the sheet "Check: {name}". Its first field is the select menu "Person", with "A new person" and each person.
39. When a person has the normal name of the suggestion, or one of its credit names, the menu starts on that person. Else it starts on "A new person".
40. With a person in the menu, the sheet lists each item of the suggestion. An item that the person has already shows "Already in {person}" and has no check box. Each other item shows "New" and starts with the selection of item 41. The primary button reads **Add 1 Item** or **Add N Items**, and it adds the items as item 30 gives.
41. An item with the normal name of the suggestion starts selected. An item with a different name in its feed starts not selected. A key with the warning of item 58 starts not selected.
42. With "A new person", the sheet has the field "Name" and the select menu "Link to an artist or label page" (ADR 0146 item 1), and the primary button reads **Make the Person**. The menu keeps one feed. Its placeholder is "None", and its hint is "The artist or label page of this person. Studio changes no feed." **Hide This Suggestion** (normal) stays in each state. The sheet changes no feed.
43. A credit row of the sheet has the release as its title, then one line for each role with its places, as item 54 gives.

### The Page Of A Person

44. The page of a person has the sections and the tiers of ADR 0182 item 10. The page keeps the changes of its fields as pending changes until **Save** (ADR 0116). The nickname saves at once (ADR 0194 item 1).
45. The head shows the picture, the nickname or the name with the pencil of ADR 0182 item 13, the usual role, and each linked publisher feed with its type dot, its link and its role. Each Nostr key shows its profile (item 57), the result of its check and a copy button.
46. "Linked artist and label pages" follows ADR 0132, ADR 0146 and ADR 0159. "Credits" holds the field "Name" (ADR 0182 item 15), the usual role, and the groups "Pictures" and "Websites" (ADR 0196). It also holds the fold of the credit names and the Nostr keys (ADR 0140), **Add a Credit Name** and **Add a Nostr Key** (ADR 0123).
47. "Payment" has the default address on its own row (ADR 0139), one group for each wallet, the `lud16` suggestion of item 24 with its lookup result, and **Add a Wallet** (ADR 0141).
48. "Support links" has each link with **Remove** and **Edit**, and **Add a Support Link** (ADR 0145 item 14).
49. "Change your feeds" holds one navigation row for each operation of item 26 except **Correct**. Each row has one line that says what the operation does. Its note is "Each operation shows the feeds and a review before Studio writes." The page has no operation button at its top, and no list of feeds with check boxes.
50. The last section has **Delete Person** (destructive) first, then **Merge with a Person** (normal) (ADR 0193).
51. **Delete Person** opens a confirmation sheet with the title "Delete {name} from People?" and the text "Studio deletes this person and its links. Studio changes no feed." The action button reads **Delete the Person**. The text is in `confirms.js`. After the delete, Studio opens the People page.
52. **Merge with a Person** opens the sheet "Merge with {name}", with the select menu "Person to merge", the links of the selected person, and the result. The merge follows items 31 and 32.

### More From Your Feeds

53. The section "More from your feeds" lists each item of the feeds that is not a link of a person and that matches this person:
    - a credit whose normal name is the name of the person or one of its credit names,
    - a credit whose `npub` is a key of the person,
    - a payee whose normal name is the name of the person or one of its credit names,
    - a payee whose wallet key is the wallet key of a wallet of the person.
54. A row gives the item, its feeds and the cause of the match. A credit row has the release as its title, or "N releases" for many releases, and then each line starts with the release. Each role has one line with its places, for example "Vocalist · track 3 "Night Song", track 5 "Low Tide"". The places give the first two places and "and N more". The name in the credit and the cause follow. A credit row with a Nostr key has the profile of the key as its title.
55. A row has **Add** (action), which adds that one item as item 30 gives. A row with the key warning of item 58 has **Correct the Feeds**, which opens "Check your feeds", and **Add Anyway**, which adds the key. Both have the role action.
56. The section uses only the data of the collection: the index of the credits, the index of the recipients and the stored lookups. It makes no relay request.

### The Profiles Of The Keys

57. A Nostr key shows the profile name and the picture of its newest kind 0, and a copy button with the full `npub`. This applies in "Check: {name}", in "More from your feeds" and on the page of a person.
58. Some keys have a profile name that does not contain the normal name of the person or of the suggestion. Such a key shows the warning "The profile name is not "{name}". Check this key." in the error color. A key with no profile shows no warning.

### Where This Person Is

59. The section "Where this person is" has one entry for each feed that pays or credits the person. One entry is open at a time (ADR 0028).
60. The summary line of an entry tells in words where the feed pays and credits the person, for example "Paid on the album and 12 tracks" or "Credited as vocalist on 2 tracks".
61. An open entry has one line for each place. The line names "Album" for the channel, or the number and the title of the track. It gives the split of the person in that place, for example "15 % split", and the roles of the credits of the person in that place.
62. The core gives the split of each place in percent, with the rule of musicindex ADR 0075 R-VALUE-5: each recipient gets its share in the total of the shares of all recipients, fee recipients included. The crate `musicindex-value` gives the arithmetic. When many addresses of the person are in one place, the split is their sum.
63. The row "Check your feeds" gives only the state of the check and the count of feeds: "N feeds need a correction" in the error color, or "Each feed is correct". It opens the page of item 64.

### Check Your Feeds

64. The page "Check your feeds" compares each feed of the person with the person. The core gives each feed a state and a list of findings:
    - A credit of the person has no Nostr key, or a key that is not a key of the person. The finding names the key, and the UI shows its profile name.
    - A credit of the person has a picture or an information address that differs from the value of the person. The core makes this check only when the person has the value.
    - A credit of the person has no role. Apps then show the credit as "Host".
    - A recipient of a wallet of the person does not use the default address. This finding is neutral, because the address pays the same wallet.
    - A recipient has the normal name of the person or of a credit name, but its wallet is not a wallet of the person.
65. A credit of the person is a credit with the identity of a credit name of the person. A credit with the normal name of the person or of a credit name is also a credit of the person, whatever its key. Thus Studio finds a credit with a wrong key.
66. The page groups the feeds as "Needs a correction", "Not your default address" and "Correct". A finding, except the neutral one, shows in the error color.
67. A feed with a finding is a `details` entry (ADR 0028). A feed in "Correct" is a row that opens its editor page. The summary line of an entry gives the title, the type and the count, for example "Album · 2 credits differ". One entry is open at a time.
68. The open entry shows one block for each finding. A block gives:
    - The place: "Channel", or the title of each item with the credit, for example "Track 3 · Neon Diner". More than three places give the first two and "and N more".
    - The kind of the value and the role of the credit, for example "Credit · Vocals · information address".
    - Two lines: "In the {release word}" with the value of the feed, and the name of the person with the value of the person. A missing value is "None".
69. The entry ends with **Use {name}'s Values** (primary) and **Open the {release word}**. The first button opens the sheet of item 71 for that one feed. The second button opens the editor page of the feed. The release word is "album", "playlist", "artist page" or "label page".
70. The core gives each finding with its values and its places. The UI calculates no difference. The check uses only the data of the collection, and makes no request to a relay or to a wallet. The UI resolves the profile names of the shown keys, as item 57 gives.

### Correct The Feeds

71. The page has **Correct N Feeds** (primary). It opens the sheet "Correct the Feeds: {name}". The sheet lists these changes with check boxes, then each feed with its changes. The primary button reads **Review in N Feeds**.
    - "Nostr key": each credit of the person gets the key of the person. A person with more than one key gets the menu "Key".
    - "Payment address": each recipient of a wallet of the person uses the default address, as **Use This Address** does.
    - "Picture and information address": each credit of the person gets the values of the person.
    - "Role": each credit of the person with no role gets the usual role and its group.
    - "Replace other addresses named "{name}"": each payee of the name gets the default address. The line gives the number of these payments and the default address. It ends with "Select this only when that address is yours too." This change starts not selected, because it changes who gets the payment.
72. The correction is the operation **Correct** of item 26. A feed with no change after the selected corrections is not in the list.
73. The operation keeps the role of each credit that has a role, because one person can have a different role in each release.

### The Sheets Of The Operations

74. Each sheet of an operation lists the feeds of the person with check boxes, below its own fields. Each check box starts selected. A feed row shows the type and the places, the line "Pays {address}", and the line "Credited as {role}". Thus no operation button is disabled because no feed is selected.
75. **Use This Address** has the select menu "Address", with each address of each wallet of the person.
76. **Rename** has the field "Name", and the check boxes "Payment name" and "Credit name". Both start selected.
77. **Add to All Releases** starts with one line: "Studio adds {name} to each release of the artist or label that you select." Then it has:
    - The select menu "Artist or label".
    - The check boxes "As a payee", with the field "Percentage", "As a credit" and "Support link".
    - One line with the result, for example "10 releases · 7 get the payee · 7 get the credit · 3 have both already".
78. **Remove from Feeds** is the name of the button and the title of the sheet. The sheet has the check boxes "Payee" and "Credit".
79. The primary button of each sheet reads **Review in N Feeds**. The review keeps **Write N Feeds** and **Discard** at its top, and the bar keeps them visible at each scroll position.

### A Payee With No Person

80. The page of a wallet with no person has the section "Person", with the navigation row "Add to a Person". The row opens a sheet with the menu "Person", as item 38 gives. The operations of the wallet are in a section "Change your feeds", as item 49 gives. In **Add to All Releases**, the recipient details are in one collapsed entry.

### The Words

81. A UI text says "changes" for a change set. Examples are "Changes to write", "Discard These Changes?" and "Discard the Changes".
82. A type menu has the items "LNURL Address" and "Node Address" (ADR 0138 item 3). A field that edits the custom value of a node is "Account". The custom key keeps its name "Custom key", with a hint.
83. A line of an address reads "LNURL address {address}", "{provider} node · account {value}", or "Node address {key}".
84. The hint of ADR 0095 item 15 reads: "Studio found no payment details for this address at {domain}. Some apps cannot pay it."
85. No UI text shows an XML attribute, such as `method="lnaddress"`. The feed values do not change: a feed keeps `type="node"`, `type="lnaddress"` and `method="lnaddress"`.

### The Pickers

86. The sheet of a recipient has the layout of ADR 0145 and ADR 0144. Its lists are "Your people", then "Payees with no person". A person row shows the nickname or the name and the address that it adds, which is the default address. A selection fills the form. An added recipient gets no split ([ADR 0030](0030-an-added-recipient-gets-no-split.md)).
87. The sheet "Add a Credit" has a list of the people. A person row shows the usual role and the Nostr key. A selection fills the form with the name, the usual role and the usual group of the person. It also fills the default picture, the default website and the key.
88. The sheet "Add a Support Link" has a list of the support links of each person. A selection fills the form.

## Alternatives Considered

- Keep the eight ADRs current, each with its "Changed by" lines. Not chosen: the global decision of 2026-11-14 permits three changes at most, and ADR 0099 had eight.
- Restate also the People rules of ADR 0112, ADR 0116, ADR 0132, ADR 0140, ADR 0182, ADR 0196 and the other ADRs of the second "Uses" line. Not chosen: each of those ADRs also states rules outside the People area, or it has fewer than four changes. This ADR names each one at its item. It does not copy its text.

The superseded ADRs give the alternatives of each original decision.

## Consequences

- One ADR gives the People area. A reader finds each item here, or at the item of a named current ADR.
- 309 code comments cite an item of a superseded ADR. The item map gives the new item. The update of these comments is open.
- A subsequent change to People changes this ADR with a "Changes" line. After three changes, a new current ADR replaces this ADR.

## Non-Goals

- A link of a person across the collections of two users.
- An import of people from a file or from a different application.
- One credit entry with many roles.
- A display rule for the support links in musicindex.org and HitIt. Each of those apps needs its own decision, and a shared rule needs a record in `../musicindex`.

## How This Decision Is Checked

Mechanical. The tests of the superseded ADRs enforce these items, and this ADR adds no test:

- `crates/core/src/people.rs`: the migration, the wallet limit, the suggestions (items 13 to 22), the operations (items 26 to 29), add, merge and delete (items 30 to 33), the check and the correction (items 64 to 73), and the places (items 6 and 62).
- `crates/api/src/tests.rs`: the export (items 34 and 35), the routes of the hidden suggestions and of **Show Again**.
- `crates/api/ui-tests/people.test.mjs`: the sections of the People page (item 37), the rows, the sheet "Check: {name}", the lines of "Where this person is" and of "More from your feeds", and the words (items 81 to 85).
- `crates/api/ui-tests/pages.test.mjs`: the sequence of the page of a person (item 44).
- `crates/api/ui-tests/confirms.test.mjs`: the delete sheet (item 51).

Visual: no new visual check. The visual checks of the superseded ADRs passed.

Human: no new human check.

## Item Map

The map gives the item of this ADR for each item of a superseded ADR. "Owned" names the current ADR that owns the rule.

| Old item | New item |
| --- | --- |
| ADR 0098 items 1 to 4 | 1 to 4 |
| ADR 0098 item 5 | 8 |
| ADR 0098 items 6, 7, 8 | 9, 10, 11 |
| ADR 0098 item 9 | 5 |
| ADR 0098 item 10 | 6 |
| ADR 0098 item 11 | 7 |
| ADR 0098 item 12 | 13 to 17 |
| ADR 0098 item 13 | 18 |
| ADR 0098 item 14 | 19 |
| ADR 0098 item 15 | 21 |
| ADR 0098 item 16 | 20 |
| ADR 0098 items 17 to 19 | 23 to 25, and 31 for the merge of item 18 |
| ADR 0098 items 20 to 23 | 26 to 29 |
| ADR 0098 items 24, 25 | 34, 35 |
| ADR 0098 item 26 | 12 |
| ADR 0099 item 1 | 36 |
| ADR 0099 item 2 | 37 |
| ADR 0099 item 3 | 38 to 42 |
| ADR 0099 item 4 | 44 to 49 |
| ADR 0099 item 5 | 64, 74 |
| ADR 0099 item 6 | 49 |
| ADR 0099 items 7 to 9 | 74 to 76 |
| ADR 0099 item 10 | 77, 80 |
| ADR 0099 items 11, 12 | 78, 79 |
| ADR 0099 items 13 to 17 | 81 to 85 |
| ADR 0099 item 18 | 26 |
| ADR 0099 items 19 to 21 | 86 to 88 |
| ADR 0102 items 1, 2 | 50, 51 |
| ADR 0102 item 3 | 33, 51 |
| ADR 0102 item 4 | 78 |
| ADR 0103 items 1 to 4 | 13 to 16 |
| ADR 0103 item 5 | 17 |
| ADR 0103 item 6 | 18 |
| ADR 0103 item 7 | 41 |
| ADR 0103 items 8, 9 | 38 to 40 |
| ADR 0103 item 10 | 30 |
| ADR 0103 items 11 to 13 | 53 to 56 |
| ADR 0103 item 14 | 50, 52 |
| ADR 0103 items 15, 16 | 31, 32 |
| ADR 0103 items 17, 18 | 57, 58 |
| ADR 0111 item 1 | 44 (owned: ADR 0182 item 10) |
| ADR 0111 item 2 | 44 (owned: ADR 0116) |
| ADR 0111 item 3 | 46 (owned: ADR 0132) |
| ADR 0111 item 4 | 49 |
| ADR 0111 items 5 to 7 | 38 to 42 |
| ADR 0111 item 8 | 80, 47 (owned: ADR 0141) |
| ADR 0111 items 9 to 11 | 64 to 66 |
| ADR 0111 item 12 | 63 |
| ADR 0111 item 13 | 70 |
| ADR 0111 items 14, 15 | 71 |
| ADR 0111 items 16, 17 | 72, 73 |
| ADR 0120 items 1 to 3 | 67 to 69 |
| ADR 0120 item 4 | 64, 69 |
| ADR 0120 item 5 | 6 |
| ADR 0120 item 6 | 70 |
| ADR 0156 item 1 | 59 (tier owned: ADR 0182 item 10) |
| ADR 0156 items 2 to 4 | 60 to 62 |
| ADR 0156 item 5 | 6 |
| ADR 0156 item 6 | 63 |
| ADR 0156 items 7, 8 | 54 |
| ADR 0158 item 1 | 21, 42 |
| ADR 0158 items 2, 3 | 37, 21 |
| ADR 0158 item 4 | 21 |
| ADR 0158 item 5 | 22, 43 |
