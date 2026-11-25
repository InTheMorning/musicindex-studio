# ADR 0262: A User Can Export And Import An Account

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25. The global decision on ADR overuse (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each subsequent change.
- Supersedes: [ADR 0107](../archive/adr/0107-a-user-can-export-and-import-an-account.md), A User Can Export And Import An Account.
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) item 9, on 2026-11-25. An import into an empty collection is on the Welcome page or on Overview (item 2).
- Uses: [ADR 0003](0003-user-unlocked-secret-vault.md) item 5 (one vault format in the two shells), [ADR 0013](0013-vault-format-and-cryptography.md), [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) (the sign-in methods and the wrappers), [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) (grant vaults and the index of app tokens), [ADR 0059](0059-uploads-in-parts-and-the-home-screen-app.md) (uploads in parts) and [ADR 0105](0105-a-public-server-holds-no-assets.md) (a server that holds no assets).

## Context

- ADR 0107 gave the account archive, with the vault encrypted and the collection plain.
- ADR 0110 replaced app tokens with public keys and removed grants after import.
- ADR 0149 and ADR 0176 moved the desktop export and import into Collections. A closed collection can be replaced.
- A passkey wrapper belongs to its server origin and cannot open the vault on a different origin (ADR 0056 item 20).
- This ADR keeps the archive format, the refusal conditions and the required checks. The archived ADR 0107 keeps its earlier context and alternatives.

## Decision

### The User Folder Is The Account

1. The collection keeps the library folder as a path relative to the data folder when the library is in the data folder. The desktop keeps an absolute path for a library outside the data folder (ADR 0036 item 1).
2. A user folder holds each part of an account that can move to a different server. `server.sqlite` holds the server parts: sessions, passkeys, challenges, invite codes and the app index. The index maps each app public key to its account, and each code hash to its waiting app. The server makes the index again from the collections at each start (ADR 0110 item 39).

### The Export

3. On a server, the Account panel of Settings has "Export Account". On the desktop app, the menu of each row of Collections has **Export…**. Below the list, the section has only **New Collection…** and **Import an Account File…** ([ADR 0183](0183-each-collection-row-has-one-actions-menu.md) items 2 to 5). The desktop app has no "Move Studio" section. A shell with no list of collections, such as `--data-dir`, keeps "Move Studio" with "Export Studio" (ADR 0176 item 11). The export needs an unlock in the last 5 minutes, as each sensitive account change does (task 069).

4. Before the export, the sheet examines the wrappers of the vault. A vault can have no passphrase wrapper, no recovery wrapper and no Nostr wrapper. Then the sheet tells the user that the vault cannot open on a different server. The export then needs a recovery key first. The sheet has only "Make a Recovery Key" and "Cancel". Studio shows the new key one time, then makes the file.

    Thus each archive has a wrapper that opens on a different server.
5. The sheet tells the user that the archive holds the collection with no encryption: the feeds, the drafts, the versions and the people. The vault and the grant vaults stay encrypted.
6. The archive is one `tar` file. It holds:
   - `manifest.json`: the archive format version, the Studio version, the schema version of the collection and the time of the export. It also holds the email of the account and the public key of each Nostr method. It holds no secret and no wrapper.
   - `vault.json` and `grants/`, as they are.
   - `collection.sqlite`, from the SQLite backup command, so that a write during the export does not damage the copy.
   - `library/`, with each feed XML. With `--hold-assets on` and on the desktop, the sheet has "Include the audio and images", which starts selected. With `--hold-assets off`, the library has only the files that the server keeps (ADR 0105 items 8 and 12).
7. The server sends the archive as a stream, and writes no copy of it to the disk.
8. The export changes nothing. The account, its sessions and its feeds stay. The user deletes the account in a different step ([ADR 0204](0204-feed-files-moves-and-the-server.md) item 31).

### The Import

9. "Import an Account" is on the sign-up page of a server. On a server with invite codes, an import needs an invite code, as a sign-up does (ADR 0204 item 29). 

    On the desktop app, "Import an Account File…" makes a new collection and opens it (ADR 0149 item 11). A closed collection also has "Replace with a File…" (ADR 0176 items 2 and 6 to 9). Studio reads and checks that file in a new folder first. A good read replaces the folder through one write of the list file, then removes the earlier folder. The collection name stays.

    A read that fails changes nothing. The open collection cannot be replaced.

    A shell with no list of collections keeps import on Releases only while its collection has no feed. A user of that shell with data uses a new data folder (`--data-dir`).

10. The browser sends the archive through the upload routes of ADR 0059. The storage limit applies (ADR 0204 item 43).
11. Studio refuses the archive in these conditions, with 422 `archive_invalid` and the cause:
    - The manifest has an unknown format version, or a collection schema that is newer than this Studio.
    - An entry has an absolute path, a `..` segment, a link or a device file.
    - An entry is not one of the files of item 6.
12. The user then opens the vault with a method of the archive: the passphrase, the recovery key or the Nostr signer. The new server makes the account with that method. An email account uses the email of the manifest, and the vault that opens is the proof of the sign-in (ADR 0013 item 16).
13. The import keeps the vault, the collection and the library as they are, subject to item 14. The archive carries the encrypted grant vault files as they are (items 5 and 6). An older collection schema migrates (ADR 0014 item 3). The import writes nothing to a host and changes no feed.

14. After the import, the account has no passkey. The account page offers "Add a Passkey". The import keeps the name of each app and sets each app to "Connect Again". The account has no grant (ADR 0110 item 41). The account page lists the apps with this notice.

### Limits

15. The archive of an export and the archive of an import have the size limit of the storage limit of the server.

## Consequences

- A user can move between two servers, or between the desktop and a server. The user keeps the drafts, the versions, the people, the host secrets. Each app must connect again, and the account has no grant (item 14).
- With `--hold-assets off`, an archive is small, because it has almost no audio.
- An archive holds personal data with no encryption. The user is responsible for where the file goes. [ADR 0108](0108-people-data-at-rest-stays-plain.md) gives the reasons for plain data at rest.
- The core gets the relative library path and its migration. The API gets the export stream and the import checks. The server gets the import at sign-up. The UI gets the two sheets.
- Mockups of the export sheet and the import page come before the task, because they are new screens.

## How This Decision Is Checked

Mechanical:

- A core test: a migration changes an absolute library path in the data folder to a relative path, and keeps a path outside the data folder.
- An API test: an export, then an import into a new data folder, gives the same collection rows and the same vault file. The app rows and the grants follow item 14 and ADR 0110 item 41: each app is "Connect Again", and no grant stays.
- An API test: the import refuses each condition of item 11, and writes no file outside the new user folder.
- A server test: an import with a passphrase vault makes an email account. An import with a Nostr vault makes a Nostr account. An import with only a passkey wrapper is refused, with `archive_invalid` and the cause.
- A server test: the export needs a recent unlock.
- The four commit checks pass.

Visual: the export sheet with each notice, and the import page, in the two themes, at 1100 px and at 390 px with touch.

Human: the operator exports an account from `studio.musicindex.org`, imports it into Studio desktop, opens a draft and writes it.

This consolidation adds no visual or human result. Earlier results stay in their review files.

## Item Map

| Superseded item | New item | Source of the change |
| --- | --- | --- |
| ADR 0107 item 1 | 1 | None. |
| ADR 0107 item 2 | 2 | ADR 0110 item 39: app public keys and waiting-code hashes replace tokens. |
| ADR 0107 item 3 | 3 | ADR 0176 items 10 and 11, as changed by ADR 0183 items 2 to 5: the row menu of Collections, or Move Studio without a collection list. |
| ADR 0107 items 4 to 8 | 4 to 8 | None. The archive carries the encrypted vault files as they are. |
| ADR 0107 item 9 | 9 | ADR 0149 item 11 and ADR 0176 items 2, 3 and 6 to 9. |
| ADR 0107 items 10 to 12 | 10 to 12 | None. |
| ADR 0107 item 13 | 13 | ADR 0110 item 41 removes grants after import. |
| ADR 0107 item 14 | 14 | ADR 0110 item 41: "Connect Again", with no grant. |
| ADR 0107 item 15 | 15 | None. |
| ADR 0107 "Acceptance Criteria" | "How This Decision Is Checked" | No new pass is recorded. |
