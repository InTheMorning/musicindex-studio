# Manage the files of a release

Open a release. Select the tab **Files**.
This page shows the files of the release and the file hosts that hold them.
Use this page to keep copies on more than one file host.
You can also move a release to a different file host.

Each button of this page also works on a server that keeps your files.
On that server, the library folder is in your account on the server. It is not on your computer.
A server can also keep no audio and no images. See [A server that keeps no audio and no images](#a-server-that-keeps-no-audio-and-no-images).

## What the page shows

- **Changes in the library folder**. This section shows only when Studio finds a difference.
- **Files**. The buttons to scan.
- **File hosts**. The file host settings of the release.
- **Library files**. Each file of the release, with its copies.
- **Other files on the file hosts**. Files in the release folder on a file host that are not copies of a library file.

## Scan for changes

- Select **Scan the File Hosts** to read the release folders on the file hosts.
- Select **Scan the Library Folder** to look for changes in the library folder on your computer.

## Changes in the library folder

Each difference is one row. Select a button to decide:

| Difference | Buttons |
| --- | --- |
| New file | **Add to the Library** |
| Missing file | **Download Again** (when Studio has a source), **Forget the File** |
| Changed file | **Use the New File**, **Restore** (when a file host holds a copy) |

## Choose the file hosts

- **Default file host of the release files**. New files go to this file host. The current files stay where they are. This change occurs immediately, and it is not in the draft.
- **File host of the feed XML**. If you select a different file host, the feed file moves when you write. The address from before gets a pointer file with the new address.

A feed file can name a new feed address in the tag `itunes:new-feed-url`. This page and **Feed facts** show that address. If it is a different address, the top of each page of the feed also shows a notice. An app or an index can follow the tag to that address.

To undo a move before you write, use **Undo** in the review sheet. See [Review and write your changes](review-and-write.md).

## Move a release

Select **Move This Release…** to move the feed file and all the files of the release in one action.

1. Select the file host. The sheet shows the plan: the feed file, the audio files, the images and the files that do not move.
2. Select the time that Studio keeps the old files: 7, 30 or 90 days. To keep the old files with no later question, clear **Delete the files … when the move is complete**.
3. Select **Add to the Draft**. Nothing changes on a file host until you write. Review shows each change.

A file that is not in the library does not move, and the plan names it. A Blossom host cannot keep a feed file, so the feed file stays on its file host.

If you change only **File host of the feed XML**, a notice gives the count of the files that stay. Select **Move the Files Too** to open the same sheet.

### After the write

The old files stay on the file host from before, because an app can still use their addresses. **Moves that wait** shows each move, with these lines:

- **New files**. The new copies passed the check of the write.
- **Music Index**. Whether Music Index follows the new feed address. Studio cannot examine each other app.
- **Waiting time**. The days since the write.

When the days are complete and Music Index gives no different address, a notice asks you to delete the old files. Select **Review the Deletion**, examine the review, and write. Studio never deletes these files by itself.

- **Delete Now…** does the same before the end of the waiting time. An app that still uses an old address then gets an error.
- **Keep the Files** removes the move from the list. The files stay.

The pointer file of the feed always stays.

## Library files

Each file is one line with its format, its path, "In use" or "Not in use", and the number of copies.
Select a line to open it.

- **Copy to** a file host. Studio sends a spare copy when you write.
- **Reference**. The feed uses the copy on this file host.
- **Delete**. Studio deletes the copy on that file host when you write.
- **Move to** a file host. The feed uses the file on that file host. When you write, Studio copies the file there. The copy on the file host from before stays for 30 days, and then Studio asks you. See [Move a release](#move-a-release).
- **File host of this file**. Select a file host for this file, or **Use the default file host of the release**.
- **Undo**. Takes a pending copy or deletion out of the draft.

The copy that the feed uses has no **Delete** button.
Move the use to a different copy first.

These actions go into the draft. Studio does them when you write.

## Files that are not in the library

These files are in the list **Library files**. Each one shows "Not in the library".
The top of the list tells you the number of these files and the space that they use.

Select **Download** to get it. The page shows the progress.
Select **Stop** to stop a download.
Select **Download All** to download each such file. Select **Stop All** to stop all downloads.
If a download fails, select **Try Again**.

A copy or a move can need a download first. Studio then asks you to confirm. The write waits for the download.

On a server, your account has a storage limit. The default is 10 GiB, and the operator can change it.
The limit includes the library files, the downloads and the music files that you send.
When the account is full, a download fails with "storage_full: the storage limit of this account is reached".
An import of a music file then shows "the storage limit of this account is reached".
Remove files that you do not use, or get a larger limit from the operator.

## A server that keeps no audio and no images

A server has one of two modes. The operator of the server sets the mode.

- **Keeps your files.** The server keeps your audio files and images in your account. This page works as this guide describes above.
- **Keeps no audio and no images.** The server keeps a file only until the write that sends it to a file host. This is the default mode.

In the second mode, the page starts with the notice "This server keeps no audio and no images."
The server also keeps the feed XML, the drafts and the versions.
Your file hosts and your own computer keep your files. Keep your own copy of each file.
Settings has the section **Files on this server**. It shows the rule and the space that your account uses.

In this mode, the page has no **Download All**, no **Download** and no **Scan the Library Folder**.
The server has no library folder.

### The status of a file

Each file shows one status on its line:

| Status | Meaning |
| --- | --- |
| On its file host | A file host has the file. The server keeps no copy. |
| Waits for the write | The server has the file, and no file host has it. The status gives the date when Studio removes the file. |
| On this server only | The server has the file, and no file host has it. This status is for a file from before the mode change. |
| Removed from this server | Studio removed the file after 7 days. |

On a narrow screen, the status is on a second line below the name of the file.

### The expiry of a new file

A new file waits on the server until you write the draft.
Studio sends a file to a file host only after you read the changes and write.
If you do not write in 7 days from the import, Studio removes the file from the server.
The draft keeps the address of the file.

When Studio removes a file that the draft uses, the page shows "1 file of this draft is missing."
Studio cannot write the draft until you add the file again. Studio does not change the draft by itself.

1. Open the file on the page **Files**. It shows "Removed from this server".
2. Select **Add the File Again**.
3. Select the same file from your computer. The name and the content must be the same as before.

If you select a different file, Studio shows an error and keeps the draft as it is.

### Before you write

The review sheet tells you that Studio removes its copies of the sent files after the write.
The sheet **Add tracks** tells you the same rule before you send the files.

### Download to This Device

Open a file on a server and select **Download to This Device**.
Your browser saves the file on your computer.
The server reads the file from its own storage or from the file host that has a copy. It does not keep a new copy.
This button shows on a server in the two modes. It does not show in the desktop app.

To keep a full library of your files, use Studio on your computer.
See [Keep a full library on your computer](start-on-desktop.md#keep-a-full-library-on-your-computer).

## Other files on the file hosts

Select **Delete** to remove a file from a file host. Studio deletes it when you write.
A file that the feed uses has a disabled **Delete** button. Remove it from the feed first.
The feed file itself has no **Delete** button.

## A change from outside Studio

When Studio starts and each time **Releases** opens, Studio reads the public address of each feed. A feed page also reads its feed when it opens. Studio compares the file with the last file that Studio read or wrote.

A row of the left panel shows one mark for its feed and its releases. A row with no mark is up to date. Hold the pointer on a mark to see its name.

- A red cloud with "!": a feed changed or moved outside Studio.
- A blue pencil: a feed has changes that are not written.
- A gray dashed circle: Studio did not read a feed correctly. The check is in progress, or it is off in **Settings**.

- **Changed outside Studio** means that the hosted file has different bytes. Select **Read Again** on the feed page. A draft with edits keeps its edits.
- **Moved outside Studio** means that the hosted file names a new address. A write from Studio can put the feed back on the old file host. **Show the Difference** opens the merge in Review. **Keep This File Host** hides the notice until the hosted file changes again.
- The feed page also asks Music Index for the address of the feed. If it reads the feed from a different address, a notice gives both addresses. A different collection or a different tool can hold this feed.

The check writes nothing and changes no draft. If a read fails, the page shows nothing new. A feed with no public address has no check. To stop these requests, open **Settings**, then **Outside changes**, and clear **Check for outside changes**.
