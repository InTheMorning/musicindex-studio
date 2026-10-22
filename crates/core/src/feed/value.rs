//! Payment blocks: `podcast:value` and `podcast:valueTimeSplit`.

use super::elements::{Recipient, RemoteItem};
use super::sync::{Children, Entry, Modeled, Names, read_list};
use crate::xml::{Element, Ns};

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
        let mut children = Children::new(element, &[(Ns::Podcast, "valueTimeSplit")], names);
        children.list(&self.recipients);
        children.list_before(&self.time_splits, &[]);
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
        let mut children = Children::new(element, &[], names);
        children.list(&self.remote_items);
        children.list(&self.recipients);
    }
}
