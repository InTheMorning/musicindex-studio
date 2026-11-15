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

### Note 2: The Desktop App Asks For The Passphrase At Each Start

Comment: a task must find the options that let the desktop app unlock the vault at its start, with no passphrase from the user.

Facts:

- [ADR 0003](../adr/0003-user-unlocked-secret-vault.md) item 2 owns the rule. The desktop shell keeps the vault as one encrypted local file, and the passphrase of the artist opens it.
- ADR 0003 names the keychain of the operating system as an alternative. The operator rejected it, because the encrypted file can move between computers.
- [ADR 0142](../adr/0142-a-user-selects-the-time-before-the-vault-locks.md) item 2 gives the desktop the lock time "Never" as the start value. That time applies only after an unlock.

Open: the task did not start. An unlock at the start must keep the vault key or the passphrase in a location that is not the vault file. That changes ADR 0003 item 2.

### Note 3: A Keyboard Shortcut For The Left Panel At A Narrow Width

Comment: a task must find the options for a keyboard shortcut that opens and closes the left panel at a narrow width. The shortcut is for a device with no touch screen.

Facts:

- At 760 px or less, Studio shows no left panel. `App` in `crates/api/ui/app.js` makes the `nav` element with the class `sidebar` only when the width is more than 760 px.
- [ADR 0088](../adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 7 owns the narrow layout of the levels. Each level is a page of the stack, and Back opens the level above.
- [ADR 0085](../adr/0085-each-artist-and-label-is-a-destination.md) item 7 puts the destinations on the root page at that width. It names musicindex ADR 0069 R-SHELL-2 item 2 as the rule of the three apps.
- The edge grip shows only on a touch screen. A device with no touch screen has the Back button of the toolbar and the Back key of the browser.
- The search found no keyboard shortcut for a move between pages in `crates/api/ui`. The key handlers there move the focus in a list, a tree, a menu or a tab row.

Open: the task did not start. The shell has no panel to open at that width, so the task must first say what the shortcut shows. A panel that opens above the page changes ADR 0088 item 7, and musicindex ADR 0069 can also apply.
