# Task 042 Review Checklist, 2026-10-27

This record holds the orchestration of [task 042](../tasks/studio-v1-task-042-core-feed-rules.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) owns the rules.

## Sequence

The packets run one at a time, because each packet changes `crates/api/ui/editor.js`:

1. Task 042a.
2. Task 042b.
3. Task 042c.

A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end. Task 041 starts after this task.

## Packets

| Packet | Status | Review |
|---|---|---|
| 042a | Not started | |
| 042b | Not started | |
| 042c | Not started | |

## The Review Of Each Packet

- Each new UI function that changes the model gives only a start value or a display value (ADR 0065 items 1 and 2).
- Each removed function has no caller in `crates/api/ui/`.
- The corpus round-trip tests pass with no change.

## Decisions

### Before The Packets

- The operator accepted ADR 0065 on 2026-10-27.
- At the acceptance, the planning model found that the UI keeps its own model after a save. `draft.js` copies only the GUIDs and the track numbers from the saved model. A value that a save step changes would not show, and the cover field would show a false note. ADR 0065 item 4 now says that the UI shows the value of the step. Packet 042a adds `withSavedCover` in `cover.js` for the cover.
- At the acceptance, ADR 0065 item 6 got two details. A clear sends an empty text in `image`, because `null` is also the value of a DeMu feed with no edit. The step does nothing when the sent `image` is the same as the draft `image`.
- At the acceptance, ADR 0065 item 7 got the route for one platform by the `id` of the block. An unknown `id` gives `404`.
- The project has no file for a packet template. `AGENTS.md` gives the constraint of ADR 0065 item 11.
- The explicit value of task 041 needs no copy after a save. The UI reads `true` and `yes` as the same value, and the step of packet 041a gives the same result on each save.
