# Studio v1 Task 240: The Other GUID Change Paths

Decision: [ADR 0249](../adr/0249-studio-strongly-discourages-a-feed-guid-change.md) items 8 to 11.

Status: Done.

- ChatGPT built the task. The main session read the diff and ran `tools/gate.sh`.
- The main session made these corrections:
    - `MusicIndex::feeds_by_url` keeps only an item with `match` equal to `feed_url`. Before, a `copy` or an `observed_url` match also gave an `index_address` change. A test proves that a copy match gives no item.
    - One function `collection_guid_notice` gives the notice of item 10, for the add by address (`import_feeds`, key `notices`) and for the found feeds (`discover`, key `guid_notice` of each feed). `FoundFeeds` shows the notice in the row of the feed. Before, the add view kept the notices in a value that it did not show.
- Operator choice of the main session (step 4): the by-url read sends its request while it holds the workspace lock, only for a feed that Studio did not read or write before. Thus the lock waits for the network only before the first write of a new feed.
- Screenshots (`.studio-dev/v-230/o240`, steps `s240.json`, script `pre-240.sh`): a different feed file was put at the Starbound path on host 3 for the run. After a scan of host 3, Starbound moved there with **Replace the File**. Review shows the Stop panel "This move replaces a file with a different GUID" with the two GUIDs and no **Keep** button. The write button is off until "Starbound - Single" is typed. The planted file was then removed.

## Problem

- A move with **Replace the File** writes the moved feed over a file with a different GUID. Review says "This move replaces a different file." and gives no GUID.
- A new feed can get an address where Music Index holds a different feed. Studio does not ask Music Index before the first write.
- The add by address skips an address of the collection with a different GUID, and gives no message.
- The check treats 404 with `superseded_by` as a feed that Music Index does not hold.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| GUID change | extend | crates/core/src/pipeline.rs | `GuidChange`, `guid_change` and `guid_change_of` from task 238 get a kind. The write guard and the Stop panel then cover items 8 and 9. |
| Replaced file mark | reuse | crates/core/src/pipeline.rs | `FeedMove::replaces_file` (`FileMark`) gives the GUID of the replaced file. |
| Index client | extend | crates/core/src/musicindex.rs | `MusicIndex` gets a read of `GET /v1/feeds/by-url`. `get_with_query` sends it. |
| Index presence | reuse | crates/core/src/musicindex.rs | `IndexPresence::Superseded` gives the new GUID of item 11. |
| Index facts | extend | crates/core/src/feed_reads.rs | `gather_index` and `IndexFacts` keep the new GUID of `superseded_by`. |
| Check | extend | crates/core/src/feed_check.rs | `check` gives `index_guid_changed`. |
| GUID panel model | extend | crates/api/ui/feedcheck.js | `guidChangeView` from task 238 gets the texts of each kind. |
| Add by address | extend | crates/api/src/routes.rs | `import_feeds` and `is_known` give the notices of item 10. |
| Found feeds view | extend | crates/api/ui/app.js | The add result view shows the notices. |

## Changes

1. Read ADR 0249 items 3, 4 and 8 to 11, ADR 0175, and tasks 238 and 239 first. Read `../stophammer/src/query.rs` from line 7125 (`GET /v1/feeds/by-url`, Stophammer ADR 0088) for the query and the response. Then read `GuidChange`, `guid_change`, `guid_change_of`, `write_draft_signed_inner`, `FeedMove`, `FileMark`, `set_feed_host_replacing` and `file_at_feed_path` in `crates/core/src/pipeline.rs`. Read `MusicIndex`, `feed_presence`, `get_with_query` and `IndexPresence` in `crates/core/src/musicindex.rs` and `TestServer` in `crates/core/src/testing.rs`. Read `gather_index`, `IndexFacts` and `ScriptedIndexReads` in `crates/core/src/feed_reads.rs`. Read `diff_json_with_streams`, `write_feed`, `import_feeds`, `is_known` and `collection_keys` in `crates/api/src/routes.rs`. Read `guidChangeView` in `crates/api/ui/feedcheck.js`, `ReviewSheet` and `ReplaceWarning` in `crates/api/ui/editor.js`, `replaceWarning` in `crates/api/ui/releasemove.js`, and `FoundFeeds` and the add by address in `crates/api/ui/app.js`.
2. Core, the kind. Add `kind: GuidChangeKind` to `GuidChange`, serialized in snake case: `feed` (task 238), `replaced_file` (item 8) and `index_address` (item 9). `guid_change_of` gives the first change in this sequence: `feed`, `replaced_file`, `index_address`. The confirmation of task 238 applies to each kind. A grant write refuses each kind.
3. Core, item 8. When the feed move of the draft has `replaces_file` with a GUID that is not a placeholder and not equal to the GUID of the draft, give a `replaced_file` change with `known` equal to that GUID and `new` equal to the draft GUID. A confirmed write does not change the known GUID of the feed for this kind.
4. Core, item 9.
    - Add `MusicIndex::feeds_by_url(url) -> Result<Vec<(String, String)>, WebError>` that gives the `feed_guid` and the `feed_url` of each item.
    - The read applies only to a feed that Studio has never read or written: the row has no `last_version`, and `Collection::feed_write(id)` gives none. Operator choice of the main session: a feed that Studio read from a host has a GUID of its own, and item 6 covers a change there. Thus an old collection sends no read for each feed.
    - The API reads before the core call, in `diff_json_with_streams` and in `write_feed`, with the public address of the draft. The core gets the result as an argument `index_at_address: IndexAtAddress` with the values `NotAsked`, `Unread` and `Read(Vec<(String, String)>)`. Do not send a request from the core write.
    - A `Read` item whose `feed_url` is equal to the address (or `same_but_spelling` gives true) and whose `feed_guid` is not the draft GUID gives an `index_address` change with `known` equal to that `feed_guid`.
    - `Unread` gives the key `"index_unread": true` in the diff JSON, and does not stop the write.
5. API and UI, items 8 and 9. `diff_json_with_streams` gives `guid_change` with its `kind`. `guidChangeView(change, typed)` gives the texts of each kind:
    - `feed`: the texts of task 238.
    - `replaced_file`: title "This move replaces a file with a different GUID". `lines`: ["The address served the GUID {known}. It then serves {new}.", "Payments that name {known} stop."]. No `keep` button.
    - `index_address`: title "Music Index has a different feed at this address". `lines`: ["Music Index holds {known} at this address. This feed has {new}.", "Payments that name {known} stop."]. No `keep` button.
    - The prompt and `confirmed` are the same for each kind.

    `ReviewSheet` draws the button only when `keep` is present. With `replaced_file`, the panel takes the place of `ReplaceWarning`. With `index_unread`, Review shows the line "Studio could not ask Music Index about this address." The change-set page uses the same model.
6. Item 10. `import_feeds` gives `notices` in its answer. When the read address of a feed is an address of the collection and the read GUID is not the GUID of that row (known GUID, else `podcast_guid`), add "{address} is in the collection already with a different GUID." Studio adds nothing for that address, as before. The add view in `crates/api/ui/app.js` shows each notice as a Notice. Put the text in one function in a UI module that has Node tests, or give it from the API.
7. Item 11. `gather_index` keeps the new GUID of `IndexPresence::Superseded` in a new field `IndexFacts::superseded_by: Option<String>`. `check` gives the finding `index_guid_changed` (`FindingKind::IndexGuidChanged`) in place of the line of ADR 0175 item 4 for that answer:
    - level: Warning.
    - title: "Music Index changed the GUID of this feed".
    - reason: "Music Index now holds this feed as {new}. This draft has {guid}."
    - no repair.
8. Tests:
    - Rust, `crates/core/tests/`: a move with **Replace the File** over a file with a different GUID gives a `replaced_file` change, and the write stops with no confirmation and writes with the title. A replaced file with the same GUID gives no change.
    - Rust: `Read` with a different `feed_guid` at the address gives an `index_address` change and stops the write. `Unread` does not stop the write. A feed with `last_version` gets `NotAsked`. A test of `feeds_by_url` with `TestServer`.
    - Rust, API tests in `crates/api/src/tests.rs`: the add by address of an address of the collection with a different GUID gives the notice and adds nothing.
    - Rust: a scripted index answer 404 with `superseded_by` gives `index_guid_changed` with the texts of step 7, and not the line of ADR 0175 item 4.
    - Node: `guidChangeView` gives the texts of each kind, and `keep` only for `feed`. The text of the add notice.
    - Each failure message names ADR 0249 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0249 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not compare GUIDs or addresses.
- Do not change a file in `../stophammer`.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 8 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- The move review with **Replace the File** over a file with a different GUID shows the Stop panel of item 8. The write button is off until the title is typed.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
