# Human Check Of 0.1.0-rc.27 On 2026-11-19

The operator did the steps of "What to test" in the [notes of 0.1.0-rc.27](../releases/0.1.0-rc.27.md). The app was a local AppImage of `98d8bfa`, which has rc.27 and the four commits after it.

## Results

| Step | Result |
| --- | --- |
| 1. Upgrade to schema version 44 on the real collection | Pass. Releases and People showed all feeds and persons. |
| 2. "Choose Folder…" in the first-run sheet | Pass. The dialog opened at the home folder, and the selected folder filled the field. |
| 3. The result of a write in Review | Open. The operator found findings 2 and 3 during this step. |

## Findings

1. The dialog of "Choose Folder…" starts at the home folder also when the field has a typed folder. ADR 0223 item 10 gives this start folder. Open: the recommendation is to start at the typed folder when it exists. The operator has not decided.
2. Defect: the row "Nostr profile" grew from 44 px to 52 px when the profile picture came. Operator choice: correct it. Now a key row keeps a 32 px place for the picture while it loads. The row "Artwork" does the same. Walk `.studio-dev/journeys/v225b.json` (mode `base`): the row with a key is 52 px before the profile comes.
3. The first tab of a release showed the kind, for example "Single", and its first section had the heading "Album". [ADR 0225](../adr/0225-the-first-tab-of-a-release-is-release.md) gives the tab "Release" and the heading of the kind. Walks `.studio-dev/journeys/v225a.json` and `v225b.json` (mode `base`).

## AppImage Build On Arch

The local build of the AppImage needs `patchelf`. Give these variables to `cargo tauri build` in `crates/desktop-app`:

- `GSTREAMER_PLUGINS_DIR` from `tools/release_desktop.py appimage-plugins --source /usr/lib/gstreamer-1.0`
- `GSTREAMER_HELPERS_DIR=/usr/lib/gstreamer-1.0`
- `NO_STRIP=true`
- `APPIMAGE_EXTRACT_AND_RUN=1`
