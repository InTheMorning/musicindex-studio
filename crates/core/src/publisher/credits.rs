//! The release credits of ADR 0257.
//!
//! Each change uses the Catalog transaction and its Undo record.
//! The caller starts the required drafts. These functions write no hosted file.

use serde::Serialize;

use super::page_change::{
    add_release_pages, catalog_change, models, publisher_above, remove_page_roles, title,
};
use super::{
    Collection, Feed, FeedId, FeedRow, PublisherError, PublisherErrorKind, PublisherRole,
    RemoteItem, catalogs_with_kept, check_connector, check_release, names, party_of, pipeline,
    roles, roles_from_rel, same_name,
};

type Models = [(FeedRow, Option<Feed>)];

/// A credit page from the collection or an external page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleasePage {
    /// The collection ID, or none for an external page.
    pub id: Option<FeedId>,
    /// The current page title, or the title or address of its link.
    pub title: String,
    /// The page identity and link data.
    pub link: RemoteItem,
}

/// The pages, publisher that follows, and alias of a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseCredits {
    /// The first artist page in file sequence.
    pub artist: Option<ReleasePage>,
    /// The first label page in file sequence.
    pub label: Option<ReleasePage>,
    /// The first publisher page in file sequence.
    pub publisher: Option<ReleasePage>,
    /// The publisher that follows, or none for "Self-published".
    pub follows: Option<ReleasePage>,
    /// The value is `true` when the publisher is the value that follows.
    pub publisher_follows: bool,
    /// The other artist pages in file sequence.
    pub other_artists: Vec<ReleasePage>,
    /// The other label pages in file sequence.
    pub other_labels: Vec<ReleasePage>,
    /// The other publisher pages in file sequence.
    pub other_publishers: Vec<ReleasePage>,
    /// The artist text when it differs from the artist page title.
    pub alias: Option<String>,
}

fn invalid(message: &str) -> PublisherError {
    PublisherError::new(PublisherErrorKind::InvalidAction, message)
}

fn check_role(role: PublisherRole) -> Result<(), PublisherError> {
    if matches!(
        role,
        PublisherRole::Artist | PublisherRole::Label | PublisherRole::Hosting
    ) {
        Ok(())
    } else {
        Err(invalid("select the artist, label or publisher role"))
    }
}

fn credit_page(feeds: &Models, link: &RemoteItem) -> ReleasePage {
    let page = feeds.iter().find(|(row, model)| {
        row.medium.as_deref() == Some("publisher") && names(link, &party_of(row, model.as_ref()))
    });
    ReleasePage {
        id: page.map(|(row, _)| row.id),
        title: page.map_or_else(
            || {
                link.title
                    .clone()
                    .or_else(|| link.feed_url.clone())
                    .or_else(|| link.feed_guid.clone())
                    .unwrap_or_else(|| "outside Studio".into())
            },
            |(row, model)| title(row, model.as_ref()),
        ),
        link: link.clone(),
    }
}

fn pages(feeds: &Models, feed: &Feed, role: PublisherRole) -> Vec<ReleasePage> {
    feed.publisher
        .iter()
        .flat_map(|links| &links.remote_items)
        .filter(|item| roles_from_rel(item.rel.as_deref().unwrap_or_default()).contains(&role))
        .map(|item| credit_page(feeds, &item.value))
        .collect()
}

fn same_page(a: Option<&ReleasePage>, b: Option<&ReleasePage>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.id.zip(b.id).is_some_and(|(a, b)| a == b) || same_name(&a.link, &b.link)
        }
        _ => false,
    }
}

fn follows(
    collection: &Collection,
    feeds: &Models,
    release: FeedId,
    artist: Option<&ReleasePage>,
    label: Option<&ReleasePage>,
) -> Result<Option<ReleasePage>, PublisherError> {
    let catalog = catalogs_with_kept(feeds, &collection.kept_levels()?);
    let above = |page: &ReleasePage| {
        let id = publisher_above(feeds, &catalog, page.id?)?;
        let (row, model) = feeds.iter().find(|(row, _)| row.id == id)?;
        let party = party_of(row, model.as_ref());
        Some(ReleasePage {
            id: Some(id),
            title: title(row, model.as_ref()),
            link: RemoteItem {
                medium: Some("publisher".into()),
                feed_guid: party.guid,
                feed_url: party.url,
                title: Some(title(row, model.as_ref())),
                rel: Some("hosting".into()),
                ..RemoteItem::default()
            },
        })
    };
    if let Some(label) = label {
        return Ok(above(label));
    }
    let Some(artist) = artist else {
        return Ok(None);
    };
    // Use the collection time, as for new_release_pages (ADR 0257 item 16a).
    let newest = feeds
        .iter()
        .filter_map(|(row, model)| {
            let model = model.as_ref()?;
            if row.id == release
                || !super::is_release(row.medium.as_deref())
                || !pages(feeds, model, PublisherRole::Label).is_empty()
                || !pages(feeds, model, PublisherRole::Artist)
                    .iter()
                    .any(|page| same_page(Some(page), Some(artist)))
            {
                return None;
            }
            Some((row, model))
        })
        .max_by_key(|(row, _)| (row.added_at, row.id.0));
    let inherited = newest.and_then(|(_, model)| {
        // The field lists only writable pages of the collection (ADR 0257 item 5).
        let mut publishers: Vec<_> = pages(feeds, model, PublisherRole::Hosting)
            .into_iter()
            .filter(|page| {
                page.id.is_some_and(|id| {
                    feeds
                        .iter()
                        .any(|(row, _)| row.id == id && row.connector_id.is_some())
                })
            })
            .collect();
        publishers.sort_by_key(|page| (page.title.to_lowercase(), page.id.map(|id| id.0)));
        publishers.into_iter().next()
    });
    Ok(inherited.or_else(|| above(artist)))
}

fn credits(
    collection: &Collection,
    feeds: &Models,
    release: FeedId,
) -> Result<ReleaseCredits, PublisherError> {
    let feed = feeds
        .iter()
        .find(|(row, _)| row.id == release)
        .and_then(|(_, model)| model.as_ref())
        .ok_or_else(|| invalid("the release model is missing"))?;
    let mut artists = pages(feeds, feed, PublisherRole::Artist).into_iter();
    let mut labels = pages(feeds, feed, PublisherRole::Label).into_iter();
    let mut publishers = pages(feeds, feed, PublisherRole::Hosting).into_iter();
    let artist = artists.next();
    let label = labels.next();
    let publisher = publishers.next();
    let follows = follows(collection, feeds, release, artist.as_ref(), label.as_ref())?;
    let alias = feed
        .author
        .clone()
        .filter(|text| !text.is_empty() && artist.as_ref().is_none_or(|page| page.title != *text));
    Ok(ReleaseCredits {
        publisher_follows: same_page(publisher.as_ref(), follows.as_ref()),
        artist,
        label,
        publisher,
        follows,
        alias,
        other_artists: artists.collect(),
        other_labels: labels.collect(),
        other_publishers: publishers.collect(),
    })
}

/// Read the release credits without changing a draft.
///
/// # Errors
///
/// Gives a publisher error when the release or collection cannot be read.
pub fn release_credits(
    collection: &Collection,
    release: FeedId,
) -> Result<ReleaseCredits, PublisherError> {
    check_release(collection, release)?;
    credits(collection, &models(collection, None)?, release)
}

/// Give the drafts for a credit change, with the publisher that follows.
/// `selection` is not supplied for an alias change.
///
/// # Errors
///
/// Gives a publisher error when the release or selected page cannot be read.
pub fn release_credit_drafts(
    collection: &Collection,
    release: FeedId,
    selection: Option<(PublisherRole, Option<FeedId>)>,
) -> Result<Vec<FeedId>, PublisherError> {
    check_release(collection, release)?;
    if selection.is_none() {
        return Ok(vec![release]);
    }
    let feeds = models(collection, None)?;
    let before = credits(collection, &feeds, release)?;
    let catalog = catalogs_with_kept(&feeds, &collection.kept_levels()?);
    let index = feeds
        .iter()
        .position(|(row, _)| row.id == release)
        .ok_or_else(|| invalid("the release is missing"))?;
    let mut ids = vec![release];
    ids.extend_from_slice(&catalog[index].publishers);
    if let Some((role, page)) = selection {
        check_role(role)?;
        let selected = page
            .map(|id| {
                let row = super::check_publisher(collection, id)?;
                let model = feeds
                    .iter()
                    .find(|(row, _)| row.id == id)
                    .and_then(|(_, model)| model.as_ref());
                let party = party_of(&row, model);
                Ok::<_, PublisherError>(credit_page(
                    &feeds,
                    &RemoteItem {
                        feed_guid: party.guid,
                        feed_url: party.url,
                        ..RemoteItem::default()
                    },
                ))
            })
            .transpose()?;
        ids.extend(page);
        if role != PublisherRole::Hosting && before.publisher_follows {
            let artist = if role == PublisherRole::Artist {
                selected.as_ref()
            } else {
                before.artist.as_ref()
            };
            let label = if role == PublisherRole::Label {
                selected.as_ref()
            } else {
                before.label.as_ref()
            };
            ids.extend(
                follows(collection, &feeds, release, artist, label)?.and_then(|page| page.id),
            );
        }
    }
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    Ok(ids)
}

fn replace_page(
    collection: &Collection,
    release: FeedId,
    role: PublisherRole,
    page: Option<FeedId>,
) -> Result<(), PublisherError> {
    if let Some(page) = page {
        let row = super::check_publisher(collection, page)?;
        check_connector(&row)?;
        if !roles(collection, page)?.contains(&role) {
            return Err(invalid("the selected page has the wrong role"));
        }
    }
    let feeds = models(collection, None)?;
    remove_page_roles(
        collection,
        &feeds,
        (release, None),
        release,
        &[role],
        &[],
        None,
    )?;
    if let Some(page) = page {
        add_release_pages(collection, release, &[(page, role)])?;
    }
    Ok(())
}

/// Set one credit field and record one Catalog step.
/// The artist page is required. The caller starts each required draft.
///
/// # Errors
///
/// Gives a publisher error when the selection is invalid or a draft change fails.
/// An error puts back each database change.
pub fn set_release_page(
    collection: &Collection,
    release: FeedId,
    role: PublisherRole,
    page: Option<FeedId>,
) -> Result<(), PublisherError> {
    check_role(role)?;
    if role == PublisherRole::Artist && page.is_none() {
        return Err(invalid("select an artist page"));
    }
    catalog_change(collection, || {
        check_connector(&check_release(collection, release)?)?;
        let before_models = models(collection, None)?;
        let before = credits(collection, &before_models, release)?;
        replace_page(collection, release, role, page)?;
        if role == PublisherRole::Artist && before.alias.is_none() {
            write_artist_title(collection, release)?;
        }
        if role != PublisherRole::Hosting && before.publisher_follows {
            let after = release_credits(collection, release)?;
            // Use the tree before the change.
            // The publisher before the change must not become a parent of the selected label.
            let following = follows(
                collection,
                &before_models,
                release,
                after.artist.as_ref(),
                after.label.as_ref(),
            )?;
            if !same_page(before.publisher.as_ref(), following.as_ref()) {
                let target = following
                    .as_ref()
                    .map(|page| {
                        page.id.ok_or_else(|| {
                            invalid("the publisher that follows is outside this collection")
                        })
                    })
                    .transpose()?;
                replace_page(collection, release, PublisherRole::Hosting, target)?;
            }
        }
        Ok(())
    })
}

fn write_artist_title(collection: &Collection, release: FeedId) -> Result<(), PublisherError> {
    let page = release_credits(collection, release)?
        .artist
        .ok_or_else(|| invalid("select an artist page"))?;
    pipeline::edit_draft(collection, release, |feed| feed.author = Some(page.title))?;
    Ok(())
}

/// Replace the release alias with the artist page title in one Catalog step.
///
/// # Errors
///
/// Gives a publisher error when the artist page is missing or a draft change fails.
pub fn clear_release_alias(collection: &Collection, release: FeedId) -> Result<(), PublisherError> {
    catalog_change(collection, || {
        check_connector(&check_release(collection, release)?)?;
        write_artist_title(collection, release)
    })
}

/// Keep the first page of a role and remove that role from the other links.
///
/// # Errors
///
/// Gives a publisher error when the role is invalid or a draft change fails.
pub fn remove_other_pages(
    collection: &Collection,
    release: FeedId,
    role: PublisherRole,
) -> Result<(), PublisherError> {
    check_role(role)?;
    catalog_change(collection, || {
        check_connector(&check_release(collection, release)?)?;
        let feeds = models(collection, None)?;
        let feed = pipeline::draft_model(collection, release)?.0;
        let pages = pages(&feeds, &feed, role);
        if let Some(first) = pages.first().filter(|_| pages.len() > 1) {
            remove_page_roles(
                collection,
                &feeds,
                (first.id.unwrap_or(release), None),
                release,
                &[role],
                &[],
                Some(&first.link),
            )?;
        }
        Ok(())
    })
}
