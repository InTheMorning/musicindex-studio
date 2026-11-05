# ADR 0129: A Draft Takes A Change Of The Hosted File

- Status: Accepted
- Date: 2026-11-05
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-05 by the operator, through the questionnaire. The operator selected the merge over a warning.
- Source: finding 8 of the [human test of `0.1.0-rc.17`](../reviews/2026-11-05-rc17-human-test-findings.md).
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the conflict check and the diff. [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for the writer. [ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md) for the entry keys. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the merge in the core.

## Context

When the hosted file changes after a draft starts, the conflict check stops the write. "Start Again from the Hosted File" then removed each edit of the draft, also for one small outside change.

## Decision

1. The button "Apply My Edits to the New File" replaces "Start Again from the Hosted File". The core compares the base of the draft, the draft and the hosted file. An edit of the draft stays. A change of the hosted file comes in. A value that both sides changed to the same value is no conflict.
2. Review shows the plan of the core before the draft changes: the edits that stay, the new changes of the hosted file, and each field that both sides changed.
3. After the merge, the hosted file is the base of the draft. The user reviews the new diff and writes it (ADR 0007). The merge writes nothing to a host.
4. A field is one value of the channel or one value of a track. The core matches a track by its GUID. Each other list is one value, and a change of one entry is a change of that list. A track that one side removes and the other side changes is a conflict of the whole track. When a track has no GUID, or two tracks have one GUID, the list of tracks is one value.
5. Each field that both sides changed needs a choice of the user, in a select menu with no start value. Without a choice, the core refuses the merge and the draft does not change.
6. A plan names the version of the hosted file. When the hosted file changed again, the core refuses the merge, and Review shows the conflict again.
7. The merged draft has the build date of the hosted file, and the write sets a new date (ADR 0127). Each entry that the hosted file also has keeps its bytes. A feed move, a host operation and a show grant of the draft stay.
8. "Discard the Draft" stays the way to remove all edits.

## Alternatives Considered

- A warning that lists the lost edits. Not chosen by the operator: a small outside change still loses each edit.
- A merge of the lines of the XML. Rejected: a merge of text can make a file that is not a feed, and it cannot name a field.
- A merge of each entry of each list. Rejected for now: an entry of those lists has no identity, only its position.

## Consequences

- A change of one credit and an outside change of a different credit are a conflict of the list "Credits". The user selects one list.
- An edit of the sequence of the tracks keeps the sequence of the draft. Tracks that the hosted file adds go after it.

## Invariants

- The merge changes only the draft.
- The feed GUID and each item GUID do not change (ADR 0008).
- The UI calculates no merge (ADR 0065).

## Acceptance Criteria

Mechanical:

- A core test: an edit stays, an outside change comes in, the hosted file is the base, and the write passes.
- A core test: a field that both sides changed needs a choice, a refused merge changes nothing, and each side of a choice applies.
- A core test: a plan of an earlier version does not apply.
- A core test: tracks merge by GUID, and each side can add a track or remove a track.
- A Node test: the model gives the words of each row, and the button waits for each choice.

Visual, by the planning model: the plan and the select menus show in Review at 1280 px and at 390 px with touch.

Human, by the operator: change a real hosted file outside Studio during a draft, apply the edits, and write.
