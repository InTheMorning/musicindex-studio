# Studio v1 Task 026: The Shell, Releases And The Outline

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), the UI work before Phase 9.
Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0037](../adr/0037-button-roles.md).
Shared rules: musicindex ADR 0027 R-SHELL-1, R-NAV-2 and R-LIST-1, ADR 0023 R-WORD-2, ADR 0020 R-VALUE-1 and ADR 0024 R-PROFILE-1.

## Goal

Studio starts on Releases. The left panel holds the destinations, or the outline of the open feed. The bar has the back control. At narrow width, Studio is one stack with a root page and an outline page. The feed tree and "Select a feed" go.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-task-026-review-checklist.md) before its commit. The packets go in this sequence:

1. [026a: the feed list facts](studio-v1-task-026a-feed-list-facts.md).
2. [026b: the shell models and the Node tests](studio-v1-task-026b-shell-models.md). It depends on 026a.
3. [026c: the shell UI](studio-v1-task-026c-shell-ui.md). It depends on 026b.
4. [026d: the Value routes page](studio-v1-task-026d-value-routes.md). It depends on 026c.
5. [026e: one stack at narrow width](studio-v1-task-026e-narrow-stack.md). It depends on 026d. It replaces the menu button and the drawer of 026c (musicindex ADR 0030).

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- The Node tests of [ADR 0042](../adr/0042-ui-models-with-node-tests.md) give the criteria of the Verification section of ADR 0039 (026b).
- No UI file has the text "Select a feed" (026c).

Visual:

- The visual criteria of ADR 0039, for the planning model with `tools/drive-ui.mjs` and then for the operator: Studio starts on Releases, and a feed opens over it with its outline. Hosts and Lock Studio are at the bottom of the left panel in a short window and in a tall window. At 390 px, no page has a menu button. Back on Releases opens the root page, and a feed opens on its outline page.
