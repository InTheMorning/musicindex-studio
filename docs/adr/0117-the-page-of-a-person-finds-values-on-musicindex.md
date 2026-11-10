# ADR 0117: The Page Of A Person Finds Values On musicindex

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screens 2 to 5 of the mockups in `.studio-dev/mockup-lookup/site/`.
- Changed on 2026-11-08: the count of a row and the notes of levels 1 to 3 say "release", not "album". A count holds the releases of all types (musicindex ADR 0023 R-SCOPE-1 item 2 and R-WORD-9 item 5). The kinds of R-WORD-9 items 1 to 4 wait for the track counts of each release.
- Changed by: [ADR 0163](0163-one-sheet-finds-and-links-a-page-on-music-index.md). Items 2, 8 and 13: the sheet is also the sheet of the links of a person, and the sheet "From Music Index" has the section "Link".
- Uses: [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for the pending changes. [ADR 0098](0098-the-people-module.md) items 5 and 6 for the fields and the links of a person. [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) for the keysend lookup. [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) for the Nostr keys. [ADR 0057](0057-the-private-network-guard-of-the-server.md) for the requests. [ADR 0109](0109-images-and-audio-load-from-their-hosts.md) for the pictures. musicindex ADR 0010, ADR 0032 R-DOT-1 and ADR 0033 R-WORD-3 for the dots and the words.

## Context

- The user types or pastes the picture, the information address, the wallets and the Nostr key of each collaborator. ADR 0113 suggests values from the feeds of the user and from the Nostr profiles only.
- The operator asked on 2026-11-02 for a lookup in the app. The lookup gets the published data of a collaborator from a known RSS source, and adds nothing to the collection.
- The musicindex API has no route that searches people by name. `/v1/search` searches feeds, tracks and publisher feeds (`type=publisher`). Thus the lookup finds the artist or label feed of the collaborator by its name, and reads that feed.
- On 2026-11-02, 176 publisher feeds of the live `/v1/artist-names?limit=200` gave these values:

| Value | Field of the API | Feeds with the value |
| --- | --- | --- |
| Picture | `image_url` | 176 of 176 |
| Website | `source_links` with `link_type` `website`, from `feed.link` | 162 of 176 |
| Nostr key | `source_ids` with the scheme `nostr_npub` | 0 of 176 |
| Credits | `source_contributors` | 0 of 176 |
| Wallet | `payment_routes` | 3 of the first 60 |

- The research note `contributor-database/index-measurements.md` in `../musicindex-research` agrees. It counts 18 different `npub` values in the index of 2026-10-25. It also counts 130 of 1,774 publisher feeds with a channel value block. Thus a picture and a website are frequent, a wallet is rare, and a Nostr key is almost never given.
- A row of `/v1/search?type=publisher&include=link_facts` gives `two_way_link_count`, `agreed_roles` and `confirmed_release_artists`. A live search for "the" on 2026-11-02 gave 100 rows: 1 with an agreed artist role, 98 with two-way links and no role, and 1 with no two-way link. In 97 of the 98, a confirmed release artist had the name of the feed.
- musicindex.org shows these levels with a filled dot or a hollow dot, and fixed words (`publisherRowKind` in `../musicindex/search.html`). Studio uses the same words, so that the apps look the same.

## Decision

### The Search

1. The head of the page of a person has "Find on Music Index" (action), below the name and the role line.
2. The button opens the sheet "Find on musicindex: {name}". The search field starts with the name of the person. The core sends the search when the sheet opens and when the user changes the text. It sends no request when the page opens.
3. The core searches `/v1/search?type=publisher&include=link_facts` of the musicindex API of the shell (`--musicindex`). The request sends only the search text.
4. The core puts each row in one of three levels. The sheet shows the levels in this sequence, each with its heading and its note. A level with no row does not show. In a level, the rows keep the sequence of the search.

| Level | Heading | Condition | Type line |
| --- | --- | --- | --- |
| 1 | "Verified role" | `agreed_roles` is not empty | The filled dot and the word of the role: "Artist · N releases", or "Label · N releases" (musicindex ADR 0033 R-WORD-3) |
| 2 | "Verified two-way links" | `two_way_link_count` is more than 0, and `agreed_roles` is empty | "Inferred artist · N releases" with a hollow artist dot, when the name matches (item 5). If not, "Feed · two-way links, no role" with a hollow feed dot |
| 3 | "Name only" | `two_way_link_count` is 0 | "Unverified link" with a hollow feed dot |

5. The name matches when a value of `confirmed_release_artists` is equal to the title of the feed, or contains it as whole words. The release artist of the row is also a name of the feed, unless its source is `placeholder`. Each name is compared in lowercase, with `&` as "and", and with each run of other characters as one space. This is the `creditMatch` rule of musicindex.org.
6. The notes are:
    - Level 1: "The feed and its releases link to each other, and both give the role."
    - Level 2: "The feed and its releases link to each other, but they give no role."
    - Level 3: "Only the name matches. No release links back to this feed."
7. Each row shows the picture of the feed, its title, the type line and a line with the host of the feed and the values that it has. Examples are "wavlake.com · a picture, a website and a Lightning address". The core reads the detail of each row for a maximum of 10 rows. A row with no detail shows only its type line.

### The Values

8. A row opens the sheet "From musicindex: {title}". The feed row has the type line of item 4 and the link "Open on musicindex.org", to `https://musicindex.org/#feed={guid}`.
9. The section "Fill" has one check box row for each value:
    - "Picture": the `image_url` of the feed, with a preview.
    - "Information address": the `website` link of `source_links`, with the line "The link of the feed".
    - "Wallet": one row for each channel route of `payment_routes` with the type `node` or `lnaddress`. The row shows the split. A Lightning address gets the keysend lookup of ADR 0095, and the row shows the result. The row has the line "Paid to:" with the name of the recipient, or "The feed gives no name for this recipient." It also has the line "A feed can pay a band member, a collaborator or an app. Add this wallet only when this recipient is this person." The operator changed this text on 2026-11-05, because the row gave no clue of who the recipient was.
    - "Nostr key": the `nostr_npub` value of `source_ids`. When the feed gives none, the row has no check box, and it reads "This feed gives no Nostr key. Use “Add a Nostr Key” to search the Nostr profiles."
10. A picture or an information address is an `http` or `https` address. A different value is not in the list.
11. A picture row and an information address row start selected when the field of the person is empty. A wallet row and a Nostr key row start not selected.
12. A row with the current value of the person, or a wallet that the person has, has no check box. It reads "Already the value of this person" or "Already a wallet of this person". A wallet of a different person has no check box, and it names that person (ADR 0098 item 6).
13. The buttons are "Back" (normal) and "Fill 1 Field" or "Fill N Fields" (primary). The number counts the selected rows. The note is "Studio puts the values in the fields. Select Save to keep them. Studio changes no feed."
14. "Fill" makes a pending change of ADR 0116 for each selected row. A filled field shows the line "From musicindex: the {type word} feed {title}". A selected wallet and a selected Nostr key are also pending changes, and Save adds them.

### Limits

15. The lookup adds nothing to the collection, reads no feed into the library and changes no feed.
16. The requests follow the network policy of the shell (ADR 0057) and the time limit of the musicindex client of the core. A request that fails shows its error in the sheet, and the page does not change.
17. The values come from public feeds and go into the fields of ADR 0098 item 5 and the links of item 6. The lookup adds no field to a person, and no value holds a secret. Thus the condition of ADR 0108 item 6 does not occur.

## Alternatives Considered

- A search when the page of a person opens. Not chosen: it sends the name of each person to a server at each visit.
- One list with a badge on each row. Not chosen by the operator: the operator asked for the three levels in sequence, each with its dot.
- A search of the credits of all the feeds by the name of a person. Not chosen now: the musicindex API has no such route, and only 2 percent of the feeds name a person. It needs a change in `../stophammer` first.
- The Nostr key from the lookup. Not chosen as the main source: the artist feeds give almost no key. "Add a Nostr Key" keeps its profile search (ADR 0074).

## Consequences

- The core gets the search of item 3, the levels of item 4, the name match of item 5 and the detail of items 7 and 9. The musicindex client of the core has `search` and `feed_routes` now.
- The API gets a route for the search and a route for the values of one feed.
- The UI gets the two sheets, and a model module that maps a level to its dot and its words.
- The levels and the name match are display rules for data from musicindex. They are not feed rules (ADR 0065).
- A task file gives the work.

## Non-goals

- The name rows of `/v1/artist-names`. A name row has no feed to read.
- A Nostr profile search in the sheet.
- A link of a person to a feed outside the collection. ADR 0118 keeps "Member of" for the feeds of the collection.

## Acceptance Criteria

Mechanical:

- A core test, against a local test server of the musicindex API: the core puts the rows in the levels of item 4, with the sequence of item 4, and applies the name match of item 5.
- A core test: the detail stops after 10 rows. A value that is not an `http` or `https` address is not in the list. A Lightning address gets the keysend lookup.
- An API test: the search route and the values route give the levels and the values.
- A Node test: the model gives the dot class and the words of each level, the start value of each check box (item 11), the rows with no check box (item 12) and the label of the primary button.
- A Node test: "Fill" makes the pending changes of ADR 0116 and sends no request.
- No test sends a request to a host outside the computer (ADR 0045).
- The four commit checks pass.

Visual: screens 2 to 5 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator opens a collaborator, selects "Find on Music Index", selects the artist feed, fills the picture and the website, and selects Save.
