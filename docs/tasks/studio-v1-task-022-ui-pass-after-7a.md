# Studio v1 Task 022: The UI Pass After Milestone 7a

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 7.
Decisions: [ADR 0037](../adr/0037-button-roles.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md).
Source: the ten open items of the [milestone 7a visual check](../reviews/2026-10-23-phase-7a-visual-check.md).

## Goal

Each button shows its role. The Files page and the review sheet show what each file and each operation is, and the user gets feedback for each operation.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-23-phase-7-review-checklist.md) before its commit.

1. [022a: the API marks and the host address](studio-v1-task-022a-api-marks-and-host-address.md).
2. [022b: the button roles and the Files page](studio-v1-task-022b-button-roles-and-files-page.md). It depends on 022a.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.

Visual:

- The operator examines the Files page, the review sheet, the Hosts screen and one sheet of the editor, in the dark theme and in the light theme. Each action button has the accent color, and each destructive button has red. No button has the fill of the surface below it.
