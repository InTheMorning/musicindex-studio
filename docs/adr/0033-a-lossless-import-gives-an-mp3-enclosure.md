# ADR 0033: A Lossless Import Gives An MP3 Enclosure

- Status: Accepted
- Date: 2026-10-23
- Owner: operator

## Context

[ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) converted an imported WAV file to FLAC, and made the FLAC file the enclosure of the track. A notice told the user that some listener apps do not play FLAC.

On 2026-10-23 the operator asked for research: which apps play FLAC, and which apps play WAV. The operator does not include Apple Podcasts, because it has no Podcast Index music. The apps that are important are modern Podcasting 2.0 apps, such as Fountain, Podcast Guru and Castamatic. The research found these facts:

- The platform decoders play FLAC: iOS 11 and each subsequent version, Android, and the browsers of MDN.
- No public source says that each of these apps plays a FLAC enclosure.
- Two Fountain feeds in the corpus have WAV enclosures. No feed in the corpus has a FLAC enclosure.
- The Podcast Namespace tag `podcast:alternateEnclosure` gives other versions of the enclosure. "The standard RSS enclosure element is always the default media to be played." An app that does not know the tag plays the enclosure.

The operator decided that Studio makes an MP3 file for the enclosure, and offers the lossless file as an alternative enclosure.

## Decision

### Files

1. An imported WAV file or FLAC file gives two files: a FLAC file and an MP3 file. A WAV file becomes FLAC as before. An imported FLAC file stays as it is.
2. Studio encodes the MP3 file with LAME, at the variable bitrate quality V0. The MP3 file gets an ID3v2 tag with four values of the source: the title, the artist, the album and the track number.
3. The two files have the name of the source file with the extensions `.mp3` and `.flac`. They go to the release folder of [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 4.
4. An imported MP3, M4A or Ogg file gives one file, as before.

### The Track

5. The `enclosure` of the track is the MP3 file.
6. The item gets two `podcast:alternateEnclosure` elements after its `enclosure`:
   - The MP3 file, with `type="audio/mpeg"`, its `length`, `title="Standard (MP3)"` and `default="true"`.
   - The FLAC file, with `type="audio/flac"`, its `length` and `title="Maximum fidelity (lossless)"`.

   The operator changed the two titles on 2026-10-23, after the check of task 019. Listener apps show the title in their choice of files. A track that Studio wrote before has the titles `MP3` and `Lossless (FLAC)`.

   Each has one `podcast:source` with the public address of the file.
7. The diff lists the two files. A pending file is in use when it is the address of an enclosure, of an image, or of a `podcast:source` (ADR 0032 item 10).
8. The track page of the editor shows each alternative enclosure as one summary line, with its title ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). The user can remove an alternative enclosure.

### The Notice

9. The import sheet shows a notice for each WAV file and each FLAC file. The notice does not use the `--error` color.
   - For a WAV file: "Studio converts this WAV file to MP3 and FLAC. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file."
   - For a FLAC file: "Studio makes an MP3 file from this FLAC file. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file."

### The Libraries

10. Studio encodes MP3 with the crate `mp3lame-encoder`, which uses `mp3lame-sys`. That crate compiles LAME 3.100 from its bundled source at build time, as `rusqlite` compiles SQLite. Studio needs no library at run time. The build needs a C compiler and, on Unix, `make` and `sh`. LAME has the LGPL license, which agrees with AGPL-3.0 ([ADR 0011](0011-agpl-license.md)).
11. Studio decodes a FLAC file for the MP3 encoder with a pure Rust decoder.

## Alternatives Considered

- FLAC as the enclosure, with no MP3 (ADR 0032). Not chosen by the operator: some modern apps possibly do not play FLAC.
- FLAC as the enclosure, with an MP3 alternative enclosure. Rejected: an app that does not know the tag plays the FLAC file.
- The pure Rust encoder `shine-rs`. Not chosen by the operator: Shine is a fixed-point encoder for weak hardware, and its quality is lower than LAME.
- A constant bitrate of 320 kbit/s or 192 kbit/s. Not chosen by the operator.
- An MP3 file for WAV imports only. Not chosen by the operator: an imported FLAC file then stays the only enclosure.
- A test of each app before the decision. Not chosen by the operator: each app plays the MP3 enclosure, also an app that does not play FLAC.

## Consequences

- Each lossless track uses the space of a FLAC file and an MP3 file on the host.
- An import of a lossless file takes more time, because Studio encodes the MP3 file.
- The build of `studio-core` needs a C toolchain. The Desktop Shell phase must make sure of the build of `mp3lame-sys` for Windows.
- The feed model gets `podcast:alternateEnclosure`. Published feeds have it with other attributes, such as `height` and `timeOffset`, and the read and the write keep them.

## Invariants

- No WAV file goes to a host.
- Each track from a lossless import has an MP3 enclosure.
- A `podcast:alternateEnclosure` is only in an item that has an `enclosure`.
