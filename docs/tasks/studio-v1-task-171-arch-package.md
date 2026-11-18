# Studio v1 Task 171: The Arch Package Of The Desktop App

Decision: [ADR 0216](../adr/0216-the-linux-packages-of-the-desktop-app.md), which cites musicindex ADR 0084.

Status: Complete on 2026-11-18 for the mechanical checks. The two checks by a person in ADR 0216 §Checks are open.

## Problem

Studio had an AppImage, a `.deb`, an `.rpm`, a `.dmg` and an `.msi`, but no PKGBUILD (musicindex ADR 0084 R-PACKAGE-1). When no program opened a link, the app showed nothing (R-PACKAGE-4 item 2). The `.deb` and the `.rpm` did not name `xdg-utils` (R-PACKAGE-4 item 3).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| PKGBUILD form | reuse | /home/citizen/build/HitIt/packaging/arch/PKGBUILD | The Git package, `pkgver` and the `rustup` toolchain step follow it. |
| Link open | extend | crates/desktop-app/src/main.rs | `open_outside` shows a dialog when the open fails. |
| Message dialog | reuse | crates/desktop-app/src/main.rs | `tauri-plugin-dialog`, as the other dialogs of the app use it. |
| Package recommends | extend | crates/desktop-app/tauri.conf.json | The Tauri bundler writes `Recommends` for the `.deb` and the `.rpm`. |
| Desktop file | new | | Searched crates/desktop-app, ../HitIt/packaging/arch and ../stophammer/packaging/arch: the Tauri bundler writes the desktop file only in its own packages. |

## Changes

1. `packaging/arch/PKGBUILD` builds `musicindex-studio-git` (ADR 0216 items 1, 2, 3 and 5).
2. `packaging/arch/musicindex-studio.desktop` starts `studio-desktop-app`.
3. `open_outside` shows a Warning dialog with the address and the package `xdg-utils` when the open fails (ADR 0216 item 4).
4. `tauri.conf.json` lists `xdg-utils` in `recommends` of the `.deb` and of the `.rpm` (ADR 0216 item 3).
5. `docs/runbooks/arch-package.md` tells how to build and install the package.

## Checks

Mechanical:

- A local `makepkg -d` with the PKGBUILD of this task. The result is in §Build Result.
- `tools/gate.sh` passes.

Visual, for a person:

- The two checks in ADR 0216 §Checks. Open.

## Build Result

On 2026-11-18, `makepkg -d` built `musicindex-studio-git-0.1.0rc.26.r1.g8bc7895-1-x86_64.pkg.tar.zst` (16 MB) from a local clone with the PKGBUILD of this task. The sandbox has no `pacman.conf`, so `-d` skipped the dependency check.

- `build()`: the release build finished in 4 min 51 s.
- `check()`: 23 tests passed, 0 failed.
- The package has `usr/bin/studio-desktop-app`, the desktop file, the 512 px icon, `LICENSE` and `README.md`. `.PKGINFO` has the `depends`, the `optdepends` and the `makedepends` of the PKGBUILD.
- `desktop-file-validate` accepts `packaging/arch/musicindex-studio.desktop`.
- makepkg gave one warning: "Package contains reference to $srcdir". The two references are the engines folder and the modules folder of the bundled OpenSSL (`openssl-sys`). Studio loads no OpenSSL engine and no OpenSSL module, so the references have no effect.
- Nobody installed the package and started the app. That is the second check by a person in ADR 0216 §Checks.
