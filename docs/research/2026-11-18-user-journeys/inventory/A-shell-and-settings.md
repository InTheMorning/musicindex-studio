<!-- Recorded output of ChatGPT inventory run A at commit 73828c0. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

# Current-code journey inventory — run A

The checkout is `73828c03a543f401d47fa29dd3a6ed4b9b68b19c`. No files were changed. No repository-changing Git commands were run.

This inventory comes from source inspection. No browser session, account operation, file-host request, or desktop application was started.

## Reading this inventory

File names below have these locations:

- UI JavaScript: `crates/api/ui/`.
- `desktop/main.rs`, `desktop/quit.rs`, and other `desktop/` references: `crates/desktop-app/src/`.
- Server references: `crates/server/src/`.
- API references: `crates/api/src/`.

For example, `app.js:1760` means `crates/api/ui/app.js:1760`.

**INFERENCE — counting convention:** Each count is calculated from the handlers and rendered controls. Counts exclude automatic work and waiting.

A typed field counts as one step, regardless of character count. A selection from a list counts as one step. Optional actions add steps.

**UNSURE — native controls:** The operating system, browser, passkey manager, and signer supply some controls. Their exact labels and internal step counts are absent from this repository.

The named jobs for live shows, grants, announcements, and release creation cross into `editor.js`. Those paths are included below. Other feed editing and People operations remain outside run A.

## 1. Start Studio and complete the first run

### Entry points and automatic start

| Entry or condition | Current behavior | Evidence |
|---|---|---|
| Launch the desktop application | Starts the local API, then opens one window and builds the tray | `desktop/main.rs:540`, `desktop/main.rs:617` |
| Launch another desktop instance | Gives the existing window focus | `desktop/main.rs:635` |
| Tray item `"Open Studio"` | Shows, restores, and focuses the window | `desktop/main.rs:254`, `desktop/main.rs:156` |
| Left click on the tray icon | Performs the same window action | `desktop/main.rs:520` |
| Open `/` in a browser | Loads `index.html`, then `app.js` | `api/lib.rs:118`, `api/ui.rs:384`, `index.html:14` |
| Local browser URL with `#token=${...}` | Stores the token in session storage and removes the fragment | `app.js:65` |

The desktop window title is `"Music Index Studio"`. Its initial size is 1280 × 860. See `desktop/main.rs:41`, `desktop/main.rs:418`.

The initial HTML shows `"Loading Studio…"`. See `index.html:17`.

`App` first requests `/auth/v1/session`. `Studio` then requests `/api/v1/status` and `/api/v1/library`. These are automatic actions. See `app.js:5459`, `app.js:5364`.

### Main path: select the library folder

Precondition: local mode, valid token, vault not locked, and `library.root === null`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1, optional | Type `"Folder"` | The library form contains the new path | `app.js:1790` |
| 1 with the default path, otherwise 2 | Click `"Use This Folder"` | Automatic library setup, then `"Studio set up your library"` | `app.js:1792`, `app.js:1767`, `app.js:1773` |
| 2 with the default path, otherwise 3 | Click `"Continue"` | Status and library reload, then the shell | `app.js:1783`, `app.js:5389` |

The result page lists `"Release folders"`, `"Feed files"`, and `"Files moved"`. See `app.js:1774`.

**INFERENCE — count:** 2 steps with the supplied folder. Typing another folder adds 1.

There is no folder-picker button in this first-run form. It uses a text field. See `app.js:1786`.

### Branches and failures

| Code condition | Result | Evidence |
|---|---|---|
| `status.vault === 'locked' && !serverMode` | Full-page `"Unlock Studio"` comes before library setup | `app.js:5386` |
| `library.root !== null` | Skips library setup | `app.js:5389` |
| `showWelcome(feeds)`, meaning no feeds | Shows Welcome within Releases | `welcome.js:69`, `app.js:3434` |
| No local token | Shows `"Open Studio from its address"` | `app.js:5377`, `app.js:1823` |
| Local status request gives 401 | Shows the same no-token page | `app.js:5378` |
| Startup API failure | Error notice and `"Try Again"` | `app.js:5381`, `app.js:5486` |
| Library setup reports file errors | `"Studio could not move each file into the library."` and `Feed ${e.feed_id}: ${e.message}` | `app.js:1779` |
| Desktop cannot find an application data folder | Native error: `"Studio could not start. The system has no application data folder."` | `desktop/main.rs:542` |
| Data-folder preparation or startup fails | `"Studio is already running."` or `Studio could not start. ${e}` | `desktop/main.rs:544`, `desktop/main.rs:592` |

The no-token page says:

> “For your safety, Studio accepts only the address that it showed when it started. Copy that full address, with the part after “#”, into the browser.”

Evidence: `app.js:1824`.

There is no confirmation sheet for library setup. The result page stays until `"Continue"`.

### First-run question

Welcome shows:

> “Welcome to Studio”

> “Studio publishes your music so that every music app can play it. What do you want to start with?”

Evidence: `welcome.js:9`.

Its entry points are:

| Exact label | Opens | Evidence |
|---|---|---|
| `"I make music"` | `"Your artist page"` sheet | `welcome.js:12`, `app.js:3512` |
| `"I run a label"` | `"Your label page"` sheet | `welcome.js:13`, `app.js:3513` |
| `"My music is online already"` | Feed discovery sheet | `welcome.js:14`, `app.js:3516` |
| `"Import the Backups"` | MSP import sheet | `welcome.js:17`, `app.js:3519` |
| `"Import an Account"` | Account import sheet, local mode only | `welcome.js:19`, `app.js:3520` |

### Main path: make the first artist or label page

Start: Welcome.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click `"I make music"` or `"I run a label"` | Corresponding name sheet | `app.js:3512` |
| 2 | Type `"Artist name"` or `"Label name"` | Name form becomes ready | `welcome.js:37`, `welcome.js:45`, `app.js:3606` |
| 3 | Click `"Continue"` | `"Where it lives"` step | `app.js:3609`, `app.js:3580` |
| 4, existing host | Click `"Make the Page"` | Automatic draft creation, then the feed editor | `app.js:3622`, `app.js:3571`, `app.js:3961` |
| 4, server default | Click `"Continue"` under `"Try it here (temporary)"` | Creates a temporary connector, then the page draft and editor | `welcome.js:91`, `app.js:3590`, `app.js:3574` |
| 4, local default | Click `"Continue"` in the local-folder form | Adds the supplied folder, then creates the page draft and opens its editor | `app.js:3591`, `app.js:759` |

The local default folder comes from `library.default_host_root`. See `app.js:3566`.

**INFERENCE — counts:** 4 steps for an existing selected host, the server temporary area, or a usable supplied local folder. Typing `"Folder"` adds 1.

The host choices are:

- `"Folder on this computer"` in local mode.
- `"Try it here (temporary)"` on the server.
- `"Nostr site (nsite)"`.
- `"S3 or Cloudflare R2"`.
- `"SFTP server"`.

Evidence: `welcome.js:88`.

Selecting another host row adds 1 step. Its connection form adds the steps in section 6.

The temporary-area note is:

> “Kept for 7 days. Pick a real file host to keep your pages.”

Evidence: `welcome.js:25`.

The sheet offers `"Cancel"` on the name step, `"Back"` on the host step, and header `"Close"`. Escape closes the dialog. See `app.js:3597`, `app.js:3608`, `app.js:3627`, `editor.js:1918`.

If the connector succeeds but page creation fails, the sheet keeps `madeHost`. It shows:

> `Studio connected the file host ${madeHost.name}.`

It offers `"Make the Page"` for another attempt. See `app.js:3611`.

**End state:** The new feed editor opens directly, including at narrow width. No file is written to the host during this path. See `app.js:3961`, `app.js:1464`.

The feed header then shows:

> `The file host has no file for this ${feed.medium === 'publisher' ? 'feed' : 'release'} yet.`

> “Open Review and write the draft to make the file.”

Evidence: `app.js:2293`.

## 2. Find and add feeds

### All shell entry points

| Entry point | Exact label | Evidence |
|---|---|---|
| Welcome discovery row | `"My music is online already"` | `welcome.js:14`, `app.js:3516` |
| Releases toolbar | `"＋ Add a Feed"` | `app.js:3444` |
| Releases empty state, when feeds exist but releases do not | `"Add a Feed"` | `app.js:3473` |
| Add feeds page | `"By address or name"` | `addfeeds.js:16`, `app.js:2966` |
| Add feeds page, with connectors | `"From a file host folder"` | `addfeeds.js:17` |
| Add feeds page | `"From MSP 2.0 backups"` | `addfeeds.js:18` |
| Welcome MSP link | `"Import the Backups"` | `app.js:3519` |
| After adding a file host | Automatically displayed `"Studio also found these feeds in this folder"` panel | `app.js:2810`, `app.js:2352` |

The empty collection uses Welcome instead of the ordinary Releases empty state. See `app.js:3434`.

There is no feed-list search or filter control in `ReleasesPage`. That renderer provides sorting, view controls, and add actions. See `app.js:3408`.

### Main path: add feeds by address

Start: ordinary Releases.

| Step | Action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click `"＋ Add a Feed"` | Add feeds page | `app.js:3444`, `app.js:3990` |
| 2 | Click `"By address or name"` | Sheet with address and search fields | `addfeeds.js:16`, `app.js:2932` |
| 3 | Type `"Feed addresses"` | Count includes each nonempty line | `app.js:1614`, `app.js:1596` |
| 4 | Click `Find My Feeds (${count})` | Automatic discovery, then grouped feed results | `app.js:1640`, `app.js:1611` |
| 5 | Click `Add ${chosen.length} Feed${chosen.length === 1 ? '' : 's'}` | Automatic import, list reload, then the first added feed opens | `app.js:1685`, `app.js:2934`, `app.js:3961` |

All addable results start selected. Feeds already in the collection have disabled checkboxes. See `app.js:1651`, `app.js:1671`.

**INFERENCE — count:** 5 steps. From the Welcome discovery row, 4 steps. Each checkbox change adds 1.

### Branch: search by name

Replace the address-entry step with:

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Type `"Or search Music Index by name"` | Query text is present | `app.js:1621` |
| 2 | Click `"Search"` | Automatic search, then result checkboxes | `app.js:1624` |
| 3 | Select `${h.title || h.feed_guid}` | Adds its GUID to the discovery selection | `app.js:1631` |

Then click `"Find My Feeds"` and the add button.

**INFERENCE — count:** 7 steps from ordinary Releases for one search-result selection. From Welcome, 6.

The search can also show `${note.message}`. An empty result shows:

> “Music Index found no feed with that name.”

Evidence: `app.js:1628`.

The search field uses `useFieldList`. The shared field list can insert query tokens. See `app.js:1591`, `app.js:1625`, `pickers.js:46`.

**UNSURE:** The complete set of field-list rows depends on the `/musicindex/fields` response. It is not a fixed list in these renderers.

### Discovery branches and results

| Condition | Visible result or action | Evidence |
|---|---|---|
| `found.feeds.length === 0` | `"Studio found no feeds."` | `app.js:1664` |
| Results exist | `Studio found ${found.feeds.length} feed${found.feeds.length > 1 ? 's' : ''}.` | `app.js:1664` |
| `group.service` | `${group.service}: Studio can read these feeds only.` | `app.js:1645` |
| Other host group | `"Studio can write these feeds when you connect the folder that holds them."` | `app.js:1646` |
| `found.truncated` | `"Studio stopped at 100 feeds. Add the others in a second search."` | `app.js:1678` |
| `found.failed.length > 0` | `"Studio could not read:"`, then failure rows | `app.js:1679` |
| All results already exist | `"Each of these feeds is in your collection."` | `app.js:1682` |
| Discovery results are displayed | `"Search Again"` clears results | `app.js:2935` |

`FoundFeeds` can render `"Open Studio"` when `onNothingToAdd` exists. The inspected shell callers do not supply that callback. See `app.js:1684`, `app.js:2934`.

There is no confirmation before importing selected feeds.

**End state:** The first added feed opens. Unsupported feed types open their facts view. No discovery-result notice is carried into that page.

### Main path: add a feed from a connected folder

Start: ordinary Releases, at least one connector.

1. Click `"＋ Add a Feed"` → Add feeds. `app.js:3444`.
2. Click `"From a file host folder"` → folder-feed sheet. `addfeeds.js:17`, `app.js:2937`.
3. Pick a row in `"Feeds in this folder"` → selected path. `app.js:1556`.
4. Click `"Add the Feed"` → automatic add and refresh, then feed page. `app.js:1579`, `app.js:1532`.

**INFERENCE — count:** 4 steps. Choosing another `"File host"` adds 1. See `app.js:1550`.

Manual-path branch:

1. Click `"The Feed Is Not in the List"`. `app.js:1576`.
2. Type `"Feed file"`. `app.js:1573`.
3. Click `"Add the Feed"`.

**INFERENCE — count:** 5 steps from Releases. If all listed feeds already exist, typing mode opens automatically, reducing this to 4.

With available rows, `"Select from the List"` restores the list. See `app.js:1577`.

The list can show:

- `"Looking for feed files…"`.
- `"Studio found no feed files in this folder."`.
- `"The folder has more files than Studio lists. Type the path of a feed that is not in the list."`.

Evidence: `app.js:1559`.

A row label is:

> `${f.title || f.path} · ${feedKind(f).label}${f.blocked ? ' · blocked' : ''} · ${f.path}${f.in_collection ? ' · in your collection' : ''}`

Evidence: `app.js:1565`.

A refresh failure triggers automatic removal of the newly added collection entry. The error then remains in the form. See `app.js:1534`.

### Main path: add the feeds offered after host connection

Start: the `"Studio also found these feeds in this folder"` panel.

1. Optionally change feed checkboxes `${f.title || f.path}`. `app.js:2357`.
2. Click `Add ${chosen.length} Feed${chosen.length === 1 ? '' : 's'}`. `app.js:2365`.

The panel automatically adds and refreshes each selected feed. It then shows:

> `Studio added ${result} feed${result === 1 ? '' : 's'} from “${connector.name}”.`

Evidence: `app.js:2341`.

**INFERENCE — count:** 1 step with default selections. Each changed checkbox adds 1.

The result stays on the File hosts page. A folder-list failure is converted to an empty list and shows no panel. See `app.js:2334`.

### Main path: import MSP backups

Start: ordinary Releases.

1. Click `"＋ Add a Feed"`. `app.js:3444`.
2. Click `"From MSP 2.0 backups"`. `addfeeds.js:18`.
3. Click `"Choose Backup Files"` → native file picker. `app.js:1745`.
4. Pick backup files → automatic JSON reads. `app.js:1743`.
5. Click `Import ${backups.length || ''} Backup${backups.length === 1 ? '' : 's'}`. `app.js:1750`.
6. Click `"Continue"` on the result. `app.js:1736`.

**INFERENCE — count:** 6 steps, treating the native file selection as one pick. From Welcome, 5. Dropping files replaces steps 3–4 with one drop.

If the import requires a vault, it opens the vault-creation flow in section 3. See `app.js:1720`.

Failure text:

- `${file.name} is not a JSON backup file.` — beside the file control. `app.js:1709`.
- `"Studio could not import:"` — result failure list. `app.js:1734`.
- Shared feed-read failures — section 14.

The result says:

> `Studio added ${result.added.length} feed${result.added.length === 1 ? '' : 's'} and keeps the edit tokens in your vault.`

Evidence: `app.js:1733`.

**End state:** Continue opens the first added feed, or Releases when no feed is supplied. The result remains until Continue.

## 3. Create, unlock, and lock the vault

### Create the vault when a secret is first needed

Entry condition: a connector or MSP operation returns `vault_required`.

Call sites:

- S3: `app.js:832`.
- SFTP: `app.js:955`.
- nsite and Blossom: `app.js:1191`.
- MSP import: `app.js:1720`.

The current form is replaced by `"Protect your keys"`. See `app.js:709`.

Main path:

1. Type `"Passphrase"`. `app.js:730`.
2. Type `"Type the passphrase again"`. `app.js:734`.
3. Click `"Make the Vault"`. `app.js:740`.
4. Click `"Show recovery key"` to reveal the key. `app.js:750`, `app.js:558`.
5. Select `"I wrote down the recovery key in a safe place"`. `app.js:751`.
6. Click `"Continue"`. `app.js:753`.

The pending connector or import operation resumes automatically.

**INFERENCE — count:** 6 steps with the default recovery key and its reveal. Clearing `"Make a recovery key, in case I forget the passphrase"` gives 4 steps.

The form waits for a strength result. Its ready condition is:

```js
strength && strength.acceptable && passphrase === confirm
```

Evidence: `app.js:718`.

Possible field messages:

- `Too short: ${chars} of ${MIN_CHARS} characters.`
- `Too easy to guess.${strength.warning ? \` ${strength.warning}\` : ''}`
- `"The passphrases are not the same."`
- `"The passphrase is too short."`
- `"The passphrase is too easy to guess."`

Evidence: `app.js:699`, `app.js:717`, `app.js:448`.

The strength line is:

> `Strength: ${STRENGTH_WORDS[strength.score]} (${strength.score} of 4)`

The words are `"Very weak"`, `"Weak"`, `"Fair"`, `"Strong"`, and `"Very strong"`. See `app.js:666`, `app.js:695`.

### Unlock at local startup

Entry condition: `status.vault === 'locked' && !serverMode`. See `app.js:5386`.

1. Type `"Passphrase"`. `app.js:1813`.
2. Click `"Unlock"`. `app.js:1816`.

Automatic work opens the vault and reloads Studio.

**INFERENCE — count:** 2 steps. There is no Cancel button on this full-page screen.

Wrong passphrase appears below the field as:

> “The passphrase is not correct.”

Evidence: `app.js:1809`, `app.js:449`.

### Unlock within a session

Entry points:

| Entry | Condition | Evidence |
|---|---|---|
| `"Unlock Studio"` in the sidebar or narrow root footer | Server vault is locked | `shell.js:287`, `app.js:3336` |
| Any API operation that needs the vault | First response is 423 `vault_locked` | `app.js:143` |
| An account-security operation | First response is 401 `unlock_required` | `app.js:144` |

The latter case adds:

> “Unlock Studio again to change how you sign in.”

Evidence: `account.js:69`.

The sheet title is `"Unlock Studio"`. It contains:

> `This step needs the vault. ${serverMode ? 'Your passphrase, your signer or your passkey opens it.' : 'Your passphrase opens it.'}`

Evidence: `app.js:4531`, `app.js:4535`.

| Method | Steps after opening the sheet | Availability |
|---|---|---|
| Passphrase | Type `"Passphrase"` → click `"Unlock"` | Local mode, or server session has `email` |
| Extension | Click `"Unlock with a Signer Extension"` → external signer interaction | Session has `pubkey` and `window.nostr` exists |
| Remote signer | Click `"Use the Remote Signer"` → external signer interaction | Session has `pubkey` and `signer` |
| Passkey | Click `"Use a Passkey"` → native passkey interaction | Session has passkeys and browser support |

Evidence: `account.js:134`, `app.js:4540`.

**INFERENCE — counts from an explicit footer entry:** Passphrase 3. Other methods 2 repository-controlled steps, plus external interactions.

For automatic entry, subtract the footer click and count the original triggering operation separately.

The sheet offers header `"Close"` and footer `"Cancel"`. Escape also closes it. Closing returns `false` to the pending operation. See `app.js:4529`, `app.js:4554`.

After successful unlock, the original request retries once. See `app.js:138`.

No available method gives:

> “This account opens with a Nostr signer. Studio found no signer extension in this browser.”

Evidence: `app.js:4551`.

Extension decryption can fail with:

> “The signer did not decrypt the vault key.”

Evidence: `app.js:4512`.

Other method failures use the authentication catalog in section 14.

### Lock Studio

Entry: `"Lock Studio"` in the sidebar or narrow root footer. It appears when `vaultOpen` is true. See `shell.js:286`, `app.js:3334`.

One click sends `/vault/lock`, then reloads Studio. The handler suppresses a lock-request error. See `app.js:3930`.

**INFERENCE — count:** 1 step.

- Local mode returns to the full-page unlock screen.
- Server mode can continue showing the shell with the vault locked.

Evidence: `app.js:5384`.

There is no confirmation or persistent success message.

## 4. Sign in, make an account, recover access, and sign out

The server uses the same browser UI. Its authentication routes return data to these components. See `server/auth.rs:36`, `app.js:4259`.

### Sign in with email and passphrase

Entry: open `/` without a server session, or return after sign-out/session loss. See `account.js:10`, `app.js:5495`.

1. Type `"Email address"`. `app.js:4288`.
2. Type `"Passphrase"`. `app.js:4290`.
3. Click `"Sign In"`. `app.js:4294`.

The button becomes `"Signing In…"`. The request uses `/auth/v1/signin/email`. See `app.js:4267`.

**INFERENCE — count:** 3 steps.

**End state:** Session reload, then Studio. Library setup can follow if needed. No sign-in success notice stays.

### Sign in with a signer extension

Entry: `"Sign In with a Signer Extension"`, shown only with `window.nostr`. See `app.js:4295`.

1. Click that button.
2. Complete the external signer interaction — **UNSURE count and labels**.

The UI requests a challenge, signs an event, and submits it. See `app.js:4049`.

**INFERENCE — count:** 1 repository-controlled step.

For an existing Nostr account, the server reports the current vault state. This sign-in path does not itself decrypt an existing vault. See `server/auth.rs:692`, `server/auth.rs:727`.

Errors include:

- `"This signer does not support NIP-44. Studio needs NIP-44 to open the vault."`
- `"The signer did not sign the sign-in request."`
- `"The signer did not encrypt the vault key."`

Evidence: `app.js:4052`, `app.js:4059`, `app.js:4070`.

### Sign in with a passkey

Entry: `"Sign In with a Passkey"`, shown when `PublicKeyCredential` is a function. See `app.js:4206`, `app.js:4296`.

1. Click that button.
2. Complete the native passkey interaction — **UNSURE count and labels**.

Automatic work obtains an assertion and PRF output, then signs in. See `app.js:4251`.

**INFERENCE — count:** 1 repository-controlled step.

Failure messages are in section 14.

### Sign in with Amber or a bunker

Entry: `"Sign In with Amber or a Bunker"`. See `app.js:4297`.

After the click, the full-page title is `"Sign in with Amber or a bunker"`. See `app.js:4282`.

Automatic work creates a connection and QR code.

| Branch | User actions after entry | Evidence |
|---|---|---|
| QR | Scan QR and allow signer requests | `app.js:4186` |
| Phone link | Click `"Open in the Signer App"`, then allow requests | `app.js:4188` |
| Bunker | Type `"Bunker address"`, then click `"Connect"` | `app.js:4190` |

The UI waits automatically and then signs in. See `app.js:4083`, `app.js:4121`.

**INFERENCE — repository-controlled counts:** QR 1 including entry. Phone link 2. Bunker 3. External signer steps remain unknown.

Status messages:

- `"Waiting for the signer…"`.
- `"Connecting to the signer…"`.
- `"Signing in. Allow the request in your signer app."`.

Evidence: `app.js:4194`.

While starting or waiting, `"Stop"` returns to Sign in. After failure, `"Back"` returns and `"Try Again"` starts another connection. See `app.js:4198`.

### Make an account

Entry: `"Make an Account"` on Sign in. See `app.js:4299`.

Email path:

1. Click `"Make an Account"` → `"Make an account"`.
2. Type `"Email address"`. `app.js:4443`.
3. Type `"Passphrase"`. `app.js:4447`.
4. Click `"Make the Account"`. `app.js:4455`.
5. Reveal `"Recovery key"` with its eye button. `app.js:4473`, `app.js:750`.
6. Select `"I wrote down the recovery key in a safe place"`. `app.js:751`.
7. Click `"Continue"`. `app.js:753`.

**INFERENCE — count:** 7 with the default recovery key and its reveal.

Branches:

- `signup === 'invite'` adds `"Invite code"` and one typed-field step. `app.js:4415`, `app.js:4445`.
- Clearing the default recovery checkbox removes recovery-key display, but adds a checkbox click. Net count: 5.
- `"Sign Up with a Signer Extension"` replaces email/passphrase entry and submission. `app.js:4456`.
- `"Sign Up with Amber or a Bunker"` opens the remote-signer page. `app.js:4458`.
- `"I Have an Account"` returns to Sign in. `app.js:4462`.
- `"Import an Account"` opens account import. `app.js:4461`.

**INFERENCE — extension signup count:** 5 repository-controlled steps with recovery-key reveal. Invite entry adds 1. External signer actions are additional.

**UNSURE:** `signup` can contain server policy values. The `SignUpPage` renderer explicitly branches only on `'invite'`. Server refusal uses `AuthNotice`.

The email field can show `"Give an email address."`. See `app.js:4418`.

### Sign in with a recovery key

Entry: `"Sign In with the Recovery Key"`. See `app.js:4300`.

1. Click that entry → recovery page.
2. Type `"Email address or npub"`. `app.js:4325`.
3. Type `"Recovery key"`. `app.js:4327`.
4. Click `"Sign In"`. `app.js:4331`.

Automatic work starts a recovery-limited session. The next page is `"Add a sign-in method"`. See `account.js:17`, `app.js:4386`.

A failed recovery attempt can show:

> “The account or the recovery key is not correct.”

Evidence: `account.js:164`, `app.js:4329`.

`"Back to Sign In"` returns before submission. See `app.js:4333`.

To continue with a passphrase:

5. Type `"New passphrase"`. `app.js:4389`.
6. Click `"Save the Passphrase"`. `app.js:4394`.

Other available actions:

- `"Use the Signer Extension"` when the account has a Nostr key and an extension exists.
- `"Use a Passkey"` when browser support exists.
- `"Sign Out"`.

Evidence: `account.js:183`, `app.js:4397`.

**INFERENCE — counts:** Recovery plus passphrase replacement: 6. Recovery plus extension/passkey: 5 repository-controlled steps, plus external interactions.

The page says:

> “The recovery key opened the vault. Add a new way to sign in before you continue.”

Evidence: `app.js:4387`.

With no choices:

> “This account signs in with a Nostr signer. Studio found no signer extension in this browser.”

Evidence: `app.js:4401`.

### Sign out

Entry points:

- Settings → `"Account"` → `"Sign Out"`. `pages.js:105`, `app.js:4659`.
- Recovery-limited `"Add a sign-in method"` page → `"Sign Out"`. `app.js:4403`.

The request is `/auth/v1/signout`, followed by session reload. See `app.js:4622`.

**INFERENCE — counts:** 3 from a shell destination. 1 from the recovery-limited page.

There is no confirmation. A failure remains as an authentication notice. Success returns to Sign in without a result notice.

### Account and passphrase changes present in this code

The Account panel contains identity display, export, passkeys, account deletion, and sign-out. See `app.js:4634`.

No email-change, Nostr-account-key-change, or ordinary passphrase-change control is rendered there.

The inspected UI submits `/vault/passphrase` only from `AddMethodPage`, after recovery. See `app.js:4352`.

This is distinct from the SFTP field `"Passphrase of the key (optional)"`, which belongs to connector creation. See `app.js:1030`.

## 5. Navigate Studio and arrange the feed list

### Destinations

The sidebar and narrow root footer render:

- `"Releases"`.
- Publisher-feed names under role headings.
- `"Not linked"` when unlinked releases exist.
- `"Other feeds"` when unsupported feed types exist.
- `"People"`.
- `"File hosts"`.
- `"Settings"`.
- `Collection: ${open.name}` when more than one collection exists.
- `"Lock Studio"` or server `"Unlock Studio"`.

Evidence: `shell.js:271`, `shell.js:454`, `collectionlist.js:6`, `app.js:3331`.

The collection-mark button opens Settings without selecting Data. See `app.js:3338`.

### Main navigation paths and counts

| Job | User steps | Result | Evidence |
|---|---|---|---|
| Open a fixed destination | Click its exact destination label | Corresponding page | `app.js:3339` |
| Open a publisher level | Click `${item.label}` | Its children and releases | `app.js:3180`, `app.js:3212` |
| Open a child level | Click `${label}` | Child level | `app.js:3239` |
| Open the publisher page | Click `${head.title}` or `"Open the Feed"` | Feed editor | `app.js:3231`, `app.js:3446` |
| Open a release, wide layout | Click `${row.title}` | Feed editor | `app.js:3457`, `shell.js:839` |
| Open a release, narrow layout | Click `${row.title}`, then `${feed.title}` | Outline, then editor | `shell.js:839`, `app.js:3287` |
| Move to the parent level | Click chevron row `${panel.back.label}` | Parent level or Releases | `app.js:3230`, `shell.js:403` |
| Go Back | Click `"Back"` | Previous navigation entry | `app.js:1972`, `app.js:3741` |
| Go Forward, mouse profile | Click `"Forward"` | Next navigation entry | `app.js:1976`, `app.js:3777` |
| Go to narrow root | Click `"Destinations"` or title button `${title}, Destinations` | Root destination page | `app.js:1969`, `app.js:1979` |

**INFERENCE — counts:** Each table action is 1 step, except narrow release-to-editor navigation, which is 2.

A chain through `n` publisher levels adds `n` selections.

### Width, shell, and feed-type branches

- Narrow means `matchMedia('(max-width: 760px)')`. `app.js:1844`.
- At narrow width, the sidebar is absent. Its content becomes root, level, and outline pages. `app.js:4017`, `shell.js:809`.
- The root page has no Back control. `app.js:3348`.
- With `pointer: coarse`, the bar renders Back without Forward. `app.js:1971`.
- Width changes preserve the current logical view, with root/outline conversions. `shell.js:844`, `app.js:3845`.
- Album, playlist, and publisher feeds can open the editor. A missing connector does not change that type check. `shell.js:100`.
- A missing `connector_id` marks a feed read-only. `shell.js:118`.
- Other feed types use `ReadOnlyFeed`. `app.js:2311`.

The unsupported-type message is:

> `This feed has ${medium}. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed.`

Here, `medium` is `the type “${feed.medium}”` or `"no type"`. See `shell.js:111`.

### Sort and list view

Entry: Releases controls when `hasRows` is true. See `app.js:3440`.

| Action | Exact labels | Steps | Evidence |
|---|---|---|---|
| Change sort | Click `${sortLabel(sort, reversed)} ⌄`, then pick an entry | 2 | `app.js:3387`, `menu.js:27` |
| Standard sort entries | `"Newest"`, `"Oldest"`, `"A to Z"`, `"Z to A"` | One selection after menu opens | `shell.js:593`, `shell.js:619` |
| Publisher group additions | `"Feed Order"`, `"Feed Order, Reversed"` | Same | `shell.js:589` |
| Change view | `"Rows"` or `"Tiles"` icon button | 1 | `app.js:3442` |
| Jump by initial | `${g.letter}` | 1 | `app.js:3398` |

**INFERENCE — counts** are shown in the table.

Global sort/view/direction use the URL query:

- `sort=az`.
- `view=tiles`.
- `reversed=1`.

Defaults are newest, rows, and unreversed. See `shell.js:715`, `shell.js:726`.

Each publisher group keeps its own sort and direction in its navigation entry. See `app.js:3425`, `app.js:4008`.

There is no confirmation or result notice for these controls.

### History and return state

Navigation persists in `sessionStorage` under `"studio-nav"`. A new session starts at Releases. See `app.js:3647`.

Back restores scroll and marks the release that the user left. Browser Back and Forward are connected to this state. See `shell.js:752`, `app.js:3789`.

The sidebar `"Releases"` row from an outline creates a new Releases view. It is not the same handler as Back. See `app.js:3783`.

### Keyboard behavior

| Context | Keys | Behavior | Evidence |
|---|---|---|---|
| Segmented tabs | Left, Right, Home, End | Move focus | `shell.js:606`, `app.js:2052` |
| Segmented tabs | Enter or Space | Select through button activation | `shell.js:601` |
| Row lists | Up, Down, Home, End | Move focus | `listkeys.js:73`, `listkeys.js:102` |
| Tile grid | Arrow keys | Move between tiles and rows | `listkeys.js:65` |
| Row or tile | Enter | Activates the selected page | `listkeys.js:92` |
| Hierarchy | Left, Right, Up, Down, Home, End, Enter | Tree movement or activation | `listkeys.js:73`, `listkeys.js:128` |
| Open `MenuControl` | Escape | Closes menu and returns focus to its button | `menu.js:17` |
| Dialog | Escape | Closes through the dialog close event | `editor.js:1918` |
| Linux desktop editable input | Ctrl+Z | Undo | `desktop/linux_page.js:55` |
| Linux desktop editable input | Ctrl+Shift+Z or Ctrl+Y | Redo | `desktop/linux_page.js:59` |

**INFERENCE — keyboard counts:** One key is one step. Reaching a row requires the applicable number of focus-movement keys, then activation.

No global app navigation shortcut is registered in the inspected `app.js` or desktop menu setup.

## 6. Add, edit, check, move, and remove file hosts

### Entry points

- Sidebar/root `"File hosts"`. `shell.js:281`, `app.js:3339`.
- Add feeds notice → `"Connect a File Host"`. `app.js:2964`.
- Outside-move notice → `"Connect a File Host…"`. `outsidecheck.js:71`, `app.js:4916`.
- Review → `"Connect the File Host of This File"`. `editor.js:2188`.

The File hosts page automatically displays its add form when `connectors.length === 0`.

With existing connectors, click `"＋ Add a File Host"`. See `app.js:2781`, `app.js:2790`.

`"Type of file host"` offers:

- `"Folder on this computer"` — local mode only.
- `"S3 or R2 bucket"`.
- `"SFTP server"`.
- `"nsite site"`.
- `"Blossom server"`.

Evidence: `app.js:1387`.

The default is local folder in local mode and S3 in server mode.

### Add a local folder

Main path from a shell destination, with no existing host:

1. Click `"File hosts"`. `shell.js:281`.
2. Type `"Folder"`. `app.js:784`.
3. Click `"Add the Folder"`. `app.js:792`.

**INFERENCE — count:** 3. Existing hosts add the add-button click, giving 4.

Optional fields each add 1:

- `"Name (optional)"`. `app.js:782`.
- `"Public address of the folder (optional)"`. `app.js:787`.

No name uses the final folder-path segment. See `app.js:775`.

### Add an S3 or R2 bucket

Required typed fields:

1. `"Endpoint address"`. `app.js:876`.
2. `"Bucket"`. `app.js:883`.
3. `"Access key ID"`. `app.js:896`.
4. `"Secret access key"`. `app.js:898`.

Then click `"Add the Bucket"`. The busy label is `"Testing the Bucket…"`. See `app.js:902`.

Optional controls:

- `"Name (optional)"`.
- `"Region"` — starts as `"auto"`.
- `"Folder in the bucket (optional)"`.
- `"Address style"` → `"Path style"` or `"Virtual host style"`.
- `"Public address of the bucket (optional)"`.

Evidence: `app.js:874`.

**INFERENCE — count from shell, no existing host:** Server 6. Local mode 7, including the type selection. Existing hosts add 1.

Vault creation, when required, adds the section 3 sequence.

### Add an SFTP server

Start after selecting `"SFTP server"`.

1. Type `"Server"`. `app.js:1012`.
2. Type `"User name"`. `app.js:1016`.
3. Type `"Folder on the server"`. `app.js:1018`.
4. Type `"Private key"` for the default login. `app.js:1027`.
5. Click `"Connect"`. `app.js:1037`.
6. Examine the automatically fetched fingerprint.
7. Click `"Accept and Add the Server"`. `app.js:1004`.

Fingerprint examination has 0 action steps.

The fingerprint screen says:

> `Studio connected to ${hostName}. Make sure that this is the fingerprint of the server:`

Evidence: `app.js:1000`.

`"Back"` returns to the fields. See `app.js:1005`.

Login branches:

| `"Login"` choice | Required secret | Count change from default key path |
|---|---|---|
| `"Private key"` | `"Private key"` | 0 |
| `"Password"` | `"Password"` | +1 for changing Login |
| `"ssh-agent"` | No typed secret | 0 net: one Login selection replaces one secret field |

Evidence: `app.js:1024`.

Optional fields are `"Name (optional)"`, `"Port"` with default 22, `"Public address of the folder (optional)"`, and `"Passphrase of the key (optional)"`.

**INFERENCE — count from shell, no existing host:** Key 8. Password 9. ssh-agent 8. Existing hosts add 1. Vault creation adds section 3.

### Add nsite or Blossom

Both use `"Signing"` with:

- `"My signer"`.
- `"Key in the vault"`.

Evidence: `app.js:1123`.

Signer path:

1. Click `"Use My Signer"`. `app.js:1126`.
2. Complete external signer interaction, or the inline remote-signer flow.
3. Automatic lookup fills relay/server lists when available. `app.js:1069`.

Vault-key path:

1. Pick `"Key in the vault"`.
2. Type `"Private key of the site"`.
3. A valid nsec change triggers automatic list lookup.

Evidence: `app.js:1133`.

If no lists exist:

> “The key has no list. Type the addresses.”

Evidence: `app.js:1138`.

nsite fields:

| Field | Requirement/default | Evidence |
|---|---|---|
| `"Name (optional)"` | Optional | `app.js:1251` |
| `"Release (optional)"` | `"No release"` or `${f.title || f.path}` | `app.js:1253` |
| `"Site name"` | Can derive from selected release | `app.js:1260`, `app.js:1227` |
| `"Relays"` | At least one address | `app.js:1263`, `app.js:1247` |
| `"Blossom servers"` | At least one address | `app.js:1265` |
| `"Gateway"` | Starts as `"nsite.lol"` | `app.js:1267`, `app.js:1212` |
| `"Title of the site (optional)"` | Optional | `app.js:1269` |

Submission: `"Add the Site"`. `app.js:1271`.

Blossom has `"Name (optional)"` and `"Blossom servers"`. Submission: `"Add the Blossom File Host"`. See `app.js:1300`.

It shows:

> “A Blossom file host keeps media only. The feed XML needs a different file host.”

Evidence: `app.js:1298`.

**INFERENCE — counts from shell, no existing host, default lists returned:** 4 repository-controlled steps for either signer path: destination, type, signer button, submit. External signer actions are additional.

Typing missing lists adds 1 for Blossom and 2 for nsite. The vault-key path adds 2 relative to that signer-button path.

### Connection result

After adding a host, the page automatically runs `/connectors/${connector.id}/match`, reloads cards, and closes the add form. See `app.js:2775`.

Possible result notices:

- `Studio can now write ${linked.feeds.length} feed${linked.feeds.length > 1 ? 's' : ''} through “${linked.connector.name}”.`
- `Studio found the public address of “${linked.connector.name}” from its feeds.`
- `The feeds in “${linked.connector.name}” give different public addresses.`
- `"Studio did not set a public address for the folder. The file host card lists the feeds."`

Evidence: `app.js:2799`.

The page scrolls the result into view. The notices stay in that page instance. See `app.js:2761`.

### Edit a file host

Entry: card `"Edit"`, absent for `connector.kind === 'temporary'`. See `app.js:2595`, `app.js:2668`.

1. Click `"Edit"`.
2. Type a changed field.
3. Click `"Save"`.

**INFERENCE — count:** 3 from the card, plus one for each additional field.

Editable fields:

- Every non-temporary host: `"Name"`.
- Local/S3/SFTP: `Public address of the ${noun}`, where `noun` is `"bucket"` for S3 and `"folder"` otherwise.
- nsite: `"Relays"`, `"Blossom servers"`, `"Gateway"`.
- Blossom: `"Blossom servers"`.

Evidence: `app.js:2633`, `app.js:2673`.

`"Cancel"` restores the card without saving. There is no confirmation. Successful Save returns to the card without a success notice.

### Check an SFTP connection and accept a changed key

Entry: SFTP card `"Check the Connection"`. See `app.js:2740`.

One click lists the connector feed folder.

A changed key shows:

> “The server gives a new host key.”

> “Make sure that this is the fingerprint of the server before you accept it:”

It offers `"Accept the New Key"`. See `app.js:2742`.

**INFERENCE — count:** 1 to check. 2 to check and accept.

If acceptance finds another key:

> “The server gives a different key now.”

> “Connect again to see it.”

Evidence: `app.js:2749`.

A successful check without a changed key has no explicit success message.

### Check nsite relays

Entry: nsite card `"Check the Relays"`. See `app.js:2727`.

One click fetches site state.

Success:

> “Each relay has the newest manifest.”

Failure state:

> “The relays do not agree yet. Studio shows the site as inconsistent until each relay has the newest manifest.”

Each differing relay shows `${r.error}`, `older manifest ${r.id.slice(0, 8)}`, or `"no manifest"`.

Evidence: `app.js:2729`.

**INFERENCE — count:** 1. The result stays in the card.

### Remove a file host

Entry: card `"Remove"`, with accessible label `Remove the file host ${connector.name}`. See `app.js:2664`.

1. Click `"Remove"` → confirmation.
2. Click `"Remove the File Host"`.

Confirmation text is in section 13.

**INFERENCE — count:** 2 from the card.

Success removes the card without a result notice.

`in_use` shows:

> “Feeds use this file host.”

> “Move those feeds to a different file host, or remove them from your collection.”

Evidence: `app.js:2795`.

### Move all releases to another host

Entry: card `"Move All Releases…"`, when `connectors.length > 1`. See `app.js:2666`.

1. Click `"Move All Releases…"`.
2. Optionally pick `"To this file host"`.
3. Click `"Review the Changes"`.
4. Click `Write ${plural(count, 'Feed', 'Feeds')}` on the change-set page.

Evidence: `editor.js:4385`, `editor.js:4408`, `editor.js:4429`, `app.js:3092`, `recipients.js:416`.

**INFERENCE — count:** 3 with the default destination. Selecting another destination adds 1.

The sheet also has:

- `Delete the files on ${host} when each move is complete` — checked initially.
- `"Keep the old files for"` → `"7 days"`, `"30 days"`, `"90 days"` — starts at 30.
- `"Cancel"` and header `"Close"`.

Evidence: `editor.js:4389`, `editor.js:4415`, `releasemove.js:103`.

Checked note:

> “Studio asks you first. The feed pointer file always stays.”

Unchecked note:

> “The old files stay, and Studio does not ask again.”

Evidence: `releasemove.js:105`.

The review page writes feeds in sequence. A failure can leave later work available for another write attempt. See `app.js:3059`.

A conflict says:

> “The hosted file changed after the draft started. Studio did not write this feed.”

Evidence: `app.js:3005`.

The page shows per-feed outcomes and write states. It does not automatically leave after completion. See `app.js:3077`, `app.js:3102`.

`"Discard"` asks for confirmation only when some feed was written. See `app.js:3081`.

A missing change set shows:

> “Studio did not find these changes.”

> “It is written or discarded.”

Action: `"Open People"`. This action also appears for this host-area page. See `app.js:3094`.

## 7. Make a new release or another new feed

### Entry points for a new release

| Entry | Condition | Evidence |
|---|---|---|
| Publisher group toolbar `"＋ Add a Release"` | Writable publisher feed includes artist role | `app.js:3445`, `welcome.js:129` |
| Artist page first-step button `"Add a Release"` beside `"Add your first album"` | No releases and no child publisher feeds | `editor.js:4019`, `welcome.js:117` |
| Publisher-page add row `"Add a Release"` | `releaseAdd(...)` returns a value | `editor.js:3970`, `welcome.js:129` |
| New label first-step row `"Add a release"` | First role is label, with no releases or children | `welcome.js:154`, `editor.js:4026` |

The current `releaseAdd` function requires artist role membership. It returns `"Add a Release"`. See `welcome.js:129`.

### Main path: release for an artist

Start: one of the artist release actions above.

1. Click the release action → `"Make a new release"` sheet.
2. Type `"Album title"`. `addfeeds.js:43`, `app.js:1478`.
3. Click `"Make the Release"`. `addfeeds.js:43`, `app.js:1501`.

The artist name is supplied by the entry point. The first connector is selected. The path is generated from the title. See `welcome.js:131`, `app.js:1416`, `app.js:1430`.

**INFERENCE — count:** 3.

Optional controls:

- `"Kind"` → `"From the tracks"`, `"Album"`, `"EP"`, `"Single"`. `addfeeds.js:35`.
- `"Artist"`. `app.js:1480`.
- `"Start from"` when existing music feeds are available. Default placeholder `"Empty album"`. `app.js:1433`, `app.js:1482`, `addfeeds.js:84`.
- `"File host"` when more than one connector exists. `app.js:1484`.
- `"Feed file"` disclosure, then `"Path"`. `app.js:1491`.

Changing an optional field adds 1. Editing Path adds 2: open disclosure and type.

### Label branch

The label’s `"Add a release"` action:

- Opens `"Add a release"` artist picker when writable artist pages exist.
- Opens artist-page creation directly when none exist.

Evidence: `editor.js:4014`.

With available artists:

1. Click `"Add a release"`.
2. Pick `"Artist"` from `"Select an artist page"`.
3. Type `"Album title"`.
4. Click `"Make the Release"`.

**INFERENCE — count:** 4. See `editor.js:2609`, `editor.js:4030`.

The picker component has an empty state:

> “No artist page is available. Make an artist page first.”

Action: `"Make an Artist Page"`. See `editor.js:2618`.

The inspected label-entry handler normally bypasses that picker when no choices exist.

### New artist, label, hosting page, show feed, or playlist

Entry: Releases → `"＋ Add a Feed"` → a row under `"Make a new feed"`.

| Row | Required fields after entry | Submit label | Sheet title |
|---|---|---|---|
| `"Artist or label"` | `"Name"`, `"Role"` | `"Make the Page"` | `Make a new ${PAGE_WORDS[role] || 'artist or label page'}` |
| `"Show"` | `"Show feed title"`, `"Artist"` | `"Make the Show Feed"` | `"Make a new show"` |
| `"Playlist"` | `"Playlist name"`, `"Author"` | `"Make the Playlist"` | `"Make a new playlist"` |

Evidence: `addfeeds.js:11`, `addfeeds.js:43`, `addfeeds.js:60`.

Role choices are `"Artist"`, `"Label"`, and `"Hosting"`. See `app.js:1489`, `shell.js:17`.

**INFERENCE — count:** 5 from Releases for each row with default host/path. Optional selections add steps.

The non-album form includes `"Type"`, so the user can change between `"Artist or label"`, `"Show"`, and `"Playlist"`. See `app.js:1470`.

No connected host gives no make rows. The page instead offers `"Connect a File Host"`. See `addfeeds.js:21`, `app.js:2963`.

### Errors and end state

- `file_exists`: `"A file exists at this path already."`
- `no_public_address`: `"Give this file host a public address first, in File hosts."`
- Other failures use the shared error notice.

Evidence: `app.js:1431`, `app.js:448`.

A mapped field error automatically opens `"Feed file"`. See `app.js:1452`.

**End state:** Collection reload, then the new editor page. This creation path writes a draft, not the host file. See `app.js:1464`, `app.js:3961`.

There is no confirmation. The editor’s unwritten-file notice remains until the relevant state changes.

## 8. Change Settings, theme, and UI scale

### Settings entry and sections

Entry: `"Settings"` in the sidebar/root footer. See `shell.js:282`.

Settings opens on `"General"` unless its navigation view supplies a valid tab. See `app.js:2888`.

| Tab | Sections and conditions | Evidence |
|---|---|---|
| `"General"` | Appearance, Lock, Window when available, Outside changes, Version | `pages.js:70`, `app.js:2873` |
| `"Account"` | Server only | `pages.js:75` |
| `"Nostr"` | Nostr search relay | `pages.js:77` |
| `"Apps"` | Connected apps | `pages.js:76` |
| `"Data"` | Collections in desktop app, Move Studio in local shell without collections, Library or Files on this server | `pages.js:81` |

Tabs use segmented buttons when they fit. Narrow width or insufficient space uses a select labeled `"Settings tabs"`. Pending options add `" (Not saved)"`. See `app.js:2208`.

### Main path for a saved setting

1. Click `"Settings"`. `shell.js:282`.
2. Select the applicable tab, if General is not the target. `app.js:2900`.
3. Type or pick one setting.
4. Click toolbar `"Save"`. `app.js:2898`.

**INFERENCE — counts:** General setting 3. Other-tab setting 4. Each extra changed field adds 1.

`"Discard"` restores saved settings. It does not open a confirmation. See `app.js:2847`, `app.js:2897`.

Successful Save leaves `"Saved"` in the toolbar. A later edit or discard changes this state. See `settingsdraft.js:33`.

The initial status is `"No changes"`. Pending states are `${count} change not saved` or `${count} changes not saved`. See `people.js:1025`.

Changing tabs keeps pending settings and asks no leave question. Leaving Settings with pending changes asks the confirmation in section 13. See `settingsdraft.js:81`.

Browser reload/close also gets a `beforeunload` handler. Its visible native text is **UNSURE**, because the browser supplies it. See `app.js:2841`.

### Every draft setting

| Control | Choices or behavior | Evidence |
|---|---|---|
| `"UI scale"` under `"Appearance"` | `"Extra small"`, `"Small"`, `"Medium"`, `"Large"`, `"Extra large"` | `app.js:4749`, `uiscale.js:8` |
| `"Lock Studio after"` | Server-supplied minute choices | `app.js:4772` |
| `"When I close the window"` | `"Keep Studio in the tray"`, `"Quit Studio"` | `app.js:4793` |
| `"Check for outside changes"` | Checkbox | `outsidecheck.js:12`, `app.js:4944` |
| `"Nostr search relay"` | Relay address or empty field | `app.js:2438` |
| `"Largest download, in MB"` | Number greater than zero | `app.js:2416` |
| `"Free space to keep, in MB"` | Number greater than zero | `app.js:2418` |

Scale changes preview immediately. Save writes `studio-ui-scale` to local storage. Discard restores the saved scale. See `app.js:4741`, `app.js:2861`, `app.js:2851`.

Appearance is absent with `pointer: coarse`. See `app.js:4740`.

Window is absent if `/window-close` is unavailable or reports `available: false`. See `app.js:4788`.

Lock text:

> “Studio closes the vault after this time with no use of a file host key. A longer time keeps the key in memory for longer.”

Evidence: `vaultlock.js:4`.

Local choices are 0, 5, 10, 30, and 60 minutes. Server choices respect configured limits and include its configured start value. See `api/workspace.rs:53`, `api/workspace.rs:1204`.

Labels include `"Never"`, `"1 hour"`, and `${minutes} minutes`. See `vaultlock.js:6`.

Nostr hint:

> “Studio sends the text of a name search to this relay. Leave the field empty to search only your feeds and Music Index.”

Evidence: `app.js:2439`.

### Settings failures

| Condition | Exact text or display | Evidence |
|---|---|---|
| Nonpositive library value | `"Enter a number greater than 0."` | `settingsdraft.js:42` |
| Nonempty relay without ws/wss prefix | `"A relay address starts with wss:// or ws://."` | `settingsdraft.js:45` |
| Save request fails | `${error.message}` or `"Could not save this setting."`, beside affected fields | `settingsdraft.js:64` |
| Some saves succeed and others fail | Successful values become saved. Failed values remain pending | `settingsdraft.js:69` |

The toolbar shows the first validation cause that disables Save. See `app.js:2892`.

`LibraryPanel` and `NostrPanel` have no rendered load-error notice in their loaders. They remain absent until data is present. See `app.js:2396`, `app.js:2429`.

### Read server storage information

Entry: Settings → `"Data"`.

When `server && !holdAssets`, `"Files on this server"` replaces Library. See `pages.js:89`.

It shows:

- `"Audio and images"` — `"Kept until the write, for a maximum of 7 days"`.
- `"Feed XML, drafts and versions"` — `"Kept"`.
- `"Space in use"` — `${size(storage.used)} of ${size(storage.limit)}`, when supplied.

Evidence: `assets.js:115`, `app.js:2385`.

It has no editing control.

### Change theme

Entry points:

- Header of each wide sidebar state. `app.js:3137`, `app.js:3142`, `app.js:3146`.
- Header of the narrow root destination page. `app.js:3350`.

The icon button’s exact accessible label and title are:

> `Use the ${next} theme`

Evidence: `app.js:412`.

One click switches between light and dark and stores the choice immediately. There is no Save or result notice. See `app.js:406`.

**INFERENCE — count:** 1 from a visible theme control. Reaching the narrow root can add one Destinations click.

At startup, the stored theme wins. Without a valid stored theme, the initial system color preference is used. See `theme.js:3`.

### Read or copy the version

Entry: Settings → `"General"` → `"Version"`.

`"Copy version"` copies `${version}`. See `app.js:4954`, `app.js:1938`.

Success gives a check icon and accessible `"Copied"` status for two seconds. Clipboard failure is silent. See `app.js:1929`.

**INFERENCE — count:** 2 from the shell: Settings, Copy version.

No update control or About page is registered in the inspected UI and desktop source. The visible version section is the application-provided version surface found here.

## 9. Manage collections, export data, import data, and delete an account

### Collections

Entry: Settings → `"Data"` → `"Collections"`.

Condition: local shell with `hasCollections`. See `pages.js:81`, `app.js:5074`.

Each row has `"Actions ⌄"`, accessible as `Actions for ${collection.name}`. See `app.js:5103`, `menu.js:27`.

| Collection state | Menu entries | Evidence |
|---|---|---|
| Open | `"Rename…"`, `"Copy…"`, `"Export…"` | `collectionlist.js:19` |
| Closed | `"Open"`, `"Rename…"`, `"Export…"`, `"Replace with a File…"`, `"Delete…"` | `collectionlist.js:25` |

Bottom actions: `"New Collection…"` and `"Import an Account File…"`. See `collectionlist.js:41`.

### Main collection paths

Counts below start in the Collections section.

| Job | User steps | End state | INFERENCE count |
|---|---|---|---|
| Open closed collection | `"Actions ⌄"` → `"Open"` | Automatic restart on that collection | 2 |
| Rename | `"Actions ⌄"` → `"Rename…"` → type `"Name"` → `"Rename"` | Sheet closes and list reloads | 4 |
| New collection | `"New Collection…"` → type `"Name"` → `"Make and Open"` | Automatic restart on new collection | 3 |
| Copy open collection | `"Actions ⌄"` → `"Copy…"` → `"Copy and Open"` | Automatic restart on copy | 3 with default name |
| Start account-file import collection | `"Import an Account File…"` → type `"Name"` → `"Make and Open"` | Opens new collection, then user imports on its first page | 3 |
| Delete closed collection | `"Actions ⌄"` → `"Delete…"` → `"Delete Collection"` | Sheet closes and list reloads | 3 |
| Replace closed collection | `"Actions ⌄"` → `"Replace with a File…"` → `"Select the File"` → native file pick → `"Replace Collection"` | Sheet closes and list reloads | 5, native pick treated as one |

Evidence: `app.js:4982`, `app.js:5021`, `app.js:5089`, `collectionlist.js:53`.

Changing the supplied copy name adds 1 step.

Every collection-opening operation shows:

> “Studio restarts on this collection…”

The text clears after ten seconds if the page remains. See `collectionlist.js:136`, `app.js:5091`.

An active write/upload can interrupt restart with the desktop work dialog in section 13.

### Collection warning and confirmation text

Copy sheet:

> `The copy needs ${size} on this computer. It holds the feeds, the drafts, the versions, the people, the library and the vault. It holds no connected app and no queued announcement.`

> “This copy can write to the same file hosts.”

> `A write from the copy changes the real feed. The passphrase of the copy is the passphrase of ${open}.`

Evidence: `collectionlist.js:105`.

Delete sheet title:

> `Delete “${name}”?`

Body:

> `${name} holds ${feedsText(collection)}. Studio removes the feeds, the drafts, the versions, the people and the vault from this computer. You cannot get them back.`

> “Nothing changes on a file host.”

Buttons: `"Delete Collection"`, `"Export First…"`, `"Cancel"`. Cancel gets initial focus. See `collectionlist.js:80`, `app.js:5061`.

Replace sheet title:

> `Replace “${name}” with a File`

Body:

> `${name} holds ${feedsText(collection)} now. Studio reads the file first, and a file that is not good changes nothing.`

> “After a good read, the feeds, the drafts, the versions, the people and the vault of the file take the place of the current ones. Nothing changes on a file host.”

> “The vault of the file opens with a method of the file, not with the method of the vault that goes.”

Buttons: `"Replace Collection"`, `"Export First…"`, `"Cancel"`. See `collectionlist.js:66`.

Enter does not start a destructive collection action. See `app.js:5038`.

### Export an account, Studio, or collection

Entry points:

- Server Account → `"Export Account"`. `app.js:4644`.
- Local shell without collections → `"Export Studio"`. `app.js:5138`.
- Collection Actions → `"Export…"`. `collectionlist.js:23`.
- Replace/delete collection sheet → `"Export First…"`. `app.js:5063`.

Main path after opening the export sheet:

1. Click `"Export Account"`, `"Export Studio"`, or `"Export Collection"` as its primary button.
2. Automatic download starts.
3. Click `"Close"` when finished with the notice.

Evidence: `move.js:25`, `app.js:4694`, `app.js:4704`.

**INFERENCE — count:** 1 step to start download after opening. Close adds 1.

When assets are held, `"Include the audio and images"` starts checked. Changing it adds 1. See `move.js:107`, `app.js:4717`.

The export notice is:

> “A person who has this file can read your feeds, drafts, versions and People list.”

> “Your passwords and keys stay locked in your vault. Keep this file private.”

Evidence: `move.js:50`.

If `!facts.movable`, the primary action is `"Make a Recovery Key"`.

Then:

1. Click `"Make a Recovery Key"`.
2. Reveal `"Recovery key"`.
3. Select `"I wrote down the recovery key in a safe place"`.
4. Click `"Continue"`.
5. Click the export primary button.

Evidence: `app.js:4684`, `app.js:4705`.

**INFERENCE — count:** 5 after opening the warning sheet, including key reveal.

For a closed collection that cannot move, no recovery-key button is offered. The hint says:

> “Open this collection, make a recovery key there, and export it again.”

Evidence: `app.js:4719`, `app.js:4721`, `collectionlist.js:98`.

After export:

> “The download started.”

> “A person who has this file can read your feeds, drafts, versions and People list. Keep this file private.”

Evidence: `move.js:117`.

This notice remains until the sheet closes.

### Import an account file

Entry points:

- Local Welcome → `"Import an Account"`. `app.js:3520`.
- Server Make an account → `"Import an Account"`. `app.js:4461`.
- Desktop collection import action first creates and opens an empty collection. It then directs the user to the first-page import. `collectionlist.js:116`.

Main passphrase path from a visible import entry:

1. Click `"Import an Account"` → `"Import an account"`.
2. Click `"Select the File"`. `app.js:5323`.
3. Pick `.tar` file → automatic upload and check. `app.js:5319`, `app.js:5254`.
4. Type `"Passphrase"` → `"Open the vault of the file"` form. `app.js:5341`.
5. Click `"Open and Make the Account"` on server, or `"Open and Import"` locally. `move.js:204`.

**INFERENCE — count:** 5, treating native file selection as one pick. Dropping a file reduces this by 1.

Branches:

| Condition | Added action or alternative | Evidence |
|---|---|---|
| Server invite signup | Type `"Invite code"` before file selection | `app.js:5317` |
| `view.needsEmail` | Type `"Email address"` | `app.js:5339` |
| Recovery method selected | `"Use the Recovery Key"` → type `"Recovery key"` | `move.js:190`, `app.js:5343` |
| Extension available and archive has Nostr wrappers | `"Use a Signer Extension"` | `move.js:170`, `app.js:5289` |
| Server and archive has Nostr wrappers | `"Use Amber or a Bunker"` | `move.js:177`, `app.js:5308` |
| No usable method | `"This browser has no way to open the vault of the file. Use a signer extension or a different computer."` | `app.js:5349` |

The upload shows:

- `Sending… ${sizeText(sent)} of ${sizeText(size)}`.
- `"Checking the file…"`.

Evidence: `move.js:142`, `move.js:146`.

Server import is a full-page card. Local import is a dialog sheet. See `app.js:5199`.

Return controls:

- Before check: `"Back to Sign Up"` on server, `"Cancel"` locally.
- After check: `"Cancel"` also requests deletion of the pending upload.
- Local sheet header `"Close"` and Escape.
- Remote signer page `"Stop"` or `"Back"`.

Evidence: `app.js:5330`, `app.js:5265`.

**End state:** Server reloads the session. Local import reloads the page. See `app.js:5277`.

The server Account panel can retain:

> “This account came from a file.”

> `Add a passkey for this server. Each connected app must use the address of this server: ${origin}.`

Without passkey support, the first sentence of the body is absent. `"Close"` dismisses the notice. See `move.js:128`, `app.js:4637`.

Import failure messages appear in section 14.

### Add or remove a passkey

Entry: server Settings → `"Account"`, with passkey browser support. See `app.js:4646`.

Add:

1. Optionally type `"Name of the new passkey"`. `app.js:4602`.
2. Click `"Add a Passkey"`. `app.js:4604`.
3. Complete native credential creation and an assertion.

The UI supplies `Passkey ${n}` when needed. See `account.js:307`.

**INFERENCE — count:** 1 repository-controlled step from the panel with the default name. Typing a name adds 1. Native steps are unknown.

Remove:

1. Click a passkey’s `"Remove"`. `app.js:4598`.
2. A recent-unlock request can add section 3.

There is no removal confirmation in this handler. The server can refuse the last method. See `app.js:4587`, `account.js:61`.

**End state:** Passkey list and session reload. No success notice.

### Delete the server account

Start: shell.

1. Click `"Settings"`.
2. Select `"Account"`.
3. Click `"Delete the Account"`. `app.js:4658`.
4. Type `"Type your email address to confirm"` or `"Type your npub to confirm"`. `app.js:4650`.
5. Click `"Yes, Delete the Account"`. `app.js:4653`.

**INFERENCE — count:** 5. A required unlock adds section 3.

The inline confirmation says:

> “Studio removes your feeds, your vault and your library files from the server. This cannot be undone.”

It offers `"Cancel"`. See `app.js:4649`.

Email confirmation ignores surrounding spaces and letter case. npub confirmation uses the trimmed exact npub. See `account.js:154`.

**End state:** Session reload, then Sign in. No result notice stays.

## 10. Connect apps and grant access to shows

### Pair an app

Entry: Settings → `"Apps"` → `"Connect an App"`. See `pages.js:105`, `app.js:2566`.

Main path from shell:

1. Click `"Settings"`.
2. Select `"Apps"`.
3. Click `"Connect an App"`.
4. Type `"Name of the app"`. `app.js:2557`.
5. Click `"Make Pairing Text"`. `app.js:2562`.
6. Click `"Copy"`. `app.js:2551`.
7. Give the text to the external app — **UNSURE external steps**.
8. Click `"Done"` after pairing. `app.js:2552`.

**INFERENCE — count:** 7 repository-controlled steps, excluding the external handoff.

The text field is hidden until its eye control is used. Copy does not require reveal. See `app.js:2547`, `app.js:571`.

Pairing notice:

> `The pairing text of “${made.name}”`

> “Select Copy, then give the text to the app in the next 15 minutes, for example with musicindex-live-publisher studio connect. Select Done after the app says that it is connected. Done before that ends the pairing text.”

Evidence: `apps.js:71`.

Automatic polling runs every two seconds. Successful pairing removes pairing text from state. See `app.js:2463`.

Success notice:

> `${made.name} is connected`

> “Compare the key with the key that the app shows. If the app did not connect, select Remove.”

Expired/removed pairing notice:

> `The pairing text of “${made.name}” ended`

> “The 15 minutes ended, or the pairing text was removed. Select Connect an App to make a new pairing text.”

Evidence: `apps.js:54`.

`"Done"` while waiting deletes the pending app. After connection, it only closes the notice. See `apps.js:81`, `app.js:2516`.

Clipboard failure has no notice. Successful Copy changes its label to `"Copied"`. See `app.js:2511`.

### Inspect, reconnect, or remove an app

Rows show:

- App name.
- `"Key"` plus short key, `"No key yet"`, or `"Token from before key pairs"`.
- Show count, waiting state, last-use time, and optional address.
- `"Live value address"` when supplied.

Evidence: `app.js:2528`, `apps.js:11`.

Legacy token state says:

> “Studio no longer accepts tokens.”

Action: `"Connect Again"`, accessible as `Connect the app ${a.name} again`. See `apps.js:12`, `app.js:2539`.

One click deletes the old registration and creates fresh pairing text for the same name. No confirmation is called. See `app.js:2507`.

Remove path:

1. Click row `"Remove"`, accessible as `Remove the app ${a.name}`.
2. Click `"Remove the App"` in the confirmation.

Evidence: `app.js:2537`, `app.js:2497`.

**INFERENCE — counts from Apps panel:** Reconnect 1 to obtain new text. Remove 2.

Successful removal reloads the list with no success notice.

### Give a connected app a show grant

Entry: feed toolbar `"Review"` for a pending live show with a GUID. See `editor.js:2301`, `editor.js:1875`.

Main path, one active app:

1. Click `"Review"`.
2. Select `Let ${app.name} start and end this show, and set its live value${suffix}`.
3. Click `"Write to the File Host"`.

Evidence: `apps.js:113`, `editor.js:1891`, `editor.js:2227`.

**INFERENCE — count:** 3 from the feed page.

With multiple apps, step 2 becomes a selection in:

> `Connected app for this show${suffix}`

Choices include `"No app"` and app names. Duplicate names can include ` · ${a.short_key}`. See `apps.js:120`.

For multiple pending shows, `suffix` is `: ${showTitle || 'Untitled show'}`. See `apps.js:96`.

Branches:

- No app: disabled `"No app"` menu and `"No app is connected. Connect an app in Settings."`. `apps.js:97`.
- nsite/Blossom `"My signer"` host blocks a new grant. Existing grants can still be removed. `apps.js:94`, `editor.js:1871`.
- Blocking note: `"Connected apps cannot write to a file host with My signer, because no person is present to sign."`. `live.js:202`.
- Review/write can require unlock or signer connection.

The grant control updates the draft grant request and returns an updated diff. The final write uses that diff. See `editor.js:1877`.

### Inspect or stop a grant

Entry: open a live-show page → `"Grant"` section. See `editor.js:3664`.

The page loads grants and app names automatically. It shows grant summary and granted-write history. See `editor.js:3605`.

Main stop path:

1. Click `"Stop the Grant"`.
2. Click `"Stop the Grant"` in the confirmation.

Evidence: `editor.js:3670`, `editor.js:3623`.

**INFERENCE — count:** 2 from the show page.

This posts the grant-end operation immediately and reloads grants. It does not wait for a feed write. See `editor.js:3626`.

Grant state text includes:

- `"No grant."`
- `The grant ended at ${timeText(grant.ended_at)}.`
- `${who} can start and end this show.`
- `${who} can start and end this show until ${timeText(limit)}.`

Evidence: `live.js:135`.

Write history can show:

> `${write.app_name} · ${what}: ${why} · ${timeText(write.written_at)}`

Refusal causes include `"the stream was not live"`, `"the feed had a draft with changes"`, `"the grant had ended"`, `"Studio refused the live value"`, and `"the grant vault did not open"`.

Evidence: `live.js:155`, `live.js:175`.

A draft can show:

> `${app} cannot start ${show} while this draft has changes`

> `Write or discard the draft before ${startText || 'the show starts'}. Select Review to write it.`

Evidence: `apps.js:140`, `editor.js:5080`.

## 11. Add, operate, and remove live shows

### Entry and feed-type conditions

`"Live Shows"` is a feed tab for music feeds. Publisher feeds and playlists use different tab lists. See `shell.js:231`.

Rendering additionally requires `!publisher && !playlist`. See `editor.js:5033`.

There is no top-level Live Shows destination in the shell.

### Add a planned show

Start: music feed editor.

1. Select `"Live Shows"`. `shell.js:235`.
2. Click `"＋ Add a Live Show"`. `editor.js:3502`.
3. Type `"Title"`. `editor.js:3453`.
4. Type `"Start time"` or click `"Now"`. `editor.js:3454`, `app.js:520`.
5. Click `"＋ Add the Show"`. `editor.js:3460`.

The UI flushes the current draft, adds the live item through the API, then opens its page automatically. See `editor.js:5059`.

**INFERENCE — count:** 5. Starting from the Live Shows tab gives 4.

The empty list says `"No live show yet."`. See `editor.js:3494`.

The add sheet has `"Cancel"`, header `"Close"`, and Escape. Errors stay inside the sheet. See `editor.js:3449`, `editor.js:3455`.

### Edit the show

Entry: `${live.item.title || 'Untitled show'}` in the Live Shows list. See `editor.js:3496`.

The page contains these controls:

| Section | Controls | Evidence |
|---|---|---|
| Show | `"Title"`, `"Description"`, `"Start time"`, `"End time"`, `"Image address"` | `editor.js:3676` |
| Stream, before ended state | `"Stream address"`, `"Stream type"`, `"Check"` | `editor.js:3689` |
| Stream type | `"Choose a type"`, core-supplied `${option.label}`, `"Other…"` | `editor.js:3694` |
| Other type | `"Other stream type"` input | `editor.js:3701` |
| Content links | `"Add a content link"`, then `"Link address"` and `"Link text"` | `editor.js:3467` |
| Live value | `"Live value address"`, `"Live value protocol"`, `"Remove the Live Value"` | `editor.js:3741` |
| More | `"Identity"` disclosure, Show GUID facts | `editor.js:3752` |
| Removal | `"Remove the Show"` | `editor.js:3767` |

Each typed field or selection is one step. Draft saving is automatic. Host publication requires Review and write.

**INFERENCE — simple edit count:** Open show 1 + changed field 1 + Review 1 + Write 1 = 4.

Start time has `"Now"`. End time has `"Now"` and `"Clear"`. See `app.js:519`.

### Check a stream

Start: open show before ended state.

1. Type `"Stream address"`. `editor.js:3691`.
2. Click `"Check"`. `editor.js:3710`.

The UI automatically requests `/stream/check`. See `editor.js:3572`.

**INFERENCE — count:** 2. With an existing address, 1.

The result is:

> `Live · ${type} · checked ${checkedAt}`

or `${result.cause || 'The stream is not live.'}`.

Evidence: `live.js:263`.

A type disagreement can offer:

> `Use ${check.typeWarning.fillLabel}`

Evidence: `editor.js:3712`.

A check-request failure displays `${checkError.message}` below the check control. See `editor.js:3714`.

Typing an address can automatically ask the core for a type. It fills only while the user has not selected a nonblank type. See `editor.js:3548`.

The check result stays until a relevant address/type change or page replacement.

### Go live or end the show

Entry: show `"Status"` section.

- `status === 'pending'` → `"Go Live"`.
- `status === 'live'` → `"End the Show"`.
- Ended or other state → no next-action button.

Evidence: `live.js:68`, `editor.js:3658`.

Main path:

1. Click `"Go Live"` or `"End the Show"`.
2. Automatic draft flush and status request.
3. Review changes sheet opens automatically.
4. Click `"Write to the File Host"`.

Evidence: `editor.js:3630`, `editor.js:2227`.

**INFERENCE — count:** 2 from the show page.

The page explicitly says:

> “The button changes the draft and opens the review sheet. The file host does not change until you write.”

Evidence: `editor.js:3660`.

There is no separate Go Live/End confirmation before Review.

A refused start can show the heading `"Go Live refused."` plus the API message. See `app.js:480`.

Review can show core stream problems as warnings or write-blocking notices. It also handles draft conflict, stale draft, address failures, and signer errors. See `editor.js:2141`, `editor.js:2174`.

**End state:** Review closes after write. The draft reloads, and the show page shows `"Live now"` or `"Ended"`. See `editor.js:2059`, `live.js:57`.

There is no dedicated “went live” toast in this handler. Announcement and write notices can remain.

### Add a recording after the show ends

Condition: `live.status === 'ended'`. The Recording section replaces Stream. See `live.js:76`, `editor.js:3721`.

Entry: `"Choose…"` under `"Recording"`, or drop a file on its field. See `editor.js:3728`, `editor.js:1122`.

The shared picker offers release files, host files, or this computer. File selection applies to the draft and closes the picker. See `editor.js:1059`, `editor.js:914`.

For computer selection:

1. Click `"Choose…"`.
2. Select the computer source.
3. Click `"Select a File"`.
4. Pick the native file.
5. Automatic upload and draft change.
6. Click `"Review"`.
7. Click `"Write to the File Host"`.

**INFERENCE — count:** 6 repository/native-selection steps, treating native file pick as one. A direct drop reduces the selection sequence to one drop.

The page says:

> “Studio puts the audio file in the enclosure of the show. The stream address goes out of the draft.”

Evidence: `editor.js:3731`.

**Scope boundary:** Enclosure properties, alternate enclosures, payment editing, and image-picker internals are shared feed-editor jobs. Their entry controls are recorded here.

### Remove a live show

1. Click `"Remove the Show"`.
2. Click `"Remove the Show"` in the confirmation.

Evidence: `editor.js:3767`, `editor.js:3641`.

**INFERENCE — count:** 2 to remove it from the draft. Publishing removal adds Review and Write, giving 4.

The UI returns to Live Shows. See `editor.js:3644`.

If the current live item disappears after another draft operation, the view automatically changes to Live Shows. See `live.js:29`, `editor.js:4995`.

## 12. Announcements and outside-file notices

### Set whether writes send announcements

Entry points:

| Entry | Exact control | Evidence |
|---|---|---|
| Review footer | `"Send a podping"` | `announce.js:20`, `editor.js:1855` |
| Change-set review page | `"Tell Podcast Index and send a podping"` | `editor.js:1846`, `app.js:3101` |

Changing either checkbox saves `/settings/announcements` immediately. See `editor.js:1828`.

**INFERENCE — count:** 2 from a feed page: Review, checkbox. From an already open change-set page, 1.

Hints:

- `"Studio sends it 30 seconds after the write."`
- `Studio sends it at ${localTime(wait.until + 30)}, after the wait of Music Index.`
- When disabled: `"No index learns about this write until it reads the feed by itself."`

Evidence: `announce.js:10`.

Skip text replaces the footer checkbox when relevant:

- `"No podping: this feed has no public address."`
- `"No podping: this write deletes the feed file."`
- `"No podping: this write does not change the feed."`

Evidence: `announce.js:25`.

There is no confirmation. Save errors use `ErrorNotice`. See `editor.js:2205`.

### Inspect an announcement

Entry: a feed’s `"Feed facts"` disclosure, summary `"GUID, file, version and announcement"`. See `editor.js:2348`.

One click reveals `"Announcement"`.

Status text:

- `"No announcement yet."`
- `"Not loaded yet."`
- `Waiting. Studio sends it at ${timeText(entry.next_at)}.`
- `Waiting. Studio tries again at ${timeText(entry.next_at)}. Last error: ${entry.last_error}`
- `Sent at ${timeText(entry.finished_at)}.`
- `Failed at ${timeText(entry.finished_at)}: ${entry.last_error}`

Evidence: `announce.js:35`, `editor.js:2332`.

The page reloads announcement state every five seconds while the newest entry waits or sends. See `editor.js:2315`.

**INFERENCE — count:** 1 from the feed page. The resulting status remains in the open disclosure.

### Send or resend a podping

Entry: move/write notice lines.

- `"Send the Podping"` for `not_sent`.
- `"Send the Podping Again"` for sent/failed.
- Both require an own public URL.

Evidence: `outsidecheck.js:94`.

One click posts `/feeds/${feed.id}/announcements` and reloads the notice. See `app.js:4872`.

**INFERENCE — count:** 1.

There is no confirmation. Errors show within the notice through `ErrorNotice`. See `app.js:4920`.

### Respond to changes in the library folder

Automatic entry: one scan after feed loading, only when `holdAssets` is true. See `app.js:3913`.

The shell banner says:

> `Studio found ${differences.length} ${differences.length === 1 ? 'change' : 'changes'} in the library folder.`

Actions:

- `"Show"` → opens Files for `differences[0].feed_id`.
- Header-style `"Close"` icon → hides the banner for this shell instance.

Evidence: `app.js:4021`.

**INFERENCE — count:** 1 for either action.

Changed feed rows also have an accessible `"Changes in the library folder"` mark. See `app.js:3460`.

The automatic first-scan failure is silent. Manual file actions belong to the editor Files job. See `app.js:3924`.

### Respond to a hosted file changed outside Studio

Automatic checks run after feed-list load and each time Releases opens. They run at most four reads together. See `app.js:4811`, `outsidecheck.js:10`.

A feed page also loads `/outside?index=true`. See `app.js:4838`.

| State | Exact text | Actions | Evidence |
|---|---|---|---|
| Changed file | `"The hosted file changed outside Studio."` and `"Read Again takes the new file, and it keeps a draft with edits."` | `"Read Again"` | `outsidecheck.js:45`, `app.js:4910` |
| Moved feed | `"A different Studio moved this feed."`, `"New address"`, `"The old address has a pointer file."` | `"Follow the Move"`, `"Keep This File Host"` | `outsidecheck.js:28` |
| Moved and connected | Same | Also `"Show the Difference"` | `outsidecheck.js:36`, `app.js:4911` |
| Moved, connected, and `new_host` exists | Same | Also `"Move It Back…"` | `outsidecheck.js:40`, `app.js:4909` |

Main changed-file path:

1. Click `"Read Again"`.
2. Automatic draft flush, feed refresh, and draft reload.

Evidence: `app.js:2246`.

**INFERENCE — count:** 1. There is no confirmation here.

Moved-feed actions:

- `"Follow the Move"` posts `/outside/follow`, reloads the collection, and reloads the notice. `app.js:4882`.
- `"Keep This File Host"` posts `/outside/keep` and reloads the notice. `app.js:4868`.
- `"Show the Difference"` opens Review in merge mode. `app.js:4911`, `editor.js:2290`.
- `"Move It Back…"` first follows the move, then opens the usual move sheet with the old connector as target. `app.js:4890`.

**INFERENCE — count:** 1 to follow, keep, open difference, or reach the return-move sheet. Subsequent merge/move review is a feed-editor path.

### Read-only host notices

| Condition | Text | Action |
|---|---|---|
| No connector and `host_for_address === true` | `"A connected file host gives the address of this feed. Use it to edit the feed here."` | `"Use This File Host"` |
| No connector and `followed_move === true` | `"This Studio is not connected to the file host of this address. Connect it to edit the feed here."` | `"Connect a File Host…"` |

Evidence: `outsidecheck.js:59`.

The first action posts `/outside/use-host`. See `app.js:4898`.

**UNSURE — second action:** Its handler calls `nav.go(...)`, but the supplied `NavContext` value has no `go` member. See `app.js:4916`, `app.js:4013`.

The source expresses an intent to open File hosts. The actual result of that click was not verified.

### Other address and index notices

`AddressDifferences` shows:

> “Other sources give a different address.”

> “Studio uses the public address above. Studio does not change the feed file.”

It lists source labels and URLs. It is hidden for an outside result with `state === 'moved'`. See `app.js:649`.

A different Music Index address shows:

> “Music Index reads this feed from a different address.”

> `${index.index}. Your feed is at ${index.own}.`

> “A different collection or a different tool can hold this feed.”

Evidence: `outsidecheck.js:77`, `app.js:4917`.

These notices have no action controls.

### Notice after a write or move

Possible headings include:

- `You moved this feed at ${localTime(check.last_move.moved_at)}. Music Index reads the new address after it gets the podping.`
- `You wrote this feed at ${time}. Music Index reads it after it gets the podping.`
- `Music Index has not read your write of ${time} yet.`
- `Music Index has your write of ${time}.`

Evidence: `outsidecheck.js:117`, `outsidecheck.js:136`.

`"Check Again"` appears after read-count or age limits. One click resets the check window and reloads. See `outsidecheck.js:146`, `app.js:4876`.

These notices update automatically. A confirmation that Music Index has the write is shown only when this page previously showed a waiting state. See `outsidecheck.js:142`.

## 13. Confirmation and desktop-dialog register

### Studio confirmation sheets

All `askConfirm` sheets render through `ConfirmHost`. They start focus on the safe button. Header `"Close"` and Escape return false. See `app.js:5410`, `confirms.js:83`.

| Trigger | Title | Exact body | Buttons | Evidence |
|---|---|---|---|---|
| Remove app | `Remove the app “${name}”?` | `"The grants of the app end at once. The app can no longer change your live shows."` | `"Remove the App"`, `"Cancel"` | `confirms.js:19` |
| Remove host | `Remove the file host “${name}”?` | `"Studio forgets this file host. The files on it do not change."` | `"Remove the File Host"`, `"Cancel"` | `confirms.js:23` |
| Stop grant | `"Stop the grant?"` | `"Connected apps can no longer change this show."` | `"Stop the Grant"`, `"Cancel"` | `confirms.js:47` |
| Remove show | `Remove “${title || 'Untitled show'}”?` | `Studio removes the show from the draft. The feed changes at the next write.${live && live.status === 'live' ? ' Apps lose the stream of this show at that write.' : ''}` | `"Remove the Show"`, `"Cancel"` | `confirms.js:51` |
| Leave Settings with pending values | `"Leave without saving?"` | `"Your Settings changes are not saved."` | `"Discard Changes"`, `"Keep Editing"` | `confirms.js:68`, `app.js:2836` |
| Discard partly written change set | `"Discard these changes?"` | `"Each feed that you did not write gets back its draft from before the change. A feed that you wrote does not change."` | `"Discard the Changes"`, `"Cancel"` | `confirms.js:27`, `app.js:3081` |
| Remove unsupported feed from collection | `Remove “${name}” from your collection?` | `"The feed file on its file host does not change. You can add the feed again later."` | `"Remove from Collection"`, `"Cancel"` | `confirms.js:15`, `app.js:2007` |
| Discard draft in Review, used by live/grant paths | `"Discard the draft?"` | `"Studio discards any change that you did not write. The feed file on its file host does not change."` | `"Discard the Draft"`, `"Cancel"` | `confirms.js:31`, `editor.js:2068` |

Collection replacement/deletion and server-account deletion use their own forms, described in section 9.

### Close the desktop window

Entry: operating-system window close control. Its exact visible label is **UNSURE**.

Code conditions:

- No tray host → quit path.
- Stored `OnClose::Tray` → hide window.
- Stored `OnClose::Quit` → quit path.
- No stored choice and a tray host → native question.

Evidence: `desktop/main.rs:95`, `desktop/main.rs:484`.

Question:

> “Keep Studio running in the tray?”

> “A connected app can only write while Studio runs. You can change this in Settings.”

Buttons:

- `"Keep in Tray"`.
- `"Quit on Close"`.

Evidence: `desktop/main.rs:45`, `desktop/main.rs:125`.

Closing this dialog without an answer leaves the window open. See `desktop/main.rs:136`.

**INFERENCE — counts:** Previously chosen hide/idle-quit path 1 close action. First question path 2 actions. Active work can add another dialog selection.

### Quit or change collections while work runs

Entry:

- Tray `"Quit"`.
- Window close configured to quit.
- Collection opening/restart.

Evidence: `desktop/main.rs:255`, `desktop/main.rs:493`, `desktop/main.rs:196`.

No active write/upload: quits without a work dialog. See `desktop/quit.rs:75`.

Active work title is generated from:

- `Studio Is Writing ${writes} Feed`.
- `Studio Is Writing ${writes} Feeds`.
- `Studio Is Receiving ${uploads} Music File`.
- `Studio Is Receiving ${uploads} Music Files`.
- Combined: `Studio Is ${w} And ${r}`.
- Fallback: `"Studio Is Working"`.

Evidence: `desktop/quit.rs:89`.

Quit body:

> “If you quit now, the write stops. You then must do the write again.”

Buttons: `"Quit After Write"`, `"Quit Now"`, `"Cancel"`.

Collection-change body:

> “If you open the collection now, the write stops. You then must do the write again.”

Buttons: `"Open After Write"`, `"Open Now"`, `"Cancel"`.

Evidence: `desktop/quit.rs:110`, `desktop/quit.rs:120`.

Wait hides the window and checks work every second. See `desktop/main.rs:309`, `desktop/main.rs:351`.

Waiting tray entries:

- `"Quits after the write…"` or `"Opens the collection after the write…"`, disabled.
- `"Open Studio"`.
- `"Quit Now"` or `"Open Now"`.

Evidence: `desktop/main.rs:260`, `desktop/quit.rs:143`.

Opening Studio during the wait cancels that wait and a pending collection change. See `desktop/quit.rs:79`, `desktop/quit.rs:175`.

**INFERENCE — count:** Idle tray quit 1 menu selection. Active-work quit 2 selections after the tray menu is accessible. Opening the tray menu can add one OS action.

### Desktop file dialogs, downloads, and links

- The desktop supplies a system folder picker to the local API. It starts in the library folder. `desktop/main.rs:573`.
- The release library-folder change uses that picker in the desktop app. `editor.js:4459`.
- First-run library selection uses a text field, as recorded earlier.
- Import and backup controls use HTML file inputs. Their native picker labels are **UNSURE**.
- Downloads go to Downloads, with home-folder fallback. Duplicate names get ` 2`, ` 3`, and so on. `desktop/downloads.rs:8`, `desktop/downloads.rs:29`.

Download-result dialog body:

- `Studio saved ${name} in your Downloads folder.`
- `The download of ${name} did not complete.`

Title: `"Music Index Studio"`. Button: native `"OK"` equivalent from `MessageDialogButtons::Ok`.

Evidence: `desktop/downloads.rs:40`, `desktop/main.rs:469`.

External web links open through the system opener. A failed opener shows:

> `Studio could not open this address in your browser:\n${address}\n\nOn Linux, Studio uses the program xdg-open for links. Install the package xdg-utils, then try again.`

Evidence: `desktop/main.rs:380`.

**UNSURE:** Native dialog button spelling and placement depend on the dialog implementation and platform.

## 14. Shared error and failure register

These catalogs apply to the journeys above. API-supplied text remains dynamic where the renderer inserts `${error.message}`.

### General API notices

`ErrorNotice` renders a strong title followed by `${error.message}`. See `app.js:441`.

| Code | Exact title |
|---|---|
| `wrong_key` | `"The passphrase is not correct."` |
| `passphrase_too_short` | `"The passphrase is too short."` |
| `passphrase_too_weak` | `"The passphrase is too easy to guess."` |
| `invalid_path` | `"That path is not in the folder."` |
| `not_found` | `"Studio did not find the file."` |
| `invalid_feed` | `"That file is not a feed that Studio can read."` |
| `offline` | `"No connection to Studio."` |
| `query_field` | `"This search cannot run."` |
| `index_answer` | `"Music Index gave a bad answer."` |
| `stream_not_live` | `"Go Live refused."` |
| Other | `"Something went wrong."` |

Evidence: `app.js:468`.

Fetch failure supplies:

> “Studio does not answer. Make sure that it is still running.”

Evidence: `app.js:170`.

Mapped field messages include:

- `"Studio did not find this file."`
- `"This path is not in the folder."`
- `"This file is not a feed that Studio can read."`
- `"Give the full path of the folder, for example /home/me/feeds."`
- `"Studio did not find this folder on this computer."`
- `"Give this file host a public address first, in File hosts."`
- `"A file exists at this path already."`
- `"Give a full address. Start it with https:// or http://."`

Evidence: `app.js:448`.

These field errors offer the same form controls for another attempt. General notices have no built-in action button.

### Connector-specific failures

S3:

- `"Studio cannot write to this provider safely"` plus `${error.message}`.
- `"The provider refused the access key."`
- `"Studio cannot connect to the endpoint."`
- `"Unlock the vault first."`

Evidence: `app.js:849`.

SFTP:

- `"Studio did not find this folder on the server."`
- `"Studio cannot write to this server safely"` plus `${error.message}`.
- `"The server refused the login."`
- `"Studio cannot connect to the server."`
- `"Unlock the vault first."`

Evidence: `app.js:969`.

nsite/Blossom:

- `"This private key is not the key of the public key."`
- `"Studio cannot connect to the relays or the servers."`
- `"Unlock the vault first."`
- For mapped invalid configuration: capitalized `${error.message}` plus `"."`.

Evidence: `app.js:1145`.

The same form stays available. The SFTP handler returns to its first step for invalid address/configuration/folder errors. See `app.js:956`.

### Authentication failures

`AuthNotice` renders these within the current page or unlock sheet. See `app.js:4038`.

| Code | Exact text | Evidence |
|---|---|---|
| `sign_in_failed` | `"The email address or the passphrase is not correct."` | `account.js:45` |
| `email_in_use` | `"Another account has this email address."` | `account.js:46` |
| `invite_required` | `"Sign-up needs an invite code that nobody used."` | `account.js:47` |
| `challenge_invalid` | `"The sign-in request is too old or was used already. Try again."` | `account.js:48` |
| `too_many_attempts` | `"Too many sign-in attempts. Try again later."` | `account.js:49` |
| `busy` | `"The server is busy. Try again in a short time."` | `account.js:50` |
| `session_ended` | `"Your session ended. Sign in again."` | `account.js:51` |
| `wrong_key` | `"The key does not open the vault."` | `account.js:52` |
| `no_signer` | `"Studio found no signer extension in this browser."` | `account.js:53` |
| `method_required` | `"Add a new sign-in method first."` | `account.js:54` |
| `method_not_usable` | `"This method cannot sign in to the account. Use a different method."` | `account.js:55` |
| `passkey_no_prf` | `"This passkey manager cannot open the vault. Use iCloud Keychain or Google Password Manager, or a different method."` | `account.js:56` |
| `passkey_invalid` | `"The passkey was not accepted. Try again or use a different method."` | `account.js:57` |
| `passkey_cancelled` | `"The passkey request was cancelled or timed out. Try again."` | `account.js:58` |
| `passkey_unsupported` | `"This browser cannot give the passkey data that Studio needs. Use a different method."` | `account.js:59` |
| `passkey_exists` | `"This passkey is already in use. Use a different passkey manager or device."` | `account.js:60` |
| `last_method` | `"This is the last way to sign in to the account. Add a different method first."` | `account.js:61` |
| `invalid_name` | `"Give the passkey a name of 1 to 64 characters."` | `account.js:62` |
| `vault_locked` | `"Unlock the vault first."` | `account.js:63` |
| `unlock_required` | `"This change needs a new unlock. Unlock Studio or sign in again, then try again."` | `account.js:64` |
| Unknown | `"Something went wrong. Try again."` | `account.js:125` |

Session-required/session-ended API responses also switch the application to Sign in. See `app.js:145`, `app.js:5478`.

### Remote signer failures

| Code | Exact text | Evidence |
|---|---|---|
| `address_not_public` | `"The server cannot connect to this relay, because it is not a public address."` | `account.js:77` |
| `signer_timeout` | `"The signer gave no answer in time. Open the signer app and try again."` | `account.js:78` |
| `signer_unavailable` | `"Studio cannot reach the relay of the signer. Try again later."` | `account.js:79` |
| `bunker_invalid` | `"This is not a bunker address. It starts with bunker:// and names a relay."` | `account.js:80` |
| `signer_rejected` | `"The signer refused the request. Allow the request in the signer app and try again."` | `account.js:81` |
| `signer_invalid` | `"The signer gave an answer that Studio does not accept. Use a different signer."` | `account.js:82` |
| `connection_unknown` | `"The connection ended. Start a new connection."` | `account.js:83` |
| `connection_not_ready` | `"The signer is not connected yet."` | `account.js:84` |
| `connection_busy` | `"A sign-in with this signer is already in progress."` | `account.js:85` |
| `signer_not_connected` | `"This session has no connected signer. Sign in with the signer again."` | `account.js:86` |
| `no_nostr_wrapper` | `"The vault has no key for this signer."` | `account.js:87` |

The inline workspace signer can also produce `"The signer did not connect."`. See `signer.js:69`.

Inline actions after failure are `"Close"` and `"Try Again"`. See `signer.js:100`.

Write-signing failures include:

- `"Studio found no signer. This write needs your signer. Connect a signer first."`
- `"Your signer changed an event. Studio wrote nothing."`
- `"Your signer did not sign. Studio wrote nothing."`
- `"The plan of this write expired. Examine the changes again, then write."`
- `"The connected signer has a different key than this file host. Studio asked for no signature. Connect the signer of the file host."`
- `"No signer is connected. Connect a signer, then write."`

Evidence: `signing.js:90`.

Review offers `"Connect a Signer"` or `"Connect a Different Signer"` when needed. See `editor.js:2213`.

### Feed-discovery and MSP failure rows

These appear under `"Studio could not read:"` or `"Studio could not import:"`.

| Kind | Exact text | Evidence |
|---|---|---|
| `not_a_feed` | `"The address gives a page that is not a feed. The site may have moved the feed. Ask the owner of the site for its new address."` | `failures.js:9` |
| `gone` | `"The address gives no file. The site removed or moved the feed."` | `failures.js:10` |
| `server_error` | `"The site gave an error. Try again later."` | `failures.js:11` |
| `unreachable` | `"Studio could not connect to the site. Try again later."` | `failures.js:12` |
| `not_public` | `Studio on a server cannot connect to ${site || 'this address'}, because it is not a public address.` | `failures.js:13` |
| `too_large` | `"The file is too large to be a feed."` | `failures.js:14` |
| `invalid_address` | `"The address is not a web address. It must start with https:// or http://."` | `failures.js:15` |
| `unknown_guid` | `"Music Index does not know this feed."` | `failures.js:16` |
| `index_unreachable` | `"Studio could not get the address of this feed from Music Index. Try again later."` | `failures.js:17` |
| `bad_backup` | `"The file is not an MSP 2.0 backup."` | `failures.js:18` |
| Other | `"Studio could not read the feed."` | `failures.js:34` |

Rows do not contain their own retry buttons. The discovery sheet supplies `"Search Again"`.

### Account-import failures

`move.js:230` selects exact archive-cause text:

| Cause | Text |
|---|---|
| `unknown_format` | `"The file is not an account file of Studio, or it has a format that this Studio does not know."` |
| `newer_schema` | `"The file comes from a newer Studio. Update this Studio, then try again."` |
| `unsafe_path` | `"The file has an entry with an unsafe path. Studio refuses it."` |
| `link_entry` | `"The file has a link. An account file has none."` |
| `special_entry` | `"The file has an entry that is not a file or a folder."` |
| `unexpected_entry` | `"The file has an entry that an account file does not have."` |
| `duplicate_entry` | `"The file has the same entry two times."` |
| `missing_manifest` | `"The file has no manifest. It is not an account file of Studio."` |
| `invalid_manifest` | `"The manifest of the file cannot be read."` |
| `missing_file` | `"The file is not complete. Export the account again."` |
| `invalid_collection` | `"The collection in the file cannot be read."` |
| `too_many_entries` | `"The file has too many entries."` |
| `too_large` | `"The file is larger than this Studio accepts."` |
| `unreadable` | `"Studio cannot read the file. It is damaged or it is not a tar file."` |
| `no_portable_method` | `"The vault in the file has only passkeys. Make a recovery key on the old server, and export again."` |

Evidence: `move.js:210`.

Other import messages:

- `"This server has an account with this email address or this key. Sign in, or use a different server."`
- `"The file is larger than the space of a new account on this server."`
- `The file is empty. Select the file that "Export Account" made.`
- `"Too many imports. Try again later."`
- `"The server did not get the whole file. Select the file again."`
- `"Give an email address for the new account."`
- `"Give an email address."`
- `"This signer has another key than the file."`
- `"The vault in the file has no key for this signer."`
- `"This collection has feeds. Import into a new collection or a new data folder."`
- `"The library folder of this collection has files. Import into a new collection or a new data folder."`
- `"The passphrase or the key does not open the vault of the file."`
- `"Sign-up needs an invite code that nobody used."`

Evidence: `move.js:240`.

Unknown archive cause:

> `Studio cannot use this file. Select a file that "Export Account" made.`

Evidence: `move.js:235`.

Drop validation:

- `"Drop one account file only"`.
- `"Select a .tar account file"`.

Evidence: `filedrop.js:12`.

The file-selection form remains available after upload/check failure. The open-vault form remains after credential failure. See `app.js:5258`, `app.js:5350`.

### Build-date notices encountered in Review

These do not stop the write.

| Condition | Heading and body | Evidence |
|---|---|---|
| `clock_ahead_seconds != null` | `"The clock of this computer is ahead."` / `"Studio uses the time of a server for the build date."` | `builddate.js:10` |
| `no_server` | `"The build date does not change."` / `"Studio could not check the time with a server. A later write sets the date."` | `builddate.js:6` |
| `later_base != null` | `"The build date of this feed is in the future."` / `"An index refuses an earlier date, so Studio keeps this date. The date is:"`, then value | `builddate.js:8` |

Renderer: `editor.js:2183`.

## 15. Page, sheet, dialog, and route register

### Pages and shell views

| Surface | Renderer | Show condition / entry | Return |
|---|---|---|---|
| Loading | `index.html:17`, `app.js:5490` | Initial load or pending state | Automatic |
| Startup error card | `app.js:5486`, `app.js:5381` | Session/status/library error | `"Try Again"` |
| No-token page | `app.js:1820` | Missing/invalid local access | User opens full startup URL |
| Local unlock | `app.js:1799` | Locked local vault | Successful Unlock |
| Library setup/result | `app.js:1760` | `library.root === null` | `"Continue"` after setup |
| Wide shell/sidebar | `app.js:4016`, `app.js:3135` | Width above 760 px | Destination/history controls |
| Narrow root | `app.js:3346` | `view.name === 'root'` | Destination selection |
| Narrow hierarchy level | `app.js:3251` | Narrow, `panel.kind === 'level'` | Back or Destinations |
| Narrow feed outline | `app.js:3359` | Selected feed and `view.name === 'outline'` | Back, Destinations, or feed head |
| Releases/group list | `app.js:3408` | Default or releases view | Sidebar/root/history |
| Welcome | `app.js:3501` | No feeds in ungrouped Releases | Starts a sheet or another destination |
| Add feeds | `app.js:2953` | `view.name === 'add-feed'` | Back, Releases, or successful add |
| File hosts | `app.js:2755` | `view.name === 'hosts'` | Back or destination |
| Settings | `app.js:2820` | `view.name === 'settings'` | Back/destination, with pending-change guard |
| Unsupported feed facts | `app.js:2001` | Feed fails `isEditable` | Back/destination |
| Host change-set review | `app.js:3014` | Move creates `{name:'change-set', area:'hosts'}` | Back, destination, or Discard |
| Sign in | `app.js:4259` | Server signed out | Signup/recovery/sign-in |
| Sign up | `app.js:4407` | SignedOut page state | `"I Have an Account"` |
| Recovery | `app.js:4306` | Recovery entry | `"Back to Sign In"` |
| Add sign-in method | `app.js:4339` | Recovery-limited session | Save method or Sign Out |
| Remote signer auth | `app.js:4101` | Remote sign-in/signup/import branch | Stop/Back |
| Server account import | `app.js:5199`, `app.js:5221` | Signup import branch | Back to Sign Up/Cancel |
| Live Shows | `editor.js:3491` | Music feed `page:'lives'` | Feed tabs/history |
| Live show | `editor.js:3506` | Valid show key and `page:'live'` | Live Shows tab/history |

People view names are dispatched at `app.js:3976`. Their journeys are outside this run.

### Sheets and inline replacements

| Surface | Renderer | Condition / entry | Close or return |
|---|---|---|---|
| Your artist/label page | `app.js:3547` | Welcome music/label row | Close, Escape, Cancel, Back |
| Add-feed row sheet | `app.js:2921` | Add feeds row or first-step action | Close/Escape |
| New-feed form | `app.js:1414` | Make row inside add sheet | Parent sheet close |
| Feed discovery/results | `app.js:1587`, `app.js:1650` | Find row | Search Again or parent close |
| MSP import | `app.js:1692` | MSP row | Continue or parent close |
| Protect your keys | `app.js:709` | `vault_required` | Replaces parent form until completed |
| Recovery-key display | `app.js:745` | New recovery key | Checkbox then Continue |
| Unlock Studio sheet | `app.js:4488` | Explicit or automatic unlock | Close, Escape, Cancel |
| Add-host form | `app.js:1387` | Empty host list or add action | Page Cancel where available |
| SFTP fingerprint step | `app.js:998` | Successful fingerprint read | Back or Accept |
| Inline workspace signer | `signer.js:12` | Host/review signer connection | Stop/Close; Try Again after failure |
| Release artist picker | `editor.js:2602` | Label release selection | Close/Escape |
| Move all releases | `editor.js:4385` | Host card action | Cancel, Close, Escape |
| Export | `app.js:4672` | Export action | Cancel/Close/Escape |
| Local account import | `app.js:5199` | Welcome import | Cancel/Close/Escape |
| New/copy/import collection | `app.js:4982` | Collection action | Close/Escape |
| Rename/replace/delete collection | `app.js:5021` | Collection row action | Close/Escape; destructive forms also Cancel |
| General confirmation | `app.js:5410` | `askConfirm` | Safe button, Close, Escape |
| Add live show | `editor.js:3431` | Add Live Show | Cancel/Close/Escape |
| Review changes | `editor.js:1958` | Review or live status action | Close/Escape, successful write, discard |
| Recording/image file picker | `editor.js:1059` | Live-show Choose control | Cancel/Close/Escape or selection |

Native desktop dialogs are registered in section 13.

### Routes

There is one application HTML route, `/`. Shell views and authentication page choices are JavaScript state, not separate URL paths.

Evidence: `api/lib.rs:118`, `app.js:3716`, `app.js:4468`.

| Route group | Purpose and source |
|---|---|
| `/`, `/ui/{*path}`, `/manifest.webmanifest` | HTML, static UI files, installation manifest. `api/lib.rs:118` |
| `/?sort=az&view=tiles&reversed=1` | Optional list-state parameters. Any subset can appear. `shell.js:715`, `shell.js:726` |
| `/#token=${...}` | Local token intake. `app.js:65` |
| `/auth/v1/session` | Determines local/server, signed-in, recovery, and collection mode. `app.js:5459` |
| `/auth/v1/signin/email`, `/signup/email`, `/strength` | Email sign-in, signup, passphrase strength. `server/auth.rs:37` |
| `/auth/v1/challenge`, `/nostr` | Extension authentication. `server/auth.rs:40` |
| `/auth/v1/nip46/connect`, `/nip46/${connection}`, `/nip46/${connection}/signin`, `/nip46/unlock` | Remote signer state/authentication/unlock. `server/auth.rs:42`, `app.js:4083` |
| `/auth/v1/recovery` | Recovery sign-in. `server/auth.rs:49` |
| `/auth/v1/passkey/options`, `/passkey`, `/passkeys/options`, `/passkeys`, `/passkeys/${credential_id}` | Passkey authentication and management. `server/auth.rs:50` |
| `/auth/v1/signout`, `/account/delete`, `/import-notice/close` | Account actions. `server/auth.rs:73`, `app.js:4617` |
| `/auth/v1/import…` | Signed-out server account import. `server/auth.rs:58`, `app.js:5145` |
| `/api/v1/status`, `/library`, `/library/limits`, `/library/scan` | Boot, library setup, limits, outside-file scan. `app.js:5366`, `app.js:1767`, `app.js:2866`, `app.js:3916` |
| `/api/v1/vault…` | Creation, strength, lock/unlock, wrappers, passphrase, recovery key. `api/lib.rs:236` |
| `/api/v1/connectors…` | Host creation/edit/removal, fingerprint, listing, site check, matching. `app.js:772`, `app.js:929`, `app.js:2616`, `app.js:2776` |
| `/api/v1/search`, `/discover`, `/feeds/import`, `/import/msp` | Feed discovery/import. `api/lib.rs:286`, `api/lib.rs:293`, `api/lib.rs:397` |
| `/api/v1/feeds`, `/feeds/new`, `/feeds/${id}/refresh` | Collection/feed creation and refresh. `api/lib.rs:399` |
| `/api/v1/settings/lock`, `/settings/outside-check`, `/settings/nostr`, `/settings/announcements`, `/window-close` | Settings. `app.js:2860`, `editor.js:1831`, `app.js:4788` |
| `/api/v1/apps`, `/apps/${id}` | App pairing/list/removal. `app.js:2459`, `app.js:2484` |
| `/api/v1/signer`, `/signer/connect`, `/signer/sign` | Workspace signer. `signer.js:33`, `signing.js:48` |
| `/api/v1/collections…`, `/collection-uploads…` | Collection management and replacement. `app.js:4998`, `app.js:5032`, `app.js:5188` |
| `/api/v1/account/export-facts`, `/account/export`, `/import…` | Local/account export and import. `app.js:4680`, `app.js:5145` |
| `/api/v1/change-sets/move`, `/change-sets/${id}` | Host bulk move and its review. `editor.js:4394`, `app.js:3025` |
| `/api/v1/feeds/${id}/outside…` | Outside-file checks and responses. `app.js:4818`, `app.js:4868` |
| `/api/v1/feeds/${id}/announcements` | Announcement reads and send/resend. `editor.js:2319`, `app.js:4873` |
| `/api/v1/feeds/${id}/live-items`, `/live-items/${guid}/status` | Live-show creation/status. `editor.js:5061`, `editor.js:3634` |
| `/api/v1/feeds/${id}/grants…` | Grant request, list, and end. `editor.js:1878`, `editor.js:3609`, `editor.js:3626` |
| `/api/v1/feeds/${id}/diff`, `/write` | Review and publication for included live/grant/move paths. `editor.js:1975`, `editor.js:2040` |
| `/api/v1/stream/type`, `/stream/check` | Stream type suggestion/check. `editor.js:3554`, `editor.js:3577` |
| `/media/download?ticket=${encodeURIComponent(ticket)}` | Download to device. `app.js:188` |
| `/app/v1/pair`, `/whoami`, `/shows`, `/shows/${grant_id}/…` | Connected-app protocol, not browser pages. `api/lib.rs:564` |

The manifest declares `"display": "standalone"`, `"start_url": "/"`, and `"scope": "/"`. See `manifest.webmanifest:4`.

**UNSURE:** Browser installation controls and their labels are supplied by the browser. No installation button was found in the inspected shell renderer.

## 16. Controls not used by the main paths above

These controls are visible in scope but are optional, supporting, or lead across the run-A boundary.

| Control | Behavior | Evidence |
|---|---|---|
| `${shown ? 'Hide' : 'Show'} ${name}` | Reveals or hides each secret field/display | `app.js:557` |
| `"Copy the feed address"` | Copies the feed URL; two-second Copied status | `app.js:1951`, `app.js:1927` |
| `${url}` feed-address link | Opens the feed URL in a new browser context or desktop opener | `app.js:1950` |
| `${written.length} ${written.length === 1 ? 'feed' : 'feeds'}` disclosure | Opens a host’s feed list | `app.js:2713` |
| `${f.title || f.path}` in a host feed list | Opens that feed directly | `app.js:2716` |
| `${f.title || f.path}` under `"Different addresses"` | Opens the differing-address feed | `app.js:2721` |
| `"Read Again"` on unsupported feed facts | Refreshes if a connector or public URL exists | `app.js:2026` |
| `"Remove from Collection"` on unsupported feed facts | Confirmation, then removes collection entry | `app.js:2025` |
| `"Open the Feed"` in a publisher list | Opens its own feed editor | `app.js:3446` |
| `${tile.title}` publisher tile | Opens its own feed editor | `app.js:3488` |
| `"Search Again"` | Clears discovery results while keeping the sheet | `app.js:2935` |
| `"Cancel"` in connected-app name form | Stops the adding form | `app.js:2563` |
| `"Actions ⌄"` | Toggles collection action menu; narrow style can show ellipsis | `app.js:5103`, `menu.js:27` |
| `"Close"` on imported-account notice | Clears the retained import notice | `app.js:4640` |
| Feed-outline `${item.label}` track row | Opens the corresponding track editor | `app.js:3312` |
| Feed-outline `"Add tracks"` | Opens album page and import sheet | `shell.js:213`, `app.js:3303` |
| `"Versions"` in the feed toolbar | Opens versions sheet; feed-editor boundary | `editor.js:2300` |
| `Artwork of ${feed.title || 'Untitled feed'}` image button | Opens artwork sheet; feed-editor boundary | `app.js:2278` |
| Live `"Image address"` → `"Choose…"` | Opens shared image picker | `editor.js:3682`, `editor.js:1132` |
| Live `"Other…"` stream type | Reveals the custom-type input | `editor.js:3699` |
| `Use ${check.typeWarning.fillLabel}` | Applies the checked server type | `editor.js:3713` |
| Live `"Remove the Live Value"` | Clears the draft live-value object immediately | `editor.js:3745` |
| Live `"Identity"` disclosure | Shows Show GUID | `editor.js:3755` |
| Recording `"Choose…"` | Opens shared audio picker | `editor.js:3728` |
| Live value-route controls | Shared `TrackPayments` editor, outside detailed run-A coverage | `editor.js:3750` |
| Change-set feed summary `${feed.title || 'Untitled feed'}` | Opens that feed’s diff; one disclosure stays open | `app.js:3113` |
| Review `"Examine the Changes"` | Rebuilds the displayed diff when absent/stale | `editor.js:2218` |
| Review `"Load the Draft Again"` | Replaces stale window draft state | `editor.js:2141` |
| Review `"Apply My Edits to the New File"` | Starts merge review; feed-editor boundary | `editor.js:2157` |
| Review `"Discard the Draft"` | Opens discard confirmation | `editor.js:2226` |

`nostrkey.js` contains display/model helpers. It does not render a separate shell page or control. Its picker use belongs to feed editing and People.

`builddate.js`, `announce.js`, `live.js`, `settingsdraft.js`, `collectionlist.js`, `theme.js`, and `uiscale.js` also contain model or startup logic. Their visible surfaces are mapped to renderers above.

**UNSURE — runtime-dependent text:** API error bodies, returned search-field rows, stream-type rows, native dialogs, and external signer/passkey steps cannot have one fixed literal inventory. Their renderer expressions and known message catalogs are recorded above.