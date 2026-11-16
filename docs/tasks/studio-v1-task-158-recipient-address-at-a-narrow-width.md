# Studio v1 Task 158: The Address Of A Recipient Row At A Narrow Width

Decision: musicindex ADR 0069 R-VALUE-2 item 1. A row of a split shows the name, the percentage and the address in a monospace font. [ADR 0105](../adr/0105-a-public-server-holds-no-assets.md) item 18 gives the method: at a narrow width, a value goes on a second line below the name.

Status: Complete on 2026-11-15. The main session made the change and recorded the visual gate.

## Operator Choices

- Operator choice (2026-11-15), through the orchestrator: this is a defect correction against R-VALUE-2 item 1, and it needs no new ADR. Below 760 px, the address goes on a second line below the name, with an ellipsis. The meter and the percentage stay on the first line. At 1280 px nothing changes.

## Defects

1. At 390 px, the columns of a closed recipient row were "9px 109px 64px 104px 0px". The fixed columns took all the width, so the address got 0 px and did not show. The fit check found nothing, because a clipped element does not overflow.
2. Found in the same check: each part of the share legend had `white-space: nowrap`. A long name made the section 444 px wide on a screen of 390 px.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Recipient row | extend | crates/api/ui/styles.css | A narrow rule for `.recipient > summary` uses the method of the `.entry-row` file status. |
| Share legend | extend | crates/api/ui/styles.css | A part of `.share-legend` moves as one unit and breaks only when it is wider than the line. |

## Acceptance Criteria

Mechanical:

1. `tools/gate.sh` passes.

Visual, by the main session. The walk data is that of ADR 0075, with a long name and address on the first recipient. No test can measure the grid:

1. At 390 px, the address column is more than 0 px wide, in Light and in Dark, in the group Recipients and in the group Fees.
2. At 1280 px, the columns do not change.
3. At 390 px, the page is not wider than the screen.

## Result

- Visual 1: at 390 px, the columns are "9px 169px 64px 54px" for a recipient and "9px 152px 64px 71px" for the fee. The address line is 307 px wide in each row. The long address ends with an ellipsis.
- Visual 2: at 1280 px, the columns are "7px 208px 64px 104px 105px", as before the change.
- Visual 3: at 390 px, the page width is 390 px and the row is 358 px wide. No finding from `$edges()` or `$overflow()` in the four views.
- The gate passed on the tree with this change.
- Human check, passed by the operator on 2026-11-15 with commits `4162348` and `88161d9`: at about 390 px, the wallet row of the section Payment of a person shows the address on a second line, in the short form of musicindex ADR 0076. The right edge does not cut it.
