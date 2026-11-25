# ADR 0265: A Changed Feed Names Studio As Its Generator

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25 by the operator. The operator asked that Studio write its generator tag in each feed that it makes and writes. In the questionnaire, the operator selected the text with the name and the version, and the replacement of the tag of a different tool.
- Technical choices: the main session selected items 3 to 13 and the shown word "Generator" of item 12. The operator did not receive them as a question.
- Bar: opposite choice by mistake. The writer rule of ADR 0008 and ADR 0012 tells an agent to keep the tag of the source.
- Changes: [ADR 0008](0008-feed-types-and-feed-identity.md) and [ADR 0012](0012-lossless-tree-with-a-typed-model.md), for the rule that the writer changes only what the user changed.
- Uses: [ADR 0127](0127-a-changed-feed-gets-a-new-build-date.md) items 2 to 6 and 15 to 22, for the time and the kinds of write. [ADR 0007](0007-draft-diff-write-and-announce.md) for the approved bytes. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the rule in the core.

## Context

- RSS 2.0 defines the channel element `generator` as "a string indicating the program used to generate the channel". The example of the specification is `MightyInHouse Content System v2.3`.
- At commit `1d934ea`, `crates/` has no code for `generator`. The feed model does not hold it, so the writer keeps the tag of the source. A feed from MSP 2.0 keeps `MSP 2.0 - Music Side Project Studio` after an edit in Studio. A new feed of Studio has no tag.
- The corpus in `crates/core/tests/corpus` has tags of 12 different tools, for example MSP 2.0, Wavlake, Fountain and Sovereign Feeds.
- The Stophammer parser does not read `generator`. Stophammer ADR 0103 rejected the tag as a mark of a person. A change of the tag thus has no effect on the index.
- ADR 0127 gives one exception to the writer rule: a change of a feed also sets its `lastBuildDate`. The core sets that date when it makes the diff of a draft that differs from its base.

## Decision

### The Exception

1. The writer changes only the elements that the user changed, with a second exception: a change of a feed also sets its channel `generator`. Only the core sets this element, and the user cannot edit it.
2. The text is `Music Index Studio VERSION`. `VERSION` is the version of the workspace in `Cargo.toml`, for example `Music Index Studio 0.1.0-rc.32`. One core constant gives the text.
3. The tag of a different tool gets the text of Studio. A tag of an earlier version of Studio gets the text of the current version.
4. A read and a write with no edit keep the same bytes. A write of only file operations, with feed XML that does not change, sets no tag. A feed that the user does not change keeps the tag of its source.

### The Time Of The Tag

5. A save that changes the feed gives the draft the tag of its base, as ADR 0127 item 3 does for the date. The compare of a save ignores `generator` and `lastBuildDate`.
6. The core sets the tag when it makes the diff of a draft that differs from its base in more than `generator` and `lastBuildDate`. The tag is thus in the approved bytes.
7. The tag needs no clock and no server. The core sets it also when no server answers and the date stays (ADR 0127 item 8).
8. A second diff with no other change keeps the tag, so the diff ID stays the same. The revision of the editor ignores the tag, as it ignores the date (ADR 0127 item 5).
9. An edit that the user undoes gives the bytes of the base. The diff then has no tag line.
10. A changed tag stays in the position of its element. A feed with no such element gets one in the position of ADR 0012 item 3.

### Each Type Of Write

11. Each type of write of ADR 0127 items 17 to 22 sets the tag when it changes the feed: a roll back, a new feed, a copy, a repair, a change set, a feed move, a granted write and a read-only feed. A granted write sets the tag in its step 3. The check of a grant ignores `generator`, as it ignores `lastBuildDate`.

### The Review

12. The diff shows the tag as each other change. The field view of ADR 0114 names it "Generator". The user has no control for this element in Review.
13. The merge of a hosted file ignores the tag, as it ignores the date (ADR 0129). The next diff sets it.

## Alternatives Considered

- The name only, `Music Index Studio`. Not chosen by the operator: a reader cannot see which release of Studio wrote the feed.
- The name, the version and the address of the site. Not chosen by the operator: the text is longer and gives no more help.
- Keep the tag of a different tool, and add a tag only to a feed with no tag. Not chosen by the operator: Studio made the new file, so the tag must name Studio.
- Set the tag at each save. Rejected: the editor saves also with no edit, and an undo must give the bytes of the base.
- Set the tag in the write. Rejected: the bytes on the host then differ from the approved diff (ADR 0007).

## Consequences

- Each diff of a changed feed has one more line, also when no server answers.
- A new version of Studio changes the line again at the next change of a feed. A feed with no change keeps the tag of the version that wrote it.
- The feed model holds one more field. The model that the UI sends cannot change it.
- The shown word "Generator" is not in `../musicindex/docs/glossary.md`. That repository owns the glossary (musicindex ADR 0087), so the orchestrator session adds the row.

## Invariants

- A read and a write with no edit give the same bytes.
- The bytes that the host gets are the bytes of the approved diff, or of a diff that a grant covers.
- Each feed that Studio changes and writes names Studio in its channel `generator`.

## Non-goals

- A tag on a feed that the user does not change.
- A `docs` element or a different channel element of RSS 2.0.
- A read of the tag for a decision. No rule of Studio reads the tag of a different tool.

## Acceptance Criteria

Mechanical:

- A core test: a read and a write with no edit give the same bytes, and a save with no edit gives no tag.
- A core test: a diff of a changed draft gives the text of item 2 in the position of the source tag. It does this also with no server answer. A second diff gives the same diff ID.
- A core test: a feed with no tag gets one element, in the position of ADR 0012 item 3.
- A core test: after a diff, the next save of the editor with its revision passes.
- A core test: an edit that the user undoes gives the bytes of the base.
- A core test: a model from the UI with a different tag does not change the tag.
- A core test: a granted write sets the tag and passes the check of its grant.
- A core test: the merge of a hosted file ignores the tag.
- A Node test: the field view has the text "Generator".

Visual, by the planning model:

- Review shows the tag line of a changed feed at 1280 px.

Human, by the operator:

- After a write to a real file host, the hosted file has the tag of item 2.
