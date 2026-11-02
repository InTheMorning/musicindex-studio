# Studio v1 Task 072: Audit Findings And Direct Media

Decision: [ADR 0109](../adr/0109-images-and-audio-load-from-their-hosts.md) for packet 072e. The other packets correct defects against ADR 0013 items 13 and 14, ADR 0054 item 13 and ADR 0055 items 9 and 21. They also correct defects against ADR 0057 and ADR 0066. They add no rule.

Status: Complete. Packets 072a to 072f are complete, with the screenshot visual check. Open: the human test of ADR 0109, a deploy with `tools/deploy-server.sh` that pins the digest, a CI run with the pinned actions, and a build of the server image from the pinned base image.

## Goal

Studio closes the lower findings of the security audit of 2026-11-01, and the UI loads images and audio from their `https` addresses.

## Context

The security audit of 2026-11-01 found these items. The report is not in the repository. Tasks 068, 069 and 071 close SEC-01 to SEC-06 and SEC-17. Task 073 closes SEC-09. This task closes the others.

| ID | Finding | Packet |
| --- | --- | --- |
| SEC-04 | `put_show_live_value` accepts each non-empty `uri` and `protocol`. | 072d |
| SEC-07 | `migrate::move_read_only_feed` reads with `Web::new()`, which has no network guard. | 072a |
| SEC-08 | A media ticket accepts each public address, and the audio route relays it. | 072e |
| SEC-10 | Two upload starts at the same time can both pass the storage check. | 072b |
| SEC-11 | Passphrases and recovery keys are in plain `String`. `Session` and `Access` show their token in `Debug`. | 072c |
| SEC-12 | `connector::from_row` always uses `NetPolicy::Open`. | 072a |
| SEC-13 | `net::is_global_v6` accepts `::/96` and `2002::/16`. | 072a |
| SEC-14 | Sign-up shows that an email has an account. The operator accepted this, because the server needs an invite code. | 072f |
| SEC-15 | The Rust base image, `cargo-chef` and the CI actions are not pinned, and the deploy uses a tag. | 072f |
| SEC-16 | A failed folder removal at account deletion is only logged. | 072c |

SEC-09 closes with task 073: a grant names one app ([ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md) item 19).

## Packets

### 072a: The network guard

1. `migrate::move_read_only_feed` takes the `Web` of the workspace, with its network policy (SEC-07).
2. `connector::from_row` takes the network policy as an argument. `address.rs` passes the policy of the workspace (SEC-12).
3. `net::is_global_v6` refuses `::/96` and `2002::/16`. The test list of `net.rs` gets both forms (SEC-13).
4. A core test fails when production code outside `web.rs` calls `Web::new()`. Its message names ADR 0057 and tells the author to pass the `Web` of the workspace.

### 072b: The storage limit

1. `start_upload` reserves the size of the upload under one lock of the workspace before it makes the file. `check_room` counts the files and the reservations. A reservation goes out when the upload ends, stops or expires (SEC-10).
2. An API test starts many uploads at the same time against a small limit, and finds that the sum stays below the limit.

### 072c: Secrets in memory and account deletion

1. Each request field that holds a passphrase or a recovery key is `Zeroizing<String>` (SEC-11).
2. `Session` and `Access` get a `Debug` output that shows no token. A test formats each with `{:?}` and finds no token.
3. Account deletion tries the removal of the user folder three times. When it fails, the server records the folder in `server.sqlite`, writes one log line for the operator, and tries again at each start (SEC-16).
4. `docs/runbooks/studio-server.md` says that a deletion removes the files but does not erase the disk.

### 072d: The live value

1. The core refuses some live values with 422 `invalid_live_value` (SEC-04). It refuses a `uri` longer than 2048 characters, a `protocol` longer than 64 characters, and a control character. The route of the app and the live item page use the same rule. No copy of the rule is in the UI (ADR 0065).

### 072e: Images and audio from their hosts (ADR 0109)

1. The page rule of `crates/api/src/ui.rs` becomes the rule of ADR 0109 item 1. The test of the page rule in `crates/api/src/tests.rs` accepts `https:` only in `img-src` and `media-src`, and finds `'self'` only in `default-src`.
2. A UI model function gives the source of an image or of the player:
   - the `https` address of a file on its host
   - the Studio route for a file that the server holds
   - nothing for an `http` address
   A Node test examines each case.
3. `useImage` and its callers use that source. An image of a held file keeps the path through the API with a `data:` address. The image cache of the workspace keeps only held files.
4. The player shows "This file uses http. Studio cannot play it here." for an `http` address.
5. A media ticket holds a library file ID of the user, not an address. The audio route reads only from the release folder. `Source::Host` and `from_host` go out of `media.rs`.
6. `docs/runbooks/studio-server.md` tells the operator to remove the query of `/media/audio` from the access log of nginx, with an example configuration.

### 072f: The runbook and the supply chain

1. `docs/runbooks/studio-server.md` section 5 says that open sign-up shows if an email has an account, and that the default invite mode prevents it (SEC-14).
2. The `Dockerfile` names the Rust base image by its digest, and `cargo install cargo-chef` names an exact version. Find the digest with `docker buildx imagetools inspect` at the time of the task.
3. `.github/workflows/ci.yml` and `desktop-release.yml` name each action by its commit SHA, with the tag in a comment, as `server-image.yml` does.
4. `tools/deploy-server.sh` reads the digest of the pulled image, and writes the image with its digest into `.env`. It then makes sure that the running container uses that digest (SEC-15).

## Files

Change: `crates/core/src/migrate.rs`, `crates/core/src/connector.rs`, `crates/core/src/address.rs`, `crates/core/src/net.rs`, `crates/core/src/pipeline.rs`, `crates/api/src/uploads.rs`, `crates/api/src/routes.rs`, `crates/api/src/access.rs`, `crates/api/src/media.rs`, `crates/api/src/ui.rs`, `crates/api/src/workspace.rs`, `crates/api/src/tests.rs`, `crates/server/src/auth.rs`, `crates/server/src/session.rs`, `crates/server/src/accounts.rs`, `crates/api/ui/app.js`, `crates/api/ui/editor.js`, a UI model module and its Node test, `Dockerfile`, `.github/workflows/ci.yml`, `.github/workflows/desktop-release.yml`, `tools/deploy-server.sh`, `docs/runbooks/studio-server.md`.

## Constraints

- No test sends a request to a host outside the computer.
- No feed rule in the UI (ADR 0065).
- Scripts, styles, fonts and frames load only from Studio (ADR 0109 item 2).
- When a new test fails, its message names the owning ADR and the fix.

## Acceptance Criteria

Mechanical:

- The tests of the packets above. Each fails when its fix is removed.
- The tests of the acceptance criteria of ADR 0109.
- The four commit checks pass.
- CI passes with the pinned actions, and the server image builds from the pinned base image.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch: an album page with covers from their hosts. Also a track that plays from its host, and the http notice of the player. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the human test of ADR 0109, and a deploy with `tools/deploy-server.sh` that pins the digest.
