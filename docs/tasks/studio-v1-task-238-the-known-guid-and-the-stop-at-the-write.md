# Studio v1 Task 238: The Known GUID And The Stop At The Write

Decision: [ADR 0249](../adr/0249-studio-strongly-discourages-a-feed-guid-change.md) items 1 to 5.

Status: Done.

- ChatGPT built the task. The main session read the diff and ran `tools/gate.sh`.
- The main session made these corrections:
    - The change-set page reads the diff of each feed at the start and after a repair. Before, a repair made the write button stay off, because only the open diff loaded again. A diff that is not loaded does not block the button, because the core refuses an unconfirmed write.
    - One function `guid_change_of` gives the comparison for `guid_change` and for the write. Before, the write had a copy.
    - A test in `crates/core/tests/placeholder_guid.rs` proves that a placeholder that gets its real GUID is not a GUID change.
- Screenshots (`.studio-dev/v-230/o238`, steps `s238.json`): the hosted Elijah Lied file got a different GUID for the run, and then the original file came back. Review shows the Stop panel above the diff. The write button is off until the title is typed. After **Keep the Old GUID**, the panel goes, and the diff shows the change back to the known GUID. This is correct, because the hosted file has the new GUID.

## Problem

- The write compares no GUID. A merge, a restored version or the Discard of a change set can give a draft with a different `podcast:guid`. The write takes it, and Review shows only an ordinary line.
- `Collection::record_state` (`crates/core/src/collection.rs`) sets `feeds.podcast_guid` at each read. Thus the collection has no GUID that a read does not change.
- A grant write (`write_granted_with` in `crates/core/src/pipeline.rs`) starts from the hosted file and writes its GUID.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Collection schema | extend | crates/core/src/collection.rs | `MIGRATIONS` adds the column `known_guid`. `FeedRow` gets the field. |
| Read record | extend | crates/core/src/collection.rs | `record_state` sets `known_guid` only when it has no value. |
| New feed | extend | crates/core/src/pipeline.rs | `new_feed_with` sets the known GUID of a new feed. |
| Single write | extend | crates/core/src/pipeline.rs | `write_draft_signed_inner` is the one path of each write, also of a grant write. The guard goes there. |
| Write route | extend | crates/api/src/routes.rs | `Write` and `write_feed` carry the confirmation. |
| API errors | extend | crates/api/src/error.rs | `From<PipelineError>` gives the new code. |
| Review JSON | extend | crates/api/src/routes.rs | `diff_json_with_streams` gives `guid_change`. |
| Refusal panel model | extend | crates/api/ui/feedcheck.js | `pageRefusalView` is the model of a Stop panel. A new function gives the model of the GUID panel next to it. |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` draws the panels above `DiffView` and has the write condition. |
| Change-set page | extend | crates/api/ui/app.js | `ChangeSetPage` and `writeChangeSetFeed` write each feed of a change set. |

## Changes

1. Read ADR 0249, ADR 0008 item 2, ADR 0122 item 8 and ADR 0255 first. Then read `MIGRATIONS`, `SCHEMA_VERSION`, `run_migrations`, `FeedRow`, `FEED_SELECT`, `add_feed` and `record_state` in `crates/core/src/collection.rs`. Read `new_feed_with`, `assign_real_guids`, `write_draft_signed_inner`, `write_draft_signed_checked`, `write_granted_with` and `is_placeholder` (or the test for a `pending-` GUID) in `crates/core/src/pipeline.rs`. Read `Write`, `write_feed` and `diff_json_with_streams` in `crates/api/src/routes.rs`, `crates/api/src/error.rs`, `ReviewSheet` in `crates/api/ui/editor.js`, `ChangeSetPage` and `writeChangeSetFeed` in `crates/api/ui/app.js`, and `pageRefusalView` in `crates/api/ui/feedcheck.js`.
2. Core, item 1:
    - Add a migration `ALTER TABLE feeds ADD COLUMN known_guid TEXT`, then `UPDATE feeds SET known_guid = podcast_guid WHERE podcast_guid IS NOT NULL AND podcast_guid NOT LIKE 'pending-%'` (use the placeholder test of the code if it is not this prefix).
    - Add `known_guid: Option<String>` to `FeedRow` and to `FEED_SELECT`.
    - `record_state` sets `known_guid` to the read GUID only when `known_guid` is NULL and the read GUID is not a placeholder. It keeps setting `podcast_guid` to the GUID of the read.
    - `new_feed_with` sets `known_guid` to the GUID of the new model when it is not a placeholder.
    - Add `Collection::set_known_guid(id, guid)`.
    - A known GUID that is NULL means: no known GUID. A draft with any GUID is then not a GUID change.
3. Core, item 2. Add one public function `guid_change(collection, id) -> Result<Option<GuidChange>, PipelineError>` with `GuidChange { known: String, new: String, title: Option<String> }`. It gives `Some` when the row has a known GUID, the draft has a `podcast:guid` that is not a placeholder, and the two are not equal. Use the draft that the write reads.
4. Core, items 3 and 4. `write_draft_signed_inner` gets an argument `confirm: Option<&str>`. Before any host change, when `guid_change` gives `Some(change)`:
    - When the write is a grant write (`LinkCheck::GrantedWrite`), refuse with the new `PipelineErrorKind::GuidChangeRefused` and the message "ADR 0249 item 5: a connected app cannot change the feed GUID".
    - Else, when `confirm`, with no white space at each end, is not equal to the title of the change (or `CHANGE` when the title is none or empty), refuse with `PipelineErrorKind::GuidChangeUnconfirmed` and the message "ADR 0249 item 4: this write changes the feed GUID. Type the feed title to confirm."
    - After a successful write of a GUID change, call `set_known_guid` with the new GUID.
    - After a successful write when the row has no known GUID and the draft GUID is not a placeholder, set the known GUID to the draft GUID.
    - Pass `None` from each other caller. `write_draft` and `write_draft_signed` keep their signatures for old callers, or get a `_confirmed` variant. Do not copy the body.
5. API. Add `confirm_guid_change: Option<String>` to `Write` (serde default). `write_feed` passes it. `crates/api/src/error.rs` maps `GuidChangeUnconfirmed` to 409 `guid_change_unconfirmed` and `GuidChangeRefused` to 409 `guid_change_refused`. `diff_json_with_streams` adds `"guid_change": {"known", "new", "title"}` when `guid_change` gives `Some`, else no key.
6. UI model, item 3. In `crates/api/ui/feedcheck.js`, next to `pageRefusalView`, add `guidChangeView(change, typed)`. It gives `null` for no change. Else it gives:
    - `title`: "This write changes the GUID".
    - `guids`: "Known: {known} → New: {new}".
    - `lines`: ["Payments from other music feeds that name {known} stop.", "Apps can show the feed as a new show."].
    - `keep`: "Keep the Old GUID".
    - `prompt`: "Type {title} to change the GUID" (`CHANGE` when the title is empty).
    - `confirmed`: true when `typed.trim()` is equal to the expected text.
7. UI, Review sheet. In `ReviewSheet`, draw the panel above `DiffView` with the style of the page refusal panel (level Stop). It has the button **Keep the Old GUID** and one text field. The button calls the route of the next paragraph, then loads the diff again. Add `!guid.confirmed` to the `disabled` condition of the write button. Send `confirm_guid_change` with the typed text in the write request. A 409 `guid_change_unconfirmed` shows the message of the error.

    The editor refuses a change of the feed GUID (`keep_identity`). Thus **Keep the Old GUID** cannot use the editor save. Add a core function `keep_known_guid(collection, id)` that sets the draft `podcast:guid` to the known GUID through `changed_draft`, and a route `POST /api/feeds/{id}/keep-known-guid` that the button calls. Register the route next to the write route.
8. UI, change-set page. In `ChangeSetPage`, each feed whose diff has `guid_change` shows the same panel in its row, open. The **Write {n} Feeds** button stays off until each such panel is confirmed. `writeChangeSetFeed` sends the typed text of that feed.
9. Tests:
    - Rust, `crates/core/src/collection.rs` tests: the migration fills `known_guid` from `podcast_guid` and leaves a placeholder NULL. `record_state` with a new GUID keeps `known_guid` and changes `podcast_guid`. `record_state` on a row with no known GUID sets it.
    - Rust, `crates/core/tests/` (the pipeline or write test file): a draft whose `podcast:guid` differs from the known GUID refuses the write with no confirmation and with a wrong text, and writes with the title. The known GUID then is the new GUID. A placeholder that gets its real GUID (ADR 0122 item 8) writes with no confirmation. A restored version (`draft_from_snapshot`) with a different GUID gives `guid_change`. `keep_known_guid` makes `guid_change` give `None`.
    - Rust: a grant write (`write_granted`) from a hosted file with a different GUID is refused with `GuidChangeRefused`.
    - Rust, API tests in `crates/api/src/tests.rs`: `GET /api/feeds/{id}/diff` gives `guid_change`. `POST /api/feeds/{id}/write` with no `confirm_guid_change` gives 409 `guid_change_unconfirmed`.
    - Node, `crates/api/ui-tests/feedcheck.test.mjs`: `guidChangeView` gives the texts of step 6. `confirmed` is false for "this is it" when the title is "This Is It", and true for "  This Is It ". A feed with no title expects `CHANGE`.
    - Each failure message names ADR 0249 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0249 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not compare GUIDs. It shows what the core gives.
- Do not change the item GUID rules.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 9 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- Review of a draft with a different GUID shows the Stop panel above the diff. The write button is off until the title is typed.
- **Keep the Old GUID** removes the panel.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
