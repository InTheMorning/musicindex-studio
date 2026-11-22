# Studio v1 Task 233: The Short Address Finding And Its Repair

Decision: [ADR 0245](../adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md) items 1 to 4 and 8. Findings 10 and 11 of [the tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md).

Status: Open.

## Problem

A credit with the information address `www.doerfelverse.com` stops the write with `bad_address`, but the feed check gives no finding and no repair. The refusal message gives model paths such as `items/0/persons/0/href`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Address walk | extend | crates/core/src/feed/links.rs | `Feed::bad_addresses` and `Checker` already walk each address field of ADR 0023 item 1 with its model path. The finding and the setter use the same walk. |
| Web address rule | reuse | crates/core/src/feed/links.rs | `is_web_address` gives the rule of ADR 0023 item 2. |
| Feed check | extend | crates/core/src/feed_check.rs | `FindingKind`, `RepairKind`, `Finding` and `check` give the findings of ADR 0174. |
| Repair plan | extend | crates/core/src/pipeline/check_repair.rs | `repair_step` and the plan of `UseTheHttpsAddress` apply one repair to the draft. |
| Repair route | reuse | crates/api/src/feed_check.rs | `repair` applies the repair of one finding to the draft. |
| Write refusal | extend | crates/core/src/pipeline.rs | The `bad_address` refusal near line 4772 lists the model paths. |
| Check labels | extend | crates/api/ui/feedcheck.js | The button labels and the kinds of the repairs. |

## Changes

1. Read ADR 0245, ADR 0023 items 1 to 4, ADR 0174 items 6 and 7, and ADR 0224 item 1 first. Then read `Feed::bad_addresses`, `feed_check::check`, `repair_step` and the plan of `UseTheHttpsAddress`.
2. In `links.rs`, give each `BadAddress` a `place`: the words of ADR 0245 item 3. The parts are joined with " · ":
   - For a field of an item, start with "Track {n} · {title}", where n is the position of the item from 1. For a live item, use "Live item · {title}".
   - For a credit, add "Credit · {name}". For a support link, add "Support link". For an image of `podcast:image`, add "Artwork".
   - End with the label of the field. Use the label that `crates/api/ui/editor.js` shows for that field: for example "Information address", "Picture address", "Audio file address", "Website", "Cover image address", "License text address", "Link address". When two labels are possible, select the one in `editor.js`.
   - Examples: "Credit · The Doerfels · Information address", "Track 3 · Neon Diner · Audio file address", "Website".
3. In `links.rs`, add `short_address_repair(text) -> Option<String>`. It gives `https://` followed by the trimmed text when the text has no `://`, and that address passes `is_web_address`. Else it gives none. `none`, an empty host and a text with a space give none.
4. In `links.rs`, add a function that sets the text of the field of one model path in the `Feed`. It uses the same walk as `bad_addresses`, so the two cannot disagree. It gives an error for a path that does not exist.
5. In `feed_check.rs`:
   - Add `FindingKind::ShortAddress` (`short_address`) and `RepairKind::AddHttps` (`add_https`).
   - Add the field `field: Option<String>` to `Finding`, with the model path. Add `place: Option<String>`. The existing findings give none.
   - `check` gives one finding for each `BadAddress` with a `short_address_repair`. The level is `Stop`, the title is "{place} has an address that is not full", the reason is "{address} has no https://. Studio cannot write it.", `url` is the address and `repair` is `AddHttps`.
6. In `check_repair.rs`, the repair `AddHttps` needs no read (`Step::Nothing`). Its plan sets the named field to the result of `short_address_repair` with the function of step 4. It refuses when the field of the draft does not have the text of the finding any more.
7. In `pipeline.rs`, the `bad_address` refusal gives the places of step 2, not the model paths. The API error keeps the code `bad_address` and also gives each model path in its details, so that the editor can still mark the field.
8. In `feedcheck.js`, add the label `add_https: 'Add https://'` and the kind `add_https: 'repair'`, next to `use_the_https_address`.
9. Tests:
   - Unit tests in `links.rs`: `short_address_repair` for `www.doerfelverse.com`, ` doerfelverse.com/artists `, `none`, `https://`, `www.a b.com` and an empty text. The place of a credit `href` in the channel and in an item. The setter changes only its one field.
   - Rust in `crates/core/tests/feed_check_repair.rs`: a credit with `www.doerfelverse.com` gives one `short_address` finding with the level `Stop`, the place "Credit · The Doerfels · Information address" and the repair `add_https`. The repair gives `https://www.doerfelverse.com`, and `bad_addresses` is then empty. A second credit with a correct address does not change.
   - Rust in `crates/api/src/tests.rs`: the check route gives the finding, the repair route applies it, and the write then works. The `bad_address` refusal gives the place.
   - Node in `crates/api/ui-tests/feedcheck.test.mjs`: the finding row of `short_address` has the button **Add https://** with the kind `repair`.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0245 and its item. A test failure message names ADR 0245, the item and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not examine an address itself.
- Do not add `https://` at the write. Only the repair adds it (ADR 0245 item 14).
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- If a step needs a design decision that this task does not give, stop and report it.

## Acceptance Criteria

Mechanical:

- The tests of step 9 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with a screenshot): the Check tab of "Bloodshot Lies - The Album" in the tutorial test data shows the finding with "Stops the write" and **Add https://**. After the repair, the finding goes.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
