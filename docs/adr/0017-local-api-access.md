# ADR 0017: Access To The Local API

- Status: Accepted
- Date: 2026-10-22. Changed on 2026-10-27 to agree with ADR 0054, ADR 0059 and ADR 0062. The operator accepted it on 2026-10-27.
- Owner: operator

## Context

The desktop shell runs the API on the artist's machine ([ADR 0001](0001-one-core-with-desktop-and-server-shells.md), [ADR 0002](0002-rust-core-json-api-browser-ui.md)).
A web page that the artist opens in a browser can send requests to `127.0.0.1`. A DNS rebinding attack can also make a public host name point to `127.0.0.1`.
The local API can open the vault and write feeds, so it must accept requests only from the Studio UI.
The server shell uses sign-in sessions instead ([ADR 0004](0004-server-sign-in-with-nostr-or-email.md)). This ADR covers only the local mode.

## Decision

1. In local mode, the API listens only on a loopback address.
2. At each start, the API makes a random access token of 32 bytes.
3. The API gives the UI address with the token in the URL fragment. The browser does not send a fragment to a server or in a `Referer` header.
4. The UI keeps the token in session storage and sends it in an `Authorization: Bearer` header with each API request.
5. The API refuses each `/api/` request without the correct token, with status 401.
6. The API refuses each request with a `Host` header that is not `127.0.0.1`, `localhost` or `[::1]` with the port of the API, with status 403. This stops DNS rebinding.
7. The API accepts a request body only as `application/json`. Only one route accepts a different body. `PUT /api/v1/uploads/{upload}/parts/{n}` takes the bytes of the part ([ADR 0059](0059-uploads-in-parts-and-the-home-screen-app.md)). That route also needs the token.
8. The API sends no CORS headers.
9. The UI files themselves need no token. They contain no secret.
10. Two kinds of route are not `/api/` routes, and the token of item 2 does not open them:
    - The audio route `/media/audio` accepts a media ticket in its query. The UI gets a ticket from an `/api/` route, with the token ([ADR 0062](0062-a-file-picker-for-each-file-field.md) item 29). A media element cannot send a header.
    - The `/app/v1/` routes accept only the token of a connected app ([ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 9).

## Alternatives Considered

- A cookie. Rejected: a browser sends a cookie with a request from a different site in some conditions. A header that only the UI script adds does not have this problem.
- No protection, because the address is local. Rejected: a different web page can send requests to a local address.

## Consequences

- An artist who opens the UI in a second browser needs the address with the token again.
- The Tauri desktop app gives the token to its own window when it opens it.

## Invariants

- No `/api/` route answers in local mode without the token.
- No `/api/` route accepts the token of a connected app or a media ticket.
- No route answers a request with a `Host` that is not a loopback name.
