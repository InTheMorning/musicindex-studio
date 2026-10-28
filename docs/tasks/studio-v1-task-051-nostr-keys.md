# Studio v1 Task 051: Nostr Keys For Artists, Labels And Contributors

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "MSP 2.0 Parity".
Decisions: [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md), [ADR 0016](../adr/0016-ui-without-a-build-step.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) and [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md).
Research: [MSP 2.0 parity and Nostr identities](../research/2026-10-28-msp-parity-and-nostr-identities.md).

Status: Complete - 2026-10-28, except the human test. The four commit checks pass. The screenshot visual check passed in the two themes and at 390 px, against the test relay. The human test is open.

## Goal

The user can give a Nostr key to the artist or label of a feed and to each person of the credits. The sheet "Find a Nostr profile" finds a key from a key, a NIP-05 address or a name, and shows the evidence for each result.

## Packets

The packets go in this sequence. Each packet needs the one before it.

1. [051a: the keys in the feed model and the key rules](studio-v1-task-051a-key-model-and-rules.md). Core only.
2. [051b: the profile route and the search relay setting](studio-v1-task-051b-profile-route.md). Core and API.
3. [051c: the rows, the sheet and the setting in the UI](studio-v1-task-051c-nostr-ui.md). UI only.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0074.

- The JSON model of a person gets `npub` (a text or `null`). The feed key stays an entry of `txts` with `purpose` `npub`. The API gives no second field for it.
- The core changes the form of a key in `pipeline::replace_model`, with the other form steps. The function `nostr_events::npub_text` gives the `npub` form of each accepted text.
- A key that NIP-19 does not decode stays in the draft as the user typed it. `Feed::bad_keys` lists it, as `Feed::bad_addresses` lists a bad address. The draft response and the save response give `bad_keys`.
- The write refuses a bad key only when the hosted feed does not have the same text in a key field. A bad key that came from the host does not stop a write (ADR 0074 item 5).
- The profile code is in a new core module `crates/core/src/profiles.rs`. The relay client gets a variant of `fetch` that gives the events that arrived before the time limit (ADR 0074 item 10).
- The search relay setting is the collection setting `nostr_search_relay`. A missing setting means `wss://search.nos.today`. An empty text means no relay search.
- The UI model module is `crates/api/ui/nostrkey.js`.

## Acceptance Criteria

Mechanical: the mechanical criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`, against the test relay of the visual check harness. In the dark theme, in the light theme and at 390 px:

- A person entry shows the row "Nostr" with "Not set", and with a profile name and a short key.
- The album page, the publisher page and the playlist page show "Nostr profile" after the artist or the author.
- The sheet shows results with a picture, a name, a NIP-05 check, a short key and the sources. A result with `domain_match` shows "Confirmed at" and the domain.
- A row with a key opens the profile card with "Change" and "Remove".
- The Settings page shows "Nostr search relay".
- `$edges()` finds no content at the edge of a group.

Human test, by the operator:

- Find the key of a contributor by name, and of an artist by NIP-05 address. Examine the diff, and write the feed.
- Copy the album credits to a track. Each copy keeps its key.
