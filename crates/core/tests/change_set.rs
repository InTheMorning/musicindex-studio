//! Change sets over the local folder connector (ADR 0091).

use std::collections::HashMap;
use std::fs;

use studio_core::change_set::{
    self, Cause, Outcome, RecipientKey, RecipientOperation, discard_change_set, level_feeds,
    make_change_set,
};
use studio_core::collection::{Collection, FeedId, KnownRecipient};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::Recipient;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::pipeline::{self, AddressCheck};
use studio_core::publisher::Catalog;

const FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Album</title>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <podcast:value type=\"lightning\" method=\"keysend\">
      <podcast:valueRecipient name=\"Old\" type=\"node\" address=\"old\" split=\"50\"/>
      <podcast:valueRecipient name=\"Other\" type=\"node\" address=\"other\" split=\"50\"/>
    </podcast:value>
    <item>
      <title>Track One</title>
      <guid>track-1</guid>
    </item>
  </channel>
</rss>
";

struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

struct Setup {
    dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    hosts: Hosts,
    library: Library,
    feeds: Vec<FeedId>,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector =
        connector::from_row(&collection.connector(connector_id).unwrap(), None).unwrap();
    let mut feeds = Vec::new();
    for name in ["one.xml", "two.xml"] {
        folder
            .write(name, FEED.as_bytes(), &Expected::Absent)
            .unwrap();
        let feed = collection
            .add_feed(
                Some(&format!("https://music.example/{name}")),
                Some((connector_id, name)),
            )
            .unwrap();
        pipeline::refresh(&mut collection, connector.as_ref(), feed).unwrap();
        feeds.push(feed);
    }
    let mut hosts = Hosts::new();
    hosts.insert(connector_id, Box::new(folder));
    let library = Library::open(dir.path().join("library")).unwrap();
    Setup {
        dir,
        collection,
        connector,
        hosts,
        library,
        feeds,
    }
}

fn key() -> RecipientKey {
    RecipientKey {
        address: "old".into(),
        custom_key: None,
        custom_value: None,
    }
}

fn change_address() -> RecipientOperation {
    RecipientOperation::ChangeAddress {
        address: "new".into(),
        custom_key: None,
        custom_value: None,
        recipient_type: Some("node".into()),
    }
}

fn start(s: &Setup) -> impl FnMut(FeedId) -> Result<(), String> + '_ {
    |id| {
        pipeline::start_draft(&s.collection, s.connector.as_ref(), id)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    }
}

fn addresses(collection: &Collection, id: FeedId) -> Vec<String> {
    let (feed, _) = pipeline::draft_model(collection, id).unwrap();
    feed.recipients()
        .iter()
        .filter_map(|r| r.address.clone())
        .collect()
}

#[test]
fn an_open_draft_keeps_its_edits_and_a_feed_with_no_draft_gets_a_draft() {
    let s = setup();
    let (one, two) = (s.feeds[0], s.feeds[1]);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), one).unwrap();
    pipeline::edit_draft(&s.collection, one, |f| f.title = Some("Edited".into())).unwrap();
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &change_address(),
        &[one, two],
    )
    .unwrap();
    assert!(set.feeds.iter().all(|f| f.outcome.is_changed()));
    let (feed, _) = pipeline::draft_model(&s.collection, one).unwrap();
    assert_eq!(feed.title.as_deref(), Some("Edited"));
    assert_eq!(addresses(&s.collection, one), ["new", "other"]);
    assert_eq!(addresses(&s.collection, two), ["new", "other"]);
    assert_eq!(change_set::change_sets(&s.collection).unwrap().len(), 1);
}

#[test]
fn a_feed_that_does_not_change_keeps_its_draft_bytes_and_makes_no_draft() {
    let s = setup();
    let (one, two) = (s.feeds[0], s.feeds[1]);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), one).unwrap();
    let before = s.collection.draft(one).unwrap().unwrap();
    let operation = RecipientOperation::Rename { name: "X".into() };
    let nobody = RecipientKey {
        address: "nobody".into(),
        custom_key: None,
        custom_value: None,
    };
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &nobody,
        &operation,
        &[one, two],
    )
    .unwrap();
    assert!(set.feeds.iter().all(|f| f.outcome
        == Outcome::NotChanged {
            cause: Cause::NoRecipient
        }));
    assert_eq!(s.collection.draft(one).unwrap().unwrap(), before);
    assert!(s.collection.draft(two).unwrap().is_none());
    assert_eq!(change_set::change_sets(&s.collection).unwrap().len(), 0);
}

#[test]
fn a_failed_start_is_the_outcome_cannot_start() {
    let s = setup();
    let mut starter = |_: FeedId| Err("no host".to_owned());
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &change_address(),
        &[s.feeds[0]],
    )
    .unwrap();
    assert_eq!(
        set.feeds[0].outcome,
        Outcome::NotChanged {
            cause: Cause::CannotStart("no host".into())
        }
    );
}

#[test]
fn discard_puts_back_each_kept_draft_byte_for_byte() {
    let s = setup();
    let (one, two) = (s.feeds[0], s.feeds[1]);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), one).unwrap();
    pipeline::edit_draft(&s.collection, one, |f| f.title = Some("Edited".into())).unwrap();
    let before = s.collection.draft(one).unwrap().unwrap();
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &change_address(),
        &[one, two],
    )
    .unwrap();
    assert_ne!(s.collection.draft(one).unwrap().unwrap(), before);
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(s.collection.draft(one).unwrap().unwrap(), before);
    assert!(s.collection.draft(two).unwrap().is_none());
    assert_eq!(s.collection.change_set_rows().unwrap().len(), 0);
}

#[test]
fn discard_does_not_change_a_feed_that_was_written() {
    let mut s = setup();
    let (one, two) = (s.feeds[0], s.feeds[1]);
    let set = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &key(),
            &change_address(),
            &[one, two],
        )
        .unwrap()
    };
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, one).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        one,
        &diff.id,
    )
    .unwrap();
    let written = fs::read_to_string(s.dir.path().join("one.xml")).unwrap();
    assert!(written.contains("address=\"new\""));
    // Feed two is still open, so the change set is open.
    let open = change_set::change_sets(&s.collection).unwrap();
    assert_eq!(open.len(), 1);
    assert!(open[0].feeds[0].written_at.is_some());
    assert!(open[0].feeds[1].written_at.is_none());

    // A new draft of feed one is not part of the change set.
    pipeline::start_draft(&s.collection, s.connector.as_ref(), one).unwrap();
    let later = s.collection.draft(one).unwrap().unwrap();
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(s.collection.draft(one).unwrap().unwrap(), later);
    assert_eq!(
        fs::read_to_string(s.dir.path().join("one.xml")).unwrap(),
        written
    );
    assert!(s.collection.draft(two).unwrap().is_none());
}

#[test]
fn level_feeds_gives_a_label_its_artist_and_the_releases_of_both_one_time() {
    let (label, artist, album_a, album_b) = (FeedId(1), FeedId(2), FeedId(3), FeedId(4));
    let mut catalogs = HashMap::new();
    catalogs.insert(
        label,
        Catalog {
            children: vec![artist],
            releases: vec![album_a, album_b],
            ..Catalog::default()
        },
    );
    catalogs.insert(
        artist,
        Catalog {
            parents: vec![label],
            releases: vec![album_a],
            ..Catalog::default()
        },
    );
    let found = level_feeds(&catalogs, label);
    assert_eq!(found, [label, artist, album_a, album_b]);
    // A cycle in the links ends.
    catalogs.get_mut(&artist).unwrap().children = vec![label];
    assert_eq!(level_feeds(&catalogs, label).len(), 4);
}

#[test]
fn a_feed_in_an_open_change_set_is_not_changed_by_a_second_one() {
    let s = setup();
    let one = s.feeds[0];
    let first = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &key(),
            &change_address(),
            &[one],
        )
        .unwrap()
    };
    assert!(first.feeds[0].outcome.is_changed());
    let kept = s.collection.draft(one).unwrap().unwrap();
    let second_operation = RecipientOperation::Rename { name: "X".into() };
    let new_key = RecipientKey {
        address: "new".into(),
        custom_key: None,
        custom_value: None,
    };
    let second = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &new_key,
            &second_operation,
            &[one],
        )
        .unwrap()
    };
    assert_eq!(
        second.feeds[0].outcome,
        Outcome::NotChanged {
            cause: Cause::InOpenChangeSet
        }
    );
    assert_eq!(s.collection.draft(one).unwrap().unwrap(), kept);
}

#[test]
fn a_discarded_draft_closes_its_row_so_discard_keeps_it_away() {
    let s = setup();
    let (one, two) = (s.feeds[0], s.feeds[1]);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), one).unwrap();
    pipeline::edit_draft(&s.collection, one, |f| f.title = Some("Edited".into())).unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), two).unwrap();
    pipeline::edit_draft(&s.collection, two, |f| f.title = Some("Two".into())).unwrap();
    let before_two = s.collection.draft(two).unwrap().unwrap();
    let set = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &key(),
            &change_address(),
            &[one, two],
        )
        .unwrap()
    };
    pipeline::discard_draft(&s.collection, one).unwrap();
    s.collection.delete_draft(one).unwrap();
    discard_change_set(&s.collection, set.id).unwrap();
    assert!(s.collection.draft(one).unwrap().is_none());
    assert_eq!(s.collection.draft(two).unwrap().unwrap(), before_two);
}

fn saved(collection: &Collection) -> Vec<KnownRecipient> {
    collection
        .known_recipients()
        .unwrap()
        .into_iter()
        .filter(|r| r.saved_id.is_some())
        .collect()
}

fn save(collection: &Collection, name: &str, custom: Option<(&str, &str)>) {
    collection
        .save_recipient(&Recipient {
            name: Some(name.into()),
            recipient_type: Some("lnaddress".into()),
            address: Some("old".into()),
            custom_key: custom.map(|c| c.0.into()),
            custom_value: custom.map(|c| c.1.into()),
            ..Recipient::default()
        })
        .unwrap();
}

#[test]
fn rename_changes_the_saved_recipient_and_discard_puts_back_its_name() {
    let s = setup();
    save(&s.collection, "Old name", None);
    let before = saved(&s.collection);
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &RecipientOperation::Rename {
            name: "New name".into(),
        },
        &s.feeds,
    )
    .unwrap();
    assert_eq!(saved(&s.collection)[0].name.as_deref(), Some("New name"));
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(saved(&s.collection), before);
}

#[test]
fn change_address_changes_the_saved_recipient_and_discard_puts_back_its_identity() {
    let s = setup();
    save(&s.collection, "Old name", Some(("k", "v")));
    let before = saved(&s.collection);
    let old_key = RecipientKey {
        address: "old".into(),
        custom_key: Some("k".into()),
        custom_value: Some("v".into()),
    };
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &old_key,
        &RecipientOperation::ChangeAddress {
            address: "new".into(),
            custom_key: Some("k2".into()),
            custom_value: Some("v2".into()),
            recipient_type: Some("node".into()),
        },
        &s.feeds,
    )
    .unwrap();
    let changed = saved(&s.collection);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].address, "new");
    assert_eq!(changed[0].custom_key.as_deref(), Some("k2"));
    assert_eq!(changed[0].custom_value.as_deref(), Some("v2"));
    assert_eq!(changed[0].recipient_type.as_deref(), Some("node"));
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(saved(&s.collection), before);
    assert_eq!(before[0].address, "old");
    assert_eq!(before[0].custom_key.as_deref(), Some("k"));
    assert_eq!(before[0].custom_value.as_deref(), Some("v"));
    assert_eq!(before[0].recipient_type.as_deref(), Some("lnaddress"));
}

#[test]
fn discard_of_a_change_set_with_no_saved_recipient_keeps_the_saved_recipients() {
    let s = setup();
    // A different recipient is saved. The key of the change set is not.
    s.collection
        .save_recipient(&Recipient {
            name: Some("Other".into()),
            address: Some("other".into()),
            ..Recipient::default()
        })
        .unwrap();
    let before = saved(&s.collection);
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &change_address(),
        &s.feeds,
    )
    .unwrap();
    assert_eq!(saved(&s.collection), before);
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(saved(&s.collection), before);
}
