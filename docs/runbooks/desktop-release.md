# Desktop release drafts

## Purpose

The desktop workflow builds five archives and nine installers from a version tag.
It tests each archive on a native GitHub runner and creates a draft GitHub Release.
The installers are the AppImage, DEB and RPM of each Linux system, the DMG of each macOS system and the MSI of Windows ([ADR 0124](../adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md)).
The release has no server binary. The server shell is not ready for public use.

## Before a release

1. Make sure the four checks in `AGENTS.md` pass.
2. Set the workspace version in `Cargo.toml` and commit it with `Cargo.lock`.
3. Write the notes of the version in `docs/releases/VERSION.md` and commit them ([ADR 0100](../adr/0100-each-release-candidate-has-a-notes-file.md)). Use the open human tests of the Studio phase plan for "What to test".
4. Run the `Desktop release` workflow by hand to check all five runners.

A manual run uploads archives and installers as workflow artifacts. It does not create a GitHub Release.
The workflow needs no repository secret for these unsigned archives.

## Make a draft

1. Push a tag that matches the workspace version, such as `v0.1.0`.
2. Wait for all five build and smoke jobs to pass.
3. Open the draft GitHub Release and check its notes, its 14 files, and `SHA256SUMS.txt`. The 14 files are five archives and nine installers. The notes are the notes file of the version, then the text about the files, the server image and unsigned apps.
4. Test each archive on a normal desktop system before you publish the draft.
5. Publish the draft only after the remaining release checks pass.

The workflow refuses a tag that does not match `Cargo.toml`.
For a release candidate, run `tools/release-candidate.sh 0.1.0-rc.10`. It stops when `docs/releases/0.1.0-rc.10.md` is not committed. Then it sets the version, pushes `main`, waits for CI and pushes the tag `v0.1.0-rc.10`.
The workflow marks a draft from a tag with a hyphen as a pre-release.
It keeps the Release as a draft. It does not publish it.
If a draft already exists, a new run replaces its assets and its notes after all five jobs pass.
A run refuses to change a published Release.

The smoke test extracts the archive, runs the binary, and checks the local UI.
It also checks the API token, the loopback Host rule, and vault data after a restart.
The installers have a file check, but no start test. The workflow examines the start bytes of each installer and gives it a fixed name. Test each installer on a normal desktop system.

The bundler refuses a pre-release version in the RPM and the MSI. The command `python tools/release_desktop.py bundle-config FORMAT` gives the replacement version for the build.

## Data and updates

The default collection folder uses the native local application data folder.
On macOS, Studio uses an old `~/.local/share/musicindex-studio` folder when it holds data and the new folder does not.
The library default stays in `~/musicindex/studio` under ADR 0036.
An archive update needs manual replacement of the application folder.
Keep the data folders during replacement and removal.

## Signing and later packages

The maintainer must supply a macOS Developer ID Application certificate and Apple notarization credentials.
The maintainer must also supply a Windows code-signing certificate or signing service.
Keep each credential in GitHub Actions secrets or a protected signing service. Do not put it in source or logs.

For macOS CI, provide `APPLE_CERTIFICATE` with an exported `.p12` file and `APPLE_CERTIFICATE_PASSWORD`.
For notarization, provide `APPLE_ID`, an app-specific `APPLE_PASSWORD`, and `APPLE_TEAM_ID`.
An App Store Connect API key is another notarization method.
See the [Tauri macOS signing guide](https://v2.tauri.app/distribute/sign/macos/) before you configure these secrets.

For Windows, select a certificate or Azure Artifact Signing account before you define its CI secrets.
The credential names depend on that choice. A new signature can still show a SmartScreen warning.
See the [Tauri Windows signing guide](https://v2.tauri.app/distribute/sign/windows/).

Choose a stable application ID and publisher name before native installers are made.
Tauri signing and its signed updater belong to the later native desktop shell.
The updater needs a private signing key in CI, its public key in the app, and an update endpoint.
Keep `TAURI_SIGNING_PRIVATE_KEY` and its optional password secret. See the [Tauri updater guide](https://v2.tauri.app/plugin/updater/).
Test installation, removal, and update recovery before enabling automatic updates.

## Failure and recovery

If a build or smoke job fails, do not publish the draft.
Fix the cause and run the workflow again. Check the new checksums before publication.
If an update fails, replace the application folder with the earlier binary.
Check the collection schema before you use an older binary with newer data.
