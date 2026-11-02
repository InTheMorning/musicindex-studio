# ADR 0110: A Connected App Signs Each Request With Its Own Key

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the mockups in `.studio-dev/mockup-110/site/grant.js`, screens 6 to 8.
- Changes: [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 9 (the token of a connected app). [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) items 4, 5, 11, 15, 17, 18 and 19 (the app key, the wrappers, the granted write, the apps from before, the routes, the index and the limits).
- Uses: [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) item 6 (a signed event of kind 27235), [ADR 0013](0013-vault-format-and-cryptography.md) (the vault format) and the [research note on host secrets under a show grant](../research/2026-11-01-connected-app-host-secrets.md).

## Context

- An app connects with a token now. Studio makes the token, shows it one time, and the user copies it into the app. Studio keeps the SHA-256 of the token and a public key that it derives from the token. The app sends the token in the `Authorization: Bearer` header of each request (ADR 0058 item 4).
- The copy of the token by the user is the weakest point. A clipboard history, a screenshot, a chat message or a shell history can keep the token. A copied request also holds the token, and the token works until the user removes the app.
- A grant vault holds the full secret of the host of the feed XML. An app token, together with a copy of the server disk, gives that secret (research note, "What An Attacker Gets").
- On 2026-11-01, the operator said that `musicindex-live-publisher` is the only app that is expected to connect. The operator also said that the live value changes during a show. The publisher thus uses its grant many times in each show. The publisher has no Studio client yet.
- The operator asked how to decrease the chance that the token leaks, and selected key pairs on 2026-11-01. The operator also selected "Last used" for each app, and the confirmation of a key by the first signed request.

## Decision

### The Key Of An App

1. Each connected app has its own secp256k1 key pair. The app makes it and keeps the private key. Studio never makes, sees, shows or keeps the private key of an app.
2. "Connect an App" asks for a name and the public key of the app (`npub` or hexadecimal). Studio keeps the name and the public key. It keeps no token and no hash of a token.
3. A new app is "Waiting for the app" until Studio gets the first request with a correct signature of that key. Then the app is active. A waiting app has no wrapper on a grant vault, and `/app/v1/` gives it only `GET /app/v1/whoami`.
4. On a server, one public key belongs to one account. A second account that adds the same key gets 409 `app_key_in_use`. An app uses a different key for each Studio account.

### The Signed Request

5. Each `/app/v1/` request has the header `Authorization: Nostr <base64 event>`. The event has kind 27235 and these tags. This is the form of NIP-98.
   - `u`: the full address of the request.
   - `method`: the HTTP method.
   - `payload`: the SHA-256 of the body, when the request has a body.
6. Studio refuses the request with 401 `app_signature_invalid` in each of these conditions:
   - The signature is not correct.
   - The public key is not the key of a connected app of this workspace.
   - The `created_at` of the event differs by more than 60 seconds from the time of the server.
   - The `u`, `method` or `payload` tag does not agree with the request.
   - Studio accepted the same event ID in the last 120 seconds.
7. `/app/v1/` accepts no other credential. `/api/` and `/auth/` refuse the header of item 5.
8. Studio records the time and the client address of the last accepted request of each app. Connected apps shows them: "Last used today at 21:02 from 203.0.113.7".

### The Grant Vault

9. A grant vault has one wrapper for each active app: the grant vault key, encrypted with NIP-44 version 2 to the public key of the app. This is the method of ADR 0058 item 5, with the key of the app in place of a key from a token.
10. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that signed the request.
11. For a granted write, the app decrypts its wrapper with its private key. It sends the grant vault key in the body of the signed request, so that the `payload` tag covers it. Studio opens the grant vault, writes through `pipeline::write_draft`, and clears the keys from memory. Studio keeps no copy of the grant vault key outside the user vault.

### Apps From Before This ADR

12. An app with a token, from before this ADR, shows "Connect Again". `/app/v1/` refuses its token with 401 `app_key_required`. A migration removes the token hashes, and each wrapper of a token key goes out of each grant vault.

### Limits And The Index

13. `server.sqlite` keeps an index from each app public key to its account, in place of the index of token hashes (ADR 0058 item 18).
14. The server permits 60 requests for each app in each minute. It permits 20 requests with an unknown key or a signature that is not correct, for each client address in each hour. After that, it gives 429 `too_many_attempts`.

### What Does Not Change

15. A grant still applies to each active app of the user (ADR 0054 item 12). The security audit item SEC-09 stays accepted while one program connects. Each instance of the program has its own key, and "Remove" disconnects only that instance.
16. The four operations of ADR 0054 item 10, the grant check box and the end of a grant do not change.

## Alternatives Considered

- Keep the token, and add "Last used" and "Make a New Token". Not chosen by the operator. The user still copies a secret by hand.
- Credentials for one file, made for each grant. Not chosen by the operator now. Only R2 and AWS S3 can make them, and the operator judged them a lot of work (research note, options 3 and 2).
- An app key that Studio makes and shows one time. Rejected: the transfer of the private key is the same weak point as the token.
- NIP-46 between Studio and the app. Rejected: the app must sign many requests with no person present, and a relay adds a dependency to each write.

## Consequences

- No secret goes from Studio to the app, or from the app to the user, at the connection.
- A copied request stops working after 60 seconds, and Studio refuses a repeated event.
- A stolen private key file of the app still works until the user removes the app. "Last used" can show its use.
- The live publisher needs a client: a key pair, the signed requests and the decryption of a wrapper. That work belongs to its own repository. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface.
- Task 073 gives the work in Studio.

## Acceptance Criteria

Mechanical:

- An API test: a correct signed request succeeds. Each condition of item 6 gives 401 `app_signature_invalid`.
- An API test: a repeated event in 120 seconds is refused.
- An API test: a waiting app becomes active after its first correct request, and has no wrapper before that.
- An API test: a granted write with the grant vault key in the signed body succeeds. A wrong key gives 423 `vault_locked`, as before.
- A server test: a second account cannot add a key that is in use.
- A server test: an app with a token gets `app_key_required`, and the migration removes its wrappers.
- A Node test: the Connected apps model gives the lines of screens 6 to 8.

Visual: screens 6 to 8 of the mockups, at 1100 px and at 390 px with touch, in the two themes.

Human: the operator connects the live publisher with its public key, runs a show with a grant, and sees "Last used" change.
