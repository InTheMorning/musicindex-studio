//! The repairs of the feed check (ADR 0174 items 6 and 10 to 15). No test
//! sends a request to the network (ADR 0045). Scripted readers serve the
//! reads.

use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{Connector, Expected, LocalFolder};
use studio_core::feed_check::{Finding, FindingKind, RepairKind};
use studio_core::feed_reads::{CheckResult, NetworkFacts, ScriptedFileReads, check_with};
use studio_core::inventory::Hosts;
use studio_core::move_link::ScriptedReader;
use studio_core::pipeline::{self, PipelineErrorKind, Proof};

const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
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
    _dir: tempfile::TempDir,
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
        _dir: dir,
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
        (check_with(&inputs.links, Some(net)), inputs.revision)
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
