# Studio v1 Task 162: The Curator Role And Its Color

Decision: musicindex ADR 0081 R-CURATOR-1 and R-CURATOR-2 (musicindex `e8d7541`). The token is `entity.dark.curator` and `entity.light.curator` of musicindex-assets (`d7f6ed2`). [ADR 0215](../adr/0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) gives the heading of the left panel.

Status: Complete on 2026-11-17, with its screenshot visual check. The orchestrator session sent this task on 2026-11-17.

The gate of R-CURATOR-1 (2026-11-17): musicindex ADR 0081 puts two conditions before work on R-CURATOR-1 starts. Stophammer ADR 0102 must be accepted, and a Stophammer release must give its fields. Stophammer ADR 0102 is accepted. No release gives its fields.

Operator choice, 2026-11-17 (sent by the orchestrator): R-CURATOR-1 and R-CURATOR-2 of musicindex ADR 0081 apply from 2026-11-17. The Stophammer release condition applies only to R-CURATOR-3 to R-CURATOR-6. The website changes the Accepted line of ADR 0081 to say this. Studio keeps this change.

## Problem

musicindex ADR 0081 adds the role curator. Studio knows eight roles, and a feed with `rel="curator"` shows no curator role. The left panel has no heading for a curator feed.

## Work

1. Copy `tokens.json` from musicindex-assets (`d7f6ed2`).
2. Add curator to the known roles of the core and of the UI. Use the sequence of R-CURATOR-1 item 1: artist, label, hosting, curator, producer, host, author, network, sponsor. The word is "Curator" (item 3).
3. Add `--c-curator` and `.dot.curator` to `styles.css`, with the values of `tokens.json`.
4. Give a curator feed the heading "Curators" in the left panel. ADR 0088 item 4 named each heading, and ADR 0088 had three "Changed by" lines. ADR 0215 restates ADR 0088 with its changes, and ADR 0088 moves to the archive.
5. Do not add curator to the Role menu (R-CURATOR-1 item 4).
6. Add curator to the list of correct roles in the error text of `crates/api/src/routes.rs`.

## Technical Choices

- The Role menu keeps "Artist", "Label", "Hosting" and "Custom" (`FIRST_CLASS_ROLES`). The check boxes of "Custom" list each known role, so they show "Curator". That list is the role set of a feed, not the Role menu.
- A curator feed has the artist rank, by the rule of ADR 0215 item 1. The core function `rank` does not change.
- The headings after Artists come from `ROLES`, so a new role needs no change of `HEADINGS`.
- ADR 0215 keeps the item numbers of ADR 0088. Code comments that cite ADR 0088 items stay correct through the item map.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Core role list | extend | crates/core/src/collection.rs | One variant of `PublisherRole`. |
| UI role list and headings | extend | crates/api/ui/shell.js | `ROLES` gets curator. `HEADINGS` uses `ROLES`. |
| Entity colors | extend | crates/api/ui/styles.css | Two values and one dot class. |
| Shared tokens | extend | crates/api/ui/assets/tokens.json | A copy of the file of musicindex-assets. |
| Color guard | reuse | crates/api/ui-tests/type-dot.test.mjs | It compares each entity color of `tokens.json` with `styles.css`. |
| Walk setup | new | .studio-dev/walk/setup-162-curator.py | Searched `.studio-dev/walk/setup-*.py`. No setup gives a feed the role curator. |

## Acceptance Criteria

Mechanical:

1. `publisher.rs`: `roles_from_rel("producer curator hosting")` gives hosting, curator, producer.
2. `words.test.mjs`: the roles curator and artist give the label "Artist · Curator" and the artist dot. The roles producer and curator give "Curator · Producer" and the curator dot.
3. `shell.test.mjs`: a level with an artist, a curator feed and a producer feed gives the headings Artists, Curators, Producers.
4. `type-dot.test.mjs`: `--c-curator` has `#eab9f9` in Dark and `#663c73` in Light.
5. `tools/gate.sh` passes, with the ADR link guard.

Visual, by the main session:

1. At 1280 px in Dark and in Light, the left panel shows "Curators" after "Artists" and before "Producers". The curator feed has the curator color on its logo and its dot.

Visual, by the operator:

1. musicindex ADR 0081 asks for the checks of musicindex ADR 0067 item 46 in the design study. In Light, the curator color is near the artist color.

## Result

- Mechanical 1 to 4: met. The core publisher tests give 19 passed. The Node tests give 838 passed.
- Mechanical 5: met.
- Visual, by the main session: met. The shots are in `.studio-dev/walk/runs/curator-dark/shots/` and `.studio-dev/walk/runs/curator-light/shots/`. The headings are Labels, Artists, Curators, Producers and "No role". The dot is `rgb(234, 185, 249)` in Dark and `rgb(102, 60, 115)` in Light. `$edges()` and `$overflow()` give no finding, and the fit check gives no finding at 390 px and 761 px.
- Visual, by the operator: met on 2026-11-17. The design study did not have curator. The main session added curator to the Creators group of the study (version 3). The operator examined the four vision modes in Dark and in Light. The palette A checks did not change: contrast 3.61:1 and group distance 0.157.
