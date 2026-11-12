# ADR 0189: A Move Sends One Podping Of The Old Address

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire, in the human check of ADR 0184. The operator selected one podping of the address before the move.
- Technical choices: items 2, 5 and 6 are choices of the planning model. Item 6 corrects a defect of the human check of 2026-11-12.
- Changes: [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) item 8, the two addresses of a move write. [ADR 0184](0184-after-a-move-studio-shows-the-podping-of-the-move.md) item 6, one line for each address, and item 10, the address that the button sends.
- Uses: [ADR 0128](0128-a-moved-feed-names-its-own-address.md) for the tag `itunes:new-feed-url`. Stophammer ADR 0052 for the move of a record in Music Index.

## Context

ADR 0052 item 8 makes a move write announce two addresses: the new address, then the address before the move. It gives this cause: an index that knows only the address before the move reads the redirect or the `itunes:new-feed-url` file. For that cause, only the address before the move is necessary.

Stophammer ADR 0052 says that "the new URL does not move a feed by its own claim". Music Index moves its record only when the body at the address before the move names the new address. A podping of the new address thus does not move the feed. An index can add the new address as a second feed. Stophammer ADR 0052 also says that the complete live algorithm of Podcast Index is not known.

In the human check of ADR 0184, the notice gave two lines. ADR 0184 item 10 made each button send the current public address, which is the new address. Thus the button of the address before the move sent a different address.

## Decision

1. A write with a move of the feed file announces one address: the address before the move.
2. A move with no address before the move announces the new address. An example is a feed that had no public address before.
3. The notice of ADR 0184 item 3 gives one podping line, for the address of item 1 or item 2, with the address first.
4. The button of ADR 0184 item 7 puts one announcement of that address into the queue of ADR 0052. The rules of the queue apply.
5. An announcement of the new address that an earlier write put in the queue stays in the queue. The notice does not show it.
6. While the line of item 3 waits, the notice loads its state again some seconds after the send time. **Read Again** also loads the notice again. Thus the line shows the sent state with no new page.

## Alternatives Considered

- Keep two podpings, and correct only the address of each button. Not chosen by the operator: the podping of the new address does not move the feed in Music Index.

## Consequences

- An index that has the new address before the move write gets no podping from it. Studio knows no such index.
- The diff of a move write lists one announcement.

## Acceptance Criteria

Mechanical:

- A Rust test: the diff and the write of a move plan one announcement, of the address before the move.
- A Rust test: a move with no address before the move plans one announcement of the new address.
- A Rust test: the last written move gives one podping line, for the address of item 1.
- A Node test: a waiting line gives the time of the next load of the notice, after the send time. A line that does not wait gives no time.
- An API test: the send of ADR 0184 item 10 after a move puts one announcement of the address before the move into the queue.

Visual, by the planning model:

- The notice with one line, at 1280 px, in the light theme.

Human, by the operator. This gate stays open until it runs:

- After a real move of a feed between two file hosts, the notice gives one line, for the address before the move. **Send the Podping** sends that address, and the line shows "sent".
