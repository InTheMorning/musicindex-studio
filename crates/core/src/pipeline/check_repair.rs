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
    let draft = current_draft(collection, id)?;
    Ok(CheckInputs {
        links: super::link_inputs_for(collection, hosts, id)?,
        revision: revision(&draft),
        base_key: sha256_hex(&[&draft.base_xml]),
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
        Some(RepairKind::UseTheHttpsAddress) => Ok(Step::Nothing),
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
    let draft = current_draft(collection, id)?;
    if revision(&draft) != sent_revision {
        return Err(PipelineError::new(
            PipelineErrorKind::StaleRevision,
            "the draft changed after this model was read; read the draft again",
        ));
    }
    let moved = feed_move(collection, hosts, &row)?;
    let url = draft_url(hosts, &row, moved.as_ref())?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let plan = plan_of(&feed, finding, url.as_deref(), proof)?;
    let before_xml = draft.draft_xml.clone();
    let changed = changed_draft(draft, |feed| plan.apply(feed))?;
    let changed = dated_like_base(changed, &before_xml)?;
    collection.save_draft(id, &changed)?;
    Ok(changed)
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
        }
    }
}

fn plan_of(
    feed: &Feed,
    finding: &Finding,
    url: Option<&str>,
    proof: &Proof,
) -> Result<Plan, PipelineError> {
    match finding.repair {
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
        _ => Err(invalid("this finding has no repair that changes the draft")),
    }
}

/// The address that a read confirmed, when it is the URL of the draft.
fn confirmed(proof: &Proof, url: Option<&str>) -> Result<String, PipelineError> {
    match proof {
        Proof::Address(address) if Some(address.as_str()) == url => Ok(address.clone()),
        _ => Err(invalid("Studio did not confirm the public address")),
    }
}
