# ADR 0186: The Merge Notice Gives The Times Of The Change

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire. The operator selected the drawing of a neutral text with two times.
- Technical choices: items 2 and 3 are choices of the planning model.
- Uses: [ADR 0129](0129-a-draft-takes-a-change-of-the-hosted-file.md) for the merge.
- Source: finding 7 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, Review of the artist feed "HeyCitizen" showed "The hosted file changed after the draft started. Studio did not write. A different tool or a different person changed the file." The hosted file had the `lastBuildDate` 18:46:24 of 2026-11-06. The probable cause is a write from a different Studio collection or from the rc.21 AppImage on the same file hosts. This is not confirmed. Thus the text can give a cause that is not correct.

## Decision

1. The notice of a hosted file that changed after the draft started has this text:
    - "The hosted file changed after your draft started. Studio did not write."
    - "Draft started {time}" and "File changed {time}".
    - "The change can come from a different collection, a different Studio version, a different tool or a different person."
2. The time of the change is the `lastBuildDate` of the hosted file. With no `lastBuildDate`, it is the `Last-Modified` time that the file host gives. With neither, the line "File changed" does not show.
3. The core gives the two times. The UI calculates no time. The time that the draft started is the time that the draft got the hosted file that it starts from. The collection keeps that time with the draft, and an edit does not change it. A draft from before this decision has no start time, and its line "Draft started" does not show.
4. The button **Apply My Edits to the New File** does not change.

## Alternatives Considered

- A neutral text with no times. Not chosen by the operator: the times help the user find the source of the change.
- Keep the text "A different tool or a different person changed the file." Not chosen by the operator: it can give a cause that is not correct.

## Consequences

- The user can compare the times with the writes of the other collections.

## Acceptance Criteria

Mechanical:

- A Node test: the notice model gives the text of item 1 with the two times, and no line "File changed" with no time.
- A Rust test: the core gives the `lastBuildDate` first, and the `Last-Modified` time when the file has no `lastBuildDate`.

Visual, by the planning model:

- The notice at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- None. The tests and the screenshots prove the change.
