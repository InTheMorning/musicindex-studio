# Studio v1 Task 121: A Write Gives Only Real Changes Of A Credit

Decision: [ADR 0185](../adr/0185-a-write-gives-only-real-changes-of-a-credit.md).

Status: Complete on 2026-11-12. The four commit checks are green. In 121a, a credit entry finds its hosted tag by its values, then by its name, so a credit above it does not change the pair. In 121b, the write leaves out a credit with no name and no Nostr key. The screenshot visual check passed at 1280 px and at 390 px, in the light and dark themes. The empty field shows "A credit needs a name.", and the credit keeps its name.

## Goal

An open credit refuses an empty name. A write leaves out a credit with no name and no Nostr key. A write keeps the attribute order of a hosted tag, so a value that comes back gives no changed line.

## Constraints

- The write rule is in the core (ADR 0065). The field model gives the field error only.
- Do not change the bytes of a tag with no changed value (ADR 0185 item 4).
- Keep the merge panel of ADR 0129 and the fill of ADR 0169 item 5.

## Files To Read First

- ADR 0185.
- `write_list` in `crates/core/src/feed/sync.rs`.
- `Feed::credit_places` in `crates/core/src/feed.rs`.
- `crates/core/src/review.rs` for the removed line.
- `crates/api/ui/credits.js` and the open credit in `crates/api/ui/editor.js`.

## Packets

### 121a: The Order Of The Attributes

ADR 0185 items 4 to 6.

- When each value of an entry is the same as in the hosted tag, `write_list` gives the hosted tag with no change.
- A changed attribute, and a value that comes back after a clear, gets its place in the hosted tag. Only a new attribute goes at the end.
- Add the two Rust tests of the ADR for the order.

### 121b: A Credit With No Name

ADR 0185 items 1 to 3.

- The write leaves out each credit with no name and no Nostr key. Review shows it as a removed line.
- The field "Name" of an open credit refuses an empty value with "A credit needs a name." The credit keeps its name from before.
- Add the Rust test and the Node test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0185.

Visual, by the planning model:

- The field error at an open credit, at 1280 px and at 390 px, in the light and dark themes.

Human: none (ADR 0185).

## Stop And Ask When

- The order rule changes the bytes of a round-trip test of the corpus.
