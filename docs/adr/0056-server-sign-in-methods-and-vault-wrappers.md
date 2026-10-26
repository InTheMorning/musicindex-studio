# ADR 0056: Server Sign-In Methods And Vault Wrappers

- Status: Accepted
- Date: 2026-10-26
- Owner: operator
- Amended: 2026-10-26 by operator decision, after packet 035b. Item 21 uses one fixed PRF salt for each passkey of the server, not a random salt for each credential. A discoverable sign-in does not know the credential before the assertion, so the browser can send only one salt.

## Context

[ADR 0004](0004-server-sign-in-with-nostr-or-email.md) gives two sign-in methods: a Nostr signer through NIP-07 or NIP-46, and an email address with a passphrase. It rejects a passkey, because "a passkey does not give a stable vault key on each device". [ADR 0013](0013-vault-format-and-cryptography.md) gives the vault wrappers. Its item 19 says that the Rust code does not use NIP-44. [ADR 0050](0050-nsite-and-blossom-signer-environment-and-plan.md) item 1 limits "My signer" to NIP-07.

The goal of the operator for Phase 12 is a change of a feed from a phone. The [Phase 12 authentication research](../research/2026-10-26-phase-12-auth-research.md) of 2026-10-26 gives these observations:

- Phone browsers almost never have a NIP-07 extension.
- Amber on Android is a NIP-46 signer. The research found no reliable source for a NIP-46 signer on iOS in 2026.
- The WebAuthn PRF extension gives a stable secret from a passkey. Sources report it for iCloud Keychain on iOS 18 and for Google Password Manager on Android. The sources for iOS conflict, so a test on a real device must confirm it.
- A passphrase of 12 characters or more with a zxcvbn score of 3 is slow to type on a phone. Each Argon2id derivation uses 64 MiB of server memory.
- The recovery key has 128 random bits. A slow derivation adds no protection to it.
- NIP-98 has no nonce. A signed event without a challenge from the server can be sent again.

On 2026-10-26 the operator selected the decisions below.

## Decision

### The Methods

1. The server has four sign-in methods. This replaces the list of ADR 0004 item 1.
   - An email address and a passphrase.
   - A Nostr signer through NIP-07, in a browser with a signer extension.
   - A Nostr signer through NIP-46. The server is the NIP-46 client.
   - A passkey with the WebAuthn PRF extension.
2. A new account starts with an email and a passphrase, with NIP-07 or with NIP-46. The user adds a passkey later, while the vault is open. A passkey is not the first method, because a passkey manager with no PRF cannot open a vault.
3. The account page lists the methods of the account. The user can add a method or remove one while the vault is open. The server refuses the removal of the last method.
4. At sign-up, Studio offers the recovery key of ADR 0013 item 9.

### The Challenge

5. Each Nostr sign-in and each passkey sign-in uses a challenge from the server: 32 random bytes. The challenge expires after 5 minutes. The server removes it in the same database step that accepts it, so it works one time only. This applies ADR 0004 item 2.
6. A Nostr sign-in event has kind 27235. Its `u` tag is `<public origin>/auth/v1/nostr`, its `method` tag is `POST`, and its `challenge` tag is the challenge. Its `created_at` is within 60 seconds of the server time. The server checks the signature with the `nostr` crate.

### Email And Passphrase

7. The passphrase opens the vault, and that is the proof of the sign-in (ADR 0013 item 16). The sign-in thus also opens the vault window of [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md).
8. For an unknown email address, the server does an Argon2id derivation on a fixed wrapper with the same parameters. It gives the same error (ADR 0013 item 17).

### NIP-07

9. The sign-in gives a session. The vault stays locked.
10. To open the vault, the UI asks the NIP-07 signer to decrypt the Nostr wrapper with `nip44.decrypt`. The UI sends the vault key to `POST /api/v1/vault/unlock-key`. The route is in both shells.
11. At sign-up, the server makes a vault with a new vault key and no passphrase. The UI asks the signer to encrypt the vault key to the public key of the user with `nip44.encrypt`, and the server keeps the wrapper.

### NIP-46

12. The core has a NIP-46 client. It uses the relay client of the core and NIP-44 of the `nostr` crate. This supersedes ADR 0013 item 19: the Rust code now does NIP-44 for NIP-46 messages and for the Nostr wrapper.
13. The user connects a signer in two ways:
    - A `nostrconnect://` address from Studio, as a link and as a QR code. On a phone, the link opens Amber.
    - A `bunker://` address that the user pastes from the signer.
14. Each connection has a new client key. The server keeps the client key in memory for the life of the session only. After a restart, the user connects the signer again when Studio needs it.
15. A NIP-46 sign-in asks the signer for its public key and for a signature of the event of item 6. A NIP-46 unlock asks the signer to decrypt the Nostr wrapper with NIP-44.
16. Each relay address of a NIP-46 connection goes through the private-network guard of [ADR 0057](0057-the-private-network-guard-of-the-server.md).

### "My Signer" Through NIP-46

17. "My signer" can sign the planned events of a write through the NIP-46 connection of the session. This supersedes ADR 0050 item 1, which limits "My signer" to NIP-07. The signing plan of ADR 0050 items 10 to 13 does not change.
18. The review sheet asks the NIP-07 signer when the browser has one, or else the NIP-46 signer of the session. Before the write, Studio still makes sure that the public key of each signed event is the public key of the host ([ADR 0044](0044-the-nsite-signer.md) item 4).
19. The NIP-46 signing works in both shells, because the client is in the core.

### Passkeys With PRF

20. A passkey is a discoverable credential with user verification. Studio accepts only the ES256 algorithm, which iCloud Keychain and Google Password Manager use. The relying party ID is the host of the public origin.
21. To add a passkey, the browser makes the credential and then does one assertion with the PRF extension. The PRF input is one fixed 32-byte salt of the server: the SHA-256 of `musicindex-studio passkey prf salt v1`. The PRF output is different for each credential, because the secret of each credential is different. The browser sends the PRF output to the server.
22. The server derives a wrapper key from the PRF output with HKDF-SHA-256, with the salt of the credential. That key encrypts the vault key with XChaCha20-Poly1305. The vault keeps a passkey wrapper with the credential ID. This adds a wrapper type to ADR 0013.
23. The server refuses a passkey that gives no PRF output, with 422 `passkey_no_prf`. The UI tells the user that this passkey manager cannot open the vault.
24. A passkey sign-in makes a session and opens the vault in one step. The server checks the signature over the authenticator data and the hash of the client data. It also checks the challenge, the origin, the hash of the relying party ID and the user verification flag. A signature counter that is not zero must increase.
25. The server keeps the credential ID, the public key, the signature counter, the salt and a name in `server.sqlite`. These are not secrets.
26. The server verifies ES256 with the `p256` crate, and derives keys with the `hkdf` crate. Both are RustCrypto crates with the MIT or Apache-2.0 license ([ADR 0011](0011-agpl-license.md)). The browser gives the public key through `getPublicKey()`, so the server needs no CBOR parser.
27. This supersedes the passkey alternative of ADR 0004. The PRF extension gives a stable vault key on each synced device.

### The Recovery Key

28. A new recovery wrapper derives its key from the 16 bytes of the recovery key with HKDF-SHA-256 and a random salt. This supersedes the Argon2id derivation of ADR 0013 item 9 for new recovery keys. A wrapper of the earlier type still opens.
29. On the server, the user signs in with the recovery key and the email address or the `npub` of the account. The vault then opens, and the user must add a new method before the session continues.

## Alternatives Considered

- NIP-07 now and NIP-46 later. Not chosen by the operator. Nostr users then cannot sign in from a phone.
- A NIP-46 client in the browser. Not chosen by the operator. The UI then needs NIP-44 code and connections to relays, which [ADR 0016](0016-ui-without-a-build-step.md) does not permit.
- A passkey as the only method, or a passkey with no PRF. Not chosen. A passkey manager with no PRF cannot open the vault.
- The `webauthn-rs` crate. Not chosen: Studio needs one algorithm and one assertion check. Its own verifier with `p256` is small and adds no OpenSSL path.
- Argon2id for the recovery key. Not chosen by the operator. It costs 64 MiB for each recovery and adds no protection to 128 random bits.

## Consequences

- On an iPhone or an Android phone, a user signs in with a passkey and opens the vault with Face ID or a fingerprint. Before that, the user adds the passkey one time, with any other method.
- An Amber user signs in from an Android phone and signs nsite and Blossom writes there.
- An iPhone user with only a Nostr signer has no phone method. That user must add a passkey, or wait for an iOS NIP-46 signer that works.
- The vault key goes through the browser for NIP-07 and through the server for each method. This was already true (ADR 0013 consequences).
- A relay can delay or drop NIP-46 messages. The UI shows a wait state and a time limit.
- The workspace gets three crates: `p256`, `hkdf` and `qrcode`. The API makes the QR code of item 13 as SVG with `qrcode`, so the UI needs no QR code. The `nostr` crate gets the features `nip44` and `nip46`.

## Verification

Mechanical:

- Unit tests open a vault with each wrapper type, and refuse a wrapper that does not decrypt.
- A test refuses a used challenge and an expired challenge. It also refuses a Nostr event with a different `u` tag or `method` tag, and an event outside the 60-second window.
- A test confirms the same error for an unknown email address and for an incorrect passphrase.
- A test signer on a local test relay does a NIP-46 sign-in and a NIP-46 unlock. It also signs a planned event.
- A test with fixed WebAuthn data from a test key adds a passkey and signs in. It refuses an incorrect origin, an incorrect challenge and a counter that does not increase.
- A test opens a vault with a new recovery key, and with a recovery wrapper of the earlier type.

Visual:

- The sign-in page with each method, the NIP-46 connection with its QR code, the passkey list of the account page. Each one in the dark theme, in the light theme and at 390 px.

Human test:

- A passkey that the user adds on an iPhone (Safari, iCloud Keychain) signs in and opens the vault on that iPhone and on a second Apple device.
- The same test on Android (Chrome, Google Password Manager).
- A NIP-46 sign-in and a "My signer" write to an nsite test host with Amber on Android.
- A NIP-46 sign-in on an iPhone with a signer that the operator selects, if one works.

## Invariants

- A signed sign-in event or a passkey assertion works one time only.
- The server keeps no PRF output, no NIP-46 client key and no vault key after the session or the vault window ends.
- Each vault wrapper encrypts the same vault key. A wrapper that does not decrypt gives an error, not partial data.
