//! Facts of an imported audio file or image file, and the WAV to FLAC
//! conversion (ADR 0026 items 1 to 4, ADR 0032 items 1 to 5).
//!
//! The core reads tags with `lofty`. It converts a WAV file to FLAC with
//! `hound` and `flacenc`. The three libraries are pure Rust.

use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flacenc::component::BitRepr;
use flacenc::error::Verify;
use lofty::config::WriteOptions;
use lofty::file::FileType;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{Tag, TagType};

/// The kind of a [`MediaError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaErrorKind {
    /// The content is not a media type that Studio accepts.
    Unsupported,
    /// A file operation or a tag operation failed.
    Io,
}

/// An error from [`import`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaError {
    kind: MediaErrorKind,
    message: String,
}

impl MediaError {
    fn new(kind: MediaErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> MediaErrorKind {
        self.kind
    }
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MediaError {}

/// The kind-specific facts of an [`Imported`] file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedKind {
    /// An audio file, with the facts from its tags.
    Audio {
        /// The title tag, or the title that the file name gives.
        title: Option<String>,
        /// The track number tag.
        track_number: Option<u32>,
        /// The duration, as `HH:MM:SS`.
        duration: Option<String>,
    },
    /// An image file.
    Image,
}

/// The facts of a file that the user selected for import (ADR 0026,
/// ADR 0032).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The name to keep for the file. A converted WAV file gets the
    /// extension `.flac`.
    pub file_name: String,
    /// The file to keep: the source file, or a new FLAC file in the same
    /// folder.
    pub path: PathBuf,
    /// The media type, such as `audio/mpeg` or `image/png`.
    pub media_type: String,
    /// The size of `path` in bytes.
    pub size: u64,
    /// The kind-specific facts.
    pub kind: ImportedKind,
    /// True when Studio converted a WAV file to FLAC.
    pub converted_from_wav: bool,
}

/// Read the facts of an audio file or an image file, and convert a WAV
/// file to FLAC with its tags (ADR 0026 items 1 to 4, ADR 0032 items 1
/// to 5).
///
/// `name` is the file name that the user selected.
///
/// # Errors
///
/// Gives [`MediaErrorKind::Unsupported`] when the content is not one of
/// the accepted media types, or when a WAV file does not have 16-bit or
/// 24-bit integer samples. Gives [`MediaErrorKind::Io`] when a file
/// operation or a tag operation fails.
pub fn import(source: &Path, name: &str) -> Result<Imported, MediaError> {
    if let Some(media_type) = sniff_image(source, name)? {
        let size = file_size(source, name)?;
        return Ok(Imported {
            file_name: name.to_owned(),
            path: source.to_owned(),
            media_type: media_type.to_owned(),
            size,
            kind: ImportedKind::Image,
            converted_from_wav: false,
        });
    }

    let probe = Probe::open(source)
        .map_err(|e| io_error(name, e))?
        .guess_file_type()
        .map_err(|e| io_error(name, e))?;
    let Some(file_type) = probe.file_type() else {
        return Err(unsupported(name));
    };

    if file_type == FileType::Wav {
        return convert_wav(source, name);
    }

    let Some(media_type) = audio_media_type(file_type) else {
        return Err(unsupported(name));
    };

    let tagged_file = probe.read().map_err(|e| io_error(name, e))?;
    let size = file_size(source, name)?;
    let tag = tagged_file.first_tag();
    let title = title_or_file_name(tag, name);
    let track_number = tag.and_then(Tag::track);
    let duration = format_duration(tagged_file.properties().duration());

    Ok(Imported {
        file_name: name.to_owned(),
        path: source.to_owned(),
        media_type: media_type.to_owned(),
        size,
        kind: ImportedKind::Audio {
            title: Some(title),
            track_number,
            duration: Some(duration),
        },
        converted_from_wav: false,
    })
}

/// Convert a WAV file to FLAC, and copy its title, artist, album and
/// track number tags into the Vorbis comments of the new file
/// (ADR 0032 item 2).
fn convert_wav(source: &Path, name: &str) -> Result<Imported, MediaError> {
    let source_tagged_file = Probe::open(source)
        .map_err(|e| io_error(name, e))?
        .read()
        .map_err(|e| io_error(name, e))?;
    let source_tag = source_tagged_file.first_tag();

    let mut reader = hound::WavReader::open(source).map_err(|e| io_error(name, e))?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int {
        return Err(MediaError::new(
            MediaErrorKind::Unsupported,
            format!("{name} has a float sample format, which Studio does not convert"),
        ));
    }
    if spec.bits_per_sample != 16 && spec.bits_per_sample != 24 {
        return Err(MediaError::new(
            MediaErrorKind::Unsupported,
            format!(
                "{name} has a {}-bit integer sample format, which Studio does not convert",
                spec.bits_per_sample
            ),
        ));
    }

    let frame_count = reader.duration();
    let samples: Vec<i32> = reader
        .samples::<i32>()
        .collect::<Result<_, hound::Error>>()
        .map_err(|e| io_error(name, e))?;

    let flac_path = source.with_extension("flac");
    encode_flac(&samples, &spec, &flac_path, name)?;

    let title = title_or_file_name(source_tag, name);
    let mut flac_tag = Tag::new(TagType::VorbisComments);
    flac_tag.set_title(title.clone());
    if let Some(artist) = source_tag.and_then(Tag::artist) {
        flac_tag.set_artist(artist.into_owned());
    }
    if let Some(album) = source_tag.and_then(Tag::album) {
        flac_tag.set_album(album.into_owned());
    }
    let track_number = source_tag.and_then(Tag::track);
    if let Some(track_number) = track_number {
        flac_tag.set_track(track_number);
    }
    flac_tag
        .save_to_path(&flac_path, WriteOptions::default())
        .map_err(|e| io_error(name, e))?;

    let size = file_size(&flac_path, name)?;
    let duration = if spec.sample_rate == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(f64::from(frame_count) / f64::from(spec.sample_rate))
    };

    Ok(Imported {
        file_name: file_name_with_extension(name, "flac"),
        path: flac_path,
        media_type: "audio/flac".to_owned(),
        size,
        kind: ImportedKind::Audio {
            title: Some(title),
            track_number,
            duration: Some(format_duration(duration)),
        },
        converted_from_wav: true,
    })
}

/// Encode `samples` to FLAC with `spec`, and write the result to `path`.
fn encode_flac(
    samples: &[i32],
    spec: &hound::WavSpec,
    path: &Path,
    name: &str,
) -> Result<(), MediaError> {
    let channels = usize::from(spec.channels);
    let bits_per_sample = usize::from(spec.bits_per_sample);
    let sample_rate = usize::try_from(spec.sample_rate).unwrap_or(usize::MAX);

    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|(_, e)| io_error(name, e))?;
    let source =
        flacenc::source::MemSource::from_samples(samples, channels, bits_per_sample, sample_rate);
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size)
        .map_err(|e| io_error(name, e))?;

    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).map_err(|e| io_error(name, e))?;
    fs::write(path, sink.as_slice()).map_err(|e| io_error(name, e))
}

/// The media type of an image, from the first bytes of `source`. Gives
/// `None` when the content does not match an accepted signature.
fn sniff_image(source: &Path, name: &str) -> Result<Option<&'static str>, MediaError> {
    let mut file = fs::File::open(source).map_err(|e| io_error(name, e))?;
    let mut head = [0u8; 12];
    let read = file.read(&mut head).map_err(|e| io_error(name, e))?;
    let head = &head[..read];

    if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(Some("image/png"));
    }
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(Some("image/jpeg"));
    }
    if head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        return Ok(Some("image/webp"));
    }
    Ok(None)
}

/// The media type that Studio accepts for an audio [`FileType`]. Gives
/// `None` for a type that Studio does not accept.
fn audio_media_type(file_type: FileType) -> Option<&'static str> {
    match file_type {
        FileType::Mpeg => Some("audio/mpeg"),
        FileType::Mp4 => Some("audio/mp4"),
        FileType::Flac => Some("audio/flac"),
        FileType::Vorbis | FileType::Opus => Some("audio/ogg"),
        _ => None,
    }
}

/// The title tag, or, with no title tag, the title from `name`.
fn title_or_file_name(tag: Option<&Tag>, name: &str) -> String {
    tag.and_then(Tag::title)
        .map(std::borrow::Cow::into_owned)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| title_from_file_name(name))
}

/// The title from a file name, with no extension and no leading track
/// number and separator, such as `01 - `.
fn title_from_file_name(name: &str) -> String {
    let stem = Path::new(name).file_stem().map_or_else(
        || name.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    );
    strip_leading_track_number(&stem).to_owned()
}

/// `text` with no leading track number and separator, such as `01 - `.
/// Gives `text` with no change when it has no such prefix.
fn strip_leading_track_number(text: &str) -> &str {
    let digits_end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    if digits_end == 0 {
        return text;
    }
    let after_digits = text[digits_end..].trim_start();
    let after_separator = after_digits
        .strip_prefix('-')
        .or_else(|| after_digits.strip_prefix('.'))
        .or_else(|| after_digits.strip_prefix('_'));
    match after_separator {
        Some(rest) => rest.trim_start(),
        None => text,
    }
}

/// `name` with its extension replaced by `extension`.
fn file_name_with_extension(name: &str, extension: &str) -> String {
    Path::new(name)
        .with_extension(extension)
        .to_string_lossy()
        .into_owned()
}

/// The duration as `HH:MM:SS`.
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// The size of the file at `path`, in bytes.
fn file_size(path: &Path, name: &str) -> Result<u64, MediaError> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|e| io_error(name, e))
}

/// A [`MediaErrorKind::Unsupported`] error that names `name`.
fn unsupported(name: &str) -> MediaError {
    MediaError::new(
        MediaErrorKind::Unsupported,
        format!("{name} is not a media type that Studio accepts"),
    )
}

/// A [`MediaErrorKind::Io`] error that names `name` and `error`.
fn io_error(name: &str, error: impl fmt::Display) -> MediaError {
    MediaError::new(MediaErrorKind::Io, format!("{name}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_leading_track_number_removes_the_number_and_the_separator() {
        assert_eq!(strip_leading_track_number("01 - Song"), "Song");
        assert_eq!(strip_leading_track_number("2. Song"), "Song");
        assert_eq!(strip_leading_track_number("3_Song"), "Song");
        assert_eq!(strip_leading_track_number("Song"), "Song");
        assert_eq!(strip_leading_track_number("42"), "42");
    }

    #[test]
    fn title_from_file_name_strips_the_extension_and_the_track_number() {
        assert_eq!(title_from_file_name("01 - Song.mp3"), "Song");
        assert_eq!(title_from_file_name("Song.mp3"), "Song");
    }

    #[test]
    fn format_duration_gives_hh_mm_ss() {
        assert_eq!(format_duration(Duration::from_secs(3_725)), "01:02:05");
    }

    #[test]
    fn sniff_image_reads_the_signature_of_each_accepted_image_type() {
        let dir = tempfile::tempdir().expect("temp dir");

        let png = dir.path().join("a.png");
        fs::write(&png, [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0]).expect("write");
        assert_eq!(sniff_image(&png, "a.png").unwrap(), Some("image/png"));

        let jpeg = dir.path().join("a.jpg");
        fs::write(&jpeg, [0xFF, 0xD8, 0xFF, 0xE0]).expect("write");
        assert_eq!(sniff_image(&jpeg, "a.jpg").unwrap(), Some("image/jpeg"));

        let webp = dir.path().join("a.webp");
        fs::write(&webp, b"RIFF....WEBPVP8 ").expect("write");
        assert_eq!(sniff_image(&webp, "a.webp").unwrap(), Some("image/webp"));

        let other = dir.path().join("a.bin");
        fs::write(&other, b"not an image").expect("write");
        assert_eq!(sniff_image(&other, "a.bin").unwrap(), None);
    }

    #[test]
    fn a_wav_file_with_float_samples_is_unsupported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("float.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&path, spec).expect("create wav");
        writer.write_sample(0.0_f32).expect("write sample");
        writer.finalize().expect("finalize wav");

        let error = convert_wav(&path, "float.wav").unwrap_err();
        assert_eq!(error.kind(), MediaErrorKind::Unsupported);
    }
}
