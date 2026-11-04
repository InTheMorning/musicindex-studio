# Tauri desktop shell research

- Date: 2026-11-03
- Status: Research. This note states no decision.
- Owner: the planning model and the operator decide the ADR.
- Scope: Phase 13 of [the Studio v1 phase plan](../plans/studio-v1-phase-plan.md) and milestone 2 of [the distribution phase plan](../plans/distribution-phase-plan.md).

## Scope

This note checks the facts that an ADR for the Tauri desktop shell needs. It covers eight questions:

1. The Tauri version and the Rust toolchain.
2. The window architecture and the access rules of a window.
3. The webview features that the Studio UI uses.
4. The application lifecycle.
5. The bundle formats and the GitHub Actions method.
6. The signing and notarization requirements.
7. The updater.
8. The application identifier and the data folder.

Each web claim has a source tag, such as [S1]. The Sources section gives the URL, the version or date shown, and the date read. Each source was read on 2026-11-03.

A claim has the mark "Not verified" when no source confirmed it. The mark also applies to a claim from a search excerpt only. It also applies to a claim that needs a test that was not run. A claim from a repository file has its file path.

## Sources

| Tag | URL | Version or date shown |
| --- | --- | --- |
| S1 | https://github.com/tauri-apps/tauri/releases | Tauri `v2.12.5` is the latest stable tag. Its date is "21 Oct" and the year is not shown. `tauri-cli` `v2.12.4` is marked Latest. `tauri-build` `v2.7.4`, `tauri-bundler` `v2.10.4`. `tauri-v3.0.0-alpha.5` is a pre-release and was excluded. |
| S2 | https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.5/Cargo.toml | `[workspace.package] rust-version = "1.90"` |
| S3 | https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.5/crates/tauri/Cargo.toml | Crate `tauri` version `2.12.5`. The Linux `webkit2gtk` dependency is `version = "2"` with feature `v2_40`. |
| S4 | https://v2.tauri.app/start/prerequisites/ | "Last updated: Aug 20, 2026" |
| S5 | https://v2.tauri.app/security/capabilities/ | No date shown |
| S6 | https://v2.tauri.app/plugin/localhost/ | Requires Rust 1.90 or later |
| S7 | https://v2.tauri.app/reference/config/ | Page date shown: Jul 23, 2023, adjacent to the `dragDropEnabled` entry |
| S8 | https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindowBuilder.html | `tauri` 2.12.5 |
| S9 | https://docs.rs/tauri/2.12.5/tauri/webview/enum.DownloadEvent.html | `tauri` 2.12.5 |
| S10 | https://docs.rs/tauri/2.12.5/tauri/enum.RunEvent.html | `tauri` 2.12.5. The page lists `ExitRequested` and `Exit`. |
| S11 | https://v2.tauri.app/distribute/sign/macos/ | No date shown |
| S12 | https://v2.tauri.app/distribute/sign/windows/ | No date shown |
| S13 | https://v2.tauri.app/plugin/updater/ | No date shown |
| S14 | https://v2.tauri.app/distribute/pipelines/github/ | Action `tauri-apps/tauri-action@v1` |
| S15 | https://v2.tauri.app/plugin/opener/ | No date shown |
| S16 | https://v2.tauri.app/plugin/single-instance/ | Requires Rust 1.90 or later |
| S17 | https://v2.tauri.app/reference/javascript/api/namespacepath/ | No date shown |
| S18 | https://v2.tauri.app/security/csp/ | No date shown |
| S19 | https://v2.tauri.app/develop/calling-rust/ | No date shown |
| S20 | https://v2.tauri.app/distribute/ | No date shown |
| S21 | https://docs.rs/directories/latest/directories/struct.ProjectDirs.html | `directories` 6.0.0 |
| S22 | https://support.apple.com/en-us/102445 | "Published Date: May 27, 2026" |
| S23 | https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/ | `ms.date` 2026-04-23 |
| S24 | https://developer.apple.com/support/compare-memberships/ | "Copyright 2026 Apple Inc." The page shows no fee. |
| S25 | https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution | Only the page title was returned. |
| S26 | https://caniuse.com/css-has | Global usage 95.91% |
| S27 | https://caniuse.com/css-nesting | Global usage 94.48% |
| S28 | https://developer.mozilla.org/en-US/docs/Web/CSS/:has | "Widely available" since December 2023, per the page |
| S29 | https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog | "Widely available" since March 2022, per the page |
| S30 | https://webkitgtk.org/ | Latest news: "WebKitGTK 2.54.2 released!", dated 23 October 2026 |
| S31 | https://webkitgtk.org/2026/03/12/webkitgtk2.50.6-released.html | Search excerpt only. The page was not fetched. |
| S32 | https://github.com/tauri-apps/tauri/issues/2287 and https://github.com/tauri-apps/tauri/discussions/2684 | Search excerpts only. Community sources, not official documentation. |
| S33 | https://docs.rs/tauri/2.12.5/tauri/webview/enum.WebviewUrl.html | HTTP 404 on the read. Not used. |

The Tauri pages in S4 to S20 come from the official Tauri 2 documentation. The fetch tool returned a summary of each page, not the full text. A quoted phrase in this note is from that summary.

## Repository facts

| Tag | File | Fact |
| --- | --- | --- |
| R1 | `AGENTS.md` | The desktop shell is a Tauri app under [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md) item 4. |
| R2 | `docs/adr/0016-ui-without-a-build-step.md` | The UI is plain HTML, CSS and JavaScript. The UI files are in the Rust binary. The UI loads nothing from a CDN. Item 4 is changed by ADR 0109 for images and audio only. |
| R3 | `docs/adr/0017-local-api-access.md` | The API listens on a loopback address only. Each start makes a 32-byte token. The UI address has the token in the URL fragment. The API refuses a `Host` that is not a loopback name, with status 403. Item 8: the API sends no CORS headers. |
| R4 | `docs/adr/0109-images-and-audio-load-from-their-hosts.md` | The page rule is `default-src 'self'; img-src 'self' data: https:; media-src 'self' https:; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`. Item 3 keeps `Referrer-Policy: no-referrer`. Item 8: on the desktop, the page is on a loopback address with `http`. |
| R5 | `crates/api/src/ui.rs:283` | The page rule of R4 is set in the Rust code. |
| R6 | `crates/api/src/access.rs:56` | `ui_url()` gives `http://127.0.0.1:{port}/#token={token}`. |
| R7 | `crates/desktop/src/main.rs` | The shell has a `tokio::main`. It binds `127.0.0.1` on `--port`, whose default is `0`. It prints the URL of R6, then serves `local_router`. `default_data_dir()` uses `directories::ProjectDirs::from("", "", "musicindex-studio")` and `data_local_dir()`. On macOS, it uses the old folder `~/.local/share/musicindex-studio` when that folder has data and the new folder has none. `harden_process()` uses `rustix` on Linux. |
| R8 | `Cargo.toml` and `rust-toolchain.toml` | The workspace version is `0.1.0-rc.15`. The workspace `rust-version` is `1.93`. The toolchain pin is `1.99.1`. |
| R9 | `crates/desktop/Cargo.toml` | The crate uses `directories` 6 and `tokio` with the features `rt-multi-thread`, `macros` and `net`. |
| R10 | `crates/core/src/library.rs:279` and `crates/api/src/routes.rs:1191` | Each one uses `directories::BaseDirs` to find the home folder. The library default is `~/musicindex/studio`. |
| R11 | `.github/workflows/desktop-release.yml` | Five native runners build the archives with `cargo build --locked --release -p studio-desktop`. A draft job makes the Release with `gh release create --draft`. The Linux runners are `ubuntu-22.04` and `ubuntu-22.04-arm`. |
| R12 | `tools/release_desktop.py` | The smoke test starts the binary with `--data-dir`. It reads the `http://127.0.0.1:` line from stdout. It checks the token, the `Host` refusal and the vault after a restart. |
| R13 | `docs/plans/distribution-phase-plan.md` | The plan says "Windows has no default path through this code." Source S21 suggests that the `directories` crate does return a Windows path. This is a conflict to check. |
| R14 | `crates/api/ui/*.js` and `styles.css` | The UI uses drag and drop onto file fields (`picker.js`, `editor.js` lines 730 to 802). It uses a `Blob` with `a.download` (`app.js:167`, `peoplepages.js:95` to `98`). It has an `<audio>` element (`editor.js:420`). It uses `showModal()` (`app.js`, `editor.js`). `styles.css` uses `:has()` (lines 330, 858, 1111), `color-mix()` (lines 90, 949, 950), `backdrop-filter` (lines 664, 665, 838), `dvh` units, `aspect-ratio`, and `::backdrop` (line 697). The file has no CSS nesting, because no line starts with `&`. |
| R15 | `crates/api/ui/*.js` | The UI uses `localStorage` (`theme.js`, `app.js`, `editor.js`) and `sessionStorage` (`app.js`). It uses `navigator.clipboard` (`app.js`), `beforeunload` (`peoplepages.js`) and `target="_blank"` links (`app.js:1854`, `editor.js:2202`, `peoplepages.js:580`). |

## Question 1: Tauri version and the Rust toolchain

Facts:

- The latest stable Tauri 2 release is `v2.12.5` [S1]. The crate `tauri` has version `2.12.5` [S3].
- The Tauri crate declares `rust-version = "1.90"` at the workspace level [S2].
- The `localhost` plugin [S6] and the `single-instance` plugin [S16] each require Rust 1.90 or later.
- The Studio toolchain pin is `1.99.1` [R8]. The workspace `rust-version` is `1.93` [R8]. The values fit: 1.90, then 1.93, then 1.99.1.
- The MSRV of every dependency in the full Tauri tree is Not verified.
- A Tauri 3 alpha exists. Its tag is a pre-release [S1]. Tauri 3 is Not verified as a plan item.
- The Linux build needs the WebKitGTK 4.1 development packages. The page names `libwebkit2gtk-4.1-dev` on Debian and Ubuntu, `webkit2gtk-4.1` on Arch, and `webkit2gtk4.1-devel` on Fedora [S4]. The page names no runtime minimum for WebKitGTK.
- The newest WebKitGTK release shown is `2.54.2`, dated 23 October 2026 [S30]. The package version in Ubuntu 22.04 is Not verified.

## Question 2: Window architecture and access rules

### Access rules of a window

- By default, API access is limited to bundled code. Remote sources get access only when a capability grants it [S5].
- The grant uses a `remote` field with `urls` in a capability file. The grant names the windows and the platforms [S5].
- A remote URL gets only the commands named in the `permissions` array of the capability [S5].
- The page warns that Tauri cannot separate a request from an embedded `<iframe>` from a request of the window on Linux and Android [S5]. It advises careful use of remote access [S5].
- The security boundary depends on the window label, not on the title [S5].
- The page on calling Rust does not state how remote URLs are handled [S19].

### Content Security Policy

- The Tauri CSP applies to the app HTML pages when it is set in the Tauri configuration [S18].
- The page does not state whether a Tauri CSP applies to a remote or a localhost URL. This is Not verified.
- The server headers of R4 apply when the axum server serves the page. Whether a Tauri window keeps those headers is Not verified.

### Window builder options

- `WebviewWindowBuilder` has `disable_drag_drop_handler`, `on_download`, `devtools` and `use_https_scheme` [S8].
- `use_https_scheme` defaults to `false`. On Windows and Android, a custom protocol uses `http://<scheme>.localhost` by default. Changing this value moves `localStorage`, IndexedDB and cookies [S8].
- The `WebviewUrl` reference page returned HTTP 404 [S33]. The meaning of `WebviewUrl::External` for a `http://127.0.0.1` URL is Not verified.

### Localhost plugin and custom protocol

- The `localhost` plugin serves the app assets through a localhost server, not the default custom protocol [S6]. Its page says: "This plugin brings considerable security risks and you should only use it if you know what you are doing." It advises the default custom protocol when unsure [S6].
- The default custom protocol and the `asset:` protocol are named in the CSP examples [S18]. Their handler API and file scope are Not verified in this note.

### Origin and storage

- The UI address of R6 includes a port. The default `--port 0` picks a free port at each start [R7].
- The web origin of an `http` address includes the port. Browser `localStorage` and `sessionStorage` are kept per origin. This rule is Not verified by a source in this note. The rule needs a test with the desktop shell before a decision.
- If the origin changes at each start, the UI loses its theme, its `localStorage` values and its `sessionStorage` token at each start. The token is random at each start [R3].

### Options for the window

Option A. An external window that loads `http://127.0.0.1:PORT/#token=...`. The axum router of the existing API serves the UI in the same process.

- The window needs no Tauri command. Option A then needs no `remote` capability, because no command is exposed [S5]. This is a derivation.
- The UI calls its own `/api/` origin. This keeps the origin rule of R3 item 8, with no CORS header.
- ADR 0016 holds, because the UI files stay in the binary and the axum server serves them [R2].
- ADR 0017 holds, because the server still binds loopback, checks the token and checks the `Host` [R3].
- The cost is the port and the storage rule. A fixed port avoids a new origin at each start, but the port may be busy. The fixed port choice is a design decision.
- The fragment token of R3 is not sent to a server by a browser. The Tauri webview behavior is Not verified.

Option B. The `localhost` plugin. The plugin serves the bundled assets on a second localhost server.

- The plugin adds a second server for the same UI. The axum server already serves the UI, so this adds risk and no function.
- The plugin page gives a security warning [S6].
- The cost is a plugin, a second port and a second origin.

Option C. Bundled UI in a custom protocol, with `/api/` calls to the axum server.

- The page of the window has a different origin from the API. The API sends no CORS header under ADR 0017 item 8 [R3]. Option C therefore needs either a CORS change, which changes ADR 0017, or a move of the API routes into the protocol handler.
- The bundled UI needs Tauri IPC or a protocol route for the token. This needs a capability, or a route change.
- ADR 0016 holds, because the UI files stay in the binary. ADR 0017 needs a change.
- The cost is the largest of the three options.

Option A needs the fewest changes. Option A still needs a test for the storage rule and the window origin before an ADR.

## Question 3: Webview features

The engines are WebKitGTK on Linux, WKWebView on macOS and WebView2 on Windows. The sources below cover only some of these engines. The table marks each unknown cell.

### Engine facts

- WebKitGTK: the newest release is `2.54.2`, dated 23 October 2026 [S30]. Tauri needs the WebKitGTK 4.1 packages on Linux [S4].
- WebView2: Microsoft Edge WebView2 is preinstalled on Windows 10 version 1803 and later. Older systems need the Evergreen Bootstrapper [S4]. The Chromium version of WebView2 is Not verified.
- WKWebView: the version of WKWebView follows the macOS version. The mapping is Not verified.

### Feature table

| Feature | Studio use (R14) | WebKitGTK | WKWebView | WebView2 |
| --- | --- | --- | --- | --- |
| `<dialog>` and `showModal()` | Yes | Not verified | Not verified | Not verified |
| `:has()` | Yes, three rules in `styles.css` | Not verified | Safari 15.4 first [S26], per the caniuse data. The engine version mapping is Not verified. | Chromium 105 first [S26], per the Edge row. The mapping is Not verified. |
| `color-mix()` | Yes, three rules | Not verified | Not verified | Not verified |
| CSS nesting | No | Not verified | Safari 17.2 full support [S27] | Chrome and Edge 120 full support [S27] |

- The `dialog` element is "Widely available" since March 2022 [S29]. The first version for each engine is Not verified.
- The `:has()` selector is "Widely available" since December 2023 [S28]. Caniuse lists Safari 15.4, iOS Safari 15.4, Chrome 105, Edge 105 and Firefox 121 as first full versions [S26]. The global usage is 95.91% [S26].
- CSS nesting is not used by Studio [R14]. Caniuse lists Safari 17.2, Chrome 120, Edge 120 and Firefox 117 as full versions [S27].
- The features `backdrop-filter`, `dvh`, `aspect-ratio` and `::backdrop` are used by Studio [R14]. Their support in each engine is Not verified.

### Drag and drop

- Studio drops files onto a file field (R14). The drop code is in `picker.js` and `editor.js`.
- Tauri enables its own drag handler by default. The setting is `dragDropEnabled` in the window configuration, and its default is `true` [S7].
- The configuration page says that disabling it "is required to use HTML5 drag and drop on the frontend on Windows" [S7]. The builder page says the same for Windows, because Tauri replaces the WebView2 handler [S8].
- The effect on macOS is Not verified. On Linux, the page gets no file with the handler off (ADR 0124, packet 088e of task 088).
- The `disable_drag_drop_handler` method is the builder form of the setting [S8].

### File downloads

- Studio saves a file with a `Blob` and a link with a `download` name [R14].
- In Tauri 2, `on_download` receives `DownloadEvent::Requested` with a `url` and a mutable `destination`. The new path must be absolute. Returning `false` on `Requested` blocks the download [S9].
- `DownloadEvent::Finished` has `url`, `path` and `success`. The `path` can be `None`. On macOS, `path` is always empty [S9].
- The default save location of each engine, without an `on_download` handler, is Not verified.
- A save dialog plugin is Not verified in this note.

### MP3 and FLAC playback

- Studio uses an `<audio>` element for each track (R14).
- The codec support of WebKitGTK, WKWebView and WebView2 for MP3 and FLAC is Not verified.
- A search excerpt suggests that the GStreamer `gst-plugins-good` package has MP3 and FLAC decoders on some distributions [S31 and search results]. This claim is Not verified. The package names for each Linux distribution are Not verified.
- The audio route and the media rules are in ADR 0109 [R4].

### Links to musicindex.org

- Studio has `target="_blank"` links with `rel="noopener"` [R14].
- The default behavior of `target="_blank"` in Tauri 2 is Not verified. The Tauri pages read in this note do not state it.
- The opener plugin provides `openUrl` from `@tauri-apps/plugin-opener` [S15]. Its permission string is `opener:allow-open-url` [S15]. The default permission allows `http`, `https`, `mailto` and `tel` URLs [S15].
- The plugin blocks other commands until the capability enables them [S15].

### Other browser features

- `localStorage` and `sessionStorage`: see Question 2, Origin and storage.
- `navigator.clipboard.writeText`: the secure-context rule and the webview behavior are Not verified.
- `beforeunload`: the behavior in each webview is Not verified.

## Question 4: Application lifecycle

### Single instance

- The `single-instance` plugin must register first, before other plugins [S16].
- The callback gets `app`, `args` and `cwd` [S16].
- The plugin has no JavaScript API. It needs no capability [S16].
- On Linux, the plugin uses DBus. The page says it does not work by default for Snap or Flatpak packages [S16].

### Last window closed

- The `RunEvent` reference lists `ExitRequested` with `code` set to `None` when the user starts the exit, and `Some` when code starts it [S10].
- The reference page lists no `Reopen` variant [S10]. Whether the variant exists in `2.12.5` is Not verified.
- A community discussion says macOS keeps the process running after the main window closes, and Windows ends it [S32]. This is not official documentation, and it is Not verified.

### API server

- The current shell runs the API in `main` with a tokio runtime. The API stops when the process ends [R7].
- The rule for running a tokio server beside a Tauri event loop is Not verified in this note.
- The vault key stays in memory only while the process runs [ADR 0003 and R7]. This is a derivation.

### macOS reopen

- A reopen from the Dock needs the `Reopen` event. This is Not verified for Tauri 2.12.5.

## Question 5: Packaging and the GitHub Actions method

### Bundle formats

- The distribution page lists these Linux formats: Debian package, Snap, AppImage, Flatpak, RPM and AUR [S20].
- The distribution page lists these macOS formats: App Bundle, DMG and App Store [S20]. It states: "Both methods requires code signing, and distributing outside the App Store also requires notarization."
- The distribution page names the Microsoft Store and a "Windows Installer" [S20]. It does not name MSI or NSIS.
- The configuration reference lists the bundle targets `deb`, `rpm`, `appimage`, `msi`, `nsis`, `app` and `dmg`. The default is `all` [S7].
- MSI builds need VBSCRIPT [S4].
- A Windows installer built from Linux or macOS needs a custom `signCommand` [S12].

### GitHub Actions

- The Tauri GitHub guide uses `tauri-apps/tauri-action@v1` [S14].
- The guide shows the inputs `releaseDraft`, `prerelease`, `tagName`, `releaseName` and `releaseBody` [S14].
- The matrix `args` for macOS are `--target aarch64-apple-darwin` and `--target x86_64-apple-darwin` [S14].
- The guide does not show the signing secret names. It refers to the signing pages [S14].
- The guide does not state whether the action attaches files to an existing draft Release. This is Not verified.
- The `uploadUpdaterJson` input is not on the guide page. Its existence is Not verified.
- The guide says that Tauri Action generates a static JSON file for GitHub Releases [S13].

### Fit with the current workflow

- The current workflow builds five archives on five native runners [R11].
- The current draft job runs `gh release create --draft`, then uploads the archives and `SHA256SUMS.txt` [R11][R12].
- Both the action and the current job can create a Release. The combined behavior is Not verified.

## Question 6: Signing and notarization

### macOS

- Signing needs an Apple Developer account. The Tauri page states a paid cost of $99 per year, and says the free tier is for testing and development only [S11].
- The Tauri page states that a Developer ID Application certificate is used outside the App Store [S11].
- Only the Account Holder can create a Developer ID Application certificate [S11].
- Notarization is required with a Developer ID Application certificate [S11].
- The free tier cannot notarize [S11].
- The Apple membership page did not show the fee [S24]. The fee of $99 per year is Not verified against an Apple page.
- The Apple notarization page returned only its title [S25]. The notarization steps of Apple are Not verified in this note.
- The signing variables are `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE` (base64 `.p12`) and `APPLE_CERTIFICATE_PASSWORD` [S11].
- The notarization variables are either `APPLE_ID`, `APPLE_PASSWORD` and `APPLE_TEAM_ID`, or `APPLE_API_ISSUER`, `APPLE_API_KEY` and `APPLE_API_KEY_PATH` [S11].
- Gatekeeper and an unnotarized app: macOS Catalina and later require notarization by default [S22]. The user can open the app with System Settings, Privacy and Security, then "Open Anyway" [S22]. The page says the warning appears again, and the user clicks Open once [S22].
- Tauri states that an unsigned browser download can be reported as "broken and can not be started" [S11]. It states that Apple Silicon requires code signing for internet apps, and an ad-hoc signature `"-"` can help [S11]. The users may still need to allow the app [S11].

### Windows

- Tauri documents these Windows signing options [S12]:
  - Azure Artifact Signing, formerly Azure Code Signing or Azure Trusted Signing, through `bundle > windows > signCommand`. It needs the variables `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` and `AZURE_TENANT_ID`.
  - Azure Key Vault with `relic`, through `signCommand`.
  - A PFX or OV certificate with `certificateThumbprint`, `digestAlgorithm` and `timestampUrl` under `windows`.
  - A custom `signCommand`.
- The Tauri page says OV certificates are usually cheaper and available to individuals [S12]. The prices are Not verified.
- SmartScreen checks the reputation of the file and of its signature [S23]. An app with no reputation shows a warning [S23].
- The Tauri page says that since 2024, EV certificates no longer give immediate SmartScreen reputation [S12]. EV and OV certificates both build reputation over time [S12].
- Reusing one certificate helps the reputation carry over [S12]. The Tauri page says that submitting the file to Microsoft may help, but it is not guaranteed [S12].
- Without signing, a user can accept the SmartScreen warning [S12]. Windows 11 Smart App Control can block an unknown unsigned app, and a user cannot allow one app only [S12].

## Question 7: The updater

- The updater needs a key pair. The command is `tauri signer generate` [S13]. Losing the private key prevents updates to existing installs [S13].
- The `pubkey` value in `plugins.updater` is the key content, not a file path [S13].
- The `endpoints` value is a list of URLs. TLS is required in production. Tauri tries the next URL only when a response is not 2xx [S13].
- The Windows `installMode` value is `passive` by default. The other values are `basicUi` and `quiet` [S13].
- The static JSON manifest has `version`, optional `notes`, optional `pub_date` in RFC 3339, and `platforms`. Each platform entry has `url` and `signature`, where `signature` is the content of the `.sig` file [S13].
- The platform keys have the form `OS-ARCH`, for example `windows-x86_64` [S13].
- The page gives this GitHub example: `https://github.com/user/repo/releases/latest/download/latest.json` [S13].
- Whether a draft Release can serve the `latest` URL is Not verified. Only the published assets are expected to serve it. This needs a GitHub check.
- The repository runbook names `TAURI_SIGNING_PRIVATE_KEY` as the CI secret [R11 and the runbook]. The runbook is `docs/runbooks/desktop-release.md`.

## Question 8: Application identifier and data folder

### Tauri paths

- The `identifier` in the Tauri configuration is required and uses reverse-DNS notation. `tauri build` rejects the default `com.tauri.dev` [S7].
- `appDataDir()` resolves to `${dataDir}/${bundleIdentifier}`. `appLocalDataDir()` resolves to `${localDataDir}/${bundleIdentifier}` [S17].
- On Linux, the base is `$XDG_DATA_HOME` or `$HOME/.local/share` [S17].
- On macOS, the base is `$HOME/Library/Application Support` [S17].
- On Windows, `appDataDir` uses `{FOLDERID_RoamingAppData}` and `appLocalDataDir` uses `{FOLDERID_LocalAppData}` [S17].
- The `app > appDirectoriesOverride` setting can replace the default [S17].

### The directories crate

- `directories` 6.0.0 gives `data_local_dir()` for `ProjectDirs::from("", "", "musicindex-studio")` [S21].
- On Linux, the path is `$XDG_DATA_HOME/musicindex-studio`, or `$HOME/.local/share/musicindex-studio` [S21].
- On macOS, the page gives the base `$HOME/Library/Application Support`. The docs state that an empty qualifier affects only macOS, and an empty organization affects macOS and Windows [S21]. The exact macOS path with empty values is inferred as `~/Library/Application Support/musicindex-studio`, and is Not verified by a run.
- On Windows, the page gives `{FOLDERID_LocalAppData}\<project>\data` [S21]. With an empty organization, the path is inferred as `%LOCALAPPDATA%\musicindex-studio\data`. This is Not verified by a run.
- R13 says Windows has no default path through this code. S21 suggests that the current code does return a Windows path. The plan text may be out of date. This is Not verified by a Windows run.

### Comparison

- The current shell uses the `directories` path of R7. On Linux, the folder is the same as the `directories` default in S21.
- The macOS folder is `~/Library/Application Support/musicindex-studio` when the new path is used. The macOS fallback is `~/.local/share/musicindex-studio` [R7].
- If the Tauri shell uses the `appDataDir` API, the folder changes. On Linux, the folder becomes `$XDG_DATA_HOME/<identifier>`. On macOS, the folder becomes `~/Library/Application Support/<identifier>`. On Windows, the folder becomes `%APPDATA%\<identifier>`. The current collection and vault would not be found at these paths without a migration.
- The identifier also sets the bundle identity of the native app. The plan asks for a stable identifier [R6]. The effects of a later change on Windows and macOS installs are Not verified.
- The library default `~/musicindex/studio` uses the home folder [R10]. The Windows home folder result from `BaseDirs` is Not verified.

## Effects on Studio

### Question 1: version and toolchain

- Current code: the toolchain pin `1.99.1` and the `rust-version` `1.93` fit the Tauri floor `1.90` [R8][S2]. No toolchain change is needed for Tauri 2.12.5, as far as the MSRV facts show.
- The Linux CI runners need the WebKitGTK 4.1 packages before the Tauri build [S4]. The Ubuntu 22.04 package version is Not verified.
- Options and costs:
  - Stay on the workspace `rust-version` and the pin. Cost: a dependency with a higher MSRV could block the build. Not verified.
  - Pin a Tauri version in `Cargo.lock` and update it on a schedule. Cost: an update step each release.

### Question 2: window and access

- Current code: `main.rs` prints the URL with the token [R7]. The API keeps the loopback bind, the token and the `Host` check [R3].
- Option A (external window to the axum server) changes the least. It keeps ADR 0016 and ADR 0017 [R2][R3]. Costs: a port choice, a storage test and a Tauri CSP check.
- Option B (`localhost` plugin) adds a second server and a plugin warning [S6]. Cost: more risk and no function.
- Option C (custom protocol) needs a CORS change or a move of the API routes [R3]. Cost: a change of ADR 0017, or a large route change.
- The fixed port choice has one design question: a fixed port avoids a new origin at each start, but it can fail when the port is busy. Not verified.

### Question 3: webview features

- The Linux webview, WebKitGTK, is the largest risk for the features of `styles.css`. The current feature support is Not verified for each engine. A visual check on each system is needed before a release.
- `:has()` has a clear first version on each caniuse row [S26]. The WebKitGTK version is Not verified.
- CSS nesting is not in use, so no effect today [R14][S27].
- Drag and drop: Studio needs HTML5 drop on fields [R14]. Options and costs:
  - Set `dragDropEnabled` to `false` on each window. Cost: Tauri file-drop events stop. Studio does not use them today [R14].
  - Keep the default. Cost: the HTML5 drop fails on Windows [S7].
- Downloads: Studio needs a save path for the account export, the People export and the "Download to This Device" link [R14]. Options and costs:
  - Add an `on_download` handler that sets `destination` [S9]. Cost: the handler chooses a folder, and the macOS `Finished` path is empty [S9].
  - Add a save dialog plugin. Cost: a new plugin and a permission. Not verified in this note.
- Links to musicindex.org: options and costs:
  - Add the opener plugin and a click handler [S15]. Cost: a new plugin, a capability and a JavaScript change.
  - Keep `target="_blank"`. Cost: the default is Not verified.
- Audio: Studio plays MP3 and FLAC in an `<audio>` element [R14]. The Linux GStreamer packages are Not verified.
- Cost: a test on each Linux distribution is needed.
- A list of GStreamer packages in the install instructions is one option. Its content is Not verified.

### Question 4: lifecycle

- Current code: the process holds the API and the vault key in memory [R7]. Exit ends the API.
- A single-instance plugin [S16] prevents two processes on one data folder. Cost: a DBus path on Linux, and no Snap or Flatpak support by default [S16].
- The last-window behavior and the macOS reopen are Not verified in official docs. Options and costs:
  - Keep the Tauri default. Cost: the default is Not verified on each system.
  - Hide the window and keep the API on macOS. Cost: a `Reopen` path that is Not verified in 2.12.5.

### Question 5: packaging and the release workflow

- Current code: the workflow makes five archives and a draft Release [R11]. The smoke test reads the URL line from stdout [R12].
- Tauri adds bundles and installers [S20][S7]. Options and costs:
  - Keep the archives and add Tauri installers later. Cost: two release paths.
  - Replace the archives with Tauri bundles through `tauri-action@v1` [S14]. Cost: the draft job and the smoke test change. The attach behavior is Not verified.
- The smoke test does not test an installer [R12]. A test of the installed app is a new step.

### Question 6: signing

- Current code: the archives are unsigned [R11][the runbook].
- macOS: a Developer ID Application certificate and notarization are needed for a clean install outside the App Store [S11]. The Tauri page gives $99 per year. The Apple fee is Not verified against Apple [S24]. Cost: the fee and the account holder role [S11].
- Windows: options and costs are in Question 6. The prices are Not verified. A reputation can take time [S12].
- The unsigned path: macOS users use "Open Anyway" [S22]. Windows users accept the SmartScreen warning [S12]. Smart App Control can block the app [S12].

### Question 7: updater

- Current code: the release runbook uses manual replacement of the app folder [R7][runbook].
- The updater needs a key pair [S13], a `pubkey`, an endpoint and a signed manifest [S13]. Cost: the private key must stay safe, and a lost key blocks updates [S13].
- GitHub Releases can host the manifest [S13]. The `latest` URL behavior with draft Releases is Not verified. The Release process must publish a Release before users can update.

### Question 8: identifier and data folder

- Current code: the data folder is the `directories` folder [R7][S21]. A user with an archive has data in that folder on Linux and macOS [R7].
- Option 1: keep the `directories` path, and set the Tauri identifier to a stable reverse-DNS value. Cost: the Tauri path API is not used. No data move is needed [S17][S21].
- Option 2: use the Tauri `appDataDir` API. Cost: a migration from `directories` folders to the identifier folder. The migration needs a test on each system [S17].
- Option 3: use `appDirectoriesOverride` to keep the old path [S17]. Cost: the override is Not verified in this note for the exact path.
- The archive user keeps the collection and vault only when the folder does not change, or when a migration runs. The macOS fallback logic of R7 would need a rule for the Tauri identifier. This is a design decision.

## Facts that change the design most

1. Tauri 2.12.5 is the latest stable release, and its MSRV `1.90` fits the Studio pin [S1][S2][R8].
2. An external window to the axum server (Option A) needs no Tauri command and keeps ADR 0016 and ADR 0017 [S5][R2][R3]. The `dragDropEnabled` setting and `on_download` need code changes [S7][S9].
3. The origin of the window includes the port. The default `--port 0` changes the port at each start. The storage effect is Not verified, and a test is needed [R7][S8].
4. The `styles.css` features `:has()`, `color-mix()` and `dialog` have first versions for Safari, Chrome and Edge in the caniuse and MDN data. The WebKitGTK and WebView2 mappings are Not verified [S26][S28][S29].
5. Tauri's default data path is the identifier path. The current `directories` path differs. Keeping the `directories` path needs no migration [S17][S21].
6. macOS needs a Developer ID Application certificate and notarization for a clean install [S11]. The Tauri page gives $99 per year [S11]. The Apple fee is Not verified against Apple [S24].
7. Windows signing has three documented options, and SmartScreen reputation takes time [S12][S23]. The costs are Not verified.
8. The updater needs a key pair that the operator keeps safe [S13].
9. The Linux media codecs are Not verified. Each Linux distribution needs a test.

## Not verified list

- The year of the Tauri 2.12.5 release date (S1 shows "21 Oct" only).
- The MSRV of every dependency in the Tauri tree.
- The WebKitGTK runtime minimum and the Ubuntu 22.04 package version.
- The first version of each engine for `<dialog>`, `color-mix()`, `backdrop-filter`, `dvh` and `aspect-ratio`.
- The WKWebView version mapping, and the WebView2 Chromium version.
- The default behavior of `target="_blank"` in Tauri 2.
- The default save location of downloads without an `on_download` handler, and the save dialog plugin.
- The MP3 and FLAC support of each webview, and the GStreamer package names for each Linux distribution.
- The origin and storage rule of `localStorage` and `sessionStorage` across starts with a changed port.
- The Tauri CSP behavior for an external `http://127.0.0.1` window.
- The last-window behavior and the `RunEvent::Reopen` variant in Tauri 2.12.5.
- The clipboard secure-context rule in each webview.
- The Apple fee (the Tauri page gives $99 per year, and the Apple page did not show it).
- The Apple notarization steps (S25 returned only its title).
- The Windows certificate and Azure Artifact Signing prices.
- Whether `tauri-action` attaches to an existing draft Release, and the `uploadUpdaterJson` input.
- Whether a draft GitHub Release serves the `latest.json` URL.
- The Windows home folder result of `BaseDirs` and the Windows data path of the current code.
- The `WebviewUrl::External` behavior (S33 returned HTTP 404).

## Questions for the operator

1. Money: the Apple Developer Program fee and any Windows certificate or Azure Artifact Signing cost. The Tauri page gives $99 per year for Apple. The Apple fee and the Windows costs are Not verified.
2. The application identifier and the publisher name. The identifier decides the data folder if the Tauri path API is used, and it sets the native app identity [S17][R6].
3. The window model: Option A, B or C in Question 2. This changes ADR 0017 only under Option C.
4. The data folder: keep the `directories` path, or move to the Tauri identifier path with a migration.
5. The release model: keep the archives and add installers, or replace the archives with `tauri-action` bundles.
6. The updater key: who keeps the private key and where the CI secret is kept.
7. The Linux packages to support, and the GStreamer test on each distribution.
8. The macOS and Windows signing: whether an unsigned release is acceptable for the first installers.

## Checks Of The Planning Model

The planning model examined two items of this note on 2026-11-03, on this computer.

- The Windows data folder. `studio-desktop` uses `directories::ProjectDirs::from("", "", "musicindex-studio")` and `data_local_dir()` (`crates/desktop/src/main.rs:35`). In `directories` 6.0.0, `project_dirs_from_path` on Windows joins the project path to the known folder Local AppData, then `data` (`src/win.rs:67` to `src/win.rs:74`). The default folder is thus `%LOCALAPPDATA%\musicindex-studio\data`. The sentence of the distribution plan "Windows has no default path through this code" is from before milestone 1. It is not correct.
- The port and the browser storage. The default of `--port` is 0, so the system gives a different port at each start (`crates/desktop/src/main.rs:55`). The origin of a page includes its port, and `localStorage` is kept for each origin. Thus each start of the current desktop loses the theme and the view of the review that the UI keeps. This is a fault of the current shell, also in a browser.
