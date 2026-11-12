# ADR 0184: After A Move Studio Shows The Podping Of The Move

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire. The operator asked for a notice that tells the user to examine the podping. The operator selected the drawing with the podping state and a send button. The button of a podping that was not sent has no word "Again".
- Technical choices: items 1, 2, 4, 6 and 9 are choices of the planning model.
- Changes: [ADR 0150](0150-studio-shows-a-change-from-outside-when-a-page-opens.md) item 5, the notice after a move by Studio. [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) item 21, a send with no write.
- Uses: [ADR 0128](0128-a-moved-feed-names-its-own-address.md) for the tag `itunes:new-feed-url`. [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) items 8 and 13 to 20 for the queue. [ADR 0136](0136-one-announcement-choice-at-the-write.md) for the choice at the write.
- Source: findings 2 and 8 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, after a move of only the media files, the **Files** page showed "The feed file names the new feed address …". The address did not change. The page shows this line each time the feed has the tag `itunes:new-feed-url`, and that tag stays after a move (ADR 0128 item 6).

After a move of the feed file, the release page showed "Music Index reads this feed from a different address." The notice ended with "A different collection or a different tool can hold this feed." Studio moved the feed some seconds before that. Music Index reads the new address after it gets the podping of the move, and the notice did not tell this.

## Decision

### The Line Of The Feed Address

1. The **Files** page shows the line of the tag `itunes:new-feed-url` only for a new value. The draft then gives the tag a value that the hosted file does not have. The text is "The feed file names the new feed address {address}."
2. When the tag of the draft is the same as the tag of the hosted file, the page shows no line. The row of the feed file in "File hosts" gives the address.

### The Notice After A Move By Studio

3. The check of ADR 0150 item 5 can give an address at Music Index that is the source address of a move. That move is the last written move of this feed in this collection. Then the release page shows the notice of a move by Studio in place of the notice of a different address.
4. The notice starts with "You moved this feed at {time}. Music Index reads the new address after it gets the podping." It has the information style, not the error style.
5. The notice then gives the state of the podping of the move write, from the announcement queue (ADR 0052 item 20):
    - "Podping: waiting, sends at {time}."
    - "Podping: sent at {time}."
    - "Podping: failed at {time}: {error}."
    - "Podping: not sent."
6. A move write announces two addresses (ADR 0052 item 8). The notice gives one line for each address, with the address first.
7. A failed podping has the button **Send the Podping Again**. A podping that was not sent has the button **Send the Podping**. A podping that waits or that is sent has no button.
8. The notice goes when the check of ADR 0150 item 5 gives the new address, or no different address.
9. An address at Music Index can be different and not the source of a move by Studio. Then the page keeps the notice of ADR 0150 item 5 and its text.

### A Send With No Write

10. The buttons of item 7 put one announcement of the current public address of the feed into the queue of ADR 0052. The rules of the queue apply, with the wait of item 14 and the tries of item 15. This changes ADR 0052 item 21 for the notice of item 3 only. No other page has a send button.
11. A feed with no public address has no button (ADR 0052 item 9).

## Alternatives Considered

- No notice until Music Index reads the new address. Not chosen by the operator: the user then cannot see a podping that failed.
- The podping state with no button. Not chosen by the operator: without a podping, Music Index reads the new address only at its next scheduled read of the feed.
- Keep the text "A different collection or a different tool can hold this feed." for each case. Not chosen by the operator: it gives a cause that is not correct after a move by Studio.

## Consequences

- After a move, the user sees if Music Index got the podping, and can send it.
- The core gives the release page the last written move of the feed, with its source address and its time.

## Acceptance Criteria

Mechanical:

- A Node test: the model of the Files page gives the line of item 1 only for a tag of the draft that differs from the hosted tag.
- A Node test: the notice model gives the notice of item 3 when the address at Music Index is the source of the last move. It gives the notice of ADR 0150 item 5 for each other address.
- A Node test: the notice model gives the four podping lines and the buttons of item 7.
- A Rust test: a send with no write puts one announcement of the public address into the queue. A feed with no public address gives a refusal.

Visual, by the planning model:

- The notice with each podping state, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- After a real move of a feed between two file hosts, the release page shows the notice and the podping state. The notice goes after Music Index reads the new address.
