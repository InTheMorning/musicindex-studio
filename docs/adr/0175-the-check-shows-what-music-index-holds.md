# ADR 0175: The Check Shows What Music Index Holds

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Proposed: 2026-11-11 by the planning model, on the operator answers of 2026-11-11 in the questionnaire. The operator selected F3 as the next item. The operator selected items 1, 3, 5, 6, 8 and 10. The operator selected the rows in the tab **Check**, the level Warning, and no mark outside the tab.
- Technical choices: items 2, 4, 7, 9 and 11 are choices of the planning model.
- Accepted: 2026-11-13 by the operator, at the mockup of packet 110c. The operator selected **Show the Ways** for each different address (item 5) through the questionnaire, from a drawing.
- Changed: 2026-11-14 by the operator, at the human check walk. Item 4 gives a hidden feed the line with no "yet".
- Revised: 2026-11-15 by operator decision, from the [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 11. Item 1 permits the requests of the notice after a write with a podping.
- Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F3.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0038](../archive/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md), [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md), [ADR 0065](0065-the-core-applies-each-feed-rule.md), [ADR 0126](0126-studio-refuses-a-bad-move-link.md), [ADR 0127](0127-a-changed-feed-gets-a-new-build-date.md), [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md). Stophammer ADR 0052 and ADR 0058 own the rules of the index. This record cites them and does not restate them as rules of Studio.
- Changed by: [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 8 to 10, on 2026-11-20. Item 1: the notice after a write has **Dismiss**, stays until **Dismiss** and keeps its height.
- Changed by: [ADR 0247](0247-studio-restores-a-changed-feed-guid-and-corrects-a-link-with-an-old-guid.md) items 1 to 6 and 10 to 11, on 2026-11-22. Items 4 and 11: a pending GUID change to the GUID of the draft gives the finding `guid_changed` with a repair, and no line "does not hold this feed yet".

## Context

A publisher cannot see what Music Index holds for a feed. ADR 0126, ADR 0127 and ADR 0128 name this view as a non-goal. ADR 0174 item 9 leaves the GUID difference between the local feed and the index to F3.

These facts of the code on 2026-11-11 apply:

- `musicindex.rs` reads `GET /v1/feeds/{guid}` and gives `IndexFeed` with the address and the title.
- `musicindex.rs` does not read `GET /v1/feeds/{guid}/copies` or `GET /v1/guid-changes`.
- ADR 0174 gives the check and the findings with the levels Notice and Question. It also gives the reads at open and at **Check Again**, and the facts in memory.

These facts of the Stophammer API on 2026-11-11 apply:

- `GET /v1/feeds/{guid}` gives `feed_url` and `last_build_date`.
- `GET /v1/feeds/{guid}/copies` gives each copy of a feed: a different URL that gives the same GUID (Stophammer ADR 0058). Each row has `url`, `open` and `differs_recipients`.
- Only the operator of the index can resolve a copy or decide a GUID change. Studio cannot.

## Decision

### The Reads

1. The reads of ADR 0174 item 16 also send `GET /v1/feeds/{guid}` and `GET /v1/feeds/{guid}/copies` to the Music Index API of Studio. For the check, they go out only when the user opens the check and when the user selects **Check Again**. A failed read gives the line of ADR 0174 item 16 with the address of the request.

   After a write with a podping, the feed page shows a notice with the state of the podping. For that notice, Studio also sends `GET /v1/feeds/{guid}` at intervals, until Music Index has read the write. A limit stops the requests, and **Check Again** on the notice starts them again. A write with a move shows only the notice of ADR 0184.
2. The facts of the index stay in memory with the other facts of ADR 0174 item 18. A feed with no `podcast:guid` gives no request.

### The Different Address

3. The finding `index_address_differs` has the level Notice. It shows when the `feed_url` of the index is not the public address of the feed.
4. A difference of spelling only (ADR 0174 item 12) gives no finding. When the index does not hold the feed, the check gives no finding and the line of item 10 says "Music Index does not hold this feed yet." For a draft that is hidden from apps, the line says "Music Index does not hold this feed.", because Music Index does not take a hidden feed.
5. The button of the finding is **Show the Ways**. Its sheet gives two ways: an HTTP 301 redirect at the old host, and a request to the operator of the index. This finding changes no feed, so a read-only feed shows the button too. Studio puts no pointer file at the address of the index.

### The Copy That Pays Other People

6. The finding `copy_other_recipients` has the new level Warning. It shows for each copy with `open` and `differs_recipients`. Its title is "A copy pays other people", and its reason gives the URL of the copy.
7. A row with the level Warning comes before each Notice and each Question. The level label uses the error color. The tab count includes it. No mark shows outside the tab **Check**, so the other two apps of the suite need no change.
8. The button **Show What to Do** opens a sheet. The sheet says that only the operator of Music Index can remove the copy. It gives a report with the GUID, the public address of the feed and the URL of the copy, and the button **Copy the Report**. Studio sends no report.
9. This button changes no feed, so a read-only feed shows it too. **Show What to Do** and **Show the Ways** are the two exceptions to ADR 0174 item 20.

### Facts That Give No Finding

10. A line under the rows says "Music Index has not read your last write yet." when the `last_build_date` of the index is earlier than the `lastBuildDate` of the last write of Studio (ADR 0127). The line is not a finding, and the count does not change.
11. A copy with the same recipients, a closed copy and a pending GUID change give no finding.

## Alternatives Considered

- An own section or an own tab for the index. Not chosen by the operator: the tab **Check** is the one place for each problem.
- A request only on a separate button, or a request some minutes after each write. Not chosen by the operator: item 16 of ADR 0174 already gives the time of each request.
- A row for a pending GUID change, and a row for a last write that the index has not read. Not chosen by the operator.
- The level Notice for a copy that pays other people. Not chosen by the operator: money can go to a different person.
- A red mark on the release in the list. Not chosen by the operator.
- **Point the Old Address Here**: a pointer file at the address of the index, when that address is on a connected file host. Not chosen by the operator: the draft holds one feed file and a move, and a pointer file on a different host needs a new draft element and a new write step.

## Consequences

- Studio sends two more requests to the index at each read of the check.
- The check gets a third level. The UI model of ADR 0174 gets the order and the color of item 7.
- A text of a sheet can become wrong when a rule of the index changes. A change of the Stophammer records needs an examination of these texts.

## Invariants

- Studio sends no request to the index outside the reads of ADR 0174 item 16.
- No finding of this record stops a write.
- Studio does not resolve a copy and does not decide a GUID change.

## Acceptance Criteria

Mechanical, in the tests of `studio-core`, `studio-api` and the Node tests of the UI models:

- A core test: an index address that differs gives `index_address_differs`. An equal address gives no finding. A difference of spelling only and an index with no record also give no finding.
- A core test: a copy with `open` and `differs_recipients` gives `copy_other_recipients` with the level Warning. A copy with the same recipients and a closed copy give none.
- A core test: the check of a feed with no `podcast:guid` sends no request to the index.
- An API test: the reads give the two findings from a scripted index. A failed read gives the line of item 1.
- A Node test: a Warning row comes first, the count includes it, and a read-only feed shows **Show What to Do** and **Show the Ways** and no other button.
- A Node test: the report has the GUID, the public address and the URL of the copy.
- The four commit checks.

Visual, by the planning model with `tools/drive-ui.mjs`:

- The two rows, the two sheets and the line of item 10, in Light and in Dark, at 1280 px and at 390 px.
- `$edges()` gives no result. At 390 px, `$fieldText()` and `$overflow()` give no result.

Human, by the operator. This gate stays open until it runs:

- In the AppImage, the check of a real feed shows what Music Index holds, and **Copy the Report** gives the report on the clipboard.
