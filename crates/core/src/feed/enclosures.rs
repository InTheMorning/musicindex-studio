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
