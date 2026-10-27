# Task 041 Review Checklist, 2026-10-27

This record holds the orchestration of [task 041](../tasks/studio-v1-task-041-msp-fields.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) owns the rules.

## Sequence

The packets run one at a time, because packets 041b, 041c and 041d all change `crates/api/ui/editor.js`:

1. Task 041a.
2. Task 041b.
3. Task 041c.
4. Task 041d.

The packets start after the operator accepts ADR 0064. A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end.

## Packets

| Packet | Status | Review |
|---|---|---|
| 041a | Not started | |
| 041b | Not started | |
| 041c | Not started | |
| 041d | Not started | |

## Decisions

### Before The Packets

- The form rule of the explicit switch (ADR 0064 item 4) is in the UI model module `explicit.js`. The editor sends the model as text, so only the UI knows that the user set the switch. The listing rule of ADR 0063 uses the same structure. ADR 0064 asks for a core test of this rule. In the packets, a Node test examines the rule, and a core test examines that a write keeps the text of the model. The verification text of ADR 0064 must agree before the operator accepts it.
- The ADR names three tasks. The packets are four, because the core elements are one packet before the UI packets.
- The page of ADR 0064 item 7 has the name "Feed settings". The album page, the playlist page and the publisher page each have a row that opens it.
- The near ratio uses a list of 19 ratios and a tolerance of 2 %. The behavior agrees with MSP 2.0. No code comes from MSP 2.0.
