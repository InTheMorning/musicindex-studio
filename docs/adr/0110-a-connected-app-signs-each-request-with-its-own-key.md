# ADR 0110: A Connected App Signs Each Request With Its Own Key

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the mockups in `.studio-dev/mockup-110/site/grant.js`, screens 6 to 8.
- Amended: 2026-11-01 by the operator, from the security review of the plan on the same day. The amendment adds items 3, 4, 6, 7, 9, 12, 13, 15, 19, 20, 24, 26 and 29. It changes items 2, 5, 8, 10, 11, 16, 18, 27 and 28. Item 17 replaces the old section "What Does Not Change". The other items have new numbers only, and SEC-09 closes.
- Changes: [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 9 (the token of a connected app) and item 12 (the apps of a grant). [ADR 0058](0058-grant-vaults-and-connected-apps-on-the-server.md) items 4, 5, 8, 9, 11, 15, 17, 18, 19 and 20 (the app key, the wrappers, a new app, a removed app, the granted write, the apps from before, the routes, the index, the limits and the text of ADR 0003 item 4). [ADR 0107](0107-a-user-can-export-and-import-an-account.md) items 2 and 14 (the index of app tokens and the apps after an import).
- Uses: [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) item 6 (a signed event of kind 27235), [ADR 0013](0013-vault-format-and-cryptography.md) (the vault format), ADR 0106 (a recent unlock, written by [task 069](../tasks/studio-v1-task-069-account-protection.md)) and the [research note on host secrets under a show grant](../research/2026-11-01-connected-app-host-secrets.md).

## Context

- An app connects with a token now. Studio makes the token, shows it one time, and the user copies it into the app. Studio keeps the SHA-256 of the token and a public key that it derives from the token. The app sends the token in the `Authorization: Bearer` header of each request (ADR 0058 item 4).
- The copy of the token by the user is the weakest point. A clipboard history, a screenshot, a chat message or a shell history can keep the token. A copied request also holds the token, and the token works until the user removes the app.
- A grant vault holds the full secret of the host of the feed XML. An app token, together with a copy of the server disk, gives that secret (research note, "What An Attacker Gets").
- On 2026-11-01, the operator said that `musicindex-live-publisher` is the only app that is expected to connect. The operator also said that the live value changes during a show. The publisher thus uses its grant many times in each show. The publisher has no Studio client yet.
- The operator asked how to decrease the chance that the token leaks, and selected key pairs on 2026-11-01. The operator also selected "Last used" for each app, and the confirmation of a key by the first signed request.
- A security review of this plan on 2026-11-01 found these gaps:
  - An app key continues to work after the session ends and after the vault locks. But a stolen session could add an app key with no new proof, and the key then got a wrapper for each grant (`sync_one_grant` in `crates/api/src/workspace.rs`). Task 069 already asks for a recent unlock before the add of each other credential.
  - Studio did not prove that the app holds the private key of the key that the user pasted. On a server, an account could also claim the key of the app of a different account first.
  - The migration kept the public key of each token app. Each holder of an old token can derive the private key of that public key with HKDF.
  - A granted app could set `podcast:liveValue` to an address of each scheme. Listener apps read the live value. Thus a false address can send the boosts of the show to a different party.
  - These parts had no rule: the replay store, the `u` and `payload` tags, the form of a key, the logs and an import.

## Decision

### The Key Of An App

1. Each connected app has its own secp256k1 key pair. The app makes it and keeps the private key. Studio never makes, sees, shows or keeps the private key of an app.
2. "Connect an App" asks for a name and the public key of the app (`npub` or hexadecimal). Studio keeps the name and the public key as 64 lowercase hexadecimal characters. It keeps no token and no hash of a token. On a server, "Connect an App" needs a recent unlock (ADR 0106).
3. Studio refuses an app key that is the Nostr sign-in key of an account on the same server, with 400 `app_key_is_sign_in_key`.
4. At "Connect an App", Studio shows a confirmation code: 8 characters of Crockford base32, from a random source. Studio keeps only the SHA-256 of the code. The code is not a host secret, and it expires after 15 minutes.
5. A new app is "Waiting for the app" until the app sends `POST /app/v1/confirm` with the code in the signed body. Then the app is active. A waiting app has no wrapper on a grant vault. It reaches only `GET /app/v1/whoami` and `POST /app/v1/confirm`.
6. Studio removes a waiting app when its code expires, or after 5 wrong codes. The user then connects the app again.
7. Studio and the app each show the key in the same short form: the first 9 and the last 4 characters of the `npub`, for example `npub1lp7w…9k2q`. The user compares the two before the user gives the code to the app.

### One Key On A Server

8. On a server, one public key is active in one account only. A waiting app does not reserve its key. Studio finds the waiting app of a confirmation from the key and the code.
9. When an app becomes active, Studio removes each waiting app of the same key in the other accounts. A second account that adds a key that is active in a different account gets 409 `app_key_in_use`. This tells the second account only that an account uses the key. The operator accepts that disclosure.

### The Signed Request

10. Each `/app/v1/` request has the header `Authorization: Nostr <base64 event>`. The event has kind 27235 and these tags. This is the form of NIP-98.
    - `u`: the full address of the request. Studio makes the expected address from the public origin of its configuration, not from the `Host` header.
    - `method`: the HTTP method.
    - `payload`: the SHA-256 of the raw body bytes. Each request with a body has this tag.
11. Studio refuses the request with 401 `app_signature_invalid` in each of these conditions:
    - The signature is not correct.
    - The public key is not the key of a waiting or active app of this workspace.
    - The `created_at` of the event differs by more than 60 seconds from the time of the server.
    - The `u` or `method` tag is not the same as the request, character for character.
    - The request has a body and the event has no `payload` tag, or the `payload` tag is not the SHA-256 of the body.
    - Studio accepted the same event ID in the last 120 seconds.
12. Studio refuses a body larger than 64 KiB before it calculates the hash. Studio calculates the hash before it reads the JSON.
13. The workspace keeps the accepted event IDs for 120 seconds in its collection, not in memory, so that a restart does not clear them. Studio records an event ID before the route runs. A request that fails after that point uses up its event, and the app signs a new event.
14. `/app/v1/` accepts no other credential. `/api/` and `/auth/` refuse the header of item 10.
15. No log holds a request body, an `Authorization` header, a signed event, a confirmation code or a grant vault key.
16. Studio records the time and the client address of the last accepted request of each app. Connected apps shows them: "Last used today at 21:02 from 203.0.113.7". Only the user of the account sees them.

### The Grant Vault

17. A grant names the apps that are active when the user approves its diff. A grant applies only to the apps that it names. An app that becomes active later gets no wrapper on that grant. The review sheet names the apps after the grant check box. When no app is active, the check box is not available, and the sheet says "Connect an app first".
18. A grant vault has one wrapper for each active app that the grant names: the grant vault key, encrypted with NIP-44 version 2 to the public key of the app. This is the method of ADR 0058 item 5, with the key of the app in place of a key from a token.
19. When the user removes an app, Studio removes its wrapper from each grant vault at once. At the next time that the user vault is open, Studio makes each grant vault of that app again with a new grant vault key (ADR 0058 item 7). The old key then opens no current grant vault.
20. The grant vault files have mode 0600 (task 068).
21. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that signed the request.
22. For a granted write, the app decrypts its wrapper with its private key. It sends the grant vault key in the body of the signed request, so that the `payload` tag covers it. Studio opens the grant vault, writes through `pipeline::write_draft`, and clears the keys from memory. Studio keeps no copy of the grant vault key outside the user vault.

### The Live Value

23. The four operations of ADR 0054 item 10 do not change.
24. The core refuses a granted live value with a `uri` that does not start with `https://` or `wss://`, or that has more than 2048 characters. It refuses a `protocol` that is not 1 to 32 characters of letters, digits, `.` and `-`. The error is 400 `live_value_invalid`.

### Apps From Before This ADR

25. An app with a token, from before this ADR, shows "Connect Again". `/app/v1/` refuses its token with 401 `app_key_required`. A migration removes the token hashes, and each wrapper of a token key goes out of each grant vault.
26. The migration also clears the public key of each token app. A token app thus has no key that Studio accepts. Studio accepts a signed request only from a waiting or active app with a key.

### Limits, The Index And An Import

27. `server.sqlite` keeps an index from each public key to its account and its state. The server makes the index again from the collections at each start, in place of the index of token hashes (ADR 0058 item 18, ADR 0107 item 2).
28. The server permits 60 requests for each app public key in each minute. It permits 20 requests with an unknown key or a signature that is not correct, for each client address in each hour. After that, it gives 429 `too_many_attempts`.
29. An import of an account (ADR 0107) keeps the name and the key of each app, and sets each app to "Connect Again". The import keeps no grant. The account page lists the apps with this notice, in place of the notice of ADR 0107 item 14.

## Alternatives Considered

- Keep the token, and add "Last used" and "Make a New Token". Not chosen by the operator. The user still copies a secret by hand.
- Credentials for one file, made for each grant. Not chosen by the operator now. Only R2 and AWS S3 can make them, and the operator judged them a lot of work (research note, options 3 and 2).
- An app key that Studio makes and shows one time. Rejected: the transfer of the private key is the same weak point as the token.
- NIP-46 between Studio and the app. Rejected: the app must sign many requests with no person present, and a relay adds a dependency to each write.
- A grant that applies to each active app (the first text of this ADR). Replaced by the amendment: a key that a stolen session adds must not get a grant that the user approved for a different app.
- A separate event kind for the app requests. Not chosen: the `u` tag already separates a sign-in from an app request, and item 3 refuses a sign-in key as an app key.
- A live value address only from a list that the user keeps. Not chosen now: the live publisher sets its own address during a show, and no list of the user exists. The operator can examine this again after the human test.
- One key exchange for each grant session, so that the grant vault key is not in each write body. Not chosen now: Studio would keep a key in memory between requests. Item 15 keeps the key out of each log.

## Consequences

- No secret goes from Studio to the app, or from the app to the user, at the connection. The confirmation code is valid for a short time and for one waiting key only.
- A stolen session cannot add an app without a recent unlock. A key that the app does not hold cannot become active.
- A new app gets no grant that exists. To give a show to a new app, the user approves a diff with a new grant.
- A copied request stops working after 60 seconds, and Studio refuses a repeated event, also after a restart.
- A stolen private key file of the app still works until the user removes the app. It can set the live value to an `https` or `wss` address of the attacker during a granted show. "Last used" and the list of granted writes of ADR 0054 item 17 show its use.
- The live publisher needs a client: a key pair, the confirmation code, the signed requests and the decryption of a wrapper. That work belongs to its own repository. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface.
- The mockups have no confirmation code and no app line in the grant sheet. Task 073 adds these screens to the mockups before its UI packet.
- Task 073 gives the work in Studio. It comes after task 068 and task 069.

## Acceptance Criteria

Mechanical:

- An API test: a correct signed request succeeds. Each condition of item 11 gives 401 `app_signature_invalid`.
- An API test: a repeated event in 120 seconds is refused, also after the workspace loads again.
- An API test: a request with a body and no `payload` tag is refused.
- An API test: a waiting app becomes active after `POST /app/v1/confirm` with the correct code, and has no wrapper before that. An expired code and a sixth wrong code each remove the waiting app.
- An API test: an `npub`, an uppercase hexadecimal key and a lowercase hexadecimal key of one key give one app.
- An API test: a granted write with the grant vault key in the signed body succeeds. A wrong key gives 423 `vault_locked`, as before.
- An API test: an app that becomes active after a grant gets no wrapper on that grant, and its granted write is refused.
- An API test: after the removal of an app and an unlock, its old grant vault key opens no grant vault.
- An API test: a granted live value with an `http` address or a wrong protocol gives 400 `live_value_invalid`.
- An API test: a granted write and a refused request put no body, signed event or grant vault key in the log.
- A server test: "Connect an App" with no recent unlock gives 401 `unlock_required`.
- A server test: a key that is active in a different account gives 409 `app_key_in_use`. A waiting key in a different account does not block the add.
- A server test: a sign-in key of an account cannot be an app key.
- A server test: an app with a token gets `app_key_required`. After the migration, a request signed with the key that HKDF derives from the old token gets 401.
- A server test: after an import, each app is "Connect Again" and the account has no grant.
- A Node test: the Connected apps model gives the lines of screens 6 to 8 and the line with the confirmation code.

Visual: screens 6 to 8 of the mockups and the new screens of the code and the grant sheet, at 1100 px and at 390 px with touch, in the two themes.

Human: the operator connects the live publisher with its public key and the code, runs a show with a grant, and sees "Last used" change.
