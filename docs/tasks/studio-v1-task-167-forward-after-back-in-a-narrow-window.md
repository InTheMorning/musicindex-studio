# Studio v1 Task 167: Forward After Back In A Narrow Window

Decision: ADR 0206 item 11 (the narrow stack and its history). This task is an investigation. It found no defect, so it has no code change and no new decision.

Status: Complete on 2026-11-17. The real-device step 10 of the rc.24 notes stays open.

## Problem

The known gaps of the rc.24 and rc.25 notes say: "In the screenshot drive, Forward did not move after a Back in a narrow window. The cause is not known."

## Source Of The Gap

The note comes from the screenshot check of task 118d. That task file says: "After **Back** and **Forward**, the sheet did not open again." The drive closed the sheet "From Music Index: Ann Bell Music" with `dialog.close()`, then clicked Back, then Forward. Its screenshot `forward-dark-390.png` shows the page of Ann with Forward not available. That is the state after a Forward that moved.

A sheet is not a history entry, so Forward does not open it again. Inference: the rc.24 notes changed "the sheet did not open again" into "Forward did not move". The run of 2026-11-12 recorded no state after Back, so this inference cannot be proved from that run.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Walk harness | reuse | .studio-dev/walk/run.sh | A copy with a different DevTools port, in the scratchpad of the session. |
| Walk data with people | reuse | .studio-dev/walk/setup-118d.py | It has the person Ann Bell. |
| History model | reuse | crates/api/ui/shell.js | Examined, not changed. |

## Drives Of 2026-11-17

The drives use `.studio-dev/walk/setup-118d.py` and a copy of `run.sh` with a different DevTools port.

| Drive | Steps file | Result |
| --- | --- | --- |
| Load at 1280 px, People, Ann Bell, width 390 px, Back, Forward | `.studio-dev/walk/steps-167-width-change.json` | Back opens People. The Forward button opens Ann Bell. |
| The same, then Back and `history.forward()` | the same file | The browser forward does not move. `history.state.mirrored` is `false`. |
| Load at 390 px with touch, Album 2, Back, `history.forward()` | `.studio-dev/walk/steps-167-narrow-load.json` | Back opens Releases. The browser forward opens Album 2. `mirrored` stays `true`. |

## Result

- The Forward button moves after Back, at 390 px and after a width change.
- After a load at a narrow width, the browser forward also moves. A swipe forward on a phone uses the same browser forward. The drive used Chromium, not Safari.
- After a change of width across 760 px, the browser Back and Forward do not follow the stack until the next load. The effect of `narrow` in `Shell` (`crates/api/ui/app.js`) sets `mirrored` to `false` for this function. Its comment gives the cause: the history entries before the change belong to the state of the other width. Back then moves the stack in place, and Forward adds a history entry.
- A phone that turns from portrait to landscape can go across 760 px. An iPhone in landscape is 844 px wide. After that change, the swipe forward does not move until the next load. This is the design of the effect above, not a fault of the drive.
- Step 10 of the rc.24 notes stays a real-device check, because only Safari on a phone can prove the swipe.
