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

The diff shows the lines of the feed file that change.
Lines that start with `+` are new. Lines that start with `-` go out.
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

## When the hosted file changed

If someone else or a different tool changed the hosted file after you started the draft, Studio shows "The hosted file changed after the draft started."
Studio writes nothing.
Select **Start Again from the Hosted File**. Studio asks you to confirm. Each change in the draft then goes out.

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
