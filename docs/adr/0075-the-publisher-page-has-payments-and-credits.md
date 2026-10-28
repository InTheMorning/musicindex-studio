# ADR 0075: The Publisher Page Has Payments And Credits

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0041](0041-publisher-feeds-and-two-way-links.md) Non-Goals, for the value block.

## Context

- MSP 2.0 edits the value block and the support links of a publisher feed (`PublisherValueSection.tsx`, `PublisherFundingSection.tsx`).
- ADR 0041 lists "A value block in a publisher feed" as a non-goal.
- The Studio model has `values`, `fundings`, `persons` and `license` on each channel (`crates/core/src/feed.rs`).
- The album page has the section `details` with the rows "Value routes" and "Credits and links" (`crates/api/ui/pages.js`, `crates/api/ui/editor.js`). "Value routes" opens `RecipientsPage`. "Credits and links" opens `CreditsPage`, with the people, the support links and the license.
- The publisher page has the sections `channel`, `releases`, `role` and `feed`. It has no section for these fields.

The operator selected this decision from a questionnaire on 2026-10-28.

## Decision

1. The publisher page gets the section `details`, with the tier `details`, and the rows "Value routes" and "Credits and links". It is after the section `releases` and before the section `role` ([ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md)).
2. The rows open the pages of the album: `RecipientsPage` and `CreditsPage`. The pages do not change.
3. The recipient picker of the album offers the same recipients on a publisher feed ([ADR 0027](0027-the-recipient-picker.md)).
4. Studio does not copy the publisher recipients into its releases, and does not copy the release recipients into the publisher feed.

## Alternatives Considered

- Payments and support links only, with no people and no license. Not selected: it needs a different page for the publisher feed, and the operator selected the pages of the album.
- Keep the non-goal of ADR 0041. Not selected: an MSP 2.0 label then cannot edit the payments of its feed.

## Consequences

- An MSP 2.0 user can edit each publisher field that MSP 2.0 edits.
- The publisher page has the same section sequence as the album page.

## Non-Goals

- `podcast:podroll` and the `feedImg` attribute stay non-goals of ADR 0041.
- The value time splits.

## Verification

Mechanical:

- A Node test: the publisher page of `pages.js` gives `channel`, `releases`, `details`, `role` and `feed`, in the sequence of their tiers.
- A core test: an edit of the publisher value block changes only that block.

Visual:

- The publisher page with the section `details`, and the two pages that its rows open. In the two themes, at desktop width and at 390 px.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.
