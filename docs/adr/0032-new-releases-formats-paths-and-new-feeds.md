# ADR 0032: New Releases: Formats, File Paths And New Feeds

- Status: Accepted. [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md) replaces the WAV conversion of items 2 and 5 on 2026-10-23. [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) replaces items 4 and 10 to 12 when Phase 7 implements it.
- Date: 2026-10-23
- Owner: operator
- Amended: 2026-10-26 by operator decision. [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 18 adds one exception to the invariant: a change that a show grant of the user covers.

## Context

Phase 6 of the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) makes new releases. [ADR 0025](0025-studio-serves-cover-art.md) leaves the size limit and the cache time of an image to its task. [ADR 0026](0026-import-music-files-through-the-draft.md) leaves the audio formats and the tag library to its task. No ADR says how Studio makes a new feed file.

On 2026-10-23 the operator answered three questions about the formats, the file paths and the fields of a new feed. The planning model added the technical choices below. Each of these follows an existing rule.

## Decision

### Audio Formats

1. The import accepts MP3, M4A (AAC), FLAC and Ogg (Vorbis and Opus) files. For art, it accepts JPEG, PNG, WebP and GIF images. A GIF image shows a notice that Apple Podcasts asks for JPEG or PNG art. The operator added GIF on 2026-10-23, in the visual check of Phase 6, because published music feeds use GIF art.
2. The import also accepts a WAV file. [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md) gives the files that Studio makes from a WAV file or a FLAC file, and the notice that the user sees.
3. Studio reads the tags with the `lofty` library. It converts WAV to FLAC with the `hound` and `flacenc` libraries. The three libraries are pure Rust. ADR 0033 adds the MP3 encoder.

### File Paths

4. The imported files of a release go to a folder adjacent to the feed file. The folder has the name of the feed file with no extension: `my-album.xml` gives `my-album/`.
5. Each file keeps its name. ADR 0033 gives the names of the files that Studio makes from a lossless file.

### A New Feed

6. To make a new album feed, the user gives a connected folder and the path of the feed file in that folder. The user also gives the album title and the artist.
7. Studio sets `podcast:guid` from the public address of the feed ([ADR 0008](0008-feed-types-and-feed-identity.md) item 2), `podcast:medium` to `music`, and `language` to `en`. The user changes the other fields in the editor.
8. A new feed needs a folder with a public address, because the GUID and the file addresses come from it ([ADR 0021](0021-public-address-of-a-feed.md)). Studio refuses a path where a file exists.
9. Studio makes the new feed as a draft with no hosted file. The user examines the diff of the full file and writes it. The write puts the file only when no file exists at the path ([ADR 0007](0007-draft-diff-write-and-announce.md)).

### Pending Files

10. A pending file goes to the host only when the draft refers to it, as the address of an enclosure or of an image. The diff lists only these files. The ID of the diff includes the path, the size and the SHA-256 of each file.
11. The write puts each file on the host before the feed file. After each file is on the host, Studio removes it from the pending files. A new write after a stop puts only the remaining files.
12. A discarded draft, and a version put in the draft, remove the pending files of the feed.

### Images

13. The core reads an image of a maximum of 10 MiB, with an `image/` media type. Studio keeps each image in memory until it stops, or until the user reads the feed again. It keeps at most 64 images.
14. The UI gets the image bytes from the API with the access token, and shows them as a `data:` address. The page policy of [ADR 0016](0016-ui-without-a-build-step.md) does not change.

## Alternatives Considered

- MP3 and M4A only. Not chosen by the operator: artists who publish FLAC or Ogg could not import.
- Refuse WAV. Not chosen by the operator: artists frequently have WAV masters.
- The files adjacent to the feed file, or with names from the track numbers. Not chosen by the operator.
- Start a new feed from the import, with the title and the artist from the tags. Not chosen by the operator.
- Write a new feed file immediately, with no diff. Rejected: ADR 0007 requires a diff that the user saw before each write.
- Add `blob:` to the page policy for images. Rejected: a `data:` address needs no change of the policy.

## Consequences

- The core gets three new dependencies.
- The data folder keeps the pending files of each feed until the write or the discard of the draft.

## Invariants

- No file and no feed file goes to a host without a diff that the user saw. The only exception is a change that a show grant of the user covers ([ADR 0054](0054-live-items-connected-apps-and-show-grants.md)).
- No WAV file goes to a host (ADR 0033).
