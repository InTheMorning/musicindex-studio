# ADR 0136: One Announcement Choice At The Write

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Changes: [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) items 5, 6, 11 and 12, and [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) item 14
- Source: [findings of local use, finding 7](../reviews/2026-11-06-local-use-findings.md)

## Context

A user who saves frequently also sends many podpings. Studio had two settings on the Settings page, far from the Write button. The hub sends the podping, so the only other choice was a request with no podping.

## Decision

1. One check box, "Tell Podcast Index and send a podping", replaces the two settings. It is directly above the Write button of the review sheet, and on the review page of a change set.
2. Studio remembers the last choice for the next write. It starts on. The Settings page has no announcement section.
3. A request always has `podping=1`.
4. A change of the check box saves the choice and gets the diff again, so the user does not approve a second time. The planned announcements stay a part of the diff ID.
5. The check box does not show when the feed has no public address. The sheet then gives that reason.
6. A write under a show grant always announces, because a live show needs it, and it has no check box.
7. One check box applies to each feed of a change set.

## Alternatives Considered

- A choice for one write only, outside the diff. Not chosen: the diff must list each planned announcement (ADR 0052 item 11).
- A start state that is always on. Not chosen: the operator selected the last choice, for a user who saves frequently.
- One check box for each feed of a change set. Not chosen: the page writes the feeds with one button.

## Consequences

- A user with the podping setting off now sends a podping with each announcement.
- A write with the check box off tells no index. An index can then find the change only when it reads the feed by itself.

## How This Decision Is Checked

Mechanical: `crates/core/tests/pipeline.rs` and `crates/api/src/tests.rs` examine the choice, the diff and a granted write. `crates/api/ui-tests/announce.test.mjs` and `crates/api/ui-tests/pages.test.mjs` examine the lines and the sections.

Visual: the planning model looks at the review sheet and the review page of a change set in the two themes and at 390 px.
