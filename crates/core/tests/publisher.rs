//! The publisher feeds and their links, over the local folder connector
//! (ADR 0255).

use studio_core::collection::{
    Collection, ConnectorId, FeedId, FeedRow, PublisherRole, inferred_roles, rel_of, roles_from_rel,
};
use studio_core::connector::{self, Connector, LocalFolder};
use studio_core::feed::{Entry, Feed, Publisher, RemoteItem};
use studio_core::library::Library;
use studio_core::pipeline;
use studio_core::publisher::{self, LinkChange, LinkSide, PublisherErrorKind};

struct Setup {
    _dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    connector_id: ConnectorId,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector = connector::from_row(
        &collection.connector(connector_id).unwrap(),
        None,
        &studio_core::net::NetPolicy::Open,
    )
    .unwrap();
    Setup {
        _dir: dir,
        collection,
        connector,
        connector_id,
    }
}

fn new_feed(s: &mut Setup, path: &str, title: &str, medium: &str) -> FeedId {
    pipeline::new_feed(
        &mut s.collection,
        s.connector.as_ref(),
        s.connector_id,
        path,
        title,
        "Artist",
        medium,
    )
    .unwrap()
}

fn model(s: &Setup, id: FeedId) -> Feed {
    pipeline::draft_model(&s.collection, id).unwrap().0
}

fn remember_written(s: &Setup, id: FeedId) {
    let bytes = s.collection.draft(id).unwrap().unwrap().draft_xml;
    s.collection.record_feed_write(id, 1, None).unwrap();
    s.collection.record_written_xml(id, &bytes).unwrap();
    let mut draft = s.collection.draft(id).unwrap().unwrap();
    draft.base_xml.clone_from(&bytes);
    s.collection.save_draft(id, &draft).unwrap();
}

#[test]
fn changed_links_reports_added_removed_and_ignores_other_draft_changes() {
    let mut s = setup();
    let (page, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Artist], true)
        .unwrap();
    remember_written(&s, release);
    publisher::link(&s.collection, page, release, None).unwrap();

    let from_page = publisher::changed_links(&s.collection, None, page).unwrap();
    assert_eq!(
        from_page.len(),
        1,
        "ADR 0251 item 2: changed_links must find an added release link"
    );
    assert_eq!(
        from_page[0].feed, release,
        "ADR 0251 item 2: changed_links must return the release feed"
    );
    assert_eq!(
        from_page[0].side,
        LinkSide::Release,
        "ADR 0251 item 2: changed_links must mark the release side"
    );
    assert_eq!(
        from_page[0].change,
        LinkChange::Added,
        "ADR 0251 item 3: changed_links must report an added link"
    );

    let from_release = publisher::changed_links(&s.collection, None, release).unwrap();
    assert_eq!(
        from_release.len(),
        1,
        "ADR 0251 item 2: changed_links must find the linked page"
    );
    assert_eq!(
        from_release[0].feed, page,
        "ADR 0251 item 2: changed_links must return the page feed"
    );
    assert_eq!(
        from_release[0].side,
        LinkSide::Page,
        "ADR 0251 item 2: changed_links must mark the page side"
    );
    assert_eq!(
        from_release[0].change,
        LinkChange::Added,
        "ADR 0251 item 3: changed_links must report an added link"
    );

    remember_written(&s, page);
    remember_written(&s, release);
    publisher::unlink(&s.collection, page, release).unwrap();
    assert_eq!(
        publisher::changed_links(&s.collection, None, release).unwrap()[0].change,
        LinkChange::Removed,
        "ADR 0251 item 3: changed_links must report an unlinked page"
    );

    publisher::link(&s.collection, page, release, None).unwrap();
    remember_written(&s, page);
    remember_written(&s, release);
    pipeline::edit_draft(&s.collection, page, |feed| {
        feed.title = Some("Renamed".into());
    })
    .unwrap();
    assert!(
        publisher::changed_links(&s.collection, None, release)
            .unwrap()
            .is_empty(),
        "ADR 0251 item 3: changed_links must ignore an unrelated draft change"
    );
}

#[test]
fn changed_links_reports_a_link_to_a_release_that_was_never_written() {
    let mut s = setup();
    let (page, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, page, release, None).unwrap();
    let changed = publisher::changed_links(&s.collection, None, page).unwrap();
    assert_eq!(
        changed.len(),
        1,
        "ADR 0251 item 2: changed_links must include a release with no written copy"
    );
    assert_eq!(
        changed[0].change,
        LinkChange::Added,
        "ADR 0251 item 3: a release with no written copy must have no written link"
    );
}

/// A publisher feed and a release, both with a draft.
fn pair(s: &mut Setup) -> (FeedId, FeedId) {
    let publisher = new_feed(s, "label.xml", "Label", "publisher");
    let release = new_feed(s, "album.xml", "Album", "music");
    (publisher, release)
}

fn remote(medium: &str, guid: &str, url: &str, title: &str, rel: Option<&str>) -> RemoteItem {
    RemoteItem {
        medium: Some(medium.to_owned()),
        feed_guid: Some(guid.to_owned()),
        feed_url: Some(url.to_owned()),
        title: Some(title.to_owned()),
        rel: rel.map(str::to_owned),
        ..RemoteItem::default()
    }
}

#[test]
fn release_levels_give_artist_and_parent_label_and_renames_move_no_folder() {
    let mut s = setup();
    let label = new_feed(&mut s, "label.xml", "Label One", "publisher");
    let artist = new_feed(&mut s, "artist.xml", "Artist One", "publisher");
    let second_label = new_feed(&mut s, "label-2.xml", "Label Two", "publisher");
    let second_artist = new_feed(&mut s, "artist-2.xml", "Artist Two", "publisher");
    let release = new_feed(&mut s, "album.xml", "Album", "music");
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection
        .set_publisher_roles(second_label, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .set_publisher_roles(second_artist, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, label, release, None).unwrap();
    publisher::link(&s.collection, artist, release, None).unwrap();
    publisher::link(&s.collection, second_label, release, Some(&[])).unwrap();
    publisher::link(&s.collection, second_artist, release, Some(&[])).unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        let links = &mut feed.publisher.as_mut().unwrap().remote_items;
        links
            .iter_mut()
            .find(|item| item.value.title.as_deref() == Some("Label Two"))
            .unwrap()
            .value
            .rel = Some("label".to_owned());
        links
            .iter_mut()
            .find(|item| item.value.title.as_deref() == Some("Artist Two"))
            .unwrap()
            .value
            .rel = Some("artist".to_owned());
    })
    .unwrap();
    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let model = pipeline::draft_model(&s.collection, row.id)
                .ok()
                .map(|(feed, _)| feed);
            (row, model)
        })
        .collect();
    assert_eq!(
        publisher::release_levels(&feeds, release),
        Some((Some("Artist One".into()), Some("Label One".into())))
    );

    s.collection
        .set_feed_folder(release, "Label One/Artist One/Album")
        .unwrap();
    pipeline::edit_draft(&s.collection, artist, |feed| {
        feed.title = Some("Renamed Artist".into());
    })
    .unwrap();
    assert_eq!(
        s.collection.feed(release).unwrap().folder.as_deref(),
        Some("Label One/Artist One/Album")
    );
}

#[test]
fn a_new_publisher_feed_is_a_publisher_row_with_no_role() {
    let mut s = setup();
    let publisher = new_feed(&mut s, "label.xml", "Label", "publisher");
    assert_eq!(model(&s, publisher).medium.as_deref(), Some("publisher"));
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.medium.as_deref(), Some("publisher"));
    assert_eq!(row.publisher_roles, Vec::new());
}

#[test]
fn link_writes_one_item_in_each_draft_with_the_same_rel() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();

    let release_guid = model(&s, release).podcast_guid.unwrap();
    let release_url = s.collection.feed(release).unwrap().public_url.unwrap();
    let publisher_model = model(&s, publisher);
    assert_eq!(publisher_model.remote_items.len(), 1);
    let item = &publisher_model.remote_items[0];
    assert_eq!(item.medium.as_deref(), Some("music"));
    assert_eq!(item.feed_guid.as_deref(), Some(release_guid.as_str()));
    assert_eq!(item.feed_url.as_deref(), Some(release_url.as_str()));
    assert_eq!(item.title.as_deref(), Some("Album"));
    assert_eq!(item.rel.as_deref(), Some("label"));

    let publisher_guid = model(&s, publisher).podcast_guid.unwrap();
    let publisher_url = s.collection.feed(publisher).unwrap().public_url.unwrap();
    let release_model = model(&s, release);
    let links = &release_model.publisher.as_ref().unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].medium.as_deref(), Some("publisher"));
    assert_eq!(links[0].feed_guid.as_deref(), Some(publisher_guid.as_str()));
    assert_eq!(links[0].feed_url.as_deref(), Some(publisher_url.as_str()));
    assert_eq!(links[0].title.as_deref(), Some("Label"));
    assert_eq!(links[0].rel.as_deref(), Some("label"));

    // The role is recorded by the first link.
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_roles,
        [PublisherRole::Label, PublisherRole::Producer]
    );
    assert_eq!(
        publisher::linked_releases(&s.collection, publisher).unwrap(),
        vec![release]
    );
}

#[test]
fn a_second_link_of_the_same_feeds_adds_no_item() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();
    assert_eq!(model(&s, publisher).remote_items.len(), 1);
    assert_eq!(model(&s, release).publisher.unwrap().remote_items.len(), 1);
}

#[test]
fn a_link_can_select_roles_and_a_second_link_merges_them() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Hosting, PublisherRole::Label],
            true,
        )
        .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Hosting]),
    )
    .unwrap();
    assert_eq!(
        model(&s, publisher).remote_items[0].value.rel.as_deref(),
        Some("hosting")
    );
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .value
            .rel
            .as_deref(),
        Some("hosting")
    );
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();
    assert_eq!(
        model(&s, publisher).remote_items[0].value.rel.as_deref(),
        Some("label hosting")
    );
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .value
            .rel
            .as_deref(),
        Some("label hosting")
    );
    assert_eq!(model(&s, publisher).remote_items.len(), 1);
}

#[test]
fn link_rejects_a_role_outside_the_publisher_role_set() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
        .unwrap();
    let error = publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Artist]),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NoRole);
}

#[test]
fn links_without_level_roles_have_no_rel_attribute() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Producer], true)
        .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();

    assert_eq!(model(&s, publisher).remote_items[0].value.rel, None);
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .value
            .rel,
        None
    );

    let other_release = new_feed(&mut s, "other.xml", "Other", "music");
    publisher::list_on_page(&s.collection, publisher, other_release, None).unwrap();
    assert_eq!(model(&s, publisher).remote_items[1].value.rel, None);
}

#[test]
fn set_roles_keeps_a_release_role_on_another_page() {
    let mut s = setup();
    let (label_a, release) = pair(&mut s);
    let label_b = new_feed(&mut s, "label-b.xml", "Label B", "publisher");
    s.collection
        .set_publisher_roles(label_a, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .set_publisher_roles(label_b, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, label_a, release, None).unwrap();
    publisher::link(&s.collection, label_b, release, None).unwrap();

    let kept = publisher::set_roles(
        &s.collection,
        label_b,
        &[PublisherRole::Label, PublisherRole::Artist],
    )
    .unwrap();

    assert_eq!(kept, [(PublisherRole::Label, release)]);
    assert_eq!(
        model(&s, label_b).remote_items[0].value.rel.as_deref(),
        Some("artist")
    );
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[1]
            .value
            .rel
            .as_deref(),
        Some("artist")
    );
}

#[test]
fn set_roles_removes_the_last_link_role_without_an_empty_rel() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();

    publisher::set_roles(&s.collection, publisher, &[PublisherRole::Producer]).unwrap();

    assert_eq!(model(&s, publisher).remote_items[0].value.rel, None);
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .value
            .rel,
        None
    );
}

#[test]
fn set_link_roles_requires_the_link_on_both_drafts() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();
    pipeline::edit_draft(&s.collection, publisher, |feed| feed.remote_items.clear()).unwrap();
    let error = publisher::set_link_roles(
        &s.collection,
        publisher,
        release,
        &[PublisherRole::Producer],
    )
    .unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NotLinked);

    publisher::link(&s.collection, publisher, release, None).unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| feed.publisher = None).unwrap();
    let error = publisher::set_link_roles(
        &s.collection,
        publisher,
        release,
        &[PublisherRole::Producer],
    )
    .unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NotLinked);
}

#[test]
fn set_link_roles_changes_only_the_selected_link() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    let other = new_feed(&mut s, "other.xml", "Other", "music");
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        other,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();
    publisher::set_link_roles(
        &s.collection,
        publisher,
        release,
        &[PublisherRole::Producer],
    )
    .unwrap();
    assert_eq!(
        model(&s, publisher).remote_items[0].value.rel.as_deref(),
        Some("producer")
    );
    assert_eq!(
        model(&s, publisher).remote_items[1].value.rel.as_deref(),
        Some("label")
    );
}

#[test]
fn changed_links_reports_a_role_change_on_both_sides() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();
    remember_written(&s, publisher);
    remember_written(&s, release);

    publisher::set_link_roles(
        &s.collection,
        publisher,
        release,
        &[PublisherRole::Producer],
    )
    .unwrap();

    assert_eq!(
        publisher::changed_links(&s.collection, None, publisher)
            .unwrap()
            .iter()
            .map(|link| link.feed)
            .collect::<Vec<_>>(),
        [release]
    );
    assert_eq!(
        publisher::changed_links(&s.collection, None, release)
            .unwrap()
            .iter()
            .map(|link| link.feed)
            .collect::<Vec<_>>(),
        [publisher]
    );
}

#[test]
fn a_new_link_goes_last_after_the_links_of_the_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Hosting], true)
        .unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                "other-guid",
                "https://music.example/other.xml",
                "Other",
                Some("artist"),
            ))],
        });
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();

    let links = model(&s, release).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].title.as_deref(), Some("Label"));
    assert_eq!(links[1].feed_guid.as_deref(), Some("other-guid"));
}

#[test]
fn a_second_curator_page_is_allowed() {
    let mut s = setup();
    let (first, release) = pair(&mut s);
    let second = new_feed(&mut s, "curator.xml", "Curator", "publisher");
    s.collection
        .set_publisher_roles(first, &[PublisherRole::Curator], true)
        .unwrap();
    s.collection
        .set_publisher_roles(second, &[PublisherRole::Curator], true)
        .unwrap();

    publisher::link(&s.collection, first, release, None).unwrap();
    publisher::link(&s.collection, second, release, None).unwrap();
}

#[test]
fn set_roles_sorts_a_new_hosting_role_before_the_artist_link() {
    let mut s = setup();
    let (artist, release) = pair(&mut s);
    let hosting = new_feed(&mut s, "hosting.xml", "Hosting", "publisher");
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection
        .set_publisher_roles(hosting, &[PublisherRole::Hosting], true)
        .unwrap();
    publisher::link(&s.collection, artist, release, None).unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        feed.publisher.as_mut().unwrap().remote_items.insert(
            0,
            Entry::new(remote(
                "publisher",
                "other-guid",
                "https://music.example/other.xml",
                "Other",
                Some("producer"),
            )),
        );
    })
    .unwrap();

    publisher::set_roles(
        &s.collection,
        artist,
        &[PublisherRole::Artist, PublisherRole::Hosting],
    )
    .unwrap();

    let links = model(&s, release).publisher.unwrap().remote_items;
    assert_eq!(links[0].value.rel.as_deref(), Some("artist hosting"));
    assert_eq!(links[1].value.rel.as_deref(), Some("producer"));
}

#[test]
fn a_second_artist_label_or_publisher_page_is_refused_before_draft_changes() {
    for role in [
        PublisherRole::Artist,
        PublisherRole::Label,
        PublisherRole::Hosting,
    ] {
        let mut s = setup();
        let (first, release) = pair(&mut s);
        let second = new_feed(&mut s, "second.xml", "Second Page", "publisher");
        s.collection
            .set_publisher_roles(first, &[role], true)
            .unwrap();
        s.collection
            .set_publisher_roles(second, &[role], true)
            .unwrap();
        publisher::link(&s.collection, first, release, None).unwrap();
        let first_items = model(&s, first).remote_items;
        let second_items = model(&s, second).remote_items;
        let release_items = model(&s, release).publisher.unwrap().remote_items;

        let error = publisher::link(&s.collection, second, release, None).unwrap_err();
        assert_eq!(error.kind(), PublisherErrorKind::RoleTaken);
        assert_eq!(model(&s, first).remote_items, first_items);
        assert_eq!(model(&s, second).remote_items, second_items);
        assert_eq!(
            model(&s, release).publisher.unwrap().remote_items,
            release_items
        );
    }
}

#[test]
fn linking_back_restores_the_written_entry_and_bytes() {
    let mut s = setup();
    let page = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Artist], true)
        .unwrap();
    let releases = [
        new_feed(&mut s, "a.xml", "Release A", "music"),
        new_feed(&mut s, "b.xml", "Release B", "music"),
        new_feed(&mut s, "c.xml", "Release C", "music"),
    ];
    for release in releases {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    let release_b = releases[1];
    remember_written(&s, page);
    remember_written(&s, release_b);
    let page_base = s.collection.draft(page).unwrap().unwrap().base_xml;
    let release_base = s.collection.draft(release_b).unwrap().unwrap().base_xml;

    publisher::unlink(&s.collection, page, release_b).unwrap();
    publisher::link(&s.collection, page, release_b, None).unwrap();

    let page_draft = s.collection.draft(page).unwrap().unwrap();
    let release_draft = s.collection.draft(release_b).unwrap().unwrap();
    assert_eq!(
        page_draft.draft_xml, page_base,
        "observation 4 of the 2026-11-23 walk: publisher::upsert must restore the page bytes"
    );
    assert_eq!(
        release_draft.draft_xml, release_base,
        "observation 4 of the 2026-11-23 walk: publisher::upsert must restore the release bytes"
    );
}

#[test]
fn link_back_keeps_written_position_and_element_after_another_edit() {
    let mut s = setup();
    let page = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Artist], true)
        .unwrap();
    let releases = [
        new_feed(&mut s, "a.xml", "Release A", "music"),
        new_feed(&mut s, "b.xml", "Release B", "music"),
        new_feed(&mut s, "c.xml", "Release C", "music"),
    ];
    for release in releases {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    let release_b = releases[1];
    remember_written(&s, page);
    let base = s.collection.draft(page).unwrap().unwrap().base_xml;
    let written_b = remote_item_bytes(&base, "Release B");

    publisher::unlink(&s.collection, page, release_b).unwrap();
    pipeline::edit_draft(&s.collection, page, |feed| {
        feed.title = Some("Renamed Artist".into());
    })
    .unwrap();
    publisher::link(&s.collection, page, release_b, None).unwrap();

    let draft = s.collection.draft(page).unwrap().unwrap();
    let feed = model(&s, page);
    assert_eq!(
        feed.remote_items
            .iter()
            .map(|entry| entry.value.title.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["Release A", "Release B", "Release C"],
        "observation 4 of the 2026-11-23 walk: publisher::upsert must restore the base position"
    );
    assert!(
        String::from_utf8_lossy(&draft.draft_xml).contains(written_b),
        "observation 4 of the 2026-11-23 walk: publisher::upsert must keep the written element bytes"
    );
}

#[test]
fn linking_an_existing_entry_without_a_title_keeps_it_absent() {
    let mut s = setup();
    let (page, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, page, release, None).unwrap();
    pipeline::edit_draft(&s.collection, page, |feed| {
        feed.remote_items[0].value.title = None;
    })
    .unwrap();
    remember_written(&s, page);

    publisher::link(&s.collection, page, release, None).unwrap();

    assert_eq!(
        model(&s, page).remote_items[0].value.title,
        None,
        "observation 4 of the 2026-11-23 walk: publisher::upsert must keep an absent title"
    );
}

fn remote_item_bytes<'a>(bytes: &'a [u8], title: &str) -> &'a str {
    let text = std::str::from_utf8(bytes).unwrap();
    let title_at = text.find(&format!("title=\"{title}\"")).unwrap();
    let start = text[..title_at].rfind("<podcast:remoteItem").unwrap();
    let end = text[title_at..].find("/>").unwrap() + title_at + 2;
    &text[start..end]
}

#[test]
fn unlink_removes_the_link_and_keeps_an_item_that_names_no_feed_of_the_collection() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
        .unwrap();
    // A link to a feed that is not in the collection, first in the list.
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "stranger-guid",
            "https://music.example/stranger.xml",
            "Stranger",
            Some("label"),
        )));
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();
    publisher::unlink(&s.collection, publisher, release).unwrap();

    let remaining = model(&s, publisher).remote_items;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].feed_guid.as_deref(), Some("stranger-guid"));
    assert!(model(&s, release).publisher.is_none());
}

#[test]
fn unlink_keeps_the_other_publisher_links_of_a_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
        .unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                "other-guid",
                "https://music.example/other.xml",
                "Other",
                Some("artist"),
            ))],
        });
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release, None).unwrap();
    publisher::unlink(&s.collection, publisher, release).unwrap();

    let links = model(&s, release).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].feed_guid.as_deref(), Some("other-guid"));
}

#[test]
fn set_roles_changes_the_rel_in_the_publisher_draft_and_in_each_linked_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(
            publisher,
            &[PublisherRole::Label, PublisherRole::Hosting],
            true,
        )
        .unwrap();
    publisher::link(
        &s.collection,
        publisher,
        release,
        Some(&[PublisherRole::Label]),
    )
    .unwrap();
    let kept = publisher::set_roles(
        &s.collection,
        publisher,
        &[
            PublisherRole::Label,
            PublisherRole::Hosting,
            PublisherRole::Producer,
        ],
    )
    .unwrap();
    assert_eq!(kept, Vec::new());

    assert_eq!(
        model(&s, publisher).remote_items[0].rel.as_deref(),
        Some("label producer")
    );
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .rel
            .as_deref(),
        Some("label producer")
    );
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_roles,
        [
            PublisherRole::Label,
            PublisherRole::Hosting,
            PublisherRole::Producer
        ]
    );
}

#[test]
fn roles_records_and_confirms_the_inferred_roles_of_the_draft() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "album-guid",
            "https://music.example/album.xml",
            "Album",
            Some(" LABEL "),
        )));
    })
    .unwrap();
    assert_eq!(
        publisher::roles(&s.collection, publisher).unwrap(),
        [PublisherRole::Label]
    );
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_roles,
        [PublisherRole::Label]
    );
}

#[test]
fn roles_infer_artist_from_a_link_without_medium_when_it_names_a_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    let release_row = s.collection.feed(release).unwrap();
    let release_guid = model(&s, release).podcast_guid.unwrap();
    let release_url = release_row.public_url.unwrap();
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        let mut item = remote(
            "music",
            &release_guid,
            &release_url,
            "Album",
            Some("artist"),
        );
        item.medium = None;
        feed.remote_items.push(Entry::new(item));
    })
    .unwrap();

    assert_eq!(
        publisher::roles(&s.collection, publisher).unwrap(),
        [PublisherRole::Artist],
        "ADR 0246 item 2: fix roles in crates/core/src/publisher.rs"
    );
}

#[test]
fn roles_gives_no_role_for_items_with_no_rel() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "album-guid",
            "https://music.example/album.xml",
            "Album",
            None,
        )));
    })
    .unwrap();
    let error = publisher::roles(&s.collection, publisher).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NoRole);
}

#[test]
fn link_gives_the_error_kinds_of_its_checks() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    let not_publisher = publisher::link(&s.collection, release, release, None).unwrap_err();
    assert_eq!(not_publisher.kind(), PublisherErrorKind::NotPublisher);
    let not_release = publisher::link(&s.collection, publisher, publisher, None).unwrap_err();
    assert_eq!(not_release.kind(), PublisherErrorKind::NotRelease);

    // A feed with no connector is read-only.
    let read_only = s.collection.add_feed(None, None).unwrap();
    s.collection
        .set_new_feed(read_only, "Read only", "music")
        .unwrap();
    let error = publisher::link(&s.collection, publisher, read_only, None).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::ReadOnly);

    // The publisher feed has no role, and its draft has no music item.
    let error = publisher::link(&s.collection, publisher, release, None).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NoRole);
    assert!(model(&s, release).publisher.is_none());
}

#[test]
fn inferred_roles_read_each_rel_as_a_set_of_roles() {
    let mut feed = Feed::default();
    assert_eq!(inferred_roles(&feed), Vec::new());
    // The HeyCitizen artist feed of 2026-10-29 (ADR 0082).
    feed.remote_items.push(Entry::new(remote(
        "music",
        "a",
        "https://x/a.xml",
        "A",
        Some("artist host author label producer"),
    )));
    assert_eq!(
        inferred_roles(&feed),
        [
            PublisherRole::Artist,
            PublisherRole::Label,
            PublisherRole::Producer,
            PublisherRole::Host,
            PublisherRole::Author
        ]
    );
    // A comma list, other cases, a word that is not a role, and a
    // repeated role. Each role of each music item counts one time.
    feed.remote_items.push(Entry::new(remote(
        "music",
        "b",
        "https://x/b.xml",
        "B",
        Some("Sponsor, band, HOST"),
    )));
    assert_eq!(inferred_roles(&feed).last(), Some(&PublisherRole::Sponsor));
    assert_eq!(inferred_roles(&feed).len(), 6);
}

#[test]
fn rel_of_writes_the_roles_in_their_sequence() {
    assert_eq!(
        rel_of(&[
            PublisherRole::Producer,
            PublisherRole::Artist,
            PublisherRole::Artist
        ]),
        "artist producer"
    );
    assert_eq!(
        roles_from_rel("producer,artist"),
        [PublisherRole::Artist, PublisherRole::Producer]
    );
    // Hosting comes third (musicindex ADR 0036 R-ROLE-1, ADR 0083).
    assert_eq!(
        rel_of(&[
            PublisherRole::Host,
            PublisherRole::Hosting,
            PublisherRole::Label
        ]),
        "label hosting host"
    );
    // Curator comes after hosting and before producer (musicindex ADR 0081
    // R-CURATOR-1 item 1).
    assert_eq!(
        roles_from_rel("producer curator hosting"),
        [
            PublisherRole::Hosting,
            PublisherRole::Curator,
            PublisherRole::Producer
        ]
    );
}

#[test]
fn a_read_records_inferred_roles_and_keeps_confirmed_roles() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    let mut feed = Feed {
        medium: Some("publisher".to_owned()),
        ..Feed::default()
    };
    feed.remote_items.push(Entry::new(remote(
        "music",
        "a",
        "https://x/a.xml",
        "A",
        Some("artist host author label producer"),
    )));
    let version = connector::Version::new("v1");
    s.collection
        .record_state(publisher, &feed, &version)
        .unwrap();
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.publisher_roles.first(), Some(&PublisherRole::Artist));
    assert_eq!(row.publisher_roles.len(), 5);
    assert!(!row.roles_confirmed);

    // A confirmed set does not change at the next read (ADR 0082 item 4).
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .record_state(publisher, &feed, &version)
        .unwrap();
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.publisher_roles, [PublisherRole::Label]);
    assert!(row.roles_confirmed);
}

#[test]
fn a_playlist_links_to_a_publisher_feed_as_an_album_does() {
    let mut s = setup();
    let publisher = new_feed(&mut s, "label.xml", "Label", "publisher");
    let playlist = new_feed(&mut s, "list.xml", "Mixtape", "musicL");
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, publisher, playlist, None).unwrap();

    // ADR 0086 item 2: the channel item has the medium of the playlist.
    let channel = &model(&s, publisher).remote_items;
    assert_eq!(channel.len(), 1);
    assert_eq!(channel[0].value.medium.as_deref(), Some("musicL"));
    assert_eq!(channel[0].value.rel.as_deref(), Some("artist"));
    let links = model(&s, playlist).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].value.medium.as_deref(), Some("publisher"));

    assert_eq!(
        publisher::linked_releases(&s.collection, publisher).unwrap(),
        vec![playlist]
    );
    // A role change reaches the playlist item, and the read of the roles
    // takes its rel.
    publisher::set_roles(&s.collection, publisher, &[PublisherRole::Label]).unwrap();
    assert_eq!(
        model(&s, publisher).remote_items[0].value.rel.as_deref(),
        Some("label")
    );
    assert_eq!(
        inferred_roles(&model(&s, publisher)),
        vec![PublisherRole::Label]
    );
}

#[test]
fn links_without_a_medium_use_the_named_release_medium_and_role() {
    let mut s = setup();
    let publisher = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let release = new_feed(&mut s, "album.xml", "Album", "music");
    let release_guid = model(&s, release).podcast_guid.unwrap();
    let orphan_guid = "feed-outside-collection";
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items = vec![
            Entry::new(RemoteItem {
                feed_guid: Some(release_guid),
                ..RemoteItem::default()
            }),
            Entry::new(RemoteItem {
                feed_guid: Some(orphan_guid.to_owned()),
                ..RemoteItem::default()
            }),
        ];
    })
    .unwrap();

    assert_eq!(
        publisher::linked_releases(&s.collection, publisher).unwrap(),
        [release],
        "ADR 0246 item 1: fix release_medium"
    );
    let library_dir = tempfile::tempdir().unwrap();
    let library = Library::open(library_dir.path()).unwrap();
    assert_eq!(
        publisher::publishers_linking(&s.collection, &library, release).unwrap(),
        [publisher],
        "ADR 0246 item 2: fix release_medium"
    );

    publisher::set_roles(&s.collection, publisher, &[PublisherRole::Artist]).unwrap();
    let links = model(&s, publisher).remote_items;
    assert_eq!(
        links[0].value.rel.as_deref(),
        Some("artist"),
        "ADR 0246 item 3: fix set_roles"
    );
    assert_eq!(
        links[0].value.medium.as_deref(),
        Some("music"),
        "ADR 0246 item 3: fix set_roles"
    );
    assert_eq!(
        links[1].value.rel, None,
        "ADR 0246 item 1: fix release_medium"
    );
    assert_eq!(
        links[1].value.medium, None,
        "ADR 0246 item 1: fix release_medium"
    );
}

#[test]
fn catalogs_give_the_releases_of_each_publisher_and_the_publishers_of_each_release() {
    let mut s = setup();
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let first = new_feed(&mut s, "first.xml", "First", "music");
    let shared = new_feed(&mut s, "shared.xml", "Shared", "music");
    let mixtape = new_feed(&mut s, "mixtape.xml", "Mixtape", "musicL");
    let half = new_feed(&mut s, "half.xml", "Half", "music");
    let alone = new_feed(&mut s, "alone.xml", "Alone", "music");
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    publisher::link(&s.collection, artist, first, None).unwrap();
    publisher::link(&s.collection, artist, shared, None).unwrap();
    publisher::link(&s.collection, label, shared, None).unwrap();
    publisher::link(&s.collection, label, mixtape, None).unwrap();
    // A link in the draft of the release only.
    let label_model = model(&s, label);
    pipeline::edit_draft(&s.collection, half, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                label_model.podcast_guid.as_deref().unwrap(),
                "https://music.example/label.xml",
                "Label",
                Some("label"),
            ))],
        });
    })
    .unwrap();

    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(&s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    let of = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        catalogs[index].clone()
    };
    assert_eq!(of(artist).releases, vec![first, shared]);
    // ADR 0085 item 2: the listed releases in their sequence, then the
    // release that names the label only.
    assert_eq!(of(label).releases, vec![shared, mixtape, half]);
    // ADR 0085 item 5: the primary publisher comes first.
    assert_eq!(of(shared).publishers, vec![label, artist]);
    assert_eq!(of(first).publishers, vec![artist]);
    assert_eq!(of(mixtape).publishers, vec![label]);
    assert_eq!(of(half).publishers, vec![label]);
    assert_eq!(of(alone).publishers, Vec::new());
    assert_eq!(of(alone).releases, Vec::new());
}

#[test]
fn a_role_that_the_two_sides_of_a_link_give_is_linked() {
    let mut s = setup();
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let album = new_feed(&mut s, "album.xml", "Album", "music");
    let other = new_feed(&mut s, "other.xml", "Other", "music");
    // Inferred sets: nothing in Studio confirms them.
    s.collection
        .set_publisher_roles(
            artist,
            &[PublisherRole::Label, PublisherRole::Artist],
            false,
        )
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], false)
        .unwrap();
    let name = |s: &Setup, release: FeedId, publisher: FeedId, path: &str, rel: &str| {
        let guid = model(s, publisher).podcast_guid.unwrap();
        let url = format!("https://music.example/{path}");
        pipeline::edit_draft(&s.collection, release, |feed| {
            feed.publisher = Some(Publisher {
                remote_items: vec![Entry::new(remote("publisher", &guid, &url, "P", Some(rel)))],
            });
        })
        .unwrap();
    };
    // The album gives the first role of the artist feed. The other album
    // names the label feed, but its rel does not give "label".
    let album_guid = model(&s, album).podcast_guid.unwrap();
    pipeline::edit_draft(&s.collection, artist, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            &album_guid,
            "https://music.example/album.xml",
            "Album",
            Some("artist"),
        )));
    })
    .unwrap();
    name(&s, album, artist, "artist.xml", "artist label");
    name(&s, other, label, "label.xml", "artist");
    let other_guid = model(&s, other).podcast_guid.unwrap();
    pipeline::edit_draft(&s.collection, label, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            &other_guid,
            "https://music.example/other.xml",
            "Other",
            Some("label"),
        )));
    })
    .unwrap();
    pipeline::edit_draft(&s.collection, album, |feed| {
        let guid = model(&s, label).podcast_guid.unwrap();
        feed.publisher
            .as_mut()
            .unwrap()
            .remote_items
            .push(Entry::new(remote(
                "publisher",
                &guid,
                "https://music.example/label.xml",
                "Label",
                Some("label"),
            )));
    })
    .unwrap();

    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(&s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    let linked = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        catalogs[index].roles_linked
    };
    let message = "ADR 0253 item 17: both sides of one link must give the first role";
    assert!(linked(artist), "{message}");
    assert!(!linked(label), "{message}");
    assert!(!linked(album), "{message}");
}

#[test]
fn the_levels_follow_the_rank_and_the_shared_releases() {
    let mut s = setup();
    let feed = |s: &mut Setup, name: &str, roles: &[PublisherRole]| {
        let id = new_feed(s, &format!("{name}.xml"), name, "publisher");
        s.collection.set_publisher_roles(id, roles, true).unwrap();
        id
    };
    let host = feed(
        &mut s,
        "host",
        &[PublisherRole::Artist, PublisherRole::Hosting],
    );
    let label_a = feed(&mut s, "label-a", &[PublisherRole::Label]);
    let label_b = feed(&mut s, "label-b", &[PublisherRole::Label]);
    let artist_x = feed(&mut s, "artist-x", &[PublisherRole::Artist]);
    let artist_y = feed(&mut s, "artist-y", &[PublisherRole::Artist]);
    let producer = feed(&mut s, "beat-lab", &[PublisherRole::Producer]);
    let artist_z = feed(&mut s, "artist-z", &[PublisherRole::Artist]);
    let albums: Vec<FeedId> = (1..=7)
        .map(|n| {
            new_feed(
                &mut s,
                &format!("album-{n}.xml"),
                &format!("Album {n}"),
                "music",
            )
        })
        .collect();
    let link = |s: &Setup, publisher: FeedId, numbers: &[usize]| {
        for &n in numbers {
            publisher::link(&s.collection, publisher, albums[n - 1], None).unwrap();
        }
    };
    for release in &albums {
        publisher::link(
            &s.collection,
            host,
            *release,
            Some(&[PublisherRole::Hosting]),
        )
        .unwrap();
    }
    let own = new_feed(&mut s, "own.xml", "Own", "music");
    publisher::link(&s.collection, host, own, None).unwrap();
    link(&s, label_a, &[1, 2, 3]);
    link(&s, label_b, &[4, 5, 6]);
    link(&s, artist_x, &[1, 2]);
    link(&s, artist_y, &[3, 4]);
    link(&s, producer, &[1]);
    link(&s, artist_z, &[7]);

    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(&s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    let of = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        let mut catalog = catalogs[index].clone();
        catalog.children.sort_by_key(|id| id.0);
        catalog.parents.sort_by_key(|id| id.0);
        catalog
    };
    let message = "ADR 0215 items 1 and 2: a feed is directly below a feed of a higher rank that shares a release";
    // The artists of the labels are below the labels, not again below the
    // hosting feed. Artist Z has no label, so it is directly below it.
    assert_eq!(
        of(host).children,
        vec![label_a, label_b, artist_z],
        "{message}"
    );
    assert_eq!(of(host).parents, Vec::new(), "{message}");
    assert_eq!(of(label_a).children, vec![artist_x, artist_y], "{message}");
    assert_eq!(of(label_b).children, vec![artist_y], "{message}");
    assert_eq!(of(label_a).parents, vec![host], "{message}");
    // An artist on two labels is below each label.
    assert_eq!(of(artist_y).parents, vec![label_a, label_b], "{message}");
    // Credit roles have no rank (ADR 0253 item 15).
    assert_eq!(of(producer).parents, Vec::new(), "{message}");
    assert_eq!(of(producer).children, Vec::new(), "{message}");
    assert_eq!(of(artist_z).parents, vec![host], "{message}");
    assert_eq!(of(albums[0]).children, Vec::new());
}

#[test]
fn a_kept_link_places_a_new_feed_below_its_label() {
    let mut s = setup();
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection.add_kept_level(label, artist).unwrap();
    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| (row, None))
        .collect();
    let kept = s.collection.kept_levels().unwrap();
    let catalogs = publisher::catalogs_with_kept(&feeds, &kept);
    let of = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        &catalogs[index]
    };
    assert_eq!(of(label).children, vec![artist]);
    assert_eq!(of(artist).parents, vec![label]);
    s.collection.add_kept_level(artist, label).unwrap();
    let reverse = s.collection.kept_levels().unwrap();
    let catalogs = publisher::catalogs_with_kept(&feeds, &reverse);
    let of = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        &catalogs[index]
    };
    assert!(!of(artist).children.contains(&label));
    assert!(!of(label).parents.contains(&artist));
}

#[test]
fn the_remove_of_a_feed_removes_its_kept_link_and_the_undo_gives_it_back() {
    let mut s = setup();
    let hosting = new_feed(&mut s, "hosting.xml", "Hosting", "publisher");
    s.collection
        .set_publisher_roles(hosting, &[PublisherRole::Hosting], true)
        .unwrap();
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection.add_kept_level(hosting, label).unwrap();
    s.collection.add_kept_level(label, artist).unwrap();
    s.collection.add_kept_level(hosting, artist).unwrap();
    // The artist is directly below the label only (ADR 0215 item 2).
    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| (row, None))
        .collect();
    let catalogs = publisher::catalogs_with_kept(&feeds, &s.collection.kept_levels().unwrap());
    let index = |id: FeedId| feeds.iter().position(|(row, _)| row.id == id).unwrap();
    assert_eq!(catalogs[index(hosting)].children, vec![label]);
    assert_eq!(catalogs[index(artist)].parents, vec![label]);

    studio_core::people::remove_feed(&s.collection, artist).unwrap();
    assert_eq!(s.collection.kept_levels().unwrap(), vec![(hosting, label)]);
    studio_core::people::undo_delete(&s.collection).unwrap();
    assert_eq!(
        s.collection.kept_levels().unwrap(),
        vec![(hosting, label), (hosting, artist), (label, artist)]
    );
    s.collection.delete_feed(label).unwrap();
    assert_eq!(s.collection.kept_levels().unwrap(), vec![(hosting, artist)]);
}

#[test]
fn a_feed_with_the_roles_artist_and_label_is_one_feed_at_the_artist_rank() {
    let mut s = setup();
    let own = new_feed(&mut s, "own.xml", "Own", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    s.collection
        .set_publisher_roles(own, &[PublisherRole::Artist, PublisherRole::Label], true)
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    let album = new_feed(&mut s, "album.xml", "Album", "music");
    publisher::link(&s.collection, own, album, Some(&[PublisherRole::Artist])).unwrap();
    publisher::link(&s.collection, label, album, None).unwrap();

    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(&s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    let index = |id: FeedId| feeds.iter().position(|(row, _)| row.id == id).unwrap();
    let message =
        "ADR 0215 item 1: the first role decides the rank, so the feed is below the label";
    assert_eq!(catalogs[index(label)].children, vec![own], "{message}");
    assert_eq!(catalogs[index(own)].parents, vec![label], "{message}");
    assert_eq!(catalogs[index(own)].children, Vec::new(), "{message}");
}

#[test]
fn a_release_with_two_label_items_reads_and_writes_the_same_bytes_without_an_edit() {
    let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <title>Release</title>
    <podcast:medium>music</podcast:medium>
    <podcast:publisher>
      <podcast:remoteItem medium="publisher" rel="label" feedGuid="label-b" feedUrl="https://music.example/b.xml"/>
      <podcast:remoteItem medium="publisher" rel="label" feedGuid="label-a" feedUrl="https://music.example/a.xml"/>
    </podcast:publisher>
  </channel>
</rss>
"#;
    let mut document = studio_core::xml::Document::parse_bytes(bytes).unwrap();
    let feed = Feed::read(&document).unwrap();
    let items = &feed.publisher.as_ref().unwrap().remote_items;
    assert_eq!(items.len(), 2);
    assert!(
        items
            .iter()
            .all(|item| item.rel.as_deref() == Some("label"))
    );
    feed.write(&mut document).unwrap();
    assert_eq!(document.write().as_bytes(), bytes);
}

#[test]
fn a_link_with_hosting_sets_the_publisher_level_and_a_credit_sets_no_level() {
    let mut s = setup();
    let page = new_feed(&mut s, "page.xml", "Page", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    s.collection
        .set_publisher_roles(
            page,
            &[
                PublisherRole::Label,
                PublisherRole::Hosting,
                PublisherRole::Producer,
            ],
            true,
        )
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(
        &s.collection,
        page,
        release,
        Some(&[PublisherRole::Hosting]),
    )
    .unwrap();
    publisher::link(&s.collection, label, release, None).unwrap();
    publisher::link(&s.collection, artist, release, None).unwrap();
    let catalogs = || {
        let feeds: Vec<_> = s
            .collection
            .feeds()
            .unwrap()
            .into_iter()
            .map(|row| {
                let feed = model(&s, row.id);
                (row, Some(feed))
            })
            .collect();
        let index = feeds.iter().position(|(row, _)| row.id == page).unwrap();
        publisher::catalogs(&feeds)[index].clone()
    };
    assert_eq!(catalogs().children, vec![label]);
    publisher::set_link_roles(&s.collection, page, release, &[PublisherRole::Producer]).unwrap();
    assert_eq!(catalogs().children, Vec::new());
}

#[test]
fn only_level_roles_have_a_rank_in_the_order_of_the_first_role() {
    use PublisherRole::{Artist, Curator, Hosting, Label, Producer};
    assert_eq!(publisher::rank(&[]), None);
    assert_eq!(publisher::rank(&[Producer]), None);
    assert_eq!(
        publisher::rank(&[Artist, Hosting]),
        publisher::rank(&[Artist])
    );
    assert_eq!(
        publisher::rank(&[Artist, Label]),
        publisher::rank(&[Artist])
    );
    assert!(publisher::rank(&[Hosting]) > publisher::rank(&[Label]));
    assert!(publisher::rank(&[Label]) > publisher::rank(&[Curator]));
    assert!(publisher::rank(&[Curator]) > publisher::rank(&[Artist]));
}

/// Give the catalogs of each feed of the collection.
fn all_catalogs(s: &Setup) -> (Vec<(FeedRow, Option<Feed>)>, Vec<publisher::Catalog>) {
    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    (feeds, catalogs)
}

#[test]
fn two_releases_that_connect_two_feeds_in_two_directions_give_one_level() {
    let mut s = setup();
    let duo = new_feed(&mut s, "duo.xml", "Duo", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let other = new_feed(&mut s, "other.xml", "Other", "music");
    let own = new_feed(&mut s, "own.xml", "Own", "music");
    s.collection
        .set_publisher_roles(duo, &[PublisherRole::Artist, PublisherRole::Hosting], true)
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    // The duo publishes a release of the label, and the label has a release
    // of the duo as artist.
    publisher::link(&s.collection, duo, other, Some(&[PublisherRole::Hosting])).unwrap();
    publisher::link(&s.collection, label, other, None).unwrap();
    publisher::link(&s.collection, duo, own, Some(&[PublisherRole::Artist])).unwrap();
    publisher::link(&s.collection, label, own, None).unwrap();

    let (feeds, catalogs) = all_catalogs(&s);
    let of = |id: FeedId| &catalogs[feeds.iter().position(|(row, _)| row.id == id).unwrap()];
    let message =
        "ADR 0253 item 15: two feeds are never each below the other; the rank of the feeds decides";
    assert_eq!(of(label).children, vec![duo], "{message}");
    assert_eq!(of(label).parents, Vec::new(), "{message}");
    assert_eq!(of(duo).children, Vec::new(), "{message}");
}

#[test]
fn each_level_role_of_a_link_counts_for_its_rank() {
    let mut s = setup();
    let page = new_feed(&mut s, "page.xml", "Page", "publisher");
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let curator = new_feed(&mut s, "curator.xml", "Curator", "publisher");
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    s.collection
        .set_publisher_roles(page, &[PublisherRole::Label, PublisherRole::Hosting], true)
        .unwrap();
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection
        .set_publisher_roles(curator, &[PublisherRole::Curator], true)
        .unwrap();
    publisher::link(&s.collection, page, release, None).unwrap();
    publisher::link(&s.collection, artist, release, None).unwrap();
    publisher::link(&s.collection, curator, release, None).unwrap();

    let (feeds, catalogs) = all_catalogs(&s);
    let of = |id: FeedId| &catalogs[feeds.iter().position(|(row, _)| row.id == id).unwrap()];
    let message = "ADR 0253 item 15: the highest level role of a link gives its rank";
    assert_eq!(of(page).children, vec![curator], "{message}");
    assert_eq!(of(curator).children, vec![artist], "{message}");
}

#[test]
fn a_kept_link_goes_only_when_a_shared_release_gives_the_level() {
    let mut s = setup();
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    s.collection
        .set_publisher_roles(
            label,
            &[PublisherRole::Label, PublisherRole::Producer],
            true,
        )
        .unwrap();
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection.add_kept_level(label, artist).unwrap();
    // The label link gives only a credit, so the release gives no level.
    publisher::link(
        &s.collection,
        label,
        release,
        Some(&[PublisherRole::Producer]),
    )
    .unwrap();
    publisher::link(&s.collection, artist, release, None).unwrap();
    let kept = s.collection.kept_levels().unwrap();
    let message = "ADR 0254 item 41: a kept link goes when a shared release puts the lower feed below the upper feed";
    let (feeds, _) = all_catalogs(&s);
    assert_eq!(
        publisher::replaced_kept_levels(&feeds, &kept),
        Vec::new(),
        "{message}"
    );

    publisher::set_link_roles(&s.collection, label, release, &[PublisherRole::Label]).unwrap();
    let (feeds, _) = all_catalogs(&s);
    assert_eq!(
        publisher::replaced_kept_levels(&feeds, &kept),
        vec![(label, artist)],
        "{message}"
    );
}

fn catalog_page(s: &mut Setup, name: &str, roles: &[PublisherRole]) -> FeedId {
    let id = new_feed(s, &format!("{name}.xml"), name, "publisher");
    s.collection.set_publisher_roles(id, roles, true).unwrap();
    id
}

fn release_link_roles(s: &Setup, release: FeedId, page: FeedId) -> Option<String> {
    let guid = model(s, page).podcast_guid.unwrap();
    model(s, release)
        .publisher
        .into_iter()
        .flat_map(|links| links.remote_items)
        .find(|item| item.feed_guid.as_deref() == Some(&guid))
        .map(|item| item.rel.clone().unwrap_or_default())
}

fn page_lists_release(s: &Setup, page: FeedId, release: FeedId) -> bool {
    let guid = model(s, release).podcast_guid.unwrap();
    model(s, page)
        .remote_items
        .iter()
        .any(|item| item.feed_guid.as_deref() == Some(&guid))
}

#[test]
fn a_catalog_label_change_changes_only_the_selected_release() {
    use PublisherRole::{Artist, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let old = catalog_page(&mut s, "Old label", &[Label]);
    let new = catalog_page(&mut s, "New label", &[Label]);
    let host = catalog_page(&mut s, "Publisher", &[Hosting]);
    let other_host = catalog_page(&mut s, "Old publisher", &[Hosting]);
    let first = new_feed(&mut s, "first.xml", "First", "music");
    let second = new_feed(&mut s, "second.xml", "Second", "musicL");
    for release in [first, second] {
        for page in [artist, old, other_host] {
            publisher::link(&s.collection, page, release, None).unwrap();
        }
    }
    s.collection.add_kept_level(host, new).unwrap();
    let untouched = s.collection.draft(second).unwrap();
    let result = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangeLabel,
        Some(new),
        &[first, first],
    )
    .unwrap();
    assert_eq!(result.changed_releases, 1);
    assert_eq!(result.kept_release_titles, ["Second"]);
    assert!(!result.nothing_changes);
    assert!(result.place.contains("New label"));
    assert!(result.place.contains("Publisher"));
    assert_eq!(release_link_roles(&s, first, old), None);
    assert_eq!(release_link_roles(&s, first, other_host), None);
    assert_eq!(release_link_roles(&s, first, new).as_deref(), Some("label"));
    assert_eq!(
        release_link_roles(&s, first, host).as_deref(),
        Some("hosting")
    );
    assert_eq!(
        release_link_roles(&s, first, artist).as_deref(),
        Some("artist")
    );
    assert!(!page_lists_release(&s, old, first));
    assert!(!page_lists_release(&s, other_host, first));
    assert!(page_lists_release(&s, new, first));
    assert!(page_lists_release(&s, host, first));
    assert_eq!(s.collection.draft(second).unwrap(), untouched);
    assert_eq!(
        release_link_roles(&s, second, old).as_deref(),
        Some("label")
    );
    assert_eq!(
        release_link_roles(&s, second, other_host).as_deref(),
        Some("hosting")
    );
}

#[test]
fn a_catalog_label_with_hosting_gets_only_the_selected_link_role() {
    use PublisherRole::{Artist, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let label = catalog_page(&mut s, "Label publisher", &[Label, Hosting]);
    let host_z = catalog_page(&mut s, "Zulu publisher", &[Hosting]);
    let host_a = catalog_page(&mut s, "Alpha publisher", &[Hosting]);
    let first = new_feed(&mut s, "first.xml", "First", "music");
    publisher::link(&s.collection, artist, first, None).unwrap();
    for (name, host) in [("seed-z", host_z), ("seed-a", host_a)] {
        let seed = new_feed(&mut s, &format!("{name}.xml"), name, "music");
        publisher::link(&s.collection, label, seed, Some(&[Label])).unwrap();
        publisher::link(&s.collection, host, seed, Some(&[Hosting])).unwrap();
    }
    publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::SignToLabel,
        Some(label),
        &[first],
    )
    .unwrap();
    assert_eq!(
        release_link_roles(&s, first, label).as_deref(),
        Some("label")
    );
    assert_eq!(
        release_link_roles(&s, first, host_a).as_deref(),
        Some("hosting")
    );
    assert_eq!(release_link_roles(&s, first, host_z), None);
    // Two selected roles on one page share one link (ADR 0257 item 7).
    publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangePublisher,
        Some(label),
        &[first],
    )
    .unwrap();
    assert_eq!(
        release_link_roles(&s, first, label).as_deref(),
        Some("label hosting")
    );
    assert_eq!(release_link_roles(&s, first, host_a), None);
}

#[test]
fn a_catalog_page_without_releases_replaces_its_kept_link() {
    use PublisherRole::{Artist, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let old = catalog_page(&mut s, "Old label", &[Label]);
    let new = catalog_page(&mut s, "New label", &[Label]);
    let host = catalog_page(&mut s, "Publisher", &[Hosting]);
    s.collection.add_kept_level(old, artist).unwrap();
    s.collection.add_kept_level(host, new).unwrap();
    let before = s.collection.draft(artist).unwrap();
    let result = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangeLabel,
        Some(new),
        &[],
    )
    .unwrap();
    assert_eq!(result.changed_releases, 0);
    assert!(!result.nothing_changes);
    assert_eq!(result.place, "On New label, published by Publisher");
    assert_eq!(
        s.collection.kept_levels().unwrap(),
        [(new, artist), (host, new)]
    );
    assert_eq!(s.collection.draft(artist).unwrap(), before);
}

#[test]
fn a_catalog_change_to_the_same_label_changes_nothing() {
    use PublisherRole::{Artist, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let label = catalog_page(&mut s, "Label", &[Label]);
    let host = catalog_page(&mut s, "Publisher", &[Hosting]);
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    for page in [artist, label, host] {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    let before: Vec<_> = [artist, label, host, release]
        .map(|id| s.collection.draft(id).unwrap())
        .into();
    let result = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangeLabel,
        Some(label),
        &[release],
    )
    .unwrap();
    assert!(result.nothing_changes);
    assert_eq!(result.changed_releases, 0);
    for (id, draft) in [artist, label, host, release].into_iter().zip(before) {
        assert_eq!(s.collection.draft(id).unwrap(), draft);
    }
    assert_eq!(s.collection.kept_levels().unwrap(), []);
}

#[test]
fn a_catalog_leave_all_removes_label_and_publisher_in_one_call() {
    use PublisherRole::{Artist, Curator, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let label = catalog_page(&mut s, "Label", &[Label]);
    let host = catalog_page(&mut s, "Publisher", &[Hosting]);
    let curator = catalog_page(&mut s, "Curator", &[Curator]);
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    for page in [artist, label, host, curator] {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    s.collection.add_kept_level(label, artist).unwrap();
    s.collection.add_kept_level(host, artist).unwrap();
    let result = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::LeaveAll,
        None,
        &[release],
    )
    .unwrap();
    assert_eq!(result.changed_releases, 1);
    assert_eq!(result.place, "Independent, not published");
    assert_eq!(release_link_roles(&s, release, label), None);
    assert_eq!(release_link_roles(&s, release, host), None);
    assert_eq!(
        release_link_roles(&s, release, artist).as_deref(),
        Some("artist")
    );
    assert_eq!(
        release_link_roles(&s, release, curator).as_deref(),
        Some("curator")
    );
    assert!(!page_lists_release(&s, label, release));
    assert!(!page_lists_release(&s, host, release));
    assert_eq!(s.collection.kept_levels().unwrap(), []);
}

#[test]
fn a_catalog_change_removes_outside_roles_and_keeps_other_roles() {
    use PublisherRole::{Artist, Curator, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let old = catalog_page(&mut s, "Old label", &[Label, Curator]);
    let new = catalog_page(&mut s, "New label", &[Label]);
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    publisher::link(&s.collection, artist, release, None).unwrap();
    publisher::link(&s.collection, old, release, None).unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        let links = &mut feed.publisher.as_mut().unwrap().remote_items;
        links.push(Entry::new(remote(
            "publisher",
            "outside-label",
            "https://outside.example/label.xml",
            "Outside label",
            Some("label curator custom"),
        )));
        links.push(Entry::new(remote(
            "publisher",
            "outside-publisher",
            "https://outside.example/publisher.xml",
            "Outside publisher",
            Some("hosting"),
        )));
    })
    .unwrap();
    publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangeLabel,
        Some(new),
        &[release],
    )
    .unwrap();
    assert_eq!(
        release_link_roles(&s, release, old).as_deref(),
        Some("curator")
    );
    assert_eq!(
        model(&s, old).remote_items[0].rel.as_deref(),
        Some("curator")
    );
    let feed = model(&s, release);
    let links = &feed.publisher.as_ref().unwrap().remote_items;
    assert_eq!(
        links
            .iter()
            .find(|item| item.feed_guid.as_deref() == Some("outside-label"))
            .unwrap()
            .rel
            .as_deref(),
        Some("curator custom")
    );
    assert!(
        !links
            .iter()
            .any(|item| item.feed_guid.as_deref() == Some("outside-publisher"))
    );
    assert_eq!(
        release_link_roles(&s, release, new).as_deref(),
        Some("label")
    );
}

#[test]
fn a_catalog_start_selection_uses_the_selected_tree_parent() {
    use PublisherRole::{Artist, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let first_label = catalog_page(&mut s, "First label", &[Label]);
    let second_label = catalog_page(&mut s, "Second label", &[Label]);
    let first = new_feed(&mut s, "first.xml", "First", "music");
    let second = new_feed(&mut s, "second.xml", "Second", "music");
    for (release, label) in [(first, first_label), (second, second_label)] {
        publisher::link(&s.collection, artist, release, None).unwrap();
        publisher::link(&s.collection, label, release, None).unwrap();
    }
    let (feeds, _) = all_catalogs(&s);
    assert_eq!(
        publisher::page_change_selection(&feeds, artist, Some(first_label)),
        [first]
    );
    assert_eq!(
        publisher::page_change_selection(&feeds, artist, Some(second_label)),
        [second]
    );
    assert_eq!(
        publisher::page_change_selection(&feeds, artist, Some(FeedId(-1))),
        [first, second]
    );
    assert_eq!(
        publisher::page_change_selection(&feeds, artist, None),
        [first, second]
    );
}

#[test]
fn a_catalog_failed_change_puts_back_each_draft_and_kept_link() {
    use PublisherRole::{Artist, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let old = catalog_page(&mut s, "Old label", &[Label]);
    let new = catalog_page(&mut s, "New label", &[Label]);
    let first = new_feed(&mut s, "first.xml", "First", "music");
    let second = new_feed(&mut s, "second.xml", "Second", "music");
    for release in [first, second] {
        publisher::link(&s.collection, artist, release, None).unwrap();
        publisher::link(&s.collection, old, release, None).unwrap();
    }
    s.collection.add_kept_level(old, artist).unwrap();
    s.collection.delete_draft(new).unwrap();
    let ids = [artist, old, new, first, second];
    let before = ids.map(|id| s.collection.draft(id).unwrap());
    let error = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::ChangeLabel,
        Some(new),
        &[first, second],
    )
    .unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::Pipeline);
    for (id, draft) in ids.into_iter().zip(before) {
        assert_eq!(s.collection.draft(id).unwrap(), draft);
    }
    assert_eq!(s.collection.kept_levels().unwrap(), [(old, artist)]);
}

#[test]
fn a_catalog_independent_action_keeps_the_publisher_and_other_link_roles() {
    use PublisherRole::{Artist, Curator, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let label = catalog_page(&mut s, "Label", &[Label, Curator]);
    let host = catalog_page(&mut s, "Publisher", &[Hosting]);
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    for page in [artist, label, host] {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    s.collection.add_kept_level(label, artist).unwrap();
    s.collection.add_kept_level(host, artist).unwrap();
    let host_before = s.collection.draft(host).unwrap();
    let result = publisher::change_page(
        &s.collection,
        None,
        artist,
        publisher::PageAction::GoIndependent,
        None,
        &[release],
    )
    .unwrap();
    assert_eq!(result.changed_releases, 1);
    assert_eq!(
        release_link_roles(&s, release, label).as_deref(),
        Some("curator")
    );
    assert_eq!(
        release_link_roles(&s, release, host).as_deref(),
        Some("hosting")
    );
    assert_eq!(s.collection.draft(host).unwrap(), host_before);
    assert!(
        !s.collection
            .kept_levels()
            .unwrap()
            .contains(&(label, artist))
    );
}

#[test]
fn a_catalog_label_changes_publisher_and_can_be_not_published() {
    use PublisherRole::{Artist, Hosting, Label};
    let mut s = setup();
    let artist = catalog_page(&mut s, "Artist", &[Artist]);
    let label = catalog_page(&mut s, "Label", &[Label]);
    let old = catalog_page(&mut s, "Old publisher", &[Hosting]);
    let new = catalog_page(&mut s, "New publisher", &[Hosting]);
    let release = new_feed(&mut s, "release.xml", "Release", "music");
    for page in [artist, label, old] {
        publisher::link(&s.collection, page, release, None).unwrap();
    }
    s.collection.add_kept_level(old, label).unwrap();
    publisher::change_page(
        &s.collection,
        None,
        label,
        publisher::PageAction::ChangePublisher,
        Some(new),
        &[release],
    )
    .unwrap();
    assert_eq!(release_link_roles(&s, release, old), None);
    assert_eq!(
        release_link_roles(&s, release, new).as_deref(),
        Some("hosting")
    );
    assert_eq!(
        release_link_roles(&s, release, artist).as_deref(),
        Some("artist")
    );
    assert_eq!(
        release_link_roles(&s, release, label).as_deref(),
        Some("label")
    );
    let result = publisher::change_page(
        &s.collection,
        None,
        label,
        publisher::PageAction::ChangePublisher,
        None,
        &[release],
    )
    .unwrap();
    assert_eq!(result.changed_releases, 1);
    assert_eq!(release_link_roles(&s, release, new), None);
    assert_eq!(
        release_link_roles(&s, release, label).as_deref(),
        Some("label")
    );
    assert_eq!(
        release_link_roles(&s, release, artist).as_deref(),
        Some("artist")
    );
    assert_eq!(s.collection.kept_levels().unwrap(), []);
}
