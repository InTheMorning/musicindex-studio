# ADR 0205: Publisher Feeds, Announcements And Show Grants

- Status: Accepted
- Date: 2026-11-14
- Owner: operator
- Accepted: 2026-11-14. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0041](../archive/adr/0041-publisher-feeds-and-two-way-links.md), Publisher Feeds And Two-Way Links.
- Supersedes: [ADR 0052](../archive/adr/0052-announcements-through-the-podcast-index-hub.md), Announcements Through The Podcast Index Hub.
- Supersedes: [ADR 0054](../archive/adr/0054-live-items-connected-apps-and-show-grants.md), Live Items, Connected Apps And Show Grants.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the draft, the diff and the write. [ADR 0008](0008-feed-types-and-feed-identity.md) for the feed types and the GUIDs. [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) for the release folder. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) for the test servers.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) (the grant vaults and the apps on the server), [ADR 0075](0075-the-publisher-page-has-payments-and-credits.md) (the payments and the credits of a publisher feed), [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) (the roles of a publisher feed), [ADR 0083](0083-hosting-is-a-first-class-role.md) (the roles of the form), [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) (the placeholder GUID), [ADR 0086](0086-a-playlist-links-to-its-artist-or-label.md) (a playlist as a release), [ADR 0110](0110-a-connected-app-signs-each-request-with-its-own-key.md) (the key of an app and the pairing), [ADR 0127](0127-a-changed-feed-gets-a-new-build-date.md) (the build date), [ADR 0135](0135-the-panel-holds-the-content-and-the-page-has-tabs.md) (the tabs and the outline), [ADR 0136](0136-one-announcement-choice-at-the-write.md) (the announcement choice), [ADR 0161](0161-a-live-item-needs-a-stream-and-a-content-link-is-recommended.md) (the checks of a live item), [ADR 0184](0184-after-a-move-studio-shows-the-podping-of-the-move.md) (the podping of a move), [ADR 0189](0189-a-move-sends-one-podping-of-the-old-address.md) (the address of a move), [ADR 0190](0190-review-gives-the-return-wait-of-music-index.md) (the send time of a return), [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) (the podping that goes again).
- Revised: 2026-11-15 by operator decision. Item 19 gives the position of the check box in the review sheet, from the selections of the operator on 2026-11-14 and 2026-11-15: the sheet has no row **Announce**. Item 38 gives the removal of a live show, which the operator selected on 2026-11-14. These rules were in ADR 0209 and ADR 0210, which are deleted.
- Changed by: [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 14 to 16, on 2026-11-20. Item 19: the XML changes come first, the file operations are in a closed fold, and the write error is above the check box.
- Changed by: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 7 to 11, on 2026-11-20. Item 8: Review of a new release also writes its artist page when the check box "Also write {artist page}" is on.

## Context

- ADR 0041 gave the publisher feeds and the links between a publisher feed and its releases. ADR 0052 gave the announcements through the Podcast Index hub. ADR 0054 gave the live items, the connected apps and the show grants. ADR 0054 also changed the invariants of ADR 0041 and ADR 0052 for a change that a show grant covers.
- Subsequent ADRs changed ADR 0041 five times, ADR 0052 five times and ADR 0054 five times.
- The global decision of 2026-11-14 (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR is that ADR for the publisher feeds, the announcements and the show grants.
- Some text of the superseded ADRs was true only before a later phase. This ADR leaves it out:
    - ADR 0041 said that Studio did not have the shell of ADR 0039. It also said that the announcement of a new publisher feed waits for Phase 10. ADR 0052 item 10 gives that announcement.
    - ADR 0054 said that the Server Shell phase must decide the connected apps of a server user. ADR 0058 decided it.
    - ADR 0054 item 7 gave v4vmm as an example of a connected app. On 2026-11-01 the operator said that musicindex-live-publisher is the only app that connects. This ADR gives musicindex-live-publisher as the example. This is not a change of a rule.
- The superseded ADRs keep their context, their alternatives and their checks. They are in `docs/archive/adr/` for research. The item map at the end of this ADR gives the new item of each superseded item.

## Decision

### Publisher Feeds

1. A publisher feed has `podcast:medium` `publisher`. It has a set of roles from the eight roles of musicindex ADR 0036 R-ROLE-1. The collection keeps the set of each publisher feed, inferred or confirmed (ADR 0082 items 1, 2 and 10).
2. The user makes a new publisher feed in "Add feeds", with a host, a file name, a name and a role. The feed GUID is the UUIDv5 of its address (ADR 0008 item 2). The draft gives the name to `title` and to `itunes:author`. The feed gets a release folder (ADR 0035 item 1). The form has the roles "Artist", "Label" and "Hosting" (ADR 0083 item 3).
3. A publisher feed on a connected host is editable. Its page has the title, the name, the description, the image, the website, the roles and the list of releases. The publisher page also edits the value block, the support links, the people and the license (ADR 0075). The pages of a publisher feed are tabs, and its outline holds its head and its releases (ADR 0135 items 2 and 4). It has a Files page, as a release has.
4. A publisher feed from a host can have no confirmed set of roles. Then Studio records an inferred set from the `rel` of its channel `remoteItem` elements (ADR 0082 items 3 and 4, ADR 0086 item 3). A set becomes confirmed when the user selects it, or when Studio records it for a change of a link (ADR 0082 item 5). A feed with no recorded set and no inferred set needs a set that the user selects before the first change of a link.

### The Link

5. A link joins a publisher feed and a release of the collection. A release is an album (`music`) or a playlist (`musicL`) (ADR 0086 item 1). Each of the two feeds is on a connected host. Studio does not offer a release that is not in the collection, or a release that is read-only.
6. In the publisher feed, a link is a channel `podcast:remoteItem` with the `medium` value of the release: `music` or `musicL` (ADR 0086 item 2). It has the `feedGuid` and the `feedUrl` of the release, its title in `title`, and the roles in `rel`. `rel` has the set of roles with one space between two roles (ADR 0082 item 8).
7. In the release, a link is a `podcast:remoteItem` with `medium="publisher"` in `podcast:publisher`. It has the `feedGuid` and the `feedUrl` of the publisher feed, its title in `title`, and the roles in `rel`. Studio does not write a channel `remoteItem` with `medium="publisher"` that is not in `podcast:publisher` (Stophammer ADR 0069 §1a).
8. An add or a removal of a link changes the draft of each of the two feeds. Studio starts a draft for a feed that has none. Each feed keeps its own diff and its own write (ADR 0007). After the change, the page names the other feed that has a changed draft.
9. A release can have more than one publisher feed. The album page shows them in the sequence of `podcast:publisher`, and the first one is the primary publisher. The user changes the sequence there. A new link goes last.
10. The publisher page shows the releases in the sequence of the channel `remoteItem` elements of item 6. The user changes the sequence there. A new link goes last.
11. A change of the set of roles changes the `rel` of each link in the draft of the publisher feed. This applies to the two kinds of release. It also changes the `rel` of the link in the draft of each linked release (ADR 0082 item 8, ADR 0086 item 3).
12. The `feedGuid` of a link is the `podcast:guid` of the draft of the other feed. When the other feed has no draft, it is the `podcast:guid` of its hosted file. The `feedUrl` is the public address of the other feed. A feed with a placeholder GUID has no public address, so its link has no `feedUrl` (ADR 0122 item 5).
13. Studio keeps each `remoteItem` that it did not make. A link to a feed that is not in the collection shows as "Not in the collection". The user can remove it from the open feed. That removal changes no other feed.
14. In a release, Studio changes only `podcast:publisher` for a link. In a publisher feed, it changes only the fields of item 3 and the channel `remoteItem` elements.

### The Method Of An Announcement

15. Studio announces a feed with one request: `GET <hub>?url=<feed address>&podping=1`. A request always has `podping=1` (ADR 0136 item 3). The hub is `https://hub.podcastindex.org/pubnotify`. A shell can select a different hub, for example a local test server.
16. The request uses no key, no Hive account and no secret.
17. A response with the status 200 is a success. A response with the JSON `status` `false` or `"false"` is a refusal. Each other status, each refusal and each network failure is a failure.
18. Studio does not send a separate notice to the musicindex API. Stophammer reads the podping and crawls the feed. This changes ADR 0035 item 21: "the musicindex API" is reached through the podping. With this change, ADR 0035 item 21 is in force, and [ADR 0204](0204-feed-files-moves-and-the-server.md) item 21 gives the announcement of a move.

### The Announcement Choice

19. One check box gives the announcement choice of a write. One check box applies to each feed of a change set (ADR 0136 items 1 and 7). This item owns its position and its label:
    - In the review sheet, the check box is in the footer, at the right side, directly above the write button. It is in view at each scroll position. Its label is "Send a podping".
    - The review sheet has no row **Announce**. It does not show the address of the podping or the time of the send.
    - When the feed has no public address, or when the write deletes the feed file, the sheet has no check box. A line with the cause is in the position of the check box.
    - On the review page of a change set, the check box has the label "Tell Podcast Index and send a podping", directly above the write button.
20. Studio remembers the last choice for the next write. It starts on. The Settings page has no announcement section (ADR 0136 item 2).

### The Addresses

21. Studio announces the public address of the feed ([ADR 0021](0021-public-address-of-a-feed.md)).
22. A write with a move of the feed XML announces one address: the address before the move. A move with no address before the move announces the new address (ADR 0189 items 1 and 2).
23. A feed with no public address gets no announcement. A write is not blocked for this reason.
24. The first write of a new release or a new publisher feed gets an announcement, as each other write does.

### The Diff

25. The diff lists each planned announcement with its address. The check box does not show when the feed has no public address. The sheet then gives that reason (ADR 0136 item 5).
26. The planned announcements are a part of the diff ID. A change of the check box saves the choice and gets the diff again, so the user does not approve a second time (ADR 0136 item 4).

### The Queue

27. A write that completes with no error puts each planned announcement into a queue in the collection. A write that stops with an error puts nothing into the queue.
28. Studio sends an announcement 30 seconds after the write, so that the host or its CDN serves the new file. The write of a return can send its podping at a later time (ADR 0190 items 6 and 7).
29. After a failure, Studio tries again at 1, 2, 4 and 8 minutes after the failure. After the fifth failure, the announcement is failed.
30. A new announcement for an address that waits in the queue replaces the waiting one. Its 30 seconds start again. An announcement that Studio sends at that time is not replaced.
31. The write does not wait for the announcement. A failed announcement does not change the write.
32. At each start, Studio marks each waiting announcement that is older than 24 hours as failed. It sends each younger one at its time.
33. Studio keeps the last 10 finished announcements of each feed, and deletes older ones.

### The Status

34. The page of a feed shows the last announcement: waiting with its send time, sent with its time, or failed with its time and its error text.
35. Studio has no button that announces a feed with no write, with one exception. The notice of a move by Studio has **Send the Podping** or **Send the Podping Again**, which puts one announcement into the queue (ADR 0184 items 3 to 10, ADR 0189 items 3 to 7). A failed line and a sent line have the button (ADR 0184 item 7, ADR 0191 item 4). A feed with no public address has no button (ADR 0184 item 11). The click of the user is the approval of that announcement.

### The Live Item Editor

36. A live item is in an album feed that Studio can write, or in a show feed. A show feed is a new feed with `podcast:medium` `music` that the user makes for live items. Its form is that of a new album feed ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)).
37. A new live item gets a new UUID v4 `guid`, the status `pending`, and a `start` time from the user. The `guid` does not change after that (ADR 0008).
38. The page of a live show edits the title, the description, the image, `start`, `end`, the stream enclosure, `podcast:contentLink` and the item value routes. It shows `podcast:liveValue`, and the user can change it or remove it. The tab Live Shows of the feed page has one row for each show, and the page of one show keeps this tab (ADR 0135 item 5). The page also removes the show:
    - The page of a live show has the button **Remove the Show** at its end. It is destructive.
    - The button opens a confirm with the title `Remove “{title}”?` and the destructive button **Remove the Show** ([ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md)).
    - The text of the confirm is "Studio removes the show from the draft. The feed changes at the next write." For a show with the status `live`, the text also says "Apps lose the stream of this show at that write."
    - After the confirm, Studio removes the live item from the draft and shows the tab **Live Shows**. Review shows the removal before the write, as each change of the draft.
    - The tab **Live Shows** has no remove control on its rows. The removal adds no rule to the UI. A rule of the core about a live item with a grant stays in the core.
39. Studio does not write a live item with no `start`, with a `status` that is not `pending`, `live` or `ended`, or with no stream. ADR 0161 items 1 to 9 give the checks of the stream and the warning for a live item with no content link. A refusal stops the write before any change on a host, as for an incorrect address.
40. "Go live" changes the status from `pending` to `live`, and "End the show" changes it from `live` to `ended`. Each one changes the draft and opens the review sheet. The user writes, and the announcement follows.
41. An `ended` live item can get a recording. The user imports an audio file, as for a track ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md)), and the enclosure of the live item points to it.

### Connected Apps

42. A connected app is a program of the user, for example musicindex-live-publisher, that asks Studio to write a live item change. Studio owns the vault, the collection and the write. The other app does not open them.
43. The user adds a connected app in Studio with a name. The app pairs with a one-time pairing text, and signs each request with its own key. Studio keeps no token of an app (ADR 0110 items 1 to 11). The user can remove the app at any time.
44. The routes for connected apps are under `/app/v1/`. Each request needs a signed event of the key of an active app, and no other credential (ADR 0110 items 12 to 16). The pairing request has the signed event of a new key (ADR 0110 items 10 and 13). The routes need a loopback `Host` on the desktop, and the server serves them on the public origin (ADR 0058 item 16). The `/api/` routes do not accept the credential of an app. Thus the rule of [ADR 0017](0017-local-api-access.md) for `/api/` does not change.
45. A connected app can only list its show grants, change the status of a granted live item, and set or remove its `podcast:liveValue`. ADR 0110 also gives the routes of the pairing, of the identity of the app, and of the wrapper of a grant (ADR 0110 item 23).

### Show Grants

46. When a diff adds or changes a `pending` live item, the review sheet has a grant check box. It names the app, for example "Let {app} start and end this show, and set its live value" (ADR 0110 item 19). The diff lists the grant, and the grant is a part of the diff ID.
47. The approved write records the grant. A grant names the feed, the `guid` of one live item and one app. It keeps the live value origin of that app (ADR 0110 items 19 and 25).
48. A grant covers only these changes of that live item: the status from `pending` to `live`, the status from `live` to `ended`, and the set, change or removal of `podcast:liveValue`. A granted write also sets `lastBuildDate` (ADR 0127 item 21). Each other change needs a normal diff.
49. A grant ends at the first of these events:
    - A write sets the status to `ended`.
    - 24 hours pass after the `end` time, or after `start` when there is no `end`.
    - The live item leaves the feed.
    - The user removes the grant.
50. For a granted change, Studio reads the hosted file, applies only the change, and makes its diff. Studio checks that the diff changes nothing that the grant does not cover, and the check ignores `lastBuildDate` (ADR 0127 item 21). Then it writes through `pipeline::write_draft` with the ID of that diff. The snapshot, the conflict check and the announcement are the same as for each write. A granted write always announces, and it has no check box (ADR 0136 item 6).
51. Studio refuses a granted change when the feed has a draft with changes, or when the grant has ended. A granted write uses the grant vault of ADR 0058, so it also works while the user vault is locked (ADR 0058 items 11 to 13). Studio thus never writes an edit of the user that the user has not approved.
52. The page of a live show lists each granted write, with the name of the app and the time. It also lists each refused granted write, with the cause (ADR 0110 item 43).

### The Change To ADR 0007

53. The ADR 0007 invariant "No write occurs without a diff that the user has seen" becomes: no write occurs without a diff that the user has seen, or a change that a show grant of the user covers.

## Alternatives Considered

- Keep ADR 0041, ADR 0052 and ADR 0054 current, each with its "Changed by" lines. Not chosen: the global decision of 2026-11-14 permits three changes at most, and each of the three ADRs had five.
- Put this area into one ADR with the ADRs that ADR 0135 or ADR 0071 also change. Not chosen: those ADRs have different subjects. The [ADR consolidation plan](../plans/adr-consolidation-plan.md) gives the areas.

The superseded ADRs give the alternatives of each original decision.

## Consequences

- One ADR gives the publisher feeds, the announcements and the show grants. A reader finds each item here, or at the item of a named current ADR.
- One link changes two drafts. The user writes each one.
- Studio depends on a public service of Podcast Index. When the hub stops or changes, the announcements fail, and the feed page shows each failure. The writes continue.
- On the desktop, an announcement waits only while Studio runs. A stop of Studio for more than 24 hours loses the waiting announcements, and the feed page shows them as failed.
- A test cannot send a request to the real hub (ADR 0045). A local test hub takes its place.
- A granted "Go live" write waits 30 seconds before its announcement (item 28), and the hub sends an update podping, not a `live` podping. Stophammer crawls the feed after that podping.
- This ADR owns the contract of the connected apps. musicindex-live-publisher cites it.
- Code comments cite items of the superseded ADRs. The item map gives the new item.
- A subsequent change to this area changes this ADR with a "Changes" line. After three changes, a new current ADR replaces this ADR.

## Non-Goals

- `podcast:podroll` and the `feedImg` attribute in a publisher feed.
- A change of the `title` or the `feedUrl` of a link after a change of the title or the address of the other feed.
- A link to a feed that is not in the collection.

## Invariants

- No feed changes without a diff that the user saw, or a change that a show grant of the user covers (item 53). The same rule applies to an announcement, with the exception of the button of item 35.
- A link has the same `rel` on its two sides.
- A link does not change the feed GUID or an item GUID (ADR 0008).
- An announcement never changes or stops a write.
- No test sends a request to a host outside the computer (ADR 0045).
- An announcement uses no secret.
- A connected app never gets a secret of the vault.
- A granted write changes only what the grant covers.
- A write to a host occurs only through `pipeline::write_draft`.

## How This Decision Is Checked

Mechanical. The tests of the superseded ADRs and of the changing ADRs enforce these items, and this ADR adds no test:

- The links (items 5 to 14):
    - A link gives the two `remoteItem` elements of items 6 and 7 in the two drafts, with the same `rel`. A removal takes the two elements out.
    - A link to a release with no draft starts its draft.
    - A change of the roles changes each `rel` in each draft of item 11.
    - A link that Studio did not make stays in the draft after each other change.
- The announcements (items 15 to 35):
    - A complete write puts the planned announcements into the queue, and the worker sends them to a local test hub.
    - A write with the check box off sends nothing.
    - A move write announces the address of item 22.
    - A change of the check box changes the diff ID.
    - The retry times, the failed state after five failures, and the replacement of a waiting announcement.
    - A waiting announcement older than 24 hours is failed at the start.
- The live items and the grants (items 36 to 52):
    - A test adds a live item, writes it, then does "Go live" and "End the show" through the draft. The written feed has each status in sequence and the same `guid`.
    - A test refuses a write of a live item with no stream, before any change on a host.
    - `/app/v1/` refuses the UI access token, and `/api/` refuses the credential of an app.
    - A test client sets the status to `live`, sets the `podcast:liveValue`, and sets the status to `ended`. The hosted file has each change, and the grant ends after the last one.
    - Tests refuse a granted change that is not covered and a change after the end of the grant. A test refuses a change while the feed has a draft with changes.

Visual: no new visual check. The visual check of ADR 0041 passed (`docs/reviews/2026-10-25-phase-8-visual-check.md`).

Human: two human tests of the superseded ADRs are open:

- ADR 0052: after a write of a real feed, the hub accepts the request. A podping with the feed address appears, and Stophammer crawls the feed.
- ADR 0054: a connected app uses the contract against a real Studio during a test show.

## Item Map

The map gives the item of this ADR for each item of a superseded ADR.

| Superseded item | New item |
| --- | --- |
| ADR 0041 items 1 to 14 | 1 to 14 |
| ADR 0052 items 1 to 4 | 15 to 18 |
| ADR 0052 items 5 and 6 | 19 and 20 |
| ADR 0052 items 7 to 21 | 21 to 35 |
| ADR 0054 items 1 to 3 | 36 to 38 |
| ADR 0054 item 4 | 39 |
| ADR 0054 items 5 to 18 | 40 to 53 |
| ADR 0041, ADR 0052 and ADR 0054 "Invariants" | "Invariants" |
| ADR 0041, ADR 0052 and ADR 0054 "Verification" | "How This Decision Is Checked" |
