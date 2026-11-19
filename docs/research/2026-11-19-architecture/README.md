# Studio Architecture, 2026-11-19

This record is advisory. It describes the code at commit 5f15240. It does not
make a rule. The ADRs in `docs/adr/` own each rule.

`architecture.html` has five drawings. Open it in a browser. Each drawing shows
one mechanism:

1. What Studio talks to.
2. The crates and the two shells.
3. One edit, from the editor to the host.
4. Where Studio keeps each kind of data.
5. A connected app changes a live show.

The earlier drawings in `HitIt/docs/architecture/architecture-diagrams.md` were a
list of modules. These drawings show the requests, the checks and the data.

## Evidence

Each name in the drawings is in the code. This table gives the location of
each fact.

| Fact | Location |
| --- | --- |
| The desktop shell tries port 42069 first. | `crates/desktop/src/lib.rs:21` |
| The default announcement hub is `https://hub.podcastindex.org/pubnotify`. | `crates/core/src/announce.rs:17` |
| The default Music Index API is `https://api.musicindex.org`. | `crates/core/src/musicindex.rs:17` |
| The UI saves the draft 700 ms after the last edit. | `crates/api/ui/draft.js:100` |
| The core writes the model into the draft. | `replace_model`, `crates/core/src/pipeline.rs:1681` |
| Review gets the diff and its id. | `diff_checked`, `crates/core/src/pipeline.rs:2321` |
| The page refusal comes from the feed check. | `page_refusal`, `crates/core/src/feed_check.rs:109` |
| The write checks the diff id and the hosted version. | `write_draft_signed_checked`, `crates/core/src/pipeline.rs:4546`, and `write_draft_signed_inner` at line 4582 |
| The write confirms each new file and address. | `confirm_new_addresses`, `crates/core/src/pipeline.rs:6190` |
| The write queues the announcements. | `queue`, `crates/core/src/announce.rs:228` |
| The desktop router. | `local_router`, `crates/api/src/lib.rs:132` |
| The server router. | `server_router`, `crates/server/src/auth.rs:34` |
| The server sends an app request to the router of one user. | `forward`, `crates/server/src/apps.rs:184` |
| The server allows only public addresses. | `crates/server/src/main.rs:122`, `crates/server/src/users.rs:216` |
| The desktop allows each address. | `crates/api/src/workspace.rs:986` |
| The session cookie name. | `crates/server/src/session.rs:23` |
| The vault uses XChaCha20-Poly1305 and Argon2id. | `crates/core/src/vault/crypto.rs:9`, line 59 |
| Studio copies a host secret into a grant vault. | `sync_grant_vaults`, `crates/api/src/workspace.rs:1817` |
| The checks of an app request. | `app_check` at `crates/api/src/routes.rs:7684`, `grant_hosts` at line 8061, `granted_write` at line 8221 |
| The write of a connected app. | `write_granted`, `crates/core/src/pipeline.rs:5117` |
| A grant allows only the show status, the live value and the build date. | `covered`, `crates/core/src/grant.rs:396` |

## How to change the drawings

The drawings are inline SVG in one HTML file. The file has no script and no
external file. Change the SVG in `architecture.html` directly. Keep each
drawing to one mechanism, and put a label on each arrow.
