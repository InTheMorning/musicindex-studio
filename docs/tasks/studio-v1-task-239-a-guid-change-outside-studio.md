# Studio v1 Task 239: A GUID Change Outside Studio

Decision: [ADR 0249](../adr/0249-studio-strongly-discourages-a-feed-guid-change.md) items 6 and 7.

Status: Done.

- ChatGPT built the task. The main session read the diff and ran `tools/gate.sh`.
- The main session made these corrections:
    - The repair request keeps `kind` as the finding kind and gets an optional `repair` for the button. Before, `kind` was a text that could be the finding kind or the repair kind, and the route compared serialized texts.
    - `feed_reads::check_with` takes the known GUID and the read GUID. Before, a second function `check_with_guids` did this, and `check_with` gave no `guid_changed_outside` finding.
- Screenshots (`.studio-dev/v-230/o239a` and `o239b`, steps `s239a.json` and `s239b.json`): the hosted Elijah Lied file got a different GUID for the run, and then the original file came back. The tab **Issues** shows the Warning with **Put Back 83685084** and **Keep the New GUID**. After **Put Back**, the row goes, and Review shows the change back to the known GUID with no Stop panel. After **Keep the New GUID**, the row goes, and Review shows no change.

## Problem

- Task 238 gives each feed a known GUID (`feeds.known_guid`). A read does not change it.
- When the hosted file gets a different GUID outside Studio, the tab **Issues** shows nothing. The user sees the change only in the Stop panel of Review.
- A finding has one repair (`Finding.repair`). Item 6 needs two buttons.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Check facts | extend | crates/core/src/feed_check.rs | `Facts` gets the known GUID and the GUID of the last read. `check` gives the finding. |
| Finding | extend | crates/core/src/feed_check.rs | `Finding` gets a second repair and a short GUID for the label. |
| Check inputs | extend | crates/core/src/pipeline/check_repair.rs | `check_inputs` reads the feed row. |
| Facts builder | extend | crates/core/src/feed_reads.rs | `facts_for` and `check_with` give the new facts. |
| Check repair | extend | crates/core/src/pipeline/check_repair.rs | `apply_check_repair`, `repair_step`, `plan_of` and `Plan` get the two repairs. |
| Known GUID store | reuse | crates/core/src/collection.rs | `Collection::set_known_guid` from task 238. |
| GUID comparison | reuse | crates/core/src/pipeline.rs | `has_placeholder_guid` and `guid_change` from task 238. |
| Finding rows | extend | crates/api/ui/feedcheck.js | `checkRows`, `REPAIRS` and `ACTIONS` give the buttons. |

## Changes

1. Read ADR 0249 items 1, 6 and 7, ADR 0174 and task 238 first. Then read `Finding`, `FindingKind`, `RepairKind`, `Facts` and `check` in `crates/core/src/feed_check.rs`, all of `crates/core/src/pipeline/check_repair.rs`, `facts_for` and `check_with` in `crates/core/src/feed_reads.rs`, `crates/api/src/feed_check.rs`, and `checkRows`, `REPAIRS` and `ACTIONS` in `crates/api/ui/feedcheck.js`. Read the task 237 finding `link_guid_old` as the model of a new finding.
2. Core, facts. Add `known_guid: Option<String>` and `read_guid: Option<String>` to `Facts`. `check_inputs` fills them from `FeedRow::known_guid` and `FeedRow::podcast_guid` (the GUID of the last read). Pass them through `facts_for` and `check_with`. Change each caller and each test that builds `Facts`.
3. Core, the finding of item 6. Add `FindingKind::GuidChangedOutside` (serialized `guid_changed_outside`). `check` gives one finding when each condition is true:
    - `known_guid` and `read_guid` are present, and `read_guid` is not a placeholder (`has_placeholder_guid`).
    - `read_guid` is not equal to `known_guid`.
    - The `podcast:guid` of the checked feed is not equal to `known_guid`. (After **Put Back**, the draft has the known GUID, and the finding goes.)

    The finding:
    - level: Warning.
    - title: "The GUID changed outside Studio".
    - reason: "Studio knows this feed as {known}. The hosted file now has {new}. Payments from other music feeds that name {known} stop. A second change has the same effect for feeds that name {new}." Each GUID is in full.
    - repair: `RepairKind::PutBackGuid` (serialized `put_back_guid`).
    - second repair: `RepairKind::KeepNewGuid` (serialized `keep_new_guid`). Add `second_repair: Option<RepairKind>` to `Finding`, serialized only when present.
    - Add `known_short: Option<String>` to `Finding`, serialized only when present. It is the first 8 characters of the known GUID. A read-only feed gets no repair and no second repair, as the other findings.
4. Core, the repairs.
    - `PutBackGuid`: `repair_step` gives `Step::Nothing`. `plan_of` gives a new `Plan::FeedGuid { guid }` with the known GUID of the row. If the draft has the known GUID already, give the error "this repair does not apply to the draft". `Plan::apply` sets `feed.podcast_guid`. The change goes through `changed_draft`, as the other repairs, so it shows in Review.
    - `KeepNewGuid`: it changes no draft. `apply_check_repair` calls `Collection::set_known_guid(id, read_guid)` and gives the current draft unchanged. If `read_guid` is none, a placeholder or equal to the known GUID, give the error "this repair does not apply to the draft". `plan_of` and `Plan` do not get this kind. `apply_check_repair` takes `&Collection` today. `set_known_guid` takes `&self`, so no signature change is necessary.
    - Give `plan_of` the row data that it needs. Do not read the row two times.
5. UI. In `crates/api/ui/feedcheck.js`:
    - Add `put_back_guid` and `keep_new_guid` to `ACTIONS` as `repair`.
    - The label of `put_back_guid` is "Put Back {known_short}". The label of `keep_new_guid` is "Keep the New GUID".
    - `checkRows` gives a second button for a finding with `second_repair`. The repair body of the second button has `kind` equal to the second repair. Keep the body of the other repairs: `kind`, `entry`, `url`.
    - The view of the tab **Issues** in `crates/api/ui/editor.js` draws the second button next to the first. Find the place that draws the button of `checkRows` and extend it. Do not copy the row.
6. Tests:
    - Rust, `crates/core/src/feed_check.rs` tests: facts with known `A`, read `B` and a feed with `B` give one `guid_changed_outside` Warning with the title, the reason, the two repairs and `known_short` of step 3. A feed with `A` gives no finding. Equal known and read GUIDs give no finding. A read placeholder gives no finding. No known GUID gives no finding.
    - Rust, `crates/core/tests/feed_check_repair.rs`: a hosted file with a different GUID, then a read. The check gives the finding. **Put Back** sets the draft GUID to the known GUID, and `pipeline::guid_change` then gives `None`. A second **Put Back** gives the error. In a second test, **Keep the New GUID** sets the known GUID to the read GUID, the finding goes, and `guid_change` gives `None`. The draft does not change.
    - Node, `crates/api/ui-tests/feedcheck.test.mjs`: a `guid_changed_outside` finding with `known_short` "83685084" gives the buttons "Put Back 83685084" and "Keep the New GUID", each with the action `repair` and its own `kind` in the body.
    - Each failure message names ADR 0249 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0249 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not compare GUIDs and does not cut a GUID. It shows what the core gives.
- Do not change the guard of task 238 at the write.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 6 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- After an outside GUID change, the tab **Issues** shows the Warning with **Put Back {8 characters}** and **Keep the New GUID**.
- After **Put Back**, the row goes, and Review shows no Stop panel.
- After **Keep the New GUID**, the row goes.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
