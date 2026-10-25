# Phase 8 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 025](../tasks/studio-v1-task-025-publisher-feeds.md)
- Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The three commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Invariants

- No feed changes without a diff that the user saw ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)). A link changes two drafts, and does not write them.
- A link has the same `rel` on its two sides (ADR 0041 items 6, 7 and 11).
- Studio writes a publisher link of a release only in `podcast:publisher` (ADR 0041 item 7).
- Studio keeps each `remoteItem` that it did not make, at its position (ADR 0041 item 13).
- A link does not change the feed GUID or an item GUID ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- A read and a write with no edit give the same bytes ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- Each `/api/` route needs the access token ([ADR 0017](../adr/0017-local-api-access.md)).
- A schema change is a new migration ([ADR 0014](../adr/0014-collection-in-sqlite.md)).
- Each labeled button has its role ([ADR 0037](../adr/0037-button-roles.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [025a](../tasks/studio-v1-task-025a-publisher-core.md) | Pass | The planning model accepted four choices of the coding agent. `PublisherError` wraps a `PipelineError`. `role` and `set_role` give `NotPublisher` too. `unlink` does not check the medium. A feed with no draft gives its hosted GUID and title. Open item for 025b: a link and a change of the role change one draft before the other. Thus each route starts each draft before the call. |
| [025b](../tasks/studio-v1-task-025b-publisher-routes.md) | Pass after one correction | Each route started the drafts before the core checked the two feeds. A refused request then left a new draft on a feed with no link. The planning model added a check of the connector and the medium before the drafts start. The planning model accepted two choices of the coding agent. An unknown role gives 422 `invalid_feed`. `changed` compares the draft bytes before and after the call. Open item: a feed from a host has no medium before its first read. Thus the pages of 025c and 025d do not offer it for a link. |
| [025c](../tasks/studio-v1-task-025c-publisher-page.md) | Pass after one correction | The planning model examined the screenshots in the dark theme and in the light theme. They show the new form, the page, the notices, the role change and the review of a linked release. The header notice said "this release" on a publisher feed. It now says "this publisher feed". The planning model accepted three choices of the coding agent. An upload refusal shows the message of the server. A feed with no title gives "Untitled feed". "Remove" asks no question, because the user examines the draft before the write. Open item: the agent saw a slow first load of the draft in a new browser profile. The cause is not known. |
| [025d](../tasks/studio-v1-task-025d-release-publishers.md) | Pass after two corrections | The planning model examined the screenshots of the album page and of the review of the release, with the label feed as the primary publisher. (1) `changed` did not list the feed `{id}`. On the album page, that is the publisher feed, so the agent named the selected feed even when its draft did not change. Each route now lists each changed feed, and each page leaves out its own feed. The API tests and packet 025b record the new rule. (2) "Move up", "Move down" and "Remove" of each row had no row name. Each one now has an `aria-label` with the title. The planning model ran the two pages again after the corrections. The notices are one for each call. |
