# Studio v1 Task 155b: A Notice After A Write With A Podping

Decision: [ADR 0175](../adr/0175-the-check-shows-what-music-index-holds.md) item 1. The main session changes item 1 in the commit of this task, with a `- Revised:` line. Item 1 then permits the requests of this notice. [ADR 0184](../adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md) gives the form of the notice.

Status: Complete on 2026-11-15. The main session recorded the visual gate.

## Result

- The delegate was `gpt-6-luna` with medium effort.
- Correction of the main session: **Check Again** could not start the requests again. The limit of 15 requests at 20 seconds is equal to the age limit of 5 minutes. Thus the write was always too old after the last request. `writeNotice` and `nextNoticeLoad` now get `since`, the time of the last **Check Again**. The age of the write starts at that time.
- Correction of the main session: the limit of 15 requests also stopped the next load of a podping that waits. The limit now applies only to the requests at 20 seconds.
- Step 7: the page uses `ANNOUNCE_RELOAD`. A write of a change set sends that event and does not call `onWritten`. A write from Review gives two requests at the same time, one from the new `last_version` and one from the event.
- The gate passed on the tree with this change (815 Node tests).
- Visual 1: the notice shows at 1280 px and at 390 px, in Light and in Dark. No finding from `$edges()` or `$overflow()`.
- Visual 2: the notice became the confirmation at 1280 px Light and at 390 px Dark. After the confirmation, the responder got no more requests.
- Visual 3: with no read, the responder got requests at 44 s and then each 20 s until 304 s. Then the requests stopped, and the notice showed the state with no reply and **Check Again**. After **Check Again**, the notice showed the waiting state again, and the requests started again at 20 second intervals.
- Visual 4: a write with the check box off showed no notice. A move is proved by tests: the server gives `last_write: null` for a move, and a Node test proves that `writeNotice` gives `null` when `moveNotice` gives a notice.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 11.

## Goal

After a write with a podping, the feed page shows a notice with the state of the podping. Studio asks Music Index at intervals. The notice becomes a confirmation when Music Index has read the write.

## Operator Choices

- Operator choice (2026-11-15), from drawings: "Notice on the feed page". The notice has the form of the notice of ADR 0184. With no reply, the notice says so and has **Check Again**.
- Operator choice (2026-11-15): Studio asks Music Index each 20 seconds, for a maximum of 15 requests. These are the values of `indexNeedsRead` for a move.
- Operator choice (2026-11-15): the confirmation stays until the user opens a different page or loads the feed page again.
- Operator choice (2026-11-15): a feed page that opens more than 5 minutes after the write sends one request. It then shows the state with no reply, and **Check Again**.
- A write with a move shows only the notice of ADR 0184.

## Texts

- While the podping waits or Music Index has not read the write: "You wrote this feed at {time}. Music Index reads it after it gets the podping." Below it, one line for each podping, as in the notice of ADR 0184.
- Confirmation: "Music Index has your write of {time}."
- No reply after the last request: "Music Index has not read your write of {time} yet." with the button **Check Again**.
- {time} is the local time, from `localTime`.

## Code Locations

- `crates/api/ui/app.js`, `OutsideNotices`, lines 4786 to 4860: the notice after a move, the request `GET /feeds/{id}/outside?index=true` and its timer.
- `crates/api/ui/outsidecheck.js`, `moveNotice`, lines 97 to 132, and `nextNoticeLoad`, lines 135 to 147.
- `crates/api/ui/releasemove.js`, `indexNeedsRead`, lines 119 to 123: 15 reads with 20 seconds between them.
- `crates/api/ui/editor.js`, `ReviewSheet.write`, lines 2035 to 2068: the end of a write, `onWritten(feed)` and the event `ANNOUNCE_RELOAD`.
- `crates/api/ui/styles.css`, `.notice`, lines 448 to 462, and `.status-tag.*` and `.podping-line`, lines 2380 to 2387.
- `crates/api/ui-tests/outsidecheck.test.mjs`, lines 32 to 100.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Notice of the feed page | extend | crates/api/ui/app.js | `OutsideNotices` also shows the notice after a write, with the same elements. |
| Notice model | extend | crates/api/ui/outsidecheck.js | A new function `writeNotice` is next to `moveNotice`. The two functions use one function for the podping lines. |
| Timer of the notice | extend | crates/api/ui/outsidecheck.js | `nextNoticeLoad` also gives the delay for a write that the index has not read. |
| Read limit | reuse | crates/api/ui/releasemove.js | The limit of 15 reads and the delay of 20 seconds have one source. |
| Notice styles | reuse | crates/api/ui/styles.css | `.notice`, `.status-tag.*` and `.podping-line` give the form. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- The notice of ADR 0184 and its texts
- The tab **Check** and the row "Announcement" of "Feed facts"
- `crates/api/src/`, `crates/core/`
- `crates/api/ui/assets/tokens.json`

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- Do not copy the lines of `moveNotice`. Move the shared part to one function.
- Each state update that uses the old value is a functional update. Each timer stops when the page closes or the feed changes.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In `outsidecheck.js`, move the podping lines of `moveNotice` to one function. Add `export function writeNotice(check, { reads, nowSeconds, confirmedHere })`. It gives `null`, or `{ tone, strong, lines, action }` with the texts of this file.
2. `writeNotice` gives `null` when `check.last_write` is `null`, when `moveNotice(check)` gives a notice, or when `index_read` is `null` and no podping waits.
3. `writeNotice` gives the confirmation only when `index_read` is `true` and the page showed the waiting state before (`confirmedHere`). With `index_read: true` on the first load, it gives `null`.
4. `writeNotice` gives the state with no reply when `index_read` is `false` and the reads reached the limit, or the write is more than 5 minutes old.
5. Extend `nextNoticeLoad`: while a podping waits, the delay of today applies. After that, while `index_read` is `false` and the limit is not reached, the delay is 20 seconds.
6. In `OutsideNotices`, show `writeNotice` with the elements of the move notice. Count the reads. **Check Again** sets the count to zero and sends one request.
7. After a write, load the outside check again. Use `ANNOUNCE_RELOAD` or `onWritten`, and say in the report which one.

## Acceptance Criteria

Mechanical:

1. A Node test of `writeNotice`: the waiting text with a podping line, the confirmation, the state with no reply and **Check Again**, and `null` for each condition of steps 2 and 3. Each failure message starts with "ADR 0175 item 1" and gives the fix.
2. A Node test of `nextNoticeLoad`: 20 seconds while `index_read` is `false`, `null` after 15 reads, `null` for `index_read: true`, and the delays of today for a move.
3. A Node test: `moveNotice` gives the same result as before for each case of its tests, with no change of their assertions.
4. A Node test reads `outsidecheck.js` and `releasemove.js`: the limit of 15 and the delay of 20 seconds have one definition.
5. `tools/gate.sh` passes.

Visual, by the main session, with screenshots and a local Music Index responder. This gate is open until the main session records it:

1. After a write with a podping, the feed page shows the notice at 1280 px and at 390 px, in Light and in Dark.
2. When the responder gives the date of the write, the notice becomes the confirmation.
3. When the responder keeps the old date, the notice shows the state with no reply and **Check Again** after the last request. The log of the responder shows that the requests stop.
4. A write with the check box off shows no notice. A move shows only the notice of ADR 0184.

## Test Commands

- `tools/node --test crates/api/ui-tests/outsidecheck.test.mjs`
- `tools/node --test crates/api/ui-tests/releasemove.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-155b-notice-after-a-write-with-a-podping.md` (this file), each section
- `docs/tasks/studio-v1-task-155a-last-write-facts-in-the-outside-check.md`, the section "Implementation Steps"
- `docs/adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md`
- `crates/api/ui/app.js`, `OutsideNotices`
- `crates/api/ui/outsidecheck.js`
- `crates/api/ui/releasemove.js`
- `crates/api/ui-tests/outsidecheck.test.mjs`

Goal:
- After a write with a podping, the feed page shows a notice with the state of the podping, asks Music Index at intervals, and shows a confirmation when Music Index has read the write.

Constraints:
- The sections "Operator Choices", "Texts", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- The notice of ADR 0184 and its texts
- The tab **Check** and the row "Announcement" of "Feed facts"
- `crates/api/src/`, `crates/core/`
- `crates/api/ui/assets/tokens.json`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `tools/node --test crates/api/ui-tests/outsidecheck.test.mjs`
- `tools/node --test crates/api/ui-tests/releasemove.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
