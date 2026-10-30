# ADR 0092: File Hosts And The Destination Icons

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) items 1, 5 and 11, for the name "Hosts" and for the marker of a destination. [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) item 1 uses the name of this record.
- Uses: musicindex ADR 0022 (the shared Lucide icons), musicindex ADR 0025 R-ICON-1 (the icon size), musicindex ADR 0032 R-DOT-1 (the type dot) and musicindex ADR 0033 R-WORD-3 (the role words).

## Context

- The destination "Hosts" holds the connectors: a local folder, S3 and R2, SFTP, Blossom and nsite ([ADR 0006](0006-connectors-and-the-v1-set.md)). Each UI text calls a connector "a host".
- The left panel shows the role heading "Hosts" for the role `host` (`crates/api/ui/shell.js`, [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 4). The role `hosting` is a different role ([ADR 0083](0083-hosting-is-a-first-class-role.md)). When the collection has a feed with the role `host`, the left panel shows "Hosts" two times with two meanings.
- The row "Hosts" shows the type dot of the role `hosting` (`crates/api/ui/app.js`, `LeftPanelFooter`). Task 026 added it in `6ceaddb`. A dot shows the type of a feed (musicindex ADR 0032 R-DOT-1), and an entity color is not a general accent ([ADR 0018](0018-passphrase-strength-and-error-color.md)).
- The other destination rows have no marker.
- v4vmm shows a monochrome Lucide icon on each destination of its sidebar. The icon of the current destination has the accent color (`src/ui/shells/library/sidebar.rs` in `../v4vmm`).
- Studio already draws the Lucide paths of `chevron-left` and `chevron-right` inline (`Chevron` in `app.js`). `musicindex-assets/icons/lucide/` has `library`, `wallet`, `hard-drive`, `settings` and `lock`. It has no `lock-open`.
- On the gloss fill of the current row, accent text has a contrast of 3.6:1 to 4.1:1 in the dark theme, and 4.2:1 to 4.7:1 in the light theme. The values come from the screenshots of task 056a on 2026-10-30. Text needs 4.5:1, and an icon needs 3:1 (musicindex ADR 0025 R-STATUS-1 note 2).
- The S3 form uses the word "provider" for the company that gives the bucket, for example AWS or Cloudflare.
- The operator selected this decision from two questionnaires on 2026-10-30. The operator examined the names "Storage" and "Providers", and did not select them.

## Decision

### The Name

1. The destination "Hosts" has the name "File hosts".
2. Each UI text that names a connector uses "file host", for example "Add a file host first" and "the file host of this feed". The word "host" alone in a UI text is the role `host` only.
3. The page title, the bar title and the bottom row show "File hosts".
4. The code names `connector` and `host`, the routes and the API fields do not change.

### The Icons

5. Each fixed destination of the left panel and of the root page shows an icon before its label:

   | Destination | Lucide icon |
   | --- | --- |
   | Releases | `library` |
   | Recipients | `wallet` |
   | File hosts | `hard-drive` |
   | Settings | `settings` |
   | Lock Studio | `lock` |
   | Unlock Studio | `lock-open` |

6. A row of a publisher feed shows its art, not an icon ([ADR 0089](0089-the-left-panel-shows-the-art-of-each-publisher-feed.md)).
7. Studio draws each icon inline, with the path of its Lucide icon and `currentColor`. The icon has the line height of the row text (musicindex ADR 0025 R-ICON-1 item 1) and a stroke of 2 (item 3). The icon has `aria-hidden="true"`, and the label gives the accessible name.
8. The icon of the current destination has the accent color. Its label keeps the text color, because accent text on the gloss fill has less than 4.5:1. The icon of each other destination has the color of its label.
9. No destination row shows a type dot.
10. `lock-open` comes into `musicindex-assets/icons/lucide/` in a commit of that repository, before the Studio commit that uses it (musicindex ADR 0022).

## Alternatives Considered

- "Storage". Not selected by the operator.
- "Providers". Not selected by the operator. The S3 form uses "provider" for the company, and "hosting provider" is the usual name of a feed that has the role `hosting`. A local folder is not a provider.
- "Connections". Not selected. Connected apps ([ADR 0054](0054-live-items-connected-apps-and-show-grants.md)) and the NIP-46 signer also connect.
- Icons on the bottom group only. Not selected by the operator. Releases then has no icon, and v4vmm has an icon on each destination.
- A monochrome icon that does not change. Not selected by the operator.
- The accent color on the label of the current destination, as in v4vmm. Not selected by the operator on 2026-10-30, after the contrast measurement of task 056a.
- `key-round` or `lock` for Unlock Studio. Not selected by the operator. `lock-open` makes a pair with `lock`.

## Consequences

- The left panel does not show one word with two meanings.
- The label of the current destination has the text color in Studio and the accent color in v4vmm.
- About 60 UI texts change. The API error messages that name a host change in the same task.
- Earlier records keep the word "host" for a connector. This record gives the UI word.

## Invariants

- The label of a destination is not a role heading of `shell.js`.
- No destination row shows a type dot.

## Verification

Mechanical:

- A Node test: each fixed destination of `leftPanel` gives the icon of item 5.
- A Node test: no destination label is the same as a role heading of `shell.js`.
- A Node test: `LeftPanelFooter` and `RootPage` of `app.js` use no `dot` class.
- A Node test: no UI text names a connector with the word "host" alone. The test finds "host" and "hosts" with no "file" before them in the UI strings. A list gives each permitted text of the role `host`.

Visual:

- The left panel and the root page with each destination current, in the two themes, at desktop width and at 390 px.
- A collection with a feed with the role `host`: the role heading "Hosts" and the destination "File hosts".
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.
