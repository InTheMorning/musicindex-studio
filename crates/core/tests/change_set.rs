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
