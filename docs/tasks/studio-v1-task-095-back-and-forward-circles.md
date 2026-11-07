# Studio V1 Task 095: Back And Forward Circles

Status: Complete, with its screenshot visual check at 1280 px in the dark and the light theme.

Owner of the rule: musicindex ADR 0044 R-NAV-4.

## Goal

On Desktop, Back and Forward are round icon buttons with the neutral fill, `--pill-bg`, and a chevron in the text color.

## Files To Inspect

`crates/api/ui/shell.js`, `crates/api/ui/styles.css`.

## Acceptance Criteria

Mechanical: a Node test proves that the two controls have the class of the filled round button.

Visual: each control is a gray circle in the dark and the light theme. A control that cannot act is dimmed.

## Test Commands

```bash
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

The Touch form of ADR 0031 R-NAV-3 changes with this task.
