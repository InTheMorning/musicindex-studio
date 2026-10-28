//! Facts of an imported audio file or image file, the WAV to FLAC
//! conversion, and the MP3 encoder (ADR 0026 items 1 to 4,
//! ADR 0032 items 1 to 5, ADR 0033 items 1 to 4, 10 and 11).
//!
//! The core reads tags with `lofty`. It converts a WAV file to FLAC
//! with `hound` and `flacenc`. It encodes a WAV file or a FLAC file to
//! MP3 with `mp3lame-encoder`, and decodes a FLAC file for the encoder
//! with `claxon`. `mp3lame-encoder` compiles LAME from its bundled
//! source at build time. Each other library is pure Rust.

use std::borrow::Cow;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flacenc::component::BitRepr;
use flacenc::error::Verify;
use lofty::config::WriteOptions;
use lofty::file::FileType;
use lofty::id3::v2::Id3v2Tag;
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
    /// A lyrics file. Studio reads no tags from it.
    Lyrics,
}

/// The facts of a file that the user selected for import (ADR 0026,
/// ADR 0032, ADR 0033).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The name to keep for the file. A WAV file or a FLAC file gets
    /// the extension `.mp3`, because the MP3 file is the enclosure of
    /// the track (ADR 0033 items 1 and 5).
    pub file_name: String,
    /// The file to keep: the source file, or a new MP3 file in the
    /// same folder.
    pub path: PathBuf,
    /// The media type, such as `audio/mpeg` or `image/png`.
    pub media_type: String,
    /// The size of `path` in bytes.
    pub size: u64,
    /// The kind-specific facts.
    pub kind: ImportedKind,
    /// True when Studio converted a WAV file to FLAC.
    pub converted_from_wav: bool,
    /// The lossless file of a WAV file or a FLAC file: the converted
    /// FLAC file, or the source FLAC file. `None` for each other file
    /// (ADR 0033 item 1).
    pub lossless: Option<ImportedFile>,
}

/// The facts of a file that goes with the main file of an [`Imported`]
/// result (ADR 0033 item 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFile {
    /// The name to keep for the file.
    pub file_name: String,
    /// The file to keep.
    pub path: PathBuf,
    /// The media type, such as `audio/flac`.
    pub media_type: String,
    /// The size of `path` in bytes.
    pub size: u64,
}

/// Read the facts of an audio file, an image file or a lyrics file. Give a
/// WAV file or a FLAC file an MP3 enclosure, with its tags (ADR 0026 items
/// 1 to 4, ADR 0032 items 1 to 5, ADR 0033 items 1 to 4, 10 and 11).
///
/// `name` is the file name that the user selected.
///
/// # Errors
///
/// Gives [`MediaErrorKind::Unsupported`] when the content is not one of
/// the accepted media types, or when a lyrics file is not valid UTF-8.
/// Gives it also when a WAV file or a FLAC file does not have 16-bit or
/// 24-bit integer samples, or when it has more than two channels. Gives [`MediaErrorKind::Io`] when a file operation or a
/// tag operation fails.
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
            lossless: None,
        });
    }

    let probe = Probe::open(source)
        .map_err(|e| io_error(name, e))?
        .guess_file_type()
        .map_err(|e| io_error(name, e))?;
    let Some(file_type) = probe.file_type() else {
        // An audio file or an image file has its result before this
        // point. A file with a lyrics extension is a lyrics file when its
        // content is valid UTF-8 (ADR 0064 item 9).
        if kind_of_path(name) == Some(FileKind::Lyrics) {
            return import_lyrics(source, name);
        }
        return Err(unsupported(name));
    };

    if file_type == FileType::Wav {
        return convert_wav(source, name);
    }
    if file_type == FileType::Flac {
        return convert_flac(source, name);
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
        lossless: None,
    })
}

/// Read the facts of a lyrics file. The content must be valid UTF-8.
fn import_lyrics(source: &Path, name: &str) -> Result<Imported, MediaError> {
    let bytes = fs::read(source).map_err(|e| io_error(name, e))?;
    if std::str::from_utf8(&bytes).is_err() {
        return Err(unsupported(name));
    }
    let media_type = crate::remote::media_type_of_name(name).ok_or_else(|| unsupported(name))?;
    Ok(Imported {
        file_name: name.to_owned(),
        path: source.to_owned(),
        media_type,
        size: bytes.len() as u64,
        kind: ImportedKind::Lyrics,
        converted_from_wav: false,
        lossless: None,
    })
}

/// Convert a WAV file to FLAC and to MP3, and copy its title, artist,
/// album and track number tags into the new files. The MP3 file is the
/// main result. The FLAC file is `lossless` (ADR 0032 item 2,
/// ADR 0033 items 1 to 4, 10 and 11).
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
    let artist = source_tag
        .and_then(Tag::artist)
        .map(std::borrow::Cow::into_owned);
    let album = source_tag
        .and_then(Tag::album)
        .map(std::borrow::Cow::into_owned);
    let track_number = source_tag.and_then(Tag::track);

    let mut flac_tag = Tag::new(TagType::VorbisComments);
    flac_tag.set_title(title.clone());
    if let Some(artist) = artist.clone() {
        flac_tag.set_artist(artist);
    }
    if let Some(album) = album.clone() {
        flac_tag.set_album(album);
    }
    if let Some(track_number) = track_number {
        flac_tag.set_track(track_number);
    }
    flac_tag
        .save_to_path(&flac_path, WriteOptions::default())
        .map_err(|e| io_error(name, e))?;
    let flac_size = file_size(&flac_path, name)?;

    let duration = if spec.sample_rate == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(f64::from(frame_count) / f64::from(spec.sample_rate))
    };

    let mp3_path = source.with_extension("mp3");
    encode_mp3(
        &samples,
        u32::from(spec.channels),
        spec.bits_per_sample,
        spec.sample_rate,
        &mp3_path,
        name,
    )?;
    write_mp3_tag(&mp3_path, &title, artist, album, track_number, name)?;
    let mp3_size = file_size(&mp3_path, name)?;

    Ok(Imported {
        file_name: file_name_with_extension(name, "mp3"),
        path: mp3_path,
        media_type: "audio/mpeg".to_owned(),
        size: mp3_size,
        kind: ImportedKind::Audio {
            title: Some(title),
            track_number,
            duration: Some(format_duration(duration)),
        },
        converted_from_wav: true,
        lossless: Some(ImportedFile {
            file_name: file_name_with_extension(name, "flac"),
            path: flac_path,
            media_type: "audio/flac".to_owned(),
            size: flac_size,
        }),
    })
}

/// Give a FLAC file an MP3 enclosure, with its tags. The FLAC file
/// stays as it is, as `lossless` (ADR 0033 items 1 to 4, 10 and 11).
fn convert_flac(source: &Path, name: &str) -> Result<Imported, MediaError> {
    let tagged_file = Probe::open(source)
        .map_err(|e| io_error(name, e))?
        .read()
        .map_err(|e| io_error(name, e))?;
    let tag = tagged_file.first_tag();
    let title = title_or_file_name(tag, name);
    let artist = tag.and_then(Tag::artist).map(std::borrow::Cow::into_owned);
    let album = tag.and_then(Tag::album).map(std::borrow::Cow::into_owned);
    let track_number = tag.and_then(Tag::track);
    let duration = format_duration(tagged_file.properties().duration());

    let mp3_path = source.with_extension("mp3");
    encode_mp3_from_flac(source, &mp3_path, name)?;
    write_mp3_tag(&mp3_path, &title, artist, album, track_number, name)?;
    let mp3_size = file_size(&mp3_path, name)?;
    let flac_size = file_size(source, name)?;

    Ok(Imported {
        file_name: file_name_with_extension(name, "mp3"),
        path: mp3_path,
        media_type: "audio/mpeg".to_owned(),
        size: mp3_size,
        kind: ImportedKind::Audio {
            title: Some(title),
            track_number,
            duration: Some(duration),
        },
        converted_from_wav: false,
        lossless: Some(ImportedFile {
            file_name: name.to_owned(),
            path: source.to_owned(),
            media_type: "audio/flac".to_owned(),
            size: flac_size,
        }),
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

/// Decode the FLAC file at `source` with `claxon`, and encode its
/// samples to MP3 at `mp3_path` (ADR 0033 items 10 and 11).
fn encode_mp3_from_flac(source: &Path, mp3_path: &Path, name: &str) -> Result<(), MediaError> {
    let mut reader = claxon::FlacReader::open(source).map_err(|e| io_error(name, e))?;
    let info = reader.streaminfo();

    let bits_per_sample = u16::try_from(info.bits_per_sample).unwrap_or(u16::MAX);
    if bits_per_sample != 16 && bits_per_sample != 24 {
        return Err(MediaError::new(
            MediaErrorKind::Unsupported,
            format!(
                "{name} has a {bits_per_sample}-bit integer sample format, which Studio does \
                 not encode"
            ),
        ));
    }

    let samples: Vec<i32> = reader
        .samples()
        .collect::<Result<_, claxon::Error>>()
        .map_err(|e| io_error(name, e))?;

    encode_mp3(
        &samples,
        info.channels,
        bits_per_sample,
        info.sample_rate,
        mp3_path,
        name,
    )
}

/// Encode `samples` to MP3 with LAME, at VBR quality V0, and write the
/// file to `path`. `samples` holds one integer sample of
/// `bits_per_sample` bits per channel per frame, channel interleaved
/// (ADR 0033 items 2 and 10).
fn encode_mp3(
    samples: &[i32],
    channels: u32,
    bits_per_sample: u16,
    sample_rate: u32,
    path: &Path,
    name: &str,
) -> Result<(), MediaError> {
    check_channel_count(channels, name)?;
    // `check_channel_count` already refused a channel count above 2.
    let channel_count = u8::try_from(channels).unwrap_or(2);

    let mut builder = mp3lame_encoder::Builder::new()
        .ok_or_else(|| io_error(name, "cannot create the LAME encoder"))?;
    builder
        .set_num_channels(channel_count)
        .map_err(|e| io_error(name, e))?;
    builder
        .set_sample_rate(sample_rate)
        .map_err(|e| io_error(name, e))?;
    builder
        .set_vbr_mode(mp3lame_encoder::VbrMode::Mtrh)
        .map_err(|e| io_error(name, e))?;
    builder
        .set_vbr_quality(mp3lame_encoder::Quality::Best)
        .map_err(|e| io_error(name, e))?;
    let mut encoder = builder.build().map_err(|e| io_error(name, e))?;

    // LAME takes each integer sample at the full 32-bit range. `samples`
    // holds each sample in its low `bits_per_sample` bits, so the loop
    // shifts it to the top of the 32-bit range.
    let shift = 32 - u32::from(bits_per_sample);
    let mut mp3 = Vec::new();
    // Encode a part of the samples at a time, so that a long file needs
    // no second copy of all its samples in memory.
    let mut left = Vec::with_capacity(ENCODE_FRAMES);
    let mut right = Vec::with_capacity(ENCODE_FRAMES);
    let chunk = ENCODE_FRAMES * usize::from(channel_count);
    for part in samples.chunks(chunk) {
        left.clear();
        right.clear();
        if channel_count == 2 {
            for frame in part.as_chunks::<2>().0 {
                left.push(frame[0] << shift);
                right.push(frame[1] << shift);
            }
        } else {
            left.extend(part.iter().map(|&sample| sample << shift));
        }
        mp3.reserve(mp3lame_encoder::max_required_buffer_size(left.len()));
        let written = if channel_count == 2 {
            encoder.encode_to_vec(
                mp3lame_encoder::DualPcm {
                    left: &left,
                    right: &right,
                },
                &mut mp3,
            )
        } else {
            encoder.encode_to_vec(mp3lame_encoder::MonoPcm(&left), &mut mp3)
        };
        written.map_err(|e| io_error(name, e))?;
    }

    // The final flush needs its own buffer, at least 7200 bytes.
    mp3.reserve(7200);
    encoder
        .flush_to_vec::<mp3lame_encoder::FlushNoGap>(&mut mp3)
        .map_err(|e| io_error(name, e))?;

    // LAME reserves the first frame of `mp3` for the VBR header, but
    // leaves it empty until the whole file is encoded. A reader needs
    // this header for the correct duration of a VBR file. The header
    // goes after any ID3v2 tag that LAME wrote; `studio-core` writes no
    // such tag, so `id3v2_tag_size` is 0, and the header is the first
    // frame of `mp3`.
    let id3v2_size = encoder.id3v2_tag_size();
    let lame_tag_size = encoder.lame_tag_size();
    if lame_tag_size > 0 {
        // The buffer needs its full size before the encoder writes to
        // it: an empty buffer gives no space, so the encoder would
        // refuse it.
        let mut vbr_header = Vec::with_capacity(lame_tag_size);
        encoder.lame_tag_encode_to_vec(&mut vbr_header);
        mp3[id3v2_size..id3v2_size + lame_tag_size].copy_from_slice(&vbr_header);
    }

    fs::write(path, &mp3).map_err(|e| io_error(name, e))
}

/// The number of sample frames that the MP3 encoder takes at a time.
const ENCODE_FRAMES: usize = 1 << 16;

/// Gives `Unsupported`, with `channels` in the message, when `channels`
/// is more than two. LAME encodes mono and stereo sound only
/// (ADR 0033 item 10).
fn check_channel_count(channels: u32, name: &str) -> Result<(), MediaError> {
    if channels > 2 {
        return Err(MediaError::new(
            MediaErrorKind::Unsupported,
            format!("{name} has {channels} channels, which Studio does not encode to MP3"),
        ));
    }
    Ok(())
}

/// Write an `ID3v2` tag to the MP3 file at `path`, with the title, the
/// artist, the album and the track number of the source (ADR 0033
/// item 2).
fn write_mp3_tag(
    path: &Path,
    title: &str,
    artist: Option<String>,
    album: Option<String>,
    track_number: Option<u32>,
    name: &str,
) -> Result<(), MediaError> {
    // An `Id3v2Tag` keeps its frames in the sequence of the calls below.
    // A general `Tag` gives the frames in a random sequence when it
    // converts to ID3v2, so two imports of the same file then give
    // different bytes (ADR 0034).
    let mut tag = Id3v2Tag::new();
    tag.set_title(title.to_owned());
    if let Some(artist) = artist {
        tag.set_artist(artist);
    }
    if let Some(album) = album {
        tag.set_album(album);
    }
    if let Some(track_number) = track_number {
        tag.set_track(track_number);
    }
    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| io_error(name, e))
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
    // Published music feeds use GIF art (ADR 0032 item 1).
    if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        return Ok(Some("image/gif"));
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

/// The kind of a file that a file field accepts (ADR 0062 item 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// An image of the formats of ADR 0032 item 1.
    Image,
    /// An audio file of the formats of ADR 0032 item 1.
    Audio,
    /// A lyrics file: `.srt`, `.vtt`, `.txt`, `.json` or `.html`
    /// (ADR 0064 item 9).
    Lyrics,
}

impl FileKind {
    /// Read the name that the API uses: `image`, `audio` or `lyrics`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "image" => Some(Self::Image),
            "audio" => Some(Self::Audio),
            "lyrics" => Some(Self::Lyrics),
            _ => None,
        }
    }
}

/// The kind of the file at `path`, from its extension. The comparison
/// ignores case. The result is `None` for any other extension (ADR 0032
/// item 1, ADR 0062 item 3, ADR 0064 item 9).
#[must_use]
pub fn kind_of_path(path: &str) -> Option<FileKind> {
    let (_, extension) = path.rsplit_once('.')?;
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(FileKind::Image),
        "mp3" | "m4a" | "flac" | "ogg" | "opus" => Some(FileKind::Audio),
        "srt" | "vtt" | "txt" | "json" | "html" => Some(FileKind::Lyrics),
        _ => None,
    }
}

/// The start of an address with the OP3 prefix (ADR 0076).
const OP3_PREFIX: &str = "https://op3.dev/e";

/// The length of the OP3 prefix in `url`, with its `pg=` part and its
/// closing `/`. Gives `None` when `url` has no OP3 prefix.
fn op3_prefix_len(url: &str) -> Option<usize> {
    let rest = url.strip_prefix(OP3_PREFIX)?;
    let after = if rest.starts_with(',') {
        rest.find('/')?
    } else if rest.starts_with('/') {
        0
    } else {
        return None;
    };
    Some(OP3_PREFIX.len() + after + 1)
}

/// The address of a file with no OP3 prefix (ADR 0076 item 1).
///
/// An address with no prefix stays as it is. Else the function removes
/// `https://op3.dev/e`, an optional `,pg=...` part and the `/` after it.
/// When the remaining text has no scheme, the function adds `https://`.
#[must_use]
pub fn plain_address(url: &str) -> Cow<'_, str> {
    let Some(length) = op3_prefix_len(url) else {
        return Cow::Borrowed(url);
    };
    let rest = &url[length..];
    if rest.starts_with("http://") || rest.starts_with("https://") {
        Cow::Borrowed(rest)
    } else {
        Cow::Owned(format!("https://{rest}"))
    }
}

/// `url` with its plain part replaced by `new_plain`. An OP3 prefix with
/// its `pg=` part stays (ADR 0076 item 4). An address with no prefix gives
/// `new_plain`.
///
/// An `http` address keeps its scheme after the prefix. An `https` address
/// has no scheme after the prefix, as in the form of the OP3 setup page.
#[must_use]
pub fn with_plain_address(url: &str, new_plain: &str) -> String {
    match op3_prefix_len(url) {
        Some(length) => {
            let tail = new_plain.strip_prefix("https://").unwrap_or(new_plain);
            format!("{}{tail}", &url[..length])
        }
        None => new_plain.to_owned(),
    }
}

#[cfg(test)]
mod plain_address_tests {
    use super::{plain_address, with_plain_address};

    #[test]
    fn plain_address_removes_each_op3_form() {
        for (given, plain) in [
            (
                "https://op3.dev/e/ipfspodcasting.net/e/music.example.com/a.mp3",
                "https://ipfspodcasting.net/e/music.example.com/a.mp3",
            ),
            (
                "https://op3.dev/e,pg=23c2d5df-52e2-4e9a-b562-54e0e0123511/music.example.com/a.mp3",
                "https://music.example.com/a.mp3",
            ),
            (
                "https://op3.dev/e/http://example.com/a.mp3",
                "http://example.com/a.mp3",
            ),
            (
                "https://op3.dev/e,pg=abc/https://example.com/a.mp3",
                "https://example.com/a.mp3",
            ),
            ("https://example.com/a.mp3", "https://example.com/a.mp3"),
            ("https://op3.dev/show/abc", "https://op3.dev/show/abc"),
            ("https://op3.dev/embed", "https://op3.dev/embed"),
        ] {
            assert_eq!(plain_address(given), plain, "address {given}");
        }
    }

    #[test]
    fn with_plain_address_keeps_the_prefix_and_its_pg_part() {
        assert_eq!(
            with_plain_address(
                "https://op3.dev/e,pg=abc/music.example.com/a.mp3",
                "https://cdn.example.com/b.mp3"
            ),
            "https://op3.dev/e,pg=abc/cdn.example.com/b.mp3"
        );
        assert_eq!(
            with_plain_address(
                "https://op3.dev/e/http://a.example/a.mp3",
                "http://b.example/b.mp3"
            ),
            "https://op3.dev/e/http://b.example/b.mp3"
        );
        assert_eq!(
            with_plain_address("https://a.example/a.mp3", "https://b.example/b.mp3"),
            "https://b.example/b.mp3"
        );
        let moved = with_plain_address(
            "https://op3.dev/e,pg=abc/a.example/a.mp3",
            "https://b.example/b.mp3",
        );
        assert_eq!(plain_address(&moved), "https://b.example/b.mp3");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kind_of_path_gives_the_kind_of_each_accepted_extension() {
        for name in [
            "a.jpg", "a.jpeg", "a.png", "a.webp", "a.gif", "A.JPG", "x/y.PnG",
        ] {
            assert_eq!(kind_of_path(name), Some(FileKind::Image), "{name}");
        }
        for name in [
            "a.mp3", "a.m4a", "a.flac", "a.ogg", "a.opus", "A.MP3", "x/y.FlAc",
        ] {
            assert_eq!(kind_of_path(name), Some(FileKind::Audio), "{name}");
        }
    }

    #[test]
    fn kind_of_path_gives_lyrics_for_each_lyrics_extension() {
        for name in [
            "a.srt", "a.vtt", "a.txt", "a.json", "a.html", "A.SRT", "x/y.VtT", "a.TXT", "a.Json",
            "a.HTML",
        ] {
            assert_eq!(kind_of_path(name), Some(FileKind::Lyrics), "{name}");
        }
        assert_eq!(FileKind::parse("lyrics"), Some(FileKind::Lyrics));
    }

    #[test]
    fn kind_of_path_gives_no_kind_for_other_files() {
        for name in ["a.xml", "a.wav", "a.srtx", "noextension", "mp3"] {
            assert_eq!(kind_of_path(name), None, "{name}");
        }
    }

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

        let gif = dir.path().join("a.gif");
        fs::write(&gif, b"GIF89a\x01\x00\x01\x00").expect("write");
        assert_eq!(sniff_image(&gif, "a.gif").unwrap(), Some("image/gif"));

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

    #[test]
    fn a_srt_file_with_utf8_text_imports_as_lyrics() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("a.srt");
        let text = "1\n00:00:01,000 --> 00:00:02,000\nHello \u{e9}\n";
        fs::write(&path, text).expect("write");
        let imported = import(&path, "a.srt").expect("import");
        assert_eq!(imported.kind, ImportedKind::Lyrics);
        assert_eq!(imported.media_type, "application/srt");
        assert_eq!(imported.size, text.len() as u64);
        assert_eq!(imported.file_name, "a.srt");
    }

    #[test]
    fn a_txt_file_that_is_not_utf8_is_unsupported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("a.txt");
        fs::write(&path, [0xc3_u8, 0x28, 0xa0, 0xa1]).expect("write");
        let error = import(&path, "a.txt").unwrap_err();
        assert_eq!(error.kind(), MediaErrorKind::Unsupported);
    }
}
