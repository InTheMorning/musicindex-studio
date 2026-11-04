# Studio v1 Task 088: The Desktop App

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Phase 13: Desktop Shell". [Distribution plan](../plans/distribution-phase-plan.md), milestone 2.
Decision: [ADR 0124](../adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md).
Research: [Tauri desktop shell research](../research/2026-11-03-tauri-desktop-shell-research.md).

Status: Open. Packets 088a, 088b and 088d are complete, with their visual checks. On 2026-11-03 the operator removed the gate of the distribution plan. Packet 088c is complete. Its check passed on the release run of `v0.1.0-rc.16`. The human test of that candidate found the defects of packet 088e.

## Goal

An artist installs Music Index Studio, and it opens in its own window. Closing the window keeps Studio in the tray, so a connected app can write under a show grant. The theme and the settings of the UI stay between starts. The collection and the vault of the archive stay.

## Packets

- 088a: the start of the shell in a library of `crates/desktop`, and the fixed port 42069. It changes `studio-desktop` only, and it corrects the loss of the UI settings at each start.
- 088b: the Tauri app `crates/desktop-app`: the window, the link rule, the download, the drop, the tray and the single instance. After the commit of 088a.
- 088c: the installers in the release workflow, adjacent to the archives, with the notes for an unsigned app. After the commit of 088b.
- 088d: "Quit" while work runs (ADR 0124 items 16 to 21): the work count in `studio-api`, the graceful shutdown, the dialog and the wait in the tray. After the commit of 088b. Packets 088c and 088d do not depend on each other.
- 088e: the drop and Ctrl+Z on Linux (ADR 0124 items 22 to 25), from the human test of `0.1.0-rc.16`.

Visual check of 088a, by the planning model: start `studio-desktop` two times with the same data folder. Change the theme between the two starts. The theme stays. Start a second `studio-desktop` while the first runs: it gets a different port and starts.

Visual check of 088b, by the planning model on Linux: the visual check of ADR 0124. It passed on 2026-11-03 on an Xvfb display with i3 and a D-Bus session, with these results:

- The window opens the UI with the token. The theme stays after a restart. A close hides the window, and the API continues.
- The tray menu has "Open Studio" and "Quit". "Open Studio" shows the window and gives it the focus. "Quit" stops the API and ends the app.
- With D-Bus, a second start shows the first window. With no D-Bus, a second start shows "Studio is already running." and ends with no second API.
- Two downloads of "Export People" give `studio-people.json` and `studio-people (1).json`, and each dialog names its file. On Linux, WebKitGTK gives the handler a free name with ` (1)`, so the ` 2` of item 5 does not occur there.
- A drop reaches the page, because the Tauri drop handler is off. The drag source of the check gave only `text/uri-list`, so the page got text and no file. A plain WebKitGTK window gave the same result. This check did not find the defect: the same result occurs with a file manager. Packet 088e corrects it.

Visual check of 088d, by the planning model on Linux: the Quit step of the visual check of ADR 0124. It passed on 2026-11-03 on Xvfb. A local folder host had a public address on a local test server that waits 40 seconds, so each write waited about 20 seconds at its confirmation. The results:

- "Quit" during the write showed "Studio Is Writing 1 Feed", the message and the three buttons.
- "Cancel" kept the app. Return selected "Quit After Write", so it is the default button on Linux.
- "Quit After Write" hid the window and showed the wait menu. The app ended after the write.
- "Open Studio" during the wait showed the window and the normal menu, and the app continued after the write.
- "Quit Now" during the write ended the app and the API immediately. "Quit" with no work ended the app with no dialog.

Check of 088c, by the planning model: the tag `v0.1.0-rc.16` took the place of the manual run. On 2026-11-03 run 41706767452 passed each of its six jobs. The Release holds the five archives, the nine installers and `SHA256SUMS.txt` with 14 lines. The downloaded Linux x86-64 DEB matches its line, and its control file gives the version `0.1.0-rc.16`.

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
16. `lock_data_dir` of `crates/desktop` locks `studio.lock` in the data folder, and `start` calls it before `Workspace::open`. The serve future holds the lock. A held lock gives a `StartError` with `already_running()`, and the app shows "Studio is already running." The visual check of 088b added this item (ADR 0124 item 10).

### Acceptance Criteria

Mechanical:

- A test of the link rule of item 4.
- A test of the name of a download of item 5, with a name that is in use.
- A test: the app gives the data folder of `studio-desktop`.
- The Linux test of `harden_process()` passes from the library of `crates/desktop`.
- A test: a second lock of one data folder fails, and the lock is free after the first shell ends.
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

On 2026-11-03 the planning model wrote this packet from four sources. These are the release workflow at commit `53c2fe0`, the source of `tauri-bundler` 2.10.4, the page of `tauri-action` and a local Linux build.

### Facts

- `tauri-action` finds or makes a Release in each job of the matrix. The draft job of the workflow already makes the draft, its notes (ADR 0100) and `SHA256SUMS.txt`. So the packet does not use `tauri-action`. Each build job runs `cargo tauri build`, and the draft job attaches the installers with the archives.
- The bundler names each file from the product name, the version and the architecture, for example `Music Index Studio_0.1.0-rc.15_amd64.deb`. The architecture word is different in each format.
- The RPM target refuses a `-` in the version, so `0.1.0-rc.15` fails. The RPM has no setting for its own version. `cargo tauri bundle` makes a bundle from the binary of `cargo tauri build` with no new compile, and its `--config` can give a different `version` and `bundle > linux > rpm > release`. The local build made `Music Index Studio-0.1.0-0.rc.15.x86_64.rpm` in this method. The release `0.rc.15` sorts before the release `1` of `0.1.0`.
- The DEB accepts `0.1.0-rc.15`. The local DEB holds `usr/bin/studio-desktop-app`, the icons and `Music Index Studio.desktop`. Its description is "Tauri desktop app of musicindex-studio", from `Cargo.toml`, and its long description is "(none)".
- The MSI target refuses a pre-release identifier that is not a number, so `0.1.0-rc.15` fails. The setting `bundle > windows > wix > version` replaces the version of the MSI. `allowDowngrades` is `true` by default, so each MSI installs over each other version.
- The AppImage step gets `linuxdeploy` at a fixed commit, and it sets `APPIMAGE_EXTRACT_AND_RUN=1`, so it needs no FUSE. Its GTK plugin reads `/etc/os-release`. The sandbox of the planning model hides that file, so the AppImage fails there. The CI run checks the AppImage. It also tries to get `linuxdeploy-plugin-appimage` from the `continuous` release. When that download fails, it uses its own copy.
- The bundler writes the bundle type into the binary for the updater. After the first bundle, it shows a warning for each other bundle. Studio has no updater (ADR 0124 item 15), so this warning has no effect.
- With `CI` set, the DMG step skips the Finder layout of the window.
- Apple Silicon needs a code signature on each app from the internet. The Tauri page on macOS signing gives the ad-hoc signature `"signingIdentity": "-"` for an app with no Apple identity. The user must still allow the app in Privacy & Security.

### Files

Inspect: `.github/workflows/desktop-release.yml`, `.github/workflows/ci.yml` (the apt packages), `tools/release_desktop.py`, `crates/desktop-app/tauri.conf.json`, `docs/runbooks/desktop-release.md`, `docs/runbooks/install-desktop.md`, `docs/user-guide/start-on-desktop.md`.

Change: `.github/workflows/desktop-release.yml`, `tools/release_desktop.py`, `crates/desktop-app/tauri.conf.json`, `docs/runbooks/desktop-release.md`, `docs/runbooks/install-desktop.md`, `docs/user-guide/start-on-desktop.md`.

Do not change: the Rust code, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. The five archives, their package step and their smoke test do not change.
2. Each build job installs the Tauri CLI with `cargo install tauri-cli --version 2.12.4 --locked`, after the Rust toolchain. This is the version of `.studio-dev/tools/tauri-cli`.
3. Both Linux jobs install `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev`, the packages of `ci.yml`. The job `linux-x86_64` also installs `openssh-server`, as before.
4. A new step in `crates/desktop-app`, after the archive step, makes the installers in `target/release/bundle/`:
    - Linux: `cargo tauri build --ci --bundles appimage,deb`, then `cargo tauri bundle --ci --bundles rpm --config "$(python tools/release_desktop.py bundle-config rpm)"`.
    - macOS: `cargo tauri build --ci --bundles dmg`.
    - Windows: `cargo tauri build --ci --bundles msi --config "$(python tools/release_desktop.py bundle-config msi)"`.
5. A new command `python tools/release_desktop.py bundle-config FORMAT` prints the JSON for `--config`, from the version of the workspace:
    - `msi` for `0.1.0-rc.15`: `{"bundle":{"windows":{"wix":{"version":"0.1.0.15"}}}}`. For `0.1.0`: the version `0.1.0`.
    - `rpm` for `0.1.0-rc.15`: `{"version":"0.1.0","bundle":{"linux":{"rpm":{"release":"0.rc.15"}}}}`. For `0.1.0`: the version `0.1.0` and the release `1`.
    - The command refuses a pre-release that is not `rc.N`, and an `N` above 65535.
6. In `bundle`, `tauri.conf.json` gets these values:
    - `"macOS": {"signingIdentity": "-"}`, the ad-hoc signature with no identity of ADR 0124 item 14.
    - `"category": "Music"`.
    - `"shortDescription": "Edit your hosted Podcasting 2.0 music feeds"`.
    - `"longDescription": "Music Index Studio edits the Podcasting 2.0 music feeds on your file hosts. It keeps your collection and your secret vault on this computer."`
7. A new command `python tools/release_desktop.py installers --target T --bundle-dir target/release/bundle --output .studio-dev/release` copies each installer of the target to a fixed name: `musicindex-studio-v{version}-{slug}{suffix}`. The suffixes are `.AppImage`, `.deb` and `.rpm` on Linux, `.dmg` on macOS and `.msi` on Windows. The command refuses a missing installer and a bundle folder with more than one file of a suffix. A `--skip SUFFIX` option is only for the local check of the planning model.
8. The same command examines the start bytes of each file, with no external tool. The bytes are `!<arch>\n` for DEB, `\xed\xab\xee\xdb` for RPM, ELF with `AI\x02` at byte 8 for AppImage, `\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1` for MSI, and `koly` at 512 bytes before the end for DMG. A wrong file stops the job.
9. `checksums` expects the five archives and the nine installers, and refuses an incomplete set. Its last line says "SHA256SUMS.txt covers all 14 desktop files".
10. The draft notes keep the notes file of the version first (ADR 0100 item 5). The fixed text after it is:

    ```text
    Desktop archives and installers. These files are unsigned. The server image is ghcr.io/inthemorning/studio-server:VERSION.

    On macOS, open the DMG and move Music Index Studio to Applications. At the first start, macOS stops the app. Open System Settings, then Privacy & Security, then select Open Anyway.

    On Windows, open the MSI. If SmartScreen shows "Windows protected your PC", select More info, then Run anyway. Smart App Control can block an unsigned app. You cannot allow only this app in Smart App Control.
    ```

11. A manual run builds the same files as workflow artifacts and makes no Release.
12. `install-desktop.md` stays the `START-HERE.md` of the archives. It gets a short section that names the installers and gives the steps of item 10. `start-on-desktop.md` names the installers in "Start the app window". `desktop-release.md` says that the draft holds 14 files, and that the installers have a file check but no start test.

### Acceptance Criteria

Mechanical:

- `bundle-config msi` and `bundle-config rpm` print the JSON of item 5 for the version of the workspace.
- On this Linux computer, `cargo tauri build --ci --bundles deb`, then the RPM command of item 4, give the DEB and the RPM. The control file of the DEB has the description of item 6.
- `installers` examines the DEB and the RPM with their fixed names, and the file check passes. The sandbox cannot make the AppImage, so a local check of `installers` gives `--skip appimage`. The workflow never gives `--skip`.
- `installers` refuses a file with wrong start bytes, and a missing installer. The final summary gives the commands that showed this.
- The four commit checks pass.

Visual, by the planning model: the operator pushes `main` and starts a manual run. Each job of the run passes. Its artifact holds the archive and the installers of its system.

Human test, by the operator: the human test of ADR 0124, with the installers of the draft.

### Escalation Triggers

- The DMG step or the MSI step needs a tool that the runner does not have.
- `cargo tauri build` changes `Cargo.lock`.
- The installer of a system needs a change of the Rust code.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md (items 12 to 15)
- docs/adr/0100-each-release-candidate-has-a-notes-file.md (item 5)
- docs/tasks/studio-v1-task-088-desktop-app.md (the Goal and packet 088c only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 088c names.

Goal:
- The draft GitHub Release of a version tag holds the five archives and nine installers. These are the AppImage, DEB and RPM of each Linux system, the DMG of each macOS system and the MSI of Windows. `SHA256SUMS.txt` lists all 14 files.

Constraints:
- Follow the Architecture section of packet 088c fully, items 1 to 12.
- Do not use `tauri-action`. Pin each new action by its commit SHA, as the workflow does now. Add no new action if a `run` step can do the work.
- The Tauri CLI is at `.studio-dev/tools/tauri-cli/bin/cargo-tauri`. Put it on `PATH` for the local build.
- Keep the Python code in the style of `tools/release_desktop.py`: the standard library only.
- Each comment and the documents are in ASD-STE100 Simplified Technical English. Check each changed document with `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics FILE`.
- Do not start a GUI app. Do not use the display `:0`.
- Keep test data in `.studio-dev/`, not in `/tmp`.
- Other sessions can write in the tree. Change only the files that the packet names.

Do not touch:
- The Rust code, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, the ADRs.
- Do not commit. Do not stage files. Do not push. Do not start a workflow.

Acceptance criteria:
- The mechanical criteria of packet 088c.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs
- python3 tools/release_desktop.py bundle-config msi
- python3 tools/release_desktop.py bundle-config rpm
- (cd crates/desktop-app && PATH="$PWD/../../.studio-dev/tools/tauri-cli/bin:$PATH" cargo tauri build --ci --bundles deb)
- (cd crates/desktop-app && PATH="$PWD/../../.studio-dev/tools/tauri-cli/bin:$PATH" cargo tauri bundle --ci --bundles rpm --config "$(python3 ../../tools/release_desktop.py bundle-config rpm)")
- python3 tools/release_desktop.py installers --target x86_64-unknown-linux-gnu --bundle-dir target/release/bundle --output .studio-dev/check-088c/release --skip .AppImage

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 088d: Quit While Work Runs

On 2026-11-03 the planning model checked the packet against the code of 088b. The check made items 1, 3, 4, 5, 7 and 10 exact, and added items 11 and 12.

### Files

Inspect: ADR 0124 items 16 to 21, `crates/api/src/lib.rs` (`serve`, `local_router`), `crates/api/src/workspace.rs` (`Workspace`, `Workspace::open`), `crates/api/src/routes.rs` (`write_feed` near line 2734, and the granted write near line 6375 that calls `pipeline::write_granted`), `crates/api/src/uploads.rs` (`put_part`, `finish_upload`, `delete_upload`), `crates/desktop/src/lib.rs`, `crates/desktop/src/main.rs`, `crates/desktop-app/src/main.rs`, `crates/server/src/` (the callers of `studio_api::serve`), the source of `tauri-plugin-dialog` 2.8.4 (`MessageDialogButtons::YesNoCancelCustom`, `MessageDialogResult`).

Change: `crates/api/src/lib.rs`, a new module `crates/api/src/work.rs`, `crates/api/src/workspace.rs` (the field and its getter), `crates/api/src/routes.rs` (the two write routes only), `crates/api/src/uploads.rs`, `crates/server/src/` only for the changed signature of `serve`, `crates/desktop/src/lib.rs`, `crates/desktop/src/main.rs`, `crates/desktop-app/src/` (new modules are allowed), `Cargo.lock` only if a dependency changes.

Do not change: `crates/core`, the UI, the ADRs, the other routes of `crates/api`.

### Architecture

These choices are fixed. Do not change them.

1. `studio-api` gives a cloneable work count, `Work`, in `work.rs`, with an `Arc` and no lock of the workspace. It has `write() -> WriteGuard`, `writes()`, `uploads()` and `uploads_at(now)`. A write holds a `WriteGuard` from its start to its end. The guard decreases the count when it drops, also on an error and on a panic. `Workspace` holds one `Work`, and `Workspace::work()` gives a clone.
2. Each request that calls the write pipeline holds a write guard. These are `write_feed` and the granted write of `/app/v1/`. A change set writes each feed through `write_feed`.
3. An upload in parts is in `uploads()` from its first part to its finish or its delete. An upload with no new part for 2 minutes is not in the count. `Work` keeps the ID of each upload and the time of its last part in memory. `put_part` sets the time, and `finish_upload` and `delete_upload` remove the ID. The count compares the times with `now`, so it needs no timer.
4. `studio_api::serve(listener, router, shutdown)` takes a shutdown future, and it uses `with_graceful_shutdown` of axum. After the signal, it accepts no connection and waits for each request that started. The server shell gives a future that does not end, so its behavior does not change.
5. `start` of `crates/desktop` takes `Workspace::work()` before `local_router`, and gives it in the field `work` of `Started`. The serve future of `start` uses the graceful shutdown of item 4.
6. The rule of ADR 0124 items 16 to 20 is a function with no Tauri and no I/O. It takes the work count, the state of the app and an event: "Quit", "Cancel", "Quit Now", "Quit After Write", "Open Studio" or "no work". It gives the next state and an action: "end now", "show the dialog", "wait", "continue", or "stop the API and end".
7. The dialog uses `tauri-plugin-dialog` with `MessageDialogButtons::YesNoCancelCustom("Quit After Write", "Quit Now", "Cancel")` and the texts of ADR 0124 item 18. The plugin cannot set the default button or a destructive mark. The final summary tells which button is the default on Linux. The title counts the work: "Studio Is Writing 1 Feed", "Studio Is Writing 2 Feeds", "Studio Is Receiving 1 Music File", or "Studio Is Writing 1 Feed And Receiving 1 Music File".
8. While the app waits, it reads the count each second. When the count is 0, it sends the shutdown signal, waits for the end of `serve`, and ends.
9. The tray menu of the wait is the menu of ADR 0124 item 20. The app puts back the menu of 088b when the user selects "Open Studio".
10. In `studio-desktop`, Ctrl+C stops the shell immediately (ADR 0124 item 21). `main.rs` gives `start` a shutdown future that does not end, and selects between the serve future and Ctrl+C.
11. The tray icon has the ID `main`, so that the app can change its menu with `tray_by_id`. A "Quit" while the dialog shows does nothing.
12. On "Quit Now", and on "Quit" with no work, the app ends as in 088b: it sends the shutdown signal and calls `exit(0)` with no wait.

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

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md (items 9 and 16 to 21)
- docs/tasks/studio-v1-task-088-desktop-app.md (the Goal and packet 088d only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 088d names.

Goal:
- "Quit" in the tray of the app waits for a write or an upload that runs, or asks the user, as ADR 0124 items 16 to 21 say.

Constraints:
- Follow the Architecture section of packet 088d fully, items 1 to 12.
- The rule of item 6 is a pure function with no Tauri and no I/O, in its own module of `crates/desktop-app`.
- The write pipeline, the feed rules and the responses of the routes do not change.
- No log, no error text and no test output holds the UI address with its token.
- Each comment, each test message and the documents are in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.
- Keep test data in `.studio-dev/`, not in `/tmp`.
- Other sessions can write in the tree. Change only the files that the packet names.

Do not touch:
- `crates/core`, the UI, the ADRs, the routes of `crates/api` other than the two write routes and the upload routes.
- Do not commit. Do not stage files.

Acceptance criteria:
- The mechanical criteria of packet 088d.
- The existing tests of `crates/api` and `crates/server` pass with no change of their assertions.

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

## Packet 088e: The Drop And Ctrl+Z On Linux

On 2026-11-04 the planning model made this packet directly, after the human test of `0.1.0-rc.16` on the AppImage. The operator selected option A in the questionnaire: the app handles the drop.

### Facts

The planning model got these facts from a drag from Thunar to a plain WebKitGTK 2.54.2 window, with no Studio code:

- The page gets the types `text/uri-list` and `text/html`, no type `Files`, and `files=0`. [WebKit bug 323277](https://bugs.webkit.org/show_bug.cgi?id=323277) reports this as a regression from a security fix. Its status was NEW on 2026-11-04.
- The page does not accept the drag, so the engine puts the `file:///` address in a text box.
- When the page accepts the drag, `getData('text/uri-list')` is empty at the drop. So the page cannot read the path.
- Ctrl+Z reaches the page with no cancel and has no effect. `document.execCommand('undo')` undoes the text.

### Files

`crates/api/src/dropped.rs` is new. `crates/desktop-app/src/linux_page.rs` and `linux_page.js` are new. The packet changes `crates/api/src/lib.rs`, `crates/desktop/src/lib.rs`, `crates/desktop-app/src/main.rs` and `crates/desktop-app/Cargo.toml`.

### Acceptance Criteria

Mechanical:

- The route needs the token and gives the file one time.
- A new drop replaces the file of the earlier drop.
- The routes of the server do not have the route.
- The four commit checks pass.

Visual, by the planning model on Xvfb with the Linux debug build and a drag from Thunar:

- During a drag over "Audio file address", the field shows its drop state and "Drop the file here to send it." The result was a pass.
- A drop of a WAV file sends it, and the address changes to the new file. The duration and the MP3 and lossless rows show. No `file:///` text shows. A file of 100 MB also gave the duration and both rows. The result was a pass.
- A drop of a text file shows "Select an audio file". A drop of two files shows "Drop one file only". The result was a pass.
- A drop on Title changes no text and sends no file. The result was a pass.
- Ctrl+Z and Ctrl+Shift+Z undo and redo the text of the album title, and the feed file name below it changes too. The result was a pass.

Human, by the operator: the drop and Ctrl+Z in the AppImage of the next candidate, from the file manager of the operator. The macOS and Windows windows do not use this packet, and a tester of those systems checks their drop.
