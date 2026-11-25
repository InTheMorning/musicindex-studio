//! The page actions of ADR 0254 items 23 to 28 and 39.

use serde::{Deserialize, Serialize};

use super::{
    Catalog, Collection, Feed, FeedId, FeedRow, Library, PublisherError, PublisherErrorKind,
    PublisherRole, catalogs, catalogs_with_kept, check_connector, check_publisher, check_release,
    link, names, nonempty_rel, party, party_of, pipeline, rank, rel_with_other_words,
    replaced_kept_levels, roles, roles_from_rel, sort_publisher_links, unlink, write_link_roles,
};

/// A change of a page in the Catalog (ADR 0254 items 23 and 39).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageAction {
    /// Put the selected releases on a label.
    SignToLabel,
    /// Replace the label of the selected releases.
    ChangeLabel,
    /// Remove the label of the selected releases.
    GoIndependent,
    /// Replace or remove the publisher of the selected releases.
    ChangePublisher,
    /// Remove the label and publisher in one change (ADR 0254 item 39).
    LeaveAll,
}

impl PageAction {
    fn removed_roles(self) -> &'static [PublisherRole] {
        use PublisherRole::{Hosting, Label};
        match self {
            Self::SignToLabel | Self::ChangeLabel | Self::LeaveAll => &[Label, Hosting],
            Self::GoIndependent => &[Label],
            Self::ChangePublisher => &[Hosting],
        }
    }
}

/// The result for the sheet of ADR 0254 item 25.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageChangeResult {
    /// The `place` of the page after the change, in music words.
    pub place: String,
    /// The number of selected releases with changed links.
    pub changed_releases: usize,
    /// The titles of the releases that were not selected.
    pub kept_release_titles: Vec<String>,
    /// The value is `true` if no release link or kept link changed.
    pub nothing_changes: bool,
}

/// Give the start selection of ADR 0254 item 28, in feed sequence.
/// `above` is the parent of the selected tree row.
#[must_use]
pub fn page_change_selection(
    feeds: &[(FeedRow, Option<Feed>)],
    page: FeedId,
    above: Option<FeedId>,
) -> Vec<FeedId> {
    let catalogs = catalogs(feeds);
    let Some(index) = feeds.iter().position(|(row, _)| row.id == page) else {
        return Vec::new();
    };
    let releases = &catalogs[index].releases;
    let selected: Vec<_> = releases
        .iter()
        .copied()
        .filter(|release| {
            feeds
                .iter()
                .position(|(row, _)| row.id == *release)
                .is_some_and(|index| {
                    above.is_some_and(|above| catalogs[index].publishers.contains(&above))
                })
        })
        .collect();
    if selected.is_empty() {
        releases.clone()
    } else {
        selected
    }
}

/// Change the selected releases and kept links of a page (ADR 0254 items 26 and 27).
/// The caller starts each required draft, as for [`link`]. This function writes no hosted file.
/// `library` supplies the hosted models of feeds that have no draft.
///
/// # Errors
///
/// Gives `InvalidAction` for an unavailable action or selection, or a publisher error when a link fails.
/// An error puts back each database change.
pub fn change_page(
    collection: &Collection,
    library: Option<&Library>,
    page: FeedId,
    action: PageAction,
    selected_page: Option<FeedId>,
    selected_releases: &[FeedId],
) -> Result<PageChangeResult, PublisherError> {
    catalog_change(collection, || {
        change_page_inner(
            collection,
            library,
            page,
            action,
            selected_page,
            selected_releases,
        )
    })
}

/// Keep a Catalog change and its Undo record in one transaction.
pub(super) fn catalog_change<T>(
    collection: &Collection,
    change: impl FnOnce() -> Result<T, PublisherError>,
) -> Result<T, PublisherError> {
    collection.catalog_transaction(|| {
        let before = collection
            .feeds()?
            .into_iter()
            .map(|row| Ok((row.id, collection.draft(row.id)?)))
            .collect::<Result<Vec<_>, crate::collection::CollectionError>>()?;
        let kept_before = collection.kept_levels()?;
        let result = change()?;
        let kept_after = collection.kept_levels()?;
        let changed = before.iter().try_fold(false, |changed, (id, draft)| {
            let after = collection.draft(*id)?;
            Ok::<_, crate::collection::CollectionError>(
                changed
                    || match (draft, after) {
                        (Some(before), Some(after)) => before.draft_xml != after.draft_xml,
                        (None, None) => false,
                        _ => true,
                    },
            )
        })?;
        if changed || kept_before != kept_after {
            crate::change_set::record_page_move_change_set(
                collection,
                &crate::change_set::RecipientOperation::Catalog,
                &before,
                &kept_before,
                &kept_after,
            )?;
        }
        Ok(result)
    })
}

pub(super) fn models(
    collection: &Collection,
    library: Option<&Library>,
) -> Result<Vec<(FeedRow, Option<Feed>)>, PublisherError> {
    collection
        .feeds()?
        .into_iter()
        .map(|row| {
            let model = if collection.draft(row.id)?.is_some() {
                Some(pipeline::draft_model(collection, row.id)?.0)
            } else {
                pipeline::written_copy(collection, library, &row)
            };
            Ok((row, model))
        })
        .collect()
}

fn invalid(message: &str) -> PublisherError {
    PublisherError::new(PublisherErrorKind::InvalidAction, message)
}

fn change_page_inner(
    collection: &Collection,
    library: Option<&Library>,
    page: FeedId,
    action: PageAction,
    selected_page: Option<FeedId>,
    selected_releases: &[FeedId],
) -> Result<PageChangeResult, PublisherError> {
    use PublisherRole::{Hosting, Label};
    let row = check_publisher(collection, page)?;
    check_connector(&row)?;
    let page_rank = rank(&row.publisher_roles);
    if !matches!(page_rank, Some(1 | 3))
        || (page_rank != Some(1)
            && action != PageAction::ChangePublisher
            && action != PageAction::LeaveAll)
    {
        return Err(invalid("this page has no such action"));
    }
    let feeds = models(collection, library)?;
    let kept = collection.kept_levels()?;
    let catalog = catalogs_with_kept(&feeds, &kept);
    let index = feeds
        .iter()
        .position(|(row, _)| row.id == page)
        .ok_or_else(|| invalid("the page is missing"))?;
    let releases = &catalog[index].releases;
    if selected_releases
        .iter()
        .any(|release| !releases.contains(release))
    {
        return Err(invalid("select only releases of this page"));
    }
    let additions = page_additions(collection, &feeds, &catalog, page, action, selected_page)?;
    let kept_release_titles = releases
        .iter()
        .filter(|id| !selected_releases.contains(id))
        .filter_map(|id| feeds.iter().find(|(row, _)| row.id == *id))
        .map(|(row, model)| title(row, model.as_ref()))
        .collect();
    let mut changed_releases = 0;
    // Use the Catalog sequence to apply each selected release one time.
    for &release in releases.iter().filter(|id| selected_releases.contains(id)) {
        if change_release_pages(collection, &feeds, page, release, action, &additions)? {
            changed_releases += 1;
        }
    }
    for &(upper, lower) in &kept {
        if lower == page && upper != page {
            let upper_row = collection.feed(upper)?;
            let removed = match action {
                PageAction::SignToLabel | PageAction::ChangeLabel | PageAction::GoIndependent => {
                    &[Label][..]
                }
                PageAction::ChangePublisher => &[Hosting][..],
                PageAction::LeaveAll => &[Label, Hosting][..],
            };
            if upper_row
                .publisher_roles
                .iter()
                .any(|role| removed.contains(role))
            {
                collection.remove_kept_level(upper, lower)?;
            }
        }
    }
    if let Some(target) = selected_page {
        collection.add_kept_level(target, page)?;
    }
    let after_feeds = models(collection, library)?;
    let after_kept = collection.kept_levels()?;
    for (upper, lower) in replaced_kept_levels(&after_feeds, &after_kept) {
        if lower == page {
            collection.remove_kept_level(upper, lower)?;
        }
    }
    let after_kept = collection.kept_levels()?;
    let place = catalog_place(&after_feeds, &after_kept, page);
    Ok(PageChangeResult {
        place,
        changed_releases,
        kept_release_titles,
        nothing_changes: changed_releases == 0 && kept == after_kept,
    })
}

fn change_release_pages(
    collection: &Collection,
    feeds: &[(FeedRow, Option<Feed>)],
    page: FeedId,
    release: FeedId,
    action: PageAction,
    additions: &[(FeedId, PublisherRole)],
) -> Result<bool, PublisherError> {
    let release_row = check_release(collection, release)?;
    check_connector(&release_row)?;
    let before = pipeline::draft_model(collection, release)?.0;
    let mut changed = remove_page_roles(
        collection,
        feeds,
        (
            page,
            feeds
                .iter()
                .find(|(row, _)| row.id == page)
                .and_then(|(row, _)| row.publisher_roles.iter().min().copied()),
        ),
        release,
        action.removed_roles(),
        additions,
        None,
    )?;
    changed |= add_release_pages(collection, release, additions)?;
    let after = pipeline::draft_model(collection, release)?.0;
    Ok(before.publisher != after.publisher || changed)
}

pub(super) fn add_release_pages(
    collection: &Collection,
    release: FeedId,
    additions: &[(FeedId, PublisherRole)],
) -> Result<bool, PublisherError> {
    let release_row = check_release(collection, release)?;
    let mut changed = false;
    for &(target, role) in additions {
        let target_party = party(collection, &collection.feed(target)?)?;
        let current = pipeline::draft_model(collection, release)?.0;
        let has_role = current
            .publisher
            .iter()
            .flat_map(|links| &links.remote_items)
            .any(|item| {
                names(&item.value, &target_party)
                    && roles_from_rel(item.rel.as_deref().unwrap_or_default()).contains(&role)
            });
        let release_party = party(collection, &release_row)?;
        let target_feed = pipeline::draft_model(collection, target)?.0;
        let both_sides = has_role
            && target_feed.remote_items.iter().any(|item| {
                names(&item.value, &release_party)
                    && roles_from_rel(item.rel.as_deref().unwrap_or_default()).contains(&role)
            });
        if !both_sides {
            link(collection, target, release, Some(&[role]))?;
            changed = true;
        }
    }
    Ok(changed)
}

fn page_additions(
    collection: &Collection,
    feeds: &[(FeedRow, Option<Feed>)],
    catalog: &[Catalog],
    page: FeedId,
    action: PageAction,
    selected_page: Option<FeedId>,
) -> Result<Vec<(FeedId, PublisherRole)>, PublisherError> {
    use PublisherRole::{Hosting, Label};
    let target_role = match action {
        PageAction::SignToLabel | PageAction::ChangeLabel => Some(Label),
        PageAction::ChangePublisher => Some(Hosting),
        PageAction::GoIndependent | PageAction::LeaveAll => None,
    };
    if (matches!(action, PageAction::SignToLabel | PageAction::ChangeLabel)
        && selected_page.is_none())
        || (target_role.is_none() && selected_page.is_some())
        || selected_page == Some(page)
    {
        return Err(invalid("select a page for this action"));
    }
    let mut additions = Vec::new();
    if let Some(target) = selected_page {
        let target_row = check_publisher(collection, target)?;
        check_connector(&target_row)?;
        let role = target_role.ok_or_else(|| invalid("this action has no selected page"))?;
        if !roles(collection, target)?.contains(&role) {
            return Err(invalid("the selected page has the wrong role"));
        }
        additions.push((target, role));
        if role == Label
            && let Some(target) = publisher_above(feeds, catalog, target)
        {
            check_connector(&collection.feed(target)?)?;
            additions.push((target, Hosting));
        }
    }
    Ok(additions)
}

/// Give the first publisher directly above a page, from A to Z.
pub(super) fn publisher_above(
    feeds: &[(FeedRow, Option<Feed>)],
    catalog: &[Catalog],
    page: FeedId,
) -> Option<FeedId> {
    let index = feeds.iter().position(|(row, _)| row.id == page)?;
    catalog[index]
        .parents
        .iter()
        .filter_map(|id| feeds.iter().find(|(row, _)| row.id == *id))
        .filter(|(row, _)| row.publisher_roles.contains(&PublisherRole::Hosting))
        .min_by_key(|(row, model)| (title(row, model.as_ref()).to_lowercase(), row.id.0))
        .map(|(row, _)| row.id)
}

pub(super) fn title(row: &FeedRow, model: Option<&Feed>) -> String {
    model
        .and_then(|feed| feed.title.clone())
        .or_else(|| row.title.clone())
        .unwrap_or_else(|| format!("Feed {}", row.id.0))
}

pub(super) fn remove_page_roles(
    collection: &Collection,
    feeds: &[(FeedRow, Option<Feed>)],
    (page, own_link_keeps): (FeedId, Option<PublisherRole>),
    release: FeedId,
    removed: &[PublisherRole],
    additions: &[(FeedId, PublisherRole)],
    kept_item: Option<&crate::feed::RemoteItem>,
) -> Result<bool, PublisherError> {
    let release_party = party(collection, &collection.feed(release)?)?;
    let release_feed = pipeline::draft_model(collection, release)?.0;
    let mut changed = false;
    for (row, model) in feeds
        .iter()
        .filter(|(row, _)| row.medium.as_deref() == Some("publisher"))
    {
        // With `own_link_keeps`, the link of the release to the page stays
        // with that role and loses its other removed roles, so a label page
        // that is also a publisher can change its publisher (ADR 0254 item
        // 26, ADR 0253). Without it, the link to the page stays whole.
        let own_role = if row.id == page {
            match own_link_keeps {
                Some(role) => Some(role),
                None => continue,
            }
        } else {
            None
        };
        let page_party = party_of(row, model.as_ref());
        let channel = model
            .iter()
            .flat_map(|feed| &feed.remote_items)
            .find(|item| names(&item.value, &release_party));
        let reverse = release_feed
            .publisher
            .iter()
            .flat_map(|links| &links.remote_items)
            .find(|item| names(&item.value, &page_party));
        let current: Vec<_> = channel
            .into_iter()
            .chain(reverse)
            .flat_map(|item| roles_from_rel(item.rel.as_deref().unwrap_or_default()))
            .collect();
        let removed: Vec<_> = removed
            .iter()
            .copied()
            .filter(|role| !additions.contains(&(row.id, *role)) && own_role != Some(*role))
            .collect();
        if !current.iter().any(|role| removed.contains(role)) {
            continue;
        }
        check_connector(row)?;
        let remaining: Vec<_> = PublisherRole::ALL
            .into_iter()
            .filter(|role| current.contains(role) && !removed.contains(role))
            .collect();
        if remaining.is_empty() {
            unlink(collection, row.id, release)?;
        } else {
            // Make the missing side of a link that has one side only (ADR 0251).
            if channel.is_none() || reverse.is_none() {
                link(collection, row.id, release, Some(&remaining))?;
            }
            write_link_roles(collection, row.id, release, &remaining)?;
        }
        changed = true;
    }
    let outside_changes = remove_outside_roles(collection, feeds, release, removed, kept_item)?;
    Ok(changed || outside_changes)
}

/// Remove the removed roles from the links of the release to pages outside
/// the collection. The kept item and the pages of the collection stay.
fn remove_outside_roles(
    collection: &Collection,
    feeds: &[(FeedRow, Option<Feed>)],
    release: FeedId,
    removed: &[PublisherRole],
    kept_item: Option<&crate::feed::RemoteItem>,
) -> Result<bool, PublisherError> {
    let feed = pipeline::draft_model(collection, release)?.0;
    let outside_changes = feed
        .publisher
        .iter()
        .flat_map(|links| &links.remote_items)
        .any(|item| {
            !kept_item.is_some_and(|kept| super::same_name(&item.value, kept))
                && !feeds.iter().any(|(row, model)| {
                    row.medium.as_deref() == Some("publisher")
                        && names(&item.value, &party_of(row, model.as_ref()))
                })
                && roles_from_rel(item.rel.as_deref().unwrap_or_default())
                    .iter()
                    .any(|role| removed.contains(role))
        });
    if outside_changes {
        pipeline::edit_draft(collection, release, |feed| {
            if let Some(links) = feed.publisher.as_mut() {
                links.remote_items.retain_mut(|item| {
                    if kept_item.is_some_and(|kept| super::same_name(&item.value, kept))
                        || feeds.iter().any(|(row, model)| {
                            row.medium.as_deref() == Some("publisher")
                                && names(&item.value, &party_of(row, model.as_ref()))
                        })
                    {
                        return true;
                    }
                    let current = roles_from_rel(item.rel.as_deref().unwrap_or_default());
                    if !current.iter().any(|role| removed.contains(role)) {
                        return true;
                    }
                    let remaining: Vec<_> = current
                        .into_iter()
                        .filter(|role| !removed.contains(role))
                        .collect();
                    if remaining.is_empty() {
                        return false;
                    }
                    item.rel = nonempty_rel(rel_with_other_words(
                        item.rel.as_deref().unwrap_or_default(),
                        &remaining,
                    ));
                    true
                });
                sort_publisher_links(links);
                if links.remote_items.is_empty() {
                    feed.publisher = None;
                }
            }
        })?;
    }
    Ok(outside_changes)
}

/// A page that is only a publisher has no page above it, so it has no place
/// (ADR 0254 item 36). A release and each other page have a place.
#[must_use]
pub fn has_catalog_place(row: &FeedRow) -> bool {
    row.medium.as_deref() != Some("publisher")
        || !row.publisher_roles.contains(&PublisherRole::Hosting)
        || row
            .publisher_roles
            .iter()
            .any(|role| matches!(role, PublisherRole::Artist | PublisherRole::Label))
}

/// The place of a page or a release in music words, for example
/// "On Harbor Records, published by Northwind" (ADR 0254 items 25 and 36).
#[must_use]
pub fn catalog_place(
    feeds: &[(FeedRow, Option<Feed>)],
    kept: &[(FeedId, FeedId)],
    page: FeedId,
) -> String {
    let catalogs = catalogs_with_kept(feeds, kept);
    let mut above = vec![page];
    let mut labels = Vec::new();
    let mut publishers = Vec::new();
    // A release has no parents. The roles of its own links give its place
    // (ADR 0253: each link has its own roles).
    if feeds
        .iter()
        .find(|(row, _)| row.id == page)
        .is_some_and(|(row, _)| row.medium.as_deref() != Some("publisher"))
    {
        for (index, (row, model)) in feeds.iter().enumerate() {
            if !catalogs[index].releases.contains(&page) {
                continue;
            }
            let roles = super::link_roles(feeds, page, index);
            let name = title(row, model.as_ref());
            if roles.contains(&PublisherRole::Label) && !labels.contains(&name) {
                labels.push(name.clone());
            }
            if roles.contains(&PublisherRole::Hosting) && !publishers.contains(&name) {
                publishers.push(name);
            }
        }
        above.clear();
    }
    // The own links of the page can give the role hosting, so a page can
    // publish its own releases (ADR 0253).
    if let Some(page_index) = feeds.iter().position(|(row, _)| row.id == page)
        && catalogs[page_index].releases.iter().any(|release| {
            super::link_roles(feeds, *release, page_index).contains(&PublisherRole::Hosting)
        })
    {
        let (row, model) = &feeds[page_index];
        publishers.push(title(row, model.as_ref()));
    }
    let mut index = 0;
    while index < above.len() {
        if let Some(feed_index) = feeds.iter().position(|(row, _)| row.id == above[index]) {
            for parent in &catalogs[feed_index].parents {
                if !above.contains(parent) {
                    above.push(*parent);
                }
                let Some(parent_index) = feeds.iter().position(|(row, _)| row.id == *parent) else {
                    continue;
                };
                let (row, model) = &feeds[parent_index];
                // The roles of the links on the shared releases give the place
                // (ADR 0253: each link has its own roles). A kept link has no
                // shared release, so the roles of the feed give it.
                let mut link_roles: Vec<PublisherRole> = catalogs[parent_index]
                    .releases
                    .iter()
                    .filter(|release| catalogs[feed_index].releases.contains(release))
                    .flat_map(|release| super::link_roles(feeds, *release, parent_index))
                    .collect();
                if link_roles.is_empty() {
                    link_roles.clone_from(&row.publisher_roles);
                }
                let name = title(row, model.as_ref());
                if link_roles.contains(&PublisherRole::Label) && !labels.contains(&name) {
                    labels.push(name.clone());
                }
                if link_roles.contains(&PublisherRole::Hosting) && !publishers.contains(&name) {
                    publishers.push(name);
                }
            }
        }
        index += 1;
    }
    labels.sort_by_key(|title| title.to_lowercase());
    publishers.sort_by_key(|title| title.to_lowercase());
    // A label page or a publisher page is not on a label, so its place names
    // only its publishers.
    let is_label = matches!(
        feeds
            .iter()
            .find(|(row, _)| row.id == page)
            .and_then(|(row, _)| row.publisher_roles.iter().min()),
        Some(PublisherRole::Label | PublisherRole::Hosting)
    );
    if is_label {
        return if publishers.is_empty() {
            "Not published".to_owned()
        } else {
            format!("Published by {}", publishers.join(", "))
        };
    }
    let label = if labels.is_empty() {
        "Independent".to_owned()
    } else {
        format!("On {}", labels.join(", "))
    };
    let publisher = if publishers.is_empty() {
        "not published".to_owned()
    } else {
        format!("published by {}", publishers.join(", "))
    };
    format!("{label}, {publisher}")
}

/// The selection and result for a Catalog sheet.
#[derive(Debug, Serialize)]
pub struct PageChangePreview {
    /// The releases selected when the sheet opens.
    pub start_selection: Vec<FeedId>,
    /// The releases selected for this preview.
    pub selected_releases: Vec<FeedId>,
    /// The place of the page before the change, for the line "Now".
    pub now: String,
    /// The result after the selected change.
    pub result: PageChangeResult,
}

/// The values of one Catalog sheet.
#[derive(Debug)]
pub struct PageChangeRequest {
    /// The page with the links to change.
    pub page: FeedId,
    /// The music change selected by the user.
    pub action: PageAction,
    /// The new label or publisher, when selected.
    pub selected_page: Option<FeedId>,
    /// The selected releases, or the start selection when not supplied.
    pub releases: Option<Vec<FeedId>>,
    /// The parent of the selected tree row.
    pub above: Option<FeedId>,
}

/// Apply or preview one page change with drafts from the shell.
/// A preview puts back all database changes before it returns.
///
/// # Errors
///
/// Gives a publisher error when the change fails, or the error of the draft reader.
pub fn page_change_with_drafts<E>(
    collection: &Collection,
    library: Option<&Library>,
    request: &PageChangeRequest,
    preview: bool,
    mut start: impl FnMut(FeedId) -> Result<(), E>,
) -> Result<PageChangePreview, E>
where
    E: From<PublisherError> + From<crate::collection::CollectionError>,
{
    let tx = collection
        .db()
        .unchecked_transaction()
        .map_err(crate::collection::CollectionError::from)?;
    let feeds = models(collection, library).map_err(E::from)?;
    let start_selection = page_change_selection(&feeds, request.page, request.above);
    let selected = request.releases.as_ref().unwrap_or(&start_selection);
    let kept = collection.kept_levels()?;
    let now = catalog_place(&feeds, &kept, request.page);
    // A sheet opens before the user selects a label. The preview then gives
    // the start selection and the place now, and it changes nothing
    // (ADR 0254 item 28).
    if preview
        && request.selected_page.is_none()
        && matches!(
            request.action,
            PageAction::SignToLabel | PageAction::ChangeLabel
        )
    {
        let result = PageChangeResult {
            place: now.clone(),
            changed_releases: 0,
            kept_release_titles: Vec::new(),
            nothing_changes: true,
        };
        let selected_releases = selected.clone();
        tx.rollback()
            .map_err(crate::collection::CollectionError::from)?;
        return Ok(PageChangePreview {
            start_selection,
            selected_releases,
            now,
            result,
        });
    }
    let before = draft_snapshot(collection)?;
    start_page_drafts(
        collection,
        &feeds,
        request.page,
        request.selected_page,
        selected,
        &mut start,
    )?;
    let result = change_page_inner(
        collection,
        library,
        request.page,
        request.action,
        request.selected_page,
        selected,
    )
    .map_err(E::from)?;
    let selected_releases = selected.clone();
    if preview || result.nothing_changes {
        tx.rollback()
            .map_err(crate::collection::CollectionError::from)?;
        return Ok(PageChangePreview {
            start_selection,
            selected_releases,
            now,
            result,
        });
    }
    // Remove drafts that the procedure did not change.
    record_catalog(collection, &before, &kept).map_err(E::from)?;
    tx.commit()
        .map_err(crate::collection::CollectionError::from)?;
    Ok(PageChangePreview {
        start_selection,
        selected_releases,
        now,
        result,
    })
}

type DraftSnapshot = Vec<(FeedId, Option<crate::collection::DraftRow>)>;

fn draft_snapshot(
    collection: &Collection,
) -> Result<DraftSnapshot, crate::collection::CollectionError> {
    collection
        .feeds()?
        .into_iter()
        .map(|row| Ok((row.id, collection.draft(row.id)?)))
        .collect()
}

fn record_catalog(
    collection: &Collection,
    before: &DraftSnapshot,
    kept: &[(FeedId, FeedId)],
) -> Result<crate::change_set::ChangeSet, PublisherError> {
    Ok(crate::change_set::record_page_move_change_set(
        collection,
        &crate::change_set::RecipientOperation::Catalog,
        before,
        kept,
        &collection.kept_levels()?,
    )?)
}

fn start_page_drafts<E>(
    collection: &Collection,
    feeds: &[(FeedRow, Option<Feed>)],
    page: FeedId,
    target: Option<FeedId>,
    releases: &[FeedId],
    start: &mut impl FnMut(FeedId) -> Result<(), E>,
) -> Result<Vec<FeedId>, E>
where
    E: From<PublisherError> + From<crate::collection::CollectionError>,
{
    let catalog = catalogs_with_kept(feeds, &collection.kept_levels()?);
    let mut ids = vec![page];
    ids.extend_from_slice(releases);
    for (index, (row, _)) in feeds.iter().enumerate() {
        if releases.contains(&row.id) {
            ids.extend_from_slice(&catalog[index].publishers);
        }
        if Some(row.id) == target {
            ids.push(row.id);
            ids.extend_from_slice(&catalog[index].parents);
        }
    }
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    for &id in &ids {
        // The change checks a read-only feed only when its links must change.
        if collection.feed(id)?.connector_id.is_some() && collection.draft(id)?.is_none() {
            start(id)?;
        }
    }
    Ok(ids)
}

/// Sign or publish the selected pages in one Catalog step.
/// The shell supplies each missing draft.
///
/// # Errors
///
/// Gives a publisher error for an invalid selection, or the error of the draft reader.
pub fn gather_pages_with_drafts<E>(
    collection: &Collection,
    library: Option<&Library>,
    parent: FeedId,
    role: PublisherRole,
    pages: &[FeedId],
    mut start: impl FnMut(FeedId) -> Result<(), E>,
) -> Result<crate::change_set::ChangeSet, E>
where
    E: From<PublisherError> + From<crate::collection::CollectionError>,
{
    collection.catalog_transaction(|| {
        let parent_row = check_publisher(collection, parent).map_err(E::from)?;
        let action = match rank(&parent_row.publisher_roles) {
            Some(3) if role == PublisherRole::Artist => PageAction::SignToLabel,
            Some(4) if matches!(role, PublisherRole::Artist | PublisherRole::Label) => {
                PageAction::ChangePublisher
            }
            _ => {
                return Err(E::from(invalid(
                    "select a label or publisher with the required role",
                )));
            }
        };
        if pages.is_empty() {
            return Err(E::from(invalid("select at least one page")));
        }
        let before = draft_snapshot(collection)?;
        let kept = collection.kept_levels()?;
        let targets = crate::feed_check::link_targets(collection)
            .map_err(PublisherError::from)
            .map_err(E::from)?;
        let mut seen = std::collections::HashSet::new();
        for &page in pages.iter().filter(|page| seen.insert(**page)) {
            let row = check_publisher(collection, page).map_err(E::from)?;
            check_connector(&row).map_err(E::from)?;
            if !row.publisher_roles.is_empty() && !row.publisher_roles.contains(&role) {
                return Err(E::from(invalid("a selected page has a different role")));
            }
            if collection.draft(page)?.is_none() {
                start(page)?;
            }
            pipeline::correct_draft_old_links(collection, page, &targets)
                .map_err(PublisherError::from)
                .map_err(E::from)?;
            let feeds = models(collection, library).map_err(E::from)?;
            let catalog = catalogs_with_kept(&feeds, &collection.kept_levels()?);
            let mut pending = vec![page];
            let mut ancestors = std::collections::HashSet::new();
            while let Some(id) = pending.pop() {
                if id == parent {
                    return Err(E::from(invalid(
                        "a selected page is already below the target page",
                    )));
                }
                if ancestors.insert(id)
                    && let Some(index) = feeds.iter().position(|(row, _)| row.id == id)
                {
                    pending.extend_from_slice(&catalog[index].parents);
                }
            }
            let releases = page_change_selection(&feeds, page, None);
            let touched = start_page_drafts(
                collection,
                &feeds,
                page,
                Some(parent),
                &releases,
                &mut start,
            )?;
            for feed in touched {
                if collection.draft(feed)?.is_some() {
                    pipeline::correct_draft_old_links(collection, feed, &targets)
                        .map_err(PublisherError::from)
                        .map_err(E::from)?;
                }
            }
            if row.publisher_roles.is_empty() {
                super::set_roles(collection, page, &[role]).map_err(E::from)?;
            }
            change_page_inner(collection, library, page, action, Some(parent), &releases)
                .map_err(E::from)?;
        }
        record_catalog(collection, &before, &kept).map_err(E::from)
    })
}
