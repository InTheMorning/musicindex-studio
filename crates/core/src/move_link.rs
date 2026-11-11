//! The links of a feed that a write follows (ADR 0126 stage 1).
//!
//! The module knows no host, no connector and no collection. It gets the
//! URLs and the draft from its caller, and it reads a public feed only
//! through [`FeedReader`].

use std::fmt;

use serde::Serialize;

use crate::discover::{self, FailKind, ReadFailure};
use crate::feed::Feed;
use crate::net::NetPolicy;
use crate::web::Web;

/// The maximum number of reads in one chain (ADR 0126 item 9, condition 1).
pub const MAX_HOPS: usize = 10;

/// The maximum number of reads of the destination in stage 2 (ADR 0126
/// item 15).
pub const STAGE_TWO_READS: usize = 3;

/// The seconds between two reads of the destination in stage 2 (ADR 0126
/// item 15).
pub const STAGE_TWO_WAIT_SECONDS: u64 = 10;

/// The URLs that a body names (ADR 0126 item 1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BodyNames {
    /// The URL in `itunes:new-feed-url`.
    pub tag: Option<String>,
    /// The URL in the `href` of the first `atom:link` with `rel="self"`.
    pub self_link: Option<String>,
}

impl BodyNames {
    /// The names of a feed.
    #[must_use]
    pub fn of(feed: &Feed) -> Self {
        Self {
            tag: feed.new_feed_url.clone(),
            self_link: feed.self_url().map(str::to_owned),
        }
    }
}

/// A feed body that [`FeedReader`] read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedBody {
    /// The URL of the body: the address after each redirect (ADR 0126
    /// item 3).
    pub url: String,
    /// The `podcast:guid` of the body.
    pub podcast_guid: Option<String>,
    /// The names of the body.
    pub names: BodyNames,
}

impl FeedBody {
    /// The facts of `feed`, which was read at `url`.
    #[must_use]
    pub fn of(feed: &Feed, url: String) -> Self {
        Self {
            url,
            podcast_guid: feed.podcast_guid.clone(),
            names: BodyNames::of(feed),
        }
    }
}

/// The read of a public feed (ADR 0126 item 7).
pub trait FeedReader: Send + Sync {
    /// Read the feed at `url`, with each redirect.
    ///
    /// # Errors
    ///
    /// Gives a [`ReadFailure`] when the read fails or the body is not an
    /// RSS feed. The kind [`FailKind::NotPublic`] means that the address is
    /// not a global address.
    fn read(&self, url: &str) -> Result<FeedBody, ReadFailure>;

    /// Wait before the next read of the destination in stage 2. The real
    /// reader waits [`STAGE_TWO_WAIT_SECONDS`]. A test reader does not wait.
    fn wait_between_reads(&self);
}

/// The real [`FeedReader`]. It reads through `discover::read_feed`.
#[derive(Debug, Clone)]
pub struct WebFeedReader {
    web: Web,
}

impl WebFeedReader {
    /// A reader with the policy `Public`, in both shells. The policy of the
    /// shell does not apply to this read (ADR 0126 item 7).
    #[must_use]
    pub fn public() -> Self {
        Self {
            web: Web::with_policy(&NetPolicy::public()),
        }
    }

    /// A reader with a client that the caller makes, for a test server.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_web(web: Web) -> Self {
        Self { web }
    }
}

impl FeedReader for WebFeedReader {
    fn read(&self, url: &str) -> Result<FeedBody, ReadFailure> {
        let public = discover::read_feed(&self.web, url)?;
        Ok(FeedBody::of(&public.feed, public.url))
    }

    fn wait_between_reads(&self) {
        std::thread::sleep(std::time::Duration::from_secs(STAGE_TWO_WAIT_SECONDS));
    }
}

/// The condition of a refusal (ADR 0126 item 9). The number is the number
/// of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Condition {
    /// 1: the chain comes back to a URL that it visited, or has more than
    /// 10 hops.
    Loop,
    /// 2: a followed URL does not answer with an RSS feed, or the read
    /// fails.
    ReadFails,
    /// 3: the body has a `podcast:guid` that is not the one of the draft.
    GuidMismatch,
    /// 4: a body names two different URLs, other than its own URL.
    ConflictingNames,
    /// 6: the draft changes the self link, and it differs from the URL of
    /// the draft.
    SelfLinkDiffers,
    /// 7: a followed URL or the destination is not a global address.
    NotPublic,
}

impl Condition {
    /// The number of the condition in ADR 0126 item 9.
    #[must_use]
    pub fn number(self) -> u8 {
        match self {
            Self::Loop => 1,
            Self::ReadFails => 2,
            Self::GuidMismatch => 3,
            Self::ConflictingNames => 4,
            Self::SelfLinkDiffers => 6,
            Self::NotPublic => 7,
        }
    }

    fn text(self) -> &'static str {
        match self {
            Self::Loop => {
                "the new addresses come back to an address that they visited, or go through more than 10 addresses"
            }
            Self::ReadFails => "an address does not give an RSS feed, or Studio cannot read it",
            Self::GuidMismatch => "a feed at an address has a different podcast:guid",
            Self::ConflictingNames => "a feed at an address names two different new addresses",
            Self::SelfLinkDiffers => {
                "the draft changes the self link to an address other than the address of the feed"
            }
            Self::NotPublic => "an address is not a public address",
        }
    }
}

/// The result of one hop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HopResult {
    /// The address gave an RSS feed that passes.
    Feed,
    /// The address is in the chain already, so Studio did not read it.
    Revisited,
    /// The chain has 10 hops, so Studio did not read this address.
    TooManyHops,
    /// The address gave a file that is not an RSS feed.
    NotAFeed,
    /// The address gave 404 or 410.
    Gone,
    /// The address gave a status that is not a success.
    ServerError,
    /// Studio could not connect.
    Unreachable,
    /// The file is too large.
    TooLarge,
    /// The text is not an `http` or `https` address.
    InvalidAddress,
    /// The address is not a global address.
    NotPublic,
    /// The feed has a different `podcast:guid`, or none.
    GuidMismatch,
    /// The feed at the destination has other names than the draft (stage 2).
    NamesDiffer,
    /// The feed names two different new addresses.
    ConflictingNames,
    /// The address is the self link, and it differs from the address of
    /// the draft. Studio did not read it.
    DiffersFromDraft,
}

/// One address of a chain with its result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hop {
    /// The address that Studio asked for.
    pub url: String,
    /// The address after each redirect, when it differs from `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    /// The result.
    pub result: HopResult,
}

impl Hop {
    fn new(url: &str, result: HopResult) -> Self {
        Self {
            url: url.to_owned(),
            final_url: None,
            result,
        }
    }
}

/// A repair that the core offers for one user action (ADR 0126 item 20).
/// Each repair changes the draft, so the diff shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Repair {
    /// Set the self link to the URL of the draft.
    UsePublicAddress,
    /// Remove `itunes:new-feed-url`.
    RemoveTag,
}

/// Where the first address of a refusal came from (ADR 0126 item 28).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LinkSource {
    /// `itunes:new-feed-url` of the draft.
    Tag,
    /// The self link of the draft.
    SelfLink,
    /// The destination of the move that the user selected.
    Move,
}

/// The refusal of a write: the condition and each hop (ADR 0126 item 10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadMoveLink {
    /// The condition.
    pub condition: Condition,
    /// Where the first address came from.
    pub source: LinkSource,
    /// The number of the condition.
    pub number: u8,
    /// The stage that refused: 1 before any file changes, 2 after the new
    /// feed file is on the destination (ADR 0126 item 13 and item 14).
    pub stage: u8,
    /// Each hop, in the sequence of the reads.
    pub hops: Vec<Hop>,
    /// The repairs that apply to the draft. The list is empty when no
    /// repair applies.
    pub repairs: Vec<Repair>,
}

impl BadMoveLink {
    fn new(condition: Condition, source: LinkSource, hops: Vec<Hop>) -> Self {
        Self {
            condition,
            source,
            number: condition.number(),
            stage: 1,
            hops,
            repairs: Vec::new(),
        }
    }

    fn with_repairs(mut self, repairs: Vec<Repair>) -> Self {
        self.repairs = repairs;
        self
    }
}

impl fmt::Display for BadMoveLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Studio refuses the write: {}", self.condition.text())?;
        if let Some(last) = self.hops.last() {
            write!(f, ". Last address: {}", last.url)?;
        }
        Ok(())
    }
}

/// A notice for Review that is not a refusal (ADR 0126 item 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "notice", rename_all = "snake_case")]
#[non_exhaustive]
pub enum LinkNotice {
    /// The self link names a different URL than the draft, and the draft
    /// does not change it. Studio does not read that URL.
    SelfLinkDiffers {
        /// The URL in the self link.
        self_url: String,
        /// The URL of the draft.
        draft_url: String,
    },
}

/// The facts of a draft that stage 1 examines.
#[derive(Debug, Clone, Copy)]
pub struct DraftLinks<'a> {
    /// The draft as a typed feed.
    pub feed: &'a Feed,
    /// The URL of the draft (ADR 0126 item 4): the destination of the move,
    /// else the public address. `None` for a feed on a file host with no
    /// public address.
    pub url: Option<&'a str>,
    /// True when the self link of the draft differs from the self link of
    /// its base.
    pub self_changed: bool,
    /// The destination of a move, when the draft moves the feed and the
    /// destination has a URL (ADR 0126 item 13).
    pub destination: Option<&'a str>,
}

/// The result of stage 1.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LinkReport {
    /// The refusal, when a condition holds.
    pub refusal: Option<BadMoveLink>,
    /// The notices for Review.
    pub notices: Vec<LinkNotice>,
    /// The repairs that apply: the repairs of the refusal, and
    /// `use_public_address` for a notice (ADR 0126 item 20).
    pub repairs: Vec<Repair>,
}

/// The facts of a draft that stage 1 examines, as owned values. The caller
/// makes them under the lock of the workspace. Then it releases the lock
/// and runs [`OwnedDraftLinks::report`], which reads the network.
#[derive(Debug, Clone)]
pub struct OwnedDraftLinks {
    /// The draft as a typed feed.
    pub feed: Feed,
    /// See [`DraftLinks::url`].
    pub url: Option<String>,
    /// See [`DraftLinks::self_changed`].
    pub self_changed: bool,
    /// See [`DraftLinks::destination`].
    pub destination: Option<String>,
}

impl OwnedDraftLinks {
    /// Run stage 1 on these facts. The call reads through `reader`.
    #[must_use]
    pub fn report(&self, reader: &dyn FeedReader) -> LinkReport {
        check(
            reader,
            &DraftLinks {
                feed: &self.feed,
                url: self.url.as_deref(),
                self_changed: self.self_changed,
                destination: self.destination.as_deref(),
            },
        )
    }
}

/// The repairs that apply to a draft with the URL `url` (ADR 0126 item 20).
/// `remove_tag` applies when the draft has the tag. `use_public_address`
/// applies when the draft has a URL and a self link that differs from it.
#[must_use]
pub fn applicable_repairs(feed: &Feed, url: Option<&str>) -> Vec<Repair> {
    let mut repairs = Vec::new();
    if repair_applies(Repair::UsePublicAddress, feed, url) {
        repairs.push(Repair::UsePublicAddress);
    }
    if repair_applies(Repair::RemoveTag, feed, url) {
        repairs.push(Repair::RemoveTag);
    }
    repairs
}

/// True when `repair` applies to the draft `feed` with the URL `url`.
#[must_use]
pub fn repair_applies(repair: Repair, feed: &Feed, url: Option<&str>) -> bool {
    match repair {
        Repair::UsePublicAddress => matches!(
            (feed.self_url(), url),
            (Some(self_url), Some(url)) if self_url != url
        ),
        // A tag that names the URL of the draft is the own-address tag of
        // ADR 0128. It is not a link to repair.
        Repair::RemoveTag => feed
            .new_feed_url
            .as_deref()
            .is_some_and(|tag| Some(tag) != url),
    }
}

/// Run stage 1 on a draft (ADR 0126 items 5 to 8 and 13).
///
/// The function makes no read when the draft has no tag that names a
/// different URL and no move. A move sends one read to the destination,
/// and the function acts only on the kind `NotPublic` of it.
#[must_use]
pub fn check(reader: &dyn FeedReader, links: &DraftLinks<'_>) -> LinkReport {
    let mut report = check_conditions(reader, links);
    if let Some(refusal) = &report.refusal {
        report.repairs.clone_from(&refusal.repairs);
    }
    if !report.notices.is_empty() && !report.repairs.contains(&Repair::UsePublicAddress) {
        report.repairs.push(Repair::UsePublicAddress);
    }
    report
}

fn check_conditions(reader: &dyn FeedReader, links: &DraftLinks<'_>) -> LinkReport {
    let names = BodyNames::of(links.feed);
    let mut report = LinkReport::default();

    // Item 6: a self link that the draft does not change gets a notice.
    if let (Some(self_url), Some(draft_url)) = (names.self_link.as_deref(), links.url)
        && self_url != draft_url
    {
        if links.self_changed {
            // Condition 6. The self link is not read, because it differs
            // from the URL of the draft already.
            report.refusal = Some(
                BadMoveLink::new(
                    Condition::SelfLinkDiffers,
                    LinkSource::SelfLink,
                    vec![Hop::new(self_url, HopResult::DiffersFromDraft)],
                )
                .with_repairs(vec![Repair::UsePublicAddress]),
            );
            return report;
        }
        report.notices.push(LinkNotice::SelfLinkDiffers {
            self_url: self_url.to_owned(),
            draft_url: draft_url.to_owned(),
        });
    }

    // Item 13: a move reads the destination only for condition 7.
    if let Some(destination) = links.destination
        && let Err(failure) = reader.read(destination)
        && failure.kind == FailKind::NotPublic
    {
        report.refusal = Some(BadMoveLink::new(
            Condition::NotPublic,
            LinkSource::Move,
            vec![Hop::new(destination, HopResult::NotPublic)],
        ));
        return report;
    }

    // Item 5: the tag is followed when it names a different URL. With no URL
    // of the draft, a self link that the draft changes is also followed.
    let tag = names.tag.as_deref().filter(|tag| Some(*tag) != links.url);
    let self_link = names
        .self_link
        .as_deref()
        .filter(|_| links.url.is_none() && links.self_changed);
    let start = match (tag, self_link) {
        // Condition 4 for the body of the draft.
        (Some(tag), Some(self_link)) if tag != self_link => {
            report.refusal = Some(
                BadMoveLink::new(
                    Condition::ConflictingNames,
                    LinkSource::Tag,
                    vec![
                        Hop::new(tag, HopResult::ConflictingNames),
                        Hop::new(self_link, HopResult::ConflictingNames),
                    ],
                )
                .with_repairs(vec![Repair::RemoveTag]),
            );
            return report;
        }
        (Some(start), _) | (None, Some(start)) => start,
        (None, None) => return report,
    };
    let mut visited: Vec<String> = links.url.map(str::to_owned).into_iter().collect();
    let mut hops = Vec::new();
    if let Err(condition) = walk(
        reader,
        links.feed.podcast_guid.as_deref(),
        start,
        &mut visited,
        &mut hops,
    ) {
        // The repair removes the tag only when the chain started from it.
        let (source, repairs) = if tag.is_some() {
            (LinkSource::Tag, vec![Repair::RemoveTag])
        } else {
            (LinkSource::SelfLink, Vec::new())
        };
        report.refusal = Some(BadMoveLink::new(condition, source, hops).with_repairs(repairs));
    }
    report
}

/// Run stage 2 on the destination of a move (ADR 0126 items 14 and 15).
///
/// The function reads `destination` up to [`STAGE_TWO_READS`] times and
/// waits between two reads through [`FeedReader::wait_between_reads`]. It
/// passes when a body is an RSS feed with the `podcast:guid` of `draft` and
/// the same names as `draft`, by exact text. A host or a CDN can serve an
/// earlier result, so a body that fails gets the next read.
///
/// # Errors
///
/// Gives the refusal of the last read. A read that fails, or a body with
/// other names than the draft, is condition 2. A body with a different
/// `podcast:guid` is condition 3. An address that is not public is
/// condition 7, and it gets no second read.
pub fn check_destination(
    reader: &dyn FeedReader,
    draft: &Feed,
    destination: &str,
) -> Result<(), BadMoveLink> {
    let names = BodyNames::of(draft);
    let mut last = None;
    for attempt in 0..STAGE_TWO_READS {
        if attempt > 0 {
            reader.wait_between_reads();
        }
        let mut hop = Hop::new(destination, HopResult::Feed);
        let condition = match reader.read(destination) {
            Err(failure) => {
                let (condition, result) = failure_result(failure.kind);
                hop.result = result;
                condition
            }
            Ok(body) => {
                if body.url != destination {
                    hop.final_url = Some(body.url.clone());
                }
                if draft
                    .podcast_guid
                    .as_deref()
                    .is_some_and(|guid| body.podcast_guid.as_deref() != Some(guid))
                {
                    hop.result = HopResult::GuidMismatch;
                    Condition::GuidMismatch
                } else if body.names != names {
                    hop.result = HopResult::NamesDiffer;
                    Condition::ReadFails
                } else {
                    return Ok(());
                }
            }
        };
        let mut refusal = BadMoveLink::new(condition, LinkSource::Move, vec![hop]);
        refusal.stage = 2;
        if condition == Condition::NotPublic {
            return Err(refusal);
        }
        refusal.repairs = applicable_repairs(draft, Some(destination));
        last = Some(refusal);
    }
    last.map_or(Ok(()), Err)
}

/// Walk a chain from `start` until a body that ends it, or a condition
/// holds (ADR 0126 item 4 and item 9). `visited` holds the URLs that the
/// chain visited, and the walk adds each one. `hops` gets each hop.
fn walk(
    reader: &dyn FeedReader,
    draft_guid: Option<&str>,
    start: &str,
    visited: &mut Vec<String>,
    hops: &mut Vec<Hop>,
) -> Result<(), Condition> {
    let mut next = start.to_owned();
    loop {
        if visited.contains(&next) {
            hops.push(Hop::new(&next, HopResult::Revisited));
            return Err(Condition::Loop);
        }
        if hops.len() >= MAX_HOPS {
            hops.push(Hop::new(&next, HopResult::TooManyHops));
            return Err(Condition::Loop);
        }
        let body = match reader.read(&next) {
            Ok(body) => body,
            Err(failure) => {
                let (condition, result) = failure_result(failure.kind);
                hops.push(Hop::new(&next, result));
                return Err(condition);
            }
        };
        visited.push(next.clone());
        let mut hop = Hop::new(&next, HopResult::Feed);
        if body.url != next {
            hop.final_url = Some(body.url.clone());
            if visited.contains(&body.url) {
                hop.result = HopResult::Revisited;
                hops.push(hop);
                return Err(Condition::Loop);
            }
            visited.push(body.url.clone());
        }
        // Condition 3.
        if draft_guid.is_some_and(|guid| body.podcast_guid.as_deref() != Some(guid)) {
            hop.result = HopResult::GuidMismatch;
            hops.push(hop);
            return Err(Condition::GuidMismatch);
        }
        // Condition 4: the names other than the own URL of the body.
        let mut others: Vec<&str> = Vec::new();
        for name in [body.names.tag.as_deref(), body.names.self_link.as_deref()]
            .into_iter()
            .flatten()
        {
            if name != next && name != body.url && !others.contains(&name) {
                others.push(name);
            }
        }
        if others.len() > 1 {
            hop.result = HopResult::ConflictingNames;
            hops.push(hop);
            return Err(Condition::ConflictingNames);
        }
        hops.push(hop);
        match others.first() {
            // A body that names nothing else is the end of the chain.
            None => return Ok(()),
            Some(name) => (*name).clone_into(&mut next),
        }
    }
}

/// The result of [`tag_chain`]: each hop, and the condition when the chain
/// stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagChain {
    /// Each hop, in the sequence of the reads.
    pub hops: Vec<Hop>,
    /// The condition that stopped the chain, or none when the chain ended
    /// at a body that names nothing else.
    pub stopped: Option<Condition>,
}

impl TagChain {
    /// The URL of each hop. The first URL is the start.
    #[must_use]
    pub fn urls(&self) -> Vec<String> {
        self.hops.iter().map(|hop| hop.url.clone()).collect()
    }
}

/// Follow the chain that starts at `start` with the rules of stage 1, for
/// the check of a feed (ADR 0174 item 13). `url` is the URL of the draft.
/// It counts as visited. The function does not change the refusals of
/// [`check`].
#[must_use]
pub fn tag_chain(
    reader: &dyn FeedReader,
    draft_guid: Option<&str>,
    start: &str,
    url: Option<&str>,
) -> TagChain {
    let mut visited: Vec<String> = url.map(str::to_owned).into_iter().collect();
    let mut hops = Vec::new();
    let stopped = walk(reader, draft_guid, start, &mut visited, &mut hops).err();
    TagChain { hops, stopped }
}

/// The condition and the hop result for a failed read.
fn failure_result(kind: FailKind) -> (Condition, HopResult) {
    match kind {
        FailKind::NotPublic => (Condition::NotPublic, HopResult::NotPublic),
        FailKind::NotAFeed => (Condition::ReadFails, HopResult::NotAFeed),
        FailKind::Gone => (Condition::ReadFails, HopResult::Gone),
        FailKind::ServerError => (Condition::ReadFails, HopResult::ServerError),
        FailKind::TooLarge => (Condition::ReadFails, HopResult::TooLarge),
        FailKind::InvalidAddress => (Condition::ReadFails, HopResult::InvalidAddress),
        _ => (Condition::ReadFails, HopResult::Unreachable),
    }
}

/// A [`FeedReader`] for a test. It serves a list of results for each URL
/// and records each read, so that a test can prove that no read occurred.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Default)]
pub struct ScriptedReader {
    script: std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            std::collections::VecDeque<Result<FeedBody, ReadFailure>>,
        >,
    >,
    reads: std::sync::Mutex<Vec<String>>,
    waits: std::sync::atomic::AtomicUsize,
}

#[cfg(any(test, feature = "testing"))]
impl ScriptedReader {
    /// A reader with no body. Each read gives the kind `Unreachable`.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Serve `body` at `url` for each read.
    #[must_use]
    pub fn with_body(self, url: &str, body: FeedBody) -> Self {
        self.with_results(url, vec![Ok(body)])
    }

    /// Serve a feed at `url` that has `guid`, the tag and the self link.
    /// `final_url` is the address after the redirects.
    #[must_use]
    pub fn with_feed(
        self,
        url: &str,
        final_url: &str,
        guid: Option<&str>,
        tag: Option<&str>,
        self_link: Option<&str>,
    ) -> Self {
        self.with_body(
            url,
            FeedBody {
                url: final_url.to_owned(),
                podcast_guid: guid.map(str::to_owned),
                names: BodyNames {
                    tag: tag.map(str::to_owned),
                    self_link: self_link.map(str::to_owned),
                },
            },
        )
    }

    /// Serve `results` at `url`, one for each read. The last result stays
    /// for each later read.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the script.
    #[must_use]
    pub fn with_results(self, url: &str, results: Vec<Result<FeedBody, ReadFailure>>) -> Self {
        self.script
            .lock()
            .expect("the script lock is not poisoned")
            .insert(url.to_owned(), results.into());
        self
    }

    /// Serve the failure `kind` at `url`.
    #[must_use]
    pub fn with_failure(self, url: &str, kind: FailKind) -> Self {
        self.with_results(
            url,
            vec![Err(ReadFailure {
                kind,
                message: "scripted failure".to_owned(),
            })],
        )
    }

    /// The URL of each read so far, in sequence.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the reads.
    #[must_use]
    pub fn reads(&self) -> Vec<String> {
        self.reads
            .lock()
            .expect("the read lock is not poisoned")
            .clone()
    }

    /// The number of waits between reads so far.
    #[must_use]
    pub fn waits(&self) -> usize {
        self.waits.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(any(test, feature = "testing"))]
impl FeedReader for ScriptedReader {
    fn read(&self, url: &str) -> Result<FeedBody, ReadFailure> {
        self.reads
            .lock()
            .expect("the read lock is not poisoned")
            .push(url.to_owned());
        let mut script = self.script.lock().expect("the script lock is not poisoned");
        let Some(results) = script.get_mut(url) else {
            return Err(ReadFailure {
                kind: FailKind::Unreachable,
                message: format!("the test reader has no body for {url}"),
            });
        };
        if results.len() > 1 {
            results
                .pop_front()
                .expect("the list has more than one result")
        } else {
            results.front().expect("the list has a result").clone()
        }
    }

    fn wait_between_reads(&self) {
        self.waits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
