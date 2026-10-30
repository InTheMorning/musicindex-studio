# Studio v1 Task 056: File Hosts And The Destination Icons

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "MSP 2.0 Parity".
Decisions: [ADR 0092](../adr/0092-file-hosts-and-the-destination-icons.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md), [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md). Shared: musicindex ADR 0022, ADR 0025 R-ICON-1 and ADR 0032 R-DOT-1.

Status: Not started. Commit `b13c30c` removed the type dot from the Hosts row before this task.

## Goal

Each fixed destination of the left panel shows its Lucide icon. Each UI text names a connector "file host", and the word "host" alone is the role `host`.

## Packets

The two packets change different parts of the UI. They go in this sequence, because the two packets change `app.js` and `shell.js`:

1. [056a: the destination icons](studio-v1-task-056a-destination-icons.md).
2. [056b: the name "File hosts"](studio-v1-task-056b-file-hosts-name.md).

Task 054c adds the Recipients row. When task 054c is complete before task 056a, task 056a gives that row its icon. When task 056a is complete first, task 054c gives the icon.

## Constraints

- An icon is not a type dot, and has no entity color (ADR 0018, musicindex ADR 0032 R-DOT-1).
- The UI loads nothing from other servers (ADR 0016). Each icon is inline.
- Each UI text is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical: the mechanical criteria of ADR 0092 §Verification, the criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The left panel and the root page show the icon of each fixed destination, aligned with its label.
- The current destination has the accent color on its icon and its label.
- A collection with a feed with the role `host` shows the role heading "Hosts" and the destination "File hosts".
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: on a phone, open each destination of the root page.
