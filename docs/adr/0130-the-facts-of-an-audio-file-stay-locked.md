# ADR 0130: The Facts Of An Audio File Stay Locked Behind Advanced

- Status: Accepted
- Date: 2026-11-05
- Owner: operator
- Accepted: 2026-11-05 by the operator, through the questionnaire. The operator selected the warning sheet, the boxes after "Advanced" also for a missing value, and each audio file.
- Changes: [ADR 0062](0062-a-file-picker-for-each-file-field.md) items 24 and 25, [ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) item 10, [ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) item 14 and [ADR 0094](0094-a-duration-is-edited-as-minutes-and-seconds.md) item 3.
- Uses: musicindex ADR 0023 R-CONFIRM-1 for the sheet. [ADR 0065](0065-the-core-applies-each-feed-rule.md): the lock is an edit aid, and the core keeps each feed rule.

## Context

The size, the type and the duration of an audio file come from the file. Studio reads them when the user adds or selects a file. The track page showed them in boxes, so a person could change a correct value by accident. The operator asked for the duration in "Properties", and for a lock that explains the risk.

## Decision

1. The summary of "Properties" gives the size, the type and the duration, for example "3.8 MB · MP3 · 2:57". A file with no duration field gives the size and the type.
2. An open "Properties" shows each value as text, then the button "Advanced". A value that is empty shows "Missing".
3. A value that is missing or not correct gets a notice that names it and points to "Advanced". The boxes do not show before "Advanced", also for a missing value. The operator selected one rule for each value.
4. "Properties" opens by itself as ADR 0062 item 25 gives, and also after a read of the duration that failed.
5. "Advanced" opens a confirmation sheet that says why a value by hand is a risk. "Edit Values" shows the boxes. The boxes stay until the user leaves the page.
6. The track page, the recording of a live show and the sheet "Add a track from an address" use this one component. The rows of alternate files are read-only already.
7. A selected file still sets its values with no warning (ADR 0062).

## Alternatives Considered

- A box with no warning for a missing value. Not chosen by the operator.
- The warning as text in the row. Not chosen by the operator: a person can skip it.
- The lock on tracks only. Not chosen by the operator: the same values stay open in other locations.

## Consequences

- A track from an address with no facts from its file host needs "Advanced" before "Add the Track".
- A feed value that is wrong stays visible as text.

## Acceptance Criteria

Mechanical:

- A Node test: the summary has the duration of a file with a duration, and two parts for a file with none.
- A Node test: the rows show each value or "Missing", and the source text of a duration that the core cannot read.
- A Node test: the notice names each value and points to "Advanced".

Visual, by the planning model: a track shows closed, open, the warning sheet and the unlocked boxes. The check uses 1000 px, and 390 px with touch. A track with a missing size opens by itself.

Human, by the operator: none beyond the human test of the next candidate.
