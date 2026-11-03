# Studio v1 Task 088: The Desktop App

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Phase 13: Desktop Shell". [Distribution plan](../plans/distribution-phase-plan.md), milestone 2.
Decision: [ADR 0124](../adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md).
Research: [Tauri desktop shell research](../research/2026-11-03-tauri-desktop-shell-research.md).

Status: Open. Packet 088a is complete, with its visual check. On 2026-11-03 the operator removed the gate of the distribution plan, so packet 088b can start.

## Goal

An artist installs Music Index Studio, and it opens in its own window. Closing the window keeps Studio in the tray, so a connected app can write under a show grant. The theme and the settings of the UI stay between starts. The collection and the vault of the archive stay.

## Packets

- 088a: the start of the shell in a library of `crates/desktop`, and the fixed port 42069. It changes `studio-desktop` only, and it corrects the loss of the UI settings at each start.
- 088b: the Tauri app `crates/desktop-app`: the window, the link rule, the download, the drop, the tray and the single instance. After the commit of 088a.
- 088c: the installers in the release workflow, adjacent to the archives, with the notes for an unsigned app. After the commit of 088b.
- 088d: "Quit" while work runs (ADR 0124 items 16 to 21): the work count in `studio-api`, the graceful shutdown, the dialog and the wait in the tray. After the commit of 088b. Packets 088c and 088d do not depend on each other.

Visual check of 088a, by the planning model: start `studio-desktop` two times with the same data folder. Change the theme between the two starts. The theme stays. Start a second `studio-desktop` while the first runs: it gets a different port and starts.

Visual check of 088b, by the planning model on Linux: the visual check of ADR 0124.

Visual check of 088d, by the planning model on Linux: the Quit step of the visual check of ADR 0124.

Human test, by the operator, after 088c and 088d: the human test of ADR 0124.

## Packet 088a: The Shell Start And The Fixed Port

### Files

Inspect: `crates/desktop/src/main.rs`, `crates/desktop/Cargo.toml`, `crates/api/src/lib.rs` (`local_router`, `Access`), `crates/api/src/workspace.rs`, `tools/release_desktop.py` (the smoke test and its port), `.github/workflows/desktop-release.yml`, `docs/user-guide/start-on-desktop.md`, `docs/runbooks/install-desktop.md`.

Change: `crates/desktop/src/main.rs`, a new `crates/desktop/src/lib.rs`, `crates/desktop/Cargo.toml`, `tools/release_desktop.py` only when its smoke test needs a port change, `docs/user-guide/start-on-desktop.md`, `docs/runbooks/install-desktop.md`.

Do not change: `crates/core`, `crates/api`, `crates/server`, the UI, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `crates/desktop` gets a library target with the start of the shell. `main.rs` reads the arguments and calls the library. The binary name `studio-desktop` and its arguments do not change.
2. The library gives:
    - `pub const DEFAULT_PORT: u16 = 42069;`
    - `pub fn default_data_dir() -> Option<PathBuf>`, the function of `main.rs` from before, with the macOS rule from before.
    - `pub async fn bind(port: Option<u16>) -> std::io::Result<TcpListener>`. With `None`, it binds `127.0.0.1:42069`, and when that port is in use, `127.0.0.1:0`. With `Some(n)`, it binds `127.0.0.1:n` only, and `Some(0)` gives a free port.
    - A function that opens the workspace and makes the token and the router. It gives the UI address with `#token=`, and a future that serves until a shutdown signal. The Tauri app of 088b calls it, so it takes the shutdown signal as a parameter.
3. The default of `--port` is `None`, which item 2 gives. The usage text says that the default is 42069, else a free port.
4. When the shell uses a free port in the place of 42069, it prints one line. The line says this, and that the UI has its default settings for this start.
5. The user guide and the install runbook give the default port.

### Acceptance Criteria

Mechanical:

- A test: `bind(None)` gives port 42069 when it is free. With a listener on 42069, `bind(None)` gives a different port. `bind(Some(n))` with a listener on `n` gives an error.
- The test must not fail when a different program has 42069 on the computer of the test. It then checks only the second case.
- The smoke test of `tools/release_desktop.py` still passes.
- The four commit checks pass.

Visual: the visual check of 088a.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md (items 7 and 11)
- docs/tasks/studio-v1-task-088-desktop-app.md (the Goal and packet 088a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 088a names.

Goal:
- The start of the desktop shell is a library function that the Tauri app can call.
- The default port is 42069, else a free port, so the UI keeps its settings between starts.

Constraints:
- Follow the Architecture section of packet 088a fully.
- The API binds only a loopback address, and each `/api/` request needs the token (ADR 0017).
- Each comment, each test message and the documents are in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Keep test data in `.studio-dev/`, not in `/tmp`.

Do not touch:
- `crates/core`, `crates/api`, `crates/server`, the UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 088a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 088b: The Tauri App

On 2026-11-03 the planning model checked the packet against the code of 088a. The check added items 11 to 15 of the Architecture and the dialog plugin of item 1. It also added the third result of item 4 and the package `libxdo-dev` of item 10.

In this packet, "Quit" stops the API immediately, as item 6 says. Packet 088d adds the dialog and the wait of ADR 0124 items 16 to 21.

### Files

Inspect: `crates/desktop/src/lib.rs` (from 088a), `Cargo.toml`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, `crates/api/src/lib.rs` (the CSP), `crates/api/ui/` (the downloads: "Export People", "Export Studio" and "Download to This Device"; the links with `target="_blank"`), `crates/api/ui/assets/icon-512.png`, `../musicindex-assets/brand/musicindex-mark.svg`, the research note.

Change: a new crate `crates/desktop-app` (its `Cargo.toml`, `build.rs`, `tauri.conf.json`, `capabilities/`, `icons/` and `src/`), the root `Cargo.toml` (the member), `Cargo.lock`, `.github/workflows/ci.yml` (the Linux packages), `docs/user-guide/start-on-desktop.md`, `crates/desktop/src/lib.rs` and `crates/desktop/src/main.rs` for item 11 only.

Do not change: `crates/core`, `crates/api`, `crates/server`, the UI, the ADRs, `.github/workflows/desktop-release.yml`.

### Architecture

These choices are fixed. Do not change them.

1. The crate `crates/desktop-app` has the package name `studio-desktop-app`, and the product name "Music Index Studio". `tauri.conf.json` has the identifier `org.musicindex.studio` and the publisher "Music Index". It uses Tauri 2 with the feature `tray-icon`, and the plugins `tauri-plugin-single-instance`, `tauri-plugin-opener` and `tauri-plugin-dialog`. The single instance plugin is the first plugin. Each Tauri crate has the newest 2.x version that builds with the Rust pin.
2. The app calls the library of `crates/desktop` with the data folder of `default_data_dir()`, `bind(None)` and its own shutdown signal. It opens one window, with the label `main` and the title "Music Index Studio", at the UI address. It sets no Tauri CSP and has no `remote` capability. The window has no Tauri command.
3. The window turns off the drop handler of Tauri (`disable_drag_drop_handler`).
4. The opener plugin opens an address in the system browser in two cases: a navigation of the window to a different origin, and a new window for any address. The window stays on its page. The rule is a function with a test: it takes the origin of the window and an address, and gives "keep", "open outside" or "block".
    - "keep": the origin of the address is the origin of the window. The origin of a `blob:` address is the origin in it. `about:blank` is also "keep".
    - "open outside": an `http`, `https` or `mailto` address of a different origin.
    - "block": each other address, for example `data:`, `javascript:` and `file:`. The window does not go to it, and the opener does not open it.
    - A new window is never "keep". A new window with an address of the origin of the window opens outside.
5. `on_download` puts each download in the downloads folder of the user. On a name that is in use, it adds ` 2`, ` 3` and so on before the extension. After the download, the app shows a message dialog of `tauri-plugin-dialog` with the name of the file and the button "OK". After a failed download, the dialog says that the download did not complete. If the webview gives no path at the end (macOS), the notice gives the planned name.
6. The tray menu has "Open Studio" and "Quit". "Open Studio" shows the window and gives it the focus. "Quit" stops the API through the shutdown signal, then ends the app.
7. A close of the window hides it. The app keeps running. An exit request of the user that is not "Quit" does not end the app (`RunEvent::ExitRequested` with `prevent_exit`).
8. A second start shows the window of the first start, and gives it the focus.
9. The icons come from `../musicindex-assets/brand/musicindex-mark.svg`, in the sizes that the Tauri bundler needs. Make them with `cargo tauri icon` of `tauri-cli` 2.x. Install it with `cargo install tauri-cli --version ^2 --locked --root .studio-dev/tools/tauri-cli`. The crate keeps the generated files. The final summary gives the command that made them.
10. CI installs the Linux packages of the Tauri prerequisites page (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`), so that the four checks build the crate. The packages go in the step that installs `openssh-server`.
11. `harden_process()` moves from `crates/desktop/src/main.rs` into the library of `crates/desktop`, as a `pub fn`. `studio-desktop` and the app call it first (ADR 0104 item 1). Its Linux test moves with it.
12. The app starts the API in the `setup` hook. It runs `bind(None)` and `start` in `tauri::async_runtime::block_on`, and spawns the serve future with `tauri::async_runtime::spawn`. The window opens only after the bind. When the start fails, the app shows the error in a dialog and ends. No log and no error text holds the UI address, because the address holds the token.
13. `tauri.conf.json` has no window and no `frontendDist`. The app makes the window in code with `WebviewUrl::External` and the UI address. `app.security.csp` is `null`.
14. The crate uses the workspace lints. The binary has `windows_subsystem = "windows"` in a build with no debug assertions.
15. The tray icon uses the default icon of the app.

### Acceptance Criteria

Mechanical:

- A test of the link rule of item 4.
- A test of the name of a download of item 5, with a name that is in use.
- A test: the app gives the data folder of `studio-desktop`.
- The Linux test of `harden_process()` passes from the library of `crates/desktop`.
- The four commit checks pass, with the new crate.

Visual: the visual check of 088b.

### Escalation Triggers

- `WebviewWindowBuilder` has no handler for a new window in the Tauri version of the lock file.
- A Tauri macro or a plugin does not build with `unsafe_code = "forbid"` of the workspace lints.
- `WebviewUrl::External` does not open `http://127.0.0.1:PORT/#token=…`, or the window loses the fragment.
- A step needs a change of `crates/api`, of the UI or of an ADR.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md (items 1 to 12)
- docs/adr/0104-key-material-stays-off-the-disk.md (item 1)
- docs/tasks/studio-v1-task-088-desktop-app.md (the Goal and packet 088b only)
- docs/research/2026-11-03-tauri-desktop-shell-research.md (the sections about the window, the links, the downloads and the drop)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 088b names.

Goal:
- The Tauri app `crates/desktop-app` opens Studio in its own window, on the local API of `crates/desktop`.
- The app stays in the tray when its window closes, and a second start shows the first window.

Constraints:
- Follow the Architecture section of packet 088b fully, items 1 to 15.
- The API binds only a loopback address, and each `/api/` request needs the token (ADR 0017).
- The window exposes no Tauri command to the page. The app has no `remote` capability.
- No log, no error text and no test output holds the UI address with its token.
- Each comment, each test message and the documents are in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Keep test data and tools in `.studio-dev/`, not in `/tmp`.
- Other sessions can write in the tree. Change only the files that the packet names.

Do not touch:
- `crates/core`, `crates/api`, `crates/server`, the UI, the ADRs, `.github/workflows/desktop-release.yml`.
- Do not commit. Do not stage files.

Acceptance criteria:
- The mechanical criteria of packet 088b.
- `cargo build -p studio-desktop-app` makes a binary that starts on this Linux computer (X11, `DISPLAY=:0`). Start it for 10 seconds, then stop it, and report what it printed. Do not do the visual check: the planning model does it.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 088c: The Installers

This packet waits for 088b. The planning model writes its Architecture section after the commit of 088b, from the facts of the release workflow and of `tauri-action` at that time. The research note marks two items as not verified: the attach of `tauri-action` to the draft Release, and the bundle names.

Its fixed choices: the release keeps the five archives and their smoke tests. The workflow adds the AppImage, the DEB and the RPM on Linux, the DMG on macOS, and the MSI on Windows, built on the native runners. No installer is signed. `SHA256SUMS.txt` lists each file. The release notes and the install runbook tell the user how to open an unsigned app on macOS and on Windows.

## Packet 088d: Quit While Work Runs

The planning model checks this packet against the code of 088b before it starts.

### Files

Inspect: ADR 0124 items 16 to 21, `crates/api/src/lib.rs` (`serve`, `local_router`), `crates/api/src/routes.rs` (`write_feed` and each other caller of the write pipeline), the routes of `/app/v1/` and of the change sets, `crates/api/src/uploads.rs`, `crates/desktop/src/lib.rs`, `crates/desktop-app/src/` (from 088b), `crates/server/src/` (the callers of `studio_api::serve`).

Change: `crates/api/src/lib.rs`, a new module of `crates/api` for the work count, the files of `crates/api` that call the write pipeline, `crates/api/src/uploads.rs`, `crates/server/src/` only for a changed signature of `serve`, `crates/desktop/src/lib.rs`, `crates/desktop-app/` (its `src/`, `Cargo.toml` and `capabilities/` for the dialog plugin), `Cargo.lock`.

Do not change: `crates/core`, the UI, the ADRs, `crates/desktop/src/main.rs` other than a changed signature.

### Architecture

These choices are fixed. Do not change them.

1. `studio-api` gives a cloneable work count, `Work`. It has `writes()` and `uploads()`. A write holds a guard from its start to its end. The guard decreases the count when it drops, also on an error and on a panic.
2. Each request that calls the write pipeline holds a write guard. This includes the writes of a change set and the granted writes of `/app/v1/`.
3. An upload in parts is in `uploads()` from its first part to its last part or its delete. An upload with no new part for 2 minutes is not in the count. The count reads the time of the last part. It does not need a timer.
4. `studio_api::serve` takes a shutdown future, and it uses the graceful shutdown of axum. After the signal, it accepts no connection and waits for each request that started.
5. `start` of `crates/desktop` gives the `Work` of the workspace in `Started`.
6. The rule of ADR 0124 items 16 to 20 is a function with no Tauri and no I/O. It takes the work count, the state of the app and an event: "Quit", "Cancel", "Quit Now", "Quit After Write", "Open Studio" or "no work". It gives the next state and an action: "end now", "show the dialog", "wait", "continue", or "stop the API and end".
7. The dialog uses `tauri-plugin-dialog` with the texts of ADR 0124 item 18. The title counts the work: "Studio Is Writing 1 Feed", "Studio Is Writing 2 Feeds", "Studio Is Receiving 1 Music File", or "Studio Is Writing 1 Feed And Receiving 1 Music File".
8. While the app waits, it reads the count each second. When the count is 0, it sends the shutdown signal, waits for the end of `serve`, and ends.
9. The tray menu of the wait is the menu of ADR 0124 item 20. The app puts back the menu of 088b when the user selects "Open Studio".
10. In `studio-desktop`, Ctrl+C stops the shell immediately (ADR 0124 item 21). The serve of `start` stops on the signal with no wait for the work.

### Acceptance Criteria

Mechanical:

- A test: a write guard increases `writes()`, and its drop decreases it, also after an error of the write.
- A test: an upload in parts is in `uploads()` after its first part, not after its last part, and not 2 minutes after its last part. The test gives the time, so it does not wait.
- A test of the function of item 6, with each event in each state.
- A test: the title of item 7 for 1 write, 2 writes, 1 upload, and 1 write with 1 upload.
- The four commit checks pass.

Visual: the visual check of 088d.

### Escalation Triggers

- A write to a file host that does not go through a request of `studio-api`.
- `tauri-plugin-dialog` cannot show three buttons with their own texts on a system. Do not change the dialog. Report the facts.
- The graceful shutdown of axum waits for a connection that stays open with no request.
