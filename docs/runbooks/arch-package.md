# The Arch Package Of The Desktop App

This runbook tells how to build and install `musicindex-studio-git` on Arch Linux. [ADR 0216](../adr/0216-the-linux-packages-of-the-desktop-app.md) owns the package. No release workflow builds it (ADR 0216 item 6).

## Before You Start

- Install `base-devel` and `git`: `sudo pacman -S --needed base-devel git`.
- The build needs approximately 10 GB of disk space and 30 minutes or more.

## Build And Install

1. Clone the repository that has the PKGBUILD:

   ```bash
   git clone https://github.com/InTheMorning/musicindex-studio.git ~/musicindex-studio-pkg
   cd ~/musicindex-studio-pkg/packaging/arch
   ```

2. Build and install the package. `makepkg` downloads the source from GitHub, installs the Rust toolchain of `rust-toolchain.toml` with `rustup`, builds the app and runs its tests:

   ```bash
   makepkg -si
   ```

3. Start "Music Index Studio" from the application menu, or run `musicindex-studio-desktop`.

## Optional Packages

- `xdg-utils`: Studio uses `xdg-open` to open a link in the browser. Without it, Studio shows a dialog with the address.
- `gst-libav`: plays AAC and the other formats of the libav plugin.

Install them with `sudo pacman -S --needed xdg-utils gst-libav`.

## Update

The package follows `main`. To update, do steps 2 and 3 again in the same folder. `makepkg` downloads the new commits.

## Remove

```bash
sudo pacman -R musicindex-studio-git
```

The data folder of Studio stays. The [desktop runbook](install-desktop.md) gives its location.
