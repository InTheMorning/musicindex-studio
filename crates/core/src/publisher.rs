//! Publisher feeds and their links to releases (ADR 0041).
//!
//! A link is a `podcast:remoteItem` in the draft of each of its two feeds.
//! The publisher feed lists the release in its channel. The release names
//! the publisher feed in `podcast:publisher`. The functions here change the
//! drafts through [`pipeline::edit_draft`]. They never start a draft. The
//! caller starts each draft before the call.

use std::fmt;

use crate::collection::{
    Collection, CollectionError, FeedId, FeedRow, PublisherRole, inferred_roles, rel_of,
};
use crate::feed::{Entry, Publisher, RemoteItem};
use crate::pipeline::{self, PipelineError};

/// The kind of a [`PublisherError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublisherErrorKind {
    /// The first feed is not a publisher feed.
    NotPublisher,
    /// The second feed is not a music feed.
    NotRelease,
    /// A feed has no connector, so Studio cannot write it.
    ReadOnly,
    /// The publisher feed has no recorded role, and its items give no role.
    NoRole,
    /// A draft has no `podcast:guid`.
    NoGuid,
    /// A feed has no public address.
    NoAddress,
    /// A draft or the collection failed. See [`PublisherError::pipeline`].
    Pipeline,
}

/// An error from the publisher functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherError {
    kind: PublisherErrorKind,
    message: String,
    pipeline: Option<PipelineError>,
}

impl PublisherError {
    fn new(kind: PublisherErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            pipeline: None,
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> PublisherErrorKind {
        self.kind
    }

    /// The wrapped pipeline error. It is present for the kind
    /// [`PublisherErrorKind::Pipeline`].
    #[must_use]
    pub fn pipeline(&self) -> Option<&PipelineError> {
        self.pipeline.as_ref()
    }
}

impl fmt::Display for PublisherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PublisherError {}

impl From<PipelineError> for PublisherError {
    fn from(error: PipelineError) -> Self {
        Self {
            kind: PublisherErrorKind::Pipeline,
            message: error.to_string(),
            pipeline: Some(error),
        }
    }
}

impl From<CollectionError> for PublisherError {
    fn from(error: CollectionError) -> Self {
        PipelineError::from(error).into()
    }
}

/// The roles of a publisher feed for a change of a link. Confirmed roles
/// come first. Studio then takes the roles that the draft gives, and
/// confirms them (ADR 0041 item 4, ADR 0082 items 4 and 5).
///
/// # Errors
///
/// Gives [`PublisherErrorKind::NotPublisher`] when the feed is not a
/// publisher feed, and [`PublisherErrorKind::NoRole`] when neither the
/// record nor the draft gives a role.
pub fn roles(
    collection: &Collection,
    publisher: FeedId,
) -> Result<Vec<PublisherRole>, PublisherError> {
    let row = check_publisher(collection, publisher)?;
    if row.roles_confirmed && !row.publisher_roles.is_empty() {
        return Ok(row.publisher_roles);
    }
    let (feed, _) = pipeline::draft_model(collection, publisher)?;
    let mut roles = inferred_roles(&feed);
    if roles.is_empty() {
        roles = row.publisher_roles;
    }
    if roles.is_empty() {
        return Err(PublisherError::new(
            PublisherErrorKind::NoRole,
            format!(
                "feed {} has no role, and its music items give none; select a role first",
                publisher.0
            ),
        ));
    }
    collection.set_publisher_roles(publisher, &roles, true)?;
    Ok(roles)
}

/// The music feeds that the draft of a publisher feed links to.
///
/// # Errors
///
/// Gives a [`PublisherError`] when the publisher draft is missing, or when
/// the collection fails.
pub fn linked_releases(
    collection: &Collection,
    publisher: FeedId,
) -> Result<Vec<FeedId>, PublisherError> {
    let (publisher_feed, _) = pipeline::draft_model(collection, publisher)?;
    let mut linked = Vec::new();
    for row in collection.feeds()? {
        if row.medium.as_deref() != Some("music") || row.connector_id.is_none() {
            continue;
        }
        let release = party(collection, &row)?;
        let is_linked = publisher_feed
            .remote_items
            .iter()
            .any(|item| item.medium.as_deref() == Some("music") && names(&item.value, &release));
        if is_linked {
            linked.push(row.id);
        }
    }
    Ok(linked)
}

/// Link a publisher feed and a release. Each draft gets its `remoteItem`
/// (ADR 0041 items 6 and 7). An existing link changes in place. A new link
/// goes last.
///
/// # Errors
///
/// Gives [`PublisherErrorKind::NotPublisher`] or
/// [`PublisherErrorKind::NotRelease`] for a feed of the wrong medium,
/// [`PublisherErrorKind::ReadOnly`] for a feed without a connector,
/// [`PublisherErrorKind::NoGuid`] or [`PublisherErrorKind::NoAddress`] for
/// a feed that cannot be named, and [`PublisherErrorKind::NoRole`] when the
/// publisher feed has no role. Each draft is changed only after the checks.
pub fn link(
    collection: &Collection,
    publisher: FeedId,
    release: FeedId,
) -> Result<(), PublisherError> {
    let publisher_row = check_publisher(collection, publisher)?;
    let release_row = check_release(collection, release)?;
    check_connector(&publisher_row)?;
    check_connector(&release_row)?;
    let publisher_party = party(collection, &publisher_row)?;
    let release_party = party(collection, &release_row)?;
    check_named(&publisher_party, "publisher")?;
    check_named(&release_party, "release")?;
    let rel = rel_of(&roles(collection, publisher)?);

    let channel_link = RemoteItem {
        medium: Some("music".to_owned()),
        feed_guid: release_party.guid.clone(),
        feed_url: release_party.url.clone(),
        title: release_party.title.clone(),
        rel: Some(rel.clone()),
        ..RemoteItem::default()
    };
    pipeline::edit_draft(collection, publisher, |feed| {
        upsert(&mut feed.remote_items, &release_party, channel_link);
    })?;

    let publisher_link = RemoteItem {
        medium: Some("publisher".to_owned()),
        feed_guid: publisher_party.guid.clone(),
        feed_url: publisher_party.url.clone(),
        title: publisher_party.title.clone(),
        rel: Some(rel.clone()),
        ..RemoteItem::default()
    };
    pipeline::edit_draft(collection, release, |feed| {
        let links = feed.publisher.get_or_insert_with(Publisher::default);
        upsert(&mut links.remote_items, &publisher_party, publisher_link);
    })?;
    Ok(())
}

/// Remove the link between a publisher feed and a release from both
/// drafts. A `podcast:publisher` with no item goes out of the release
/// draft. A draft with no matching item is not changed.
///
/// # Errors
///
/// Gives a [`PublisherError`] when a draft is missing or the collection
/// fails.
pub fn unlink(
    collection: &Collection,
    publisher: FeedId,
    release: FeedId,
) -> Result<(), PublisherError> {
    let publisher_party = party(collection, &collection.feed(publisher)?)?;
    let release_party = party(collection, &collection.feed(release)?)?;

    let (publisher_feed, _) = pipeline::draft_model(collection, publisher)?;
    if publisher_feed
        .remote_items
        .iter()
        .any(|item| names(&item.value, &release_party))
    {
        pipeline::edit_draft(collection, publisher, |feed| {
            feed.remote_items
                .retain(|item| !names(&item.value, &release_party));
        })?;
    }

    let (release_feed, _) = pipeline::draft_model(collection, release)?;
    let has_link = release_feed.publisher.as_ref().is_some_and(|links| {
        links
            .remote_items
            .iter()
            .any(|item| names(&item.value, &publisher_party))
    });
    if has_link {
        pipeline::edit_draft(collection, release, |feed| {
            if let Some(links) = feed.publisher.as_mut() {
                links
                    .remote_items
                    .retain(|item| !names(&item.value, &publisher_party));
                if links.remote_items.is_empty() {
                    feed.publisher = None;
                }
            }
        })?;
    }
    Ok(())
}

/// Record and confirm the roles of a publisher feed. The `rel` of each
/// music item of its draft changes, and so does the `rel` of its link in
/// each linked release (ADR 0041 item 11, ADR 0082 item 8).
///
/// # Errors
///
/// Gives [`PublisherErrorKind::NotPublisher`] when the feed is not a
/// publisher feed, [`PublisherErrorKind::NoRole`] when `roles` is empty,
/// and a [`PublisherError`] when a draft is missing.
pub fn set_roles(
    collection: &Collection,
    publisher: FeedId,
    roles: &[PublisherRole],
) -> Result<(), PublisherError> {
    let row = check_publisher(collection, publisher)?;
    if roles.is_empty() {
        return Err(PublisherError::new(
            PublisherErrorKind::NoRole,
            "select one or more roles",
        ));
    }
    let publisher_party = party(collection, &row)?;
    collection.set_publisher_roles(publisher, roles, true)?;
    let rel = rel_of(roles);

    pipeline::edit_draft(collection, publisher, |feed| {
        for item in &mut feed.remote_items {
            if item.medium.as_deref() == Some("music") {
                item.rel = Some(rel.clone());
            }
        }
    })?;
    for release in linked_releases(collection, publisher)? {
        pipeline::edit_draft(collection, release, |feed| {
            if let Some(links) = feed.publisher.as_mut() {
                for item in &mut links.remote_items {
                    if names(&item.value, &publisher_party) {
                        item.rel = Some(rel.clone());
                    }
                }
            }
        })?;
    }
    Ok(())
}

/// What a link needs from one feed.
struct Party {
    /// The `podcast:guid` of the draft. With no draft, it is the GUID of
    /// the hosted file (ADR 0041 item 12).
    guid: Option<String>,
    /// The public address of the feed.
    url: Option<String>,
    /// The title of the draft, or of the last read.
    title: Option<String>,
}

fn party(collection: &Collection, row: &FeedRow) -> Result<Party, PublisherError> {
    let (guid, title) = if collection.draft(row.id)?.is_some() {
        let (feed, _) = pipeline::draft_model(collection, row.id)?;
        (feed.podcast_guid, feed.title)
    } else {
        (row.podcast_guid.clone(), row.title.clone())
    };
    Ok(Party {
        guid,
        url: row.public_url.clone(),
        title: title.or_else(|| row.title.clone()),
    })
}

/// True when a remote item names the feed. A `feedGuid` decides. An item
/// with no `feedGuid` matches by its `feedUrl` (ADR 0041 item 12).
fn names(item: &RemoteItem, feed: &Party) -> bool {
    match item.feed_guid.as_deref() {
        Some(guid) => feed.guid.as_deref() == Some(guid),
        None => item.feed_url.is_some() && item.feed_url == feed.url,
    }
}

/// Change the first item that names the feed, and keep its position. Add
/// the item last when no item names the feed.
fn upsert(items: &mut Vec<Entry<RemoteItem>>, feed: &Party, link: RemoteItem) {
    if let Some(entry) = items.iter_mut().find(|entry| names(&entry.value, feed)) {
        entry.value.feed_guid = link.feed_guid;
        entry.value.feed_url = link.feed_url;
        entry.value.title = link.title;
        entry.value.medium = link.medium;
        entry.value.rel = link.rel;
    } else {
        items.push(Entry::new(link));
    }
}

fn check_publisher(collection: &Collection, id: FeedId) -> Result<FeedRow, PublisherError> {
    let row = collection.feed(id)?;
    if row.medium.as_deref() != Some("publisher") {
        return Err(PublisherError::new(
            PublisherErrorKind::NotPublisher,
            format!("feed {} is not a publisher feed", id.0),
        ));
    }
    Ok(row)
}

fn check_release(collection: &Collection, id: FeedId) -> Result<FeedRow, PublisherError> {
    let row = collection.feed(id)?;
    if row.medium.as_deref() != Some("music") {
        return Err(PublisherError::new(
            PublisherErrorKind::NotRelease,
            format!("feed {} is not a music feed", id.0),
        ));
    }
    Ok(row)
}

fn check_connector(row: &FeedRow) -> Result<(), PublisherError> {
    if row.connector_id.is_none() {
        return Err(PublisherError::new(
            PublisherErrorKind::ReadOnly,
            format!(
                "feed {} has no connector, so Studio cannot write it",
                row.id.0
            ),
        ));
    }
    Ok(())
}

fn check_named(feed: &Party, name: &str) -> Result<(), PublisherError> {
    if feed.guid.is_none() {
        return Err(PublisherError::new(
            PublisherErrorKind::NoGuid,
            format!("the draft of the {name} has no podcast:guid"),
        ));
    }
    if feed.url.is_none() {
        return Err(PublisherError::new(
            PublisherErrorKind::NoAddress,
            format!("the {name} has no public address"),
        ));
    }
    Ok(())
}
