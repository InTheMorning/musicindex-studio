# Studio distribution phase plan

- Status: In progress
- Date: 2026-10-26
- Scope: desktop downloads first, then native installers and server packages

## Goal

An artist can download Studio, start it, and keep its data through an update.
A server operator can later install a supported server package without a Rust toolchain.
The release process must test each artifact on its target operating system.

## State before milestone 1

The desktop binary runs the local API and prints a browser address.
It has no native window or installer. [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md) assigns the native window to Tauri.
[Phase 13](studio-v1-phase-plan.md) assigns desktop packaging to the Desktop Shell phase.
The UI is in the binary, so an archive needs no separate UI files ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).

The desktop data path uses `HOME` or `XDG_DATA_HOME`. Windows has no default path through this code.
The library default and `~` expansion also use `HOME`. Some tests require Unix permissions, symbolic links, or `sshd`.

The workspace has Linux CI, but it has no release workflow, checksums, signing, or artifact smoke test.
The local Linux release binary links system `libz`, `libgcc_s`, and `glibc`.
The Ubuntu 22.04 runner sets the Linux compatibility floor for the first archives.

Phase 12 is in progress. Its server core, phone, and grant vault milestones are complete, and its runbook exists.
The deployment on `studio.musicindex.org` and the human tests remain open.
No server artifact is a production release in this plan until those gates pass.

## Milestone 1 status

The release workflow and five native build jobs are on `main`.
A manual run on `8e820e6` built, packaged, and tested the five archives on their native runners.
No workflow run has made a draft Release. The first release candidate tag is `v0.1.0-rc.1`.
The draft job marks a tag with a hyphen as a pre-release.

## Platform order

| Platform | First desktop milestone | Later package |
| --- | --- | --- |
| Windows x86-64 | Native runner build, smoke test, ZIP | Signed EXE or MSI |
| macOS Intel and Apple Silicon | Native runner build, smoke test, tarball | Signed and notarized DMG |
| Linux x86-64 and ARM64 | Native runner build, smoke test, tarball | AppImage, DEB, RPM, then Flatpak |
| Windows ARM64 | Evaluate after x86-64 passes | Native installer if dependencies and signer support it |

GitHub lists standard Windows, macOS, and Linux runners for these architectures in its [runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
The first milestone uses native runners for each listed archive.
An archive is an interim download. It does not meet the final one-click installation goal.

## Milestone 1: Desktop release drafts

1. Use the operating system's application data folder for a new desktop collection and vault.
2. Keep an existing macOS data folder available when the new default path changes.
3. Keep `--data-dir` for a user who selected a custom folder.
4. Build five desktop binaries on native GitHub runners.
5. Package each binary with the license and short start instructions.
6. Run each packaged binary on its build runner. Check its version, local UI, API access rules, and vault after restart.
7. Check that the Git tag matches the Cargo workspace version.
8. Give each artifact a name with the product, version, operating system, and architecture.
9. Put SHA-256 checksums and generated release notes in a draft GitHub Release.
10. Keep manual workflow runs as build checks with no Release.

A tag such as `v0.1.0` starts the draft release. A person reviews and publishes the draft.
The workflow does not push a tag or publish a Release on its own.
The release contains desktop artifacts only. It does not contain `studio-server`.

## Milestone 2: Native desktop installers

Start this milestone after Phase 12 and its human tests are complete.
Build the Tauri shell required by ADR 0002. Keep the same core, API, and UI.
Use Tauri's [distribution tools](https://v2.tauri.app/distribute/) for the native application and installers.
Test the installed app, its update path, and its removal on each target operating system.
An uninstaller must leave the user's collection, vault, and library available for a later install.

Add signed Windows installers after the signing method is ready.
Add macOS DMGs after Developer ID signing and notarization are ready.
Add Linux AppImage, DEB, and RPM packages after installation tests pass.
Evaluate Flatpak after the Linux packages, because its sandbox changes file and host access.

## Milestone 3: Server distribution

Wait for the open Phase 12 tasks and human tests.
Then build Linux x86-64 and ARM64 binaries, system packages, and one multi-architecture OCI image.
Test server startup, health, sign-in, persistent data, upgrade, and shutdown on each image architecture.
Use a non-root image user and a documented persistent data volume.
Evaluate Windows and macOS server binaries after the Linux server release works.
Evaluate FreeBSD and OpenBSD with native builders and test hosts before naming them supported targets.
Rust supplies host tools for FreeBSD x86-64 and ARM64 ([Rust FreeBSD target notes](https://doc.rust-lang.org/rustc/platform-support/freebsd.html)).
OpenBSD targets have Tier 3 support and no prebuilt Rust artifacts ([Rust OpenBSD target notes](https://doc.rust-lang.org/rustc/platform-support/openbsd.html)).
FreeBSD is the more practical first experiment. Neither BSD target has an available Studio build or smoke test.

## Signing and updates

The maintainer must supply a Developer ID Application certificate and notarization credentials for macOS.
The maintainer must supply a Windows code-signing service or certificate for trusted Windows installers.
Tauri documents [macOS signing](https://v2.tauri.app/distribute/sign/macos/) and [Windows signing](https://v2.tauri.app/distribute/sign/windows/).
No signing credential belongs in the repository, an artifact, a log, or a pull request.

Secure automatic updates need a signed Tauri update bundle, an update endpoint, and a recovery test.
The [Tauri updater](https://v2.tauri.app/plugin/updater/) gives that mechanism for the later native shell.
The interim archives use manual replacement. Their checksums help users check a download.

## Test strategy

Keep the four repository checks before each commit.
Run the release smoke test on the packaged binary, after the archive is made.
The smoke test must check the local UI, refuse an API request without its token, and keep a vault through restart.
The Linux runner can also run the full workspace tests with a local `sshd`.
Record each native runner result. A local Linux build does not verify a Windows or macOS artifact.

## Risks and rollback

An unsigned macOS or Windows archive can trigger an operating system warning.
A Linux binary can need a newer system library than an older distribution has.
Native installers need an application icon, a stable application ID, and removal checks.
The first milestone does not claim that those installers exist.

A failed draft Release stays unpublished. Fix the cause before publication.
A user can run the earlier binary with the same data folder after an application update fails.
The collection format still needs its own migration checks before a rollback across schema changes.

## Open questions

- Which application ID and publisher name will identify the signed native app?
- Which Windows signing service or certificate will the maintainer use?
- Which Linux distribution versions will the DEB and RPM packages support?
- Which operator will perform the installation and update tests on physical desktop systems?
