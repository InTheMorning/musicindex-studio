# Start Studio on your computer

This page is for the desktop app only.
The desktop app runs on your computer and opens in your web browser.

On a server, you do not start Studio. You open the address of the server and sign in.
See [Sign in to a Studio server](sign-in.md).

## Start the app

The desktop app has no installer at this time.
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

## Why the address has a token

The token is a long secret code.
Studio accepts only the full address with the token.
If you open the address without the token, Studio shows "Open Studio from its address".
Copy the full address from the terminal again.

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
| `--port N` | Sets the port. The default lets the system choose a free port. |
| `--musicindex URL` | Uses a different musicindex address. The default is `https://api.musicindex.org`. |
| `--podcastindex-hub URL` | Sets the hub that gets the announcements after a write. |
| `--version` | Prints the version of Studio and stops. |
| `--help` | Prints the options and stops. |

## Stop Studio

Press Ctrl+C in the terminal.
Studio does not change a feed when you stop it.
Your drafts stay in the collection.

## Next step

[Set up Studio for the first time](first-run.md).
