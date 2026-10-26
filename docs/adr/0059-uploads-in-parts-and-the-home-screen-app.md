# ADR 0059: Uploads In Parts And The Home Screen App

- Status: Accepted
- Date: 2026-10-26
- Owner: operator

## Context

The goal of the operator for Phase 12 is a change of a feed from a phone ([ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).

An import sends each music file to `POST /api/v1/feeds/{id}/files` in one request, with a limit of 2 GiB ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)). A WAV file or a FLAC file can be hundreds of MiB. On a mobile network, one request of that size often fails, and the user must then send the full file again. A reverse proxy also limits the size of a request body. The PodSovereign server sends uploads in parts of 8 MB for the same reason (`research_notes/studio/podsovereign-comparison.md` in `musicindex-research`).

A phone user opens a web app from the home screen more easily than from a browser address. A web app manifest gives that. A service worker is not necessary for it.

On 2026-10-26 the operator accepted Phase 12 with these functions in milestone 12b.

## Decision

### Uploads In Parts

1. An upload has these routes, in both shells:
   - `POST /api/v1/feeds/{id}/uploads` with `{"name", "size", "replace"}` starts an upload. It gives `{"upload", "part_size"}`. The part size is 8 MiB.
   - `PUT /api/v1/uploads/{upload}/parts/{n}` sends part `n`, from 0. Each part except the last has the part size. The body limit of this route is the part size.
   - `GET /api/v1/uploads/{upload}` gives the size and the numbers of the parts that Studio has.
   - `POST /api/v1/uploads/{upload}/finish` does the import of [ADR 0026](0026-import-music-files-through-the-draft.md). Its response is the response of the import route before this ADR.
   - `DELETE /api/v1/uploads/{upload}` stops an upload and removes its parts.
2. Studio writes each part at its position in one file in `incoming/`. A part that comes again replaces the same bytes.
3. `finish` refuses an upload with a missing part, with 409 `upload_incomplete`. The size limit of a file stays 2 GiB.
4. An upload with no new part for 24 hours expires. Studio removes its file at the start and each hour.
5. The parts count against the storage limit of ADR 0055 item 21.
6. The UI sends each file in parts, in both shells. After a failed part, it tries again 3 times. After a closed page, the user selects the same file again. The UI then finds the upload by the feed, the name and the size, and continues from the first missing part.
7. The route `POST /api/v1/feeds/{id}/files` for a full upload goes away in the same change. The routes that list and remove a pending file do not change.

### The Home Screen App

8. Studio serves `/manifest.webmanifest` with the name "Music Index Studio", the short name "Studio", `start_url` `/`, `display` `standalone` and the theme colors of the dark theme.
9. The icons are PNG files of 192 px and 512 px, and a maskable icon, in `crates/api/ui/assets/`. They come from the brand files of `musicindex-assets` (musicindex ADR 0022).
10. `index.html` links the manifest and an `apple-touch-icon`. The manifest and the icons need no session.
11. Studio has no service worker. A page with no network shows the error of the API, as now.

## Alternatives Considered

- The tus protocol. Rejected: it needs a client library in the UI or code of the same size, and Studio needs only one client.
- Keep the full upload for the desktop. Rejected: two paths for one function. The parts work on a loopback address too.
- A service worker for work with no network. Rejected: Studio needs the host for each write, and a cache adds a source of old data.

## Consequences

- A phone can send a large file over an unstable network.
- The reverse proxy needs a body limit of only a little more than 8 MiB.
- The phone user adds Studio to the home screen and opens it like an app.

## Verification

Mechanical:

- An API test sends a file in parts in a random sequence, sends one part two times, and finishes. The imported file has the same SHA-256 as the source.
- A test refuses `finish` with a missing part, and a part larger than the part size.
- A test confirms that an expired upload has no file.
- A test confirms that `/manifest.webmanifest` and the icons load with no token and no session.
- The Node tests examine the part plan of the UI model: the parts to send after a list of received parts.

Visual:

- The import sheet during an upload in parts, and after a retry. Each one in the dark theme, in the light theme and at 390 px.

Human test:

- A phone on a mobile network sends a WAV file of more than 200 MiB. The network changes one time during the upload.
- "Add to Home Screen" on an iPhone and on an Android phone opens Studio with no browser bar.

## Invariants

- An import uses the bytes that the user sent, in their positions. A missing part stops the import.
