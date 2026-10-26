# ADR 0055: The Server Shell: User Folders, Accounts, Sessions And The Vault Window

- Status: Accepted
- Date: 2026-10-26
- Owner: operator

## Context

[ADR 0001](0001-one-core-with-desktop-and-server-shells.md) gives Studio a server shell. A host operator runs it as a public service, or an artist runs it as a personal server. `studio-server` prints its version only.

The goal of the operator for Phase 12: an artist changes a feed from a phone, away from home.

The API serves one user today. `Workspace` holds one collection, one vault and one library ([ADR 0014](0014-collection-in-sqlite.md), [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md)). `local_router` gives each route the shared `Arc<Mutex<Workspace>>`, and `access.rs` checks the token of [ADR 0017](0017-local-api-access.md).

Earlier records leave these items to the Server Shell phase:

- ADR 0014: the server keeps all collections in one database, with a user ID on each row.
- [ADR 0013](0013-vault-format-and-cryptography.md), open question: one vault file for each user, or rows in a database.
- ADR 0035 item 4: where the server keeps the library of each user.
- [ADR 0052](0052-announcements-through-the-podcast-index-hub.md): the server uses the same announcement queue for each user.

The research of 2026-10-26 ([Phase 12 authentication research](../research/2026-10-26-phase-12-auth-research.md)) examines the session on a phone. A phone browser often closes a tab in the background. A long session that keeps the vault open keeps the vault key in server memory for a long time.

On 2026-10-26 the operator selected the decisions below.

## Decision

### One Folder For Each User

1. The server keeps one data folder for each user: `<data>/users/<user folder>/`. The folder name is 16 random bytes as hexadecimal. It is not the email address or the public key.
2. Each user folder has the files of a desktop data folder: `collection.sqlite`, `vault.json`, `incoming/` and the library folder `library/`. This supersedes the ADR 0014 consequence "one database with a user ID on each row". It answers the open question of ADR 0013: one vault file for each user.
3. The library of a user is `library/` in the user folder (ADR 0035 item 4). The server sets it when it makes the user folder. The user cannot set a different library folder.
4. Each user has one `Workspace` with its own lock, its own download queue and its own announcement worker. The server opens a workspace at the first request of its user. At the start, it also opens each workspace with a waiting announcement or a queued download (ADR 0038 item 9, ADR 0052).
5. The `/api/v1/` routes and their handlers are the same in both shells. The API crate gives a router for one workspace with no access layer. The desktop adds the layer of ADR 0017. The server adds the session layer, finds the workspace of the session, and sends the request to the router of that workspace.

### The Accounts

6. The server keeps a separate database `<data>/server.sqlite`. It holds the accounts, the sign-in methods of each account, the sessions, the sign-in challenges, the invite codes and the index of app tokens ([ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md)). It holds no feed data and no secret.
7. The operator selects open sign-up or sign-up with an invite code. The default is an invite code. `studio-server invite` makes a single-use code. A personal server is a server with one account and no open sign-up.
8. The server sends no email. An email address is only the name of an account. The server keeps it in lowercase with no spaces at the start or the end. The server does not check that the user owns the address.
9. A user can delete the account. The server asks for an unlock of the vault first. Then it removes the user folder, the account rows and each session of the user. It does not change a feed on a host.

### Sessions

10. A sign-in gives a session. The session token is 32 random bytes. The server keeps only its SHA-256.
11. The browser keeps the token in the cookie `__Host-studio-session` with `HttpOnly`, `Secure`, `SameSite=Strict` and `Path=/`. The UI script cannot read it.
12. Each session also has a CSRF token: the SHA-256 of a fixed label and the session token. The sign-in response and `GET /auth/v1/session` give it to the UI. The UI sends it in the `X-Studio-CSRF` header with each `/api/` request. The server sends no CORS headers, so a page of a different site cannot send this header. A request without the correct header gets 403 `csrf_required`.
13. A session ends after 7 days with no request, after 30 days in total, at sign-out, and when the user deletes the account. The operator can change the two limits. The server keeps the sessions in `server.sqlite`, so a restart does not end them.

### The Vault Window

14. A session and an open vault are two different states. With a session and a locked vault, a user can read the collection and edit a draft. The user can also see a diff that needs no secret of a host.
15. The vault opens only through an unlock of the user: a sign-in with a passphrase, or an unlock with a method of [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md).
16. The server locks the vault of a user after 10 minutes with no use of a vault secret. It also locks the vault at "Lock Studio", at sign-out, at the end of the last session of the user, and when the process stops. The operator can change the limit of 10 minutes. A lock removes the vault key from memory ([ADR 0003](0003-user-unlocked-secret-vault.md), ADR 0013 item 15).
17. A route that needs a secret while the vault is locked gives 423 `vault_locked`, as on the desktop. The UI then asks for an unlock and sends the request again.

### Server Access

18. The server listens on an address that the operator gives. The default is `127.0.0.1:8080`, behind a reverse proxy with TLS. The operator gives the public origin, such as `https://studio.musicindex.org`. The origin must use `https`. Only a loopback origin can use `http`, for development and checks.
19. The server refuses each request with a `Host` that is not the host of the public origin, with 403. A request that changes data must also have an `Origin` header equal to the public origin.
20. The UI and its security headers are the same in both shells ([ADR 0016](0016-ui-without-a-build-step.md)). The UI asks `GET /auth/v1/session` first. The desktop answers `{"mode": "local"}`. The server answers with the mode `server` and the state of the session. The UI shows the sign-in page when the server has no session.

### Limits

21. Each user has a storage limit for the user folder. The default is 10 GiB, and the operator can change it. An upload or a download that goes over the limit stops with 507 `storage_full`.
22. The server runs a maximum of 2 Argon2id derivations at the same time. The operator can change the number. A sign-in that must wait more than 10 seconds gets 503 `busy`.
23. The server permits 10 failed sign-ins for each account in each hour, and 30 for each client address in each hour. After that, it gives 429 `too_many_attempts`. The client address comes from the header that the operator names for the reverse proxy.

### Functions Of The Desktop Only

24. The server refuses the local folder connector with 403 `not_on_server`. That connector reads and writes folders of the server computer.
25. The server refuses `PUT /library` with a folder other than the folder of item 3.

## Alternatives Considered

- One database with a user ID on each row (ADR 0014). Not chosen by the operator. Each query of the core must then filter on the user, and one missed filter shows the data of a different user.
- The session token in session storage, as on the desktop. Not chosen by the operator. A phone browser closes background tabs, and the user must then sign in again many times.
- A long session that keeps the vault open. Not chosen by the operator. The server would keep many vault keys in memory for hours.
- Email to confirm an address or to reset a passphrase. Not chosen by the operator. A reset cannot open the vault, and SMTP adds a dependency for each operator.

## Consequences

- The core does not change for the user folders. The isolation of users is in the folder structure.
- A user stays signed in on a phone for days, and opens the vault for each write session.
- The server keeps a vault key in memory only while the user writes, plus 10 minutes.
- A server restart locks each vault, but keeps each session.
- An email address can be in use by a person who does not own it. The server does not use the address for any message.
- A runbook gives the proxy, the options and the backup of `<data>/`. Task 037 writes it in `docs/runbooks/`.

## Verification

Mechanical:

- An API test signs in two users. A session of the first user cannot read or change a feed, a draft, a connector or a file of the second user.
- A test confirms that `/api/` refuses a request with no session, with a session of an ended account, and with no correct `X-Studio-CSRF` header.
- A test confirms that a `Host` other than the public host gets 403.
- With a short idle limit, a test confirms that the vault locks. A route that needs a secret then gives 423.
- A test confirms that `server.sqlite` contains no session token and no CSRF token.
- A test confirms the refusal of the local folder connector and of a different library folder.

Visual:

- The sign-in page, the sign-up page, the unlock sheet and the account page. Each one in the dark theme, in the light theme and at 390 px.

## Invariants

- A session can read and write only the folder of its own user.
- The server keeps no vault key after a lock, a sign-out or the idle limit.
- No session token or vault key is in a log, a database row or an API response.
