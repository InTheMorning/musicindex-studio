//! Payment blocks: `podcast:value` and `podcast:valueTimeSplit`.

use super::CreditPlace;
use super::elements::{Recipient, RemoteItem};
use super::sync::{Children, Entry, KeyError, Modeled, Names, adopt_list, read_list};
use crate::xml::{Element, Ns};

/// `podcast:value`: a payment block for a channel or an item.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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
            value_type: element.attribute("type").map(str::to_owned),
            method: element.attribute("method").map(str::to_owned),
            suggested: element.attribute("suggested").map(str::to_owned),
            recipients: read_list(element),
            time_splits: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("type", self.value_type.as_deref());
        element.set_attribute("method", self.method.as_deref());
        element.set_attribute("suggested", self.suggested.as_deref());
        let mut children = Children::new(element, &[(Ns::Podcast, "valueTimeSplit")], names, None);
        children.list(&self.recipients);
        children.list_before(&self.time_splits, &[]);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(&mut self.recipients, draft.map(|d| d.recipients.as_slice()))?;
        adopt_list(
            &mut self.time_splits,
            draft.map(|d| d.time_splits.as_slice()),
        )
    }
}

/// The meaning of the `fee` text of a recipient (musicindex ADR 0215 R-FEED-6
/// item 3, musicindex ADR 0086 R-FEED-4 item 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeReading {
    /// `true`, `1` or `yes`.
    Fee,
    /// `false`, `0`, `no`, an empty value or no attribute.
    NotFee,
    /// Each other value. It means not a fee, and the feed check gives the
    /// warning `unknown_fee` for it.
    Unknown,
}

impl FeeReading {
    /// Reads the `fee` text. The reader removes the spaces at the two ends
    /// and ignores the letter case. It does not change the text in the feed.
    #[must_use]
    pub fn of(text: Option<&str>) -> Self {
        let Some(text) = text.map(str::trim) else {
            return Self::NotFee;
        };
        let known = |words: &[&str]| words.iter().any(|w| text.eq_ignore_ascii_case(w));
        if known(&["true", "1", "yes"]) {
            Self::Fee
        } else if text.is_empty() || known(&["false", "0", "no"]) {
            Self::NotFee
        } else {
            Self::Unknown
        }
    }
}

/// The place of a recipient: the place of its value block, and the split of
/// the recipient in that block (ADR 0203 item 62).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecipientPlace {
    /// The place of the value block. Its fields are next to `percent` in
    /// JSON.
    #[serde(flatten)]
    pub place: CreditPlace,
    /// The split of the recipient in percent. `None` when the share is not
    /// a number. A place from before ADR 0203 has none.
    #[serde(default)]
    pub percent: Option<f64>,
}

/// `podcast:valueTimeSplit`: different recipients for a period of playback.
///
/// The specification permits one `podcast:remoteItem` or one or more
/// `podcast:valueRecipient` elements. The model keeps lists of both, so that
/// it also keeps a feed that does not obey this rule.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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
            start_time: element.attribute("startTime").map(str::to_owned),
            duration: element.attribute("duration").map(str::to_owned),
            remote_start_time: element.attribute("remoteStartTime").map(str::to_owned),
            remote_percentage: element.attribute("remotePercentage").map(str::to_owned),
            remote_items: read_list(element),
            recipients: read_list(element),
        }
    }

    fn apply(&self, element: &mut Element, names: &mut Names) {
        element.set_attribute("startTime", self.start_time.as_deref());
        element.set_attribute("duration", self.duration.as_deref());
        element.set_attribute("remoteStartTime", self.remote_start_time.as_deref());
        element.set_attribute("remotePercentage", self.remote_percentage.as_deref());
        let mut children = Children::new(element, &[], names, None);
        children.list(&self.remote_items);
        children.list(&self.recipients);
    }

    fn adopt(&mut self, draft: Option<&Self>) -> Result<(), KeyError> {
        adopt_list(
            &mut self.remote_items,
            draft.map(|d| d.remote_items.as_slice()),
        )?;
        adopt_list(&mut self.recipients, draft.map(|d| d.recipients.as_slice()))
    }
}
