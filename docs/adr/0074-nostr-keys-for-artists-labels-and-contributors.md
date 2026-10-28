# ADR 0074: Nostr Keys For Artists, Labels And Contributors

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 8, for the copy of the album credits.

## Context

The [MSP 2.0 parity and Nostr identities research](../research/2026-10-28-msp-parity-and-nostr-identities.md) gives the facts and the sources of this ADR.

- MSP 2.0 writes the Nostr key of an album artist as `<podcast:txt purpose="npub">npub1…</podcast:txt>` in the channel. It writes the key of a person as the attribute `npub` of `podcast:person`.
- The podcast namespace does not name the `purpose` value `npub` or the `npub` attribute. musicindex and v4vmm read both forms.
- MSP 2.0 gives two text fields and a button "use my key". It has no search, no check of a key and no profile preview.
- The Studio `Person` has no `npub` field. The lossless tree keeps the attribute, but the UI cannot edit it. "Credits for this track" copies the album persons from the model (`crates/api/ui/credits.js:13`), so each copy loses its key.
- The Studio model has the `podcast:txt` entries of a channel, but the UI has no control for them.
- On 2026-10-28, a NIP-50 search for "phafe" on `wss://search.nos.today` found the person "Matt Finlay". The corpus feed gives the band "Fentlay" a different key. A search by name can find a related key that is not the correct key.
- On 2026-10-28, `wss://search.nos.today` sent its first profile after 1.2 seconds and `EOSE` after 4.3 to 10.8 seconds.
- musicindex `/v1/search` gives feeds, not keys. `/v1/feeds/{guid}` gives the keys of one feed in `source_ids` and `source_contributors`.

The operator selected each decision of this ADR from a questionnaire on 2026-10-28.

## Decision

### The Feed Model

1. `Person` gets the field `npub` for the attribute `npub` of `podcast:person`. A read and a write with no edit give the same bytes.
2. The feed key is the channel `podcast:txt` with `purpose="npub"`. The album page, the publisher page and the playlist page edit it. A track has no key of its own.
3. Studio keeps each form of a key that a feed has. Studio writes the forms of item 1 and item 2, and no other form.

### The Core Applies The Key Rules

4. At the draft save, the core examines each key that the user changed. It accepts `npub1…`, `nprofile1…`, 64 hexadecimal characters and a `nostr:` URI (NIP-21). It writes the `npub` form.
5. The core refuses a changed key that NIP-19 does not decode. The error is on the field. A key that the user did not change stays as it is.
6. These rules are feed rules of [ADR 0065](0065-the-core-applies-each-feed-rule.md). The UI does not change the form of a key.

### The Profile Route

7. The new route `GET /api/nostr/profiles?q=TEXT&feed=ID` gives the profiles that match the text. The UI connects to no relay and to no other server ([ADR 0016](0016-ui-without-a-build-step.md)).
8. The core reads the text as one of these forms, in this sequence:
   - A key of item 4.
   - A web address that contains an `npub` or an `nprofile`, such as `https://njump.me/npub1…`.
   - A NIP-05 address, `name@domain`, or a domain for `_@domain`.
   - Other text is a name.
9. A key or a NIP-05 address gives one profile. A name gives a list of profiles from three sources:
   - The collection: each person and each feed key of the user's feeds, with a name that contains the text.
   - musicindex: the core reads `/v1/feeds/{guid}?include=source_ids,source_contributors` for the first five feed results of `/v1/search`. It keeps each key with a name that contains the text.
   - The search relay: a NIP-50 request for kind 0.
10. The core asks the three sources at the same time. Each source has a limit of 5 seconds. A slow or failed source does not stop the others. The answer names each source that did not answer. The relay source keeps each profile that arrives before the limit, also when the relay did not send `EOSE`.
11. The core merges the results by key. A result gives the key, the display name, the picture address, the NIP-05 address, the result of the NIP-05 check and each source. For musicindex, the source gives the number of feeds.
12. The core gets the kind 0 of each key from `LOOKUP_RELAYS` (`crates/core/src/nostr_events.rs:27`). The newest event wins.
13. The core checks each NIP-05 address. When `feed` is given and the NIP-05 domain is the domain of the feed `link`, the result has `domain_match`.
14. Each request of the route goes through the network policy. On the server, the private-network guard applies ([ADR 0057](0057-the-private-network-guard-of-the-server.md)).
15. The UI gets each profile picture through `/images` ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 13).

### The Search Relay Setting

16. The Settings page gets the field "Nostr search relay", in the tier `setup` ([ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)). Its start value is `wss://search.nos.today`.
17. An empty field stops the relay search. The other sources and the key forms continue to work.

### The UI

18. Each person entry gets the row "Nostr". Its summary is the profile name and a short key, for example "Matt Finlay · npub1g2n…ck". With no key, the summary is "Not set".
19. The first section of the album page, the publisher page and the playlist page gets the row "Nostr profile". The row is after the field of the artist or the author.
20. Each of the two rows opens the sheet "Find a Nostr profile". The sheet has one field, "npub, NIP-05 address or name", and the button "Search".
21. Each result is one row. The row shows the picture, the display name, the NIP-05 address with its check result, a short key and its sources: "In your feeds", "In 3 musicindex feeds" or "Nostr search".
22. A result with `domain_match` shows "Confirmed at" and the domain.
23. The user selects a result. Studio sets the key. In a person entry with no name or no picture, two check boxes offer the profile name and the profile picture as start values.
24. When the session has a Nostr key, the sheet shows the button "Use my key".
25. A row with a key opens the sheet with the profile card of that key, and the buttons "Change" and "Remove". A key with no profile shows "Studio found no profile for this key".

### The Credits Of A Track

26. "Credits for this track" copies each album person with its key. This changes ADR 0064 item 8 for the key only.

## Alternatives Considered

- Write `href="nostr:npub1…"` on a person. Not selected: musicindex does not read it as a key, and the person then has no web address.
- A key on each track, as an item `podcast:txt`. Not selected: the operator wants keys for artists, labels and contributors. MSP 2.0 does not write it.
- Start with the person key only. Not selected: one sheet serves the two keys, and the operator selected both.
- A person search in musicindex. Not selected now: it needs a musicindex ADR and a Stophammer change. It can replace item 9 later with no change to the UI.
- No musicindex source. Not selected: the keys that feed authors wrote are the strongest evidence for a name.
- A fixed search relay. Not selected: no other relay passed the test on 2026-10-28, and a relay can stop.
- The UI connects to the relays. Rejected: [ADR 0016](0016-ui-without-a-build-step.md).

## Consequences

- An MSP 2.0 user can edit each Nostr key that MSP 2.0 writes.
- A user can find a key by name, and sees the evidence for each result before the selection.
- The search relay and musicindex receive the text of each name search. The NIP-05 domain receives each NIP-05 check.
- The core gets a NIP-50 client. The test relay of the core needs the `search` filter for the tests.

## Invariants

- Studio writes a Nostr key only in the forms of items 1 and 2.
- Studio sets no key without a selection or an entry by the user.

## Non-Goals

- Nostr Music events (kinds 36787 and 34139).
- A payment address from the `lud16` of a profile. A subsequent decision can add it.
- A proposal to the podcast namespace for the `npub` attribute.

## Verification

Mechanical:

- The corpus tests: `phafe-music-the-singles.xml` gives the same bytes after a read and a write with no edit.
- A core test sets a person key and a feed key. The write changes only those elements.
- A core test saves each key form of item 4, and gets the `npub` form. A changed key that does not decode gives an error on its field. A key that did not change stays as it is.
- An API test of `/nostr/profiles` with the test relay: a key, an `nprofile`, a web address, a NIP-05 address with a local test server, and a name with each of the three sources. A source that does not answer gives the other results and names that source.
- An API test: `domain_match` is true only for the domain of the feed `link`.
- A Node test: `withTrackCredits` keeps the key of each person.
- A Node test: the summary of the row "Nostr" with a key and with no key.

Visual:

- The person entry, the row "Nostr profile" and the sheet with results, in the two themes, at desktop width and at 390 px.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.
