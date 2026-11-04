//! Change sets over the local folder connector (ADR 0091).

use std::collections::HashMap;
use std::fs;

use studio_core::change_set::{
    self, Cause, CreditMatch, Outcome, PersonOperation, RecipientKey, RecipientOperation, Wallet,
    discard_change_set, level_feeds, make_change_set, make_person_change_set, person_operation,
};
use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::Recipient;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::move_link::ScriptedReader;
use studio_core::people::{self, Person, PersonFields};
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

/// A feed whose value block holds the two members of one wallet.
const WALLET_FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Album</title>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <podcast:value type=\"lightning\" method=\"keysend\">
      <podcast:valueRecipient name=\"Ann\" type=\"node\" address=\"node-key\" split=\"40\"/>
      <podcast:valueRecipient name=\"Ann\" type=\"lnaddress\" address=\"ann@example.com\" split=\"20\"/>
      <podcast:valueRecipient name=\"Other\" type=\"node\" address=\"other\" split=\"40\"/>
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
    setup_with(FEED)
}

fn setup_with(xml: &str) -> Setup {
    setup_each(&[xml, xml])
}

/// A setup with one feed for each text in `xmls`.
fn setup_each(xmls: &[&str]) -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector = connector::from_row(
        &collection.connector(connector_id).unwrap(),
        None,
        &studio_core::net::NetPolicy::Open,
    )
    .unwrap();
    let mut feeds = Vec::new();
    for (name, xml) in ["one.xml", "two.xml"].into_iter().zip(xmls) {
        folder
            .write(name, xml.as_bytes(), &Expected::Absent)
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

fn plain(address: &str) -> RecipientKey {
    RecipientKey {
        address: address.into(),
        custom_key: None,
        custom_value: None,
    }
}

fn key() -> Wallet {
    Wallet::single(plain("old"))
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
    let nobody = Wallet::single(plain("nobody"));
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
            cause: Cause::NoRecipient,
            parts: Vec::new()
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
            cause: Cause::CannotStart("no host".into()),
            parts: Vec::new()
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
        &ScriptedReader::empty(),
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
    let new_key = Wallet::single(plain("new"));
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
            cause: Cause::InOpenChangeSet,
            parts: Vec::new()
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

/// Each person of the collection, as the test compares them.
fn persons(collection: &Collection) -> Vec<Person> {
    people::people(collection).unwrap()
}

/// Make a person with one wallet of the address `address`.
fn save(collection: &Collection, name: &str, address: &str, custom: Option<(&str, &str)>) {
    people::save_recipient(
        collection,
        &Recipient {
            name: Some(name.into()),
            recipient_type: Some("lnaddress".into()),
            address: Some(address.into()),
            custom_key: custom.map(|c| c.0.into()),
            custom_value: custom.map(|c| c.1.into()),
            ..Recipient::default()
        },
    )
    .unwrap();
}

#[test]
fn rename_sets_the_name_of_the_person_and_discard_keeps_the_new_name() {
    let s = setup();
    save(&s.collection, "Old name", "old", None);
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
    assert_eq!(persons(&s.collection)[0].name, "New name");
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(
        persons(&s.collection)[0].name,
        "New name",
        "ADR 0098 item 23: Discard must not set the name of a person back."
    );
}

#[test]
fn change_address_does_not_change_the_wallets_of_the_person() {
    let s = setup();
    save(&s.collection, "Old name", "old", None);
    let before = persons(&s.collection);
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &RecipientOperation::ChangeAddress {
            address: "new".into(),
            custom_key: Some("k2".into()),
            custom_value: Some("v2".into()),
            recipient_type: Some("node".into()),
        },
        &s.feeds,
    )
    .unwrap();
    assert_eq!(
        persons(&s.collection),
        before,
        "ADR 0098 item 23: a change set must not change a person."
    );
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(persons(&s.collection), before);
    assert_eq!(before[0].wallets[0].key.address, "old");
}

#[test]
fn a_change_set_does_not_change_a_person_with_no_wallet_of_it() {
    let s = setup();
    // A different recipient is saved. The key of the change set is not.
    save(&s.collection, "Other", "other-saved", None);
    let before = persons(&s.collection);
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &RecipientOperation::Rename { name: "New".into() },
        &s.feeds,
    )
    .unwrap();
    assert_eq!(persons(&s.collection), before);
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(persons(&s.collection), before);
}

#[test]
fn a_change_set_with_no_changed_feed_keeps_the_person_and_goes_at_the_next_one() {
    let s = setup();
    save(&s.collection, "Old name", "old", Some(("k", "v")));
    let before = persons(&s.collection);
    // No test feed has this key, so no feed changes.
    let other = Wallet::single(RecipientKey {
        address: "old".into(),
        custom_key: Some("k".into()),
        custom_value: Some("v".into()),
    });
    let first = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &other,
            &RecipientOperation::Rename {
                name: "New name".into(),
            },
            &s.feeds,
        )
        .unwrap()
    };
    assert_eq!(
        persons(&s.collection),
        before,
        "ADR 0098 item 23: a Rename with no changed feed must not rename a person."
    );
    assert!(
        change_set::find_change_set(&s.collection, first.id).is_ok(),
        "The review must find the change set before the next change set."
    );
    let mut starter = start(&s);
    make_change_set(
        &s.collection,
        &mut starter,
        &key(),
        &change_address(),
        &s.feeds,
    )
    .unwrap();
    // SQLite can give the new change set the ID of the deleted one, so the
    // test examines the operation of each row.
    let rows = s.collection.change_set_rows().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "ADR 0091 item 19: a closed change set must go at the next change set."
    );
    assert!(rows[0].operation.contains("change_address"));
}

#[test]
fn the_write_of_the_last_changed_feed_deletes_the_change_set() {
    let mut s = setup();
    let one = s.feeds[0];
    let set = {
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
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, one).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        &ScriptedReader::empty(),
        one,
        &diff.id,
    )
    .unwrap();
    assert!(
        change_set::find_change_set(&s.collection, set.id).is_err(),
        "ADR 0091 item 19: the write of the last changed feed must delete the change set."
    );
}

fn two_member_wallet() -> Wallet {
    Wallet {
        key: plain("node-key"),
        members: vec![plain("node-key"), plain("ann@example.com")],
    }
}

fn wallet_splits(collection: &Collection, id: FeedId) -> Vec<(String, String)> {
    let (feed, _) = pipeline::draft_model(collection, id).unwrap();
    feed.recipients()
        .iter()
        .map(|r| {
            (
                r.address.clone().unwrap_or_default(),
                r.split.clone().unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn rename_and_change_address_change_each_member_of_a_wallet_in_one_feed() {
    let s = setup_with(WALLET_FEED);
    let one = s.feeds[0];
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &two_member_wallet(),
        &RecipientOperation::Rename {
            name: "Anna".into(),
        },
        &[one],
    )
    .unwrap();
    assert!(set.feeds[0].outcome.is_changed());
    let (feed, _) = pipeline::draft_model(&s.collection, one).unwrap();
    let names: Vec<_> = feed.recipients().iter().map(|r| r.name.clone()).collect();
    assert_eq!(
        names,
        [
            Some("Anna".into()),
            Some("Anna".into()),
            Some("Other".into())
        ],
        "ADR 0095 item 9: Rename must act on each member of the wallet."
    );
    discard_change_set(&s.collection, set.id).unwrap();
    let mut starter = start(&s);
    make_change_set(
        &s.collection,
        &mut starter,
        &two_member_wallet(),
        &RecipientOperation::ChangeAddress {
            address: "new".into(),
            custom_key: None,
            custom_value: None,
            recipient_type: Some("lnaddress".into()),
        },
        &[one],
    )
    .unwrap();
    assert_eq!(
        addresses(&s.collection, one),
        ["new", "new", "other"],
        "ADR 0095 item 9: Change address must act on each member of the wallet."
    );
}

#[test]
fn remove_scales_the_other_splits_by_the_total_of_the_members() {
    let s = setup_with(WALLET_FEED);
    let one = s.feeds[0];
    let mut starter = start(&s);
    make_change_set(
        &s.collection,
        &mut starter,
        &two_member_wallet(),
        &RecipientOperation::Remove,
        &[one],
    )
    .unwrap();
    assert_eq!(
        wallet_splits(&s.collection, one),
        [("other".to_owned(), "100".to_owned())]
    );
}

#[test]
fn use_lnaddress_writes_the_attributes_and_the_method() {
    let mut s = setup_with(WALLET_FEED);
    let one = s.feeds[0];
    let set = {
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &two_member_wallet(),
            &RecipientOperation::UseLnaddress {
                address: "ann@example.com".into(),
            },
            &[one],
        )
        .unwrap()
    };
    assert!(set.feeds[0].outcome.is_changed());
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, one).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        &ScriptedReader::empty(),
        one,
        &diff.id,
    )
    .unwrap();
    let written = fs::read_to_string(s.dir.path().join("one.xml")).unwrap();
    assert!(written.contains("method=\"lnaddress\""), "{written}");
    assert!(!written.contains("node-key"), "{written}");
    assert!(written.contains("type=\"lnaddress\""), "{written}");
}

#[test]
fn use_lnaddress_with_no_node_recipient_gives_the_cause_and_keeps_the_bytes() {
    let xml = WALLET_FEED.replace(
        "      <podcast:valueRecipient name=\"Ann\" type=\"node\" address=\"node-key\" split=\"40\"/>\n",
        "",
    );
    let s = setup_with(&xml);
    let one = s.feeds[0];
    let mut starter = start(&s);
    let set = make_change_set(
        &s.collection,
        &mut starter,
        &two_member_wallet(),
        &RecipientOperation::UseLnaddress {
            address: "ann@example.com".into(),
        },
        &[one],
    )
    .unwrap();
    assert_eq!(
        set.feeds[0].outcome,
        Outcome::NotChanged {
            cause: Cause::NoNodeRecipient,
            parts: Vec::new()
        }
    );
    assert!(s.collection.draft(one).unwrap().is_none());
}

#[test]
fn change_address_sets_the_method_only_for_the_type_lnaddress() {
    let method_after = |recipient_type: &str| {
        let s = setup();
        let one = s.feeds[0];
        let mut starter = start(&s);
        make_change_set(
            &s.collection,
            &mut starter,
            &key(),
            &RecipientOperation::ChangeAddress {
                address: "new@example.com".into(),
                custom_key: None,
                custom_value: None,
                recipient_type: Some(recipient_type.into()),
            },
            &[one],
        )
        .unwrap();
        let (feed, _) = pipeline::draft_model(&s.collection, one).unwrap();
        feed.values[0].value.method.clone()
    };
    assert_eq!(method_after("lnaddress").as_deref(), Some("lnaddress"));
    assert_eq!(method_after("node").as_deref(), Some("keysend"));
}

#[test]
fn an_operation_with_no_type_is_refused_and_leaves_no_change_set() {
    let s = setup();
    for recipient_type in [None, Some("  ".to_owned())] {
        let change = RecipientOperation::ChangeAddress {
            address: "new".into(),
            custom_key: None,
            custom_value: None,
            recipient_type: recipient_type.clone(),
        };
        let add = RecipientOperation::Add {
            name: None,
            recipient_type,
            address: "n".into(),
            custom_key: None,
            custom_value: None,
            percent: 10,
        };
        for operation in [change, add] {
            let mut starter = start(&s);
            let error = make_change_set(&s.collection, &mut starter, &key(), &operation, &s.feeds)
                .unwrap_err();
            assert!(
                format!("{error:?}").contains("needs a type"),
                "ADR 0095 item 14: an operation with no type must give the error invalid."
            );
            assert_eq!(s.collection.change_set_rows().unwrap().len(), 0);
        }
    }
}

#[test]
fn rename_sets_the_name_of_the_person_with_any_member_of_the_wallet() {
    let s = setup_with(WALLET_FEED);
    let id = people::add_person(
        &s.collection,
        &PersonFields {
            name: "Ann".into(),
            ..PersonFields::default()
        },
    )
    .unwrap();
    // The person holds the Lightning address of the wallet, not its key.
    people::add_wallet(
        &s.collection,
        id,
        &plain("ann@example.com"),
        "lnaddress",
        people::LinkEvidence::User,
    )
    .unwrap();
    let mut starter = start(&s);
    make_change_set(
        &s.collection,
        &mut starter,
        &two_member_wallet(),
        &RecipientOperation::Rename {
            name: "Anna".into(),
        },
        &s.feeds,
    )
    .unwrap();
    assert_eq!(people::person(&s.collection, id).unwrap().name, "Anna");
}

/// A feed that pays the one address `address`.
fn paying(address: &str) -> String {
    FEED.replace("address=\"old\"", &format!("address=\"{address}\""))
}

/// A person with a wallet for each of `addresses`, with the first one as
/// the default address.
fn person_with(collection: &Collection, name: &str, addresses: &[&str]) -> people::PersonId {
    let id = people::add_person(
        collection,
        &PersonFields {
            name: name.into(),
            ..PersonFields::default()
        },
    )
    .unwrap();
    for address in addresses {
        people::add_wallet(
            collection,
            id,
            &plain(address),
            "node",
            people::LinkEvidence::User,
        )
        .unwrap();
    }
    id
}

fn no_lookups() -> HashMap<String, studio_core::keysend::Lookup> {
    HashMap::new()
}

#[test]
fn person_operation_makes_the_wallet_and_the_credit_match_of_the_person() {
    let s = setup();
    let id = person_with(&s.collection, "Ann", &["first", "second"]);
    people::add_credit_name(&s.collection, id, "Ann Lee", "", "").unwrap();
    people::add_key(&s.collection, id, "npub1ann").unwrap();
    let (wallet, operation) = person_operation(
        &s.collection,
        id,
        &PersonOperation::Remove {
            payee: true,
            credit: true,
        },
        &no_lookups(),
    )
    .unwrap();
    assert_eq!(wallet.key, plain("first"));
    assert_eq!(wallet.members, [plain("first"), plain("second")]);
    let RecipientOperation::RemovePerson { credits, .. } = operation else {
        panic!("ADR 0098 item 20: Remove must make the operation RemovePerson.");
    };
    assert_eq!(
        credits,
        CreditMatch {
            identities: vec![people::Identity::of("Ann Lee", "", "").unwrap().key()],
            npubs: vec!["npub1ann".into()],
        }
    );
}

#[test]
fn use_this_address_changes_a_feed_of_each_wallet_and_keeps_the_person() {
    let s = setup_each(&[&paying("first"), &paying("second")]);
    let id = person_with(&s.collection, "Ann", &["first", "second", "third"]);
    let mut starter = start(&s);
    let set = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &PersonOperation::UseAddress {
            key: plain("third"),
        },
        &s.feeds,
    )
    .unwrap();
    assert!(
        set.feeds.iter().all(|f| f.outcome.is_changed()),
        "ADR 0098 item 20: Use this address must change each wallet of the person."
    );
    assert_eq!(set.person, Some(id));
    assert_eq!(addresses(&s.collection, s.feeds[0]), ["third", "other"]);
    assert_eq!(addresses(&s.collection, s.feeds[1]), ["third", "other"]);
    let listed = change_set::change_sets(&s.collection).unwrap();
    assert_eq!(listed[0].person, Some(id));
}

#[test]
fn use_this_address_refuses_an_address_that_is_not_in_a_wallet_of_the_person() {
    let s = setup();
    let id = person_with(&s.collection, "Ann", &["first"]);
    let mut starter = start(&s);
    let error = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &PersonOperation::UseAddress {
            key: plain("stranger"),
        },
        &s.feeds,
    )
    .unwrap_err();
    assert!(format!("{error}").contains("not in a wallet"));
    assert_eq!(s.collection.change_set_rows().unwrap().len(), 0);
}

#[test]
fn rename_sets_the_name_of_the_person_when_a_feed_changed_and_discard_keeps_it() {
    let s = setup();
    let id = person_with(&s.collection, "Old", &["old"]);
    let mut starter = start(&s);
    let rename = PersonOperation::Rename {
        name: "New".into(),
        payment: true,
        credit: false,
    };
    let set = make_person_change_set(&s.collection, &mut starter, id, &rename, &s.feeds).unwrap();
    assert!(set.feeds.iter().all(|f| f.outcome.is_changed()));
    assert_eq!(people::person(&s.collection, id).unwrap().name, "New");
    discard_change_set(&s.collection, set.id).unwrap();
    assert_eq!(
        people::person(&s.collection, id).unwrap().name,
        "New",
        "ADR 0098 item 23: Discard must not set the name of a person back."
    );
}

#[test]
fn rename_that_changes_no_feed_keeps_the_name_of_the_person() {
    let s = setup();
    let id = person_with(&s.collection, "Old", &["nobody"]);
    let mut starter = start(&s);
    let set = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &PersonOperation::Rename {
            name: "New".into(),
            payment: true,
            credit: true,
        },
        &s.feeds,
    )
    .unwrap();
    assert!(set.feeds.iter().all(|f| !f.outcome.is_changed()));
    assert_eq!(people::person(&s.collection, id).unwrap().name, "Old");
}

#[test]
fn a_person_with_no_default_address_gives_an_invalid_request_for_a_payee() {
    let s = setup();
    let id = person_with(&s.collection, "Ann", &[]);
    let operation = PersonOperation::AddToReleases {
        payee_percent: Some(10),
        credit: true,
        support_link: None,
    };
    let error = person_operation(&s.collection, id, &operation, &no_lookups()).unwrap_err();
    assert!(
        format!("{error}").contains("no address"),
        "ADR 0098 item 20: a payee with no default address must give an invalid request."
    );
    let mut starter = start(&s);
    assert!(make_person_change_set(&s.collection, &mut starter, id, &operation, &s.feeds).is_err());
    assert_eq!(s.collection.change_set_rows().unwrap().len(), 0);
}

#[test]
fn add_to_releases_writes_the_usual_role_and_the_usual_group() {
    let s = setup();
    let operation = PersonOperation::AddToReleases {
        payee_percent: None,
        credit: true,
        support_link: None,
    };
    let make = |name: &str, role: &str, group: &str| {
        let id = people::add_person(
            &s.collection,
            &PersonFields {
                name: name.into(),
                usual_role: role.into(),
                usual_group: group.into(),
                ..PersonFields::default()
            },
        )
        .unwrap();
        let (_, op) = person_operation(&s.collection, id, &operation, &no_lookups()).unwrap();
        let RecipientOperation::AddToReleases {
            credit: Some(credit),
            ..
        } = op
        else {
            panic!("ADR 0112 item 6: AddToReleases must carry a credit.");
        };
        (credit.role, credit.group)
    };
    assert_eq!(
        make("A", "Vocalist", ""),
        ("vocalist".to_owned(), "music".to_owned()),
        "ADR 0112 item 6: a usual role of the list with no usual group must get the first group of the role."
    );
    assert_eq!(
        make("B", "Editor", "Video Post-Production"),
        ("editor".to_owned(), "video post-production".to_owned()),
        "ADR 0112 item 6: AddToReleases must write the usual group."
    );
    assert_eq!(
        make("C", "Field Recordist", "Sound"),
        ("Field Recordist".to_owned(), "Sound".to_owned()),
        "ADR 0112 item 6: a usual role that is not of the list must keep its text and its group."
    );
}

/// A feed with a person credit, and one payee for each case of ADR 0111.
const CORRECT_FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Album</title>
    <podcast:medium>music</podcast:medium>
    <podcast:person role=\"vocalist\" href=\"https://old.example\" npub=\"npub1city\">Ann</podcast:person>
    <podcast:person>Ann</podcast:person>
    <podcast:value type=\"lightning\" method=\"keysend\">
      <podcast:valueRecipient name=\"Ann\" type=\"node\" address=\"second\" split=\"30\" fee=\"true\"/>
      <podcast:valueRecipient name=\"Ann\" type=\"node\" address=\"first\" split=\"20\"/>
      <podcast:valueRecipient name=\"ann\" type=\"node\" address=\"stranger\" split=\"50\"/>
    </podcast:value>
  </channel>
</rss>
";

fn correct_feed() -> studio_core::feed::Feed {
    let doc = studio_core::xml::Document::parse(CORRECT_FEED).unwrap();
    studio_core::feed::Feed::read(&doc).unwrap()
}

fn correct_person(s: &Setup) -> people::PersonId {
    let id = people::add_person(
        &s.collection,
        &PersonFields {
            name: "Ann".into(),
            usual_role: "Vocalist".into(),
            img: "https://img.example/ann.png".into(),
            href: "https://ann.example".into(),
            ..PersonFields::default()
        },
    )
    .unwrap();
    for address in ["first", "second"] {
        people::add_wallet(
            &s.collection,
            id,
            &plain(address),
            "node",
            people::LinkEvidence::User,
        )
        .unwrap();
    }
    people::add_key(&s.collection, id, "npub1ann").unwrap();
    id
}

fn correct(
    s: &Setup,
    id: people::PersonId,
    operation: &PersonOperation,
) -> (Wallet, RecipientOperation) {
    person_operation(&s.collection, id, operation, &no_lookups()).unwrap()
}

#[test]
fn correct_applies_only_the_selected_changes_and_keeps_a_role() {
    let s = setup();
    let id = correct_person(&s);
    let (wallet, op) = correct(
        &s,
        id,
        &PersonOperation::Correct {
            key: Some("npub1ann".into()),
            payment: false,
            details: false,
            role: false,
            other_payees: false,
        },
    );
    let mut feed = correct_feed();
    assert!(change_set::apply(&mut feed, &wallet, &op).is_changed());
    assert_eq!(feed.persons[0].value.npub.as_deref(), Some("npub1ann"));
    assert_eq!(feed.persons[1].value.npub.as_deref(), Some("npub1ann"));
    assert_eq!(
        feed.persons[0].value.href.as_deref(),
        Some("https://old.example"),
        "ADR 0111 item 15: Correct must change only the selected parts."
    );
    assert_eq!(feed.persons[1].value.role, None);
    assert_eq!(
        feed.values[0].value.recipients[2].value.address.as_deref(),
        Some("stranger")
    );
}

#[test]
fn correct_gives_a_credit_with_no_role_the_usual_role_and_keeps_the_other_roles() {
    let s = setup();
    let id = correct_person(&s);
    let (wallet, op) = correct(
        &s,
        id,
        &PersonOperation::Correct {
            key: None,
            payment: false,
            details: true,
            role: true,
            other_payees: false,
        },
    );
    let mut feed = correct_feed();
    assert!(change_set::apply(&mut feed, &wallet, &op).is_changed());
    assert_eq!(
        feed.persons[0].value.role.as_deref(),
        Some("vocalist"),
        "ADR 0111 item 17: Correct must keep the role of a credit that has one."
    );
    assert_eq!(feed.persons[1].value.role.as_deref(), Some("vocalist"));
    assert_eq!(feed.persons[1].value.group.as_deref(), Some("music"));
    assert_eq!(
        feed.persons[0].value.href.as_deref(),
        Some("https://ann.example")
    );
    assert_eq!(
        feed.persons[1].value.img.as_deref(),
        Some("https://img.example/ann.png")
    );
}

#[test]
fn correct_with_other_payees_keeps_the_split_and_the_fee() {
    let s = setup();
    let id = correct_person(&s);
    let (wallet, op) = correct(
        &s,
        id,
        &PersonOperation::Correct {
            key: None,
            payment: true,
            details: false,
            role: false,
            other_payees: true,
        },
    );
    let mut feed = correct_feed();
    assert!(change_set::apply(&mut feed, &wallet, &op).is_changed());
    let list: Vec<(String, String, bool)> = feed.values[0]
        .value
        .recipients
        .iter()
        .map(|e| {
            (
                e.value.address.clone().unwrap(),
                e.value.split.clone().unwrap(),
                e.value.fee.is_some(),
            )
        })
        .collect();
    // The fee recipient stays apart. The other two merge: 20 + 50.
    assert_eq!(
        list,
        [
            ("first".to_owned(), "30".to_owned(), true),
            ("first".to_owned(), "70".to_owned(), false)
        ],
        "ADR 0111 item 15: Correct must keep the split and the fee of each payee."
    );
}

#[test]
fn correct_gives_not_changed_for_a_feed_with_no_change_and_refuses_a_wrong_key() {
    let s = setup();
    let id = correct_person(&s);
    let (wallet, op) = correct(
        &s,
        id,
        &PersonOperation::Correct {
            key: None,
            payment: true,
            details: true,
            role: true,
            other_payees: true,
        },
    );
    let mut feed =
        studio_core::feed::Feed::read(&studio_core::xml::Document::parse(FEED).unwrap()).unwrap();
    let outcome = change_set::apply(&mut feed, &wallet, &op);
    assert!(
        !outcome.is_changed(),
        "ADR 0111 item 16: a feed with no change must give Not changed."
    );
    let error = person_operation(
        &s.collection,
        id,
        &PersonOperation::Correct {
            key: Some("npub1other".into()),
            payment: false,
            details: false,
            role: false,
            other_payees: false,
        },
        &no_lookups(),
    )
    .unwrap_err();
    assert!(format!("{error}").contains("not a key of the person"));
    let (_, nothing) = correct(
        &s,
        id,
        &PersonOperation::Correct {
            key: None,
            payment: false,
            details: false,
            role: false,
            other_payees: false,
        },
    );
    assert!(change_set::check_operation(&nothing).is_err());
}

#[test]
fn correct_writes_through_a_change_set() {
    let s = setup_with(CORRECT_FEED);
    let id = correct_person(&s);
    let mut starter = start(&s);
    let set = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &PersonOperation::Correct {
            key: Some("npub1ann".into()),
            payment: true,
            details: false,
            role: false,
            other_payees: false,
        },
        &s.feeds,
    )
    .unwrap();
    assert!(
        set.feeds.iter().all(|f| f.outcome.is_changed()),
        "ADR 0111 item 16: Correct must make a change set with a diff for each feed."
    );
    assert_eq!(set.person, Some(id));
}

/// A publisher feed with no credit.
const PUBLISHER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Band</title>
    <podcast:guid>5a3b1c2d-0000-5000-8000-000000000081</podcast:guid>
    <podcast:medium>publisher</podcast:medium>
    <item>
      <title>Release</title>
      <guid>release-1</guid>
    </item>
  </channel>
</rss>
";

/// A publisher feed with the credit of `Ann` in its channel.
const PUBLISHER_WITH_CREDIT: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Band</title>
    <podcast:guid>5a3b1c2d-0000-5000-8000-000000000081</podcast:guid>
    <podcast:medium>publisher</podcast:medium>
    <podcast:person role=\"vocalist\" group=\"music\" npub=\"npub1ann\">Ann</podcast:person>
    <item>
      <title>Release</title>
      <guid>release-1</guid>
    </item>
  </channel>
</rss>
";

/// A person named `Ann` with one Nostr key and no wallet.
fn ann(s: &Setup) -> people::PersonId {
    ann_with_key(s, "npub1ann")
}

/// A person named `Ann` with the Nostr key `npub`, and no wallet.
fn ann_with_key(s: &Setup, npub: &str) -> people::PersonId {
    let id = people::add_person(
        &s.collection,
        &PersonFields {
            name: "Ann".into(),
            usual_role: "Vocalist".into(),
            img: "https://img.example/ann.png".into(),
            ..PersonFields::default()
        },
    )
    .unwrap();
    people::add_key(&s.collection, id, npub).unwrap();
    id
}

fn credit_feed(role: &str, group: &str) -> PersonOperation {
    PersonOperation::CreditFeed {
        role: role.into(),
        group: group.into(),
    }
}

/// The text that `after` has and `before` has not, when `after` is `before`
/// with one inserted text. Gives `None` for any other change.
fn inserted_text<'a>(before: &str, after: &'a str) -> Option<&'a str> {
    let prefix = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let longest = before.len().min(after.len()) - prefix;
    let suffix = before
        .bytes()
        .rev()
        .zip(after.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(longest);
    let removed = before.len() - prefix - suffix;
    (removed == 0).then(|| &after[prefix..after.len() - suffix])
}

#[test]
fn credit_feed_adds_one_person_with_the_role_and_its_group() {
    let s = setup_with(PUBLISHER);
    let id = ann(&s);
    let mut starter = start(&s);
    let set = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &credit_feed("Vocalist", ""),
        &s.feeds[..1],
    )
    .unwrap();
    assert!(set.feeds[0].outcome.is_changed());
    let (feed, _) = pipeline::draft_model(&s.collection, s.feeds[0]).unwrap();
    assert_eq!(feed.persons.len(), 1);
    let credit = &feed.persons[0].value;
    assert_eq!(credit.name, "Ann");
    assert_eq!(credit.role.as_deref(), Some("vocalist"));
    assert_eq!(credit.group.as_deref(), Some("music"));
    assert_eq!(credit.npub.as_deref(), Some("npub1ann"));
    assert_eq!(credit.img.as_deref(), Some("https://img.example/ann.png"));
}

#[test]
fn credit_feed_keeps_the_other_bytes_of_the_feed() {
    let s = setup_with(PUBLISHER);
    let before = fs::read_to_string(s.dir.path().join("one.xml")).unwrap();
    let id = ann(&s);
    let mut starter = start(&s);
    make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &credit_feed("Vocalist", "Music"),
        &s.feeds[..1],
    )
    .unwrap();
    let draft = s.collection.draft(s.feeds[0]).unwrap().unwrap();
    let after = String::from_utf8(draft.draft_xml).unwrap();
    let inserted = inserted_text(&before, &after)
        .expect("ADR 0118 item 5: the credit must be the only change to the feed.");
    assert!(inserted.contains("role=\"vocalist\""), "{inserted}");
    assert!(inserted.contains("group=\"music\""), "{inserted}");
}

#[test]
fn remove_credit_feed_removes_the_credit_of_the_person() {
    let s = setup_with(PUBLISHER_WITH_CREDIT);
    let id = ann(&s);
    let mut starter = start(&s);
    let remove = PersonOperation::Remove {
        payee: false,
        credit: true,
    };
    let set =
        make_person_change_set(&s.collection, &mut starter, id, &remove, &s.feeds[..1]).unwrap();
    assert!(set.feeds[0].outcome.is_changed());
    let (feed, _) = pipeline::draft_model(&s.collection, s.feeds[0]).unwrap();
    assert!(
        feed.persons.is_empty(),
        "ADR 0118 item 6: the credit must go."
    );
}

#[test]
fn credit_feed_refuses_two_feeds_or_an_album_and_makes_no_draft() {
    let publishers = setup_each(&[PUBLISHER, PUBLISHER]);
    let id = ann(&publishers);
    let mut starter = start(&publishers);
    assert!(
        make_person_change_set(
            &publishers.collection,
            &mut starter,
            id,
            &credit_feed("Vocalist", ""),
            &publishers.feeds,
        )
        .is_err(),
        "ADR 0118 item 5: a channel credit needs one feed."
    );
    let album = setup();
    let id = ann(&album);
    let mut starter = start(&album);
    assert!(
        make_person_change_set(
            &album.collection,
            &mut starter,
            id,
            &credit_feed("Vocalist", ""),
            &album.feeds[..1],
        )
        .is_err(),
        "ADR 0118 item 5: a channel credit needs a publisher feed."
    );
    for feed in &publishers.feeds {
        assert!(publishers.collection.draft(*feed).unwrap().is_none());
    }
    assert!(album.collection.draft(album.feeds[0]).unwrap().is_none());
    assert_eq!(publishers.collection.change_set_rows().unwrap().len(), 0);
    assert_eq!(album.collection.change_set_rows().unwrap().len(), 0);
}

/// A person named `Bea` with the credit name `Ann` of another person.
fn other_person_with_ann_credit(s: &Setup) -> people::PersonId {
    let bea = people::add_person(
        &s.collection,
        &PersonFields {
            name: "Bea".into(),
            ..PersonFields::default()
        },
    )
    .unwrap();
    people::add_credit_name(&s.collection, bea, "Ann", "", "npub1ann").unwrap();
    bea
}

#[test]
fn credit_feed_links_the_credit_name_and_a_remove_after_its_write_removes_the_credit() {
    let mut s = setup_with(PUBLISHER);
    // The write checks each Nostr key, so the key must be a valid npub.
    let id = ann_with_key(
        &s,
        "npub19573mxeqcq5k6xt7hcp2e4wgd9wakh7mvz7l5zq7l3f3t4glnz6su85g79",
    );
    let mut starter = start(&s);
    make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &credit_feed("Vocalist", ""),
        &s.feeds[..1],
    )
    .unwrap();
    let person = people::person(&s.collection, id).unwrap();
    assert_eq!(
        person.credit_names.len(),
        1,
        "ADR 0118 item 11: the credit name must be linked to the person."
    );
    drop(starter);
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feeds[0]).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        &ScriptedReader::empty(),
        s.feeds[0],
        &diff.id,
    )
    .unwrap();
    let mut starter = start(&s);
    let remove = PersonOperation::Remove {
        payee: false,
        credit: true,
    };
    let set =
        make_person_change_set(&s.collection, &mut starter, id, &remove, &s.feeds[..1]).unwrap();
    assert!(set.feeds[0].outcome.is_changed());
    let (feed, _) = pipeline::draft_model(&s.collection, s.feeds[0]).unwrap();
    assert!(
        feed.persons.is_empty(),
        "ADR 0118 item 11: the remove must find the credit that the change set added."
    );
}

#[test]
fn credit_feed_refuses_a_credit_name_of_another_person_and_makes_no_draft() {
    let s = setup_with(PUBLISHER);
    let id = ann(&s);
    other_person_with_ann_credit(&s);
    let mut starter = start(&s);
    assert!(
        make_person_change_set(
            &s.collection,
            &mut starter,
            id,
            &credit_feed("Vocalist", ""),
            &s.feeds[..1],
        )
        .is_err(),
        "ADR 0118 item 11: a credit name of another person must stop the change set."
    );
    assert!(s.collection.draft(s.feeds[0]).unwrap().is_none());
    assert_eq!(s.collection.change_set_rows().unwrap().len(), 0);
}

#[test]
fn credit_feed_keeps_its_own_operation_kind_in_the_change_set() {
    let s = setup_with(PUBLISHER);
    let id = ann(&s);
    let mut starter = start(&s);
    let set = make_person_change_set(
        &s.collection,
        &mut starter,
        id,
        &credit_feed("Vocalist", ""),
        &s.feeds[..1],
    )
    .unwrap();
    let row = s.collection.change_set_row(set.id).unwrap().unwrap();
    let operation: serde_json::Value = serde_json::from_str(&row.operation).unwrap();
    assert_eq!(
        operation["kind"], "credit_feed",
        "ADR 0118 item 12: the stored operation must keep its own kind."
    );
}
