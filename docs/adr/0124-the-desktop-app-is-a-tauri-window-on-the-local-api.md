# ADR 0124: The Desktop App Is A Tauri Window On The Local API

- Status: Accepted
- Date: 2026-11-03
- Owner: operator
- Accepted: 2026-11-03 by the operator, through the questionnaire. The operator selected the window on the local API, the current data folder and unsigned first installers. The operator also selected installers adjacent to the archives, the ID `org.musicindex.studio`, the tray, no updater at first, and the fixed port 42069.
- Changed: on 2026-11-03 the operator changed the default port from 39017 to 42069. The Ponder dev server also has the default port 42069. When it runs, item 7 gives Studio a free port for that start.
- Changed: on 2026-11-03 the operator added items 16 to 21, through the questionnaire. "Quit" waits for a write that runs, or asks the user. The operator selected the dialog with three buttons and a wait with no limit. The operator also selected writes and uploads as the work, and the wait in the tray menu.
- Changed: on 2026-11-03 the operator added the lock of item 10, through the questionnaire. The visual check of packet 088b found that the single instance plugin needs D-Bus on Linux. With no D-Bus, a second start opened a second API on the same data folder.
- Changed: on 2026-11-03 the planning model added the ad-hoc signature of item 14, from the Tauri page on macOS signing. It needs no Apple account and no secret. The user must open the app with "Open Anyway" one time.
- Changed: on 2026-11-04 the operator added items 22 to 25 and changed items 3 and 4, through the questionnaire, after the human test of `0.1.0-rc.16`. On Linux a drop did not reach the field, and Ctrl+Z did nothing. [WebKit bug 323277](https://bugs.webkit.org/show_bug.cgi?id=323277) gives the cause of the drop.
- Changes: [ADR 0002](0002-rust-core-json-api-browser-ui.md) item 4, the form of the Tauri shell. The [distribution plan](../plans/distribution-phase-plan.md), milestone 2.
- Uses: [ADR 0016](0016-ui-without-a-build-step.md), [ADR 0017](0017-local-api-access.md), [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) for the granted writes, and the [Tauri desktop shell research](../research/2026-11-03-tauri-desktop-shell-research.md).

## Context

- `studio-desktop` runs the local API on a loopback address and prints the UI address with its access token. The user opens it in a browser. The release gives it as an archive with no installer.
- ADR 0002 item 4 makes the desktop shell a Tauri app. The research of 2026-11-03 compared three forms of the window. A window on the local API needs no change of ADR 0016 or ADR 0017. A UI in the Tauri protocol is on a different origin from the API, and ADR 0017 refuses CORS.
- The default port of `studio-desktop` is 0, so each start has a different origin. The UI keeps the theme and the view of the review in `localStorage`, so each start loses them.
- A connected app writes under a show grant through the local API (ADR 0058). The API must run for that write.

## Decision

### The Window

1. A new crate `crates/desktop-app` is the Tauri app. Tauri 2 is the version, with the Rust pin of `rust-toolchain.toml`.
2. The app runs the `studio-api` router in its own process on the loopback address, with the access token of ADR 0017. It opens one window at `http://127.0.0.1:PORT/#token=…`. The window exposes no Tauri command to the page, so the app has no `remote` capability.
3. The UI and the access rules of the API do not change for the app. The UI has no code for Tauri. The API has one more route for the app, in item 22.
4. On macOS and on Windows, the window turns off the drop handler of Tauri, so that the drop of a file goes to the page (ADR 0062). Items 22 to 24 give the drop on Linux.
5. A link that leaves the origin of the window, or a link with `target="_blank"`, opens in the system browser. The window does not go to it.
6. A download of the page goes to the downloads folder of the user, and the app tells the user the name of the file.

### The Port

7. The default port of the two desktop shells, the app and `studio-desktop`, is 42069. When a different program has that port, the shell uses a free port for that start. The UI then has its default settings for that start. `--port` still sets a port, and `--port 0` gives a free port.

### The App And Its Tray

8. The app has a tray icon, with the menu items "Open Studio" and "Quit". Linux gives no click event of the tray icon, so the menu is the control on each system.
9. When the user closes the window, the window hides, and the app and its API keep running. A connected app can thus write under a show grant. "Open Studio" shows the window again. "Quit" stops the API and ends the app.
10. A second start of the app shows the window of the first one, through the single instance plugin. No second API starts. Each desktop shell also locks a file in its data folder at its start. When a different shell holds that lock, the start stops with the message "Studio is already running." The lock works with no D-Bus, and it also applies to `studio-desktop`.

### The Data And The Name

11. The app uses the data folder of `studio-desktop`: `directories::ProjectDirs::from("", "", "musicindex-studio")` and `data_local_dir()`. A user of the archive keeps the collection and the vault, with no move.
12. The name is "Music Index Studio", the publisher is "Music Index", and the identifier is `org.musicindex.studio`. The identifier does not change after the first installer.

### The Release

13. The release keeps the five archives of `studio-desktop`, and adds the Tauri installers: AppImage, DEB and RPM on Linux, DMG on macOS, MSI on Windows. A Flatpak waits (distribution plan).
14. The first installers are not signed. The macOS app has an ad-hoc signature with no identity, because Apple Silicon needs a signature on an app from the internet. The release notes tell the user how to open an unsigned app on macOS and on Windows.
15. The first installers have no updater. The user installs a new version over the old one. The updater needs a decision with its key and its recovery test.

### Quit While Work Runs

16. When no work runs, "Quit" ends the app immediately and shows no dialog. The Apple guidance shows an alert only when a step can cause a loss that the person cannot repair (`presentations/alerts.md` of the `apple-hig` skill).
17. Work is a request that writes to a file host, and an upload in parts. A write is each request that calls the write pipeline, also the writes of a change set and the granted writes. An upload is work from its first part to its last part. An upload with no new part for 2 minutes is not work. A download into the library and the announcement queue are not work, because they continue at the next start (ADR 0038 item 9, ADR 0052 item 18).
18. When work runs, "Quit" shows a dialog of the operating system. It is not a sheet of the page, because the window can be hidden. The title names the work, for example "Studio Is Writing 1 Feed". The message is "If you quit now, the write stops. You then must do the write again." The buttons are "Cancel", "Quit Now" and "Quit After Write". "Quit After Write" is the default button. When the dialog of the system can mark a destructive button, "Quit Now" has that mark.
19. "Cancel" closes the dialog, and the app continues. "Quit Now" ends the app immediately, and the write stops.
20. "Quit After Write" hides the window. The API continues, and the app waits with no time limit until no work runs. Then it stops the API, waits for each request that started, and ends.

    While it waits, the tray menu starts with the item "Quits after the write…", which the user cannot select. Then it has "Open Studio" and "Quit Now". "Open Studio" shows the window and cancels the quit. "Quit Now" ends the app immediately.
21. These items apply to the app only. In `studio-desktop`, Ctrl+C stops the shell immediately.

### The Page On Linux

WebKitGTK gives a page no file from a drop, and the page cannot read the file address of the drag. It also does not connect Ctrl+Z to the undo command. A plain window of the same engine shows both results.

22. The local router has the route `GET /api/v1/dropped-files/{id}`, with the access rule of ADR 0017. It gives the bytes of each file of the last drop, one time. The ID is random, and the app sets it for each drop. The page cannot name a path. The router of the server does not have this route.
23. On Linux, the window keeps the drop handler of Tauri, so the engine does not put the file address in a text box. The app gives the page the names of the dropped files, the position and the ID of each regular file. A file path does not go to the page.
24. On Linux, the app adds a script to the window. For a drag of `text/uri-list`, the script sends each `dragover` to the page as a file drag, so a field shows its drop state with no change to the UI. At a drop, it sends a `drop` with the files to the element at the position. When that element does not accept a file drag, or it takes one file and the drop has many, the script gets no bytes.
25. On Linux, the same script connects Ctrl+Z to the undo command, and Ctrl+Shift+Z and Ctrl+Y to the redo command, in a text field.
26. The AppImage holds a fixed list of GStreamer plugins (`bundleMediaFramework`, `APPIMAGE_PLUGINS` in `tools/release_desktop.py`). The release workflow installs the plugin packages and `patchelf`, and gives the bundler only that list. The release check refuses an AppImage with a different set. The human test of `0.1.0-rc.17` found that Play hung the app with no audio sink. The human test of `0.1.0-rc.18` found that all the plugins of the runner made the first play of each start wait, and a press of Play on the real file took 4.72 s against 0.41 s with the list.
27. On Linux, the same script makes the toolbar opaque through `--bar-bg`. WebKitGTK does not always draw a backdrop blur, and the page text then shows through the translucent bar. The UI stays the same as in a browser. A browser and the other systems keep the translucent bar of musicindex ADR 0003.

## Alternatives Considered

- A file address that the page reads from the drag on Linux. Not possible: WebKitGTK gives the page an empty value.
- No drop on Linux, with only the file button. Not chosen: the operator selected a drop that works.
- The localhost plugin. Not chosen: it adds a second server for the same UI, and its page warns of security risks.
- The UI in the Tauri protocol. Not chosen: the API is on a different origin, and ADR 0017 refuses CORS.
- The data folder of the Tauri identifier. Not chosen: it needs a move of the collection and the vault on each system.
- The app quits when the window closes. Not chosen: a granted write needs the API.
- Keep the settings of the UI in the collection. Not chosen: the fixed port keeps them with less work.
- "Quit" stops the app immediately, also during a write. Not chosen: a write can stop at the middle with no warning.
- "Quit" waits a maximum of 30 seconds. Not chosen: the operator selected a wait with no limit, and "Quit Now" gives the user the immediate stop.
- A dialog with two buttons, "Cancel" and "Quit After Write". Not chosen: the user cannot then stop a long upload.
- A dialog also for a download and for the announcement queue. Not chosen: they continue at the next start.

## Consequences

- `crates/desktop` gets a library with the start of the shell, and `studio-desktop` and the app use it.
- `studio-api` counts the work of item 17, and it serves with a graceful shutdown. The library of `crates/desktop` gives the count to the app.
- CI installs the Linux libraries of Tauri, and the four checks build the app crate.
- The release workflow builds the installers on the native runners, adjacent to the archives.
- The human tests of the app need a person with each system. The operator has a Linux computer and an iPhone.

## Invariants

- The API binds only a loopback address, and each `/api/` request needs the token (ADR 0017).
- The UI loads no script, style, font or frame from other servers (ADR 0016).
- No signing key or credential is in the repository, an artifact or a log.
- The page cannot name a file path for the route of item 22.

## Non-goals

- Signed installers, notarization and the updater.
- A Flatpak, a Snap, Windows ARM64 and a mobile app.
- A Tauri command or a native menu for a function of Studio.

## Acceptance Criteria

Mechanical:

- A test: the shell start gives port 42069 when it is free, and a different port when a listener has it.
- A test: the link rule of item 5 opens an address of a different origin outside the window, and keeps an address of the origin in it.
- A test: the app opens the data folder of `studio-desktop`.
- A test: the route of item 22 needs the token and gives the file one time. A new drop replaces the earlier file. The routes of the server do not have this route.
- A test: a second lock of one data folder fails, and the lock is free after the first shell ends.
- A test: the work count of item 17 increases during a write and during an upload in parts, and decreases after it. An upload with no new part for 2 minutes is not in the count.
- A test: the decision of items 16 to 20 is a function of the work count and the button. It gives "end now", "show the dialog", "wait" or "continue".
- The four commit checks pass, with the app crate in the workspace.
- A release check: `tools/release_desktop.py installers` refuses an AppImage whose plugins are not the list of item 26.

Visual, by the planning model on Linux: the app starts and shows the UI in its window. It keeps the theme after a restart, hides on close, and shows again from the tray. A file drop and a download work. A page that scrolls shows no text through the toolbar (item 27). "Quit" during a slow write shows the dialog, and each of its three buttons does the action of items 19 and 20.

Human, by the operator: install the app on each system that the operator has, and do the visual steps. Play a track, and hear the audio (item 26). Install a new version over the old one, and keep the collection and the vault.
