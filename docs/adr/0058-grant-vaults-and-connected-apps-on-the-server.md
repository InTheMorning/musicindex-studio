# ADR 0058: Grant Vaults And Connected Apps On The Server

- Status: Accepted
- Date: 2026-10-26
- Owner: operator

## Context

[ADR 0054](0054-live-items-connected-apps-and-show-grants.md) lets a connected app such as v4vmm start and end a live show and set its live value under a show grant. A granted write needs the secret of a host. ADR 0054 item 16 refuses a granted change while the vault is locked. Its consequences leave the connected apps of a server user to the Server Shell phase.

[ADR 0003](0003-user-unlocked-secret-vault.md) item 4 says: "The server can write to a host only while the user's session is open. The server has no key that opens a vault without the user." On the server, the vault key exists only in the vault window of [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md).

A show can start hours after the user prepares it. The research of 2026-10-26 ([Phase 12 authentication research](../research/2026-10-26-phase-12-auth-research.md)) examined these methods:

- Refuse the write while the vault is locked. A show can then fail at its start.
- Presigned operations at the grant time. They work for S3 and R2. For nsite and Blossom, each signed event names the hash of one file, so a live value from the app cannot be signed in advance. SFTP has no presigned form.
- A time-limited unlock: the server keeps the secrets in memory for the grant period. It is the weakest method in a breach.

The planning model added one more method: a grant vault that only the token of an app opens.

On 2026-10-26 the operator selected the decisions below.

## Decision

### The Grant Vault

1. A grant vault is a small vault with the format of [ADR 0013](0013-vault-format-and-cryptography.md), with its own random vault key. Each show grant has one grant vault: `grants/<grant ID>.json` in the data folder of the user.
2. The grant vault holds copies of the secrets of the host of the feed XML only. For an nsite host with "Key in the vault", this includes the key of the site. It holds no secret of another host.
3. The user vault keeps the key of each grant vault as a record. Thus Studio can change a grant vault while the user vault is open.
4. Each connected app has an app key: a secp256k1 key that Studio derives from the app token with HKDF-SHA-256. When the user adds an app, Studio derives the key and keeps only its public key, with the hash of the token.
5. A grant vault has one wrapper for each connected app: the grant vault key, encrypted to the public app key with NIP-44 version 2. Studio can add this wrapper with no app token.

### When Studio Changes A Grant Vault

6. The approved write that records a grant ([ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 12) makes the grant vault. The user vault is open at that time when the host needs a secret.
7. Studio makes each grant vault of a feed again in two conditions, while the user vault is open:
   - The user changes the secret of the host of the feed XML.
   - The user moves the feed XML to a different host.
8. When the user adds an app, Studio adds its wrapper to each active grant vault at the next time that the user vault is open.
9. When the user removes an app, Studio removes its wrapper from each grant vault.
10. When a grant ends (ADR 0054 item 14), Studio deletes its grant vault and its record in the user vault. Studio checks the end time before each granted write, at the start, and each hour.

### The Granted Write

11. Each granted write uses the grant vault, in both shells. Studio derives the app key from the token of the request and opens the grant vault. It makes the host of the feed XML and writes through `pipeline::write_draft`. Then it clears the keys from memory.
12. This supersedes ADR 0054 item 16 for a locked vault. A granted write works while the user vault is locked. The refusals for a draft with changes and for an ended grant do not change.
13. A host that needs a secret that the grant vault does not hold gives 423 `vault_locked`.
14. A host of the feed XML with "My signer" cannot do a granted write, because no person is present to sign. The review sheet disables the grant check box for that feed and tells the user the reason.
15. An app from before this ADR has no app key. Studio shows "Connect again" for it, and does not use it for a grant.

### Connected Apps On The Server

16. The server serves `/app/v1/` on the public origin. This supersedes the loopback `Host` of ADR 0054 item 9 for the server. The server applies the `Host` rule of ADR 0055 item 19. The desktop does not change.
17. `/app/v1/` accepts only an app token. It does not accept the session cookie. `/api/` does not accept an app token.
18. `server.sqlite` keeps an index from the hash of each app token to its user. The collection of the user keeps the app as on the desktop.
19. The server permits 60 requests for each app in each minute, and 20 unknown tokens for each client address in each hour. After that, it gives 429 `too_many_attempts`.

### The Change To ADR 0003

20. This supersedes ADR 0003 item 4 in part. The item becomes: "The server can write to a host only while the user's session is open, or under a show grant with the token of a connected app of the user. The server has no key that opens a vault without the user or that token."

## Alternatives Considered

- Refuse a granted write while the vault is locked. Not chosen by the operator. A show can fail at its start.
- Presigned operations. Not chosen by the operator. They do not cover SFTP or a live value on nsite and Blossom.
- A time-limited unlock in server memory. Not chosen by the operator. Each breach of a running server then exposes the secrets of each active grant.
- A wrapper with a key from the token itself. Rejected: Studio keeps only the hash of the token, so it cannot add a wrapper for a new grant without the token.
- Each host of the feed in the grant vault. Not chosen by the operator. A granted change writes only the feed XML.
- Connected apps on the server not in Phase 12. Not chosen by the operator.

## Consequences

- A copy of the host secret of the feed XML is on the disk of the server while a grant is active. The disk alone or the token alone opens nothing. The disk and the token together give that host secret, not only the change of one live item.
- On the desktop, the token is in the other app on the same computer. There the grant vault gives almost no protection against a person who has the computer. It permits a granted write while the vault is locked.
- v4vmm and musicindex-live-publisher can use a server Studio with the public address in place of the loopback address. The contract of ADR 0054 does not otherwise change.
- The secp256k1 code and NIP-44 of the core ([ADR 0051](0051-the-core-signs-nostr-events-with-secp256k1.md), [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md)) make the app wrapper. No new crate is necessary for it.

## Verification

Mechanical:

- A test makes a grant, locks the user vault, and does "Go live", a live value and "End the show" with the app token. The hosted feed has each change.
- A test confirms that the grant vault does not open with a different app token. It holds only the secret of the host of the feed XML.
- After the end of the grant, a test finds no grant vault file. The removal of an app removes its wrapper.
- A test confirms that an app that the user adds during a grant can write after the next unlock, and gets 423 before it.
- A server test does a granted write through the public origin. It refuses the session cookie on `/app/v1/` and an app token on `/api/`.
- A test confirms that no file or database row contains an app token or a grant vault key as plain text.

Visual:

- The grant check box when it is disabled for "My signer", and the app list with "Connect again". Each one in the dark theme, in the light theme and at 390 px.

Human test:

- v4vmm or the live publisher does a test show against a server Studio, with no browser session open. This test waits for that repository.

## Invariants

- A granted write uses only the secrets of the grant vault.
- A grant vault exists only while its grant is active.
- A connected app never gets a secret of the vault.
