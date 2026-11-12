# Review and write your changes

Studio changes a hosted file only when you write.
This page shows how to look at your changes, write them and go back.


## Your changes are a draft

When you edit a feed, Studio saves each change in a draft.
The toolbar shows the state of the draft.
You can close Studio and come back. The draft stays.

## Review the changes

1. Select **Review** in the toolbar.
2. The sheet "Review changes" opens. Studio examines the draft each time that the sheet opens.
3. Read the list of actions and the diff.

The review has two views. Use the menu **View** to select one. Studio keeps your selection on this device.

- **Simplified** is the default. It shows **What changes**. This list gives each change in words, for example "Title: Old name → New name". Each time shows in your local time.
- **Detailed** shows the changed lines of the feed file. Below a changed line with a time, a second line gives your local time.

When many entries get the same new value, **What changes** gives one line, for example "6 credits get the Nostr key City Radio".
It shows two of the changes. Select the line "4 more credits" to see the others.
A Nostr key shows the profile name and the short key. If the profile name is not the name of the credit, Studio shows a warning. Check the key.

A change can be in the file, but not in the list. Then the list ends with a row, for example "1 more change in the XML".
Select **Detailed** to see each changed line.

In **Detailed**, a line that starts with `+` is new. A line that starts with `−` goes out. Studio marks the changed part of each line.

Studio shows three lines around each change. Select "Show N more lines" to open a run of hidden lines.
In **Detailed**, select **Show the Whole File** to see each line. Then select **Show Only the Changes** to go back.
When the draft has nothing to write, Studio says that it matches the hosted file.

The sheet can also list these actions before the diff:

| Action | Meaning |
| --- | --- |
| Upload | Studio sends a file to the file host. The feed uses it. |
| Copy | Studio sends a spare copy. The feed does not use it. |
| Delete | Studio deletes the file after it writes the feed. |
| Confirm | Studio makes sure that the file host gives this file. |
| Move | The feed XML moves to a different file host. |
| Announce | Studio tells Podcast Index about the write. See [Announcements](announcements.md). |

Select **Undo** on a file host operation row to remove it from the draft.
Upload and Announce rows have no **Undo** button.
Studio does not delete a file when a feed in your library refers to it. The row tells you.

Select **Examine the Changes** to look at the draft again.

## Write to the file host

1. Select **Write to the File Host**.
2. Studio writes the feed. The sheet closes.

On a server, a write that needs a secret needs an open vault.
If the vault is locked, Studio shows the sheet **Unlock Studio**. After the unlock, Studio writes.
See [Passphrase, unlock and lock](vault.md).

If a feed needs your Nostr signer, the sheet says "This write signs N events with your signer."
The button then shows "Signing 0 of N…", and the number increases after each signature.
Your signer asks you to approve each signature.

Studio uses the NIP-07 signer extension of the browser.
If the browser has none, Studio uses a remote signer, such as Amber, through NIP-46.
If no remote signer is connected, select **Connect a Signer**. Then scan the QR code or paste a bunker address. See [Sign in to a Studio server](sign-in.md#connect-a-remote-signer).
If the connected signer has a different key than the file host, select **Connect a Different Signer**.
If you have no signer, Studio writes nothing.

Before each write, Studio keeps the file that the write replaces.

The button **Write to the File Host** is not on the sheet in these cases:

- The draft has nothing to write.
- An address in the draft is not correct. Studio lists the fields. Correct them.

Pending file operations and grants can show the button when the feed XML has no change.

If an action waits for a download, the button is disabled. Studio shows "Wait for each download, then write."

## When Studio refuses a write

A feed can name a new address in the tag `itunes:new-feed-url`. Studio follows that address before it writes.
Studio reads each address that the new feed names. It does this when you open the sheet, and again when you write.
Studio refuses the write in these cases:

- The addresses come back to an address that they visited.
- An address does not give an RSS feed, or Studio cannot read it.
- A feed at an address has a different `podcast:guid`.
- A feed at an address names two different new addresses.
- You change the self link to an address other than the address of the feed.
- An address is not a public address.

The sheet shows "Studio refuses this write." It lists each address, with the cause below it.
Studio tells you what changed on the file hosts. You cannot write over a refusal.
The button **Write to the File Host** is disabled while the refusal shows.

If a button shows, select it:

- **Remove the Tag** removes `itunes:new-feed-url` from the draft.
- **Use the Public Address** sets the self link to the address of the feed.

The button changes the draft. Studio then examines the changes again. The diff shows only that one change.

A refusal can come after the new feed file is on the new file host. This happens when the new file host does not give the feed that Studio put there.
Studio then writes no pointer file, and the feed stays at its address. The file stays on the new file host.
The next write replaces it. Correct the cause, then select **Write to the File Host** again.

Studio can also show "The self link names a different address." This is a notice, not a refusal.
Studio can write the feed. Select **Use the Public Address** to make the self link the same as the address of the feed.

## The tag in a moved feed

Each move adds the tag `itunes:new-feed-url` to the feed, with the new address of the feed. The diff shows this change.
Do not remove this tag. The index can need it if you move the feed back later.
A feed that you do not move gets no such tag.

## Move a feed back to an earlier file host

You can move a feed back to a file host that it used before.
That file host has a pointer file at the earlier path. The move row says "The pointer file at the new address becomes the feed."
Studio replaces the pointer file with the feed only when the file has the same `podcast:guid` as the feed.
If the file has a different `podcast:guid`, Studio writes nothing and shows an error.
The file host that the feed leaves gets a pointer file with the new address.

The sheet also shows "This is a return to an earlier address." This is a warning, not a refusal, and you can write the feed.
The index can wait before it follows a return, and the wait grows with each return. During a wait, the index keeps the address from before the return.

## The build date

When you change a feed, Studio sets the build date of the feed: the element `lastBuildDate`. The diff shows it as "Build date". You cannot edit this value.

An index refuses a date that is earlier than the date that it holds. Thus Studio does not use only the clock of this computer. It also asks a server for the time, and it uses the earliest time.
Studio can show these notices. None of them stops the write:

- "The clock of this computer is ahead." Studio uses the time of a server.
- "The build date does not change." No server answered, so the feed keeps its date. A later write sets the date.
- "The build date of this feed is in the future." Studio keeps the date, because an index refuses an earlier date. Only the operator of the index can correct a stored date.

## When the hosted file changed

If the hosted file changed after you started the draft, Studio shows "The hosted file changed after your draft started. Studio did not write."
The notice gives the time that your draft started and the time that the file changed.
The change can come from a different collection, a different Studio version, a different tool or a different person. Compare the times with your writes from other places.
Select **Apply My Edits to the New File**. Studio shows what stays and what comes from the hosted file:

- **Your edits stay** lists your edits. Studio applies them to the new file.
- **New from the hosted file** lists the changes that your draft gets.
- **Both changed. Select one** lists each field that both sides changed. For each field, select your edit or the value of the hosted file.

Select **Apply to the Draft**. Nothing changes on a file host. Examine the changes, then write them. To remove your edits, select **Discard the Draft**.

## Discard the draft

Select **Discard the Draft** in the sheet.
Studio asks you to confirm.
Each change that you did not write goes out. The hosted file does not change.

## Go back to an earlier version

1. Select **Versions** in the toolbar. The sheet "Earlier versions" opens.
2. Find the version by its date.
3. Select **Put in the Draft**.
4. Select **Review**, examine the changes and select **Write to the File Host**.

An earlier version replaces the draft. If the draft has changes, Studio asks you to confirm.
The write that restores a version is a normal write. It keeps the current file as a new version.
