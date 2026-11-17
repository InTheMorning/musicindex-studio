# Studio v1 Task 163: An Upload Stays After A Locked Vault

Decision: ADR 0059 item 3 (the finish of an upload). This task corrects a defect, so it has no new decision.

Status: Complete on 2026-11-17, with its screenshot visual check. The operator found the fault in step 3 of the rc.24 notes.

## Problem

On `studio.musicindex.org`, the operator selected an MP3 file in "Add tracks" on an iPhone. The row showed the error "upload 7a160d0c…" in red. The text went out of the card at its right edge.

The cause is in `finish_upload`. The route removes the upload row, and then the import starts. When the vault is locked, the import refuses a feed with a host that needs a secret (`vault_locked`). The UI then asks for the unlock and sends the finish again (`withUnlock`). The row is gone, so the second finish gives `not_found` with the text "upload ID".

## Work

1. In `finish_upload`, do the check of the vault before the removal of the upload row. A feed with no host keeps the error of the import (ADR 0067 item 7).
2. Let the state text of an import row wrap in the card. Give a failed state the attribute `data-fit-allow-break`, because an error can contain an ID or an address.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Finish of an upload | extend | crates/api/src/uploads.rs | One check before the removal of the row. |
| Writer check | reuse | crates/api/src/routes.rs | `writer_check` gives `vault_locked` and changes nothing. |
| Import row | extend | crates/api/ui/editor.js | One attribute on the state text. |
| Styles | extend | crates/api/ui/styles.css | The state text can shrink and wrap. |

## Acceptance Criteria

Mechanical:

1. `tests.rs` `a_finish_with_a_locked_vault_keeps_the_upload`: a finish with a locked vault gives 423 `vault_locked`. After the unlock, the same finish gives 201 with the kind `audio`. Before the change, the second finish gave 404 "upload ID".
2. `an_upload_for_a_feed_with_no_host_gives_409_read_only` passes without a change.
3. `tools/gate.sh` passes.

Visual, by the main session:

1. At 390 px in Dark and in Light, the long error of a failed import row is in the card, on more than one line. `$edges()`, `$overflow()` and the fit check give no result.

Visual, by the operator. This gate is open:

1. After a deploy, on the iPhone, add an MP3 file in "Add tracks" with the vault locked. Studio asks for the unlock, and then the row shows the facts of the file.

## Result

- Mechanical 1 and 2: met.
- Mechanical 3: met.
- Visual, by the main session: met. The page used `styles.css` with the markup of `ImportRow`. The shots are in the scratchpad of the session.
