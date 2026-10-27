# Task 041 Review Checklist, 2026-10-27

This record holds the orchestration of [task 041](../tasks/studio-v1-task-041-msp-fields.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) owns the rules.

## Sequence

The packets run one at a time, because packets 041b to 041e all change `crates/api/ui/editor.js`:

1. Task 041a.
2. Task 041b.
3. Task 041c.
4. Task 041d.
5. Task 041e.

The packets start after the operator accepts ADR 0064. A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end.

## Packets

| Packet | Status | Review |
|---|---|---|
| 041a | Not started | |
| 041b | Not started | |
| 041c | Not started | |
| 041d | Not started | |
| 041e | Not started | |

## Decisions

### Before The Packets

- The first version of the packets put the form rule of the explicit value in the UI. It also said that the core does not change a value. No decision gave that rule. The operator decided on 2026-10-27 that the core applies the form rule at the draft save. ADR 0064 item 4 and packet 041a give this rule.
- The operator decided on 2026-10-27 that a lyrics file has its own kind in the picker. ADR 0064 item 9 gives this rule, and packet 041c adds the kind.
- The operator decided on 2026-10-27 that the page of ADR 0064 item 7 has the name "Feed settings". The album page, the playlist page and the publisher page each have a row that opens it.
- The packets are five: the core elements, the channel fields, the lyrics kind, the track fields and the artwork list.
- ADR 0064 stays proposed until the operator accepts it.
- The operator decided on 2026-10-27 that the work of ADR 0065 comes before task 041. ADR 0065 moves the feed rules of the UI to the core.
- The near ratio uses a list of 19 ratios and a tolerance of 2 %. The behavior agrees with MSP 2.0. No code comes from MSP 2.0.
