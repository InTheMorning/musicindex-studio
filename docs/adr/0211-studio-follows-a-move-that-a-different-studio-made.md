# ADR 0211: Studio Follows A Move That A Different Studio Made

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire, from drawings. The operator selected "One warning with 'Follow the Move'" for the feed page. The operator selected "Follow as read-only, and offer to connect" for a move to a file host with no connection. The operator selected "Warning for a change and for a move" for the colors.
- Technical choices: the planning model selected items 2 to 5, 8 to 12, 14, 16 and 20 to 22.
- Changes: [ADR 0150](0150-studio-shows-a-change-from-outside-when-a-page-opens.md) items 2 to 4: the color of each result, and the notice of a move.
- Changes: [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) item 2: the color of the mark of an outside change.
- Uses: [ADR 0021](0021-public-address-of-a-feed.md) for the sources of a feed address. [ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md) for a read-only feed. [ADR 0129](0129-a-draft-takes-a-change-of-the-hosted-file.md) for the comparison. [ADR 0128](0128-a-moved-feed-names-its-own-address.md) and [ADR 0190](0190-review-gives-the-return-wait-of-music-index.md) for a return. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1 and musicindex ADR 0068 R-STATUS-1 for the style of a warning.
- Source: defects 3 and 4, missing function 1 and design item 8 of the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md).

## Context

On 2026-11-14 the desktop app moved a test feed from the file host `heycitizen` to the file host `bastone`. The server holds the same feed, and it did not make the move. On the server, the folder of the feed is still on `heycitizen`, where the pointer file is now.

The feed page of the server showed two plain notices: "Other sources give a different address." and "The hosted file changed outside Studio." **Read Again** did not remove them. The walk record gives the cause:

- **Read Again** reads the file in the folder of the file host. That file is the pointer file.
- The outside check reads the public address of the feed. That address is already the new address, which the pointer file names.
- Thus the two files are always different. The result is "changed", not "moved", because the new address is already the public address.

The user has no way to make this Studio use the new address. The only way is to remove the feed and to add it again, which loses the draft and the versions.

The tile of the feed said "Changed outside Studio" in blue. The left panel showed a red cloud. The notices had no status color.

## Decision

### The Result "Moved"

1. A feed has the result "moved" when the file at its folder address is a pointer file. A pointer file names a different address in `itunes:new-feed-url`.
2. The folder address is the address that the file host gives for the path of the feed (ADR 0021). The check reads it as a public address, so it needs no vault (ADR 0150 item 6).
3. The result "moved" also includes the case of ADR 0150 item 3: the file at the public address of the feed names a new address.
4. The core gives the result and the new address. The UI adds no rule (ADR 0150 item 9).
5. The result "moved" comes before the result "changed". A feed with the result "moved" shows no notice "Other sources give a different address" for the same address.

### The Warning On The Feed Page

6. One warning replaces the notices: "A different Studio moved this feed." It shows the label "New address", the address, and the name of the connected file host that gives it, when one does. It says "The old address has a pointer file."
7. The warning has **Follow the Move**, **Move It Back…** and **Show the Difference**. **Show the Difference** shows in the conditions of ADR 0150 item 4.
8. **Keep This File Host** of ADR 0150 item 4 stays in the warning, after the other buttons.

### Follow The Move

9. **Follow the Move** changes only the collection of this Studio. It writes nothing to a file host.
10. Studio reads the feed at the new address. It refuses to follow when the read fails, or when the feed there has a different `podcast:guid`. The error names the reason.
11. When a connected file host gives the new address, the feed gets that file host and that path. The old path becomes a pointer file record of the feed.
12. Studio then records the file at the new address as the last read, as **Read Again** does. A draft with no edits moves to that file. A draft with edits stays, and the merge of ADR 0129 applies.
13. After the follow, the feed has no result "moved", and the public address is the new address.

### No Connection To The New File Host

14. When no connected file host gives the new address, the feed follows the new address as a read-only feed (ADR 0067). It keeps its draft, its versions and its release folder.
15. The feed page then shows a plain notice: "This Studio is not connected to the file host of this address. Connect it to edit the feed here." It has the button **Connect a File Host…**, which opens **File hosts**.
16. When a file host that gives the address is connected later, the notice has **Use This File Host**. It gives the feed that file host and that path, as item 11.

### Move It Back

17. **Move It Back…** opens the review sheet. The write puts the feed file at the old address again.
18. The write is a return in Music Index. The warning and the wait of ADR 0128 and ADR 0190 apply.

### The Colors

19. An outside change and an outside move have the Warning color in each place: the mark of a tile, the mark of the left panel, and the notice on the feed page. The mark of a tile and the notice have the triangle icon.
20. The mark of the left panel keeps its cloud icon and its place in the sequence of ADR 0165 item 2. Only its color changes.
21. The notice "The hosted file changed outside Studio." is a warning, with **Read Again**.
22. The notice "Music Index reads this feed from a different address." of ADR 0150 item 5 does not change in this ADR.

## Alternatives Considered

- **Read Again** follows the move with no question. Not chosen by the operator: Studio then changes the file host of a feed with no confirm.
- Only a clear warning, and the user removes the feed and adds it again. Not chosen: the draft and the versions are lost.
- **Follow the Move** is off until a connected file host gives the new address. Not chosen: the notices stay until the user connects the file host.
- Red for a move, and the Warning color for a change. Not chosen: red says that something is broken, and a move is not a fault.
- Remove **Keep This File Host**. Not chosen at this time: it needs the removal of a route and its tests, and the operator did not ask for it.

## Consequences

- A feed can change from editable to read-only by a button. The notice of item 15 tells the way back.
- Studio cannot know which tool made the move. The words "A different Studio" also apply to a different tool.
- The check reads one more public address for a feed whose folder address is not its public address.
- The warning has four buttons.

## How This Decision Is Checked

Mechanical:

- A core test proves the result "moved" for a pointer file at the folder address, also when the public address is already the new address.
- A core test proves the follow with a connected file host: the new file host and path, the pointer record, and no result "moved" after it.
- A core test proves the follow with no connected file host: a read-only feed with the new public address, its draft and its versions.
- A core test proves the refusal for a failed read and for a different `podcast:guid`.
- An API test proves the route of the follow and its error codes.
- A Node test proves the warning view, the notice of item 15, and the Warning tone of each mark.
- The five commit checks pass.

Visual, by the planning model:

- The warning, the page after the follow, and the read-only notice, at 1280 px light and 390 px dark, with the fit checks.
- A tile and a row of the left panel with the mark of a change and of a move.

Human, by the operator. This gate is open:

- On the server, open the test feed that the desktop app moved. Select **Follow the Move**. The page shows the new address and the notice of item 15.
- Connect the new file host on the server. The feed is editable again.
