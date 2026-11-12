# ADR 0185: A Write Gives Only Real Changes Of A Credit

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire. The operator selected a refusal at the field and a write that leaves out a credit with no name.
- Technical choices: items 4 to 6 are choices of the planning model. The operator saw them in the explanation before the questionnaire.
- Uses: [ADR 0129](0129-a-draft-takes-a-change-of-the-hosted-file.md) for the merge panel. [ADR 0169](0169-an-open-credit-can-find-on-music-index.md) item 5 for the fill.
- Source: findings 16 and 20 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, the draft of the artist feed "HeyCitizen" had an empty credit. The credit had no name, no role, no group, no picture, no address and no Nostr key. The core writes such a credit as an empty `<podcast:person/>` tag. The save of a draft does not refuse it. The sheet "Add a Credit" refuses an empty name, but the field "Name" of an open credit accepts an empty value.

In the test of task 104, the operator cleared the picture address of a credit. **Find on Music Index** then filled the same address again. Review showed the line of that credit as changed, because `img` moved from the first place to the last place. The core sets each value on a copy of the hosted tag (`write_list` in `crates/core/src/feed/sync.rs`). A cleared attribute goes, and a value that comes back goes at the end.

## Decision

### A Credit With No Name

1. The field "Name" of an open credit refuses an empty value. It shows "A credit needs a name." at the field, and the credit keeps its name from before.
2. A write leaves out each credit with no name and no Nostr key. This applies to a credit that a draft holds now. `Feed::credit_places` already treats such a credit as no credit.
3. Review shows a credit that the write leaves out as a removed line.

### The Order Of The Attributes

4. When each value of an entry is the same as in the hosted tag, the write gives the hosted tag with no change.
5. A changed attribute keeps its place in the hosted tag. A value that comes back after a clear also gets its place in the hosted tag.
6. Only an attribute that the hosted tag does not have goes at the end.

## Alternatives Considered

- Refuse at the field only. Not chosen by the operator: a draft that holds an empty credit now then writes an empty tag.
- Leave out at the write only. Not chosen by the operator: the user then gets no sign at the field.

## Consequences

- A draft with no changed value gives no changed line and no new `lastBuildDate`.
- The empty credit in the draft of the artist feed "HeyCitizen" goes at the next write.

## Acceptance Criteria

Mechanical:

- A Node test: the field model of an open credit refuses an empty name and keeps the name from before.
- A Rust test: a write of a draft with a credit with no name and no Nostr key gives no `<podcast:person>` tag for it.
- A Rust test: a draft that clears and then fills the same picture address of a credit gives the hosted bytes of that tag.
- A Rust test: a changed attribute keeps its place, and a new attribute goes at the end.

Visual, by the planning model:

- The field error at an open credit, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- None. The tests and the screenshots prove the change.
