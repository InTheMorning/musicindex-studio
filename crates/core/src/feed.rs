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

mod sync;

use std::fmt;

pub use sync::Entry;
use sync::{Make, Modeled, Names, read_list, read_text, write_list, write_single, write_text};

use crate::xml::{Document, Element, Ns};

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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    /// `podcast:publisher`.
    pub publisher: Option<Publisher>,
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
            locked: channel.child(Ns::Podcast, "locked").map(Locked::read),
            author: read_text(channel, Ns::Itunes, "author"),
            image: channel
                .child(Ns::Itunes, "image")
                .and_then(|e| e.attribute("href"))
                .map(str::to_owned),
            publisher: channel.child(Ns::Podcast, "publisher").map(Publisher::read),
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
        let mut names = Names::new(doc.root());
        let channel = channel_mut(doc.root_mut()).ok_or(NO_CHANNEL)?;
        let tail = CHANNEL_TAIL;
        let n = &mut names;
        write_text(channel, Ns::None, "title", self.title.as_deref(), tail, n);
        write_text(
            channel,
            Ns::None,
            "description",
            self.description.as_deref(),
            tail,
            n,
        );
        write_text(channel, Ns::None, "link", self.link.as_deref(), tail, n);
        write_text(
            channel,
            Ns::None,
            "language",
            self.language.as_deref(),
            tail,
            n,
        );
        write_text(
            channel,
            Ns::Podcast,
            "guid",
            self.podcast_guid.as_deref(),
            tail,
            n,
        );
        write_text(
            channel,
            Ns::Podcast,
            "medium",
            self.medium.as_deref(),
            tail,
            n,
        );
        let locked = self
            .locked
            .as_ref()
            .map(|l| move |e: &mut Element, n: &mut Names| l.apply(e, n));
        write_single(
            channel,
            Ns::Podcast,
            "locked",
            dyn_fn(locked.as_ref()),
            tail,
            n,
        );
        write_text(
            channel,
            Ns::Itunes,
            "author",
            self.author.as_deref(),
            tail,
            n,
        );
        let image = self
            .image
            .as_deref()
            .map(|href| move |e: &mut Element, _: &mut Names| e.set_attribute("href", Some(href)));
        write_single(
            channel,
            Ns::Itunes,
            "image",
            dyn_fn(image.as_ref()),
            tail,
            n,
        );
        let publisher = self
            .publisher
            .as_ref()
            .map(|p| move |e: &mut Element, n: &mut Names| p.apply(e, n));
        write_single(
            channel,
            Ns::Podcast,
            "publisher",
            dyn_fn(publisher.as_ref()),
            tail,
            n,
        );
        write_list(channel, &self.remote_items, tail, n);
        write_list(channel, &self.values, tail, n);
        write_list(channel, &self.items, &[(Ns::Podcast, "liveItem")], n);
        write_list(channel, &self.live_items, &[], n);
        names.finish(doc.root_mut());
        Ok(())
    }
}

fn dyn_fn<F: Fn(&mut Element, &mut Names)>(f: Option<&F>) -> Option<Make<'_>> {
    f.map(|f| f as Make<'_>)
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
        crate::xml::Node::Element(e) if e.is(Ns::None, "channel") => Some(e),
        _ => None,
    })
}

/// Read each named attribute into a struct field.
fn attr(element: &Element, name: &str) -> Option<String> {
    element.attribute(name).map(str::to_owned)
}

/// `podcast:locked`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Locked {
    /// The text: `yes` or `no`.
    pub value: String,
    /// The `owner` attribute.
    pub owner: Option<String>,
}

impl Locked {
    fn read(element: &Element) -> Self {
        Self {
            value: element.text(),
            owner: attr(element, "owner"),
        }
    }

    fn apply(&self, element: &mut Element, _: &mut Names) {
        element.set_attribute("owner", self.owner.as_deref());
        element.set_text(&self.value);
    }
}

/// `podcast:publisher`: the link from a feed to its publisher feeds.
///
/// Podcast Namespace PR #793 lets a feed have more than one publisher, each
/// a `podcast:remoteItem` with `medium="publisher"` and a `rel` role set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Publisher {
    /// The `podcast:remoteItem` elements.
    pub remote_items: Vec<Entry<RemoteItem>>,
}

impl Publisher {
    fn read(element: &Element) -> Self {
        Self {
            remote_items: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        write_list(element, &self.remote_items, &[], names);
    }
}

/// `podcast:remoteItem`: a reference to a feed or to an item in a feed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteItem {
    /// `feedGuid`.
    pub feed_guid: Option<String>,
    /// `feedUrl`.
    pub feed_url: Option<String>,
    /// `itemGuid`.
    pub item_guid: Option<String>,
    /// `medium`.
    pub medium: Option<String>,
    /// `title`.
    pub title: Option<String>,
    /// `rel`: a set of role words (Podcast Namespace PR #793).
    pub rel: Option<String>,
}

impl Modeled for RemoteItem {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "remoteItem";

    fn read(element: &Element) -> Self {
        Self {
            feed_guid: attr(element, "feedGuid"),
            feed_url: attr(element, "feedUrl"),
            item_guid: attr(element, "itemGuid"),
            medium: attr(element, "medium"),
            title: attr(element, "title"),
            rel: attr(element, "rel"),
        }
    }

    fn apply(&self, element: &mut Element, _: &mut Names) {
        element.set_attribute("feedGuid", self.feed_guid.as_deref());
        element.set_attribute("feedUrl", self.feed_url.as_deref());
        element.set_attribute("itemGuid", self.item_guid.as_deref());
        element.set_attribute("medium", self.medium.as_deref());
        element.set_attribute("title", self.title.as_deref());
        element.set_attribute("rel", self.rel.as_deref());
    }
}

/// `podcast:value`: a payment block for a channel or an item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueBlock {
    /// `type`, such as `lightning`.
    pub value_type: Option<String>,
    /// `method`, such as `keysend` or `lnaddress`.
    pub method: Option<String>,
    /// `suggested`.
    pub suggested: Option<String>,
    /// The `podcast:valueRecipient` elements.
    pub recipients: Vec<Entry<Recipient>>,
    /// The `podcast:valueTimeSplit` elements.
    pub time_splits: Vec<Entry<ValueTimeSplit>>,
}

impl Modeled for ValueBlock {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "value";

    fn read(element: &Element) -> Self {
        Self {
            value_type: attr(element, "type"),
            method: attr(element, "method"),
            suggested: attr(element, "suggested"),
            recipients: read_list(element),
            time_splits: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("type", self.value_type.as_deref());
        element.set_attribute("method", self.method.as_deref());
        element.set_attribute("suggested", self.suggested.as_deref());
        write_list(
            element,
            &self.recipients,
            &[(Ns::Podcast, "valueTimeSplit")],
            names,
        );
        write_list(element, &self.time_splits, &[], names);
    }
}

/// `podcast:valueRecipient`: one destination of a payment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Recipient {
    /// `name`.
    pub name: Option<String>,
    /// `type`, such as `node` or `lnaddress`.
    pub recipient_type: Option<String>,
    /// `address`.
    pub address: Option<String>,
    /// `split`: the number of shares.
    pub split: Option<String>,
    /// `customKey`.
    pub custom_key: Option<String>,
    /// `customValue`.
    pub custom_value: Option<String>,
    /// `fee`.
    pub fee: Option<String>,
}

impl Modeled for Recipient {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "valueRecipient";

    fn read(element: &Element) -> Self {
        Self {
            name: attr(element, "name"),
            recipient_type: attr(element, "type"),
            address: attr(element, "address"),
            split: attr(element, "split"),
            custom_key: attr(element, "customKey"),
            custom_value: attr(element, "customValue"),
            fee: attr(element, "fee"),
        }
    }

    fn apply(&self, element: &mut Element, _: &mut Names) {
        element.set_attribute("name", self.name.as_deref());
        element.set_attribute("type", self.recipient_type.as_deref());
        element.set_attribute("address", self.address.as_deref());
        element.set_attribute("split", self.split.as_deref());
        element.set_attribute("customKey", self.custom_key.as_deref());
        element.set_attribute("customValue", self.custom_value.as_deref());
        element.set_attribute("fee", self.fee.as_deref());
    }
}

/// `podcast:valueTimeSplit`: different recipients for a period of playback.
///
/// The specification permits one `podcast:remoteItem` or one or more
/// `podcast:valueRecipient` elements. The model keeps lists of both, so that
/// it also keeps a feed that does not obey this rule.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueTimeSplit {
    /// `startTime`, in seconds.
    pub start_time: Option<String>,
    /// `duration`, in seconds.
    pub duration: Option<String>,
    /// `remoteStartTime`, in seconds.
    pub remote_start_time: Option<String>,
    /// `remotePercentage`.
    pub remote_percentage: Option<String>,
    /// The `podcast:remoteItem` elements.
    pub remote_items: Vec<Entry<RemoteItem>>,
    /// The `podcast:valueRecipient` elements.
    pub recipients: Vec<Entry<Recipient>>,
}

impl Modeled for ValueTimeSplit {
    const NS: Ns = Ns::Podcast;
    const NAME: &'static str = "valueTimeSplit";

    fn read(element: &Element) -> Self {
        Self {
            start_time: attr(element, "startTime"),
            duration: attr(element, "duration"),
            remote_start_time: attr(element, "remoteStartTime"),
            remote_percentage: attr(element, "remotePercentage"),
            remote_items: read_list(element),
            recipients: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("startTime", self.start_time.as_deref());
        element.set_attribute("duration", self.duration.as_deref());
        element.set_attribute("remoteStartTime", self.remote_start_time.as_deref());
        element.set_attribute("remotePercentage", self.remote_percentage.as_deref());
        write_list(element, &self.remote_items, &[], names);
        write_list(element, &self.recipients, &[], names);
    }
}

/// `enclosure`: the media file of an item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Enclosure {
    /// `url`.
    pub url: Option<String>,
    /// `length`, in bytes.
    pub length: Option<String>,
    /// `type`: the media type.
    pub media_type: Option<String>,
}

impl Enclosure {
    fn read(element: &Element) -> Self {
        Self {
            url: attr(element, "url"),
            length: attr(element, "length"),
            media_type: attr(element, "type"),
        }
    }

    fn apply(&self, element: &mut Element) {
        element.set_attribute("url", self.url.as_deref());
        element.set_attribute("length", self.length.as_deref());
        element.set_attribute("type", self.media_type.as_deref());
    }
}

/// An `item`: a track or an episode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Item {
    /// `title`.
    pub title: Option<String>,
    /// The text of `guid`. It does not change without a user action (ADR 0008).
    pub guid: Option<String>,
    /// `enclosure`.
    pub enclosure: Option<Enclosure>,
    /// `pubDate`, as the source wrote it.
    pub pub_date: Option<String>,
    /// `itunes:duration`, as the source wrote it.
    pub duration: Option<String>,
    /// The `podcast:value` blocks of the item. They take the place of the
    /// channel value blocks for this item.
    pub values: Vec<Entry<ValueBlock>>,
}

impl Item {
    fn read_fields(element: &Element) -> Self {
        Self {
            title: read_text(element, Ns::None, "title"),
            guid: read_text(element, Ns::None, "guid"),
            enclosure: element.child(Ns::None, "enclosure").map(Enclosure::read),
            pub_date: read_text(element, Ns::None, "pubDate"),
            duration: read_text(element, Ns::Itunes, "duration"),
            values: read_list(element),
        }
    }

    fn apply_fields(&self, element: &mut Element, names: &mut Names) {
        write_text(
            element,
            Ns::None,
            "title",
            self.title.as_deref(),
            &[],
            names,
        );
        write_text(element, Ns::None, "guid", self.guid.as_deref(), &[], names);
        let enclosure = self
            .enclosure
            .as_ref()
            .map(|en| move |e: &mut Element, _: &mut Names| en.apply(e));
        write_single(
            element,
            Ns::None,
            "enclosure",
            dyn_fn(enclosure.as_ref()),
            &[],
            names,
        );
        write_text(
            element,
            Ns::None,
            "pubDate",
            self.pub_date.as_deref(),
            &[],
            names,
        );
        write_text(
            element,
            Ns::Itunes,
            "duration",
            self.duration.as_deref(),
            &[],
            names,
        );
        write_list(element, &self.values, &[], names);
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
}

/// `podcast:liveItem`: a live or scheduled show.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
            status: attr(element, "status"),
            start: attr(element, "start"),
            end: attr(element, "end"),
            item: Item::read_fields(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("status", self.status.as_deref());
        element.set_attribute("start", self.start.as_deref());
        element.set_attribute("end", self.end.as_deref());
        self.item.apply_fields(element, names);
    }
}
