# Studio v1 Task 193: The Fixed Rows Are At The Bottom Of The Root Page

Decision: [ADR 0226](../adr/0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) items 7 and 8.

Status: Open.

## Problem

Finding 12 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). People, File hosts and Settings are at the top of the narrow root page and at the bottom of the wide left panel.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Narrow root page | extend | crates/api/ui/app.js | `RootPage` near line 3418 puts the fixed rows between Releases and the feed groups. |
| Destination rows | extend | crates/api/ui/app.js | `destinationRows` near line 3246. Only `RootPage` uses its parameter `between`. |
| Panel footer | reuse | crates/api/ui/app.js | `LeftPanelFooter` near line 3398 gives the rows of `panel.bottom`, as in the wide panel. |
| Root order test | extend | crates/api/ui-tests/narrow-root.test.mjs | The test of ADR 0223 item 7 near line 30. |

## Changes

1. `RootPage` renders `destinationRows(panel.top, go)` and then `LeftPanelFooter` with the full panel, as the wide panel does.
2. Remove the parameter `between` of `destinationRows`.
3. The test of ADR 0223 item 7 changes to ADR 0226 items 7 and 8: `RootPage` gives the full panel to `LeftPanelFooter`, and does not split `panel.bottom`.

## Checks

Mechanical:

- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual, by the main session with screenshots at 390 px and 1280 px: the root page and the left panel have Releases, the feed groups, then People, File hosts and Settings.
