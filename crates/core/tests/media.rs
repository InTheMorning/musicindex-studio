//! Tests for the facts of an imported file, and the WAV to FLAC
//! conversion (ADR 0026, ADR 0032).
//!
//! `media/README.md` gives the commands that made the test files.

use std::path::PathBuf;

use lofty::prelude::*;
use studio_core::media::{ImportedKind, MediaErrorKind, import};

/// The path of a file in `tests/media`.
fn media_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/media")
        .join(file)
}

#[test]
fn each_tagged_audio_file_gives_its_facts() {
    let cases = [
        ("tagged.mp3", "audio/mpeg"),
        ("tagged.m4a", "audio/mp4"),
        ("tagged.flac", "audio/flac"),
        ("tagged.opus", "audio/ogg"),
    ];
    for (file, media_type) in cases {
        let imported = import(&media_path(file), file).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(imported.media_type, media_type, "{file}");
        assert_eq!(imported.file_name, file, "{file}");
        assert!(!imported.converted_from_wav, "{file}");

        let ImportedKind::Audio {
            title,
            track_number,
            duration,
        } = imported.kind
        else {
            panic!("{file} did not give audio facts");
        };
        assert_eq!(title.as_deref(), Some("Test Title"), "{file}");
        assert_eq!(track_number, Some(3), "{file}");
        assert_eq!(duration.as_deref(), Some("00:00:01"), "{file}");
    }
}

#[test]
fn a_png_file_gives_its_media_type() {
    let imported = import(&media_path("image.png"), "image.png").unwrap();
    assert_eq!(imported.media_type, "image/png");
    assert_eq!(imported.kind, ImportedKind::Image);
    assert!(!imported.converted_from_wav);
}

#[test]
fn a_text_file_gives_unsupported() {
    let error = import(&media_path("not-media.txt"), "not-media.txt").unwrap_err();
    assert_eq!(error.kind(), MediaErrorKind::Unsupported);
}

#[test]
fn a_file_with_no_title_tag_gives_the_title_from_its_name() {
    let imported = import(&media_path("untagged.mp3"), "02 - No Title Tag.mp3").unwrap();
    let ImportedKind::Audio { title, .. } = imported.kind else {
        panic!("did not give audio facts");
    };
    assert_eq!(title.as_deref(), Some("No Title Tag"));
}

#[test]
fn a_wav_file_becomes_a_flac_file_with_the_same_title_and_track_number() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("tagged.wav");
    std::fs::copy(media_path("tagged.wav"), &source).expect("copy the source file");

    let before = lofty::read_from_path(&source).expect("read the source file");
    let before_duration = before.properties().duration();

    let imported = import(&source, "tagged.wav").unwrap_or_else(|e| panic!("{e}"));
    assert!(imported.converted_from_wav);
    assert_eq!(imported.media_type, "audio/flac");
    assert_eq!(imported.file_name, "tagged.flac");
    assert_eq!(imported.path, dir.path().join("tagged.flac"));

    let ImportedKind::Audio {
        title,
        track_number,
        ..
    } = imported.kind
    else {
        panic!("did not give audio facts");
    };
    assert_eq!(title.as_deref(), Some("Test Title"));
    assert_eq!(track_number, Some(3));

    let after = lofty::read_from_path(&imported.path).expect("read the FLAC file");
    assert_eq!(after.file_type(), lofty::file::FileType::Flac);
    let after_tag = after.first_tag().expect("the FLAC file has a tag");
    assert_eq!(
        after_tag.title().map(std::borrow::Cow::into_owned),
        Some("Test Title".to_owned())
    );
    assert_eq!(after_tag.track(), Some(3));

    let after_duration = after.properties().duration();
    let diff = before_duration.as_secs_f64() - after_duration.as_secs_f64();
    assert!(diff.abs() <= 0.1, "the duration differs by {diff} s");
}
