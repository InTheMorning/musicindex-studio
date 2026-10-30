# Manage the files of a release

Open a release. Select **Files** in the outline.
This page shows the files of the release and the file hosts that hold them.
Use this page to keep copies on more than one file host.
You can also move a release to a different file host.

Each button of this page also works on a server.
On a server, the library folder is in your account on the server. It is not on your computer.

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

- **Default file host of the release files**. New files go to this file host.
- **File host of the feed XML**. If you select a different file host, the feed file moves when you write. The address from before gets a redirect or a pointer file with the new address. The page tells you which.

To undo a move before you write, use **Undo** in the review sheet. See [Review and write your changes](review-and-write.md).

## Library files

Each file is one line with its format, its path, "In use" or "Not in use", and the number of copies.
Select a line to open it.

- **Copy to** a file host. Studio sends a spare copy when you write.
- **Reference**. The feed uses the copy on this file host.
- **Delete**. Studio deletes the copy on that file host when you write.
- **Move to** a file host. The feed uses the file on that file host. Studio copies it there when you write.
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

## Other files on the file hosts

Select **Delete** to remove a file from a file host. Studio deletes it when you write.
A file that the feed uses has a disabled **Delete** button. Remove it from the feed first.
The feed file itself has no **Delete** button.
