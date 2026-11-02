# Handoff: The Studio Client Of The Live Publisher

Date: 2026-11-01. This note is advisory. [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md) owns the interface in Studio. The live publisher owns its own design, in its own repository and its own decision records. This note makes no rule for that repository.

## Purpose

`musicindex-live-publisher` is the only app that the operator expects to connect to Studio. It has no Studio client on 2026-11-01. This note gives the interface that ADR 0110 sets, so that the live publisher can plan its client.

## What The Client Does

1. It makes one secp256k1 key pair for each Studio account. It keeps the private key in a file with mode 0600, in a folder with mode 0700. This is the method of the relay tokens in `src/relay.rs` and `systemd/*.service`.
2. It shows its public key, for example with `musicindex-live-publisher studio connect`. The user pastes that key into Studio, in Settings, Connected apps, "Connect an App". The key is not secret.
3. It signs each request to `/app/v1/` with an event of kind 27235, in the header `Authorization: Nostr <base64 event>`. The event has these tags:
   - `u`: the full address of the request.
   - `method`: the HTTP method.
   - `payload`: the SHA-256 of the body, when the request has a body.
4. It sends a new event for each request, with the current time. Studio refuses an event that is older than 60 seconds, and an event that it already accepted.
5. Its first signed request confirms the key in Studio. Until then, Studio answers only `GET /app/v1/whoami`.

## The Routes

The routes are the routes of ADR 0054, with one new route.

| Route | Purpose |
| --- | --- |
| `GET /app/v1/whoami` | The app and its state. |
| `GET /app/v1/shows` | The show grants of the app: the grant ID, the feed, the live item, its status, `start`, `end` and `live_value`. |
| `GET /app/v1/shows/{grant_id}/wrapper` | New. The grant vault key, encrypted with NIP-44 version 2 to the public key of the app. |
| `POST /app/v1/shows/{grant_id}/status` | `live` or `ended`. This writes the feed. |
| `PUT /app/v1/shows/{grant_id}/live-value` | The `uri` and the `protocol` of `podcast:liveValue`. This writes the feed. |
| `DELETE /app/v1/shows/{grant_id}/live-value` | Removes `podcast:liveValue`. This writes the feed. |

## A Write

1. The client gets the wrapper of the grant one time, and decrypts it with its private key. The result is the grant vault key.
2. The client adds `grant_key` to the body of each request that writes the feed. The `payload` tag of the signed event covers the body.
3. Studio opens the grant vault, writes the feed, and clears the keys from memory.
4. The client keeps the grant vault key in memory only, and only while the grant is open. It writes the key to no file and no log.

## The Answers Of Studio

- 401 `app_signature_invalid`: the signature, the time, the address, the method or the body hash is not correct, or Studio already accepted the event.
- 401 `app_key_required`: a request with a token from before ADR 0110.
- 423 `vault_locked`: the grant vault key is not correct, or the host needs a secret that the grant vault does not hold.
- 429 `too_many_attempts`: more than 60 requests in one minute.

## Open Points For The Live Publisher

- Where it keeps the address of each Studio, and the key of each account.
- What it does when Studio refuses a write during a show.
- When it gets the wrapper again, for example after a restart.
