# ADR 0122: A New User Starts With A Default File Host

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator: "we should simply force user to configure a host at the start, with a default setting to an intuitive writable local folder or temporary memory", and "use a placeholder bogus guid with an explanation why it can not be generated at this time". The operator selected the temporary area as the default on a server.
- Changed: on 2026-11-03, the human test found that a track cannot go on a folder with no public address. The operator selected the placeholder address of items 12 to 15. The same test found that the links depended on the sequence of the writes.
- Supersedes: [ADR 0121](../archive/adr/0121-a-folder-with-no-public-address-holds-practice-pages.md), practice pages.
- Changes: [ADR 0119](0119-a-new-user-starts-as-a-musician-or-a-label.md) item 6, step 2. [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) items 6 to 9, for a file host with no public address.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) for feed identity. [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) for a feed move. [ADR 0041](0041-publisher-feeds-and-two-way-links.md) items 6, 7 and 12 for the links. [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) for the user folder.

## Context

- ADR 0121 made practice pages: a page with a temporary GUID, no write, a badge and "Publish for Real". On 2026-11-02 the operator said: "the practice page is probably the wrong idea".
- The GUID of a new feed is the UUIDv5 of its public address (ADR 0032). A folder on this computer has no public address, so Studio cannot make the real GUID there.
- A write to a file host with no public address works. Studio skips the announcement (`AnnounceSkip::NoAddress`).
- The artist page and its albums name each other by `feedGuid` (ADR 0041 item 12). A placeholder GUID must thus be different for each page.
- A server shell has no folder on the computer of the user.

## Decision

### Step 2 Is Required And Has A Default

1. Step 2 of the Welcome sheet, "Where it lives", always shows, also with a connected file host. The user cannot make the first page without a file host.
2. On the desktop, the first row is "Folder on this computer", selected at the start. Its folder starts as `~/musicindex/pages`, in the same folder as the default library folder. Studio makes the folder when the user continues. The public address of the folder is optional.
3. On a server, the first row is "Try it here (temporary)", selected at the start. Its text is "Kept for 7 days. Pick a real file host to keep your pages." Items 9 and 10 give the temporary area.
4. The other rows are "Nostr site (nsite)", "S3 or Cloudflare R2" and, on the desktop, "SFTP server". With a connected file host, the step also shows a select menu of the connected file hosts.

### The Placeholder GUID

5. A new feed on a file host with no public address gets a placeholder GUID: the text `pending-` and a random UUID. Each page has its own placeholder, so the links between pages work.
6. Feed facts shows the placeholder with the text "Studio cannot make the GUID yet. A GUID comes from the public address of the feed, and this file host has none. Studio sets the GUID when the feed gets a public address."
7. A feed with a placeholder GUID can be written to its file host, as each feed can. Studio announces nothing for it, because it has no public address.
8. The feed gets its real GUID, the UUIDv5 of its public address, when it first has a public address:
    - When the user moves it to a file host with a public address (ADR 0038). The core sets the GUID with the move of the draft.
    - When the user gives its file host a public address. The core sets the GUID of each feed of that file host with a placeholder.
    - In the same step, the core changes each `feedGuid` and `feedUrl` that named the placeholder, in the drafts of the collection. A written feed with no draft that names the placeholder gets a draft from its file. This also applies to a feed with a GUID from a public address. The user writes the changed drafts through Review (ADR 0007).

### The Temporary Area Of A Server

9. On a server, the temporary area is a folder in the user folder of the account, with the connector kind `temporary`. It has no public address, so its feeds get placeholder GUIDs. A user can add one temporary area only.
10. The server deletes the files of a temporary area 7 days after its last write. The feeds and their drafts stay in the collection. A feed of the area then shows "The temporary area has expired. Move this feed to a real file host." A move from the temporary area to a real file host uses item 8.

### The Placeholder Address Of A File

12. A file of a feed on a file host with no public address gets a placeholder address: the text `pending:` and the path of the file on the file host, for example `pending:my-album/01 Song.mp3`. The core gives it as the planned address of the file. Thus the import of a track works, and the write puts the file on the file host.
13. A placeholder address gets no OP3 prefix (ADR 0076). Studio reads the OP3 switch from the enclosures, so the switch stays off until the feed has a public address. The user sets it after the move.
14. The page of a track shows the placeholder address with the text "Studio cannot make the address of this file yet. The address comes from the public address of the feed, and this file host has none. Studio sets the address when the feed gets a public address." The player plays the file from the library.
15. At item 8, the core changes each placeholder address in the draft of the feed. The new address is the planned address of the file on the file host with the public address.

### Removed

11. The badge "Practice", "Publish for Real", the error `practice_page`, the route `POST /practice/publish` and the rule that refuses a write go. The code of the placeholder GUID and of the change of the links stays, for item 8.

## Alternatives Considered

- Practice pages (ADR 0121). Superseded by the operator.
- Studio serves the folder at a loopback address, and the GUID comes from that address. Not chosen: two users can get the same GUID.
- One placeholder GUID for all pages. Not chosen: the links between pages use the GUID.

## Consequences

- The core makes a feed with a placeholder GUID on a file host with no public address, and sets the real GUID at item 8.
- The server shell gets the connector kind `temporary` and the deletion of item 10.
- The UI removes the practice parts, and shows the required step 2, the explanation of item 6 and the text of item 10.
- Task files give the work.

## Non-goals

- A temporary area on the desktop.
- A change of a feed GUID after a write to a public address.

## Acceptance Criteria

Mechanical:

- A core test: a new feed on a local folder with no public address has a `pending-` GUID, and two new feeds have two different GUIDs.
- A core test: a write of such a feed puts the file, and queues no announcement.
- A core test: a move to a file host with a public address sets the real GUID and changes the links in the drafts that named the placeholder.
- A core test: a new public address of the file host sets the real GUID of each feed with a placeholder.
- A server test: the temporary area is a folder in the user folder, and the deletion of item 10 removes its files after 7 days.
- A Node test: step 2 shows with a connected file host, and it has its default row on the desktop and on a server.
- A core test: a track of a feed on a folder with no public address has a `pending:` address, and the write puts its file in the folder.
- A core test: a move to a file host with a public address changes each `pending:` address of the draft to the planned address. The write then puts the file on the new file host.
- A Node test: the texts of items 3, 6, 10 and 14, and the player source of a held file with a placeholder address.
- The four commit checks pass.

Visual: a new user on the desktop continues with the default folder, and makes an artist page and an album. The user writes them, then moves them to a file host with a public address. On a server, a new user continues with the temporary area. The check uses 1100 px, and 390 px with touch, in the two themes.

Human: the operator starts as a new user on the desktop and on the server. Then the operator moves the pages to a real file host.
