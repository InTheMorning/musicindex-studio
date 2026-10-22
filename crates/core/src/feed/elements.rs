//! Elements that hold only attributes, or text and attributes.

use super::sync::{Modeled, Names};
use crate::xml::{Element, Ns};

/// Define a struct for an element with optional text and attributes.
///
/// Each attribute is an `Option<String>`, so that a read and a write keep the
/// difference between a missing attribute and an empty one. The text, when
/// there is one, is a `String`.
macro_rules! element_type {
    (
        $(#[$meta:meta])*
        $ty:ident = ($ns:expr, $name:literal) {
            $(@text $(#[$tmeta:meta])* $text:ident;)?
            $($(#[$fmeta:meta])* $field:ident => $attr:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        pub struct $ty {
            $($(#[$tmeta])* pub $text: String,)?
            $($(#[$fmeta])* pub $field: Option<String>,)*
        }

        impl Modeled for $ty {
            const NS: Ns = $ns;
            const NAME: &'static str = $name;

            fn read(element: &Element) -> Self {
                Self {
                    $($text: element.text(),)?
                    $($field: element.attribute($attr).map(str::to_owned),)*
                }
            }

            fn apply(&self, element: &mut Element, _: &mut Names) {
                $(element.set_attribute($attr, self.$field.as_deref());)*
                $(element.set_text(&self.$text);)?
            }
        }
    };
}

element_type! {
    /// `podcast:remoteItem`: a reference to a feed or to an item in a feed.
    RemoteItem = (Ns::Podcast, "remoteItem") {
        /// `feedGuid`.
        feed_guid => "feedGuid",
        /// `feedUrl`.
        feed_url => "feedUrl",
        /// `itemGuid`.
        item_guid => "itemGuid",
        /// `medium`.
        medium => "medium",
        /// `title`.
        title => "title",
        /// `rel`: a set of role words (Podcast Namespace PR #793).
        rel => "rel",
    }
}

element_type! {
    /// `podcast:valueRecipient`: one destination of a payment.
    Recipient = (Ns::Podcast, "valueRecipient") {
        /// `name`.
        name => "name",
        /// `type`, such as `node` or `lnaddress`.
        recipient_type => "type",
        /// `address`.
        address => "address",
        /// `split`: the number of shares.
        split => "split",
        /// `customKey`.
        custom_key => "customKey",
        /// `customValue`.
        custom_value => "customValue",
        /// `fee`.
        fee => "fee",
    }
}

element_type! {
    /// `enclosure`: the media file of an item.
    Enclosure = (Ns::None, "enclosure") {
        /// `url`.
        url => "url",
        /// `length`, in bytes.
        length => "length",
        /// `type`: the media type.
        media_type => "type",
    }
}

element_type! {
    /// `podcast:locked`.
    Locked = (Ns::Podcast, "locked") {
        @text
        /// The text: `yes` or `no`.
        value;
        /// `owner`.
        owner => "owner",
    }
}

element_type! {
    /// `podcast:person`: a person who made the feed or the item.
    Person = (Ns::Podcast, "person") {
        @text
        /// The name of the person.
        name;
        /// `role`. A missing role means `host`.
        role => "role",
        /// `group`. A missing group means `cast`.
        group => "group",
        /// `img`: the URL of a picture of the person.
        img => "img",
        /// `href`: the URL of information about the person.
        href => "href",
    }
}

element_type! {
    /// `podcast:funding`: a link where listeners can give money.
    Funding = (Ns::Podcast, "funding") {
        @text
        /// The text that an app shows with the link.
        label;
        /// `url`.
        url => "url",
    }
}

element_type! {
    /// `podcast:transcript`: a transcript or captions file of an item.
    Transcript = (Ns::Podcast, "transcript") {
        /// `url`.
        url => "url",
        /// `type`: the media type.
        media_type => "type",
        /// `language`.
        language => "language",
        /// `rel`, such as `captions`.
        rel => "rel",
    }
}

element_type! {
    /// `podcast:txt`: free text for a purpose, such as a proof of control.
    Txt = (Ns::Podcast, "txt") {
        @text
        /// The text.
        value;
        /// `purpose`.
        purpose => "purpose",
    }
}

element_type! {
    /// `podcast:license`.
    License = (Ns::Podcast, "license") {
        @text
        /// The license identifier, such as `cc-by-4.0`.
        identifier;
        /// `url`: the full text of the license.
        url => "url",
    }
}

element_type! {
    /// `podcast:image`: an image with its intended use.
    Image = (Ns::Podcast, "image") {
        /// `href`.
        href => "href",
        /// `alt`.
        alt => "alt",
        /// `aspect-ratio`.
        aspect_ratio => "aspect-ratio",
        /// `width`, in pixels.
        width => "width",
        /// `height`, in pixels.
        height => "height",
        /// `type`: the media type.
        media_type => "type",
        /// `purpose`: a set of use words.
        purpose => "purpose",
    }
}

element_type! {
    /// `atom:link`, such as the `self` link that gives the feed URL.
    AtomLink = (Ns::Atom, "link") {
        /// `href`.
        href => "href",
        /// `rel`, such as `self` or `hub`.
        rel => "rel",
        /// `type`.
        media_type => "type",
    }
}
