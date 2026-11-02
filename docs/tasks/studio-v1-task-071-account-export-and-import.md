# Studio v1 Task 071: Account Export And Import

Decision: [ADR 0107](../adr/0107-a-user-can-export-and-import-an-account.md) and [ADR 0108](../adr/0108-people-data-at-rest-stays-plain.md). It uses ADR 0013, ADR 0014 item 3, ADR 0056, ADR 0058 item 18, ADR 0059 and ADR 0105.

Status: In progress. Packet 071a is complete.

## Goal

A user can export an account as one file, and import it at sign-up on a different server or on an empty desktop. The People page of a server tells the user that the operator can read the people.

## Context

- The mockups are in `.studio-dev/mockup-107/site/move.js`. Git ignores that folder. Screens 1 to 8 are ADR 0107, screen 9 is ADR 0108 and screen 10 is task 069. Use their texts.
- The core has `OpenVault::add_recovery_key` (`crates/core/src/vault.rs`), but no API route calls it. ADR 0107 item 4 needs it.
- `VaultFile::methods` gives the unlock methods of a vault with no secret. It tells if a vault has a wrapper that opens on a different server.
- `Cargo.lock` has `flate2` and no `tar` crate.
- The server makes the index of app tokens again from the collections at each start (`crates/server/src/users.rs`).

## Packets

### 071a: The core

1. A migration of the collection changes the setting `library_root` to a path relative to the data folder, when the library is in the data folder. A path outside the data folder does not change. `Workspace` reads a relative path against its data folder (ADR 0107 item 1).
2. A new module `account_archive` in the core:
   - `Manifest`: the format version 1, the Studio version, the schema version, the time of the export, the email and the public key of each Nostr method. It has no secret and no wrapper.
   - `write_archive(folder, manifest, include_assets, out)` writes the `tar` stream of ADR 0107 item 6. It copies `collection.sqlite` with the SQLite backup command into a temporary file, and removes the temporary file at the end, also after a failure.
   - `check_archive(reader)` reads each entry and gives the manifest, or the cause of a refusal of ADR 0107 item 11.
   - `unpack_archive(reader, folder)` writes each entry into a new, empty folder. It does the checks of item 11 again for each entry, before it writes the entry.
3. Add the `tar` crate (MIT or Apache-2.0) with no default features. Do not use its `unpack` functions. The module writes each file itself, after its own path checks.

### 071b: The export

1. `POST /api/v1/vault/recovery-key` makes a new recovery wrapper and gives the key one time. It needs a recent unlock (task 069).
2. `GET /api/v1/account/export-facts` gives the counts of screen 1, the size of the archive, and `movable`: true when the vault has a passphrase, recovery or Nostr wrapper.
3. `POST /api/v1/account/export` needs a recent unlock. It gives a media ticket of ADR 0062 item 29 for the archive. `GET` of the ticket sends the `tar` stream with `Content-Disposition: attachment`, and writes no copy to the disk. It refuses a vault that is not movable, with 409 `vault_not_movable`.
4. On the server, the manifest gets the email or the Nostr key of the account from `server.sqlite`.

### 071c: The import

1. On the server, `POST /auth/v1/import` starts an import with an invite code when the server needs one (ADR 0055 item 7). It gives an upload of ADR 0059. The upload counts against the storage limit of a new account.
2. `POST /auth/v1/import/{upload}/check` runs `check_archive`, and gives the facts of screen 6 or 422 `archive_invalid` with the cause.
3. `POST /auth/v1/import/{upload}/open` takes the passphrase, the recovery key, or a NIP-07 or NIP-46 unlock. It unpacks the archive into a new user folder, opens the vault, and makes the account with that method. An email account uses the email of the manifest. When the email or the Nostr key is in use on this server, it refuses with 409 `account_exists`. Then it starts a session.
4. On a failure after the unpack, Studio removes the new user folder.
5. On the desktop, `POST /api/v1/import` does steps 1 to 3 into the data folder. It refuses with 409 `collection_not_empty` when the collection has a feed (ADR 0107 item 9).
6. After an import, the account page shows the notice of screen 8 until the user adds a passkey or closes it.

### 071d: The UI

1. The Account panel of a server gets "Move this account" with "Export Account". Settings of the desktop gets the section "Move Studio" with "Export Studio", in the tier `setup`. Add it to `pages.js`.
2. The export sheet shows screen 1, or screen 2 with "Include the audio and images" when the workspace holds assets. It shows screen 3 when `movable` is false: only "Cancel" and "Make a Recovery Key". After the new key shows, the sheet continues to screen 1.
3. The sign-up page gets "Import an Account". The import pages show screens 5 and 6.
4. The empty Releases page of the desktop gets "Import an Account" (screen 7).
5. A model module gives the facts lines and the notices from the API data. A Node test examines each screen state.

### 071e: The People line and the runbook

1. On a server, the People page starts with the text of ADR 0108 item 3. A Node test of the People page model gives the text on a server and no text on the desktop.
2. `docs/runbooks/studio-server.md` gets the text of ADR 0108 item 4.
3. `docs/user-guide/account.md` explains the export, the recovery key and the import. `docs/user-guide/start-on-desktop.md` explains the import into an empty desktop.

## Files

Add: `crates/core/src/account_archive.rs`, a UI model module and its Node test.

Change: `Cargo.toml` and `crates/core/Cargo.toml`, `crates/core/src/lib.rs`, `crates/core/src/collection.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/media.rs`, `crates/api/src/lib.rs`, `crates/server/src/auth.rs`, `crates/server/src/accounts.rs`, `crates/server/src/users.rs`, `crates/api/ui/app.js`, `crates/api/ui/pages.js`, `crates/api/ui/peoplepages.js`, `docs/runbooks/studio-server.md`, `docs/user-guide/account.md`, `docs/user-guide/start-on-desktop.md`.

## Constraints

- An import writes nothing to a host and changes no feed (ADR 0107 item 13).
- No archive entry is written outside the new user folder. The check happens before the write of each entry.
- No secret is in the manifest, in a log or in an API response, except the one recovery key of 071b item 1.
- No feed rule in the UI (ADR 0065).
- When a new test fails, its message names ADR 0107 or ADR 0108 and the fix.

## Acceptance Criteria

Mechanical:

- The tests of the acceptance criteria of ADR 0107 and ADR 0108.
- A core test: `unpack_archive` refuses an entry with `../`, an absolute path, a link and a device file, and the folder then has no file.
- A core test: an archive from a collection with a relative library path opens in a different data folder.
- An API test: `POST /api/v1/vault/recovery-key` needs a recent unlock, and the new key opens the vault.
- A server test: an import with the email of an account that exists gives `account_exists` and leaves no user folder.
- A desktop test: an import into a collection with a feed gives `collection_not_empty`.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the two themes:

- Screens 1 to 9 of the mockups.
- `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the human test of ADR 0107.
