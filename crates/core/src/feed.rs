//! Typed model of a music feed (ADR 0008, ADR 0012).
//!
//! [`Feed::read`] projects the modeled elements of a [`Document`] into typed
//! values. [`Feed::write`] puts the values back into the same document and
//! changes only the elements whose values changed. Each element that the
//! model does not cover stays in the document as the source wrote it.
//!
//! Attribute values are strings, as the source wrote them. A missing
//! attribute is `None`, so that a read and a write keep the difference
//! between a missing attribute and an empty one.

mod elements;
mod enclosures;
mod links;
mod sync;
mod value;

use std::fmt;

pub use elements::{
    AtomLink, Enclosure, Funding, Image, License, Locked, Person, Recipient, RemoteItem, Source,
    Transcript, Txt,
};
pub use enclosures::AlternateEnclosure;
pub use links::{BadAddress, is_web_address};
use sync::{Children, Modeled, Names, adopt_list, adopt_one, read_list, read_one, read_text};
pub use sync::{Entry, KeyError};
pub use value::{ValueBlock, ValueTimeSplit};

use crate::xml::{Document, Element, Node, Ns};

/// The elements of `channel` that come after its other elements.
const CHANNEL_TAIL: &[(Ns, &str)] = &[(Ns::None, "item"), (Ns::Podcast, "liveItem")];

/// An error from [`Feed::read`] or [`Feed::write`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedError {
    message: &'static str,
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for FeedError {}

const NO_CHANNEL: FeedError = FeedError {
    message: "the document has no rss/channel element",
};

/// A feed: the modeled content of its `channel`.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Feed {
    /// `title`.
    pub title: Option<String>,
    /// `description`.
    pub description: Option<String>,
    /// `link`.
    pub link: Option<String>,
    /// `language`.
    pub language: Option<String>,
    /// `podcast:guid`. It does not change after the feed has one (ADR 0008).
    pub podcast_guid: Option<String>,
    /// `podcast:medium`, such as `music`, `publisher` or `musicL`.
    pub medium: Option<String>,
    /// `podcast:locked`.
    pub locked: Option<Locked>,
    /// `itunes:author`.
    pub author: Option<String>,
    /// The `href` of `itunes:image`.
    pub image: Option<String>,
    /// `itunes:new-feed-url`: the new URL after a host move.
    pub new_feed_url: Option<String>,
    /// The `atom:link` elements. See [`Feed::self_url`].
    pub atom_links: Vec<Entry<AtomLink>>,
    /// The `podcast:image` elements.
    pub images: Vec<Entry<Image>>,
    /// The `podcast:person` elements.
    pub persons: Vec<Entry<Person>>,
    /// The `podcast:funding` elements.
    pub fundings: Vec<Entry<Funding>>,
    /// The `podcast:txt` elements.
    pub txts: Vec<Entry<Txt>>,
    /// `podcast:license`.
    pub license: Option<License>,
    /// `podcast:publisher`.
    pub publisher: Option<Publisher>,
    /// `podcast:podroll`.
    pub podroll: Option<Podroll>,
    /// The `podcast:remoteItem` elements of the channel. A publisher feed
    /// lists its albums here, and a musicL feed lists its tracks here.
    pub remote_items: Vec<Entry<RemoteItem>>,
    /// The `podcast:value` blocks of the channel.
    pub values: Vec<Entry<ValueBlock>>,
    /// The `item` elements.
    pub items: Vec<Entry<Item>>,
    /// The `podcast:liveItem` elements.
    pub live_items: Vec<Entry<LiveItem>>,
}

impl Feed {
    /// Read the modeled elements of the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`FeedError`] when the document has no `rss/channel` element.
    pub fn read(doc: &Document) -> Result<Self, FeedError> {
        let channel = channel(doc.root()).ok_or(NO_CHANNEL)?;
        Ok(Self {
            title: read_text(channel, Ns::None, "title"),
            description: read_text(channel, Ns::None, "description"),
            link: read_text(channel, Ns::None, "link"),
            language: read_text(channel, Ns::None, "language"),
            podcast_guid: read_text(channel, Ns::Podcast, "guid"),
            medium: read_text(channel, Ns::Podcast, "medium"),
            locked: read_one(channel),
            author: read_text(channel, Ns::Itunes, "author"),
            image: read_href(channel, Ns::Itunes, "image"),
            new_feed_url: read_text(channel, Ns::Itunes, "new-feed-url"),
            atom_links: read_list(channel),
            images: read_list(channel),
            persons: read_list(channel),
            fundings: read_list(channel),
            txts: read_list(channel),
            license: read_one(channel),
            publisher: read_one(channel),
            podroll: read_one(channel),
            remote_items: read_list(channel),
            values: read_list(channel),
            items: read_list(channel),
            live_items: read_list(channel),
        })
    }

    /// Put the values into `doc` and change only what is different.
    ///
    /// # Errors
    ///
    /// Gives a [`FeedError`] when the document has no `rss/channel` element.
    pub fn write(&self, doc: &mut Document) -> Result<(), FeedError> {
        self.write_hosted(doc, None)
    }

    /// Put the values into `doc` and change only what is different.
    ///
    /// `hosted` is the hosted file of the draft, or none. A new direct
    /// child of the channel or of an item can go to its hosted position.
    /// This happens when `hosted` has an element of that type (ADR 0031).
    ///
    /// # Errors
    ///
    /// Gives a [`FeedError`] when the document has no `rss/channel` element.
    pub fn write_hosted(
        &self,
        doc: &mut Document,
        hosted: Option<&Document>,
    ) -> Result<(), FeedError> {
        let mut names = Names::new(doc.root(), hosted);
        let channel_hosted = names.hosted_channel();
        let channel = channel_mut(doc.root_mut()).ok_or(NO_CHANNEL)?;
        let mut c = Children::new(channel, CHANNEL_TAIL, &mut names, channel_hosted);
        c.text(Ns::None, "title", self.title.as_ref());
        c.text(Ns::None, "description", self.description.as_ref());
        c.text(Ns::None, "link", self.link.as_ref());
        c.text(Ns::None, "language", self.language.as_ref());
        c.text(Ns::Podcast, "guid", self.podcast_guid.as_ref());
        c.text(Ns::Podcast, "medium", self.medium.as_ref());
        c.one(self.locked.as_ref());
        c.text(Ns::Itunes, "author", self.author.as_ref());
        c.href(Ns::Itunes, "image", self.image.as_ref());
        c.text(Ns::Itunes, "new-feed-url", self.new_feed_url.as_ref());
        c.list(&self.atom_links);
        c.list(&self.images);
        c.list(&self.persons);
        c.list(&self.fundings);
        c.list(&self.txts);
        c.one(self.license.as_ref());
        c.one(self.publisher.as_ref());
        c.one(self.podroll.as_ref());
        c.list(&self.remote_items);
        c.list(&self.values);
        c.list_before(&self.items, &[(Ns::Podcast, "liveItem")]);
        c.list_before(&self.live_items, &[]);
        names.finish(doc.root_mut());
        Ok(())
    }

    /// Give each entry of the sent model the source element of the draft
    /// entry with the same key, so that the writer keeps its source bytes
    /// (ADR 0022 items 2 to 4).
    ///
    /// # Errors
    ///
    /// Gives a [`KeyError`] when a key is not in the draft, or when a key
    /// occurs two times in one list.
    pub fn adopt(&mut self, draft: &Feed) -> Result<(), KeyError> {
        adopt_list(&mut self.atom_links, Some(&draft.atom_links))?;
        adopt_list(&mut self.images, Some(&draft.images))?;
        adopt_list(&mut self.persons, Some(&draft.persons))?;
        adopt_list(&mut self.fundings, Some(&draft.fundings))?;
        adopt_list(&mut self.txts, Some(&draft.txts))?;
        adopt_one(self.publisher.as_mut(), draft.publisher.as_ref())?;
        adopt_one(self.podroll.as_mut(), draft.podroll.as_ref())?;
        adopt_list(&mut self.remote_items, Some(&draft.remote_items))?;
        adopt_list(&mut self.values, Some(&draft.values))?;
        adopt_list(&mut self.items, Some(&draft.items))?;
        adopt_list(&mut self.live_items, Some(&draft.live_items))
    }

    /// The feed URL that the feed gives for itself: the `href` of the first
    /// `atom:link` with `rel="self"`.
    #[must_use]
    pub fn self_url(&self) -> Option<&str> {
        self.atom_links
            .iter()
            .find(|l| l.rel.as_deref() == Some("self"))
            .and_then(|l| l.href.as_deref())
    }

    /// Each `podcast:valueRecipient` of each value block of the channel, of
    /// each item and of each live item (ADR 0027 item 4).
    ///
    /// The list does not have a recipient of a value time split. A
    /// recipient with no address, or with an address of only spaces, is not
    /// in the list.
    #[must_use]
    pub fn recipients(&self) -> Vec<&Recipient> {
        let mut found = Vec::new();
        for value in &self.values {
            push_recipients(&mut found, &value.recipients);
        }
        for item in &self.items {
            for value in &item.values {
                push_recipients(&mut found, &value.recipients);
            }
        }
        for live in &self.live_items {
            for value in &live.item.values {
                push_recipients(&mut found, &value.recipients);
            }
        }
        found
    }
}

/// Add each recipient of `entries` that has an address of one or more
/// characters that are not spaces.
fn push_recipients<'a>(found: &mut Vec<&'a Recipient>, entries: &'a [Entry<Recipient>]) {
    for entry in entries {
        let Some(address) = entry.value.address.as_deref() else {
            continue;
        };
        if address.trim().is_empty() {
            continue;
        }
        found.push(&entry.value);
    }
}

fn channel(root: &Element) -> Option<&Element> {
    root.is(Ns::None, "rss")
        .then(|| root.child(Ns::None, "channel"))
        .flatten()
}

fn channel_mut(root: &mut Element) -> Option<&mut Element> {
    if !root.is(Ns::None, "rss") {
        return None;
    }
    root.children_mut().iter_mut().find_map(|n| match n {
        Node::Element(e) if e.is(Ns::None, "channel") => Some(e),
        _ => None,
    })
}

fn read_href(parent: &Element, ns: Ns, local: &str) -> Option<String> {
    parent
        .child(ns, local)
        .and_then(|e| e.attribute("href"))
        .map(str::to_owned)
}

/// `podcast:publisher`: the link from a feed to its publisher feeds.
///
/// Podcast Namespace PR #793 lets a feed have more than one publisher, each
/// a `podcast:remoteItem` with `medium="publisher"` and a `rel` role set.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Publisher {
    /// The `podcast:remoteItem` elements.
    pub remote_items: Vec<Entry<RemoteItem>>,
}

impl Modeled for Publisher {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "publisher";

    fn read(element: &Element) -> Self {
        Self {
            remote_items: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        Children::new(element, &[], names, None).list(&self.remote_items);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(
            &mut self.remote_items,
            draft.map(|d| d.remote_items.as_slice()),
        )
    }
}

/// `podcast:podroll`: feeds that the creator recommends.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Podroll {
    /// The `podcast:remoteItem` elements.
    pub remote_items: Vec<Entry<RemoteItem>>,
}

impl Modeled for Podroll {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "podroll";

    fn read(element: &Element) -> Self {
        Self {
            remote_items: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        Children::new(element, &[], names, None).list(&self.remote_items);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(
            &mut self.remote_items,
            draft.map(|d| d.remote_items.as_slice()),
        )
    }
}

/// An `item`: a track or an episode.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Item {
    /// `title`.
    pub title: Option<String>,
    /// `description`.
    pub description: Option<String>,
    /// The text of `guid`. It does not change without a user action (ADR 0008).
    pub guid: Option<String>,
    /// `enclosure`.
    pub enclosure: Option<Enclosure>,
    /// The `podcast:alternateEnclosure` elements: other files of the media
    /// of the item, such as a lossless file next to an MP3 file
    /// (ADR 0033). A model with no such field also reads, as an empty list.
    #[serde(default)]
    pub alternate_enclosures: Vec<Entry<AlternateEnclosure>>,
    /// `pubDate`, as the source wrote it.
    pub pub_date: Option<String>,
    /// `itunes:duration`, as the source wrote it.
    pub duration: Option<String>,
    /// The text of `podcast:episode`: the track number of an album track.
    /// The core sets it from the track sequence (ADR 0024).
    pub episode: Option<String>,
    /// The `href` of `itunes:image`.
    pub image: Option<String>,
    /// The `podcast:image` elements.
    pub images: Vec<Entry<Image>>,
    /// The `podcast:person` elements. They take the place of the channel
    /// persons for this item.
    pub persons: Vec<Entry<Person>>,
    /// The `podcast:funding` elements.
    pub fundings: Vec<Entry<Funding>>,
    /// The `podcast:transcript` elements.
    pub transcripts: Vec<Entry<Transcript>>,
    /// The `podcast:txt` elements.
    pub txts: Vec<Entry<Txt>>,
    /// `podcast:license`.
    pub license: Option<License>,
    /// The `podcast:value` blocks of the item. They take the place of the
    /// channel value blocks for this item.
    pub values: Vec<Entry<ValueBlock>>,
}

impl Item {
    fn read_fields(element: &Element) -> Self {
        Self {
            title: read_text(element, Ns::None, "title"),
            description: read_text(element, Ns::None, "description"),
            guid: read_text(element, Ns::None, "guid"),
            enclosure: read_one(element),
            alternate_enclosures: read_list(element),
            pub_date: read_text(element, Ns::None, "pubDate"),
            duration: read_text(element, Ns::Itunes, "duration"),
            episode: read_text(element, Ns::Podcast, "episode"),
            image: read_href(element, Ns::Itunes, "image"),
            images: read_list(element),
            persons: read_list(element),
            fundings: read_list(element),
            transcripts: read_list(element),
            txts: read_list(element),
            license: read_one(element),
            values: read_list(element),
        }
    }

    fn apply_fields(&self, element: &mut Element, names: &mut Names) {
        let had_guid = element.child(Ns::None, "guid").is_some();
        self.apply_children(element, names);
        // A new `guid` is a UUID, not a link. RSS 2.0 reads a `guid` without
        // `isPermaLink="false"` as a link. An existing `guid` keeps its bytes.
        if !had_guid && self.guid.is_some() {
            let guid = element.children_mut().iter_mut().find_map(|n| match n {
                Node::Element(e) if e.is(Ns::None, "guid") => Some(e),
                _ => None,
            });
            if let Some(guid) = guid {
                guid.set_attribute("isPermaLink", Some("false"));
            }
        }
    }

    fn apply_children(&self, element: &mut Element, names: &mut Names) {
        // The hosted parent of the item is the item of the hosted file with
        // the same GUID. A new item uses the last hosted item as its model
        // (ADR 0031 item 2).
        let hosted = names.hosted_item(self.guid.as_deref());
        let mut c = Children::new(element, &[], names, hosted);
        c.text(Ns::None, "title", self.title.as_ref());
        c.text(Ns::None, "description", self.description.as_ref());
        c.text(Ns::None, "guid", self.guid.as_ref());
        c.one(self.enclosure.as_ref());
        // A new alternate enclosure with no hosted position goes directly
        // after the enclosure (ADR 0031, ADR 0033 item 6).
        c.list_after(&self.alternate_enclosures, &[(Ns::None, "enclosure")]);
        c.text(Ns::None, "pubDate", self.pub_date.as_ref());
        c.text(Ns::Itunes, "duration", self.duration.as_ref());
        c.text(Ns::Podcast, "episode", self.episode.as_ref());
        c.href(Ns::Itunes, "image", self.image.as_ref());
        c.list(&self.images);
        c.list(&self.persons);
        c.list(&self.fundings);
        c.list(&self.transcripts);
        c.list(&self.txts);
        c.one(self.license.as_ref());
        c.list(&self.values);
    }

    fn adopt_fields(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(&mut self.images, draft.map(|d| d.images.as_slice()))?;
        adopt_list(&mut self.persons, draft.map(|d| d.persons.as_slice()))?;
        adopt_list(&mut self.fundings, draft.map(|d| d.fundings.as_slice()))?;
        adopt_list(
            &mut self.transcripts,
            draft.map(|d| d.transcripts.as_slice()),
        )?;
        adopt_list(&mut self.txts, draft.map(|d| d.txts.as_slice()))?;
        adopt_list(
            &mut self.alternate_enclosures,
            draft.map(|d| d.alternate_enclosures.as_slice()),
        )?;
        adopt_list(&mut self.values, draft.map(|d| d.values.as_slice()))
    }
}

impl Modeled for Item {
    const NS: Ns = Ns::None;
    const NAME: &'static str = "item";

    fn read(element: &Element) -> Self {
        Self::read_fields(element)
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        self.apply_fields(element, names);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        self.adopt_fields(draft)
    }
}

/// `podcast:liveItem`: a live or scheduled show.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct LiveItem {
    /// `status`: `pending`, `live` or `ended`.
    pub status: Option<String>,
    /// `start`, as an ISO 8601 time.
    pub start: Option<String>,
    /// `end`, as an ISO 8601 time.
    pub end: Option<String>,
    /// The elements that a live item has in common with an item.
    pub item: Item,
}

impl Modeled for LiveItem {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "liveItem";

    fn read(element: &Element) -> Self {
        Self {
            status: element.attribute("status").map(str::to_owned),
            start: element.attribute("start").map(str::to_owned),
            end: element.attribute("end").map(str::to_owned),
            item: Item::read_fields(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("status", self.status.as_deref());
        element.set_attribute("start", self.start.as_deref());
        element.set_attribute("end", self.end.as_deref());
        self.item.apply_fields(element, names);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        self.item.adopt_fields(draft.map(|d| &d.item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipient(address: &str) -> Entry<Recipient> {
        Entry::new(Recipient {
            address: Some(address.to_owned()),
            ..Recipient::default()
        })
    }

    #[test]
    fn recipients_come_from_the_channel_items_and_live_items_but_not_a_time_split() {
        let feed = Feed {
            values: vec![Entry::new(ValueBlock {
                recipients: vec![recipient("channel"), recipient(""), recipient("  ")],
                time_splits: vec![Entry::new(ValueTimeSplit {
                    recipients: vec![recipient("split")],
                    ..ValueTimeSplit::default()
                })],
                ..ValueBlock::default()
            })],
            items: vec![Entry::new(Item {
                values: vec![Entry::new(ValueBlock {
                    recipients: vec![recipient("item")],
                    ..ValueBlock::default()
                })],
                ..Item::default()
            })],
            live_items: vec![Entry::new(LiveItem {
                item: Item {
                    values: vec![Entry::new(ValueBlock {
                        recipients: vec![recipient("live")],
                        ..ValueBlock::default()
                    })],
                    ..Item::default()
                },
                ..LiveItem::default()
            })],
            ..Feed::default()
        };
        let addresses: Vec<_> = feed
            .recipients()
            .into_iter()
            .map(|r| r.address.as_deref().unwrap())
            .collect();
        assert_eq!(addresses, ["channel", "item", "live"]);
    }
}
