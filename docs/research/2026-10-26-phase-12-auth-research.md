# Phase 12 Authentication, Vault Unlock and Convenience Research

Research date: 2026-10-26.
Scope: the server shell of musicindex-studio.
This report states no rule. The owning ADRs own each rule.
The research agent read the repository and changed no file in it.
The report was composed in STE. The shared checker still reports about 120 findings. Most are dictionary limits on approved word meanings. This report does not claim full STE compliance.

## Method And Sources

| Source | URL | Date seen | Confidence |
| --- | --- | --- | --- |
| NIP-46 specification (GitHub master) | https://github.com/nostr-protocol/nips/blob/master/46.md | Fetched 2026-10-26. The page gives no commit date. | Primary |
| NIP-98 mirror | https://nips.4rs.nl/nips/98 | Search result, 2026-10-26 | Secondary |
| Amber (NIP-55 and NIP-46 signer, Android) | https://f-droid.org/packages/com.greenart7c3.nostrsigner/ and https://opensats.org/projects/amber | Search results, 2026-10-26. Posts date from 2025. | Secondary |
| Apple Developer Forums, PRF over cross-device flow | https://developer.apple.com/forums/thread/733413 and https://developer.apple.com/forums/thread/774112 | Search result, 2026-10-26. Posts date from 2024 and 2025. | Community reports |
| Corbado, passkeys and PRF | https://www.corbado.com/blog/passkeys-prf-webauthn | Search result, 2026-10-26 | Vendor, secondary |
| Chromium Intent to Ship, WebAuthn PRF | https://groups.google.com/a/chromium.org/g/blink-dev/c/iTNOgLwD2bI/m/Oz1C7oEYAgAJ | Search result, 2026-10-26 | Primary, date not confirmed |
| OWASP Password Storage Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html | Search snippet only. The page was not fetched. | Secondary |
| Apple Authentication Services documentation | https://developer.apple.com/documentation/authenticationservices/public-key-credential-registration-and-assertion/ | HTTP 404 when fetched. | Not verified |
| Studio ADRs 0003, 0004, 0013, 0018, 0044, 0050, 0054 | `docs/adr/` in the repository | Read 2026-10-26 | Project owner |

Each claim marked [UNVERIFIED] lacks a source that I could read in full.

## Facts From The Code And The ADRs

- `crates/core/src/vault/crypto.rs` uses Argon2id with 64 MiB, 3 passes and 1 lane (ADR 0013 item 7).
- `crates/api/ui/app.js` `takeToken()` stores the desktop access token in `sessionStorage` and reads it from the URL fragment.
- ADR 0004 rejects a passkey because it gives "no stable vault key on each device".
- ADR 0013 item 3 already allows one vault to have several wrappers.
- ADR 0013 item 9 says a recovery key of 16 random bytes uses Argon2id with the passphrase parameters.
- ADR 0013 item 8 says the browser signer decrypts the Nostr wrapper. The server never gets the signer private key.
- ADR 0050 items 1 and 12 limit "My signer" to NIP-07 and make a plan with fixed event times.
- ADR 0054 item 16 refuses a granted write while the vault is locked.
- ADR 0016 says the UI loads nothing from other servers. A NIP-46 client in the browser opens WebSocket connections to relays. ADR 0050 notes that the browser makes a relay connection in that method. This needs a decision.

## Problem 1: Sign-In On A Phone

### Option 1a: NIP-46 remote signer

NIP-46 keeps the private key in a separate signer. The client sends `sign_event`, `nip44_encrypt`, `nip44_decrypt` and `get_public_key` requests over relays. The signer can ask the user to approve through an `auth_url` response. The `nostrconnect://` start method needs a `secret`, and the client must check it. The `bunker://` secret works for one connection only. ([NIP-46](https://github.com/nostr-protocol/nips/blob/master/46.md), fetched 2026-10-26)

- Security: the server never sees the key. The relay sees only encrypted traffic. The signer decides which relays it uses. Client metadata is not authenticated. The spec does not define how permissions are enforced.
- Phone convenience: Amber on Android is a NIP-46 signer. Amber also works with the Remote NIP-07 bridge for desktop browsers ([Remote NIP-07 listing](https://chromeboard.com/extension/remote-nip-07-dkeaighngpmilhnepghmdfjlmlgaomcl), 2026-10-26, secondary). On iOS, the sources say that no NIP-55 or NIP-46 signer app was confirmed. Some NIP-07 Safari extensions exist for iOS 18 and later, for example Nostash and Nostore (search results, 2026-10-26, secondary, no source URL retained for these two). The iOS signer status is [UNVERIFIED] for 2026.
- Cost in this codebase: high. The browser needs a NIP-44 path and a relay client. This may conflict with the "loads nothing from other servers" rule in ADR 0016. The rule may cover only code and assets, not open connections. ADR 0050 already defers NIP-46 for this reason.
- Failure modes: a relay outage stops sign-in. A lost signer loses access. The phone may suspend the signer app while the browser waits.

### Option 1b: Passkey with the WebAuthn PRF extension

A passkey with PRF gives a secret that the authenticator derives from a salt. The vault wrapper key could come from that secret. ADR 0004 rejects passkeys for v1 because the key is not stable across devices. Synced passkeys change that: one synced credential gives the same PRF output on each synced device.

- Support, with confidence limits:
  - Android: Corbado says Google Password Manager passkeys carry PRF across Chrome, Edge and Samsung Internet ([Corbado](https://www.corbado.com/blog/passkeys-prf-webauthn), 2026-10-26, vendor). A Chromium Intent to Ship exists ([Chromium](https://groups.google.com/a/chromium.org/g/blink-dev/c/iTNOgLwD2bI/m/Oz1C7oEYAgAJ)). The date of the current release is [UNVERIFIED].
  - iOS and Safari: iCloud Keychain PRF support is reported from iOS 18 and Safari 18 ([Corbado](https://www.corbado.com/blog/passkeys-prf-webauthn), 2026-10-26). Apple forum reports say that PRF failed in the QR cross-device flow and that the result differed between hybrid and on-device use ([Apple thread 774112](https://developer.apple.com/forums/thread/774112), 2026-10-26). The 2025 and 2026 status is [UNVERIFIED].
- Security: the PRF output is a high-entropy secret, so offline guessing is not possible. The server stores a wrapper ciphertext only. Phishing resistance is strong, because the credential is bound to the origin.
- Failure modes: a lost passkey provider account loses the vault. Output consistency between devices is not confirmed. A user with unsynced hardware keys gets different output. Each platform change can break a wrapper, so a test must check the output on each platform.
- Cost: medium. The Rust side needs a stored salt and a wrapper type. The browser side needs WebAuthn calls. No build step is needed.

### Option 1c: Passphrase on the phone

ADR 0013 uses Argon2id in the server. The 64 MiB cost falls on the server during each sign-in. OWASP lists 19 MiB with 2 passes and 1 lane as a minimum for Argon2id, from the search snippet ([OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html), 2026-10-26, secondary). The ADR setting is above that minimum.

- The memory cost is a load risk, not a security gap. A concurrency limit and a per-account rate limit reduce it. The limit is not in the code yet.
- Typing cost on a phone is the main convenience problem. A password manager autofill, and paste, fix most of it. ADR 0018 forbids advice in the UI, so the user brings the manager.
- Deriving the key in the browser with WebAssembly does not help. The server needs the vault key to write during the session, so the server still runs the derivation or receives the key.

### Option 1d: Email plus a sign-in code (not evaluated in depth)

This needs an email channel. ADR 0004 and the operator decision exclude sending email. The option is out of scope.

## Problem 2: The Session On A Phone

Facts:
- Discard of a background tab in the browser does not end the server session. The server session continues until its timeout. The browser can lose its copy of a token.
- Cookies need CSRF protection. Fetch metadata (`Sec-Fetch-Site`) and a per-session CSRF token both reduce this risk. The browser support for `Sec-Fetch-Site` is [UNVERIFIED] in this pass.

Options:
- 2a. HttpOnly cookie with the `__Host-` prefix, `Secure`, `SameSite=Strict`, plus a CSRF token in each request. The browser does not read the session token, so an XSS flaw cannot copy it. The current desktop `sessionStorage` token pattern is not used.
- 2b. Session token in `sessionStorage` (the desktop pattern). Simple. It is readable by any script in the origin.
- 2c. Idle timeout (about 15 to 30 minutes) and absolute timeout (about 8 to 12 hours). A phone user who returns then signs in again with a fast method (passkey or signer).
- 2d. Lazy unlock: the server keeps the vault key only during one write request. The browser sends the proof for each write. This shrinks the memory window, but it asks for a proof on each write.
- Memory exposure: the server must clear the vault key with `zeroize` on lock and on timeout (ADR 0013 item 13). Memory swap and core dumps are [UNVERIFIED] for the deployment host.

Recommendation for this problem: 2a with 2c. Keep 2d as a choice for the "personal server" profile.

## Problem 3: Signatures For nsite And Blossom

Options:
- 3a. NIP-46 signer for "My signer" (supersedes ADR 0050 item 1 and the NIP-07-only limit of ADR 0044 item 3).
- 3b. Keep NIP-07 only. A phone user with no extension uses "Key in the vault".
- 3c. "Key in the vault" for every user. The vault key then holds the Nostr key, which ADR 0044 already accepts. A breach of the running server exposes this key while the session is open.
- 3d. Pre-signed plan (ADR 0050 items 10 to 12). The user signs the planned events during the review step. The review step is the only signer step. This works with 3a and 3b, and it already exists in the design.

Points:
- The plan model removes the per-write signature round trip. A NIP-46 round trip then takes place once for each review.
- The plan window is 10 minutes or more. Relay latency for NIP-46 can be seconds. A test must cover a slow signer.
- Blossom authorization events have a one-hour expiration in ADR 0050 item 13. This is a fixed time, so a signed event can be stored until its expiration.
- NIP-26 delegation could let a key sign for another key. Its status is [UNVERIFIED] in this pass.

## Problem 4: Writes With No Person Present

The core fact: an unattended write needs the host secret on the server at the time of the write. ADR 0003 item 4 says the server has no key that opens a vault without the user. This is a direct conflict. Options:

- 4a. Refuse the write when the vault is locked (ADR 0054 item 16, current). A show that starts at a time with no session fails. The artist may lose a show.
- 4b. Keep a grant-scoped credential in the vault. The grant holds one restricted secret. Example: an S3 key that can write one prefix. The secret is still plain on the server during the write. This needs a supersession of ADR 0003 item 4.
- 4c. Pre-signed host operations made at grant time, with no long-lived secret on the server:
  - nsite and Blossom: the user signs the events with a fixed expiration during the grant setup. The grant then includes the signed events. The connected app sends an event that already exists. This needs no secret. The events cover one manifest, so a live stop or start needs one signed event per state change.
  - S3 and R2: a presigned URL for one object key with a fixed expiration. The maximum expiration is [UNVERIFIED] for the provider in use. The key of the object is fixed, so a live status file works, but a new object key does not.
  - SFTP: no presigned form exists in the protocol. This option does not apply.
- 4d. Run granted writes on the user's own computer (the desktop shell or a home server) only, where the vault stays unlocked by the local passphrase. This matches ADR 0054 item 7 and the desktop model.
- 4e. A time-limited unlock key given to the server at grant time. The key is held in server memory only for the grant period. This is a supersession of ADR 0003 item 4 and of ADR 0054 item 16. It is the most convenient option and the weakest in a breach.

Points: 4c is the best fit for nsite and Blossom, and for S3 and R2 with a fixed object. 4e is the only option that covers SFTP and a changed object key without a user present. It needs an explicit decision by the operator and a new ADR.

## Problem 5: More Than One Device And Method

- The vault already allows several wrappers (ADR 0013 item 3). Adding a method needs an open session. This is the correct limit: a method can be added only by a user who already opened the vault.
- A second device can enroll a passkey or a signer during an open session.
- A user who loses every method and the recovery key loses the secrets. The limit is in ADR 0003.
- Each wrapper type needs its own test: unlock, change, remove, and refusal of a wrapper that does not decrypt (ADR 0013 invariant).

## Problem 6: Recovery

Options:
- 6a. Recovery key of 16 random bytes, shown once (ADR 0013 item 9). Keep it. Change the derivation: 16 random bytes already have 128 bits of entropy. Argon2id is not needed. A fast KDF such as HKDF-SHA-256 is enough, and it avoids 64 MiB of server memory for recovery. This is a change to ADR 0013 item 9, and the ADR text must be updated.
- 6b. Two wrappers from different providers (for example a passkey and a signer). This reduces the chance of a total loss. It needs no new code.
- 6c. Printed or stored recovery key with a short check. The UI must show the key once and require the user to confirm a copy.
- 6d. Shamir secret sharing among the user's methods or devices. Out of scope for v1. Maturity: [UNVERIFIED] for a Rust crate in this codebase.
- 6e. Social recovery through Nostr guardians. Not verified. Not recommended for v1.

## Problem 7: Attacks On Public Sign-In

| Attack | Current design | Control |
| --- | --- | --- |
| Passphrase guessing offline | The wrapper and the Argon2id parameters are in the database after a breach. | zxcvbn score 3 and 12 characters (ADR 0018). Argon2id 64 MiB. The score 3 range is about 10^8 to 10^10 guesses from memory of the zxcvbn documentation [UNVERIFIED in this pass]. |
| Passphrase guessing online | Rate limit not yet built. The 64 MiB cost is also a denial-of-service risk. | Per-account and per-address limits. A cap on the number of concurrent Argon2id runs. Same error and same time for each unknown account (ADR 0013 item 17). |
| Replay of a signed sign-in event | NIP-98 has no nonce. A timestamp window of 60 seconds is only a suggestion. | Server challenge of 32 bytes, stored with a short expiry, consumed in one atomic step, bound to URL, method, origin and the expiry. This is ADR 0004 item 2. |
| Phishing of a NIP-07 signature | A signature can be made for another site. | Bind the event to the origin and the challenge. The signer must show the origin to the user. [UNVERIFIED] that current signers show it. |
| Stolen session token | Desktop uses `sessionStorage`. | HttpOnly `__Host-` cookie, short timeouts, token stored hashed on the server. |
| Stolen app token | ADR 0054 item 8 stores a SHA-256 hash of the token. | Keep. Add a rate limit and a revocation check on each request. |
| XSS | The UI loads no other origin code (ADR 0016). | A strict Content-Security-Policy with no inline scripts, if the UI allows it. [UNVERIFIED] for the current UI. |
| Server memory breach | The vault key is in memory during a session. | Short session, lazy unlock (2d), zeroize, no core dumps. |

## Cross-Problem Solutions

- Passkey with PRF addresses problems 1, 5 and 6 together, as a second wrapper.
- NIP-46 addresses problems 1 and 3 together.
- Pre-signed plans and presigned URLs address problems 3 and 4 together.
- A signed challenge with origin binding addresses problem 7 for both Nostr and passkey sign-in.

## Ranked Recommendation For A Phase 12 Design

1. Sign-in: keep email and passphrase as the universal method, because it already works on every device. Add NIP-07 for desktop. Add NIP-46 for phones, with Amber on Android first. Make NIP-46 a separate record, because it changes ADR 0004 and ADR 0050 item 1. Treat iOS support as a risk until a test on a real device passes.
2. Passkey with PRF: test it as a second wrapper. Do not make it the only method. Require a cross-device test on Android and iOS before the record is accepted. This supersedes the passkey rejection of ADR 0004 only if the test passes.
3. Session: HttpOnly `__Host-` cookie, `SameSite=Strict`, CSRF token, idle and absolute timeouts, a `lock` action, `zeroize` on each path. Add a limit on concurrent Argon2id runs and a rate limit per account.
4. Signatures: plan-time signing (ADR 0050) with NIP-46 or NIP-07 at review. Use "Key in the vault" only as a fallback.
5. Granted writes: use presigned operations for nsite, Blossom and fixed S3 or R2 objects. Keep the refusal while the vault is locked for SFTP and other connectors. Offer a time-limited unlock (4e) only after an explicit operator decision, with a new ADR that supersedes ADR 0003 item 4.
6. Recovery: keep the recovery key. Change its derivation to a fast KDF (ADR 0013 item 9). Recommend two wrappers from different providers.
7. Attacks: origin-bound single-use challenges for all sign-in events, rate limits, and the memory controls above.

Residual risks:
- A breach of the running server exposes each open vault for the session length.
- A stolen passphrase guessed offline is a risk if the passphrase is weak.
- Granted writes without a session remain weak for SFTP.
- Relay trust in NIP-46. A relay can delay or drop messages.
- A passkey provider account loss.
- iOS signer status is unknown.

## Not Verified

- iOS signer app support for NIP-46 or NIP-55 in 2026.
- Current PRF support and output consistency on iOS and Android, and in each release after 2025.
- The OWASP page itself. Only a search snippet was read.
- Apple's Authentication Services documentation (HTTP 404).
- The NIP-98 text and its replay section. Only a mirror was read.
- NIP-26 status.
- Presigned URL maximum expiration for each provider.
- `Sec-Fetch-Site` browser support.
- zxcvbn score ranges.
- Guess rates for Argon2id on GPU hardware.
- Whether current NIP-07 signers show the requesting origin.
- Social and Shamir recovery crate maturity.
- Whether ADR 0016 allows a WebSocket to a relay from the browser.

## Operator Decisions

On 2026-10-26 the operator selected the decisions of ADR 0055 to ADR 0059. The planning model examined the presigned operations of problem 4. A presigned S3 or R2 address accepts each content. An nsite or Blossom event names the hash of one file, so a live value from an app cannot be signed in advance. The operator selected the grant vault of ADR 0058 in place of presigned operations.
