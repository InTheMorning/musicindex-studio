# Walk Notes Of 2026-11-15

Date: 2026-11-15.

Status: advisory. This review records the comments of the operator about the candidate `0.1.0-rc.23`. It states no rule. The owner of each item owns the decision.

## Notes

Each note gives the comment of the operator in neutral form, and then the facts from the code.

### Note 1: The Row **Announce** In The Review Sheet

Comment: the row **Announce** in the review of a draft is not necessary. The check box "Send a podping" is in view and gives the same choice.

Facts:

- [ADR 0209](../adr/0209-the-review-sheet-keeps-the-podping-box-in-view-and-names-each-delete.md) items 1 to 3 own the two parts. Item 2 keeps the row **Announce** at the top, with the full words and the address. Item 3 puts the time of the send in that row.
- The consequences of ADR 0209 say that the address of the podping and the check box are in two places.
- `AnnounceLines` in `crates/api/ui/editor.js` makes the row. `announceLines` in `crates/api/ui/announce.js` gives its lines.
- The row gives four things that the check box does not give:
    - The address that the podping names. A move gives two addresses (ADR 0189 item 1).
    - The time of the send. A wait of Music Index moves that time (ADR 0190 items 6 and 7).
    - The line "No announcement. No index learns about this write until it reads the feed by itself." when the check box is off.
    - The cause of no announcement, when the feed has no public address or the write deletes the feed file. The sheet shows no check box in these two conditions.

Open: no decision. A removal of the row changes ADR 0209 items 2 and 3. Each of the four things then needs a new position, or a decision removes it.
