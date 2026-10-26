# ADR 0052: Announcements Through The Podcast Index Hub

- Status: Accepted
- Date: 2026-10-25
- Owner: operator
- Amended: 2026-10-26 by operator decision. [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 18 adds one exception to the invariant: a change that a show grant of the user covers.

## Context

[ADR 0007](0007-draft-diff-write-and-announce.md) item 5 tells Studio to send a podping and a Podcast Index pubnotify after a write, with a setting for each. ADR 0007 did not decide the send method. [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 21 tells Studio to give the new address of a moved feed to the musicindex API and Podcast Index. [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 21 keeps ADR 0035 item 21 out of force until Phase 10, and puts the announcement in the same diff. [ADR 0041](0041-publisher-feeds-and-two-way-links.md) leaves the announcement of a publisher feed to Phase 10.

The research gives these observations:

- PodSovereign sends `GET https://hub.podcastindex.org/pubnotify?url=<feed>&podping=1`. It uses no key and no Hive account. It waits 30 seconds, so that a burst of changes gives one request. It tries again at 30 seconds, 1, 2, 4 and 8 minutes. A response with the JSON `status` `false` is a refusal (`musicindex-research`, `sources/music-feeds/hosting/podsovereign/server/src/podping/podping.ts`, and `research_notes/studio/podsovereign-comparison.md`).
- MSP 2.0 sends the pubnotify to `https://api.podcastindex.org/api/1.0/hub/pubnotify` with no key. It sends the podping through its own `podping-hivepinger` service, with a Hive account and a posting key of MSP (`sources/music-feeds/editors/msp-2.0/api/pubnotify.ts` and `docs/superpowers/specs/2026-04-17-podping-hivepinger-design.md`).
- The musicindex API has no route that accepts a notice of a changed feed (`../musicindex/api.json`). Stophammer reads podpings, and a podping starts a crawl (stophammer ADR 0062 and ADR 0064).
- Only the PodSovereign source shows the `podping=1` parameter. The research did not confirm with a live request that the hub sends the podping.

On 2026-10-25 the operator selected the method, the settings, the queue and the visibility below.

## Decision

### The Method

1. Studio announces a feed with one request: `GET <hub>?url=<feed address>`. The query has `podping=1` when the podping setting is on. The hub is `https://hub.podcastindex.org/pubnotify`. A shell can select a different hub, for example a local test server.
2. The request uses no key, no Hive account and no secret.
3. A response with the status 200 is a success. A response with the JSON `status` `false` or `"false"` is a refusal. Each other status, each refusal and each network failure is a failure.
4. Studio does not send a separate notice to the musicindex API. Stophammer reads the podping and crawls the feed. This changes ADR 0035 item 21: "the musicindex API" is reached through the podping. With this change, ADR 0035 item 21 and ADR 0038 item 21 are in force.

### The Settings

5. Studio has two global settings. "Tell Podcast Index after each write" controls the request. "Send a podping with it" controls `podping=1`. Both are on at first.
6. When the Podcast Index setting is off, Studio sends no request, and the podping setting has no effect. The hub sends the podping, so a podping with no pubnotify is not possible.

### The Addresses

7. Studio announces the public address of the feed ([ADR 0021](0021-public-address-of-a-feed.md)).
8. A write with a move of the feed XML announces two addresses: the new address, then the address before the move. An index that knows only the address from before then reads the redirect or the `itunes:new-feed-url` file.
9. A feed with no public address gets no announcement. A write is not blocked for this reason.
10. The first write of a new release or a new publisher feed gets an announcement, as each other write does.

### The Diff

11. The diff lists each planned announcement, with its address and its podping setting. When Studio plans no announcement, the diff gives the reason: the setting is off, or the feed has no public address.
12. The planned announcements are a part of the diff ID. A change of a setting after the diff makes the diff not current, and the user approves the new diff.

### The Queue

13. A write that completes with no error puts each planned announcement into a queue in the collection. A write that stops with an error puts nothing into the queue.
14. Studio sends an announcement 30 seconds after the write, so that the host or its CDN serves the new file.
15. After a failure, Studio tries again at 1, 2, 4 and 8 minutes after the failure. After the fifth failure, the announcement is failed.
16. A new announcement for an address that waits in the queue replaces the waiting one. Its 30 seconds start again. An announcement that Studio sends at that time is not replaced.
17. The write does not wait for the announcement. A failed announcement does not change the write.
18. At each start, Studio marks each waiting announcement that is older than 24 hours as failed. It sends each younger one at its time.
19. Studio keeps the last 10 finished announcements of each feed, and deletes older ones.

### The Status

20. The page of a feed shows the last announcement: waiting with its send time, sent with its time, or failed with its time and its error text.
21. Studio has no button that announces a feed with no write. ADR 0007 rejected manual announce buttons only, and the operator did not select a button in addition to the automatic send.

## Alternatives Considered

- The hub for the pubnotify, and a `podping-hivepinger` service with a Hive account of Studio for the podping, as MSP 2.0 does. Not chosen by the operator. Studio would operate a service and keep a Hive posting key, and a desktop app cannot keep that key safe.
- The pubnotify only, with no podping. Not chosen by the operator. musicindex would then see a change only at its next poll.
- A setting for each feed, in addition to or in place of the global settings. Not chosen by the operator. It adds UI and tests, and the global settings cover the need now.
- One request at once after the write, with no queue. Not chosen by the operator. A CDN can serve the old file for some seconds. Stophammer ADR 0062 records a podping that arrived before the CDN served the new file.
- An "Announce again" button. Not chosen by the operator.
- Announce only the new address after a feed move. Not chosen by the operator.
- A check box for the announcement at the first write of a new feed. Not chosen by the operator. The diff line shows the announcement before the write.

## Consequences

- The open question of ADR 0007 about the podping method is closed.
- Studio depends on a public service of Podcast Index. When the hub stops or changes, the announcements fail, and the feed page shows each failure. The writes continue.
- On the desktop, an announcement waits only while Studio runs. A stop of Studio for more than 24 hours loses the waiting announcements, and the feed page shows them as failed.
- The `announce` field of the diff, which was always `false`, goes out. The diff gets the planned announcements and the reason for no announcement.
- A test cannot send a request to the real hub ([ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md)). A local test hub takes its place. A human test with a real feed is necessary to confirm `podping=1`.
- The Server Shell phase uses the same queue for each user. The operator of the server can select the hub.

## Verification

Mechanical:

- A test confirms that a complete write puts the planned announcements into the queue, and that the worker sends them to a local test hub.
- A test confirms that a write with the Podcast Index setting off sends nothing.
- A test confirms `podping=1` with the podping setting on, and no `podping` parameter with it off.
- A test confirms the two addresses of a feed move, in their sequence.
- A test confirms that a change of a setting changes the diff ID.
- A test confirms the retry times, the failed state after five failures, and the replacement of a waiting announcement.
- A test confirms that a waiting announcement older than 24 hours is failed at the start.

Visual:

- The review sheet shows the announcement lines, and the reason when there is no announcement.
- The feed page shows the last announcement in each of its three states.
- The Hosts page shows the two settings. The podping setting is not available when the Podcast Index setting is off.

Human test:

- After a write of a real feed, the hub accepts the request, a podping with the feed address appears, and stophammer crawls the feed.

## Invariants

- No announcement occurs without a diff that the user has seen, or a change that a show grant of the user covers ([ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0054](0054-live-items-connected-apps-and-show-grants.md)).
- An announcement never changes or stops a write.
- No test sends a request to a host outside the computer ([ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md)).
- An announcement uses no secret.
