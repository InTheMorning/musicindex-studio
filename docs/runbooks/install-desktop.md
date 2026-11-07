# Start Music Index Studio

This archive contains the desktop shell. It starts Studio on your computer and prints a browser address.
It does not open a desktop window. The GitHub Release also has installers for the app window.

1. Extract the full archive into a folder that you can keep.
2. On Windows, open `studio-desktop.exe` in the extracted folder.
3. On macOS or Linux, open a terminal in that folder and run `./studio-desktop`.
4. Open the printed address in a browser on the same computer. Studio uses port 42069, else a free port.
5. Keep Studio running while you use the browser.

The GitHub Release has an installer for each system. These are an AppImage, a DEB and an RPM on Linux, a DMG on macOS and an MSI on Windows. These installers have no code signature.

On macOS, open the DMG and move Music Index Studio to Applications. At the first start, macOS stops the app. Open System Settings, then Privacy & Security, then select Open Anyway.

On Windows, open the MSI. If SmartScreen shows "Windows protected your PC", select More info, then Run anyway. Smart App Control can block an unsigned app. You cannot allow only this app in Smart App Control.

The address contains a private access token. Do not share it or put it in a public log.
Studio keeps the collection and vault in the local application data folder of your operating system.
It keeps the release library in `~/musicindex/studio` unless you select a different folder.
An update does not remove these folders. Keep a backup of your data before an update.

| System | Default collection and vault folder |
| --- | --- |
| Windows | `%LOCALAPPDATA%\musicindex-studio\data` |
| macOS | `~/Library/Application Support/musicindex-studio` |
| Linux | `$XDG_DATA_HOME/musicindex-studio`, or `~/.local/share/musicindex-studio` |

This folder holds each collection in `collections/`. The first start of a version with collections moves the current data to `collections/main/`. Before the move, keep a backup.

To use a different folder as one collection, start Studio with `--data-dir PATH`. Studio does not move that folder.
To see the version, run `./studio-desktop --version`, or `studio-desktop.exe --version` on Windows.
To stop Studio, close its terminal or press Ctrl+C.

These archives have no code signature. Windows or macOS can show a trust warning.
Check the `SHA256SUMS.txt` file from the same GitHub Release before you run a download.
Only download from the Music Index Studio GitHub Releases page.

To remove this version, delete the extracted application folder.
Delete your collection, vault, or library only if you want to remove your data.
