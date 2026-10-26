# ADR 0054: Live Items, Connected Apps And Show Grants

- Status: Accepted
- Date: 2026-10-26
- Owner: operator

## Context

[ADR 0008](0008-feed-types-and-feed-identity.md) item 1 makes live items (`podcast:liveItem`) a v1 feed type. The core reads them and keeps them losslessly. Studio has no editor for them.

The research of 2026-10-26 gives these observations:

- A live item has the child elements of an item. `status` is required and is `pending`, `live` or `ended`. `start` is required, and `end` is recommended. `contentLink` must be present. A changed `guid` means a new stream (`musicindex-research`, `sources/podcast-namespace/docs/tags/live-item.md`).
- Stophammer keeps each live item as an RSS fact. An `ended` live item with an enclosure becomes a track. A podping starts the crawl (stophammer ADR 0064).
- `podcast:liveValue` is not in the namespace sources. Two corpus feeds have it, with `uri` and `protocol` attributes. v4vmm shows the tag with a "Copy feed tag" action and does not write a feed (v4vmm ADR 0059).
- v4vmm and musicindex-live-publisher are Rust programs on the computer of the user. Neither one writes feed XML.
- The hub request of [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) has no reason parameter. Thus a podping from the hub has no `live` or `liveEnd` reason.

[ADR 0007](0007-draft-diff-write-and-announce.md) says that no write occurs without a diff that the user has seen. During a show, the user works in v4vmm or with the live publisher, not in Studio.

On 2026-10-26 the operator selected the decisions below.

## Decision

### The Live Item Editor

1. A live item is in an album feed that Studio can write, or in a show feed. A show feed is a new feed with `podcast:medium` `music` that the user makes for live items. Its form is that of a new album feed ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)).
2. A new live item gets a new UUID v4 `guid`, the status `pending`, and a `start` time from the user. The `guid` does not change after that ([ADR 0008](0008-feed-types-and-feed-identity.md)).
3. The live item page edits the title, the description, the image, `start`, `end`, the stream enclosure, `podcast:contentLink` and the item value routes. It shows `podcast:liveValue`, and the user can change it or remove it.
4. Studio does not write a live item with no `start`, no `contentLink`, or a `status` that is not `pending`, `live` or `ended`. The write stops before any change on a host, as for an incorrect address.
5. "Go live" changes the status from `pending` to `live`, and "End the show" changes it from `live` to `ended`. Each one changes the draft and opens the review sheet. The user writes, and the announcement of ADR 0052 follows.
6. An `ended` live item can get a recording. The user imports an audio file, as for a track ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md)), and the enclosure of the live item points to it.

### Connected Apps

7. A connected app is a program on the same computer, such as v4vmm or musicindex-live-publisher, that asks Studio to write a live item change. Studio owns the vault, the collection and the write. The other app does not open them.
8. The user adds a connected app in Studio with a name. Studio makes a token of 32 random bytes, and shows it one time. The collection keeps only the SHA-256 of the token. The user can remove the app at any time.
9. The routes for connected apps are under `/app/v1/`. They need a loopback `Host` and the token of an app. They do not accept the access token of the UI. The `/api/` routes do not accept the token of an app. Thus the rule of [ADR 0017](0017-local-api-access.md) for `/api/` does not change.
10. A connected app can only list its show grants, change the status of a granted live item, and set or remove its `podcast:liveValue`.

### Show Grants

11. When a diff adds or changes a `pending` live item, the review sheet has a check box: "Let connected apps start and end this show, and set its live value". The diff lists the grant, and the grant is a part of the diff ID.
12. The approved write records the grant. A grant names the feed and the `guid` of one live item. It applies to each connected app.
13. A grant covers only these changes of that live item: the status from `pending` to `live`, the status from `live` to `ended`, and the set, change or removal of `podcast:liveValue`. Each other change needs a normal diff.
14. A grant ends at the first of these events:
    - A write sets the status to `ended`.
    - 24 hours pass after the `end` time, or after `start` when there is no `end`.
    - The live item leaves the feed.
    - The user removes the grant.
15. For a granted change, Studio reads the hosted file, applies only the change, and makes its diff. Studio checks that the diff changes nothing that the grant does not cover. Then it writes through `pipeline::write_draft` with the ID of that diff. The snapshot, the conflict check and the announcement are the same as for each write.
16. Studio refuses a granted change when the feed has a draft with changes, when the vault is locked, or when the grant has ended. Studio thus never writes an edit of the user that the user has not approved.
17. The live item page lists each granted write, with the name of the app and the time.

### The Change To ADR 0007

18. This ADR supersedes the ADR 0007 invariant "No write occurs without a diff that the user has seen" in part. The invariant becomes: no write occurs without a diff that the user has seen, or a change that a show grant of the user covers.

## Alternatives Considered

- A quick write of the status with no review sheet in Studio. Not chosen by the operator. A status change in Studio uses the review sheet.
- Studio keeps `podcast:liveValue` with no editor. Not chosen by the operator. v4vmm and the live publisher ask Studio to write it.
- The other apps link `studio-core` as a crate and write the feed themselves. Not chosen by the operator. Two programs would open the same collection and vault.
- The other app shows the diff, and the user approves each change there. Not chosen by the operator. Each app would need a diff view.
- A grant of each field of the live item, or of the status only. Not chosen by the operator.
- A Nostr key of each app in place of a token. Not chosen by the operator.
- Live items only in album feeds, or only in show feeds. Not chosen by the operator.
- Leave the routes for other apps to a later phase. Not chosen by the operator. Milestone 11b builds them, with a test client.

## Consequences

- `AGENTS.md` restates the changed invariant and the changed write rule.
- The connected apps and the grants add two tables to the collection.
- A granted "Go live" write waits 30 seconds before its announcement (ADR 0052 item 14), and the hub sends an update podping, not a `live` podping. Stophammer crawls the feed after that podping.
- A grant does not depend on the vault being open at the time of the grant. Each granted write needs the open vault, because the connector needs its secrets.
- v4vmm and musicindex-live-publisher need a change in their own repositories to use the routes. This ADR owns the contract. Their records cite it.
- The Server Shell phase must decide the connected apps of a server user.

## Verification

Mechanical:

- A test adds a live item, writes it, then does "Go live" and "End the show" through the draft. The written feed has each status in sequence and the same `guid`.
- A test refuses a write of a live item with no `contentLink`, before any change on a host.
- A test confirms that `/app/v1/` refuses the UI access token, and that `/api/` refuses an app token.
- A test client with an app token sets the status to `live`, sets the `podcast:liveValue`, and sets the status to `ended`. The hosted file has each change, and the grant ends after the last one.
- Tests refuse a granted change that is not covered, and a change after the end of the grant. A test refuses a change while the feed has a draft with changes.
- A test confirms that the collection keeps no app token as plain text.

Visual:

- The live item page with its status buttons and its list of granted writes. The grant check box in the review sheet, and the Connected apps panel. Each one in the dark theme, in the light theme and at 390 px.

Human test:

- v4vmm or the live publisher uses the contract against a real Studio during a test show, when that repository has built its side.

## Invariants

- A connected app never gets a secret of the vault.
- A granted write changes only what the grant covers.
- A write to a host occurs only through `pipeline::write_draft`.
