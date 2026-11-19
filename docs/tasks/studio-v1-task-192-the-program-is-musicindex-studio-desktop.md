# Studio v1 Task 192: The Program Is musicindex-studio-desktop

Decision: [ADR 0226](../adr/0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) items 5 and 6.

Status: Open.

## Problem

Finding 6 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). The installed program is `studio-desktop-app`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Cargo binary | extend | crates/desktop-app/Cargo.toml | The `[[bin]]` name near line 11. The package name stays. |
| Arch package | extend | packaging/arch/PKGBUILD | The install line near line 98. |
| Arch desktop file | extend | packaging/arch/musicindex-studio.desktop | `Exec` and `StartupWMClass`. |
| Runbook | extend | docs/runbooks/arch-package.md | Line 25 names the program. |
| Desktop app tests | extend | crates/desktop-app/src/main.rs | The test module has source guards, for example `the_opener_plugin_adds_no_link_script`. |

## Changes

1. The `[[bin]]` name is `musicindex-studio-desktop`.
2. The PKGBUILD installs `target/release/musicindex-studio-desktop`.
3. The desktop file has `Exec=musicindex-studio-desktop` and `StartupWMClass=musicindex-studio-desktop`.
4. The runbook names the new program. The log prefix `studio-desktop-app:` of `main.rs` and `renderer.rs` becomes `musicindex-studio-desktop:`.
5. A test reads `Cargo.toml` and the desktop file. The binary name, `Exec` and `StartupWMClass` must agree.

## Checks

Mechanical:

- `cargo test -p studio-desktop-app` and `bash tools/gate.sh` pass.
- A local `makepkg -d` in a copy of `packaging/arch` installs `usr/bin/musicindex-studio-desktop`.
- A local AppImage build has `usr/bin/musicindex-studio-desktop`.

Human: the operator installs the Arch package and starts Music Index Studio from the application menu. The window list shows the app icon.
