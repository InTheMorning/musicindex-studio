# Operator Human Checks

This runbook gives each open human check of Studio as a command block. It also gives the items that only the operator can supply, and the decisions that wait for the operator.

This runbook makes no rule. Each check names its owner:

- The [notes of 0.1.0-rc.20](../releases/0.1.0-rc.20.md), "What to test".
- [ADR 0128](../adr/0128-a-moved-feed-names-its-own-address.md) and [task 091](../tasks/studio-v1-task-091-own-address-tag.md).
- [ADR 0119](../adr/0119-a-new-user-starts-as-a-musician-or-a-label.md), [ADR 0154](../adr/0154-the-search-fields-have-a-field-list.md) and [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md).
- The [visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md).

When a check passes, record the result in its owner, and remove the block here.

The agent screenshot checks of these changes passed in Chromium. These blocks hold only the steps that need the operator: the real AppImage, WebKit, the iPhone, real file hosts, the network and a deploy.

## State On 2026-11-09

- Local `main` has commits that are not on `origin/main`. The local tag `v0.1.0-rc.20` is on `e882cd9`, which is `origin/main`.
- The commits after the tag hold ADR 0156 to ADR 0160, the glow of the left panel and the correction of the sort menu. The rc.20 release does not hold them. Decision 2 below is about this.
- The agent environment has no GitHub login and no SSH access to GitHub. The operator does each push, each release and each deploy.

## Before The Checks

Run these commands one time. They set the variables that the blocks use.

```sh
cd ~/build/musicindex-studio
export STUDIO_REPO="$PWD"
export STUDIO_VERSION=0.1.0-rc.20
```

## Check A: The AppImage Of The Release Candidate

Owner: the notes of 0.1.0-rc.20, steps 1 to 7 and 15 to 18. This check also covers WebKit, because the Linux app shows its pages in WebKitGTK.

Necessary: the draft release of `v0.1.0-rc.20` (decision 2), and a copy of your data folder.

```sh
# 1. Keep a copy of your data. The first start moves the data into collections/main/.
cp -a ~/.local/share/musicindex-studio ~/musicindex-studio-backup-$(date +%F)

# 2. Get the AppImage of the draft release.
mkdir -p ~/studio-rc && cd ~/studio-rc
gh release download "v$STUDIO_VERSION" --repo InTheMorning/musicindex-studio \
  --pattern "*linux-x86_64.AppImage" --clobber
chmod +x musicindex-studio-v$STUDIO_VERSION-linux-x86_64.AppImage

# 3. Start it with a log of the audio. Measure the time from Play to the sound.
GST_DEBUG=2 ./musicindex-studio-v$STUDIO_VERSION-linux-x86_64.AppImage 2> play-log.txt

# 4. After the app starts, examine the moved data.
ls ~/.local/share/musicindex-studio/collections/main/
```

Look at these items, and expect these results:

1. Your feeds, your people and your vault are in the app.
2. `ls` shows the data in `collections/main/`.
3. In **Settings**, **Collections**, make a new collection and select **Open**. The app restarts on the Welcome page, with the blue row in the left panel.
4. Open **Main collection** again. Your feeds are there.
5. Start a real upload. During the upload, select **Open** on a different collection, then **Cancel**. Nothing changes.
6. Press Play on a track. Write down the seconds until you hear the sound. Send that time and `~/studio-rc/play-log.txt`.
7. A left click on the tray icon opens the window. The first close of the window asks one question, one time.
8. On a computer with an NVIDIA card, the first page loads fast (decision 5).

If data is missing at item 1 or 2, stop. Do not remove the copy.

## Check B: WebKit On The Current Main Branch

Owner: the visual checks of 2026-11-08, section "Limits". The screenshots of ADR 0152 to ADR 0160 ran only in Chromium. The rc.20 release does not hold ADR 0156 to ADR 0160, so this check makes an AppImage from local `main`.

Necessary: `cargo tauri` and `patchelf`. The first command installs the Tauri CLI at the version of the release workflow.

```sh
cd "$STUDIO_REPO"
cargo install tauri-cli --version 2.12.4 --locked
command -v patchelf || sudo pacman -S --needed patchelf

cd crates/desktop-app
GSTREAMER_PLUGINS_DIR="$(python ../../tools/release_desktop.py appimage-plugins \
  --source /usr/lib/gstreamer-1.0 --output "$PWD/../../.studio-dev/gst-plugins")" \
  cargo tauri build --bundles appimage -- --locked

# Start the local AppImage on a copy of your data, not on your real data.
mkdir -p "$STUDIO_REPO/.studio-dev/webkit-check-home"
cp -a ~/.local/share/musicindex-studio "$STUDIO_REPO/.studio-dev/webkit-check-home/"
XDG_DATA_HOME="$STUDIO_REPO/.studio-dev/webkit-check-home" \
  "$STUDIO_REPO"/target/release/bundle/appimage/*.AppImage
```

The copy has your vault and can write to your real feeds. Do not write a feed from this copy.

Look at these items, and expect these results:

1. **Releases**, the sort menu: the menu opens below its button, in the window.
2. A feed page: the page tabs are one segmented control (ADR 0157).
3. **People**, a person: the list "where the person is" shows the split of each track (ADR 0156).
4. A search field of Music Index: a click opens the field list. Type `ki`. The list shows `kind:`. Up, Down, Enter and Escape work (ADR 0154).
5. The left panel has the accent glow. With **Increase Contrast** on, it has no glow.
6. No text overlaps a control, and no control is outside the window.

## Check C: The iPhone And The Server

Owner: the notes of 0.1.0-rc.20, steps 19 to 22. Safari on the iPhone is also WebKit.

Necessary: the server image of `v0.1.0-rc.20` (decision 2), and access to the server.

```sh
cd "$STUDIO_REPO"
# Makes the candidate, waits for CI and the image, then deploys it.
tools/release-candidate.sh "$STUDIO_VERSION"
```

Look at these items on the iPhone, and expect these results:

1. Sign in. Your account and your feeds are there.
2. **Settings** has no **Collections** section.
3. **Releases**: the sort menu shows **Oldest** and **Z to A**. A feed page shows its tabs as one select menu.
4. **People**: the sort menu gives **Z to A**, then **Newest**.
5. A credit sheet, **Find on Music Index**: no list shows before you tap the field. Type `ki`. The list shows `kind:` above the keyboard. Tap `kind:`. The keyboard stays open and the values show.

## Check D: A Feed Return Between Two Real File Hosts

Owner: ADR 0128, human criterion, and task 091. The agent check of 2026-11-08 sent the bytes that Studio writes to a copy of Stophammer at `398d10f`, and each result was correct. This check uses the live index and two real file hosts.

Necessary: a test release on file host A, and a second real file host B in the same collection. Use the AppImage of check A or `cargo run -p studio-desktop`.

```sh
# Put the GUID of the test feed here. The Files page of the feed shows it.
GUID=put-the-feed-guid-here
index() { curl -s "https://api.musicindex.org/v1/feeds/$GUID" | python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["feed_url"])'; }

index   # Before: the address on host A.
# In Studio: Files, Move This Release..., host B, 7 days. Review, then Write.
index   # After the move: the address on host B.
# In Studio: Files, Move This Release..., host A. Review shows the return warning. Write.
index   # After the return: the address on host A, with no wait.
```

Look at these items, and expect these results:

1. Review of the return shows the warning below the move row. The first move has no warning.
2. Each `index` call after a write gives the new address.
3. The feed and each file play from the host of each step.

If an address does not change, wait 10 minutes and run `index` again. Then send the output of each `index` call.

## Check E: Pictures From Outside Hosts

Owner: ADR 0159 human criterion, and the notes of 0.1.0-rc.20, step 23. The sandbox of the agents cannot load pictures from outside hosts, so the screenshots show initials or an empty circle.

```sh
cd "$STUDIO_REPO"
cargo run -p studio-desktop -- --data-dir .studio-dev/data
# Open the printed address in your browser.
```

Look at these items, and expect these results:

1. **People**, a person, **Find on Music Index**: each result row shows its picture.
2. The suggestions of a person show the picture adjacent to **Use**.
3. A release with a real Nostr key: its picture and name show after a short time.
4. Stop the network. The page stays usable.
5. Link a person to the page of a collaborator. After the collaborator changes a picture, open the person again. The new picture shows.

## Check F: The First-Visit Search

Owner: ADR 0119 and ADR 0154. The visual check of 2026-11-08 used data with feeds, so the Welcome page did not show. An agent can also run this check with a screenshot, because it needs no device.

```sh
cd "$STUDIO_REPO"
DATA="$(mktemp -d "$PWD/.studio-dev/first-visit.XXXX")"
cargo run -p studio-desktop -- --data-dir "$DATA"
# Open the printed address in your browser.
```

Look at these items, and expect these results:

1. Give a library folder in `.studio-dev/`. The Welcome page shows.
2. Select **My music is online already**. A click in the search field opens the field list.
3. Type `artist:` and two letters. The list shows artist names from Music Index.
4. Enter with no selected row runs the search, and the results show.

## What Only The Operator Can Supply

| Necessary item | Why | Supply |
| --- | --- | --- |
| A GitHub login, or the operator, for each push and release | The agent environment has no GitHub login and no SSH access. A reader with no login cannot see the draft release. | Check C runs `tools/release-candidate.sh`. |
| The iPhone | It is the only phone. No agent can operate Safari on a phone. | Check C |
| A real Linux desktop with sound and a tray | The sandbox has no sound device, no `/etc/os-release` and no tray. A local AppImage stops there. | Checks A and B |
| Two real file hosts with secrets | A real move uses the vault and the hosts of the operator. | Check D |
| The network to outside picture hosts and relays | The sandbox of the agents stops these requests. | Check E |
| Empty data for the first visit | The check data has feeds, so the Welcome page does not show. | Check F gives an empty folder. An agent can do this check. |
| A computer with an NVIDIA card | Only a computer with that card can show the restart of the app. | Check A, item 8 |
| A macOS or Windows computer | Only a tester with those systems can examine their installers. | None. These tests are blocked. |

## Decisions For The Operator

1. **The empty `package.json` at the root.** The orchestrator reported an empty, untracked `package.json` at the repository root. The Node test runner reads it and stops. The session that wrote this runbook did not see the file in its sandbox, so no agent examined its content.

   Other empty files of the same date are at the root: `.env*`, `.npmrc`, `.yarnrc*`, `bunfig.toml`, `.gitmodules` and the lock files. `.git/info/exclude` hides them. A sandbox tool possibly makes them. This cause is not verified.

   Recommendation: remove `package.json` after you look at it. If it comes back, tell the agents, so that they can find the tool that makes it.

   ```sh
   cd ~/build/musicindex-studio
   ls -la package.json && cat package.json   # Look first. Expect 0 bytes.
   git status --short --ignored package.json # Expect "!!" (ignored) or "??" (untracked).
   [ -f package.json ] && [ ! -s package.json ] && rm package.json
   node --test crates/api/ui-tests/*.test.mjs  # Expect "# fail 0".
   ```

2. **The content of the rc.20 release.** The tag holds the commits to `e882cd9`. The commits after it hold ADR 0156 to ADR 0160.

   Recommendation: release rc.20 at its tag, as its notes describe it. Put these commits in rc.21, with a new notes file. Check B covers their WebKit check before rc.21. First, look for the tag on GitHub:

   ```sh
   git -C ~/build/musicindex-studio ls-remote --tags origin v0.1.0-rc.20
   ```

3. **The test feed and the hosts of check D.** Recommendation: use a test release that no listener uses, on two hosts that you can clean. A real feed gets the wait of ADR 0083 after each move.

4. **The local commits.** Recommendation: push `main` with check C. `tools/release-candidate.sh` pushes `main` before the tag.

5. **The NVIDIA step.** Recommendation: if you have no computer with an NVIDIA card, record step 18 as blocked, as the Android tests are.

## Older Open Human Tests

The notes of 0.1.0-rc.20, "Known gaps", list the older open human tests. They are Phase 12, tasks 065 to 084, 0.1.0-rc.10 to 0.1.0-rc.19 and the server part of task 086. Their owners give their steps. This runbook does not give them again.
