# ADR 0109: Images And Audio Load From Their Hosts

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Changes: [ADR 0016](0016-ui-without-a-build-step.md) item 4 (the UI loads nothing from a third-party server). [ADR 0062](0062-a-file-picker-for-each-file-field.md) items 26 to 29 (the audio route and the media tickets). The image route of [ADR 0025](0025-studio-serves-cover-art.md).
- Uses: [ADR 0057](0057-the-private-network-guard-of-the-server.md) (the network guard) and [ADR 0105](0105-a-public-server-holds-no-assets.md) (a server that holds no assets).

## Context

- ADR 0016 item 4 permits no load from a third-party server. The page rule of `crates/api/src/ui.rs` is `default-src 'self'; img-src 'self' data:`. Thus the browser loads each image and each audio file through Studio.
- The image route reads each image from its address on the server, and keeps it in memory. The UI shows it as a `data:` address.
- The audio route reads a file from the release folder, or from its host, and sends it to the player. The player cannot send the access header, so it uses a media ticket in the address (ADR 0062 item 29).
- With `--hold-assets off` (ADR 0105), almost each play goes from the host through the server to the browser. This uses the bandwidth of the server.
- The security audit of 2026-11-01 found that a signed-in user can get a ticket for each public address (SEC-08). The server then relays each public file for that user. The ticket is also in the address, so it goes into the log of the reverse proxy.
- The operator decided on 2026-11-01 that the player and the images load from their hosts. The operator stated that hosts exist to give their files, and that most v4v apps load files from the hosts.
- A load from a host can decrease privacy. It does not decrease security. The host sees the address of the user and the time. No script, style or frame of a different server can run in the page.

## Decision

1. The UI loads images and audio from their `https` addresses. The page rule becomes `default-src 'self'; img-src 'self' data: https:; media-src 'self' https:; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`.
2. Scripts, styles, fonts and frames load only from Studio. ADR 0016 item 4 stays in force for them.
3. Each response keeps `Referrer-Policy: no-referrer`, so that a host does not see the address of the Studio server.
4. An image or an audio file on an `http` address does not load. An image shows the empty art. The player shows "This file uses http. Studio cannot play it here." Studio does not relay it.
5. A file that the server holds, and that no host has yet, loads through Studio. This is a pending file of ADR 0105 item 8, a file of ADR 0105 item 12, or a library file of the desktop. The audio route and the image route stay only for these files. A ticket and the image route accept a library file of the user by its ID, not an address.
6. The audio route no longer reads from a host. Studio relays no file from a host to the browser.
7. The core still reads a file from its host when an operation needs the bytes. Examples are the facts of an audio file, a move and a lossless conversion. The network guard applies (ADR 0057).
8. On the desktop, the page is on a loopback address with `http`. An image or audio file on `https` loads there with no change.

## Alternatives Considered

- Keep the relay, and accept only the addresses of the user's own data. Not chosen by the operator. It keeps the bandwidth of each play on the server.
- Audio from the hosts, and images through Studio. Not chosen by the operator. Images from servers that the user did not choose, such as the pictures of Nostr profiles, then stay private. The operator did not see a need for this.
- A relay for `http` addresses only. Not chosen by the operator. It keeps a part of the relay of SEC-08.

## Consequences

- The server sends no audio from the hosts. The bandwidth of a play goes from the host to the browser.
- SEC-08 closes. A ticket holds a library file of the user, so the server relays no other address. A ticket for a held file is still in the address. The runbook tells the operator to remove the query from the log of the reverse proxy.
- A host sees the address of the user when the UI shows one of its files. This also applies to each server of an image in a feed or in a Nostr profile.
- The image cache of the workspace and the `data:` addresses of the UI go out, except for held files.
- The test of the page rule changes in the same commit as the code: it accepts `https:` only in `img-src` and `media-src`.

## Acceptance Criteria

Mechanical:

- An API test: the page rule is the rule of item 1. `script-src`, `style-src` and `default-src` give only `'self'`, and the rule has no `unsafe`.
- An API test: a ticket for an address, not a file ID, is refused. A ticket for a file of a different user is refused.
- An API test: the audio route sends a held file, and refuses a file that is only on a host.
- A Node test: the source of an image and of the player is the `https` address of a file on its host. It is the Studio route for a held file, and empty for an `http` address.

Visual: an album page with covers, and a track that plays from its host. Use a server with `--hold-assets off`, at 1100 px and at 390 px with touch.

Human: the operator plays a track on `studio.musicindex.org`, and the network panel of the browser shows the request to the host, not to Studio.
