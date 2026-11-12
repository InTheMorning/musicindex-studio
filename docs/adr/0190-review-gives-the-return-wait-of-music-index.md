# ADR 0190: Review Gives The Return Wait Of Music Index

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-12 by the operator, through the questionnaire, after the human check of ADR 0128. The operator selected "Ask Music Index": Studio reads the end of the wait from the index.
- Technical choices: items 3 to 7 are choices of the planning model.
- Changes: [ADR 0128](0128-a-moved-feed-names-its-own-address.md) items 7, 8 and 10. The warning now gives the time from the index.
- Uses: Stophammer ADR 0097 (commit `96c4141` in `../stophammer`, not released on 2026-11-12) for the field `return_wait_until`. Stophammer ADR 0083 and ADR 0096 for the wait. [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) for the queue, and [ADR 0189](0189-a-move-sends-one-podping-of-the-old-address.md) for the podping of a move.

## Context

In the human check of 2026-11-12 (task 091), a second return of the test album got a wait of 1 hour from Stophammer ADR 0083 section 5. The Review warning of ADR 0128 gave no time, and it said "you do not need to do more". After the wait, the index kept the earlier address, because no new podping came.

Stophammer ADR 0097 answers the Studio request:

- The waits are shorter: the first return is free, then 1 minute, 2 minutes, 4 minutes and so on, to a limit of 1 hour. The count starts again after 1 day. Server variables can change these values.
- The index does not read the feed again when a wait ends. Studio sends a podping after the wait.
- `GET /v1/feeds/{guid}` gives `return_wait_until` in `data`: an integer of Unix seconds, or `null` when no wait is active. Before that time, the index refuses a move back to a URL that it left. Only the primary node keeps this state. A community node always gives `null`. A server before ADR 0097 gives no key.

## Decision

### The Warning

1. When Review shows the return warning of ADR 0128, Studio reads `return_wait_until` of the feed from the Music Index API of the workspace.
2. With a time in the future, the warning says: "Music Index waits until {time} before it follows this return. Studio sends the podping after that time." The time is local, as in the notice of ADR 0184.
3. With `null`, or a time that is not in the future, the warning says: "Music Index follows this return at once. Each return in one day makes the wait for the next return longer."
4. With no key, or when the read fails, the warning says: "Music Index can wait before it follows a return, and the wait grows with each return. After the wait, send the podping again." The text does not say "you do not need to do more".
5. The warning gives no table of waits. Stophammer owns the values, and a server variable can change them.

### The Podping After The Wait

6. The write of a return reads the time again. When the time is in the future, the podping of ADR 0189 item 1 gets a different send time: `return_wait_until` plus 30 seconds. The notice of ADR 0184 shows the line as waiting, with that time (ADR 0189 item 7).
7. When that time is more than 23 hours after the write, Studio does not move the send time. ADR 0052 item 18 would mark the line as failed. The warning of item 2 then says: "Send the podping again after that time."
8. The read of `return_wait_until` is not part of the diff ID. A change of the time after the diff does not make the diff old.

## Alternatives Considered

- Studio counts its own returns and calculates the wait. Not chosen by the operator: a move by a different tool is not in the count, and a server variable can change the values.
- Only a clear text with the table of waits. Not chosen by the operator.

## Consequences

- Until the release of Stophammer ADR 0097, the index gives no key, and the warning uses item 4.
- A community node gives `null`. When the Music Index API of a workspace is a community node, the warning of item 3 can be wrong. Studio uses the API of the workspace as the only source.

## Acceptance Criteria

Mechanical:

- A core or API test: a time in the future, `null`, a time in the past, no key and a failed read each give the warning of items 2, 3, 3, 4 and 4.
- A test: a return write with a time in the future queues the podping at that time plus 30 seconds. A time more than 23 hours after the write keeps the usual send time.
- A test: a change of the time does not change the diff ID.

Visual, by the planning model:

- Review with the warnings of items 2, 3 and 4, at 1280 px in the light theme.

Human, by the operator, after a release and a deploy of Stophammer ADR 0097:

- A return in the wait gives the time of the index in Review. After that time, the notice line changes to "Sent" with no click, and the index follows the return.
