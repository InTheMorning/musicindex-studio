# Studio v1 Task 018b: The Tags And The WAV Conversion

Parent: [task 018](studio-v1-task-018-import-music-files.md). Decisions: [ADR 0026](../adr/0026-import-music-files-through-the-draft.md) items 1 to 4, and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) items 1 to 5.
Sequence: it depends on nothing. It can run with task 016.

## Goal

Read the facts of an imported audio file or image file. Convert a WAV file to FLAC, with its tags.

## Files

Inspect: `crates/core/src/lib.rs`, `crates/core/Cargo.toml`, `crates/core/src/web.rs` (`MediaFacts`, as a model of a small result type).

Change: `crates/core/Cargo.toml`, `Cargo.lock`, a new `crates/core/src/media.rs`, `lib.rs`, and new test files in `crates/core/tests/media/`.

Do not change the other files.

## Architecture

These choices are fixed. Do not change them.

- Dependencies of `studio-core`: `lofty` 0.25, `hound` 3.5 and `flacenc` 0.5 with no default features.
- `media::import(source: &Path, name: &str) -> Result<Imported, MediaError>`. `name` is the file name that the user selected.
- `pub struct Imported { pub file_name: String, pub path: PathBuf, pub media_type: String, pub size: u64, pub kind: ImportedKind, pub converted_from_wav: bool }`. `path` is the file to keep: the source, or a new FLAC file in the same folder.
- `pub enum ImportedKind { Audio { title: Option<String>, track_number: Option<u32>, duration: Option<String> }, Image }`. `duration` is `HH:MM:SS`.
- The type comes from the content, with `lofty` for audio. The accepted media types are `audio/mpeg`, `audio/mp4`, `audio/flac`, `audio/ogg`, `image/jpeg`, `image/png`, `image/webp` and `image/gif` (GIF added on 2026-10-23, ADR 0032 item 1). An M4A file gives `audio/mp4`. An Ogg Opus file gives `audio/ogg`.
- Other content gives a `MediaError` of the type `Unsupported`, with the name of the file.
- A WAV file (RIFF WAVE, 16-bit or 24-bit integer samples) becomes FLAC with `flacenc`. Studio reads the samples with `hound`. With `lofty`, it copies four tags of the WAV file into the Vorbis comments of the FLAC file: the title, the artist, the album and the track number. `file_name` gets the extension `.flac`. `converted_from_wav` is `true`.
- A WAV file with float samples gives `Unsupported`, with a message that names the sample format.
- With no title tag, the title is the file name with no extension and no leading track number and separator, such as `01 - `.

## Steps

1. Add the dependencies and `media.rs`.
2. Make the test files with `ffmpeg` and `lame`: a tagged MP3, M4A, FLAC and Ogg Opus file, each of 1 second of silence, and a tagged 16-bit WAV file. Make a 1 × 1 PNG. Keep each file below 30 KB. Put them in `crates/core/tests/media/`, with a `README.md` that gives the commands.
3. Add the tests.

## Constraints

- Keep each existing test.
- The libraries are pure Rust. Do not add a library that needs a C library or a system program at run time.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- Each test file gives its media type, its title, its track number and its duration.
- The WAV file gives a FLAC file that `lofty` reads, with the same title and track number. Its duration is 0.1 s or less different from the WAV file.
- A text file gives `Unsupported`.
- A file with no title tag gives the title from its name.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when a library needs a C library, or when `flacenc` cannot encode 24-bit samples.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-018b-tags-and-conversion.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- crates/core/src/lib.rs
- crates/core/Cargo.toml

Goal:
- Read the facts of an imported audio or image file, and convert a WAV file to FLAC with its tags.

Constraints:
- Follow the Architecture section of the task file fully.
- Pure Rust libraries only. Test files below 30 KB each.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The files that the Files section does not name.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
