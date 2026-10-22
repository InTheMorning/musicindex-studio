# Studio v1 Task 008: UI Shell, Settings And First-Run Wizard

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 4.
Decisions: [ADR 0005](../adr/0005-collection-and-first-run-discovery.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md), [ADR 0016](../adr/0016-ui-without-a-build-step.md) and [ADR 0017](../adr/0017-local-api-access.md).

## Goal

Make the UI shell in the Afterglow direction: a sidebar with the collection, a detail pane, the settings screen for connectors, and the first-run wizard.

## Wizard Steps

1. Make the vault: a passphrase, a confirmation and an optional recovery key that the wizard shows one time.
2. Add a host: a local folder, with an optional public URL.
3. Add a feed: its path in the folder. Studio reads it and shows its title.

The other discovery methods of ADR 0005 (a pasted URL, a musicindex search and an MSP 2.0 backup) need an HTTP client. Task 009 adds them.

## Acceptance Criteria

Mechanical:

- The UI files contain no address of a server other than the own API (ADR 0016).
- Each API call of the UI sends the token header (ADR 0017).

Visual (the operator checks these):

- The shell uses the Afterglow tokens, the Figtree font, the one-color mark and the dark and light themes.
- The wizard, the unlock screen and the settings screen work from the keyboard. Each control is a minimum of 44 px high.
- The empty, loading and error states have a design.
