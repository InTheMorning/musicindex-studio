# ADR 0067: A Read-Only Feed Opens In The Editor

- Status: Accepted
- Date: 2026-10-27
- Owner: operator
- Accepted: 2026-10-27 by the operator.
- Corrected: 2026-10-27, before the first task. Item 7 said that the feed keeps its path. A read-only feed has no path, so item 7 now gives the path of ADR 0035 item 13.

## Context

A read-only feed is a feed with no connector ([ADR 0005](0005-collection-and-first-run-discovery.md) item 3, [ADR 0006](0006-connectors-and-the-v1-set.md) item 5). The UI opens the editor only for a feed with a connector (`isEditable` in `crates/api/ui/shell.js`). Each other feed gets the read-only view: the large hero and four facts. The core starts a draft only through a connector (`ensure_draft` in `crates/api/src/routes.rs`).

On 2026-10-27 the operator examined a read-only album in this view. The view has these defects:

- The hero shows the Music Index mark, not the cover. The outline shows the cover of the same feed.
- The view keeps the large hero that [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) removed from the editor page.
- The "File" fact tells the user to connect a folder, also for a feed on a remote host.
- The view shows no track, and the outline has only "Feed". The user cannot see how Studio edits the feed.

Studio has the XML of each read-only feed in its release folder ([ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 4). On 2026-10-27 the operator decided that a read-only feed opens in the editor with a draft. The user chooses a host at the write. The operator also decided that Review gives two choices: connect the host of the file, or move the feed to a different host.

A move from a host that Studio cannot write cannot use the redirect or the pointer file of [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) items 19 and 20. The research note `research_notes/aggregation/podcastindex-guid-behavior.md` in `../musicindex-research` quotes the namespace rule: the `podcast:guid` stays the same when a feed moves to a new host. The note did not examine how Podcast Index handles two addresses with the same GUID.

## Decision

### The Editor

1. A read-only feed of a type that the editor opens, an album, a publisher feed or a playlist, opens in the editor. A feed of a different type keeps the read-only view.
2. The editor of a read-only feed has the same pages, fields and outline as for a feed with a connector. The eyebrow of the compact header shows "read-only".
3. The core keeps a draft for a read-only feed. The base of the draft is the XML of the last public read, and its version is the SHA-256 of those bytes.
4. "Read again" reads the public address. When the public file changed after the start of the draft, the draft keeps its base. The write then shows a conflict ([ADR 0007](0007-draft-diff-write-and-announce.md) item 3).

### The Write

5. The draft of a read-only feed has no host. Review shows the diff, and the write needs a host first. Review gives two choices: "Connect the host of this file" and "Move to a host".
6. "Connect the host of this file" opens Hosts. When a connector links the feed by its `podcast:guid` ([ADR 0020](0020-first-run-asks-one-question.md)), the feed gets that connector and keeps its address. The draft stays. When the SHA-256 of the host file is not the base of the draft, the write shows a conflict.
7. "Move to a host" selects a connected host. That host becomes the host of the feed XML and the default host of new files ([ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 12). A read-only feed has no path. On the new host, the path of the feed XML is the release folder, then the file name of the public address (ADR 0035 item 13). The feed keeps its `podcast:guid` and each item GUID ([ADR 0008](0008-feed-types-and-feed-identity.md)).
8. A move from a read-only feed sets no redirect and writes no pointer file, because Studio cannot write the first host. This changes ADR 0035 items 19 and 20 only for a move from a read-only feed.
9. Before the write of such a move, Review tells the user that the file at the address before the move stays and does not change. An app that reads that address continues to receive the feed from before the move.
10. The write announces the new address and the address before the move, as [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) item 8 says.
11. Each file of the feed keeps its address after the move. The user can move each file on the Files page ([ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md)).

### What Does Not Change

12. A link between a publisher feed and a release needs a connector for each feed ([ADR 0041](0041-publisher-feeds-and-two-way-links.md) item 5). A read-only feed has no link until it has a host.
13. No write occurs without a diff that the user saw. The write occurs only through `pipeline::write_draft` ([ADR 0015](0015-connector-interface-and-write-pipeline.md)).

## Alternatives Considered

- An editor with disabled fields and no draft. Not chosen by the operator: the user cannot try an edit.
- A read-only list of the tracks in the read-only view. Not chosen by the operator: the user does not see how Studio edits.
- Only "Connect the host of this file". Not chosen by the operator: a feed on a service with no connector then stays a draft that Studio cannot write.
- Only "Move to a host". Not chosen by the operator: a user who controls the first host then gets a new address with no need.

## Consequences

- `isEditable` changes, and the editor shows a draft with no host.
- The core starts a draft with no connector, and the write refuses that draft until it has a host.
- The large hero stays only in the read-only view of the other feed types. Its cover and its "File" text need a correction in the task of this ADR.
- After a move from a read-only feed, the address before the move continues to give the feed from before the move. Studio cannot stop that.

## Invariants

- The write of a draft with no host does not occur.
- A move from a read-only feed keeps the `podcast:guid` and each item GUID.
- No write occurs without a diff that the user saw.

## Verification

Mechanical:

- A core test starts a draft for a feed with no connector, and the write refuses it.
- A core test links a connector to a read-only feed with a draft, and a different host file gives a conflict.
- A core test moves a read-only feed to a host, with no redirect and no pointer file, and the feed keeps its GUIDs.
- A Node test: the editor model opens a read-only album, publisher feed and playlist, and does not open a podcast feed.

Visual:

- The editor of a read-only album shows its cover, its tracks and "read-only" in the eyebrow.
- Review of a read-only draft shows the two choices and the warning of item 9.
