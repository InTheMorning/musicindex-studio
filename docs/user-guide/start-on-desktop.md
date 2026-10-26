# Start Studio on your computer

This page is for the desktop app only.
The desktop app runs on your computer and opens in your web browser.

<!-- PHASE-12-GAP: say that this page is for the desktop only, and link the sign-in page for the server; source: ADR 0055 -->

## Start the app

The desktop app has no installer at this time.
You start it from the source code with this command:

```sh
cargo run -p studio-desktop -- --data-dir .studio-dev/data
```

The command prints the address of Studio. It looks like this:

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
If you do not use it, Studio uses `~/.local/share/musicindex-studio`.

You can also use these options.

| Option | What it does |
| --- | --- |
| `--port N` | Sets the port. The default lets the system choose a free port. |
| `--musicindex URL` | Uses a different musicindex address. The default is `https://api.musicindex.org`. |
| `--podcastindex-hub URL` | Sets the hub that gets the announcements after a write. |

## Stop Studio

Press Ctrl+C in the terminal.
Studio does not change a feed when you stop it.
Your drafts stay in the collection.

## Next step

[Set up Studio for the first time](first-run.md).
