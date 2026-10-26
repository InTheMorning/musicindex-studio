# Review and write your changes

Studio changes a hosted file only when you write.
This page shows how to look at your changes, write them and go back.

<!-- PHASE-12-GAP: say that Studio asks for an unlock when a write needs a secret and the vault is locked, and that a signer can sign through a remote connection; source: ADR 0055 item 17, ADR 0056 items 17 to 19 -->

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
If the draft is the same as the hosted file, Studio says so.

The sheet can also list these actions before the diff:

| Action | Meaning |
| --- | --- |
| Upload | Studio sends a file to the host. The feed uses it. |
| Copy | Studio sends a spare copy. The feed does not use it. |
| Delete | Studio deletes the file after it writes the feed. |
| Confirm | Studio makes sure that the host gives this file. |
| Move | The feed XML moves to a different host. |
| Announce | Studio tells Podcast Index about the write. See [Announcements](announcements.md). |

Select **Undo** on a row to remove that action from the draft. An upload has no **Undo** button.
Studio does not delete a file when a feed in your library refers to it. The row tells you.

Select **Examine the changes** to look at the draft again.

## Write to the host

1. Select **Write to the host**.
2. Studio writes the feed. The sheet closes.

If a feed needs your Nostr signer, the sheet says "This write signs N events with your signer."
The button then shows "Signing 1 of N…".
Your signer asks you to approve each signature.
If you have no signer, Studio writes nothing.

Before each write, Studio keeps the file that the write replaces.

The button **Write to the host** is not on the sheet in these cases:

- The draft has no change.
- An address in the draft is not correct. Studio lists the fields. Correct them.

If an action waits for a download, the button is disabled. Studio shows "Wait for each download, then write."

## When the hosted file changed

If someone else or a different tool changed the hosted file after you started the draft, Studio shows "The hosted file changed after the draft started."
Studio writes nothing.
Select **Start again from the hosted file**. Studio asks you to confirm. Each change in the draft then goes out.

## Discard the draft

Select **Discard the draft** in the sheet.
Studio asks you to confirm.
Each change that you did not write goes out. The hosted file does not change.

## Go back to an earlier version

1. Select **Versions** in the toolbar. The sheet "Earlier versions" opens.
2. Find the version by its date.
3. Select **Put in the draft**.
4. Select **Review**, examine the changes and select **Write to the host**.

An earlier version replaces the draft. If the draft has changes, Studio asks you to confirm.
The write that restores a version is a normal write. It keeps the current file as a new version.
