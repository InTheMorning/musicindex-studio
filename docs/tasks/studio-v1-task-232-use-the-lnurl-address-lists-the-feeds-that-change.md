# Studio v1 Task 232: Use The LNURL Address Lists The Feeds That Change

Decision: [ADR 0243](../adr/0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) item 16. Finding 6 of [the tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md).

Status: Done. The main session corrected the doc comment of `has_node_recipient`, so that it states the `lnaddress` exception.

## Problem

The sheet of Use the LNURL Address of a wallet selects each feed that uses the wallet. A feed where each recipient of the wallet is already an `lnaddress` recipient does not change (outcome `no_node_recipient`). The sheet counts it in **Review in {N} Feeds**, and the review has fewer feeds than the button.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Uses route | extend | crates/api/src/routes.rs | `recipient_uses` gives one row for each feed of the wallet. Each row also gives `node`. |
| Node recipient rule | extend | crates/core/src/change_set.rs | `use_lnaddress_in_block` decides `Cause::NoNodeRecipient`. Its test of a `node` recipient of the wallet becomes one public function that the route also calls. |
| Wallet sheet model | extend | crates/api/ui/recipients.js | `walletFeedRows` and `walletSheet` give the rows and the button. |
| Wallet form | extend | crates/api/ui/peoplepages.js | `WalletForm` shows the note of item 16. |

## Changes

1. Read ADR 0243 item 16, ADR 0207 items 10 and 44, `use_lnaddress_in_block`, `recipient_uses`, `walletFeedRows`, `walletSheet` and `WalletForm` first.
2. In the core, make one public function that tells if a list of recipients has a `node` recipient of a wallet, by the same rule as `use_lnaddress_in_block`. `use_lnaddress_in_block` calls it, so the two places cannot disagree.
3. `recipient_uses` gives `"node": true` on a row when the channel, an item or a live item of the feed has a `node` recipient of the wallet, through the function of step 2.
4. `walletSheet('lnurl', uses)` (use the kind name that the code uses for Use the LNURL Address) lists only the rows with `node: true`. It also gives `note`: "{N} feeds pay this address already." when N rows have `node: false`, else no note. "1 feed pays this address already." for one. The button counts only the listed and selected rows. The other sheets do not change.
5. `WalletForm` shows `note` below the list of feeds.
6. Tests:
   - Rust in `crates/core/tests/change_set.rs` or the unit tests of `change_set.rs`: the function of step 2 gives true for a `node` recipient of the wallet and false for an `lnaddress` recipient of the same wallet.
   - Rust in `crates/api/src/tests.rs`: `recipients/uses` gives `node: true` for a feed with a node recipient and `node: false` for a feed with only the Lightning address of the same wallet.
   - Node in `crates/api/ui-tests/recipients.test.mjs`: the sheet of Use the LNURL Address with three rows, one with `node: false`, gives two rows, the button **Review in 2 Feeds**, and the note "1 feed pays this address already." The sheet of Change Address with the same rows gives three rows and no note.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0243 item 16. A test failure message names ADR 0243 item 16 and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI reads `node` and does not examine the recipients itself.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The tests of step 6 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with a screenshot): the sheet of Use the LNURL Address of Middle Season on the tutorial test data shows 6 feeds, **Review in 6 Feeds** and "2 feeds pay this address already."

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
