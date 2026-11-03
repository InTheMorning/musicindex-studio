# Start Studio on your computer

This page is for the desktop app only.
The desktop app runs on your computer and opens in your web browser.

On a server, you do not start Studio. You open the address of the server and sign in.
See [Sign in to a Studio server](sign-in.md).

## Start the app

The archive starts Studio in your web browser. For the app window, see "Start the app window" below.
A GitHub Release can give an archive for Windows, macOS or Linux.
The archive has the file `START-HERE.md`. It gives the steps to start Studio from the archive.
See [Start Music Index Studio](../runbooks/install-desktop.md).

1. Extract the full archive into a folder that you can keep.
2. On Windows, open `studio-desktop.exe`.
3. On macOS or Linux, open a terminal in that folder and run `./studio-desktop`.

You can also start Studio from the source code with this command:

```sh
cargo run -p studio-desktop -- --data-dir .studio-dev/data
```

Studio prints its version, its data folder and its address. The address looks like this:

```text
http://127.0.0.1:PORT/#token=...
```

1. Copy the full address. Include the text after `#`.
2. Paste the address into your web browser.

Studio shows the Studio page. Keep the terminal open while you use Studio.

## Start the app window

The desktop app opens Studio in its own window. It does not use your web browser.
The GitHub Release has an installer for each system. These installers have no code signature. See [Start Music Index Studio](../runbooks/install-desktop.md) for the steps on macOS and Windows.

- Linux: install the DEB or the RPM, or run the AppImage.
- macOS: open the DMG and move Music Index Studio to Applications.
- Windows: open the MSI.

You can also start the app from the source code:

```sh
cargo run -p studio-desktop-app
```

The app uses the same data folder as `studio-desktop`, and it needs no token from you.
Links to other web sites open in your web browser.
A download goes to your Downloads folder, and the app shows the name of the file.

When you close the window, the app stays in the tray, and Studio continues to run.
Select **Open Studio** in the tray menu to show the window again.
Select **Quit** in the tray menu to stop Studio.
When you start the app a second time, it shows the first window.
If a different Studio uses the same data folder, the app shows "Studio is already running." and stops.
This also applies to `studio-desktop`. Only one Studio can use a data folder at a time.

## Why the address has a token

The token is a long secret code.
Studio accepts only the full address with the token.
If you open the address without the token, Studio shows "Open Studio from its address".
Copy the full address from the terminal again.

Studio tries port 42069 at each start. The same port lets the browser keep the settings of Studio, such as the theme.
If a different program uses port 42069, Studio prints a line and uses a free port for that start.
The settings of Studio are then the default settings for that start.

Studio makes a new token each time that it starts.
After you stop and start Studio, copy the new address.

## Choose where Studio keeps its data

The option `--data-dir` sets the folder for the vault and the collection.
If you do not use it, Studio uses the data folder of your operating system:

| System | Default folder |
| --- | --- |
| Windows | `%LOCALAPPDATA%\musicindex-studio\data` |
| macOS | `~/Library/Application Support/musicindex-studio` |
| Linux | `$XDG_DATA_HOME/musicindex-studio`, or `~/.local/share/musicindex-studio` |

On macOS, an earlier Studio kept its data in `~/.local/share/musicindex-studio`.
If only that folder has data, Studio continues to use it.

You can also use these options.

| Option | What it does |
| --- | --- |
| `--port N` | Sets the port. The default is 42069. When a different program uses it, Studio uses a free port. |
| `--musicindex URL` | Uses a different musicindex address. The default is `https://api.musicindex.org`. |
| `--podcastindex-hub URL` | Sets the hub that gets the announcements after a write. |
| `--version` | Prints the version of Studio and stops. |
| `--help` | Prints the options and stops. |

## Keep a full library on your computer

Studio on your computer keeps a library folder. This folder holds the audio files and the images of your releases.
A server can keep no audio and no images. Then your computer holds the full library.

1. Connect the same file hosts in Studio on your computer. See [Connect a file host](hosts.md).
2. Add the feeds that you want to keep. See [Add feeds](add-feeds.md).
3. Open a release. Select **Files** in the outline.
4. Select **Download All** to download each file that is not in the library.
5. Select **Stop All** to stop the downloads.

You can also select **Download** for one file.
To keep a file from a server, select **Download to This Device** on that server.
See [Manage the files of a release](release-files.md).

## Move Studio to a different computer

Studio can make one file with your feeds, drafts, versions, people and vault.
The file holds your feeds, drafts and people with no encryption. Keep the file safe.

1. Select **Settings** and find **Move Studio**.
2. Select **Export Studio**.
3. Read the facts. Clear **Include the audio and images** to make a smaller file.
4. Select **Export Studio**, and keep the file that your browser downloads.

To import a file, start Studio with an empty data folder. Use `--data-dir` with a new folder.
An import is possible only while the collection has no feed. It cannot replace data.

1. Open **Releases**. The page is empty.
2. Select **Import an Account**.
3. Select **Select the File**, and select the file of **Export Account** from a server or of **Export Studio**.
4. Read the facts of the file.
5. Open the vault of the file with its passphrase or with its recovery key.
6. Select **Open and Import**.

Studio then has your feeds, drafts, versions, people, vault and connected apps.
The import changes no feed and writes nothing to your file hosts.
If the data folder has a vault, Studio asks you to unlock it first.
If the collection has a feed, Studio shows "This Studio has feeds. Import into a new data folder."

## Stop Studio

Press Ctrl+C in the terminal.
Studio does not change a feed when you stop it.
Your drafts stay in the collection.

## Next step

[Set up Studio for the first time](first-run.md).
