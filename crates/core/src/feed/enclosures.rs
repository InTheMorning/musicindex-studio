//! Alternate media files of an item: `podcast:alternateEnclosure` (ADR 0033).

use super::elements::Source;
use super::sync::{Children, Entry, KeyError, Modeled, Names, adopt_list, read_list};
use crate::xml::{Element, Ns};

/// `podcast:alternateEnclosure`: another file of the media of an item.
/// Studio uses it to offer a lossless file beside the MP3 enclosure
/// (ADR 0033).
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct AlternateEnclosure {
    /// `type`: the media type of the file.
    pub media_type: Option<String>,
    /// `length`, in bytes.
    pub length: Option<String>,
    /// `bitrate`, in bits per second.
    pub bitrate: Option<String>,
    /// `title`, such as `Lossless (FLAC)`.
    pub title: Option<String>,
    /// `rel`.
    pub rel: Option<String>,
    /// `default`: `true` when an app with a choice of files must play this
    /// one first.
    pub default: Option<String>,
    /// The `podcast:source` elements: each address of the file.
    pub sources: Vec<Entry<Source>>,
}

impl Modeled for AlternateEnclosure {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "alternateEnclosure";

    fn read(element: &Element) -> Self {
        Self {
            media_type: element.attribute("type").map(str::to_owned),
            length: element.attribute("length").map(str::to_owned),
            bitrate: element.attribute("bitrate").map(str::to_owned),
            title: element.attribute("title").map(str::to_owned),
            rel: element.attribute("rel").map(str::to_owned),
            default: element.attribute("default").map(str::to_owned),
            sources: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("type", self.media_type.as_deref());
        element.set_attribute("length", self.length.as_deref());
        element.set_attribute("bitrate", self.bitrate.as_deref());
        element.set_attribute("title", self.title.as_deref());
        element.set_attribute("rel", self.rel.as_deref());
        element.set_attribute("default", self.default.as_deref());
        Children::new(element, &[], names, None).list(&self.sources);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(&mut self.sources, draft.map(|d| d.sources.as_slice()))
    }
}

/// The two alternative enclosures of a lossless import (ADR 0033 item 6).
/// The first is the MP3 file, which an app plays first. The second is the
/// lossless FLAC file. `bitrate`, `rel` and each `contentType` are empty.
#[must_use]
pub fn lossless_pair(
    mp3_url: &str,
    mp3_size: u64,
    flac_url: &str,
    flac_size: u64,
) -> Vec<Entry<AlternateEnclosure>> {
    let source = |uri: &str| {
        Entry::new(Source {
            uri: Some(uri.to_owned()),
            content_type: None,
        })
    };
    vec![
        Entry::new(AlternateEnclosure {
            media_type: Some("audio/mpeg".to_owned()),
            length: Some(mp3_size.to_string()),
            title: Some("Standard (MP3)".to_owned()),
            default: Some("true".to_owned()),
            sources: vec![source(mp3_url)],
            ..AlternateEnclosure::default()
        }),
        Entry::new(AlternateEnclosure {
            media_type: Some("audio/flac".to_owned()),
            length: Some(flac_size.to_string()),
            title: Some("Maximum fidelity (lossless)".to_owned()),
            sources: vec![source(flac_url)],
            ..AlternateEnclosure::default()
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0033 item 6: the pair has the MP3 file first, with `default`.
    #[test]
    fn lossless_pair_gives_the_mp3_entry_and_the_flac_entry() {
        let pair = lossless_pair(
            "https://h.example/a.mp3",
            10,
            "https://h.example/a.flac",
            99,
        );
        assert_eq!(pair.len(), 2);
        let mp3 = &pair[0].value;
        assert_eq!(mp3.media_type.as_deref(), Some("audio/mpeg"));
        assert_eq!(mp3.length.as_deref(), Some("10"));
        assert_eq!(mp3.title.as_deref(), Some("Standard (MP3)"));
        assert_eq!(mp3.default.as_deref(), Some("true"));
        assert_eq!(mp3.sources.len(), 1);
        assert_eq!(
            mp3.sources[0].value.uri.as_deref(),
            Some("https://h.example/a.mp3")
        );
        let flac = &pair[1].value;
        assert_eq!(flac.media_type.as_deref(), Some("audio/flac"));
        assert_eq!(flac.length.as_deref(), Some("99"));
        assert_eq!(flac.title.as_deref(), Some("Maximum fidelity (lossless)"));
        assert_eq!(flac.default, None);
        assert_eq!(
            flac.sources[0].value.uri.as_deref(),
            Some("https://h.example/a.flac")
        );
        for enc in [mp3, flac] {
            assert_eq!((&enc.bitrate, &enc.rel), (&None, &None));
            assert_eq!(enc.sources[0].value.content_type, None);
        }
    }
}
