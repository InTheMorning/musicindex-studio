# Manage the files of a release

Open a release. Select **Files** in the outline.
This page shows the files of the release and the hosts that hold them.
Use this page to keep copies on more than one host.
You can also move a release to a different host.

<!-- PHASE-12-GAP: say which actions of this page work on the server, and how the server handles downloads and the storage limit; source: ADR 0055 item 21, ADR 0057 -->

## What the page shows

- **Changes in the library folder**. This section shows only when Studio finds a difference.
- **Files**. The buttons to scan.
- **Hosts**. The host settings of the release.
- **Library files**. Each file of the release, with its copies.
- **Other files on the hosts**. Files in the release folder on a host that are not copies of a library file.

## Scan for changes

- Select **Scan the hosts** to read the release folders on the hosts.
- Select **Scan the library folder** to look for changes in the library folder on your computer.

## Changes in the library folder

Each difference is one row. Select a button to decide:

| Difference | Buttons |
| --- | --- |
| New file | **Add to the library** |
| Missing file | **Download again** (when Studio has a source), **Forget the file** |
| Changed file | **Use the new file**, **Restore** (when a host holds a copy) |

## Choose the hosts

- **Default host of the release files**. New files go to this host.
- **Host of the feed XML**. If you select a different host, the feed file moves when you write. The address from before gets a redirect or a pointer file with the new address. The page tells you which.

To undo a move before you write, use **Undo** in the review sheet. See [Review and write your changes](review-and-write.md).

## Library files

Each file is one line with its format, its path, "In use" or "Not in use", and the number of copies.
Select a line to open it.

- **Copy to** a host. Studio sends a spare copy when you write.
- **Reference**. The feed uses the copy on this host.
- **Delete**. Studio deletes the copy on that host when you write.
- **Move to** a host. The feed uses the file on that host. Studio copies it there when you write.
- **Host of this file**. Select a host for this file, or **Use the default host of the release**.
- **Undo**. Takes a pending copy or deletion out of the draft.

The copy that the feed uses has no **Delete** button.
Move the use to a different copy first.

These actions go into the draft. Studio does them when you write.

## Files that are not in the library

These files are in the list **Library files**. Each one shows "Not in the library".
The top of the list tells you the number of these files and the space that they use.

Select **Download** to get it. The page shows the progress.
Select **Stop** to stop a download.
Select **Download all** to download each such file. Select **Stop all** to stop all downloads.
If a download fails, select **Try again**.

A copy or a move can need a download first. Studio then asks you to confirm. The write waits for the download.

## Other files on the hosts

Select **Delete** to remove a file from a host. Studio deletes it when you write.
A file that the feed uses has a disabled **Delete** button. Remove it from the feed first.
The feed file itself has no **Delete** button.
