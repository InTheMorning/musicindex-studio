# ADR 0204: Feed Files, Feed Moves And The Server Shell

- Status: Accepted
- Date: 2026-11-14
- Owner: operator
- Accepted: 2026-11-14. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0038](../archive/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md), Published Feeds, Downloads, The Folder Scan And Feed Moves.
- Supersedes: [ADR 0055](../archive/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md), The Server Shell: User Folders, Accounts, Sessions And The Vault Window.
- Uses: [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) and [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) for the library and its records. [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) for the announcement of a move. [ADR 0013](0013-vault-format-and-cryptography.md), [ADR 0014](0014-collection-in-sqlite.md), [ADR 0017](0017-local-api-access.md) and [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) for the vault, the collection, the local access and the sign-in methods.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0105](0105-a-public-server-holds-no-assets.md) (a server that holds no assets), [ADR 0106](0106-a-sensitive-account-change-needs-a-recent-unlock.md) (the limits of failed sign-ins), [ADR 0107](0107-a-user-can-export-and-import-an-account.md) (the export and the import of an account), [ADR 0110](0110-a-connected-app-signs-each-request-with-its-own-key.md) (the index of the apps), [ADR 0126](0126-studio-refuses-a-bad-move-link.md) (the check of a move link), [ADR 0128](0128-a-moved-feed-names-its-own-address.md) (the tag of a moved feed), [ADR 0142](0142-a-user-selects-the-time-before-the-vault-locks.md) (the lock time), [ADR 0179](0179-a-release-keeps-one-library-record-for-each-file.md) (one record for each file), [ADR 0189](0189-a-move-sends-one-podping-of-the-old-address.md) (the address that a move announces). [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) and [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) give the exceptions for a granted write.

## Context

- ADR 0038 gave the published feeds in the library, the downloads, the folder scan and the move of the feed XML. ADR 0055 gave the server shell: the user folders, the accounts, the sessions and the vault window.
- Subsequent ADRs changed four items of ADR 0038 and five items of ADR 0055. ADR 0105 changed the two ADRs for a server with `--hold-assets off`.
- The global decision of 2026-11-14 (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR is that ADR for the feed files, the feed moves and the server shell.
- ADR 0052 item 4 put ADR 0038 item 21 in force. ADR 0038 item 18 also said that the diff tells the user that Studio does not tell the index. That sentence is not true after ADR 0052 item 4, and the code has no such text. This ADR leaves out that sentence, and item 21 gives the announcement.
- The superseded ADRs keep their context, their alternatives and their checks. They are in `docs/archive/adr/` for research. The item map at the end of this ADR gives the new item of each superseded item.

## Decision

### Remote Library Files

1. Each address of a feed with a release folder is a library file: each enclosure, each alternative enclosure source and each image. Each address that has no library file gets a row with `local` false and no SHA-256. The `source_url` of the row is the address.
    - Studio reads the planned address and the copies of each local file. When one of these reads fails, Studio adds no remote row for that feed in that pass. It writes the error to the log. The next record time of item 3 tries again (ADR 0179 item 1).
    - A feed address that is not on a connected file host still gets a remote row (ADR 0179 item 2).
2. The path of a remote file in its release folder is the last segment of its address, with the percent encoding decoded. When that path is in use, Studio adds ` 2`, ` 3` and so on before the extension. The size is the enclosure length when the feed gives one, otherwise 0. The media type is the enclosure type, otherwise the type of the extension.
3. Studio records the remote files at these times: when the user adds a published feed, after each read of the hosted file and after each write. At the start, it records them for each feed with a release folder. A remote row whose address is no longer in the hosted feed goes out, because it has no bytes. ADR 0179 items 3 and 4 give the removal of a second record of one file at these times.
4. A published feed gets its release folder and its feed XML when the user adds it, as in ADR 0035 item 5. This applies to a read-only feed and to a feed that a folder match links.
5. A copy of a remote file is each host file with the `source_url` as its public address. When Studio knows the SHA-256, each host file with the same SHA-256 is also a copy.

### Downloads

6. The Files page shows the number and the total size of the files that are not in the library, with "Download all". Each remote file has its own "Download". No download starts without a selection of the user.
7. Studio downloads two files at a time for all feeds. The Files page shows the progress of each file, with "Stop".
8. When a connected host has a copy of the file, Studio reads it through the connector. Otherwise it uses HTTP. An HTTP download writes into a hidden partial file in the release folder. After a stop or a restart, it continues from the end of the partial file with a range request.
9. The collection records each download that is not complete. At the start, Studio continues each one.
10. Studio refuses a download when the size is more than `download_max_file_bytes`, or when the free space after the download is less than `download_min_free_bytes` (ADR 0036 item 17). The error names the setting.
11. After a download, the row gets `local` true, its size and its SHA-256. When the size is different from the enclosure length in the feed, the Files page shows the two values. Studio does not change the feed for it.
12. A copy or a move of a remote file asks the user first, with the size of the download. Studio then adds the draft operation and starts the download. The diff marks the operation as "Waiting for the download", and the write refuses the draft until each download of the draft is complete.

### The Folder Scan

13. A library file records a stamp: its size and its modification time. The folder scan compares each release folder with the records, with no read of a file whose stamp did not change.
14. The scan gives these differences:
    - New: a file in a release folder with no row. The feed XML, hidden files and partial files are not differences.
    - Missing: a row with `local` true and no file.
    - Changed: a file whose stamp and SHA-256 are different from its row.
15. Studio scans at the start and when the user asks. It changes nothing without a decision of the user. The actions are:
    - New: "Add to the library" gives a library file that the feed does not use.
    - Missing: "Download again" when a copy or a `source_url` is known. "Forget the file" sets `local` false, or removes the row when Studio knows no copy and no address.
    - Changed: "Use the new file" records the new size, stamp and SHA-256. Each address of the file in the draft changes to its planned address, which is the hash name of ADR 0035 item 16 when a different file is at the clean path. Each copy of the bytes from before that the draft referred to gets a deletion in the draft. "Restore" downloads the recorded bytes from a copy, and puts them back into the release folder.

### A Move Of The Feed XML

16. The Files page has "Host of the feed XML". A different host puts a feed move into the draft. The feed XML keeps its path on the new host, so the files of the release keep their paths too (ADR 0035 item 13).
17. `Connector` has `can_redirect()` and `set_redirect(path, to_url)`. A connector that cannot make an HTTP 301 redirect gives false. The local folder connector cannot.
18. The diff shows the new address, and the method at the address from before: a redirect, or a pointer file with `itunes:new-feed-url`.
19. Each move gives the feed on the new host an `itunes:new-feed-url` with the URL of the destination (ADR 0128 items 1 to 6 and 12). The write does these steps in this sequence (ADR 0126 item 16):
    1. The uploads and the copies.
    2. The feed XML on the new host, with its new self link.
    3. Stage 2 of ADR 0126, which reads the destination.
    4. The redirect, or the pointer file at the address from before.
    5. The new host, path and address of the feed in the collection.
    6. The check of ADR 0043 items 7 and 8.
    7. The deletions.
20. The collection records each pointer file. Studio refuses a deletion of a pointer file, as ADR 0035 item 20 says. A move can go back to a host with a pointer file of this feed at the path. That pointer file then becomes the feed, and the host from before gets its own pointer file (ADR 0126 items 21 and 22).

### Announcement

21. The write of a move announces the feed as [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) gives. ADR 0255 item 18 puts ADR 0035 item 21 in force: Stophammer reads the podping, so Studio sends no separate notice to the musicindex API. A write with a move announces one address: the address before the move. A move with no address before the move announces the new address (ADR 0255 item 22, from [ADR 0189](0189-a-move-sends-one-podping-of-the-old-address.md) items 1 and 2).

### A Server That Holds No Assets

22. On a server with `--hold-assets off`, items 6 to 10 and items 13 to 15 do not apply. That server has no downloads into the library and no folder scan (ADR 0105 item 14).

### One Folder For Each User

23. The server keeps one data folder for each user: `<data>/users/<user folder>/`. The folder name is 16 random bytes as hexadecimal. It is not the email address or the public key.
24. Each user folder has the files of a desktop data folder: `collection.sqlite`, `vault.json`, `incoming/` and the library folder `library/`. A user folder holds each part of an account that can move to a different server (ADR 0107 item 2). The server keeps no collection of a user in a shared database. Each user has one vault file.
25. The library of a user is `library/` in the user folder (ADR 0035 item 4). The server sets it when it makes the user folder. The collection keeps the library as a path relative to the data folder (ADR 0107 item 1). The user cannot set a different library folder.
26. Each user has one `Workspace` with its own lock, its own download queue and its own announcement worker. The server opens a workspace at the first request of its user. At the start, it also opens each workspace with a waiting announcement or a queued download (item 9, [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) items 27 to 32).
27. The `/api/v1/` routes and their handlers are the same in the two shells. The API crate gives a router for one workspace with no access layer. The desktop adds the layer of ADR 0017. The server adds the session layer, finds the workspace of the session, and sends the request to the router of that workspace.

### The Accounts

28. The server keeps a separate database `<data>/server.sqlite`. It holds the accounts, the sign-in methods of each account, the sessions, the sign-in challenges, the invite codes and the index of the connected apps. It holds no feed data and no secret. It holds only the parts of an account that belong to one server. The server makes the index of the apps again from the collections at each start (ADR 0107 item 2, ADR 0110 item 39).
29. The operator selects open sign-up or sign-up with an invite code. The default is an invite code. `studio-server invite` makes a single-use code. A personal server is a server with one account and no open sign-up. A sign-up can import an account. On a server with invite codes, the import needs an invite code (ADR 0107 item 9).
30. The server sends no email. An email address is only the name of an account. The server keeps it in lowercase with no spaces at the start or the end. The server does not check that the user owns the address.
31. A user can delete the account. The server asks for an unlock of the vault first (ADR 0106 item 2). Then it removes the user folder, the account rows and each session of the user. It does not change a feed on a host.

### Sessions

32. A sign-in gives a session. The session token is 32 random bytes. The server keeps only its SHA-256.
33. The browser keeps the token in the cookie `__Host-studio-session` with `HttpOnly`, `Secure`, `SameSite=Strict` and `Path=/`. The UI script cannot read it.
34. Each session also has a CSRF token: the SHA-256 of a fixed label and the session token. The sign-in response and `GET /auth/v1/session` give it to the UI. The UI sends it in the `X-Studio-CSRF` header with each `/api/` request. The server sends no CORS headers, so a page of a different site cannot send this header. A request without the correct header gets 403 `csrf_required`.
35. A session ends after 7 days with no request, after 30 days in total, at sign-out, and when the user deletes the account. The operator can change the two limits. The server keeps the sessions in `server.sqlite`, so a restart does not end them.

### The Vault Window

36. A session and an open vault are two different states. With a session and a locked vault, a user can read the collection and edit a draft. The user can also see a diff that needs no secret of a host.
37. The vault opens only through an unlock of the user: a sign-in with a passphrase, or an unlock with a method of ADR 0056.
38. The server locks the vault of a user after the lock time of the user with no use of a vault secret. The time starts at `--vault-idle-minutes`, with the default 10 minutes, and `--vault-idle-max-minutes` limits it (ADR 0142 items 1 to 5). The server also locks the vault at "Lock Studio", at sign-out, at the end of the last session of the user, and when the process stops. A lock removes the vault key from memory ([ADR 0003](0003-user-unlocked-secret-vault.md), ADR 0013 item 15).
39. A route that needs a secret while the vault is locked gives 423 `vault_locked`, as on the desktop. The UI then asks for an unlock and sends the request again. A granted write uses the grant vault while the user vault is locked ([ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) items 12 and 13).

### Server Access

40. The server listens on an address that the operator gives. The default is `127.0.0.1:8080`, behind a reverse proxy with TLS. The operator gives the public origin, such as `https://studio.musicindex.org`. The origin must use `https`. Only a loopback origin can use `http`, for development and checks.
41. The server refuses each request with a `Host` that is not the host of the public origin, with 403. A request that changes data must also have an `Origin` header equal to the public origin.
42. The UI and its security headers are the same in the two shells ([ADR 0016](0016-ui-without-a-build-step.md)). The UI asks `GET /auth/v1/session` first. The desktop answers `{"mode": "local"}`. The server answers with the mode `server` and the state of the session. The UI shows the sign-in page when the server has no session.

### Limits

43. Each user has a storage limit for the user folder. The default is 10 GiB, and the operator can change it. An upload or a download that goes over the limit stops with 507 `storage_full`. With `--hold-assets off`, the limit counts the pending files, the files that no host has, the temporary files and the databases (ADR 0105 item 20).
44. The server runs a maximum of 2 Argon2id derivations at the same time. The operator can change the number. A sign-in that must wait more than 10 seconds gets 503 `busy`.
45. The server permits 10 failed sign-ins in each hour for each pair of account and client address (ADR 0106 item 5). It permits 100 for each account from all addresses, and 30 for each client address. After that, it gives 429 `too_many_attempts`. The client address comes from the header that the operator names for the reverse proxy.

### Functions Of The Desktop Only

46. The server refuses the local folder connector with 403 `not_on_server`. That connector reads and writes folders of the server computer.
47. The server refuses `PUT /library` with a folder other than the folder of item 25.

## Alternatives Considered

- Keep ADR 0038 and ADR 0055 current, each with its "Changed by" lines. Not chosen: the global decision of 2026-11-14 permits three changes at most, and each of the two ADRs had four.
- Put this area into one ADR with the ADRs that ADR 0135 or ADR 0071 also change. Not chosen: those ADRs have different subjects. The [ADR consolidation plan](../plans/adr-consolidation-plan.md) gives the areas.

The superseded ADRs give the alternatives of each original decision.

## Consequences

- One ADR gives the feed files, the feed moves and the server shell. A reader finds each item here, or at the item of a named current ADR.
- The SHA-256 of a library file is optional, and each use of it handles a remote file.
- The API keeps a download queue with two workers. Each connector tells if it can make a redirect.
- The core does not change for the user folders. The isolation of users is in the folder structure.
- A user stays signed in on a phone for days, and opens the vault for each write session. A server restart locks each vault, but keeps each session.
- An email address can be in use by a person who does not own it. The server does not use the address for any message.
- Code comments cite items of the superseded ADRs. The item map gives the new item.
- A subsequent change to this area changes this ADR with a "Changes" line. After three changes, a new current ADR replaces this ADR.

## Invariants

- No download starts without a selection of the user.
- The write refuses a draft with an operation that waits for a download.
- Studio does not delete a pointer file.
- The scan reads only a file whose stamp changed.
- No upload, copy, redirect, pointer file or deletion occurs without a diff that the user saw. A change that a show grant of the user covers is the exception ([ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) item 53).
- A session can read and write only the folder of its own user.
- The server keeps no vault key after a lock, a sign-out or the lock time.
- No session token or vault key is in a log, a database row or an API response. The one exception is the vault key for a signer, after a recent unlock ([ADR 0106](0106-a-sensitive-account-change-needs-a-recent-unlock.md) item 2).

## How This Decision Is Checked

Mechanical. The tests of the superseded ADRs enforce these items, and this ADR adds no test:

- `crates/core/src/download.rs` and `crates/core/src/folder_scan.rs`: the downloads (items 6 to 11) and the folder scan (items 13 to 15).
- `crates/core/src/move_link.rs` and `crates/api/src/tests.rs`: the move of the feed XML and its sequence (items 16 to 20).
- `crates/server/src/tests.rs`:
    - Two users cannot read or change the data of the other user (items 23 to 27).
    - `/api/` refuses a request with no session, with a session of an ended account, and with no correct `X-Studio-CSRF` header (item 34).
    - A `Host` other than the public host gets 403 (item 41).
    - With a short lock time, the vault locks, and a route that needs a secret gives 423 (items 38 and 39).
    - `server.sqlite` contains no session token and no CSRF token (item 32).
    - The server refuses the local folder connector and a different library folder (items 46 and 47).

Visual: no new visual check. The visual checks of the superseded ADRs passed.

Human: no new human check.

## Item Map

The map gives the item of this ADR for each item of a superseded ADR.

| Superseded item | New item |
| --- | --- |
| ADR 0038 items 1 to 20 | 1 to 20 |
| ADR 0038 item 21 | 21 |
| ADR 0038 "Invariants" | "Invariants" |
| ADR 0055 items 1 to 25 | 23 to 47 |
| ADR 0055 "Verification" | "How This Decision Is Checked" |
| ADR 0055 "Invariants" | "Invariants" |
| ADR 0105 item 14, for ADR 0038 | 22 |
