# Design Note: A People Module

- Date: 2026-11-01
- Status: Advisory. This note makes no rule. The ADRs that come after it make the rules.
- Asked by: the operator, on 2026-11-01, after the mockups of contacts on the Recipients page.

## Purpose

This note examines an address book of people for Studio. It lists the jobs that the user does, the module that owns each job, and the data that each module keeps. The operator then reviews it before the mockups and the ADRs.

## The Decisions Of 2026-11-01

The operator selected these items from a questionnaire on 2026-11-01:

- The address book is a module of Studio, with the name "People" for this note. It is not a part of the Recipients page, and it is not a service that the other apps share.
- A person record holds payments, credits, Nostr keys and support links (`podcast:funding`).
- The code starts after the human tests of Phase 12. This note and the mockups can start before that.
- The operator said that each app of the suite must show the support links, because stophammer now keeps them. Each app needs its own decision for this. See "Support Links In The Suite".
- The operator asked to use the research on PodPeopleDB as a source of ideas for the suite. See "Ideas From The Contributor Database Research".

## How Studio Keeps Data About People Now

Studio has no record of a person. The data is in three places that do not refer to each other:

| Data | Where | Owner |
| --- | --- | --- |
| Value recipients of each feed | Table `feed_recipients`, from each read and write | [ADR 0027](../adr/0027-the-recipient-picker.md) |
| Recipients that the user saved | Table `saved_recipients` | ADR 0027 item 5 |
| The wallet of a Lightning address | Table `keysend_lookups` | [ADR 0095](../adr/0095-a-lightning-address-and-its-node-key-are-one-recipient.md) |
| Credits (`podcast:person`) | Only in the feed model of each feed. No table. | [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md), [ADR 0075](../adr/0075-the-publisher-page-has-payments-and-credits.md) |
| Nostr keys of persons and feeds | Only in the feed model. The profile search gets profiles from relays and musicindex. | [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md) |
| Support links (`podcast:funding`) | Only in the feed model, as `fundings` on the channel and on each item. The editor shows them as "Support links". No table and no picker. | ADR 0023, ADR 0075 |

The Recipients page of [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) does two jobs:

1. It is a directory of payees: who gets paid, and through which addresses.
2. It is a tool for batch edits: each operation makes a change set and writes feeds.

The mockups of contacts (`.studio-dev/mockup-098/`) added a third level to the first job, and the names multiplied: recipient, wallet, contact, route and change set. The UI review of 2026-11-01 found the same problem in the current screens.

## The Jobs

Each job has an example from the operator, or from the review of 2026-11-01. "People" owns the data of a person. "Feed edits" owns each change of a feed.

| Job | Example | Owner of the data | Owner of the change |
| --- | --- | --- | --- |
| J1. Change the address of a person in many feeds | Mary Kate Ultra moves from her Fountain wallet to her home node | People | Feed edits (change set) |
| J2. Add a person to all releases of an artist or label | A guest gets 5 % in 10 albums | People | Feed edits (change set) |
| J3. Remove or rename a person in many feeds | A band member leaves | People | Feed edits (change set) |
| J4. Add a person as a recipient of one feed | The value editor picker | People | The value editor |
| J5. Credit a person with a role | "Vocals" on one track | People | The credits editor |
| J6. Give a person a Nostr key | Find the npub of a guest | People | The credits editor, at the save |
| J7. See where a person is used | Which feeds pay Mary Kate, and through which address | People, from the feed index | None |
| J8. Find a missing address | The Lightning address of a wallet ([ADR 0097](../adr/0097-studio-suggests-the-lightning-address-of-a-wallet.md)) | People, from data that Studio finds | None |
| J9. Start the address book from the feeds | The first visit after the update | People, from the feed index | None |
| J10. Add a support link to many feeds | A label puts its donation page into each release | People | Feed edits (change set) |
| J11. Add a person to selected tracks of one album | A guest plays on tracks 3 and 7 | People | The credits editor |

## Three Types Of Data

The module keeps three types of data. Each type has a different owner and a different lifetime.

1. **Data that the user owns.** The person, its name, its picture and its link. Also the wallets, the Nostr keys and the support links that the user links to it, its usual roles and its default address. Only the user changes this data. A read of a feed does not change it.
2. **Data that Studio finds.** The keysend lookups, the providers of nodes, the Nostr profiles with their `lud16`, and the suggestions. Studio can find this data again when necessary. It is a proof or a suggestion, not a link by itself.
3. **Data that the feeds give.** The recipients and the credits that each feed has now. The feeds own this data. People reads it to count the uses and to make suggestions. People does not write it.

## A Possible Boundary

This section is a proposal for the module ADR. It is not a decision.

- The core gets the module `people`. It keeps the data of type 1 in its own tables. It has no feed rule and no HTTP server code ([ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md)).
- `people` writes no feed. A change of a feed from a person goes through a change set ([ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md)), and each feed writes through `pipeline::write_draft`.
- `people` reads the feed index. The table `feed_recipients` is an index of the recipients. Studio has no index of the credits and the keys of each feed, and J7 and J9 need it.
- The value editor, the credits editor and the profile search read from `people`. A person is the start value of a recipient or of a credit. The feed keeps its own copy of each value, so a feed does not change when the person changes. This is an edit aid of [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).
- The wallet of ADR 0095 stays as it is. A person holds one or more wallets. The keysend lookup links the addresses of one wallet. Only the user links two wallets to one person.
- `saved_recipients` becomes a part of `people`. A migration moves each saved recipient into a person.
- In the left panel, "People" replaces "Recipients". The page of a person holds its batch edits. A batch edit for a wallet with no person stays available.
- The same module serves the desktop shell and the server shell. On the server, each user folder has its own collection, so each user has an address book of its own ([ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).

## Support Links In The Suite

- Stophammer [ADR 0082](../../../stophammer/docs/adr/0082-a-funding-link-is-a-source-link.md) keeps each `podcast:funding` tag of a channel, an item and a live item as a `source_links` row. The row has the `link_type` `funding` and a `label`. A client reads them with `include=source_links` on the feed route. Release 0.18.0 holds the change. On 2026-11-01 that release was open, and the refill pass of the held feeds comes after its deploy.
- ADR 0082 §5 gives the data only. How an app shows a support link is a rule of that app, or a shared rule of musicindex.
- Studio reads and writes the tag today. musicindex.org and v4vmm do not show it. The research note on the funding tag (2026-10-31) found no use of the tag in the other projects of the suite.
- A support link is a link of a feed or a track. It is not a payment route of a person. People can keep a support link as a start value of a person or of a publisher feed. The link goes into a feed only through an action of the user.
- A shared display rule for the three apps needs a record in `../musicindex`, as each other shared interface rule does.

## Ideas From The Contributor Database Research

The research area `research_notes/contributor-database/` of `../musicindex-research` (2026-10-31) examined PodPeopleDB, a community database of podcast hosts and guests. These ideas apply to Studio and to the suite. They are ideas, not rules.

For Studio now:

- **The four steps of the Studio convenience note.** Known people and known support links, with a picker, as the recipients have. A role menu from the Podcast Taxonomy, with free text. A contributor card that joins a person to a wallet and a support link. Change sets for people and support links. The People module of this note holds the four steps.
- **The identity of a known person.** The research recommends: the `npub` when the person has one, else the name with the information address, else the name alone. Inside the collection of one user, a name match is a statement of that user about their own feeds.
- **The raw role text stays.** Music feeds write roles such as "vocals", "guitarist" and "band", which are not in the taxonomy. A menu helps, and the feed keeps the text that the user wrote.
- **The `lud16` of a Nostr profile.** A signed profile gives the key, the name, the picture and a Lightning address. ADR 0074 does not use `lud16`. The research recommends a new decision that uses it, with the keysend check of ADR 0095.

For the suite later:

- **Entity first.** A publisher feed with its releases, where both sides state the link, is the anchor of the identity of an artist or a label. The stophammer index already proves these links.
- **Evidence on each link.** A link of a person to a wallet keeps its type of evidence: the value block of the entity feed, a feed that names the person both as a credit and as a recipient, the keysend lookup, a Nostr signature, or a reviewer. Money follows a wallet link, so a wallet link needs proof before it is public.
- **The data flow of PodPeopleDB.** The feed is the authority, and the community fills the gaps. Each row has a source and a status. A merge keeps the old IDs, and a changes feed serves mirrors.
- **A shared registry.** The research recommends a small registry of the suite, seeded from the index. It is alternative B of the research, and it is outside Studio. A Studio export of people can feed it later.
- **Privacy.** A list of the wallets of a person across feeds is a new collection of public data. Discussion 662 of the podcast namespace raised this risk. The address book of Studio stays in the collection of one user.

## The Effect On Current Decisions

- ADR 0027: the picker offers people, not only recipients.
- ADR 0091: the Recipients page becomes the People destination. The change sets do not change.
- ADR 0095: the wallet and its lookup stay. The page of a wallet changes.
- ADR 0074: the profile search becomes a source of people.
- ADR 0096: the name "Add to an Artist or Label". The mockups proposed "Add to All Releases".
- musicindex ADR 0023 R-WORD-2 item 3 says that the Studio page "Recipients" becomes "Value routes". A People destination removes the page that the rule names. The rule then needs a record in `../musicindex`.
- The draft ADR 0098 (contacts on the Recipients page) is replaced by this direction. It was not committed.

## Results Of The Mockups That Stay Valid

The mockups and the review of 2026-11-01 found these items. They apply to the People pages too:

- A sheet button that makes a review says "Review in N Feeds". The write is on the review page.
- A change that is not written shows "N feeds to write", not "changed".
- Plain words: "Lightning node", "Lightning address", "Account" in place of "Custom value", and "changes" in place of "change set". No XML text in the UI.
- A feed row on the page of a person shows the address that the feed uses now, on its own line.
- The default address of a person is a select menu on the page of the person.
- The sheet of a batch add says what happens, asks which releases first, then how much, and puts the recipient details in a collapsed entry.

## Answers Of The Operator

The operator answered these questions from two questionnaires on 2026-11-01. Each answer is the recommendation that the questionnaire gave. The ADRs make the rules.

1. **Rename.** The Rename sheet of a person has two check boxes, "Payment name" and "Credit name". Both start selected. The review shows each change.
2. **An artist or a label.** Entity first. An artist or a label is its publisher feed. A person is a member of that entity, and it can link to the publisher feed.
3. **One person from a credit and a payee.** Studio suggests a link, and the user confirms it. The suggestion uses the `npub` first, else the name with the information address, else the name alone. Studio makes no link without the user.
4. **The name of the destination.** "People". Before the ADR, a comparison with the words of musicindex.org and v4vmm confirms it.
5. **An export.** People has an export. It gives each person with its links and the evidence of each link: the statement of the user, a keysend lookup or a Nostr signature.
6. **The first visit.** The page shows suggestions from the feeds, for example "a credit in 3 feeds, a payee in 6 feeds". The user confirms each suggestion to make a person.
7. **`lud16`.** Studio uses the Lightning address of a Nostr profile as a suggestion, with the keysend check of ADR 0095. A new decision replaces the non-goal of ADR 0074.
8. **Many roles.** The first version keeps one entry for each role, as now. One entry with many roles can come later. The feed output does not change.

## Next Steps

1. The operator reviews this note.
2. Mockups of the People destination, the page of a person, and the pickers of the value editor and the credits editor.
3. The ADRs: one for the module boundary and the data, then one for each group of screens.
4. A phase in the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), after the human tests of Phase 12.
