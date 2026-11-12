# ADR 0187: Review Checks The Stream Type Of A Live Show

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire. The operator gave the three cases in the human checks. The operator then selected the drawing of a stop for a live stream and a warning for a stream that is not live.
- Technical choices: items 1, 5 and 7 are choices of the planning model.
- Changes: [ADR 0162](0162-studio-fills-and-checks-the-stream-of-a-live-show.md) item 8, a write does not check the stream. Item 9, the warning shows only at the field.
- Uses: [ADR 0162](0162-studio-fills-and-checks-the-stream-of-a-live-show.md) items 2, 3 and 10 for the check and the labels. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the decision in the core.
- Source: finding 18 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

The warning of ADR 0162 item 9 shows only at the field of the live show. It does not stop a write, and Review does not show it. Thus a user can write a feed with a stream type that the apps cannot play.

## Decision

1. Review checks the stream of each live item that the draft adds or changes, with the check of ADR 0162 item 2. The checks run when Review opens.
2. The stream can be live with a server type that does not agree with the stream type. Then Review shows an error at that item: "The feed gives {label}, but the stream server sends {server label}. Apps cannot play this stream." The write is not available.
3. The error has the button **Use {server label}**. It sets the stream type of the draft, and Review shows the new diff.
4. When the stream is not live, or the check gets no answer, Review shows a warning at that item: "Studio could not check the stream: {cause}." The write is available.
5. When the stream is live and the types agree, Review shows nothing for the stream.
6. The core refuses a write with the error of item 2. Thus a write that does not come from Review gets the same refusal.
7. A granted write from a connected app gets the same check and the same refusal (ADR 0162 item 7).
8. Item 2 uses the same agreement test as ADR 0162 item 5. A different name of a menu type agrees with that type.

## Alternatives Considered

- A warning in each case, with a write that is always available. Not chosen by the operator: a user can then write a stream that the apps cannot play.

## Consequences

- Review waits for at most 8 seconds for each changed live item.
- A connected app that finds a different type can correct the feed through its grant. The check then agrees.

## Acceptance Criteria

Mechanical:

- A Rust test: with a test stream that is live and sends a different type, the write refuses with the error of item 2.
- A Rust test: with a stream that is not live, the write completes, and the diff gives the warning of item 4.
- A Rust test: a different name of a menu type agrees.
- A Node test: the Review model gives the error with **Use {server label}** and no write, and gives the warning with the write.

Visual, by the planning model:

- Review with the error and with the warning, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- With a real Icecast mount that is live, a stream type that does not agree stops the write in Review. **Use {server label}** makes the write available.
