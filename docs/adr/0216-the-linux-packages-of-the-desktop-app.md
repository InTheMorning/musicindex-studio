# ADR 0216: The Linux Packages Of The Desktop App

- Status: Accepted
- Date: 2026-11-18
- Owner: operator
- Accepted: 2026-11-18. The orchestrator session relayed the request under musicindex ADR 0084 R-PACKAGE-5, which the operator accepted on 2026-11-18.
- Bar: crosses repositories. musicindex ADR 0084 R-PACKAGE-5 asks each repository for its own packaging record.
- Uses: musicindex ADR 0084 (the Linux packages of each suite app). This record cites its rules and does not write them again.
- Uses: [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) item 26 (the GStreamer plugins of the AppImage).

## Context

- `.github/workflows/desktop-release.yml` builds an AppImage, a `.deb`, an `.rpm`, a `.dmg` and an `.msi` with the Tauri bundler. Studio had no PKGBUILD.
- An **external program** is a program of the host that Studio starts. This is the term of musicindex ADR 0084 R-PACKAGE-4.
- Examined on 2026-11-18, in the code outside the tests:
  - `crates/desktop-app/src/main.rs` `open_outside` opens a link through `tauri-plugin-opener`. On Linux, the `open` crate 5.4.5 starts `xdg-open`, then `gio`, `gnome-open` or `kde-open`.
  - `crates/desktop-app/src/renderer.rs` starts the app binary again. It is not an external program.
  - The SFTP connector uses the libssh2 library. It starts no program.
- The rc.24 AppImage has one program in `usr/bin`: `studio-desktop-app`. It has no `xdg-open`. The bundler `tauri-bundler` 2.10.4 of `tauri-cli` 2.12.4 says: "Bundling xdg-open in an AppImage does not work and therefore was disabled."
- The AppImage has 25 GStreamer plugins and the two helpers `gst-plugin-scanner` and `gst-ptp-helper`. WebKitGTK loads the plugins as libraries, and GStreamer starts its helpers. Studio starts none of them.

## Decision

1. The PKGBUILD is `packaging/arch/PKGBUILD`. It builds the package `musicindex-studio-git` from the Git source with `makepkg`. It installs `studio-desktop-app`, a desktop file and the app icon.
2. The PKGBUILD builds with the feature `tauri/custom-protocol`, as `cargo tauri build` does.
3. R-PACKAGE-4 applies to `xdg-open`. The PKGBUILD lists `xdg-utils` in `optdepends`. `crates/desktop-app/tauri.conf.json` lists `xdg-utils` in `recommends` of the `.deb` and of the `.rpm`.
4. When no program opens a link, the app shows a dialog with the address and the package `xdg-utils` (R-PACKAGE-4 item 2).
5. R-PACKAGE-4 does not apply to the GStreamer plugins and their helpers, because Studio does not start them. The AppImage keeps them (ADR 0124 item 26). The PKGBUILD lists `gst-plugins-base` and `gst-plugins-good` in `depends`.
6. No release workflow builds the PKGBUILD. A user builds it with `makepkg` on the Arch computer (`docs/runbooks/arch-package.md`). The GitHub runners of `desktop-release.yml` are Ubuntu, macOS and Windows runners. An Arch build needs a separate container job. A later record can add it.
7. The `.rpm`, the `.dmg` and the `.msi` stay (R-PACKAGE-3).

## Alternatives

- A versioned PKGBUILD from the source archive of each tag. It needs a checksum change for each release. The Git package follows `main` with no change, as the PKGBUILD of HitIt does.
- An Arch job in `desktop-release.yml`. It adds a container job and an artifact that no user asked for. Item 6 keeps it open.

## Consequences

- The `.deb` and the `.rpm` of rc.27 and later have the `Recommends` line. Nobody installed them on a host with no `xdg-utils`.
- The `Exec` line of the desktop file and the binary name are the same as in the `.deb`: `studio-desktop-app`.

## Checks

Mechanical:

- A local `makepkg -d` builds the package and runs the tests of `studio-desktop-app` (task 171).
- The manual check for R-PACKAGE-4 item 3: `packaging/arch/PKGBUILD` has `xdg-utils` in `optdepends`, and `tauri.conf.json` has `xdg-utils` in `recommends`. No test examines the two files, because no change has removed the lines yet. The global rule "Prefer Guards To Prose" asks for a test after the first incident.

Visual, for a person:

- On a Linux host with no `xdg-open` and no `gio`, select a link that opens outside the app. The dialog shows the address and `xdg-utils` (musicindex ADR 0084 R-PACKAGE-4 item 2).
- Install the Arch package, and start Music Index Studio from the application menu. The window shows the UI, and a track plays.
