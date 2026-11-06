# Human Test Findings Of 0.1.0-rc.18

Date: 2026-11-05.

Status: advisory, and open. This review records the results of the human test of `0.1.0-rc.18` until now. It states no rule. A rule needs an ADR.

The operator runs the test on a Linux x86-64 computer, with the AppImage.

## Results

| Check | Result | Evidence |
|---|---|---|
| Play in the AppImage | Passed. The operator heard the audio. | The operator, and a screenshot with a moving position |
| The first play of each start | **Slow.** The operator thought that the app had crashed. | Finding 1 |
| The second track of a start | Starts immediately | The operator |
| A false error during the wait | **Failed** | Finding 2 |
| The other steps of the notes | Open | |

## Findings

### 1. The First Play Of Each Start Waits

Corrected for the next candidate (ADR 0124 item 26). The log shows a scan of each GStreamer plugin before the first play. rc.18 held 244 plugins, because the bundler copies each plugin of the runner.

On a virtual screen, the real rc.18 file took 4.72 s from the press of Play to the audio sink. With 25 plugins and the same libraries, it took 0.41 s. That check stops at the sink, because the sandbox has no sound device. No CI run has built the shorter list, and no test has heard it.

### 2. A False Error During The Wait

Corrected for the next candidate. The player showed "Studio cannot play this file" while the audio started, and the text stayed after the audio started. The player now shows "Starting…" during the wait, and it clears an error when the audio starts. A Chromium check passed. The check forced the error, so the cause of the first error in WebKit is not known.

### 3. A Long Title Widened The Left Panel

Corrected for the next candidate. A long release title gave the left panel a scrollbar and no ellipsis.

### 4. The Dot Of A Head

Corrected for the next candidate (musicindex ADR 0038 R-DOT-4). A head with a dot is more clear.
