# ADR 0173: A Live Show Problem Shows At Its Field

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Accepted: 2026-11-11 by the operator, through the questionnaire of task 100. The operator selected "At its own field".
- Technical choices: items 2 and 3 are choices of the planning model.
- Changes: [ADR 0161](0161-a-live-item-needs-a-stream-and-a-content-link-is-recommended.md) item 6, for the position of a problem on the page. The texts, the levels and the colors of item 6 stay.
- Evidence: finding 3 of the [rc.21 findings](../reviews/2026-11-10-rc21-human-test-findings.md), and packet 100c of [task 100](../tasks/studio-v1-task-100-stream-type-fixes.md).

## Context

The live show page showed each problem of its show in the Status section, above the fields. A paste in the stream address field removed the line "This show has no stream address." when the draft saved. A delete showed it again. The page above the field changed by 35 px at 1100 px and by 43 px at 390 px. Chromium keeps the field at the same position with scroll anchoring. WebKit possibly does not.

## Decision

1. The live show page shows each problem of the core at the end of the group of the field that the problem names. A line that comes or goes then does not move a field above it.
2. The positions are these:
    - `start` and `end`: at the end of the Show group.
    - `stream`: at the end of the Stream group, below the Check Stream row.
    - `contentLink`: at the end of the Content links group.
    - `status`, and each field with no position on the page: in the Status section.
3. When the show has a recording, the page has no Stream section. A `stream` problem then shows in the Status section.

## How This Decision Is Checked

Mechanical: a Node test gives the position of each field of item 2. It gives the Status section for an unknown field. It also gives the Status section for a `stream` problem of a show with a recording.

Visual, by the agent: a paste in the stream address field, then a delete. The position of the field on the screen is the same before and after, with scroll anchoring off. The screenshots use the light and the dark theme, at 1100 px and at 390 px with touch.

Human, by the operator: a paste and a delete in the stream address field on the iPhone.

## Alternatives Considered

- Keep the line in the Status section, and keep its space when there is no problem. The Status section then has an empty line when the show is correct.
- No change, and a test on the iPhone first.

## Consequences

- A user sees each problem near the field to correct.
- The Status section shows only the status and the problems of the status.
