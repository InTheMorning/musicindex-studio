# ADR 0107: A User Can Export And Import An Account

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Amended: 2026-11-01 by the operator, from the mockups in `.studio-dev/mockup-107/site/move.js`. Item 4: an export of a vault that cannot move needs a recovery key first. Item 3: the desktop export is in Settings. Item 9: the desktop import is only on the empty Releases page.
- Changed by: [ADR 0110](0110-a-connected-app-signs-each-request-with-its-own-key.md). Item 2: the index holds the public keys of the apps. Item 14: after an import, each app shows "Connect Again", and the account has no grant.
- Changed by: [ADR 0176](0176-the-user-controls-each-collection.md). Item 3 and item 9 on the desktop app: the export and the import are in "Collections", and an import can replace a closed collection.
- Changes: [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 2 and 3 (the user folder and its library), and item 7 (sign-up).
- Uses: [ADR 0003](0003-user-unlocked-secret-vault.md) item 5 (one vault format in the two shells), [ADR 0013](0013-vault-format-and-cryptography.md), [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) (the sign-in methods and the wrappers), [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) (grant vaults and the index of app tokens), [ADR 0059](0059-uploads-in-parts-and-the-home-screen-app.md) (uploads in parts) and [ADR 0105](0105-a-public-server-holds-no-assets.md) (a server that holds no assets).

## Context

- Studio has no export of an account and no import. A user can move to a different server or to a personal server. That user must then add each feed again and enter each host secret again. The move loses the drafts, the versions, the People data and the connected apps.
- The vault format is the same in the two shells (ADR 0003 item 5). A user folder of the server has the files of a desktop data folder (ADR 0055 item 2). A copy of a user folder is thus almost a complete account.
- Two parts of an account are not in the user folder. `server.sqlite` holds the account, its email, its Nostr sign-in link and its passkeys. The collection can keep the library as an absolute path, such as `/data/users/<folder>/library`.
- A passkey belongs to the relying party ID of its server (ADR 0056 item 20). On a different origin, it gives a different PRF output, and its wrapper does not open the vault. A vault with only passkey wrappers thus cannot move.
- The project is AGPL-3.0 (ADR 0011), and the user controls the secrets (ADR 0003). A user who cannot move to a different server does not fully control the data.
- On 2026-11-01, the operator selected a plain archive with the vault encrypted in it.

## Decision

### The User Folder Is The Account

1. The collection keeps the library folder as a path relative to the data folder when the library is in the data folder. The desktop keeps an absolute path for a library outside the data folder (ADR 0036 item 1).
2. A user folder holds each part of an account that can move to a different server. `server.sqlite` holds only the parts that belong to one server: the sessions, the passkeys, the challenges, the invite codes and the index of app tokens. The server makes the index of app tokens again from the collections at each start (ADR 0058 item 18).

### The Export

3. On a server, the Account panel of Settings has "Export Account". On the desktop, Settings has the section "Move Studio" with "Export Studio". The export needs an unlock in the last 5 minutes, as each sensitive account change does (task 069).
4. Before the export, the sheet examines the wrappers of the vault. A vault can have no passphrase wrapper, no recovery wrapper and no Nostr wrapper. Then the sheet tells the user that the vault cannot open on a different server. The export then needs a recovery key first. The sheet has only "Make a Recovery Key" and "Cancel". Studio shows the new key one time, then makes the file. Thus each archive has a wrapper that opens on a different server.
5. The sheet tells the user that the archive holds the collection with no encryption: the feeds, the drafts, the versions and the people. The vault and the grant vaults stay encrypted.
6. The archive is one `tar` file. It holds:
   - `manifest.json`: the archive format version, the Studio version, the schema version of the collection and the time of the export. It also holds the email of the account and the public key of each Nostr method. It holds no secret and no wrapper.
   - `vault.json` and `grants/`, as they are.
   - `collection.sqlite`, from the SQLite backup command, so that a write during the export does not damage the copy.
   - `library/`, with each feed XML. With `--hold-assets on` and on the desktop, the sheet has "Include the audio and images", which starts selected. With `--hold-assets off`, the library has only the files that the server keeps (ADR 0105 items 8 and 12).
7. The server sends the archive as a stream, and writes no copy of it to the disk.
8. The export changes nothing. The account, its sessions and its feeds stay. The user deletes the account in a different step (ADR 0055 item 9).

### The Import

9. "Import an Account" is on the sign-up page of a server. On the desktop, it is on the Releases page only while the collection has no feed, so that an import cannot replace data. A user with data uses a new data folder (`--data-dir`). On a server with invite codes, the import needs an invite code, as a sign-up does (ADR 0055 item 7).
10. The browser sends the archive through the upload routes of ADR 0059. The storage limit applies (ADR 0055 item 21).
11. Studio refuses the archive in these conditions, with 422 `archive_invalid` and the cause:
    - The manifest has an unknown format version, or a collection schema that is newer than this Studio.
    - An entry has an absolute path, a `..` segment, a link or a device file.
    - An entry is not one of the files of item 6.
12. The user then opens the vault with a method of the archive: the passphrase, the recovery key or the Nostr signer. The new server makes the account with that method. An email account uses the email of the manifest, and the vault that opens is the proof of the sign-in (ADR 0013 item 16).
13. The import keeps the vault, the grant vaults, the collection and the library as they are. An older collection schema migrates (ADR 0014 item 3). The import writes nothing to a host and changes no feed.
14. After the import, the account has no passkey. The account page offers "Add a Passkey". Each connected app must use the address of the new server. The account page lists the apps with this notice.

### Limits

15. The archive of an export and the archive of an import have the size limit of the storage limit of the server.

## Alternatives Considered

- An archive that a passphrase encrypts. Not chosen by the operator. It protects the collection too, but the user must keep one more secret. The vault is already encrypted.
- An export of the feeds only, as OPML or as feed XML files. Rejected: it loses the drafts, the versions, the people and the host secrets. A user can already add each feed from its address.
- A move from server to server with no file, through an API between the two servers. Rejected: the two operators must trust each other, and a desktop cannot take part.
- A passkey wrapper that moves. Not possible: the PRF output belongs to the relying party ID (ADR 0056 item 20).

## Consequences

- A user can move between two servers, or between the desktop and a server. The user keeps the drafts, the versions, the people, the host secrets and the grants.
- With `--hold-assets off`, an archive is small, because it has almost no audio.
- An archive holds personal data with no encryption. The user is responsible for where the file goes. [ADR 0108](0108-people-data-at-rest-stays-plain.md) gives the reasons for plain data at rest.
- The core gets the relative library path and its migration. The API gets the export stream and the import checks. The server gets the import at sign-up. The UI gets the two sheets.
- Mockups of the export sheet and the import page come before the task, because they are new screens.

## Acceptance Criteria

Mechanical:

- A core test: a migration changes an absolute library path in the data folder to a relative path, and keeps a path outside the data folder.
- An API test: an export, then an import into a new data folder, gives the same collection rows, the same vault file and the same grant vaults.
- An API test: the import refuses each condition of item 11, and writes no file outside the new user folder.
- A server test: an import with a passphrase vault makes an email account. An import with a Nostr vault makes a Nostr account. An import with only a passkey wrapper is refused, with `archive_invalid` and the cause.
- A server test: the export needs a recent unlock.
- The four commit checks pass.

Visual: the export sheet with each notice, and the import page, in the two themes, at 1100 px and at 390 px with touch.

Human: the operator exports an account from `studio.musicindex.org`, imports it into Studio desktop, opens a draft and writes it.
