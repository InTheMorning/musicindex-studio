# Start Music Index Studio

This archive contains the desktop shell. It starts Studio on your computer and prints a browser address.
It does not open a desktop window. A native window and installers are planned for a later release.

1. Extract the full archive into a folder that you can keep.
2. On Windows, open `studio-desktop.exe` in the extracted folder.
3. On macOS or Linux, open a terminal in that folder and run `./studio-desktop`.
4. Open the printed address in a browser on the same computer.
5. Keep Studio running while you use the browser.

The address contains a private access token. Do not share it or put it in a public log.
Studio keeps the collection and vault in the local application data folder of your operating system.
It keeps the release library in `~/musicindex/studio` unless you select a different folder.
An update does not remove these folders. Keep a backup of your data before an update.

| System | Default collection and vault folder |
| --- | --- |
| Windows | `%LOCALAPPDATA%\musicindex-studio\data` |
| macOS | `~/Library/Application Support/musicindex-studio` |
| Linux | `$XDG_DATA_HOME/musicindex-studio`, or `~/.local/share/musicindex-studio` |

To use a different collection folder, start Studio with `--data-dir PATH`.
To see the version, run `./studio-desktop --version`, or `studio-desktop.exe --version` on Windows.
To stop Studio, close its terminal or press Ctrl+C.

These archives have no code signature. Windows or macOS can show a trust warning.
Check the `SHA256SUMS.txt` file from the same GitHub Release before you run a download.
Only download from the Music Index Studio GitHub Releases page.

To remove this version, delete the extracted application folder.
Delete your collection, vault, or library only if you want to remove your data.
