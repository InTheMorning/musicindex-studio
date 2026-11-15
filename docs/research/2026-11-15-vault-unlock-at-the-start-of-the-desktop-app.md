# Research: Vault Unlock At The Start Of The Desktop App

Date: 2026-11-15. This note gives options for an operator decision. It makes no rule.

## Summary

- The desktop opens its workspace at start, and the vault stays locked until an unlock route opens it.
- The vault supports passphrase, recovery, Nostr and passkey wrappers. It has no operating system wrapper.
- `keyring` 4.2.0 uses the Linux Secret Service backend by default with its `v1` feature.
- An AppImage is not a sandbox. A running session bus and its socket are still required for Secret Service access.
- WebKitGTK has WebAuthn PRF code, but its GLib port rejects WebAuthn requests in the reviewed source.
- An automatic local unlock gives access to any process that can read the saved credential.
- The passphrase wrapper remains portable between computers. A local operating system wrapper does not.
- The server shell should keep its current user-driven unlock behavior.

## Option 1: Operating System Secret Store Wrapper

### How it works and evidence

Studio generates a random secret and stores it in the operating system secret store. Studio adds a wrapper for that secret to the vault file. At desktop start, Studio reads the secret and opens the wrapper. The passphrase wrapper stays in the file.

The vault code supports a vault key and wrapper methods. `OpenVault` holds the key in memory. `VaultFile::open_with_key` checks a supplied key against the vault check (`crates/core/src/vault.rs:511-539`). This method needs a new wrapper type or a wrapper that uses the random secret. The current vault format has no operating system wrapper.

The `keyring` crate is a candidate. Docs.rs lists version 4.2.0, MIT OR Apache-2.0, and a Rust version requirement of 1.88.0. Its upstream release page dates v4.2.0 to 2026-08-29.

The crate's default `v1` feature selects Keychain Services on macOS, Windows Credential Manager on Windows, and Secret Service on Unix-like systems. Its Linux `v1` dependency is `zbus-secret-service-keyring-store`. The older `dbus-secret-service-keyring-store` and `linux-keyutils-keyring-store` are optional dependencies, not default `v1` backends. [S1] [S2] [S3]

The Secret Service API runs over D-Bus and defines service prompts. Its store must be available on the user's session bus. A missing provider or session bus prevents this backend from completing a store request. The `keyring` and backend documentation gives errors, but it does not promise one stable error for every missing-provider case. [S4] [S5]

An AppImage runs as a regular application and does not provide a sandbox. It can use the host session bus when the user's environment and bus socket are available. The AppImage documentation does not state this specific Secret Service behavior. This access conclusion is an inference from the AppImage model and the Secret Service D-Bus requirement. [S6] [S4]

Apple Keychain Services can present access prompts according to the item's access controls and application identity. The reviewed Apple documentation does not promise a prompt-free read for every packaged app. Windows Credential Manager exposes credentials to the logged-on user through its API. Microsoft documents UI functions for credential requests, but the reviewed page does not say that an application read of an existing generic credential always prompts. [S7] [S8]

### Good points

- The passphrase wrapper remains available on another computer.
- Studio stores a random secret, not the user's passphrase.
- Each operating system provides a store for saved credentials.
- Studio can read the secret without a prompt when the store permits access.

### Bad points

- A process running as the user may be able to request the saved secret.
- Linux needs a Secret Service provider and a working session bus.
- Keychain access controls can require user approval. Prompt behavior depends on the item and application.
- A copied vault file cannot use the local wrapper on another computer.
- The app still needs tests with packaged builds on each target.

### Decision changes

ADR 0003 item 2 must allow the desktop to open its vault with a local operating system wrapper. Its portability alternative must distinguish the portable passphrase wrapper from this local wrapper. ADR 0142 item 2 must state whether automatic unlock changes startup behavior. The "Never" idle-lock value remains a separate setting.

## Option 2: Store The Passphrase In The Operating System Secret Store

### How it works and evidence

Studio stores the artist's passphrase in the operating system secret store after explicit consent. At start, Studio reads it and uses the current passphrase unlock route. The route derives a wrapping key with Argon2id (`crates/core/src/vault.rs:518-535`). The workspace unlock method uses that route (`crates/api/src/workspace.rs:1465-1471`). The desktop currently starts its workspace without a passphrase (`crates/desktop/src/lib.rs:297-304`).

The stores, crate version, license, release date, Linux backend and access limits are the same as Option 1. [S1] [S2] [S3] [S4] [S7] [S8]

### Good points

- This needs no new vault wrapper format.
- The passphrase can still open a vault copied to another computer.
- The operating system store can provide its own access controls.

### Bad points

- Studio stores a reusable passphrase, not a random vault secret.
- A process with access to the store may obtain the passphrase.
- A passphrase change needs a reliable update of the stored value.
- The same Linux, AppImage and store prompt limits apply as in Option 1.

### Decision changes

ADR 0003 item 2 must allow Studio to retrieve and use a saved passphrase. The artist's passphrase still opens the vault, but the user need not enter it at each start. ADR 0142 item 2 must state the default for saved passphrase use. The idle-lock choice remains independent.

## Option 3: Use A Passkey At Desktop Start

### How it works and evidence

The core supports a wrapper derived from a WebAuthn PRF output. It identifies the wrapper by credential ID and salt (`crates/core/src/vault.rs:296-318`). The API exposes passkey unlock (`crates/api/src/workspace.rs:182-195`). The UI uses WebAuthn through browser APIs (`crates/api/ui/app.js:4163-4214`). A desktop start could request an assertion and use its PRF output.

The Web Authentication standard defines the PRF extension. WebKit merged PRF and `hmac-secret` implementation code in November 2025. Later WebKit work states that the GLib port's WebAuthn request path rejects requests as unsupported. This means the reviewed WebKitGTK code does not establish working WebAuthn PRF in WebKitGTK. [S9] [S10]

The WebKit pull request establishes upstream implementation, but it does not identify the first released Safari or WKWebView version with PRF support. A search of Apple's WebAuthn and Authentication Services documentation found no statement about PRF support in WKWebView. The WebView2 documentation reviewed for this task also gave no PRF support statement. These platform results remain unknown. [S11] [S12]

### Good points

- The vault format already supports this wrapper.
- The passphrase wrapper can stay for recovery and portability.
- The authenticator can require user presence or verification.

### Bad points

- An assertion normally needs user action, so startup may not be silent.
- PRF support and credential availability vary by authenticator and platform.
- The reviewed sources do not confirm PRF support in the three Tauri webviews.
- A passkey-only vault is not portable without a recovery method, as account code warns (`crates/api/src/account.rs:160`).

### Decision changes

ADR 0003 item 2 must name a passkey wrapper as another desktop unlock method. ADR 0142 item 2 must state whether startup may ask for a passkey. The idle-lock value remains independent.

## Option 4: Keep The Current Passphrase Prompt

### How it works and evidence

The desktop calls `Workspace::open` and gives it no passphrase (`crates/desktop/src/lib.rs:289-304`). `Workspace::open` sets `vault: None` (`crates/api/src/workspace.rs:956-986`). The vault stays locked until an unlock route succeeds. The UI posts the passphrase to `/vault/unlock` (`crates/api/ui/app.js:1799-1818`). ADR 0142 makes "Never" the desktop idle-lock start value. It does not unlock the vault after a process restart (`docs/adr/0142-a-user-selects-the-time-before-the-vault-locks.md`, item 2).

### Good points

- Studio needs no local copy of an unlock secret.
- The passphrase wrapper works when the encrypted vault file moves.
- This matches the current rule of ADR 0003 item 2.

### Bad points

- The artist must enter the passphrase after each application start.
- The desktop cannot use vault secrets until the user unlocks it.

### Decision changes

No change to ADR 0003 item 2 or ADR 0142 item 2.

## Option Comparison

| Option | Linux | macOS | Windows |
| --- | --- | --- | --- |
| OS secret store wrapper | Secret Service through `zbus`, with session provider and bus | Keychain Services, which can prompt under some access controls | Windows Credential Manager, used under the logged-on user |
| Saved passphrase | Same Secret Service dependency | Keychain Services | Windows Credential Manager |
| Passkey PRF | Reviewed WebKitGTK GLib path rejects WebAuthn requests | Upstream WebKit PRF code exists. Released WKWebView support is unverified | WebView2 PRF support is unverified |
| Current prompt | Works with current desktop code | Works with current desktop code | Works with current desktop code |

AppImage does not add a sandbox. Secret Service access still requires the session bus and its socket. The project's release workflow builds Linux archives on Ubuntu runners. That does not prove a user's session has a Secret Service provider (`.github/workflows/desktop-release.yml`). See Tauri research, dated 2026-11-03.

## Answers To The Questions

1. **Storage by target:** The default `keyring` `v1` backend uses Secret Service, Keychain Services, and Windows Credential Manager on Linux, macOS, and Windows. Linux kernel keyring is optional, and the sources do not establish a TPM option for all targets. Inference: an AppImage can use the session bus when the environment and socket are available. [S1] [S2] [S3] [S4] [S6]

2. **Additional wrapper:** The vault can keep the passphrase wrapper and add a separate wrapper in principle. It does not yet have an operating system wrapper. Passkey wrappers already exist, but their metadata and PRF handling differ from a keychain secret.

3. **Rust crate and platform support:** `keyring` has MIT or Apache licensing, and its latest release was in August 2026. Its Linux default backend uses zbus and Secret Service, but needs a provider and session bus. The backend docs do not define a stable error when either is absent. WebKit's GLib port rejects WebAuthn requests, and Apple and WebView2 searches found no released PRF support statement. [S1] [S2] [S3] [S4] [S5] [S10] [S11] [S12]

4. **Other programs:** A process that runs as the same user may be able to request a saved secret. The exact access and prompt depend on store controls. The comparison with "Never" is that a saved secret can survive process exit, while the opened vault key stays in process memory. The workspace clears its vault on lock (`crates/api/src/workspace.rs:1610-1613`).

5. **Store unavailable or wrong secret:** Studio should start locked and show the current unlock screen. If a retrieved secret fails, Studio should report unlock failure and offer the passphrase route. This behavior is a proposal, not current code.

6. **Setting:** A separate desktop setting could offer "Ask at start" and "Unlock from this device." "Ask at start" is the safer start value. This is a proposal. ADR 0142's lock-time setting controls idle locking, not startup unlock (`crates/api/src/workspace.rs:1202-1254`).

7. **Server:** This option should apply only to the desktop. The server has separate per-user vaults and sign-in methods (ADR 0003 items 3-4). It has no desktop operating system store. ADR 0003 item 4 says no other server key opens a vault without the user.

## Recommendation

Choose Option 1 only if the operator accepts access by processes that can use the user's operating system store. Keep the passphrase wrapper, make local unlock opt-in, and retain the passphrase route when the store is absent or fails. Test packaged Linux, macOS and Windows builds before release. Keep the server unchanged. If silent startup is not required, keep Option 4.

## Items Not Found

- The reviewed AppImage concepts page and the search `AppImage session bus Secret Service` did not state whether every AppImage inherits the session bus socket. The access conclusion above is an inference from its non-sandboxed application model and the D-Bus requirement.
- The reviewed Apple Keychain Services documentation did not state whether this packaged Studio app will prompt. Search: Apple Keychain Services generic password read prompt app access control.
- The reviewed Microsoft Credential Manager page did not state whether reading an existing generic credential prompts the user. Search: Windows Credential Manager CredRead generic credential UI prompt.
- The upstream WebKit pull request confirms merged PRF implementation code. A search of Apple's WebAuthn, Authentication Services, and WKWebView documentation found no released WKWebView version with PRF support.
- A WebView2 feedback search for "WebAuthn PRF" found no support statement. The candidate Microsoft Learn URL returned 404.
- Searches of the keyring and zbus backend docs for "no service," "missing provider," and "session bus" found no stable error for all failure cases.
- A docs.rs search for "Rust TPM keyring Linux macOS Windows" found no verified cross-platform TPM storage API.

## Sources Read On 2026-11-15

| Tag | Source | Finding |
| --- | --- | --- |
| S1 | [docs.rs keyring 4.2.0 source](https://docs.rs/crate/keyring/4.2.0/source/Cargo.toml) | License, Rust version, version and optional Linux backend dependencies. |
| S2 | [docs.rs keyring v1 module](https://docs.rs/keyring/4.2.0/keyring/v1/index.html) | Default target stores and the `v1` API. |
| S3 | [keyring-rs releases](https://github.com/open-source-cooperative/keyring-rs/releases) | v4.2.0 release date: 2026-08-29. |
| S4 | [Secret Service API](https://specifications.freedesktop.org/secret-service/latest/) | Secret Service uses D-Bus and defines prompts. |
| S5 | [docs.rs zbus Secret Service store](https://docs.rs/zbus-secret-service-keyring-store/1.0.1/zbus_secret_service_keyring_store/) | Backend uses Secret Service. Its docs have a Headless usage section. |
| S6 | [AppImage concepts](https://docs.appimage.org/introduction/concepts.html) | AppImage is a regular executable application. Its page does not describe session bus access. |
| S7 | [Apple Keychain Services](https://developer.apple.com/documentation/security/keychain_services) | The page title loaded, but its JavaScript content did not load. The exact packaged-app prompt behavior was not established. |
| S8 | [Microsoft Credentials Management](https://learn.microsoft.com/en-us/windows/win32/secauthn/credentials-management) | The page describes credential APIs and UI functions. Generic credential read prompt behavior was not established. |
| S9 | [Web Authentication Level 3](https://www.w3.org/TR/webauthn-3/) | Standard PRF extension definition. |
| S10 | [WebKit PRF implementation](https://github.com/WebKit/WebKit/pull/53734) and [GLib WebAuthn change](https://github.com/WebKit/WebKit/pull/72235) | PRF implementation merged. The GLib request path rejects WebAuthn as unsupported. |
| S11 | [Apple WebAuthn documentation](https://developer.apple.com/documentation/authenticationservices) and [WebKit PRF implementation](https://github.com/WebKit/WebKit/pull/53734) | Search did not identify a released WKWebView version with PRF support. |
| S12 | [WebView2 feedback search](https://github.com/MicrosoftEdge/WebView2Feedback/issues?q=WebAuthn+PRF) | Search found no PRF support statement. The candidate Microsoft Learn URL returned 404. |

