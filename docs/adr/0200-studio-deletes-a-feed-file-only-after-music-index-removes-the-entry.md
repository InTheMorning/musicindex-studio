# ADR 0200: Studio Deletes A Feed File Only After Music Index Removes The Entry

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, during the human check of ADR 0198. The operator selected "Only a held feed" for the scope, and "No text" for a hint about a delete outside the app.
- Technical choices: the planning model selected items 3 to 8. The operator note "consider 'retire it first'" is open (see Open Points).
- Changes: [ADR 0131](0131-delete-a-feed-from-its-file-host.md) items 1, 4, 6 and 9.
- Uses: [ADR 0198](0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md) item 4: the UI adds no rule that the core holds.

## Context

The sheet "Delete from File Host" has three rows. They are **Move It to a Different File Host**, **Retire It** with the tag "Recommended", and the red **Delete the File Only**. Step 3 of the Retire block offers **Delete Anyway** before Music Index removes the entry (ADR 0131 item 9). Both controls delete the file of a feed that an index can still hold. The entry then keeps its last payment recipients, and the user cannot correct it.

The retire state of the core (`feed_retire::retire_state`) already gives `removal_done`. It is true only when `GET /v1/feeds/{guid}` of Music Index gives a 404 with no `superseded_by`.

## Decision

1. A feed that Music Index holds has no delete in Studio. The user can delete its file only with the tools of the file host, outside Studio.
2. The sheet "Delete from File Host" then has two rows: **Move It to a Different File Host** and **Retire It**. **Retire It** has no tag "Recommended". The sheet gets no new text.
3. "Music Index holds the feed" means each answer that is not `removed`: `not_yet`, `held_with_other_guid`, `not_known_failed`, `not_known_no_guid` and `not_read`. When Studio cannot read Music Index, or the feed has no GUID, Studio cannot prove that Music Index does not hold it.
4. When the sheet opens, it reads `POST /feeds/{id}/retire/state` one time. The row **Delete the File Only** shows only when the answer has `removal_done: true`. Then the sheet has the three rows and the tag of ADR 0131 item 4. Until the answer arrives, the sheet shows the two rows.
5. Step 3 of the Retire block shows no button before `removal_done` is true. Its text stays "Wait until Music Index removes the entry." **Delete Anyway** goes.
6. The core refuses a delete draft when Music Index holds the feed. The route `POST /feeds/{id}/delete/draft` reads Music Index one time with no lock of the workspace (ADR 0015), as the route of the retire state does. Any answer other than `removed` gives `409` with the code `index_holds_feed`. The UI does not repeat this rule; it uses `removal_done` from the core.
7. Thus two routes send a request to Music Index: the retire state and the delete draft. ADR 0131 item 6 said that only the retire state sends one.
8. The restore of ADR 0131 item 14 does not change.

## Alternatives Considered

- Remove **Delete the File Only** for each feed. Not chosen by the operator: a feed that no index holds can then not be deleted in Studio.
- A line of text that tells the user to delete the file with the tools of the file host. Not chosen by the operator.
- Hide the row only in the UI. Rejected: a different client of the API could still start the delete. The rule is in the core, and the route enforces it.
- Count a failed read as "not held". Rejected: a failed network read must not open a delete.

## Consequences

- A user who wants to delete an indexed feed must first retire it, or delete the file outside Studio.
- A feed with no GUID cannot be deleted in Studio.
- The sheet can show its third row a short time after it opens.
- The delete draft needs a network read. With no network, the delete stays off.

## Open Points

- The operator note "consider 'retire it first'". The text of **Retire It** already says "After Music Index removes the entry, you can delete the file." The planning model kept the label **Retire It**. The operator decides at the human check whether a label or a text "Retire it first" is necessary.

## How This Decision Is Checked

Mechanical:

- A Node test proves that the choices of the sheet have two rows and no tag with `removal_done: false`, with no answer, and with each answer of item 3. With `removal_done: true` they have three rows and the tag.
- A Node test proves that step 3 has no button before `removal_done`, and has **Delete the File** after it.
- A Rust test proves that the delete draft gives `index_holds_feed` for each answer of item 3, and starts with `removed`.
- The four commit checks pass.

Visual, by the planning model:

- The sheet of a held feed and of a feed that Music Index does not hold, at 1280 px light and 390 px dark.
- The Retire block of a held feed: step 3 has no button.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, open **Delete from File Host…** of a feed that Music Index holds. The sheet has only **Move It to a Different File Host** and **Retire It**.
