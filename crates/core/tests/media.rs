//! Tests for the facts of an imported file, the WAV to FLAC
//! conversion, and the MP3 encoder (ADR 0026, ADR 0032, ADR 0033).
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
    // A WAV file and a FLAC file get an MP3 enclosure (ADR 0033). Each
    // file has its own test below.
    let cases = [
        ("tagged.mp3", "audio/mpeg"),
        ("tagged.m4a", "audio/mp4"),
        ("tagged.opus", "audio/ogg"),
    ];
    for (file, media_type) in cases {
        let imported = import(&media_path(file), file).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(imported.media_type, media_type, "{file}");
        assert_eq!(imported.file_name, file, "{file}");
        assert!(!imported.converted_from_wav, "{file}");
        assert!(imported.lossless.is_none(), "{file}");

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
    assert!(imported.lossless.is_none());
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
    assert_eq!(imported.media_type, "audio/mpeg");
    assert_eq!(imported.file_name, "tagged.mp3");
    assert_eq!(imported.path, dir.path().join("tagged.mp3"));

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

    let mp3 = lofty::read_from_path(&imported.path).expect("read the MP3 file");
    assert_eq!(mp3.file_type(), lofty::file::FileType::Mpeg);
    let mp3_tag = mp3.first_tag().expect("the MP3 file has a tag");
    assert_eq!(
        mp3_tag.title().map(std::borrow::Cow::into_owned),
        Some("Test Title".to_owned())
    );
    assert_eq!(mp3_tag.track(), Some(3));

    let mp3_duration = mp3.properties().duration();
    let diff = before_duration.as_secs_f64() - mp3_duration.as_secs_f64();
    assert!(diff.abs() <= 0.1, "the duration differs by {diff} s");

    let lossless = imported
        .lossless
        .expect("a WAV file gives a lossless FLAC file");
    assert_eq!(lossless.media_type, "audio/flac");
    assert_eq!(lossless.file_name, "tagged.flac");
    assert_eq!(lossless.path, dir.path().join("tagged.flac"));

    let flac = lofty::read_from_path(&lossless.path).expect("read the FLAC file");
    assert_eq!(flac.file_type(), lofty::file::FileType::Flac);
    let flac_tag = flac.first_tag().expect("the FLAC file has a tag");
    assert_eq!(
        flac_tag.title().map(std::borrow::Cow::into_owned),
        Some("Test Title".to_owned())
    );
    assert_eq!(flac_tag.track(), Some(3));
}

#[test]
fn a_flac_file_gives_an_mp3_file_with_its_tags() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("tagged.flac");
    std::fs::copy(media_path("tagged.flac"), &source).expect("copy the source file");

    let imported = import(&source, "tagged.flac").unwrap_or_else(|e| panic!("{e}"));
    assert!(!imported.converted_from_wav);
    assert_eq!(imported.media_type, "audio/mpeg");
    assert_eq!(imported.file_name, "tagged.mp3");
    assert_eq!(imported.path, dir.path().join("tagged.mp3"));

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

    let mp3 = lofty::read_from_path(&imported.path).expect("read the MP3 file");
    assert_eq!(mp3.file_type(), lofty::file::FileType::Mpeg);
    let mp3_tag = mp3.first_tag().expect("the MP3 file has a tag");
    assert_eq!(
        mp3_tag.title().map(std::borrow::Cow::into_owned),
        Some("Test Title".to_owned())
    );
    assert_eq!(mp3_tag.track(), Some(3));

    // The FLAC file stays as it is: the source file, with its size
    // (ADR 0033 item 1).
    let lossless = imported
        .lossless
        .expect("a FLAC file gives a lossless file");
    assert_eq!(lossless.media_type, "audio/flac");
    assert_eq!(lossless.path, source);
    let source_size = std::fs::metadata(&source)
        .expect("read the source metadata")
        .len();
    assert_eq!(lossless.size, source_size);
}

/// A WAV file longer than one part of the MP3 encoder gives an MP3 file
/// with the same duration. The encoder takes 65536 sample frames at a
/// time.
#[test]
fn a_long_wav_file_gives_an_mp3_file_with_the_same_duration() {
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("long.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&wav, spec).unwrap();
    for i in 0..3 * 44_100_u32 {
        // A 441 Hz tone, at a quarter of the full range.
        let phase = f64::from(i % 100) / 100.0 * std::f64::consts::TAU;
        #[allow(clippy::cast_possible_truncation)]
        let sample = (phase.sin() * 8000.0) as i16;
        writer.write_sample(sample).unwrap();
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();

    let imported = import(&wav, "long.wav").unwrap();
    let mp3 = lofty::read_from_path(&imported.path).unwrap();
    let duration = mp3.properties().duration().as_secs_f64();
    assert!(
        (duration - 3.0).abs() <= 0.1,
        "the MP3 file is {duration} s"
    );
}

/// Two imports of the same lossless file give MP3 files with the same
/// bytes, so a second import can use the file on the host (ADR 0034).
#[test]
fn two_imports_of_the_same_wav_file_give_the_same_mp3_bytes() {
    let mut mp3s = Vec::new();
    for _ in 0..3 {
        let dir = tempfile::tempdir().unwrap();
        let wav = dir.path().join("tagged.wav");
        std::fs::copy(media_path("tagged.wav"), &wav).unwrap();
        let imported = import(&wav, "tagged.wav").unwrap();
        mp3s.push(std::fs::read(&imported.path).unwrap());
    }
    assert_eq!(mp3s[0], mp3s[1]);
    assert_eq!(mp3s[1], mp3s[2]);
}
