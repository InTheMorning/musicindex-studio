//! The check of a feed (ADR 0174 items 1 to 12).
//!
//! The check examines a feed and gives a list of findings. It sends no
//! request and changes nothing. The caller gathers the facts of the reads
//! and gives them as data in [`Facts`].

use std::collections::{HashMap, HashSet};

use crate::collection::{Collection, PublisherRole, roles_from_rel};
use crate::feed::{Feed, RemoteItem};

mod index;

pub use index::{IndexFacts, IndexLine, copy_report, index_line, index_read};

/// The kind of a finding. Each kind is one row of the table in ADR 0174
/// item 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// A link has an unknown GUID and names a collection feed by address (ADR 0247 item 8).
    LinkGuidOld,
    /// The hosted feed GUID changed outside Studio (ADR 0249 item 6).
    GuidChangedOutside,
    /// The feed has a public address and no self link.
    NoSelfLink,
    /// The self link differs from the public address.
    SelfLinkDiffers,
    /// The self link or the tag differs from the public address only in its
    /// spelling (item 12).
    SelfLinkSpelling,
    /// An enclosure or an image has an `http` address, and the `https`
    /// address gives the same file.
    HttpFile,
    /// An enclosure has no `length`, or `length` is 0.
    NoFileSize,
    /// The host gives a `Content-Length` that differs from `length`.
    WrongFileSize,
    /// A track has no `itunes:duration`.
    NoDuration,
    /// The tag names a URL, and the body at that URL names a third URL.
    MoveNotConfirmed,
    /// The feed has no `language`.
    NoLanguage,
    /// The index holds the feed at an address other than the public
    /// address (ADR 0175 item 3).
    IndexAddressDiffers,
    /// An open copy of the feed has other payment recipients (ADR 0175
    /// item 6).
    CopyOtherRecipients,
    /// The release has no artist page (ADR 0213 item 18).
    NoArtistPage,
    /// The playlist or show has no publisher page (ADR 0213 item 18).
    NoPublisherPage,
    /// A music release has no tracks or live items.
    NoTracks,
    /// A music release has no cover image.
    NoCover,
    /// A track has no enclosure.
    NoAudioFile,
    /// A music release has no payment recipient.
    NoRecipient,
    /// A recipient address matches the other recipient type.
    WrongAddressType,
    /// A recipient has a `fee` value that is not known. Studio reads it as
    /// not a fee (musicindex ADR 0215 R-FEED-6 item 3, ADR 0244 item 2).
    UnknownFee,
    /// An address field has text with no `https://`, and the text with
    /// `https://` in front is a web address (ADR 0245 item 1).
    ShortAddress,
}

/// How a finding asks for action (ADR 0174 item 3, ADR 0175 item 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingLevel {
    /// Review refuses the write until the user repairs it.
    Stop,
    /// Money can go to other people. The user cannot repair it.
    Warning,
    /// The finding has a repair or a button that opens the field.
    Notice,
    /// The finding needs a value from the user.
    Question,
}

/// The repair that a finding offers (ADR 0174 item 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairKind {
    /// **Use the GUID of the linked feed** (ADR 0247 item 8).
    UseTheGuidOf,
    /// **Put Back** the known feed GUID (ADR 0249 item 6).
    PutBackGuid,
    /// **Keep the New GUID** (ADR 0249 item 6).
    KeepNewGuid,
    /// **Add the Link**.
    AddTheLink,
    /// **Use the Public Address**.
    UseThePublicAddress,
    /// **Use the https Address**.
    UseTheHttpsAddress,
    /// **Read the Size**.
    ReadTheSize,
    /// **Open the Track**.
    OpenTheTrack,
    /// **Select a Language**.
    SelectALanguage,
    /// **Link a Page** (ADR 0213 item 19).
    LinkAPage,
    /// **Add Tracks**.
    AddTracks,
    /// **Add a Cover**.
    AddACover,
    /// **Add a Recipient**.
    AddARecipient,
    /// **Use LNURL Address**.
    UseLnurlAddress,
    /// **Use Node Address**.
    UseNodeAddress,
    /// **Add https://** (ADR 0245 item 1).
    AddHttps,
}

/// The reason why a feed cannot be written under ADR 0213 item 21.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageRefusal {
    /// A release has no artist page.
    NoArtistPage,
    /// A playlist or show has no publisher page.
    NoPublisherPage,
}

impl PageRefusal {
    /// The API code for this refusal.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NoArtistPage => "no_artist_page",
            Self::NoPublisherPage => "no_publisher_page",
        }
    }
}

/// Give the page-rule refusal for `feed`, or none when its links obey
/// ADR 0213 item 17.
#[must_use]
pub fn page_refusal(feed: &Feed) -> Option<PageRefusal> {
    if feed.medium.as_deref() == Some("publisher") {
        return None;
    }
    let publisher_links = feed
        .publisher
        .as_ref()
        .map_or(&[][..], |publisher| publisher.remote_items.as_slice());
    let has_publisher = publisher_links
        .iter()
        .any(|link| link.value.medium.as_deref() == Some("publisher"));
    if feed.medium.as_deref() == Some("musicL") {
        return (!has_publisher).then_some(PageRefusal::NoPublisherPage);
    }
    if feed.medium.as_deref() != Some("music") {
        return None;
    }
    if !feed.items.is_empty() || feed.live_items.is_empty() {
        let has_artist = publisher_links.iter().any(|link| {
            link.value.medium.as_deref() == Some("publisher")
                && link
                    .value
                    .rel
                    .as_deref()
                    .is_some_and(|rel| roles_from_rel(rel).contains(&PublisherRole::Artist))
        });
        (!has_artist).then_some(PageRefusal::NoArtistPage)
    } else {
        (!has_publisher).then_some(PageRefusal::NoPublisherPage)
    }
}

/// One finding of the check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Finding {
    /// The kind of the finding.
    pub kind: FindingKind,
    /// The level of the finding.
    pub level: FindingLevel,
    /// The title for the user.
    pub title: String,
    /// The reason for the user.
    pub reason: String,
    /// The `guid` of the item that the finding is about. It is none for a
    /// finding about the feed.
    pub entry: Option<String>,
    /// The repair of the finding, when one exists.
    pub repair: Option<RepairKind>,
    /// The second repair when one exists (ADR 0249 item 6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_repair: Option<RepairKind>,
    /// The first eight characters of the known GUID (ADR 0249 item 6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_short: Option<String>,
    /// The address that the finding names, when it names one. A repair
    /// changes only this address.
    pub url: Option<String>,
    /// The model path of the address field that the finding is about
    /// (ADR 0245 item 2). It is none for the other findings.
    pub field: Option<String>,
    /// The place of that field in words for the user (ADR 0245 item 3).
    pub place: Option<String>,
    /// The title of the link target, when the finding names one (ADR 0247 item 8).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_title: Option<String>,
}

/// A collection feed that a remote item can name (ADR 0247 item 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkTarget {
    /// The current GUID of the target feed (ADR 0247 item 7).
    pub guid: String,
    /// The public address of the target feed (ADR 0247 item 7).
    pub url: String,
    /// The current title of the target feed (ADR 0247 item 8).
    pub title: Option<String>,
}

/// A remote item with an old GUID and its current target (ADR 0247 item 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldLink {
    /// The GUID that the remote item currently names (ADR 0247 item 7).
    pub old_guid: String,
    /// The collection feed that the remote item names by address (ADR 0247 item 7).
    pub target: LinkTarget,
}

/// Read the link targets of the collection without a request (ADR 0247 item 7).
///
/// # Errors
///
/// Gives an error when the collection or a feed identity cannot be read (ADR 0247 item 7).
pub fn link_targets(
    collection: &Collection,
) -> Result<Vec<LinkTarget>, crate::pipeline::PipelineError> {
    collection
        .feeds()?
        .into_iter()
        .try_fold(Vec::new(), |mut targets, row| {
            let (guid, title) = crate::pipeline::feed_identity(collection, &row)?;
            if let (Some(guid), Some(url)) = (guid, row.public_url) {
                targets.push(LinkTarget { guid, url, title });
            }
            Ok(targets)
        })
}

/// Give each old link in channel, podroll, and publisher order (ADR 0247 item 7).
#[must_use]
pub fn old_links(feed: &Feed, targets: &[LinkTarget]) -> Vec<OldLink> {
    let mut found = Vec::new();
    let links = feed
        .remote_items
        .iter()
        .chain(
            feed.podroll
                .iter()
                .flat_map(|podroll| &podroll.remote_items),
        )
        .chain(
            feed.publisher
                .iter()
                .flat_map(|publisher| &publisher.remote_items),
        );
    for entry in links {
        let item = &entry.value;
        let Some(old_guid) = item.feed_guid.as_deref() else {
            continue;
        };
        if targets.iter().any(|target| target.guid == old_guid) {
            continue;
        }
        for target in targets {
            let Some(url) = item.feed_url.as_deref() else {
                continue;
            };
            if (url == target.url || same_but_spelling(url, &target.url))
                && target.guid != old_guid
                && !found.iter().any(|old: &OldLink| old.target == *target)
            {
                found.push(OldLink {
                    old_guid: old_guid.to_owned(),
                    target: target.clone(),
                });
                break;
            }
        }
    }
    found
}

/// Correct old links in the selected target set (ADR 0247 item 7).
#[must_use]
pub fn correct_old_links(
    feed: &mut Feed,
    targets: &[LinkTarget],
    target_guid: Option<&str>,
) -> usize {
    let mut changed = 0;
    let fix = |item: &mut RemoteItem, changed: &mut usize| {
        let Some(guid) = item.feed_guid.as_deref() else {
            return;
        };
        if targets.iter().any(|target| target.guid == guid) {
            return;
        }
        let Some(url) = item.feed_url.as_deref() else {
            return;
        };
        if let Some(target) = targets.iter().find(|target| {
            target_guid.is_none_or(|guid| guid == target.guid)
                && target.guid != guid
                && (url == target.url || same_but_spelling(url, &target.url))
        }) {
            item.feed_guid = Some(target.guid.clone());
            *changed += 1;
        }
    };
    for entry in &mut feed.remote_items {
        fix(&mut entry.value, &mut changed);
    }
    if let Some(podroll) = &mut feed.podroll {
        for entry in &mut podroll.remote_items {
            fix(&mut entry.value, &mut changed);
        }
    }
    if let Some(publisher) = &mut feed.publisher {
        for entry in &mut publisher.remote_items {
            fix(&mut entry.value, &mut changed);
        }
    }
    changed
}

impl Finding {
    fn naming(mut self, url: &str) -> Self {
        self.url = Some(url.to_owned());
        self
    }
}

/// The facts that the caller gathers before the check (ADR 0174 item 4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// The feeds that remote items can name (ADR 0247 item 8).
    pub link_targets: Vec<LinkTarget>,
    /// The public address of the feed.
    pub public_address: Option<String>,
    /// The links that [`crate::move_link::check`] refuses. The check gives
    /// no finding for them (ADR 0174 item 5).
    pub refused_links: Vec<String>,
    /// True when the draft changes the self link of the base. ADR 0126
    /// item 6 then gives its notice in Review, and the check gives no
    /// `self_link_differs` finding (ADR 0174 item 7).
    pub self_changed: bool,
    /// The `Content-Length` of each enclosure URL, from a read.
    pub content_lengths: HashMap<String, u64>,
    /// The `http` URLs whose `https` address gives the same file
    /// (ADR 0174 item 14).
    pub same_https: HashSet<String>,
    /// The chain that the read of the tag followed. The first URL is the
    /// tag. The last URL is the end of the chain. It is none when no read
    /// ran.
    pub move_chain: Option<Vec<String>>,
    /// What the index holds for the feed (ADR 0175 item 2). It is none when
    /// no read ran, and always none for a feed with no `podcast:guid`,
    /// because the check sends no request for such a feed.
    pub index: Option<IndexFacts>,
    /// The GUID that Studio keeps for this feed (ADR 0249 item 6).
    pub known_guid: Option<String>,
    /// The GUID of the last hosted read (ADR 0249 item 6).
    pub read_guid: Option<String>,
}

/// Examine `feed` and give its findings. The function sends no request and
/// changes nothing.
#[must_use]
pub fn check(feed: &Feed, facts: &Facts) -> Vec<Finding> {
    let mut found = Vec::new();
    if let (Some(known), Some(read)) = (&facts.known_guid, &facts.read_guid)
        && !crate::pipeline::has_placeholder_guid(read)
        && read != known
        && feed.podcast_guid.as_deref() != Some(known.as_str())
    {
        let short: String = known.chars().take(8).collect();
        found.push(Finding {
            kind: FindingKind::GuidChangedOutside,
            level: FindingLevel::Warning,
            title: "The GUID changed outside Studio".to_owned(),
            reason: format!("Studio knows this feed as {known}. The hosted file now has {read}. Payments from other music feeds that name {known} stop. A second change has the same effect for feeds that name {read}."),
            entry: None,
            repair: Some(RepairKind::PutBackGuid),
            second_repair: Some(RepairKind::KeepNewGuid),
            known_short: Some(short),
            url: None, field: None, place: None, target_title: None,
        });
    }
    if let Some(refusal) = page_refusal(feed) {
        let (kind, title, reason) = match refusal {
            PageRefusal::NoArtistPage => (
                FindingKind::NoArtistPage,
                "No artist page",
                "This release has no artist page. Studio cannot write it before the link.",
            ),
            PageRefusal::NoPublisherPage => (
                FindingKind::NoPublisherPage,
                "No linked page",
                "This feed has no artist, label or hosting page. Studio cannot write it before the link.",
            ),
        };
        found.push(Finding {
            kind,
            level: FindingLevel::Stop,
            title: title.to_owned(),
            reason: reason.to_owned(),
            entry: None,
            repair: Some(RepairKind::LinkAPage),
            second_repair: None,
            known_short: None,
            url: None,
            field: None,
            place: None,
            target_title: None,
        });
    }
    check_old_links(feed, facts, &mut found);
    check_links(feed, facts, &mut found);
    check_release(feed, &mut found);
    check_move(feed, facts, &mut found);
    for entry in &feed.items {
        check_item(&entry.value, facts, &mut found);
    }
    check_recipient_types(feed, &mut found);
    check_fee_values(feed, &mut found);
    check_short_addresses(feed, &mut found);
    check_images(feed, facts, &mut found);
    index::check_index(facts, &mut found);
    if feed.language.as_deref().is_none_or(|l| l.trim().is_empty()) {
        found.push(Finding {
            kind: FindingKind::NoLanguage,
            level: FindingLevel::Question,
            title: "No language".to_owned(),
            reason: "Select the language of this feed.".to_owned(),
            entry: None,
            repair: Some(RepairKind::SelectALanguage),
            second_repair: None,
            known_short: None,
            url: None,
            field: None,
            place: None,
            target_title: None,
        });
    }
    found
}

fn notice(
    kind: FindingKind,
    title: String,
    reason: String,
    entry: Option<String>,
    repair: Option<RepairKind>,
) -> Finding {
    Finding {
        kind,
        level: FindingLevel::Notice,
        title,
        reason,
        entry,
        repair,
        second_repair: None,
        known_short: None,
        url: None,
        field: None,
        place: None,
        target_title: None,
    }
}

fn check_old_links(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    for link in old_links(feed, &facts.link_targets) {
        let name = link
            .target
            .title
            .clone()
            .unwrap_or_else(|| link.target.url.clone());
        let finding = Finding {
            kind: FindingKind::LinkGuidOld,
            level: FindingLevel::Warning,
            title: format!("A link to {name} has an old GUID"),
            reason: format!(
                "The link names the GUID {}. The feed {name} has the GUID {}.",
                link.old_guid, link.target.guid
            ),
            entry: None,
            repair: Some(RepairKind::UseTheGuidOf),
            second_repair: None,
            known_short: None,
            url: Some(link.target.url),
            field: None,
            place: None,
            target_title: Some(name),
        };
        found.push(finding);
    }
}

fn check_release(feed: &Feed, found: &mut Vec<Finding>) {
    if feed.medium.as_deref() != Some("music") {
        return;
    }
    if feed.items.is_empty() && feed.live_items.is_empty() {
        found.push(notice(
            FindingKind::NoTracks,
            "No tracks".into(),
            "Listeners have nothing to play.".into(),
            None,
            Some(RepairKind::AddTracks),
        ));
    }
    if feed.image.as_deref().is_none_or(str::is_empty)
        && feed
            .rss_image
            .as_ref()
            .and_then(|i| i.url.as_deref())
            .is_none_or(str::is_empty)
    {
        found.push(notice(
            FindingKind::NoCover,
            "No cover image".into(),
            "Apps show a blank square in place of the cover.".into(),
            None,
            Some(RepairKind::AddACover),
        ));
    }
    let has_recipient = feed.values.iter().any(|v| !v.value.recipients.is_empty())
        || feed.items.iter().any(|i| {
            i.value
                .values
                .iter()
                .any(|v| !v.value.recipients.is_empty())
        });
    if !has_recipient {
        found.push(notice(
            FindingKind::NoRecipient,
            "No payment recipient".into(),
            "Listeners cannot pay you.".into(),
            None,
            Some(RepairKind::AddARecipient),
        ));
    }
}

/// One warning for each recipient with a `fee` value that is not known
/// (ADR 0244 item 2). The warning changes no text and removes no recipient.
fn check_fee_values(feed: &Feed, found: &mut Vec<Finding>) {
    use crate::feed::{FeeReading, ValueBlock};
    fn recipients(block: &ValueBlock) -> impl Iterator<Item = &crate::feed::Recipient> {
        block
            .recipients
            .iter()
            .chain(block.time_splits.iter().flat_map(|s| &s.value.recipients))
            .map(|r| &r.value)
    }
    // `Feed::recipients` leaves out a time split and a recipient with no
    // address, so this walk is separate.
    let blocks = feed
        .values
        .iter()
        .chain(feed.items.iter().flat_map(|i| &i.value.values))
        .chain(feed.live_items.iter().flat_map(|l| &l.value.item.values))
        .map(|v| &v.value);
    let mut seen = HashSet::new();
    for recipient in blocks.flat_map(recipients) {
        let Some(text) = recipient.fee.as_deref() else {
            continue;
        };
        if FeeReading::of(Some(text)) != FeeReading::Unknown {
            continue;
        }
        let name = recipient
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or("This recipient");
        if !seen.insert((name, recipient.address.as_deref(), text)) {
            continue;
        }
        found.push(Finding {
            kind: FindingKind::UnknownFee,
            level: FindingLevel::Warning,
            title: format!("{name} has an unknown fee value"),
            reason: format!(
                "The fee value is \u{201c}{text}\u{201d}. Studio reads it as not a fee. Apps can read it in a different way."
            ),
            entry: None,
            repair: None,
            second_repair: None,
            known_short: None,
            url: None,
            field: None,
            place: None,
            target_title: None,
        });
    }
}

/// One finding for each address field that lacks `https://` and that the
/// repair can complete (ADR 0245 items 1 and 2).
fn check_short_addresses(feed: &Feed, found: &mut Vec<Finding>) {
    for bad in feed.bad_addresses() {
        if crate::feed::short_address_repair(&bad.value).is_none() {
            continue;
        }
        found.push(Finding {
            kind: FindingKind::ShortAddress,
            level: FindingLevel::Stop,
            title: format!("{} has an address that is not full", bad.place),
            reason: format!("{} has no https://. Studio cannot write it.", bad.value),
            entry: None,
            repair: Some(RepairKind::AddHttps),
            second_repair: None,
            known_short: None,
            url: Some(bad.value),
            field: Some(bad.field),
            place: Some(bad.place),
            target_title: None,
        });
    }
}

fn check_recipient_types(feed: &Feed, found: &mut Vec<Finding>) {
    use crate::keysend::{AddressFormat, address_format};
    let mut seen = HashSet::new();
    let mut recipients = feed.values.iter().flat_map(|v| &v.value.recipients).chain(
        feed.items
            .iter()
            .flat_map(|i| i.value.values.iter().flat_map(|v| &v.value.recipients)),
    );
    for recipient in &mut recipients {
        let Some(address) = recipient.value.address.as_deref() else {
            continue;
        };
        let Some(format) = address_format(address) else {
            continue;
        };
        let is_wrong = matches!(
            (recipient.value.recipient_type.as_deref(), format),
            (Some("node"), AddressFormat::Lnurl) | (Some("lnaddress"), AddressFormat::Node)
        );
        if !is_wrong {
            continue;
        }
        if !seen.insert(address) {
            continue;
        }
        let (type_name, article, repair) = match format {
            AddressFormat::Lnurl => ("LNURL address", "an", RepairKind::UseLnurlAddress),
            AddressFormat::Node => ("Node address", "a", RepairKind::UseNodeAddress),
        };
        let current = if format == AddressFormat::Lnurl {
            "Node address"
        } else {
            "LNURL address"
        };
        let name = recipient
            .value
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or("This recipient");
        found.push(
            notice(
                FindingKind::WrongAddressType,
                format!("{name} has the wrong address type"),
                format!("{address} is {article} {type_name}. The type is {current}."),
                None,
                Some(repair),
            )
            .naming(address),
        );
    }
}

fn is_refused(facts: &Facts, link: &str) -> bool {
    facts.refused_links.iter().any(|r| r == link)
}

/// The findings about the self link and the tag (items 7 and 12).
fn check_links(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let Some(public) = facts.public_address.as_deref() else {
        return;
    };
    let Some(link) = feed.self_url() else {
        found.push(notice(
            FindingKind::NoSelfLink,
            "No self link".to_owned(),
            "The feed does not state its address.".to_owned(),
            None,
            Some(RepairKind::AddTheLink),
        ));
        return;
    };
    let link_spelling = link != public && same_but_spelling(link, public);
    let tag = feed.new_feed_url.as_deref();
    let tag_spelling = tag.is_some_and(|t| t != public && same_but_spelling(t, public));
    if link != public && !link_spelling && !facts.self_changed && !is_refused(facts, link) {
        found.push(notice(
            FindingKind::SelfLinkDiffers,
            "The self link differs".to_owned(),
            format!("The self link names {link}. The public address is {public}."),
            None,
            Some(RepairKind::UseThePublicAddress),
        ));
    }
    let spelling_link = link_spelling && !is_refused(facts, link);
    let spelling_tag = tag_spelling && tag.is_some_and(|t| !is_refused(facts, t));
    if spelling_link || spelling_tag {
        let reason = match (spelling_link, spelling_tag) {
            (true, true) => {
                "The self link and the tag differ from the public address only in spelling."
            }
            (true, false) => "The self link differs from the public address only in spelling.",
            _ => "The tag differs from the public address only in spelling.",
        };
        found.push(notice(
            FindingKind::SelfLinkSpelling,
            "The address has a different spelling".to_owned(),
            reason.to_owned(),
            None,
            Some(RepairKind::UseThePublicAddress),
        ));
    }
}

/// The finding about the chain of the tag (item 13).
fn check_move(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let (Some(tag), Some(chain)) = (feed.new_feed_url.as_deref(), facts.move_chain.as_ref()) else {
        return;
    };
    let Some(end) = chain.last() else {
        return;
    };
    if end == tag || is_refused(facts, tag) {
        return;
    }
    found.push(notice(
        FindingKind::MoveNotConfirmed,
        "The move is not confirmed".to_owned(),
        format!("The tag names {tag}. The chain of addresses ends at {end}."),
        None,
        None,
    ));
}

/// The findings about one item (items 7 and 14).
fn check_item(item: &crate::feed::Item, facts: &Facts, found: &mut Vec<Finding>) {
    let track = item
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or("A track");
    let entry = item.guid.clone();
    let Some(enclosure) = &item.enclosure else {
        found.push(notice(
            FindingKind::NoAudioFile,
            format!("{track} has no audio file"),
            "Listeners cannot play this track.".into(),
            entry,
            Some(RepairKind::OpenTheTrack),
        ));
        return;
    };
    let length = enclosure.length.as_deref().map(str::trim);
    let size = length.and_then(|l| l.parse::<u64>().ok()).unwrap_or(0);
    if length.is_none_or(|l| l.is_empty() || l == "0") {
        let reason = if length == Some("0") {
            "The enclosure gives the size 0."
        } else {
            "The enclosure gives no size."
        };
        found.push(notice(
            FindingKind::NoFileSize,
            format!("{track} has no file size"),
            reason.to_owned(),
            entry.clone(),
            Some(RepairKind::ReadTheSize),
        ));
    } else if let Some(url) = enclosure.url.as_deref()
        && let Some(host_size) = facts.content_lengths.get(url)
        && *host_size != size
    {
        found.push(notice(
            FindingKind::WrongFileSize,
            format!("{track} has a wrong file size"),
            format!("The enclosure gives the size {size}. The host gives the size {host_size}."),
            entry.clone(),
            Some(RepairKind::ReadTheSize),
        ));
    }
    if let Some(url) = enclosure.url.as_deref()
        && facts.same_https.contains(url)
        && url.starts_with("http://")
    {
        found.push(
            notice(
                FindingKind::HttpFile,
                format!("{track} has an http file"),
                "The https address gives the same file.".to_owned(),
                entry.clone(),
                Some(RepairKind::UseTheHttpsAddress),
            )
            .naming(url),
        );
    }
    if item.duration.as_deref().is_none_or(|d| d.trim().is_empty()) {
        found.push(notice(
            FindingKind::NoDuration,
            format!("{track} has no duration"),
            "Apps cannot show the length of this track.".to_owned(),
            entry,
            Some(RepairKind::OpenTheTrack),
        ));
    }
}

/// The finding about the image of the feed (item 14). The `itunes:image`
/// and the RSS image can name the same URL. That URL gives one finding.
fn check_images(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let rss = feed.rss_image.as_ref().and_then(|i| i.url.as_deref());
    let mut urls = vec![feed.image.as_deref(), rss];
    urls.dedup();
    for url in urls.into_iter().flatten() {
        if url.starts_with("http://") && facts.same_https.contains(url) {
            found.push(
                notice(
                    FindingKind::HttpFile,
                    "The image has an http address".to_owned(),
                    "The https address gives the same file.".to_owned(),
                    None,
                    Some(RepairKind::UseTheHttpsAddress),
                )
                .naming(url),
            );
        }
    }
}

/// True when `a` and `b` differ only in spelling (ADR 0174 item 12). The
/// two URLs are equal after these changes: lowercase the scheme and the
/// host, remove the default port, and give an empty path the path `/`.
/// Equal text is not a spelling difference.
#[must_use]
pub fn same_but_spelling(a: &str, b: &str) -> bool {
    if a == b {
        return false;
    }
    matches!((normalize(a), normalize(b)), (Some(x), Some(y)) if x == y)
}

/// The spelling form of an absolute `http` or `https` URL, or none.
fn normalize(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(end);
    let (userinfo, hostport) = authority
        .rsplit_once('@')
        .map_or(("", authority), |(u, h)| (u, h));
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) if !p.contains(']') && p.bytes().all(|c| c.is_ascii_digit()) => (h, p),
        _ => (hostport, ""),
    };
    if host.is_empty() {
        return None;
    }
    let default = if scheme == "http" { "80" } else { "443" };
    let mut out = format!("{scheme}://");
    if !userinfo.is_empty() {
        out.push_str(userinfo);
        out.push('@');
    }
    out.push_str(&host.to_ascii_lowercase());
    if !port.is_empty() && port != default {
        out.push(':');
        out.push_str(port);
    }
    if !tail.starts_with('/') {
        out.push('/');
    }
    out.push_str(tail);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{
        AtomLink, Enclosure, Entry, Item, LiveItem, Publisher, Recipient, RemoteItem, RssImage,
        ValueBlock,
    };

    const PUBLIC: &str = "https://music.example/feed.xml";

    fn facts() -> Facts {
        Facts {
            public_address: Some(PUBLIC.to_owned()),
            ..Facts::default()
        }
    }

    /// A feed with no condition of the table.
    fn clean() -> Feed {
        let mut feed = Feed {
            language: Some("en".to_owned()),
            ..Feed::default()
        };
        feed.atom_links.push(Entry::new(AtomLink {
            href: Some(PUBLIC.to_owned()),
            rel: Some("self".to_owned()),
            ..AtomLink::default()
        }));
        feed
    }

    #[test]
    fn outside_guid_change_gives_two_repairs_only_when_the_draft_has_not_put_it_back() {
        let mut feed = clean();
        feed.podcast_guid = Some("B-83685084-new".to_owned());
        let mut facts = facts();
        facts.known_guid = Some("A-83685084-known".to_owned());
        facts.read_guid = Some("B-83685084-new".to_owned());
        let found = check(&feed, &facts);
        let finding = found
            .iter()
            .find(|f| f.kind == FindingKind::GuidChangedOutside)
            .expect("ADR 0249 item 6: fix check when changed GUID facts do not give a finding");
        assert_eq!(
            finding.level,
            FindingLevel::Warning,
            "ADR 0249 item 6: fix check level"
        );
        assert_eq!(
            finding.title, "The GUID changed outside Studio",
            "ADR 0249 item 6: fix check title"
        );
        assert_eq!(
            finding.reason,
            "Studio knows this feed as A-83685084-known. The hosted file now has B-83685084-new. Payments from other music feeds that name A-83685084-known stop. A second change has the same effect for feeds that name B-83685084-new.",
            "ADR 0249 item 6: fix check reason"
        );
        assert_eq!(
            finding.repair,
            Some(RepairKind::PutBackGuid),
            "ADR 0249 item 6: fix check first repair"
        );
        assert_eq!(
            finding.second_repair,
            Some(RepairKind::KeepNewGuid),
            "ADR 0249 item 6: fix check second repair"
        );
        assert_eq!(
            finding.known_short.as_deref(),
            Some("A-836850"),
            "ADR 0249 item 6: fix check short GUID"
        );

        feed.podcast_guid = facts.known_guid.clone();
        assert!(
            !check(&feed, &facts)
                .iter()
                .any(|f| f.kind == FindingKind::GuidChangedOutside),
            "ADR 0249 item 6: fix check put-back condition"
        );
        feed.podcast_guid = facts.read_guid.clone();
        facts.read_guid = facts.known_guid.clone();
        assert!(
            !check(&feed, &facts)
                .iter()
                .any(|f| f.kind == FindingKind::GuidChangedOutside),
            "ADR 0249 item 6: fix check equal GUID condition"
        );
        facts.read_guid = Some("pending-outside".to_owned());
        assert!(
            !check(&feed, &facts)
                .iter()
                .any(|f| f.kind == FindingKind::GuidChangedOutside),
            "ADR 0249 item 6: fix check placeholder condition"
        );
        facts.read_guid = Some("B-83685084-new".to_owned());
        facts.known_guid = None;
        assert!(
            !check(&feed, &facts)
                .iter()
                .any(|f| f.kind == FindingKind::GuidChangedOutside),
            "ADR 0249 item 6: fix check missing-known condition"
        );
    }

    fn track(title: &str, length: Option<&str>, duration: Option<&str>) -> Entry<Item> {
        Entry::new(Item {
            title: Some(title.to_owned()),
            guid: Some(format!("guid-{title}")),
            enclosure: Some(Enclosure {
                url: Some(format!("http://files.example/{title}.mp3")),
                length: length.map(str::to_owned),
                media_type: Some("audio/mpeg".to_owned()),
            }),
            duration: duration.map(str::to_owned),
            ..Item::default()
        })
    }

    fn kinds(found: &[Finding]) -> Vec<FindingKind> {
        found.iter().map(|f| f.kind).collect()
    }

    fn publisher_link(rel: &str) -> Entry<RemoteItem> {
        Entry::new(RemoteItem {
            medium: Some("publisher".to_owned()),
            rel: Some(rel.to_owned()),
            ..RemoteItem::default()
        })
    }

    fn old_guid_link(guid: Option<&str>, url: Option<&str>) -> Entry<RemoteItem> {
        Entry::new(RemoteItem {
            feed_guid: guid.map(str::to_owned),
            feed_url: url.map(str::to_owned),
            title: Some("This Is It".to_owned()),
            medium: Some("music".to_owned()),
            rel: Some("artist".to_owned()),
            ..RemoteItem::default()
        })
    }

    fn old_link_facts() -> Facts {
        Facts {
            link_targets: vec![LinkTarget {
                guid: "new-guid".to_owned(),
                url: "https://host/a.xml".to_owned(),
                title: Some("This Is It".to_owned()),
            }],
            ..Facts::default()
        }
    }

    #[test]
    fn old_channel_link_gives_one_warning_and_duplicate_links_give_one_finding() {
        let mut feed = clean();
        feed.remote_items
            .push(old_guid_link(Some("old-guid"), Some("https://host/a.xml")));
        feed.remote_items
            .push(old_guid_link(Some("old-guid"), Some("https://host/a.xml")));
        let found: Vec<_> = check(&feed, &old_link_facts())
            .into_iter()
            .filter(|finding| finding.kind == FindingKind::LinkGuidOld)
            .collect();
        assert_eq!(
            found.len(),
            1,
            "ADR 0247 item 8: fix check_old_links to group links by target"
        );
        assert_eq!(
            found[0].level,
            FindingLevel::Warning,
            "ADR 0247 item 8: fix check_old_links level"
        );
        assert_eq!(
            found[0].title, "A link to This Is It has an old GUID",
            "ADR 0247 item 8: fix check_old_links title"
        );
        assert_eq!(
            found[0].reason,
            "The link names the GUID old-guid. The feed This Is It has the GUID new-guid.",
            "ADR 0247 item 8: fix check_old_links reason"
        );
        assert_eq!(
            found[0].url.as_deref(),
            Some("https://host/a.xml"),
            "ADR 0247 item 8: fix check_old_links URL"
        );
        assert_eq!(
            found[0].repair,
            Some(RepairKind::UseTheGuidOf),
            "ADR 0247 item 8: fix check_old_links repair"
        );
    }

    #[test]
    fn old_link_check_requires_an_unknown_guid_and_accepts_url_spelling() {
        let facts = old_link_facts();
        for (guid, url, expected) in [
            (Some("new-guid"), Some("https://host/a.xml"), false),
            (None, Some("https://host/a.xml"), false),
            (Some("old-guid"), Some("HTTPS://Host:443/a.xml"), true),
        ] {
            let mut feed = clean();
            feed.remote_items.push(old_guid_link(guid, url));
            let found = check(&feed, &facts)
                .iter()
                .any(|finding| finding.kind == FindingKind::LinkGuidOld);
            assert_eq!(
                found, expected,
                "ADR 0247 item 7: fix old_links GUID and spelling checks"
            );
        }
    }

    #[test]
    fn old_link_check_reads_podroll_and_publisher_links() {
        for publisher in [false, true] {
            let link = old_guid_link(Some("old-guid"), Some("https://host/a.xml"));
            let mut feed = clean();
            if publisher {
                feed.publisher = Some(Publisher {
                    remote_items: vec![link],
                });
            } else {
                feed.podroll = Some(crate::feed::Podroll {
                    remote_items: vec![link],
                });
            }
            assert!(
                check(&feed, &old_link_facts())
                    .iter()
                    .any(|finding| finding.kind == FindingKind::LinkGuidOld),
                "ADR 0247 item 7: fix old_links traversal"
            );
        }
    }

    #[test]
    fn page_findings_follow_adr_0213_for_releases_playlists_shows_and_publishers() {
        let mut release = clean();
        release.medium = Some("music".to_owned());
        release.items.push(track("One", Some("1"), Some("1")));
        let found = check(&release, &Facts::default());
        assert!(
            kinds(&found).contains(&FindingKind::NoArtistPage),
            "ADR 0213 item 18: add an artist page link to this release"
        );
        let finding = found
            .iter()
            .find(|f| f.kind == FindingKind::NoArtistPage)
            .unwrap();
        assert_eq!(
            finding.level,
            FindingLevel::Stop,
            "ADR 0213 item 19: show Link a Page as a repair"
        );
        assert_eq!(
            finding.repair,
            Some(RepairKind::LinkAPage),
            "ADR 0213 item 19: show Link a Page as a repair"
        );

        let mut empty_release = clean();
        empty_release.medium = Some("music".to_owned());
        assert_eq!(
            page_refusal(&empty_release),
            Some(PageRefusal::NoArtistPage),
            "ADR 0213 item 17: a music feed with no live show needs an artist page"
        );

        release.publisher = Some(Publisher {
            remote_items: vec![publisher_link("artist")],
        });
        assert_eq!(
            page_refusal(&release),
            None,
            "ADR 0213 item 17: a release with an artist link has no page refusal"
        );
        release.publisher = Some(Publisher {
            remote_items: vec![publisher_link("label")],
        });
        assert_eq!(
            page_refusal(&release),
            Some(PageRefusal::NoArtistPage),
            "ADR 0213 item 17: a label link does not satisfy the artist page rule"
        );

        let mut show = clean();
        show.medium = Some("music".to_owned());
        show.live_items.push(Entry::new(LiveItem::default()));
        assert_eq!(
            page_refusal(&show),
            Some(PageRefusal::NoPublisherPage),
            "ADR 0213 item 17: a show without a publisher link needs one"
        );
        assert_eq!(check(&show, &Facts::default())[0].level, FindingLevel::Stop);
        show.publisher = Some(Publisher {
            remote_items: vec![publisher_link("label")],
        });
        assert_eq!(
            page_refusal(&show),
            None,
            "ADR 0213 item 17: a show with a publisher link has no page refusal"
        );

        let mut playlist = clean();
        playlist.medium = Some("musicL".to_owned());
        assert_eq!(
            page_refusal(&playlist),
            Some(PageRefusal::NoPublisherPage),
            "ADR 0213 item 17: musicL without a publisher link needs one"
        );
        assert_eq!(
            check(&playlist, &Facts::default())[0].level,
            FindingLevel::Stop
        );
        playlist.publisher = Some(Publisher {
            remote_items: vec![publisher_link("hosting")],
        });
        assert_eq!(
            page_refusal(&playlist),
            None,
            "ADR 0213 item 17: musicL with a publisher link has no page refusal"
        );

        let mut publisher_feed = clean();
        publisher_feed.medium = Some("publisher".to_owned());
        assert_eq!(
            page_refusal(&publisher_feed),
            None,
            "ADR 0213 item 17: publisher feeds get no page finding"
        );
    }

    #[test]
    fn a_feed_with_no_condition_gives_no_finding() {
        let mut feed = clean();
        feed.items.push(track("One", Some("100"), Some("3:00")));
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn no_self_link_needs_a_public_address() {
        let mut feed = clean();
        feed.atom_links.clear();
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::NoSelfLink]);
        assert_eq!(found[0].level, FindingLevel::Notice);
        assert_eq!(found[0].repair, Some(RepairKind::AddTheLink));
        assert_eq!(found[0].entry, None);
        assert_eq!(kinds(&check(&feed, &Facts::default())), []);
    }

    #[test]
    fn a_different_self_link_gives_a_finding() {
        let mut feed = clean();
        feed.set_self_url("https://other.example/feed.xml");
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::SelfLinkDiffers]);
        assert_eq!(found[0].repair, Some(RepairKind::UseThePublicAddress));
        // A different path is not a spelling difference.
        feed.set_self_url("https://music.example/other.xml");
        assert_eq!(
            kinds(&check(&feed, &facts())),
            [FindingKind::SelfLinkDiffers]
        );
        // A draft that changes the self link gives no finding. Review gives
        // the notice of ADR 0126 item 6.
        let changed = Facts {
            self_changed: true,
            ..facts()
        };
        assert_eq!(kinds(&check(&feed, &changed)), []);
    }

    #[test]
    fn a_spelling_difference_of_the_self_link_gives_its_own_finding() {
        let mut feed = clean();
        feed.set_self_url("HTTPS://Music.Example:443/feed.xml");
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::SelfLinkSpelling]);
        assert_eq!(found[0].repair, Some(RepairKind::UseThePublicAddress));
    }

    #[test]
    fn a_spelling_difference_of_the_tag_gives_a_finding() {
        let mut feed = clean();
        feed.new_feed_url = Some("https://MUSIC.example/feed.xml".to_owned());
        assert_eq!(
            kinds(&check(&feed, &facts())),
            [FindingKind::SelfLinkSpelling]
        );
        // A tag that names the public address exactly is correct.
        feed.new_feed_url = Some(PUBLIC.to_owned());
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn the_spelling_rule_has_three_changes() {
        let p = "https://music.example/feed.xml";
        assert!(same_but_spelling("HTTPS://music.example/feed.xml", p));
        assert!(same_but_spelling("https://MUSIC.Example/feed.xml", p));
        assert!(same_but_spelling("https://music.example:443/feed.xml", p));
        assert!(same_but_spelling(
            "http://a.example:80/x",
            "http://a.example/x"
        ));
        assert!(same_but_spelling(
            "https://music.example",
            "https://music.example/"
        ));
        // Equal text, a different path, a different port and a different scheme.
        assert!(!same_but_spelling(p, p));
        assert!(!same_but_spelling("https://music.example/other.xml", p));
        assert!(!same_but_spelling("https://music.example/Feed.xml", p));
        assert!(!same_but_spelling("https://music.example:8443/feed.xml", p));
        assert!(!same_but_spelling("http://music.example/feed.xml", p));
        assert!(!same_but_spelling("not a url", p));
    }

    #[test]
    fn a_refused_link_gives_no_finding() {
        let other = "https://other.example/feed.xml";
        let mut feed = clean();
        feed.set_self_url(other);
        let mut facts = facts();
        facts.refused_links.push(other.to_owned());
        assert_eq!(kinds(&check(&feed, &facts)), []);

        let spelled = "https://MUSIC.example/feed.xml";
        feed.set_self_url(PUBLIC);
        feed.new_feed_url = Some(spelled.to_owned());
        facts.refused_links.push(spelled.to_owned());
        assert_eq!(kinds(&check(&feed, &facts)), []);
    }

    #[test]
    fn a_missing_or_zero_size_gives_a_finding() {
        for (length, reason) in [
            (None, "The enclosure gives no size."),
            (Some("0"), "The enclosure gives the size 0."),
        ] {
            let mut feed = clean();
            feed.items.push(track("One", length, Some("3:00")));
            let found = check(&feed, &facts());
            assert_eq!(kinds(&found), [FindingKind::NoFileSize]);
            assert_eq!(found[0].title, "One has no file size");
            assert_eq!(found[0].reason, reason);
            assert_eq!(found[0].entry.as_deref(), Some("guid-One"));
            assert_eq!(found[0].repair, Some(RepairKind::ReadTheSize));
        }
    }

    #[test]
    fn a_track_with_no_duration_gives_a_finding() {
        let mut feed = clean();
        feed.items.push(track("One", Some("5"), None));
        feed.items.push(track("Two", Some("5"), Some("  ")));
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::NoDuration; 2]);
        assert_eq!(
            found[0].reason,
            "Apps cannot show the length of this track."
        );
        assert_eq!(found[0].repair, Some(RepairKind::OpenTheTrack));
        // An item with no enclosure gives the audio file finding.
        let mut feed = clean();
        feed.items.push(Entry::new(Item::default()));
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::NoAudioFile]);
        assert_eq!(found[0].entry, None);
        assert_eq!(found[0].repair, Some(RepairKind::OpenTheTrack));
    }

    #[test]
    fn release_findings_apply_only_to_music_releases() {
        let mut feed = clean();
        feed.medium = Some("music".into());
        let found = check(&feed, &facts());
        assert!(found.iter().any(|f| f.kind == FindingKind::NoTracks));
        assert!(found.iter().any(|f| f.kind == FindingKind::NoCover));
        assert!(found.iter().any(|f| f.kind == FindingKind::NoRecipient));
        feed.live_items.push(Entry::new(LiveItem::default()));
        assert!(
            !check(&feed, &facts())
                .iter()
                .any(|f| f.kind == FindingKind::NoTracks)
        );
        feed.medium = Some("podcast".into());
        assert!(!check(&feed, &facts()).iter().any(|f| matches!(
            f.kind,
            FindingKind::NoTracks | FindingKind::NoCover | FindingKind::NoRecipient
        )));
    }

    #[test]
    fn recipient_type_finding_uses_the_inferred_format() {
        let mut feed = clean();
        let recipient = Recipient {
            name: Some("Payee".into()),
            recipient_type: Some("node".into()),
            address: Some("alice@example.com".into()),
            ..Recipient::default()
        };
        feed.values.push(Entry::new(ValueBlock {
            recipients: vec![Entry::new(recipient)],
            ..ValueBlock::default()
        }));
        let found = check(&feed, &facts());
        let finding = found
            .iter()
            .find(|f| f.kind == FindingKind::WrongAddressType)
            .unwrap();
        assert_eq!(finding.title, "Payee has the wrong address type");
        assert_eq!(
            finding.reason,
            "alice@example.com is an LNURL address. The type is Node address."
        );
        assert_eq!(finding.repair, Some(RepairKind::UseLnurlAddress));
    }

    #[test]
    fn recipient_with_no_recognized_address_format_gives_no_type_finding() {
        let mut feed = clean();
        let recipient = Recipient {
            recipient_type: Some("lnaddress".into()),
            address: Some("not an address".into()),
            ..Recipient::default()
        };
        feed.values.push(Entry::new(ValueBlock {
            recipients: vec![Entry::new(recipient)],
            ..ValueBlock::default()
        }));
        assert!(
            !check(&feed, &facts())
                .iter()
                .any(|finding| finding.kind == FindingKind::WrongAddressType)
        );
    }

    #[test]
    fn a_feed_with_no_language_gives_a_question() {
        for language in [None, Some(" ".to_owned())] {
            let mut feed = clean();
            feed.language = language;
            let found = check(&feed, &facts());
            assert_eq!(kinds(&found), [FindingKind::NoLanguage]);
            assert_eq!(found[0].level, FindingLevel::Question);
            assert_eq!(found[0].repair, Some(RepairKind::SelectALanguage));
        }
    }

    #[test]
    fn a_media_type_no_image_and_no_lock_give_no_finding() {
        let mut feed = clean();
        let mut item = track("One", Some("100"), Some("3:00"));
        item.value.enclosure.as_mut().unwrap().media_type = Some("audio/flac".to_owned());
        item.value.image = None;
        feed.items.push(item);
        feed.locked = None;
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn a_split_sum_other_than_100_gives_no_finding() {
        let mut feed = clean();
        let recipient = |split: &str| {
            Entry::new(Recipient {
                name: Some("One".to_owned()),
                split: Some(split.to_owned()),
                ..Recipient::default()
            })
        };
        feed.values.push(Entry::new(ValueBlock {
            recipients: vec![recipient("7"), recipient("3"), recipient("1")],
            ..ValueBlock::default()
        }));
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn an_http_image_gives_one_finding_for_one_address() {
        let url = "http://files.example/cover.jpg";
        let mut feed = clean();
        feed.image = Some(url.to_owned());
        feed.rss_image = Some(RssImage {
            url: Some(url.to_owned()),
            ..RssImage::default()
        });
        let mut facts = facts();
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.same_https.insert(url.to_owned());
        let found = check(&feed, &facts);
        assert_eq!(kinds(&found), [FindingKind::HttpFile]);
        assert_eq!(found[0].repair, Some(RepairKind::UseTheHttpsAddress));
    }

    #[test]
    fn the_network_facts_give_their_findings() {
        let mut feed = clean();
        feed.new_feed_url = Some("https://new.example/feed.xml".to_owned());
        feed.items.push(track("One", Some("100"), Some("3:00")));
        let url = "http://files.example/One.mp3".to_owned();
        let mut facts = facts();
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.content_lengths.insert(url.clone(), 100);
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.content_lengths.insert(url.clone(), 120);
        facts.same_https.insert(url);
        facts.move_chain = Some(vec![
            "https://new.example/feed.xml".to_owned(),
            "https://third.example/feed.xml".to_owned(),
        ]);
        let found = check(&feed, &facts);
        assert_eq!(
            kinds(&found),
            [
                FindingKind::MoveNotConfirmed,
                FindingKind::WrongFileSize,
                FindingKind::HttpFile
            ]
        );
        facts.move_chain = Some(vec!["https://new.example/feed.xml".to_owned()]);
        assert!(!kinds(&check(&feed, &facts)).contains(&FindingKind::MoveNotConfirmed));
    }
}
