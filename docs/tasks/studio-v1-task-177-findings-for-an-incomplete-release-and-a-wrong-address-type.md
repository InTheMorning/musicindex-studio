# Studio v1 Task 177: Findings For An Incomplete Release And A Wrong Address Type

Decision: [ADR 0219](../adr/0219-a-release-checklist-and-a-check-of-the-address-type.md) items 1 to 5 and 9. [ADR 0174](../adr/0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) items 1 to 7.

Status: Open.

## Problem

The check of a feed has no finding for a release with no tracks, no cover, a track with no audio file or no payment recipient. It also has no finding for a recipient address in the format of the other type. Task 179 shows the first four findings as a checklist on the release page, so this task must give them first.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Finding kinds and check | extend | crates/core/src/feed_check.rs | `FindingKind` near line 20, `RepairKind` near line 68, `check` near line 199, `check_item` near line 334. Add the kinds and the conditions of ADR 0219 item 1. |
| Check repair | extend | crates/core/src/pipeline/check_repair.rs | `repair_step` near line 114 makes the draft change of a repair. Add the repair of ADR 0219 item 4. |
| Address format | extend | crates/core/src/keysend.rs | `lookup_url` near line 120 accepts the LNURL address form. Add one function that gives the format of an address (LNURL, node or none), and use it in the check. Do not copy the pattern. |
| Check tab rows | extend | crates/api/ui/feedcheck.js | `REPAIRS` and `ACTIONS` near line 50, `checkRows` near line 118. Add the buttons of ADR 0219 item 1. |
| Check page buttons | extend | crates/api/ui/editor.js | The action switch near line 4972. Add the actions that open a field or a sheet (ADR 0219 item 3). |

## Changes

1. Core: the five kinds of ADR 0219 item 1, with the level Notice and the exact title and reason texts of the table. `no_audio_file` and `wrong_address_type` give one finding for each track or each address. Put the GUID of the track in `entry` for `no_audio_file`.
2. Core: item 2. `no_tracks`, `no_cover` and `no_recipient` apply only when `podcast:medium` is `music`. A release with live items and no track gives no `no_tracks`.
3. Core: the new repair kinds `add_tracks`, `add_a_cover`, `add_a_recipient`, `use_lnurl_address` and `use_node_address`. `no_audio_file` uses the kind `open_the_track` that exists.
4. Core: the repair of item 4. It changes `type` of each recipient with that address and the other type, in the feed value block and in each track value block. It changes nothing else. It goes through the draft, as the other repairs.
5. Core: item 5. An address that matches no format gives no finding.
6. UI: `REPAIRS` gets the button texts **Add Tracks**, **Add a Cover**, **Add a Recipient**, **Use LNURL Address** and **Use Node Address**. `ACTIONS` gives `repair` for the two type buttons and an action that opens a field for the three others.
7. UI: on the Check page, **Add Tracks** opens the step that adds tracks to the release. **Add a Cover** opens the artwork sheet of the feed. **Add a Recipient** opens the payment page of the feed with the form "Add a Recipient" open. Read how the release page opens these today and use the same calls.

## Files

- Change: `crates/core/src/feed_check.rs`, `crates/core/src/pipeline/check_repair.rs`, `crates/core/src/keysend.rs`, `crates/api/ui/feedcheck.js`, `crates/api/ui/editor.js`.
- Change other files only when the call of a repair needs it, for example the API route of the repairs. Report each one.
- Change tests: the tests in `crates/core/src/feed_check.rs`, `crates/core/tests/feed_check_repair.rs`, `crates/api/ui-tests/feedcheck.test.mjs`. A test that counts the findings of a fixture can change when the fixture now gives a new finding. Report each such test.

## Checks

Mechanical:

- Core: for each kind of item 1, a feed with the condition gives the finding with its title, reason and repair, and a feed without it gives none.
- Core: a feed that is not a release gives no `no_tracks`, `no_cover` or `no_recipient`.
- Core: the repair of item 4 changes the type in the feed value block and in a track value block, and the address, the split and the name do not change.
- Core: an address that matches no format gives no finding.
- UI: `checkRows` gives the button and the action of each new kind.
- `cargo test -p studio-core`, `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual, by the main session with screenshots: the Check tab of a release with the new findings, at 1280 px and 390 px.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of ADR 0219. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.
