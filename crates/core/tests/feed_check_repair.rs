//! The repairs of the feed check (ADR 0174 items 6 and 10 to 15). No test
//! sends a request to the network (ADR 0045). Scripted readers serve the
//! reads.

use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{Connector, Expected, LocalFolder, Version};
use studio_core::feed_check::{Finding, FindingKind, FindingLevel, RepairKind};
use studio_core::feed_reads::{CheckResult, NetworkFacts, ScriptedFileReads, check_with};
use studio_core::inventory::Hosts;
use studio_core::move_link::ScriptedReader;
use studio_core::pipeline::{self, PipelineErrorKind, Proof};

const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
const TARGET_GUID: &str = "a17393e3-1b1e-5cef-ace4-edaa54e1f811";
const URL: &str = "https://a.example/album.xml";
const ONE: &str = "http://files.example/one.mp3";
const TWO: &str = "http://files.example/two.mp3";

fn feed_xml(link: Option<&str>, tag: Option<&str>) -> String {
    let link = link.map_or(String::new(), |l| {
        format!("    <atom:link href=\"{l}\" rel=\"self\" type=\"application/rss+xml\"/>\n")
    });
    let tag = tag.map_or(String::new(), |t| {
        format!("    <itunes:new-feed-url>{t}</itunes:new-feed-url>\n")
    });
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">
  <channel>
    <title>Album</title>
    <language>en</language>
{link}{tag}    <podcast:guid>{GUID}</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <item>
      <title>One</title>
      <guid>one</guid>
      <itunes:duration>180</itunes:duration>
      <enclosure url=\"{ONE}\" type=\"audio/mpeg\"/>
    </item>
    <item>
      <title>Two</title>
      <guid>two</guid>
      <itunes:duration>180</itunes:duration>
      <enclosure url=\"{TWO}\" length=\"10\" type=\"audio/mpeg\"/>
    </item>
  </channel>
</rss>
"
    )
}

struct Env {
    dir: tempfile::TempDir,
    collection: Collection,
    hosts: Hosts,
    feed: FeedId,
}

fn env(xml: &str) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://a.example".into()));
    folder
        .write("album.xml", xml.as_bytes(), &Expected::Absent)
        .unwrap();
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let a = collection
        .add_connector(LocalFolder::KIND, "A", &folder.config(), None)
        .unwrap();
    let feed = collection
        .add_feed(Some(URL), Some((a, "album.xml")))
        .unwrap();
    pipeline::refresh(&mut collection, &folder, feed).unwrap();
    pipeline::start_draft(&collection, &folder, feed).unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(a, Box::new(folder));
    Env {
        dir,
        collection,
        hosts,
        feed,
    }
}

impl Env {
    fn text(&self) -> String {
        String::from_utf8(self.collection.draft(self.feed).unwrap().unwrap().draft_xml).unwrap()
    }

    fn result(&self, net: &NetworkFacts) -> (CheckResult, String) {
        let inputs = pipeline::check_inputs(&self.collection, &self.hosts, self.feed).unwrap();
        (
            check_with(
                &inputs.links,
                Some(net),
                &inputs.link_targets,
                inputs.known_guid.clone(),
                inputs.read_guid.clone(),
            ),
            inputs.revision,
        )
    }

    fn finding(&self, net: &NetworkFacts, kind: FindingKind, entry: Option<&str>) -> Finding {
        self.result(net)
            .0
            .findings
            .into_iter()
            .find(|f| f.kind == kind && f.entry.as_deref() == entry)
            .unwrap_or_else(|| panic!("no finding {kind:?}"))
    }

    /// Read for the repair of `finding`, and apply it.
    fn repair(
        &self,
        net: &NetworkFacts,
        finding: &Finding,
        feeds: &ScriptedReader,
        files: &ScriptedFileReads,
    ) -> Result<(), studio_core::pipeline::PipelineError> {
        let inputs = pipeline::check_inputs(&self.collection, &self.hosts, self.feed).unwrap();
        let _ = net;
        let step = pipeline::repair_step(&inputs, finding)?;
        let proof = pipeline::run_step(&step, feeds, files)?;
        pipeline::apply_check_repair(
            &self.collection,
            &self.hosts,
            self.feed,
            finding,
            &inputs.revision,
            &proof,
        )
        .map(|_| ())
    }
}

fn body(guid: &str) -> ScriptedReader {
    ScriptedReader::empty().with_feed(URL, URL, Some(guid), None, None)
}

fn none() -> NetworkFacts {
    NetworkFacts::default()
}

#[test]
fn add_the_link_with_a_body_that_has_another_guid_changes_no_draft_and_gives_the_reason() {
    let e = env(&feed_xml(None, None));
    let before = e.text();
    let finding = e.finding(&none(), FindingKind::NoSelfLink, None);
    assert_eq!(finding.repair, Some(RepairKind::AddTheLink));
    let error = e
        .repair(
            &none(),
            &finding,
            &body("another"),
            &ScriptedFileReads::empty(),
        )
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
    assert!(error.to_string().contains("podcast:guid"), "{error}");
    // An address that Studio cannot read changes nothing either.
    let error = e
        .repair(
            &none(),
            &finding,
            &ScriptedReader::empty(),
            &ScriptedFileReads::empty(),
        )
        .unwrap_err();
    assert!(error.to_string().contains("could not read"), "{error}");
    assert_eq!(e.text(), before);
}

fn guid_change_env() -> (Env, Finding, String) {
    let mut e = env(&feed_xml(Some(URL), None));
    e.collection.set_known_guid(e.feed, GUID).unwrap();
    let mut hosted = studio_core::feed::Feed::read(
        &studio_core::xml::Document::parse_bytes(
            feed_xml(Some(URL), None)
                .replace(GUID, "new-guid")
                .as_bytes(),
        )
        .unwrap(),
    )
    .unwrap();
    hosted.podcast_guid = Some("new-guid".to_owned());
    e.collection
        .record_state(e.feed, &hosted, &Version::new("outside-read"))
        .unwrap();
    let mut draft = e.collection.draft(e.feed).unwrap().unwrap();
    draft.draft_xml = feed_xml(Some(URL), None)
        .replace(GUID, "new-guid")
        .into_bytes();
    e.collection.save_draft(e.feed, &draft).unwrap();
    let inputs = pipeline::check_inputs(&e.collection, &e.hosts, e.feed).unwrap();
    let result = studio_core::feed_reads::check_with(
        &inputs.links,
        None,
        &inputs.link_targets,
        inputs.known_guid.clone(),
        inputs.read_guid.clone(),
    );
    let finding = result
        .findings
        .into_iter()
        .find(|finding| finding.kind == FindingKind::GuidChangedOutside)
        .unwrap();
    (e, finding, inputs.revision)
}

#[test]
fn put_back_guid_restores_the_known_guid_and_rejects_a_second_repair() {
    let (e, finding, revision) = guid_change_env();
    let inputs = pipeline::check_inputs(&e.collection, &e.hosts, e.feed).unwrap();
    let step = pipeline::repair_step(&inputs, &finding).unwrap();
    pipeline::apply_check_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        &finding,
        &revision,
        &Proof::Nothing,
    )
    .unwrap();
    assert!(
        e.text()
            .contains(&format!("<podcast:guid>{GUID}</podcast:guid>")),
        "ADR 0249 item 6: fix Plan::apply for PutBackGuid"
    );
    assert_eq!(
        pipeline::guid_change(&e.collection, e.feed).unwrap(),
        None,
        "ADR 0249 item 6: fix guid_change after PutBackGuid"
    );
    assert_eq!(step, pipeline::Step::Nothing);
    let fresh = pipeline::check_inputs(&e.collection, &e.hosts, e.feed).unwrap();
    let error = pipeline::apply_check_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        &finding,
        &fresh.revision,
        &Proof::Nothing,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("repair does not apply"),
        "ADR 0249 item 6: fix apply_check_repair second PutBack"
    );
}

#[test]
fn keep_new_guid_changes_the_known_guid_and_leaves_the_draft_unchanged() {
    let (e, mut finding, revision) = guid_change_env();
    let before = e.text();
    finding.repair = Some(RepairKind::KeepNewGuid);
    pipeline::apply_check_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        &finding,
        &revision,
        &Proof::Nothing,
    )
    .unwrap();
    assert_eq!(
        e.text(),
        before,
        "ADR 0249 item 6: fix apply_check_repair for KeepNewGuid"
    );
    assert_eq!(
        e.collection.feed(e.feed).unwrap().known_guid.as_deref(),
        Some("new-guid"),
        "ADR 0249 item 6: fix Collection::set_known_guid call"
    );
    assert_eq!(
        pipeline::guid_change(&e.collection, e.feed).unwrap(),
        None,
        "ADR 0249 item 6: fix guid_change after KeepNewGuid"
    );
}

#[test]
fn address_type_repair_changes_matching_feed_and_track_recipients_only() {
    let xml = feed_xml(Some(URL), None)
        .replace("<channel>", "<channel><podcast:value type=\"lightning\" method=\"keysend\"><podcast:valueRecipient name=\"Feed payee\" type=\"node\" address=\"alice@example.com\" split=\"70\"/></podcast:value>")
        .replace("<title>One</title>", "<title>One</title><podcast:value type=\"lightning\" method=\"keysend\"><podcast:valueRecipient name=\"Track payee\" type=\"node\" address=\"alice@example.com\" split=\"30\"/></podcast:value>");
    let e = env(&xml);
    let (result, revision) = e.result(&none());
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|f| f.kind == FindingKind::WrongAddressType)
            .count(),
        1
    );
    let finding = result
        .findings
        .into_iter()
        .find(|f| f.kind == FindingKind::WrongAddressType)
        .unwrap();
    assert_eq!(finding.repair, Some(RepairKind::UseLnurlAddress));
    let inputs = pipeline::check_inputs(&e.collection, &e.hosts, e.feed).unwrap();
    let step = pipeline::repair_step(&inputs, &finding).unwrap();
    let proof =
        pipeline::run_step(&step, &ScriptedReader::empty(), &ScriptedFileReads::empty()).unwrap();
    pipeline::apply_check_repair(&e.collection, &e.hosts, e.feed, &finding, &revision, &proof)
        .unwrap();
    let text = e.text();
    assert!(text.contains(
        "name=\"Feed payee\" type=\"lnaddress\" address=\"alice@example.com\" split=\"70\""
    ));
    assert!(text.contains(
        "name=\"Track payee\" type=\"lnaddress\" address=\"alice@example.com\" split=\"30\""
    ));
}

#[test]
fn add_the_link_with_the_right_body_adds_the_link_with_the_exact_address() {
    let e = env(&feed_xml(None, None));
    let before = e.text();
    let finding = e.finding(&none(), FindingKind::NoSelfLink, None);
    e.repair(&none(), &finding, &body(GUID), &ScriptedFileReads::empty())
        .unwrap();
    let after = e.text();
    assert_ne!(before, after);
    assert!(after.contains(&format!("href=\"{URL}\"")), "{after}");
    assert!(after.contains("rel=\"self\""), "{after}");
    assert!(after.contains(ONE) && after.contains(TWO));
    // The finding is gone.
    let found = e.result(&none()).0.findings;
    assert!(found.iter().all(|f| f.kind != FindingKind::NoSelfLink));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "ADR 0247 item 8: the repair test checks both link locations and discard behavior"
)]
fn use_the_guid_of_repairs_channel_and_podroll_links_only() {
    let main = feed_xml(Some(URL), None).replace(
        "</channel>",
        "<podcast:remoteItem feedGuid=\"old-guid\" feedUrl=\"https://a.example/release.xml\" title=\"This Is It\" medium=\"music\" rel=\"artist\"/><podcast:podroll><podcast:remoteItem feedGuid=\"old-guid\" feedUrl=\"https://a.example/release.xml\" title=\"This Is It\" medium=\"music\" rel=\"artist\"/></podcast:podroll></channel>",
    );
    let mut e = env(&main);
    let folder = LocalFolder::new(e.dir.path(), Some("https://a.example".into()));
    folder
        .write(
            "release.xml",
            feed_xml(Some("https://a.example/release.xml"), None)
                .replace(GUID, TARGET_GUID)
                .replace("<title>Album</title>", "<title>This Is It</title>")
                .as_bytes(),
            &Expected::Absent,
        )
        .unwrap();
    let connector = e.collection.feed(e.feed).unwrap().connector_id.unwrap();
    let target = e
        .collection
        .add_feed(
            Some("https://a.example/release.xml"),
            Some((connector, "release.xml")),
        )
        .unwrap();
    pipeline::refresh(&mut e.collection, &folder, target).unwrap();
    pipeline::start_draft(&e.collection, &folder, target).unwrap();
    let target_before = e.collection.draft(target).unwrap().unwrap();
    let before = studio_core::feed::Feed::read(
        &studio_core::xml::Document::parse_bytes(e.text().as_bytes()).unwrap(),
    )
    .unwrap();
    let finding = e.finding(&none(), FindingKind::LinkGuidOld, None);
    assert_eq!(
        finding.title, "A link to This Is It has an old GUID",
        "ADR 0247 item 8: fix check_old_links"
    );
    e.repair(
        &none(),
        &finding,
        &ScriptedReader::empty(),
        &ScriptedFileReads::empty(),
    )
    .unwrap();
    let after = studio_core::feed::Feed::read(
        &studio_core::xml::Document::parse_bytes(e.text().as_bytes()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        after.remote_items[0].value.feed_guid.as_deref(),
        Some(TARGET_GUID),
        "ADR 0247 item 8: fix Plan::apply"
    );
    assert_eq!(
        after.remote_items[0].value.feed_url, before.remote_items[0].value.feed_url,
        "ADR 0247 item 8: fix correct_old_links attribute scope"
    );
    assert_eq!(
        after.remote_items[0].value.title, before.remote_items[0].value.title,
        "ADR 0247 item 8: fix correct_old_links attribute scope"
    );
    assert_eq!(
        after.remote_items[0].value.medium, before.remote_items[0].value.medium,
        "ADR 0247 item 8: fix correct_old_links attribute scope"
    );
    assert_eq!(
        after.remote_items[0].value.rel, before.remote_items[0].value.rel,
        "ADR 0247 item 8: fix correct_old_links attribute scope"
    );
    let before_podroll = &before.podroll.as_ref().unwrap().remote_items[0].value;
    let after_podroll = &after.podroll.as_ref().unwrap().remote_items[0].value;
    assert_eq!(
        after_podroll.feed_guid.as_deref(),
        Some(TARGET_GUID),
        "ADR 0247 item 8: fix Plan::apply for podroll"
    );
    assert_eq!(
        (
            &after_podroll.feed_url,
            &after_podroll.title,
            &after_podroll.medium,
            &after_podroll.rel
        ),
        (
            &before_podroll.feed_url,
            &before_podroll.title,
            &before_podroll.medium,
            &before_podroll.rel
        ),
        "ADR 0247 item 8: fix correct_old_links attribute scope"
    );
    let error = e
        .repair(
            &none(),
            &finding,
            &ScriptedReader::empty(),
            &ScriptedFileReads::empty(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        PipelineErrorKind::InvalidOperation,
        "ADR 0247 item 8: fix plan_of repeat repair handling"
    );
    assert!(
        error
            .to_string()
            .contains("this repair does not apply to the draft"),
        "ADR 0247 item 8: fix plan_of repeat repair handling: {error}"
    );
    assert_eq!(
        e.collection.draft(target).unwrap().unwrap(),
        target_before,
        "ADR 0247 item 8: apply_check_repair changed another draft"
    );
}

#[test]
fn use_the_public_address_for_the_tag_spelling_keeps_the_tag_and_sets_its_text() {
    let spelled = "https://A.example:443/album.xml";
    let e = env(&feed_xml(Some(URL), Some(spelled)));
    let finding = e.finding(&none(), FindingKind::SelfLinkSpelling, None);
    assert_eq!(finding.repair, Some(RepairKind::UseThePublicAddress));
    e.repair(&none(), &finding, &body(GUID), &ScriptedFileReads::empty())
        .unwrap();
    let after = e.text();
    assert!(
        after.contains(&format!("<itunes:new-feed-url>{URL}</itunes:new-feed-url>")),
        "{after}"
    );
    assert!(!after.contains(spelled));
    assert_eq!(after.matches(URL).count(), 2, "{after}");
}

#[test]
fn use_the_public_address_for_a_different_self_link_does_not_touch_the_tag() {
    let tag = "https://B.example/album.xml";
    let e = env(&feed_xml(Some("https://old.example/album.xml"), Some(tag)));
    let finding = e.finding(&none(), FindingKind::SelfLinkDiffers, None);
    e.repair(&none(), &finding, &body(GUID), &ScriptedFileReads::empty())
        .unwrap();
    let after = e.text();
    assert!(after.contains(&format!("href=\"{URL}\"")), "{after}");
    assert!(after.contains(tag), "the tag stays: {after}");
}

#[test]
fn use_the_https_address_changes_only_the_named_url() {
    let e = env(&feed_xml(Some(URL), None));
    let mut net = none();
    net.same_https.insert(ONE.to_owned());
    net.same_https.insert(TWO.to_owned());
    let before = e.text();
    let finding = e.finding(&net, FindingKind::HttpFile, Some("one"));
    assert_eq!(finding.url.as_deref(), Some(ONE));
    e.repair(
        &net,
        &finding,
        &ScriptedReader::empty(),
        &ScriptedFileReads::empty(),
    )
    .unwrap();
    let after = e.text();
    assert_eq!(
        after,
        before.replace(ONE, "https://files.example/one.mp3"),
        "only the url of the track One changes"
    );
}

#[test]
fn read_the_size_sets_the_length_of_the_enclosure() {
    let e = env(&feed_xml(Some(URL), None));
    let before = e.text();
    let finding = e.finding(&none(), FindingKind::NoFileSize, Some("one"));
    assert_eq!(finding.repair, Some(RepairKind::ReadTheSize));
    // A host that gives no size changes nothing.
    let error = e
        .repair(
            &none(),
            &finding,
            &ScriptedReader::empty(),
            &ScriptedFileReads::empty(),
        )
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
    assert_eq!(e.text(), before);
    e.repair(
        &none(),
        &finding,
        &ScriptedReader::empty(),
        &ScriptedFileReads::empty().with_size(ONE, 4321),
    )
    .unwrap();
    assert_eq!(
        e.text(),
        before.replace(
            &format!("<enclosure url=\"{ONE}\" type=\"audio/mpeg\"/>"),
            &format!("<enclosure url=\"{ONE}\" type=\"audio/mpeg\" length=\"4321\"/>")
        )
    );
}

#[test]
fn a_stale_revision_and_a_wrong_proof_change_nothing() {
    let e = env(&feed_xml(None, None));
    let before = e.text();
    let finding = e.finding(&none(), FindingKind::NoSelfLink, None);
    let error = pipeline::apply_check_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        &finding,
        "stale",
        &Proof::Address(URL.to_owned()),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::StaleRevision);
    let inputs = pipeline::check_inputs(&e.collection, &e.hosts, e.feed).unwrap();
    for proof in [
        Proof::Nothing,
        Proof::Address("https://other.example/x.xml".to_owned()),
    ] {
        let error = pipeline::apply_check_repair(
            &e.collection,
            &e.hosts,
            e.feed,
            &finding,
            &inputs.revision,
            &proof,
        )
        .unwrap_err();
        assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
    }
    assert_eq!(e.text(), before);
}

#[test]
fn add_https_completes_one_short_credit_address_and_leaves_the_other_credit() {
    let owner = "ADR 0245 items 1 and 2: fix check_short_addresses or the AddHttps plan";
    let xml = feed_xml(Some(URL), None).replace(
        "<podcast:medium>",
        "<podcast:person href=\"www.doerfelverse.com\">The Doerfels</podcast:person><podcast:person href=\"https://other.example/a\">Other</podcast:person><podcast:medium>",
    );
    let e = env(&xml);
    let found: Vec<Finding> = e
        .result(&none())
        .0
        .findings
        .into_iter()
        .filter(|f| f.kind == FindingKind::ShortAddress)
        .collect();
    assert_eq!(found.len(), 1, "{owner}");
    let finding = &found[0];
    assert_eq!(finding.level, FindingLevel::Stop, "{owner}");
    assert_eq!(finding.repair, Some(RepairKind::AddHttps), "{owner}");
    assert_eq!(
        finding.place.as_deref(),
        Some("Credit \u{b7} The Doerfels \u{b7} Information address"),
        "{owner}"
    );
    e.repair(
        &none(),
        finding,
        &ScriptedReader::empty(),
        &ScriptedFileReads::empty(),
    )
    .unwrap();
    let text = e.text();
    assert!(
        text.contains("href=\"https://www.doerfelverse.com\""),
        "{owner}: {text}"
    );
    assert!(text.contains("href=\"https://other.example/a\""), "{owner}");
    let draft = e.collection.draft(e.feed).unwrap().unwrap();
    let feed = studio_core::feed::Feed::read(
        &studio_core::xml::Document::parse_bytes(&draft.draft_xml).unwrap(),
    )
    .unwrap();
    assert!(feed.bad_addresses().is_empty(), "{owner}");
    // The finding goes, and a second repair of it is refused.
    let error = e
        .repair(
            &none(),
            finding,
            &ScriptedReader::empty(),
            &ScriptedFileReads::empty(),
        )
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation, "{owner}");
}
