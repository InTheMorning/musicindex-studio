# Studio v1 Task 169: The Slow Scroll Of The Review Sheet

Decision: none. This task is an investigation. It found no CSS cause, so it has no code change and no new decision.

Status: Complete on 2026-11-17 as an investigation. The cause on the computer of the operator is not known.

## Problem

The [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), check 4, step 2, records this fault: a scroll in the review sheet is slow, and `WebKitWebProcess` uses the most CPU. A fast scroll mode or the tilt wheel makes it much worse. The notes of rc.24 and rc.25 give it as a known fault.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| WebKitGTK drive | extend | .studio-dev/wk-drive.py | A copy with frame time steps, in .studio-dev/task-169/wk169.py. |
| Walk data | reuse | .studio-dev/v-116a-117/data | The script copies it and adds a long draft of Album 2. |
| Check script | new | .studio-dev/task-169/t169.sh | Searched .studio-dev/walk and tools: no script measures frame times in WebKitGTK. |

## Method

1. `.studio-dev/task-169/setup-169.py` gives Album 2 a draft with 300 new tracks. Each track has a value block with three recipients. The draft has 3615 lines.
2. `.studio-dev/task-169/t169.sh` serves Studio on a copy of the walk data. It opens the page in WebKitGTK 4.1 on Xvfb `:98`, with `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
3. For each CSS variant, the drive loads a new page, opens the review sheet and adds the variant as a style element.
4. The drive scrolls the scroll area by 60 px at each animation frame for 3 seconds. Then it scrolls by 240 px for 3 seconds. It records the time between frames. A frame time of 17 ms is 60 frames each second.
5. It does steps 3 and 4 for the "Simplified" view (2412 elements) and for the "Detailed" view (3606 diff lines, 7208 elements).

## Result

| Variant | Simplified, first scroll, p90 | Detailed, first scroll, p90 | Second scroll, p90 |
| --- | --- | --- | --- |
| No change | 34 to 40 ms | 31 to 37 ms | 17 to 18 ms |
| No `box-shadow` on the sheet | 36 ms | 27 ms | 17 ms |
| No `::backdrop` background | 36 ms | 27 ms | 17 ms |
| No `filter` and no `backdrop-filter` | 31 to 48 ms | 29 to 35 ms | 17 to 18 ms |
| `contain: strict` on the scroll areas | 35 ms | 29 ms | 17 ms |
| `content-visibility: auto` on the rows | 35 ms | 29 ms | 17 ms |

- The first scroll over new rows falls to approximately 30 frames each second at p90. A second scroll over the same rows stays at 60 frames each second, also with the fast step of 240 px.
- No CSS variant changes the first scroll more than the difference between two runs of the same variant. The shadow, the backdrop and the blur filters are not the cause in this engine.
- A first drive measured the variants one after the other in one page. Each variant after the first was fast, because WebKitGTK had painted the rows before. That sequence gave an incorrect result for the shadow. Thus each variant starts in a new page.

## Limits

- Xvfb has no graphics card, so WebKitGTK paints in software. On a computer with an NVIDIA card, the app starts with `WEBKIT_DISABLE_DMABUF_RENDERER=1` (`crates/desktop-app/src/renderer.rs`). The drive did not use the Tauri binary.
- The sandbox cannot read `/proc`, so the drive measured no CPU use.
- Inference: the CPU use of `WebKitWebProcess` comes from the paint of rows that come into view. The cost then increases with the number of rows in the sheet. The "Detailed" view shows each line that the diff gives, also the lines of the kind `same` (`shownLines` in `crates/api/ui/review.js`).

## Open Items

- A WebKitGTK profile on the computer of the operator can find the cause there. It is a human check.
- A possible change is a "Detailed" view that shows only the changed lines with some lines around each change. It is a design decision of the operator, and this task does not make it.
