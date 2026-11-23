//! The repairs of the feed check (ADR 0174 items 6 and 10 to 15).
//!
//! Each repair changes the draft through the model, so the diff shows only
//! this change. A repair writes no file. A repair that needs a read has
//! three steps, so that a caller sends the read with no lock of the
//! workspace: [`repair_step`] names the read, [`run_step`] sends it, and
//! [`apply_check_repair`] changes the draft.

use super::{
    Collection, DraftRow, Feed, FeedId, Hosts, PipelineError, PipelineErrorKind, changed_draft,
    current_draft, dated_like_base, draft_url, feed_move, revision, set_self_link,
};
use crate::digest::sha256_hex;
use crate::feed_check::{self, Finding, FindingKind, RepairKind};
use crate::feed_reads::{FileReads, https_of};
use crate::move_link::{FeedReader, OwnedDraftLinks};
use crate::xml::Document;

/// Correct a bad or missing medium through the draft (ADR 0250 item 7).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::StaleRevision`] when the draft changed.
/// Gives [`PipelineErrorKind::InvalidOperation`] for an unsupported medium
/// or a draft without a medium problem (ADR 0250 item 7).
pub fn correct_medium(
    collection: &Collection,
    id: FeedId,
    sent_revision: &str,
    medium: &str,
) -> Result<DraftRow, PipelineError> {
    if !["music", "musicL", "publisher"].contains(&medium) {
        return Err(PipelineError::new(
            PipelineErrorKind::InvalidOperation,
            "ADR 0250 item 7: correct_medium accepts only music, musicL or publisher",
        ));
    }
    let mut draft = checked_draft(collection, id, sent_revision)?;
    let mut document = Document::parse_bytes(&draft.draft_xml)?;
    let problem = crate::medium_problem::medium_problem(&document).ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::InvalidOperation,
            "ADR 0250 item 7: correct_medium needs a draft with a medium problem",
        )
    })?;
    let channel = document
        .root_mut()
        .children_mut()
        .iter_mut()
        .find_map(|node| match node {
            crate::xml::Node::Element(channel) if channel.is(crate::xml::Ns::None, "channel") => {
                Some(channel)
            }
            _ => None,
        })
        .ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::InvalidOperation,
                "ADR 0250 item 7: correct_medium needs an RSS channel",
            )
        })?;
    match problem.kind {
        crate::medium_problem::MediumProblemKind::MisspelledTag
        | crate::medium_problem::MediumProblemKind::OtherNamespace => {
            crate::medium_problem::replace_bad_elements(channel, problem.kind, medium);
        }
        crate::medium_problem::MediumProblemKind::MisspelledValue => {
            let element = channel
                .children_mut()
                .iter_mut()
                .find_map(|node| match node {
                    crate::xml::Node::Element(element)
                        if element.is(crate::xml::Ns::Podcast, "medium") =>
                    {
                        Some(element)
                    }
                    _ => None,
                })
                .ok_or_else(|| {
                    PipelineError::new(
                        PipelineErrorKind::InvalidOperation,
                        "ADR 0250 item 7: correct_medium could not find podcast:medium",
                    )
                })?;
            element.set_text(medium);
        }
        crate::medium_problem::MediumProblemKind::NoTag => {
            let mut feed = Feed::read(&document)?;
            feed.medium = Some(medium.to_owned());
            feed.write(&mut document)?;
        }
    }
    draft.draft_xml = document.write().into_bytes();
    save_repaired_draft(collection, id, &draft)?;
    Ok(draft)
}

/// The facts of a draft for the check, with its revision.
#[derive(Debug, Clone)]
pub struct CheckInputs {
    /// The facts of stage 1 of ADR 0126, with the draft as a typed feed.
    pub links: OwnedDraftLinks,
    /// The revision of the draft.
    pub revision: String,
    /// A key for the base file of the draft. The last network facts hold
    /// while the key stays the same (ADR 0174 item 18).
    pub base_key: String,
    /// The `lastBuildDate` text of the base file, that is, of the feed that
    /// Studio last wrote (ADR 0175 item 10).
    pub base_build_date: Option<String>,
    /// The feeds that remote items can name (ADR 0247 item 8).
    pub link_targets: Vec<feed_check::LinkTarget>,
    /// The GUID that Studio keeps for this feed (ADR 0249 item 6).
    pub known_guid: Option<String>,
    /// The GUID of the last hosted read (ADR 0249 item 6).
    pub read_guid: Option<String>,
}

/// The inputs of the check for the draft of a feed. The function makes no
/// network read, so a caller can hold a lock while it runs.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft, or the collection
/// or the host set fails.
pub fn check_inputs(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
) -> Result<CheckInputs, PipelineError> {
    let row = collection.feed(id)?;
    let draft = current_draft(collection, id)?;
    let link_targets = feed_check::link_targets(collection)?;
    Ok(CheckInputs {
        links: super::link_inputs_for(collection, hosts, id)?,
        revision: revision(&draft),
        base_key: sha256_hex(&[&draft.base_xml]),
        base_build_date: Document::parse_bytes(&draft.base_xml)
            .ok()
            .and_then(|doc| Feed::read(&doc).ok())
            .and_then(|base| base.last_build_date),
        link_targets,
        known_guid: row.known_guid,
        read_guid: row.podcast_guid,
    })
}

/// The read that a repair needs before it changes the draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// The repair needs no read.
    Nothing,
    /// Read the public address. The body there must be an RSS feed with the
    /// `podcast:guid` of the draft (ADR 0174 item 10).
    ConfirmAddress {
        /// The public address.
        address: String,
        /// The `podcast:guid` of the draft.
        guid: Option<String>,
    },
    /// Ask the host of an enclosure for its size (ADR 0174 item 15).
    ReadSize {
        /// The address of the enclosure.
        url: String,
    },
}

/// What a read proved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    /// The repair needs no read.
    Nothing,
    /// The body at the address is the feed of the draft.
    Address(String),
    /// The host gives this size for the address.
    Size {
        /// The address of the enclosure.
        url: String,
        /// The `Content-Length` of the host.
        length: u64,
    },
}

fn invalid(message: impl Into<String>) -> PipelineError {
    PipelineError::new(PipelineErrorKind::InvalidOperation, message)
}

/// The item of `entry` in `feed`, when `entry` names one.
fn item_of<'a>(feed: &'a Feed, entry: Option<&str>) -> Option<&'a crate::feed::Item> {
    let guid = entry?;
    feed.items
        .iter()
        .map(|e| &e.value)
        .find(|item| item.guid.as_deref() == Some(guid))
}

/// The read that the repair of `finding` needs.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::InvalidOperation`] when the finding has no
/// repair that changes the draft, or the draft has no element that the
/// finding names.
pub fn repair_step(inputs: &CheckInputs, finding: &Finding) -> Result<Step, PipelineError> {
    let links = &inputs.links;
    match finding.repair {
        Some(RepairKind::AddTheLink | RepairKind::UseThePublicAddress) => {
            let address = links
                .url
                .clone()
                .ok_or_else(|| invalid("the feed has no public address"))?;
            Ok(Step::ConfirmAddress {
                address,
                guid: links.feed.podcast_guid.clone(),
            })
        }
        Some(
            RepairKind::UseTheHttpsAddress
            | RepairKind::UseLnurlAddress
            | RepairKind::UseNodeAddress
            | RepairKind::AddHttps
            | RepairKind::UseTheGuidOf
            | RepairKind::PutBackGuid
            | RepairKind::KeepNewGuid,
        ) => Ok(Step::Nothing),
        Some(RepairKind::ReadTheSize) => {
            let url = item_of(&links.feed, finding.entry.as_deref())
                .and_then(|item| item.enclosure.as_ref())
                .and_then(|enclosure| enclosure.url.clone())
                .ok_or_else(|| invalid("the finding names no enclosure"))?;
            Ok(Step::ReadSize { url })
        }
        _ => Err(invalid("this finding has no repair that changes the draft")),
    }
}

/// Send the read of `step`. The function needs no lock of the workspace.
///
/// # Errors
///
/// For [`Step::ConfirmAddress`], gives [`PipelineErrorKind::InvalidOperation`]
/// with the reason when Studio cannot read the address, the body is not an
/// RSS feed, or the body has another `podcast:guid`. For
/// [`Step::ReadSize`], gives the same kind when the host gives no size.
pub fn run_step(
    step: &Step,
    feeds: &dyn FeedReader,
    files: &dyn FileReads,
) -> Result<Proof, PipelineError> {
    match step {
        Step::Nothing => Ok(Proof::Nothing),
        Step::ConfirmAddress { address, guid } => {
            let Some(guid) = guid else {
                return Err(invalid(
                    "the feed has no podcast:guid, so Studio cannot confirm its address",
                ));
            };
            let body = feeds.read(address).map_err(|_| {
                invalid(format!(
                    "Studio could not read {address}, so it did not change the link"
                ))
            })?;
            if body.podcast_guid.as_deref() != Some(guid.as_str()) {
                return Err(invalid(format!(
                    "the feed at {address} has a different podcast:guid, so Studio did not change the link"
                )));
            }
            Ok(Proof::Address(address.clone()))
        }
        Step::ReadSize { url } => match files.size(url) {
            Ok(Some(length)) if length > 0 => Ok(Proof::Size {
                url: url.clone(),
                length,
            }),
            Ok(_) => Err(invalid(format!("the host of {url} gives no file size"))),
            Err(_) => Err(invalid(format!("Studio could not read {url}"))),
        },
    }
}

/// Change the draft of a feed for the repair of `finding` (ADR 0174 item
/// 6). The draft changes only the elements that the finding names.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::StaleRevision`] when `sent_revision` is not
/// the revision of the draft, and [`PipelineErrorKind::InvalidOperation`]
/// when `proof` does not fit the repair or the repair does not apply. The
/// draft then does not change.
pub fn apply_check_repair(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    finding: &Finding,
    sent_revision: &str,
    proof: &Proof,
) -> Result<DraftRow, PipelineError> {
    let row = collection.feed(id)?;
    let draft = checked_draft(collection, id, sent_revision)?;
    if finding.repair == Some(RepairKind::KeepNewGuid) {
        let Some(guid) = row.podcast_guid.as_deref() else {
            return Err(invalid("this repair does not apply to the draft"));
        };
        if crate::pipeline::has_placeholder_guid(guid) || Some(guid) == row.known_guid.as_deref() {
            return Err(invalid("this repair does not apply to the draft"));
        }
        collection.set_known_guid(id, guid)?;
        return Ok(draft);
    }
    let moved = feed_move(collection, hosts, &row)?;
    let url = draft_url(hosts, &row, moved.as_ref())?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let targets = feed_check::link_targets(collection)?;
    let plan = plan_of(
        &feed,
        finding,
        url.as_deref(),
        proof,
        &targets,
        row.known_guid.as_deref(),
    )?;
    let before_xml = draft.draft_xml.clone();
    let changed = changed_draft(draft, |feed| plan.apply(feed))?;
    let changed = dated_like_base(changed, &before_xml)?;
    save_repaired_draft(collection, id, &changed)?;
    Ok(changed)
}

fn checked_draft(
    collection: &Collection,
    id: FeedId,
    sent_revision: &str,
) -> Result<DraftRow, PipelineError> {
    let draft = current_draft(collection, id)?;
    if revision(&draft) != sent_revision {
        return Err(PipelineError::new(
            PipelineErrorKind::StaleRevision,
            "the draft changed after this model was read; read the draft again",
        ));
    }
    Ok(draft)
}

fn save_repaired_draft(
    collection: &Collection,
    id: FeedId,
    draft: &DraftRow,
) -> Result<(), PipelineError> {
    collection.save_draft(id, draft)?;
    Ok(())
}

/// The change of one repair, after its checks.
enum Plan {
    /// Add or set the self link, and set the tag when `tag` is true.
    Links {
        address: String,
        link: bool,
        tag: bool,
    },
    /// Change the `http` address of an enclosure.
    Enclosure { guid: String, url: String },
    /// Change the `http` address of the feed image.
    Image { url: String },
    /// Set the length of an enclosure.
    Length { guid: String, length: u64 },
    RecipientType {
        address: String,
        recipient_type: &'static str,
    },
    /// Set an address field to its text with `https://` in front (ADR 0245
    /// item 2).
    FullAddress { field: String, address: String },
    /// Correct links that name this feed by address (ADR 0247 item 8).
    LinkGuid {
        url: String,
        targets: Vec<feed_check::LinkTarget>,
    },
    /// Restore the GUID that Studio keeps (ADR 0249 item 6).
    FeedGuid { guid: String },
}

impl Plan {
    fn apply(&self, feed: &mut Feed) {
        match self {
            Self::Links { address, link, tag } => {
                if *link {
                    set_self_link(feed, address);
                }
                if *tag {
                    feed.new_feed_url = Some(address.clone());
                }
            }
            Self::Enclosure { guid, url } => {
                let secure = https_of(url);
                if let Some(enclosure) = feed
                    .items
                    .iter_mut()
                    .find(|e| e.value.guid.as_deref() == Some(guid.as_str()))
                    .and_then(|e| e.value.enclosure.as_mut())
                {
                    enclosure.url = secure;
                }
            }
            Self::Image { url } => {
                let secure = https_of(url);
                if feed.image.as_deref() == Some(url.as_str()) {
                    feed.image.clone_from(&secure);
                }
                if let Some(image) = feed.rss_image.as_mut()
                    && image.url.as_deref() == Some(url.as_str())
                {
                    image.url = secure;
                }
            }
            Self::Length { guid, length } => {
                if let Some(enclosure) = feed
                    .items
                    .iter_mut()
                    .find(|e| e.value.guid.as_deref() == Some(guid.as_str()))
                    .and_then(|e| e.value.enclosure.as_mut())
                {
                    enclosure.length = Some(length.to_string());
                }
            }
            Self::FullAddress { field, address } => {
                // The plan found the field, so the setter cannot fail.
                let _ = feed.set_address(field, address);
            }
            Self::RecipientType {
                address,
                recipient_type,
            } => {
                let change = |recipients: &mut [crate::feed::Entry<crate::feed::Recipient>]| {
                    for recipient in recipients {
                        if recipient.value.address.as_deref() == Some(address.as_str())
                            && ((*recipient_type == "lnaddress"
                                && recipient.value.recipient_type.as_deref() == Some("node"))
                                || (*recipient_type == "node"
                                    && recipient.value.recipient_type.as_deref()
                                        == Some("lnaddress")))
                        {
                            recipient.value.recipient_type = Some((*recipient_type).to_owned());
                        }
                    }
                };
                for block in &mut feed.values {
                    change(&mut block.value.recipients);
                }
                for item in &mut feed.items {
                    for block in &mut item.value.values {
                        change(&mut block.value.recipients);
                    }
                }
            }
            Self::LinkGuid { url, targets } => {
                if let Some(target) = targets.iter().find(|target| target.url == *url) {
                    let _ = feed_check::correct_old_links(feed, targets, Some(&target.guid));
                }
            }
            Self::FeedGuid { guid } => feed.podcast_guid = Some(guid.clone()),
        }
    }
}

fn plan_of(
    feed: &Feed,
    finding: &Finding,
    url: Option<&str>,
    proof: &Proof,
    targets: &[feed_check::LinkTarget],
    known_guid: Option<&str>,
) -> Result<Plan, PipelineError> {
    match finding.repair {
        Some(RepairKind::PutBackGuid) => put_back_guid_plan(feed, known_guid),
        Some(RepairKind::AddTheLink) => {
            let address = confirmed(proof, url)?;
            if feed.self_url().is_some() {
                return Err(invalid("the feed has a self link already"));
            }
            Ok(Plan::Links {
                address,
                link: true,
                tag: false,
            })
        }
        Some(RepairKind::UseThePublicAddress) => {
            let address = confirmed(proof, url)?;
            let own = feed.self_url();
            let tag = feed.new_feed_url.as_deref();
            let (link, tag) = match finding.kind {
                FindingKind::SelfLinkDiffers => (own.is_some_and(|l| l != address), false),
                // The tag stays. Only its spelling changes (ADR 0128).
                FindingKind::SelfLinkSpelling => (
                    own.is_some_and(|l| feed_check::same_but_spelling(l, &address)),
                    tag.is_some_and(|t| feed_check::same_but_spelling(t, &address)),
                ),
                _ => (false, false),
            };
            if !link && !tag {
                return Err(invalid("this repair does not apply to the draft"));
            }
            Ok(Plan::Links { address, link, tag })
        }
        Some(RepairKind::UseTheHttpsAddress) => {
            let named = finding
                .url
                .clone()
                .filter(|u| u.starts_with("http://"))
                .ok_or_else(|| invalid("the finding names no http address"))?;
            if let Some(guid) = finding.entry.clone() {
                let current = item_of(feed, Some(&guid))
                    .and_then(|item| item.enclosure.as_ref())
                    .and_then(|enclosure| enclosure.url.as_deref());
                if current != Some(named.as_str()) {
                    return Err(invalid("the enclosure does not have this address"));
                }
                Ok(Plan::Enclosure { guid, url: named })
            } else {
                let rss = feed.rss_image.as_ref().and_then(|i| i.url.as_deref());
                if feed.image.as_deref() != Some(named.as_str()) && rss != Some(named.as_str()) {
                    return Err(invalid("the feed image does not have this address"));
                }
                Ok(Plan::Image { url: named })
            }
        }
        Some(RepairKind::ReadTheSize) => {
            let Proof::Size { url: read, length } = proof else {
                return Err(invalid("this repair needs a read of the size"));
            };
            let guid = finding
                .entry
                .clone()
                .ok_or_else(|| invalid("the finding names no track"))?;
            let current = item_of(feed, Some(&guid))
                .and_then(|item| item.enclosure.as_ref())
                .and_then(|enclosure| enclosure.url.as_deref());
            if current != Some(read.as_str()) {
                return Err(invalid("the enclosure does not have this address"));
            }
            Ok(Plan::Length {
                guid,
                length: *length,
            })
        }
        Some(RepairKind::UseTheGuidOf) => {
            let url = finding
                .url
                .clone()
                .ok_or_else(|| invalid("the finding names no feed address"))?;
            let target = targets
                .iter()
                .find(|target| target.url == url)
                .ok_or_else(|| invalid("this repair does not apply to the draft"))?;
            if feed_check::old_links(feed, targets)
                .iter()
                .all(|link| link.target != *target)
            {
                return Err(invalid("this repair does not apply to the draft"));
            }
            Ok(Plan::LinkGuid {
                url,
                targets: targets.to_vec(),
            })
        }
        Some(RepairKind::UseLnurlAddress | RepairKind::UseNodeAddress) => {
            recipient_type_plan(feed, finding)
        }
        Some(RepairKind::AddHttps) => full_address_plan(feed, finding),
        _ => Err(invalid("this finding has no repair that changes the draft")),
    }
}

/// The plan of the repair `AddHttps`. It refuses when the field of the draft
/// does not have the text of the finding any more (ADR 0245 item 2).
fn put_back_guid_plan(feed: &Feed, known_guid: Option<&str>) -> Result<Plan, PipelineError> {
    let guid = known_guid.ok_or_else(|| invalid("this repair does not apply to the draft"))?;
    if feed.podcast_guid.as_deref() == Some(guid) {
        return Err(invalid("this repair does not apply to the draft"));
    }
    Ok(Plan::FeedGuid {
        guid: guid.to_owned(),
    })
}

fn full_address_plan(feed: &Feed, finding: &Finding) -> Result<Plan, PipelineError> {
    let (Some(field), Some(named)) = (finding.field.clone(), finding.url.as_deref()) else {
        return Err(invalid("the finding names no address field"));
    };
    let current = feed
        .bad_addresses()
        .into_iter()
        .find(|bad| bad.field == field)
        .filter(|bad| bad.value == named)
        .ok_or_else(|| invalid("the field does not have this address any more"))?;
    let address = crate::feed::short_address_repair(&current.value)
        .ok_or_else(|| invalid("Studio cannot complete this address"))?;
    Ok(Plan::FullAddress { field, address })
}

fn recipient_type_plan(feed: &Feed, finding: &Finding) -> Result<Plan, PipelineError> {
    let address = finding
        .url
        .clone()
        .ok_or_else(|| invalid("the finding names no recipient address"))?;
    let target = if finding.repair == Some(RepairKind::UseLnurlAddress) {
        "lnaddress"
    } else {
        "node"
    };
    let old = if target == "lnaddress" {
        "node"
    } else {
        "lnaddress"
    };
    let matches = feed
        .values
        .iter()
        .flat_map(|v| &v.value.recipients)
        .chain(
            feed.items
                .iter()
                .flat_map(|i| i.value.values.iter().flat_map(|v| &v.value.recipients)),
        )
        .any(|r| {
            r.value.address.as_deref() == Some(address.as_str())
                && r.value.recipient_type.as_deref() == Some(old)
        });
    if !matches {
        return Err(invalid("this repair does not apply to the draft"));
    }
    Ok(Plan::RecipientType {
        address,
        recipient_type: target,
    })
}

/// The address that a read confirmed, when it is the URL of the draft.
fn confirmed(proof: &Proof, url: Option<&str>) -> Result<String, PipelineError> {
    match proof {
        Proof::Address(address) if Some(address.as_str()) == url => Ok(address.clone()),
        _ => Err(invalid("Studio did not confirm the public address")),
    }
}
