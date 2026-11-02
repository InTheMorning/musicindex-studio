# ADR 0110: A Connected App Signs Each Request With Its Own Key

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the mockups in `.studio-dev/mockup-110/site/grant.js`, screens 6 to 8.
- Amended: 2026-11-01 by the operator, from the security review of the plan on the same day. The mockups of screens 1 to 3, 6 to 6c and 7 to 11 give its screens. The amendment changes each section of the decision, and closes SEC-09. The app gets a pairing text from Studio, and a grant names one app and fixes its live value address. A shared client crate does the protocol for the app.
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
- After the review, the operator said that the protocol was too large for two of our own apps. The operator also said that the risk of a stolen key was too high. The operator selected a pairing text, a fixed live value address and a shared client crate. The operator asked for clear guides, so that each app uses the protocol correctly.

## Decision

### The Key Of An App

1. Each connected app has its own secp256k1 key pair. The app makes it and keeps the private key. Studio never makes, sees, shows or keeps the private key of an app.
2. The app selects how it keeps its private key, for example a file with mode 0600 or the key store of the operating system. Studio makes no rule for that storage.
3. Studio keeps the public key of an app as 64 lowercase hexadecimal characters. On a server, one public key is active in one account only.
4. Studio refuses an app key that is the Nostr sign-in key of an account on the same server, with 400 `app_key_is_sign_in_key`.

### The Pairing Text

5. "Connect an App" asks only for a name (screen 6). On a server, it needs a recent unlock (ADR 0106).
6. Studio then makes a waiting app and shows its pairing text one time (screen 6b). The pairing text is the public origin of Studio and a code, for example `https://studio.musicindex.org#pair=K7M2-Q9XD`. The code has 8 characters of Crockford base32 from a random source.
7. The pairing text is a secret value of ADR 0078. Studio keeps only the SHA-256 of the code. These rules decrease the risk of a screen share or a screenshot:
    - Studio hides the full pairing text until the user selects the eye button.
    - Copy puts the pairing text on the clipboard while it stays hidden, and the button then shows "Copied". Studio writes to the clipboard only when the user selects Copy.
    - The pairing text is not in the address bar, the page title, the waiting row, a notification or an error.
    - While the notice is open, Studio reads the state of the waiting app again every 2 seconds. When the app pairs, the notice shows "Connected", and the pairing text goes out of the page (screen 6c).
8. The code works one time, for 15 minutes. Studio removes the waiting app and its code in each of these conditions:
    - The 15 minutes end.
    - The user selects Done before the app pairs. The hint of screen 6b tells the user to select Done after the app says that it is connected.
    - The user selects "Remove" on the waiting row.
9. The code is not case-sensitive. Studio and the client crate ignore spaces and hyphens, read `O` as `0`, and read `I` and `L` as `1`, as Crockford base32 gives.
10. The user pastes the pairing text into the app. The app makes its key pair, and sends `POST /app/v1/pair`, signed with its new key. The body holds the code and the live value origin of the app (item 24).
11. Studio finds the waiting app from the SHA-256 of the code. It keeps the public key and the live value origin, and makes the app active. A code that no waiting app has gives 401 `app_code_invalid`. A code that an app used in the last 15 minutes gives 409 `app_code_used`. That error tells the user that a different program possibly used the pairing text, and the user must remove that app. A key that is active in a different account gives 409 `app_key_in_use`.

### The Signed Request

12. Each `/app/v1/` request has the header `Authorization: Nostr <base64 event>`. The event has kind 27235 and these tags. This is the form of NIP-98.
    - `u`: the full address of the request. Studio makes the expected address from the public origin of its configuration, not from the `Host` header.
    - `method`: the HTTP method.
    - `payload`: the SHA-256 of the raw body bytes. Each request with a body has this tag.
13. Studio refuses the request with 401 `app_signature_invalid` in each of these conditions:
    - The signature is not correct.
    - The public key is not the key of an active app of this workspace. `POST /app/v1/pair` is the only route that accepts a new key.
    - The `created_at` of the event differs by more than 60 seconds from the time of the server.
    - The `u` or `method` tag is not the same as the request, character for character.
    - The request has a body and the event has no `payload` tag, or the `payload` tag is not the SHA-256 of the body.
    - Studio accepted the same event ID in the last 120 seconds.
    - The `created_at` is more than 60 seconds earlier than the latest `created_at` that Studio accepted from that app. This stops an old request after a change of the server clock.
14. Studio refuses a body larger than 64 KiB before it calculates the hash. Studio calculates the hash before it reads the JSON.
15. The workspace keeps the accepted event IDs for 120 seconds, and the latest accepted `created_at` of each app, in its collection, not in memory, so that a restart does not clear them. Studio records an event ID before the route runs. A request that fails after that point uses up its event, and the app signs a new event.
16. `/app/v1/` accepts no other credential. `/api/` and `/auth/` refuse the header of item 12.
17. No log holds a request body, an `Authorization` header, a signed event, a pairing text or a grant vault key.
18. Studio records the time and the client address of the last accepted request of each app. Connected apps shows them, with the short key and the live value origin (screen 7). Only the user of the account sees them.

### The Grant

19. A grant names one app. With one active app, the grant check box names that app (screen 1). With two or more, a select menu gives the app, and its default is "No app" (screen 2). With no active app, the menu is disabled (screen 3). This changes ADR 0054 item 12.
20. A grant vault has one wrapper: the grant vault key, encrypted with NIP-44 version 2 to the public key of the app that the grant names. This is the method of ADR 0058 item 5, with the key of the app in place of a key from a token.
21. When the user removes an app, Studio removes its wrapper from each grant vault at once, and its grants end. At the next time that the user vault is open, Studio makes each of those grant vaults again with a new grant vault key (ADR 0058 item 7). The old key then opens no current grant vault.
22. The grant vault files have mode 0600 (task 068).
23. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that the grant names. For a granted write, the app decrypts its wrapper, and sends the grant vault key in the body of the signed request. Studio opens the grant vault, writes through `pipeline::write_draft`, and clears the keys from memory. Studio keeps no copy of the grant vault key outside the user vault.

### The Live Value

24. The live value origin of an app is the scheme, the host and the port of the address where its listeners get the live value. The scheme is `https` or `wss`. The app gives it at pairing. To change it, the user connects the app again.
25. The review sheet shows the live value origin of the app below the grant check box (screen 1). A grant keeps that origin.
26. The core refuses a granted live value with 400 `live_value_invalid` in each of these conditions:
    - The origin of the `uri` is not the origin of the grant.
    - The `uri` has more than 2048 characters.
    - The `protocol` is not 1 to 32 characters of letters, digits, `.` and `-`.
27. The four operations of ADR 0054 item 10 do not change.

### The Client Crate

28. The Studio repository has a crate `studio-app-client` for apps. It does the full client protocol: the pairing, the signed requests, the wrapper and the four operations. An app gives it a signer, so that the app keeps its key in its own storage (item 2).
29. The crate keeps the grant vault key in memory only, and clears it when the grant ends. It writes no pairing text, signed event or grant vault key to a log.
30. The API tests of Studio use the crate as their client. Thus the client and the server cannot drift apart. A change of the protocol changes the crate in the same commit.
31. The crate has the AGPL-3.0 license of the workspace. The live publisher has the same license.
32. The documentation of the crate gives an example of each step, and a list of the rules for the app: keep the private key private, use one pairing for each copy of the app, keep the grant vault key in memory only, and log no secret.
33. The crate reads a pairing text in two ways, and in no other way:
    - From a terminal prompt that does not show the typed or pasted text.
    - From the clipboard. After the read, the crate clears the clipboard if it still holds the pairing text.
34. The crate accepts the full pairing text, or the code alone when the app already knows the Studio origin. The crate refuses a pairing text in a command argument or an environment variable, because a shell history and a process list keep them. The crate shows no part of the code after the read.
35. The crate refuses a Studio origin that is not `https`, except a loopback address of the desktop shell. The grant vault key and the `Date` header of Studio thus never cross a network without TLS.
36. When Studio refuses a request with 401, the crate reads the time of Studio from the `Date` header of the answer. If the difference from its own clock is 5 minutes or less, it signs the request again with the corrected time, and sends it one time. It keeps that correction for the next requests, and examines it again at each 401. When the difference is more than 30 seconds, the crate gives a warning that names the difference, so that the app can tell the user to correct the clock. When the difference is more than 5 minutes, the crate makes no correction, and gives the error `ClockWrong`.

### Apps From Before This ADR

37. An app with a token, from before this ADR, shows "Connect Again" (screen 8). `/app/v1/` refuses its token with 401 `app_key_required`. A migration removes the token hashes, and each wrapper of a token key goes out of each grant vault.
38. The migration also clears the public key of each token app, and ends each grant from before. A token app thus has no key that Studio accepts.

### Limits, The Index And An Import

39. `server.sqlite` keeps an index from each public key to its account, and from each code hash to its waiting app. The server makes the index again from the collections at each start, in place of the index of token hashes (ADR 0058 item 18, ADR 0107 item 2).
40. The server permits 60 requests for each app public key in each minute. It permits 20 requests with an unknown key, a signature that is not correct or a wrong code, for each client address in each hour. After that, it gives 429 `too_many_attempts`.
41. An import of an account (ADR 0107) keeps the name of each app, and sets each app to "Connect Again". The import keeps no grant. The account page lists the apps with this notice, in place of the notice of ADR 0107 item 14.

### Signals To The User

42. While a grant is active and its feed has a draft with changes, each editor page of that feed shows a notice at the top (screens 9 and 10). The notice names the app and the show, and gives the start time of the show. The core gives this state with the feed, so that the UI has no copy of the rule of ADR 0054 item 16.
43. When Studio refuses a granted write, the list of granted writes of ADR 0054 item 17 shows the app, the cause and the time (screen 11). The causes are a draft with changes, a grant that ended, a live value that item 26 refuses, and a grant vault that does not open.

## Alternatives Considered

- Keep the token, and add "Last used" and "Make a New Token". Not chosen by the operator. The user still copies a secret by hand.
- Credentials for one file, made for each grant. Not chosen by the operator now. Only R2 and AWS S3 can make them, and the operator judged them a lot of work (research note, options 3 and 2).
- An app key that Studio makes and shows one time. Rejected: the transfer of the private key is the same weak point as the token.
- NIP-46 between Studio and the app. Rejected: the app must sign many requests with no person present, and a relay adds a dependency to each write.
- A grant that applies to each active app (the first text of this ADR). Replaced by the amendment: a key that a stolen session adds must not get a grant that the user approved for a different app.
- A grant that names each app that is active at its approval. Not chosen by the operator: one app for each grant is clearer. A backup copy of the app thus cannot run a show that names a different copy.
- The user pastes the public key of the app into Studio, and the app then sends a confirmation code. Replaced by the pairing text: the user makes one copy, not two, and compares no keys.
- One shared vault that Studio and the live publisher both open. Rejected: the live publisher then needs connectors and a feed writer, and nothing enforces the limits of a grant. A stolen key also gets each host secret, for each feed, with no end.
- A separate event kind for the app requests. Not chosen: the `u` tag already separates a sign-in from an app request, and item 4 refuses a sign-in key as an app key.
- A live value address only from a list that the user keeps. Replaced by the live value origin of each app (item 24), which needs no list.
- A notice to the user at each change of the live value. Not chosen: item 26 stops the change, and a notice only shows it.
- One key exchange for each grant session, so that the grant vault key is not in each write body. Not chosen now: Studio would keep a key in memory between requests. Item 15 keeps the key out of each log.

## Consequences

- No long-term secret goes from Studio to the app, or from the app to the user. The pairing text is valid one time, for 15 minutes.
- A stolen session cannot add an app without a recent unlock. Only the app that has the pairing text can register a key.
- A grant applies only to the app that it names. A new app or a backup copy gets no grant that exists. To give a show to a different copy, the user approves a diff with a new grant.
- A copied request stops working after 60 seconds, and Studio refuses a repeated event, also after a restart or a change of the server clock.
- A clock difference of 5 minutes or less causes a warning, not a failed show. A forged `Date` header can only delay a request of the app by 5 minutes or less, and only with a TLS connection that the attacker already controls.
- A stolen private key file of the app still works until the user removes the app. It can start and end a granted show, and set the live value to an address on the live value origin of the app. It cannot send the boosts to a different origin. "Last used" and the list of granted writes of ADR 0054 item 17 show its use.
- The live publisher uses `studio-app-client`. It adds a command that takes the pairing text and its live value origin, and it selects its key storage. That work belongs to its own repository. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface.
- A draft with changes still stops a granted write, as ADR 0054 item 16 gives. Item 40 tells the user before the show, and item 43 tells the user after a refusal.
- Task 073 gives the work in Studio. It comes after task 068 and task 069.

## Acceptance Criteria

Mechanical:

- An API test, through `studio-app-client`: a pairing with a correct code makes an active app, and the signed requests succeed.
- An API test: each condition of item 13 gives 401 `app_signature_invalid`.
- An API test: a repeated event in 120 seconds is refused, also after the workspace loads again.
- An API test: after the server clock goes back by one hour, an event that Studio accepted before is refused.
- A client test: the crate corrects a clock difference of 5 minutes or less with one second request, and gives `ClockWrong` above 5 minutes.
- An API test: a pairing with an expired code, a used code or an unknown code gives 401 `app_code_invalid`.
- An API test: a granted write with the grant vault key in the signed body succeeds. A wrong key gives 423 `vault_locked`, as before.
- An API test: an app that the grant does not name gets 404 for that grant, and no wrapper.
- An API test: after the removal of an app and an unlock, its old grant vault key opens no grant vault.
- An API test: a granted live value on a different origin, with an `http` address or with a wrong protocol gives 400 `live_value_invalid`.
- An API test: a pairing, a granted write and a refused request put no body, signed event, pairing text or grant vault key in the log.
- A server test: "Connect an App" with no recent unlock gives 401 `unlock_required`.
- A server test: a pairing with a key that is active in a different account gives 409 `app_key_in_use`.
- A server test: a sign-in key of an account cannot be an app key.
- A server test: an app with a token gets `app_key_required`. After the migration, a request signed with the key that HKDF derives from the old token gets 401.
- A server test: after an import, each app is "Connect Again" and the account has no grant.
- A Node test: the Connected apps model and the grant line give the lines of screens 1 to 3, 6 to 6c, 7 and 8.
- An API test: with an active grant and a draft with changes, the feed state gives the draft notice. A granted write then fails, and the list of granted writes holds the cause.
- A Node test: the model of an editor page gives the notice of item 42 only when the feed state gives it.
- `cargo test --doc -p studio-app-client` runs the examples of the crate documentation.

Visual: screens 1 to 3, 6 to 6c and 7 to 11 of the mockups, at 1100 px and at 390 px with touch, in the two themes.

Human: the operator connects the live publisher with a pairing text, runs a show with a grant, and sees "Last used" change. The live publisher cannot set a live value on a different origin.
