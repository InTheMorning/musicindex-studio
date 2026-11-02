# Handoff: The Studio Client Of The Live Publisher

Date: 2026-11-01. This note is advisory. [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md) owns the interface in Studio. The live publisher owns its own design, in its own repository and its own decision records. This note makes no rule for that repository.

## Purpose

`musicindex-live-publisher` is the only app that the operator expects to connect to Studio. It has no Studio client on 2026-11-01. This note tells the live publisher how to use the Studio client crate, so that it can plan its client.

## Use The Client Crate

Studio gives the crate `studio-app-client` in `crates/app-client` (task 073). The crate does the full protocol: the pairing, the signed requests, the wrapper and the four operations. Studio tests the crate against its own API in each CI run. The live publisher thus does not write the protocol itself.

- The crate has the AGPL-3.0 license, the same as the live publisher.
- The live publisher depends on the crate through a Git tag of the Studio repository.
- The crate documentation gives an example of each step. Its README lists the rules for an app.

## What The Live Publisher Does

1. It adds a command that reads a pairing text, for example `musicindex-live-publisher studio connect`. The user makes the pairing text in Studio, in Settings, Connected apps, "Connect an App". The command reads the text as "Read The Pairing Text" below gives.
2. It gives the crate its live value origin: the scheme, the host and the port where its listeners get the live value, for example `wss://live.musicindex.org`. The live publisher can make it from its relay `endpoint` setting. Studio refuses each later live value on a different origin.
3. It selects how it keeps its private key. The crate gives `FileSigner` (a file with mode 0600 in a folder with mode 0700). The live publisher can give its own `AppSigner`, for example for systemd credentials or the key store of the operating system.
4. It makes one key pair for each Studio account and each computer. The pairing makes the key, so the user copies no key.
5. During a show, it calls `set_status` and `set_live_value` of the `Client`. The crate gets the grant vault key, keeps it in memory only, and clears it at the end of the grant.

## Rules For The Live Publisher

These rules come from ADR 0110 item 32. The crate README gives the same list.

- Keep the private key private. Do not put it in a log, a backup that other persons can read, or a support message.
- Use one pairing for each computer or copy of the app.
- Keep the grant vault key in memory only. The crate does this. Do not copy it out of the crate.
- Log no pairing text, signed event, request body or grant vault key.
- Keep the clock of the computer correct. Studio refuses a request that is more than 60 seconds from its own time.

## Read The Pairing Text

These rules decrease the risk of a screen share, a recording or a shell history. The crate module `input` applies them (ADR 0110 items 33 and 34).

- Read the pairing text from the clipboard, or from a prompt that does not show the text. Offer the clipboard first, for example "Press Enter to read the pairing text from the clipboard".
- After a read from the clipboard, the crate clears the clipboard if it still holds the pairing text.
- Do not take the pairing text as a command argument or an environment variable. A shell history and a process list keep them.
- Show no part of the code after the read. Show only the result and the short key, for example "Connected to studio.musicindex.org, key npub1lp7w…9k2q". Studio shows the same short key, and the user can compare the two.
- Save the Studio origin after the first pairing. Then accept the code alone, for example `K7M2-Q9XD`, for a user who types it from a phone screen. The crate accepts small letters, spaces and a missing hyphen.
- When the crate gives `CodeUsed`, tell the user that a different program possibly used the pairing text. The user must then remove that app in Studio.

## The Pairing

1. Studio shows a pairing text, for example `https://studio.musicindex.org#pair=K7M2-Q9XD`. Studio hides it, and the user selects Copy. It works one time, for 15 minutes. If the user selects Done in Studio before the pairing, the text stops working.
2. The live publisher makes a key pair, and sends `POST /app/v1/pair`, signed with the new key. The body has `code` and `live_value_origin`.
3. Studio makes the app active. The notice in Studio changes to "Connected", and the user sees the short key and the live value address on the app row.

## A Grant

- A grant names one app. The user selects that app when the user approves the show in Studio.
- A grant gives the four operations on one live item, until the grant ends: go live, end, set the live value and remove the live value.
- A different copy of the live publisher cannot use the grant. The user approves the show again for that copy.

## The Routes

The crate calls these routes. The live publisher does not call them directly.

| Route | Purpose |
| --- | --- |
| `POST /app/v1/pair` | `code` and `live_value_origin`. This makes the app active. |
| `GET /app/v1/whoami` | The app, its state and its live value origin. |
| `GET /app/v1/shows` | The show grants of the app: the grant ID, the feed, the live item, its status, `start`, `end`, `live_value` and the live value origin. |
| `GET /app/v1/shows/{grant_id}/wrapper` | The grant vault key, encrypted with NIP-44 version 2 to the public key of the app. |
| `POST /app/v1/shows/{grant_id}/status` | `live` or `ended`. This writes the feed. |
| `PUT /app/v1/shows/{grant_id}/live-value` | The `uri` and the `protocol` of `podcast:liveValue`. The `uri` is on the live value origin, with a maximum of 2048 characters. The `protocol` has 1 to 32 letters, digits, `.` or `-`. This writes the feed. |
| `DELETE /app/v1/shows/{grant_id}/live-value` | Removes `podcast:liveValue`. This writes the feed. |

## The Errors Of The Crate

| Error | Cause | What to do |
| --- | --- | --- |
| `CodeInvalid` | The pairing text is not correct or expired, or the user selected Done before the pairing. | Ask the user for a new pairing text. |
| `CodeUsed` | A program used this pairing text in the last 15 minutes. | Tell the user that a different program possibly used it. The user removes that app in Studio, and makes a new pairing text. |
| `SignatureInvalid` | The clock is wrong, or Studio does not know the key. | Examine the clock. If the clock is correct, ask the user to connect the app again. |
| `KeyRequired` | The app uses a token from before ADR 0110. | Ask the user to connect the app again. |
| `KeyInUse` | A different account uses this key. | Make a new key, and pair again. |
| `LiveValueInvalid` | The live value is not on the live value origin, or its protocol is not permitted. | Examine the relay setting. To change the origin, the user connects the app again. |
| `NotGranted` | The grant does not name this app, or it ended. | Stop the writes for this show. Tell the operator. |
| `VaultLocked` | The grant vault key is not correct, or the host needs a secret that the grant vault does not hold. | Stop the writes for this show. Tell the operator. |
| `TooManyAttempts` | More than 60 requests in one minute. | Wait one minute, then send again. |
| `Network` | Studio did not answer. | Send again with a new request. The crate signs a new event. |

## Open Points For The Live Publisher

- Where it keeps the Studio address and the key of each account.
- What it does when Studio refuses a write during a show.
- When it reads the shows again, for example after a restart.
- Which `AppSigner` it uses.
