# Studio v1 Task 181: A Link To The Release On Music Index

Decision: [ADR 0220](../adr/0220-a-release-page-links-its-page-on-music-index.md).

Status: Open.

## Problem

No control on a release page opens the release on musicindex.org (observation O8). The outside check of the release page asks the index, but its answer does not tell if the index holds the feed.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Outside check route | extend | crates/api/src/routes.rs | `outside_check` near line 1856 and `outside_check_reads` near line 1977. Add `held` to the answer (ADR 0220 item 2). |
| Outside check of the release page | extend | crates/api/ui/app.js | `OutsideNotices` near line 4853 loads the answer. Give `held` to the header. |
| Release page header | extend | crates/api/ui/app.js | `EditableFeed` near line 2243, the `feed-head` block with the feed address. |
| Music Index feed link | reuse | crates/api/ui/people.js | `musicindexFeedUrl` near line 2496. Do not write a second builder. |
| Link text | reuse | crates/api/ui/peoplepages.js | "Open on musicindex.org" near line 822, with `target="_blank"` and `rel="noopener"`. |

## Changes

1. API: the answer of `GET /feeds/{id}/outside` gets `held`. It is `true` when the index holds the feed, and `false` when the index does not hold it. It is `null` when the request has no `index=true` or the read failed. Read `IndexPresence` in `crates/core/src/musicindex.rs` for the values.
2. UI: a model function gives the link of the header from the feed and the outside answer: `{ href, text }` with `musicindexFeedUrl(feed.podcast_guid)` and "Open on musicindex.org", or null. It gives null when `held` is not `true` or when the feed has no `podcast_guid`.
3. UI: the release page header shows the link under the feed address. It opens in the system browser through `target="_blank"`. `OutsideNotices` keeps its notices. Give the answer to the header with a callback or by a move of the load to `EditableFeed`. Send no second request.

## Files

- Change: `crates/api/src/routes.rs`, `crates/api/ui/app.js`, and the UI module of the model function of change 2. Put that function next to `indexNotice` in `crates/api/ui/outsidecheck.js`.
- Change tests: the API test of the outside route (find it in `crates/api/src/tests/`), `crates/api/ui-tests/outsidecheck.test.mjs`.

## Checks

Mechanical:

- API: `held` is `true` for a held feed, `false` for a feed that the index does not hold, and `null` with no `index=true`.
- UI: the model function gives the link for a GUID with `held` `true`, and null with `held` `false`, with `held` `null` and with no GUID.
- `cargo test -p studio-api`, `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px: the release page header with the link.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of ADR 0220. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.
