# Studio v1 Task 019b: The MP3 Encoder

Parent: [task 019](studio-v1-task-019-lossless-import.md). Decision: [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) items 1 to 4, 10 and 11.
Sequence: it depends on nothing. It can go at the same time as task 019a.

## Goal

An imported WAV file or FLAC file gives an MP3 file and a FLAC file, each with the tags of the source.

## Files

Inspect: `crates/core/src/media.rs`, `crates/core/Cargo.toml`, `crates/core/tests/media/` and its `README.md`, the media tests.

Change: `crates/core/Cargo.toml`, `Cargo.lock`, `media.rs`, the media tests, and `crates/core/tests/media/README.md` only if a new test file is necessary.

Do not change: `crates/api`, the UI, the feed model, `pipeline.rs`.

## Architecture

These choices are fixed. Do not change them.

- New dependencies of `studio-core`: `mp3lame-encoder` 0.2 and `claxon` 0.4. Do not add a different encoder or decoder.
- `Imported` keeps its fields. For a WAV file or a FLAC file, the fields give the MP3 file: `file_name` with the extension `.mp3`, `path`, `media_type` `audio/mpeg`, `size`. `converted_from_wav` keeps its meaning.
- `Imported` gets `pub lossless: Option<ImportedFile>`, with `pub struct ImportedFile { pub file_name: String, pub path: PathBuf, pub media_type: String, pub size: u64 }`. For a WAV file or a FLAC file, it gives the FLAC file: the converted file, or the source FLAC file. For each other file, it is `None`.
- The MP3 file and a converted FLAC file are new files in the folder of `source`. The caller removes them.
- The encoder uses LAME VBR quality V0 (`Quality::Best` for the VBR quality in `mp3lame-encoder`, or the equivalent value 0), the sample rate of the source and its channel count. A source with more than two channels gives `Unsupported`, with a message that names the channel count.
- The MP3 file gets an ID3v2 tag through `lofty` with the title, the artist, the album and the track number of the source. The title follows the rule of task 018b when the source has no title tag.
- `ImportedKind::Audio` gives the title, the track number and the duration of the source.
- The samples for the encoder come from `hound` for a WAV file, and from `claxon` for a FLAC file. 16-bit and 24-bit integer samples are accepted.

## Steps

1. Add the dependencies. Make sure that `cargo build` compiles LAME from the bundled source, with no system library.
2. Add the encoder and the `lossless` field.
3. Add the tests with the existing test files.

## Constraints

- Keep each existing test. Change only the assertions that the new MP3 file changes, such as the `file_name` of the WAV result.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The WAV test file gives an MP3 file that `lofty` reads as MP3. It has the title and the track number of the WAV file. Its duration is 0.1 s or less different from the WAV file. `lossless` gives a FLAC file that `lofty` reads with the same tags.
- The FLAC test file gives an MP3 file with its tags. `lossless` is the source FLAC file, with its size.
- The MP3, M4A and Opus test files give `lossless: None` and no new file.
- `ldd` of the test binary shows no `libmp3lame`.
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

Stop and report when `mp3lame-sys` needs a system library, or when its build fails. Stop and report when `claxon` cannot decode the FLAC test file.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-019b-mp3-encoder.md
- docs/adr/0033-a-lossless-import-gives-an-mp3-enclosure.md
- crates/core/src/media.rs
- crates/core/Cargo.toml

Goal:
- An imported WAV file or FLAC file gives an MP3 file and a FLAC file, each with the tags of the source.

Constraints:
- Follow the Architecture section of the task file fully.
- Use only mp3lame-encoder and claxon as new dependencies.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api, the UI, the feed model, pipeline.rs.
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
