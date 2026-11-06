# Studio V1 Task 094: Sort Reverse

Status: Ready.

Owner of the rules: musicindex ADR 0043 R-LIST-6 and R-LIST-7.

## Goal

The Releases page has the reverse icon after the tiles icon. Its names follow R-LIST-7 item 5.

## Files To Inspect

`crates/api/ui/app.js` (`SORT_LABELS`, the Releases page), `crates/api/ui/shell.js` (the address of the list), `crates/api/ui/styles.css` (`.releases-tools`), `../musicindex-assets/icons/lucide/`.

## Constraints

- Studio holds all the rows of this list, so the whole list reverses.
- The address keeps a reversed state, as R-LIST-1 item 5 states for a sort.
- A feed group keeps its own state, as it does for its sort.
- The icons come from `musicindex-assets`. Add them there first (v4vmm packet 001 of ADR 0043 does this). Do not add them twice.

## Acceptance Criteria

Mechanical, with Node tests:

- Each sort has its reversed name.
- The reverse command changes the sequence of the rows and the address.
- `aria-pressed` follows the state.

Visual: the control looks like the Sort Control Mockup in the dark and the light theme.

## Test Commands

```bash
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

A second list in Studio has a sort. Add it to this task before the work starts.
