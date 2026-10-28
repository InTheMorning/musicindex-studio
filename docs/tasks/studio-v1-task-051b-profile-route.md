# Studio v1 Task 051b: The Profile Route And The Search Relay Setting

Parent: [task 051](studio-v1-task-051-nostr-keys.md). Decision: [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md) items 7 to 17.

Status: Complete - 2026-10-28. The four commit checks pass. These are the differences from the packet:

- Each `picture` in the answer is the address from the kind 0 event, not an `/images` path. The UI loads it through `useImage`, and "Use the profile picture" puts this address into the person.
- The last recorded state of a feed with no draft is its newest snapshot. A feed with no draft and no snapshot gives no keys to the source "In your feeds".
- The NIP-05 check ignores each answer that came through a redirect, as NIP-05 requires.

## Goal

`GET /api/nostr/profiles` gives the profiles for a key, a web address, a NIP-05 address or a name. A name search uses the collection, musicindex and the search relay. The collection setting `nostr_search_relay` gives the search relay.

## Files

Inspect: `crates/core/src/connector/relay.rs` (`fetch`, `connect`), `crates/core/src/testing/relay.rs` (`TestRelay`, `on_req`), `crates/core/src/nostr_events.rs` (`LOOKUP_RELAYS`, `npub_text` of task 051a), `crates/core/src/musicindex.rs` (`MusicIndex`, `search`, `SearchHit`), `crates/core/src/web.rs` (`Web`, `get_with_query`), `crates/core/src/net.rs` (`NetPolicy`), `crates/core/src/collection.rs` (`setting`, `set_setting`), `crates/core/src/announce.rs` (`AnnounceSettings`, a settings example), `crates/api/src/routes.rs` (`search`, `get_announce_settings`, `put_announce_settings`), `crates/api/src/workspace.rs` (`lookup_relays`, `ImageCache`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs` (the musicindex test server of `search_gives_feed_hits_from_musicindex`).

Change: a new file `crates/core/src/profiles.rs`, `crates/core/src/lib.rs`, `crates/core/src/connector/relay.rs`, `crates/core/src/testing/relay.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, and the server route list when the server shell needs it.

Do not change: the UI, the feed model, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `relay.rs` gets `fetch_until(relay_url, filter, limit, policy) -> Result<Vec<Event>, ConnectorError>`. It gives the events that arrived before `EOSE` or before the time limit. It gives an error only when the relay does not connect.
- `TestRelay` applies the `search` field of a filter: an event matches when its `content` contains each word of `search`, with no difference of letter case. The tests of the relay continue to pass.
- `profiles.rs`:
  - `parse_query(text) -> Query`, with `Query::Key(PublicKey)`, `Query::Nip05 { name, domain }` and `Query::Name(String)`. The sequence of ADR 0074 item 8. In a web address, find the first word that starts with `npub1` or `nprofile1`, and give it to `npub_text`. A text with one `@` and a dot after it is a NIP-05 address. A text with no space, with a dot and with no `@` is a domain, with the name `_`.
  - `Profile { npub, name, picture, nip05, nip05_ok, domain_match, sources }`. `name` is `display_name`, or `name` when `display_name` is empty. `Source` is `Collection`, `MusicIndex { feeds }` or `Relay`.
  - `Found { profiles, failed: Vec<String> }`. `failed` holds `collection`, `musicindex` or `relay`.
  - `find(context, query, feed_link) -> Found`. `context` holds the `Web`, the `NetPolicy`, the lookup relays, the search relay (an `Option`), the `MusicIndex` client and the keys of the collection as `(name, npub)` pairs.
  - A key gives one profile. A NIP-05 address gives one profile, or none when the domain does not give a key for the name.
  - A name: the three sources run in `std::thread::scope`, with a limit of 5 seconds each.
    - The collection source keeps each pair with a name that contains the text. Letter case is not important.
    - The musicindex source reads `/v1/search?q=TEXT&type=feed`. Then it reads `/v1/feeds/{guid}?include=source_ids,source_contributors` for the first five hits.
    - From each feed, it keeps each `nostr_npub` of `source_ids`, with the feed artist as its name. It also keeps each contributor `npub` with its name. It keeps a key only when its name contains the text, and it counts the feeds of each key.
    - The relay source uses `fetch_until` with `{"kinds": [0], "search": TEXT, "limit": 20}`.
  - After the sources, the core reads the kind 0 of each key from the lookup relays with `fetch_until` and a limit of 5 seconds. The newest event of a key wins. A key with no kind 0 keeps the name from its source.
  - NIP-05: `GET https://DOMAIN/.well-known/nostr.json?name=NAME` through `Web`, with a limit of 64 KiB. `nip05_ok` is true when `names[NAME]` is the hexadecimal key. Check each NIP-05 address of a kind 0, with the same limit of 5 seconds.
  - `domain_match` is true when `nip05_ok` is true and the NIP-05 domain is the host of `feed_link`. Compare the two with no `www.` prefix and in lower case.
  - The result sequence: the profiles with `domain_match`, then the profiles with more sources, then the others. In each group, the sequence of the first source.
- The route `GET /api/nostr/profiles?q=TEXT&feed=ID` gives `{ "profiles": [...], "failed": [...] }`. `feed` is optional, and gives the `link` of the draft of that feed. Each `picture` in the answer is the address of the image route `/images` of the API, as the other images have. An empty `q` gives 422 `invalid_query`.
- The keys of the collection come from the draft model of each feed, or from its last recorded state when it has no draft. They include each person key and each feed key.
- The routes `GET /api/settings/nostr` and `PUT /api/settings/nostr` read and set `{ "search_relay": TEXT }`. A missing setting gives `wss://search.nos.today`. The PUT refuses a text that is not empty and does not start with `wss://` or `ws://`.
- Each request goes through the `NetPolicy` of the workspace (ADR 0057).

## Steps

1. Write `fetch_until` and the `search` filter of `TestRelay`, with tests.
2. Write `profiles.rs` with unit tests for `parse_query`, the merge and the sequence.
3. Write the routes, with API tests against `TestRelay`, a local NIP-05 server and the local musicindex test server.
4. Run the four commit checks.

## Constraints

- No test sends a request to a host outside the computer.
- No secret is in a log or a response.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- A core test: `fetch_until` gives the events of a relay that does not send `EOSE`, after the limit.
- A core test: `parse_query` gives `Key` for an `npub`, an `nprofile`, a hexadecimal key, `nostr:npub1…` and `https://njump.me/npub1…`. It gives `Nip05` for `name@example.com` and for `example.com`. It gives `Name` for `Matt Finlay`.
- An API test: a key gives its profile from the test relay.
- An API test: a NIP-05 address gives its profile with `nip05_ok`. A local NIP-05 server with a different key gives `nip05_ok` false.
- An API test: a name gives profiles from the three sources, with their sources merged by key.
- An API test: a source that does not answer gives the other results, and `failed` names it.
- An API test: `domain_match` is true only for the host of the feed `link`.
- An API test: an empty search relay setting gives no relay request.
- An API test on the server shell: a NIP-05 domain at a private address gives no request with the policy `Public`.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `Web` cannot apply the network policy to a NIP-05 request.
- Stop and report when the image route cannot give a profile picture from its address.
- Stop and report when the collection has no function that gives the last recorded state of a feed.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md
- docs/tasks/studio-v1-task-051-nostr-keys.md
- docs/tasks/studio-v1-task-051b-profile-route.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- GET /api/nostr/profiles gives profiles for a key, a web address, a NIP-05 address or a name, from the collection, musicindex and the search relay. The settings routes give the search relay.

Constraints:
- Follow the Architecture section of the task file fully.
- No test sends a request to a host outside the computer.

Do not touch:
- The UI, the feed model, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
