# ADR 0191: A Return Warning Has The Warning Color, And A Sent Podping Can Go Again

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-12 by the operator, in the human check of ADR 0190. The operator said that the return warning did not look like a warning. The operator accepted the two proposals of the planning model, and asked for the warning at the top of Review (item 6).
- Technical choices: the form of item 1 follows musicindex ADR 0025 and the status mark of HitIt. The operator asked that the warning follow the guidelines. The text of item 4 is a choice of the planning model.
- Changes: [ADR 0184](0184-after-a-move-studio-shows-the-podping-of-the-move.md) item 7, the buttons of the podping line. [ADR 0190](0190-review-gives-the-return-wait-of-music-index.md) items 2 to 4, the style of the warning. The rule "a notice border shows the error only" in `crates/api/ui/styles.css`.
- Uses: musicindex ADR 0025 R-STATUS-1 for the Warning color. [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) for the queue. [ADR 0189](0189-a-move-sends-one-podping-of-the-old-address.md) item 4 for the address that the button sends.
- Source: the human check of task 127 on 2026-11-12.

## Context

In the human check of ADR 0190, the Review warning of a return used the plain notice: a gray fill with no border. The operator did not see it as a warning.

In the same check, the podping line of an earlier move showed "Sent" at 4:06 PM. Music Index did not follow that podping, because an earlier server refused the move. ADR 0184 item 7 gives no button to a sent line. Thus the user could not send the podping again from the notice.

A "Sent" state tells the user only that the hub took the podping. It does not tell the user that Music Index followed it.

## Decision

1. The title of a notice with a warning starts with the triangle icon (`alert`). The icon and the title have the Warning color (musicindex ADR 0025 R-STATUS-1). The icon shape and the title give the status, not the color only (R-STATUS-1 item 4). The notice keeps the notice fill and has no border (R-ELEVATION-1 item 1).
2. The return warning of ADR 0190 item 2 (a wait) and item 4 (no answer) has the warning style of item 1. The warning of ADR 0190 item 3 (Music Index follows at once) stays a plain notice.
3. While Review reads the wait, the warning has the text of ADR 0190 item 4 and the warning style.
4. A sent podping line of ADR 0184 has the button **Send the Podping Again**. The button sends as ADR 0189 item 4 gives.
5. A waiting line has no button. A feed with no public address has no button (ADR 0184 item 11).
6. Review shows the return warning at the top of its body, directly below the status line.

## Alternatives Considered

- A 1 px border in the Warning color. Removed: a color is then the only sign of the status (R-STATUS-1 item 4), and a layer has no border (R-ELEVATION-1 item 1).
- No button on a sent line. Not chosen by the operator: a podping that the hub took can give no result in Music Index.

## Consequences

- A return with a wait, or with no answer, is easy to see in Review.
- The user can send the podping of a move again at each time after the send.

## Acceptance Criteria

Mechanical:

- A Node test: the return warning gives the tone `warning` for a wait, for no answer and for no state. It gives the tone `plain` for "follows at once".
- A Node test: a sent podping line has the action **Send the Podping Again**. A waiting line has no action. With no public address, no line has an action.

Visual, by the planning model:

- Review with the warning of ADR 0190 items 2, 3 and 4, at 1280 px in the light and dark themes. The warning is directly below the status line. Only the warnings of items 2 and 4 have the triangle icon and the title in the Warning color. No warning has a border.
- The move notice with a sent line and its button, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate passed on 2026-11-12, in a release build of `1ca71ba`. Review gave the wait until 9:15:37 PM with the icon and the colored title, and the podping line gave "Waiting · 9:16:07 PM":

- In the release build, the return warning with a wait is easy to see as a warning.
