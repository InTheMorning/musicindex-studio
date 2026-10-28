# MSP 2.0 Parity And Nostr Identities

Research date: 2026-10-28.
Scope: the functions of MSP 2.0 that Studio does not have, and the Nostr keys of artists, publishers and contributors.
This report states no rule. The owning ADRs own each rule. The proposal at the end needs an ADR before work starts.
The report was composed in STE. The shared checker reports findings that are mostly dictionary limits on technical names, such as "profile", "kind" and "route". This report does not claim full STE compliance.

## Method And Sources

| Source | Where | Date seen |
| --- | --- | --- |
| MSP 2.0, branch `publisher-parties` at `09f3cf9` | `../MSP-2.0` working tree | 2026-10-28 |
| MSP 2.0, `origin/master`, 27 commits after `09f3cf9` | `git show origin/master:<path>` | 2026-10-28 |
| Studio core, API and UI | `crates/` | 2026-10-28 |
| Podcast namespace specification | `../musicindex-research/sources/podcast-namespace/docs/tags/` | 2026-10-28 |
| Stophammer (the musicindex API) | `../stophammer/src`, `../musicindex/api.json` | 2026-10-28 |
| v4vmm | `../v4vmm/src/metadata_service.rs`, `identity_ingest.rs` | 2026-10-28 |
| NIP-11 documents of five relays | HTTPS requests | 2026-10-28 |
| A NIP-50 search test on three relays | A Node WebSocket script | 2026-10-28 |

The source of the crawler (`stophammer-crawler` and `stophammer-parser`) is not on this computer. The mapping of `podcast:txt purpose="npub"` to `entity_ids` comes from the example in `api.json`. It is not examined in code.

[ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) compared the fields at `09f3cf9`. This report does not repeat those fields.

## Part 1: Functions That Studio Does Not Have

### Gaps That Can Stop A Move From MSP 2.0

1. **A feed that MSP 2.0 hosts is read-only in Studio.** ADR 0006 item 4 rejects an MSP connector. The user must move the feed to a connected host ([ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)). Studio cannot write a redirect into the feed on MSP. This is an inference from ADR 0006 and ADR 0067. It is not tested.
2. **A publisher feed cannot list an album that the user does not control.** [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) item 5 offers only releases in the collection that are not read-only. MSP 2.0 lets a label add a Wavlake or Fountain album through a Podcast Index search (`CatalogFeedsSection.tsx`, `api/pisearch.ts`). This is a decision, not a defect.
3. **The publisher page has no payments and no support links.** MSP 2.0 edits both (`PublisherValueSection.tsx`, `PublisherFundingSection.tsx`). The Studio model has `values` and `fundings` on each channel (`crates/core/src/feed.rs:106,119`). `crates/api/ui/pages.js:23-28` gives the publisher page no section for them. ADR 0041 lists "A value block in a publisher feed" as a non-goal. No ADR excludes the support links.
4. **No OP3 prefix.** MSP 2.0 adds the OP3 prefix to each enclosure address (`origin/master:src/utils/xmlGenerator.ts:441`). Studio keeps a prefix that a feed has, because the address does not change. Studio does not add a prefix to a new track. No ADR names OP3.
5. **No Nostr keys for people.** Part 2 gives the details.

### Other Gaps

- **Feed check.** MSP 2.0 `origin/master` examines links, artwork size and shape, `http` addresses and spaces in addresses (`src/utils/feedChecks.ts`). Studio refuses a malformed address only (ADR 0023).
- **First submission to Podcast Index.** MSP 2.0 uses `add/byfeedurl` with a key (`api/pisubmit.ts`). Studio sends only the hub notification ([ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md)). Nobody confirmed that the hub adds a feed that it does not know.
- **A feed address check before the submission.** MSP 2.0 finds a host that refuses robots (`src/utils/verifyFeedUrl.ts`). Studio has no such check.
- **A new album from an album that exists.** MSP 2.0 copies a feed with new GUIDs (`regenerateGuids.ts`). Studio has no copy function and no ADR for one.
- **Nostr Music events.** MSP 2.0 imports and publishes kinds 36787 and 34139 (`nostrMusicConverter.ts`, `nostrSync.ts`). Studio has no ADR for them.
- **More than one role in one person entry.** MSP 2.0 shows one person with many roles. Studio uses one entry for each role. The feed output is the same.
- **Shared payment presets.** MSP 2.0 adds a support split for MSP 2.0 and Podcast Index. The ADR 0064 invariant "Studio writes no field that the user did not set" is against a default split.
- **Video feeds.** The plan lists them as a non-goal (`docs/plans/studio-v1-phase-plan.md:22`).
- **Charts and the admin page.** These are functions of the MSP service, not of an editor.

### Functions That Studio Has And MSP 2.0 Does Not Have

Playlists, live items, licenses, hide from apps, alternate enclosures, five connectors, snapshots, and the two-way publisher links with `rel`.

## Part 2: Nostr Identities

### Where A Feed Holds A Nostr Key

| Form | Parent | Written by | Read by | In the specification |
| --- | --- | --- | --- | --- |
| `<podcast:txt purpose="npub">npub1…</podcast:txt>` | channel | MSP 2.0 for albums (`xmlGenerator.ts:336-338`) | musicindex as `entity_ids` with the scheme `nostr_npub` (`api.json` example). v4vmm reads it from the item first, then from the channel (`metadata_service.rs:55-61`). | `podcast:txt` is in the specification. The `purpose` value `npub` is not in it. |
| `npub="npub1…"` on `podcast:person` | channel and item | MSP 2.0 (`xmlGenerator.ts:200`) | Stophammer keeps it in `source_contributor_claims.npub` (`api.rs:794`, migration 0033). v4vmm reads it (`identity_ingest.rs:239`). | No. `docs/tags/person.md` lists only `role`, `group`, `img` and `href`. |

MSP 2.0 does not write a key into a publisher feed or into an item `podcast:txt`. The corpus has one feed with each form: `phafe-music-the-singles.xml`.

### What Studio Does Now

- `Person` has no `npub` field (`crates/core/src/feed/elements.rs:120-132`). The lossless tree keeps the attribute on a read and a write with no edit. The corpus test enforces that.
- **A defect, found by reading the code:** "Credits for this track" copies the album persons from the model (`crates/api/ui/credits.js:13-16`). The model has no `npub`, so each copy loses its key. This is not tested.
- The model has `txts` on the channel and on each item (`crates/core/src/feed.rs:108`). The UI has no control for them (`crates/api/ui/editor.js:364`).
- The core already parses `npub` and hexadecimal keys (`nostr_events.rs:167`). It already reads kind 10002 from `LOOKUP_RELAYS`, `wss://purplepag.es` and `wss://user.kindpag.es` (`nostr_events.rs:27`).
- The server shell has Nostr sign-in with NIP-07 and NIP-46. The UI knows the `npub` of a signed-in user (`crates/api/ui/account.js:134-137`).
- The UI gets each image through `/images` of the local API (`crates/api/src/workspace.rs:47-50`). A profile picture can use the same route.

### What MSP 2.0 Gives The User

- A text field "Artist npub", and a button that fills it with the key of the signed-in user (`Editor.tsx:435-450`).
- A text field "Nostr npub" in each person entry, on the album and on each track (`Editor.tsx:526`, `Editor.tsx:1317`).
- A button "Use my Nostr wallet" that fills a payment address with `lud16` from the kind 0 profile of the user (`RecipientsList.tsx:65-89`).
- No search, no check of the key and no profile preview. The user must find the `npub` in another app.

### Sources For A Search By Name

| Source | Result of the test on 2026-10-28 |
| --- | --- |
| The collection of the user | The core has each person and each key of the user's feeds. No network request. |
| musicindex `/v1/search` | It gives feeds, tracks and publishers. Its `include` list has only `link_facts`. It gives no key. `/v1/feeds/{guid}` gives `source_ids` and `source_contributors`, with their keys, for one feed. |
| `wss://search.nos.today` | NIP-11 lists NIP-50. A kind 0 search for "phafe" gave three profiles. The first was "Matt Finlay", with the NIP-05 address `mattfinlay@phafe.com`. |
| `wss://relay.primal.net` | It refused the filter: "unrecognised filter item: search". |
| `wss://relay.nostr.band` | No answer in 8 seconds. Its NIP-11 document did not parse. |
| `wss://purplepag.es` | NIP-11 lists no NIP-50. It can give the kind 0 of a known key. |
| NIP-05 | `https://phafe.com/.well-known/nostr.json?name=mattfinlay` gave the key `50a63cca…`. |

**A caution from the test.** The corpus feed gives the band "Fentlay" the key `2d3d1d9b…`. The name search found the person "Matt Finlay" with the key `50a63cca…`. These are two different keys. A search by name can find a related key that is not the correct key. The user must see the evidence for each result before the user selects it.

## Proposal: Find A Nostr Profile

This proposal follows "By address or name" on the Add feeds page. That page accepts a pasted address or a name, and shows the results as rows that the user selects.

### The User Flow

1. A person entry gets a row "Nostr". Its summary is the profile name and a short key, for example "Matt Finlay · npub1g2n…ck". With no key, the summary is "Not set".
2. The album page, the publisher page and the playlist page get the row "Nostr profile" for `podcast:txt purpose="npub"`. The row is in the identity section, after the artist name.
3. The row opens the sheet "Find a Nostr profile". It has one field: "npub, NIP-05 address or name".
4. The field accepts each of these forms:
   - `npub1…`, `nprofile1…`, a key of 64 hexadecimal characters, or a `nostr:` URI (NIP-21).
   - A profile address of a web client that contains an `npub`, such as `https://njump.me/npub1…`.
   - A NIP-05 address, `name@domain`, or `domain` for `_@domain`.
   - Other text is a name.
5. A key or a NIP-05 address gives one result. A name gives a list of results.
6. Each result is one row. The row shows the picture, the display name, the NIP-05 address with its check result, and a short key.
7. The row also shows where Studio found the key. The sources are: "In your feeds", "In 3 musicindex feeds", and "Nostr search".
8. When the NIP-05 domain is the domain of the feed `link`, the row shows "Confirmed at phafe.com". This is the strongest evidence that Studio can give.
9. The user selects a row. Studio sets the key. When the person has no name or no picture, two check boxes offer the profile name and picture as start values.
10. When the user is signed in with Nostr, the sheet shows the button "Use my key", as MSP 2.0 does.
11. A row with a key opens to show the profile card of that key. The user then sees whose key the feed holds. A key with no profile shows "Studio found no profile for this key".

### The Core And The API

- A new route `GET /api/nostr/profiles?q=` gives the results. The UI connects to no relay and to no other server ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- The core parses each key form and gives the `npub` form. Each conversion of a key is in the core, because a conversion changes a feed value ([ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md)).
- The core refuses an `npub` that NIP-19 does not decode, at the draft save.
- A name search runs the three sources in parallel, in this sequence of results: the collection, musicindex, then the search relay. Each source has a time limit, and a slow source does not stop the others.
- A NIP-05 request goes through the network policy. On the server, the private-network guard applies ([ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md)).
- The core gets the kind 0 of a key from `LOOKUP_RELAYS`.
- A track copy of the album credits keeps the key, because the model then has the `npub` field.

### Options For The musicindex Source

- **Option A: Studio reads each feed.** For the first five feed results of `/v1/search`, Studio reads `/v1/feeds/{guid}?include=source_ids,source_contributors`. This needs no change in musicindex. It makes six requests for one search.
- **Option B: musicindex gives a person search.** A new route gives the persons and the keys that match a name, with the count of feeds. This needs a musicindex ADR and a Stophammer change. It gives the best results.
- **Option C: no musicindex source.** The collection and the search relay only.
- **Recommendation:** start with option A, and record option B as a later change. Option A works now, and option B can replace it with no change in the UI.

### Options For The Form In The Feed

- **Option 1: the forms of MSP 2.0.** `podcast:txt purpose="npub"` for the feed, and the `npub` attribute for a person. musicindex and v4vmm read these forms now.
- **Option 2: `href="nostr:npub1…"` on a person.** This form is in the specification, but it uses the only `href` of the person. musicindex does not read it as a key.
- **Recommendation:** option 1. Studio must keep each form that a feed has. A proposal to the podcast namespace for the `npub` attribute is a separate task.

### Other Nostr Gaps

- "Use my Nostr wallet": a recipient row can offer the `lud16` of the profile of the signed-in user. It can also offer the `lud16` of the profile of a selected person. This is a small addition after the profile route exists.
- Nostr Music events (kinds 36787 and 34139) need their own research and ADR.

## Operator Answers

The operator answered on 2026-10-28:

1. The first ADR covers the person key and the feed key together.
2. The musicindex source starts with option A. Option B is a later change.
3. The track page gets no key. The ADR covers artists, publishers and contributors only.
4. Studio uses `wss://search.nos.today` as the default search relay. A setting on the Settings page can change it.
