# Studio v1 Task 050: Payments And Credits On The Publisher Page

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "MSP 2.0 Parity".
Decisions: [ADR 0075](../adr/0075-the-publisher-page-has-payments-and-credits.md), [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Open.

## Goal

The publisher page has a details section with the rows "Value routes" and "Credits and links". The rows open the pages of the album.

## Files

Inspect: `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/editor.js` (`AlbumPage` and its `details` render function, `PublisherPage` and its `shown` map, `RecipientsPage`, `AlbumPayments`, `CreditsPage`, `recipientSummary`, `creditSummary`, the page dispatch near line 3316), `crates/api/ui/shell.js` (the outline rows of a publisher feed), `docs/user-guide/artist-and-label-feeds.md`.

Change: `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/editor.js`, `docs/user-guide/artist-and-label-feeds.md`. Change `crates/api/ui/shell.js` only when the outline must show the two pages for a publisher feed, and then say so.

Do not change: the Rust crates, `RecipientsPage`, `CreditsPage`, `AlbumPayments`, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `pages.js`: the page `publisher` gives `channel`, `releases`, then `{ id: 'details', tier: 'details', rows: ['recipients', 'credits'] }`, then `role` and `feed`.
- `PublisherPage` gets a render function `details` with the heading "Details". Its two rows are the rows of the album `details` section, with the same labels, summaries and `openPage` calls. Move the two row renderers into one shared function that the two pages call. Do not copy them.
- The page dispatch opens `CreditsPage` and `RecipientsPage` for a publisher feed. The branch `publisher && page !== 'files'` must not catch the pages `credits` and `recipients`.
- When `AlbumPayments` shows a part for the tracks, that part shows nothing for a publisher feed, because a publisher feed has no tracks. When it shows a track text with no tracks, stop and report.

## Steps

1. Change `pages.js` and add the sequence of the publisher page to `pages.test.mjs`.
2. Change `editor.js`.
3. Add a section to `artist-and-label-feeds.md`: the user opens "Value routes" or "Credits and links" in the details of the feed.
4. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the changed guide lines.
5. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). Name each new function that changes the model in your report.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- `pages.test.mjs` passes, and gives the publisher sequence `channel`, `releases`, `details`, `role`, `feed`.
- The tier test of ADR 0071 passes for the publisher page.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The publisher page shows "Details" with the two rows, after the releases.
- "Value routes" opens the recipients page. A new recipient goes into the draft of the publisher feed.
- "Credits and links" opens the page with the people, the support links and the license.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: add a value route and a support link to an artist feed, examine the diff, and write it.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when a page of the album needs a change to work on a publisher feed.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0075-the-publisher-page-has-payments-and-credits.md
- docs/tasks/studio-v1-task-050-publisher-payments-and-credits.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The publisher page has the section "Details" with the rows "Value routes" and "Credits and links", which open the pages of the album.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- The Rust crates, RecipientsPage, CreditsPage, AlbumPayments, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
