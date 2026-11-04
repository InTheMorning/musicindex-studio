//! Stage 1 of ADR 0126: the write refuses a bad link before any file changes.
//! No test sends a request to the network (ADR 0045). A scripted reader
//! serves the bodies.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use studio_core::collection::{Collection, ConnectorId, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::discover::FailKind;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::move_link::{
    self, Condition, DraftLinks, FeedReader, Hop, HopResult, LinkNotice, ScriptedReader,
    WebFeedReader,
};
use studio_core::pipeline::{self, AddressCheck, PipelineError, PipelineErrorKind};

const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
const URL: &str = "https://a.example/album.xml";
const DEST: &str = "https://b.example/album.xml";

fn feed_xml(self_url: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">
  <channel>
    <title>Album</title>
    <atom:link href=\"{self_url}\" rel=\"self\" type=\"application/rss+xml\"/>
    <podcast:guid>{GUID}</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <item>
      <title>Track One</title>
      <guid>track-1</guid>
    </item>
  </channel>
</rss>
"
    )
}

struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

struct Env {
    dir_a: tempfile::TempDir,
    dir_b: tempfile::TempDir,
    collection: Collection,
    hosts: Hosts,
    library: Library,
    feed: FeedId,
    b: ConnectorId,
    original: String,
}

/// A feed on host A, with a draft. The self link of the file is
/// `self_url`, and the feed has the public address `URL`.
fn env(self_url: &str) -> Env {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let folder_a = LocalFolder::new(dir_a.path(), Some("https://a.example".into()));
    let folder_b = LocalFolder::new(dir_b.path(), Some("https://b.example".into()));
    let original = feed_xml(self_url);
    folder_a
        .write("album.xml", original.as_bytes(), &Expected::Absent)
        .unwrap();
    let mut collection = Collection::open(&dir_a.path().join("collection.sqlite")).unwrap();
    let a = collection
        .add_connector(LocalFolder::KIND, "A", &folder_a.config(), None)
        .unwrap();
    let b = collection
        .add_connector(LocalFolder::KIND, "B", &folder_b.config(), None)
        .unwrap();
    let reader = connector::from_row(
        &collection.connector(a).unwrap(),
        None,
        &studio_core::net::NetPolicy::Open,
    )
    .unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(a, Box::new(folder_a));
    hosts.insert(b, Box::new(folder_b));
    let library = Library::open(dir_a.path().join("library")).unwrap();
    let feed = collection
        .add_feed(Some(URL), Some((a, "album.xml")))
        .unwrap();
    pipeline::refresh(&mut collection, reader.as_ref(), feed).unwrap();
    collection.set_feed_folder(feed, "album").unwrap();
    library
        .write_feed_xml("album", "album.xml", original.as_bytes())
        .unwrap();
    pipeline::start_draft(&collection, reader.as_ref(), feed).unwrap();
    Env {
        dir_a,
        dir_b,
        collection,
        hosts,
        library,
        feed,
        b,
        original,
    }
}

/// The files of a host folder, without the files of Studio.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            !entry
                .file_name()
                .to_string_lossy()
                .starts_with("collection.sqlite")
        })
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}

impl Env {
    fn edit(&self, change: impl FnOnce(&mut studio_core::feed::Feed)) {
        pipeline::edit_draft(&self.collection, self.feed, change).unwrap();
    }

    fn write(&mut self, reader: &dyn FeedReader) -> Result<(), PipelineError> {
        let diff = pipeline::diff(&self.collection, &self.hosts, &self.library, self.feed).unwrap();
        pipeline::write_draft(
            &mut self.collection,
            &self.hosts,
            &self.library,
            &PassAll,
            reader,
            self.feed,
            &diff.id,
        )
        .map(|_| ())
    }

    /// Write and expect the refusal `condition`. Gives its hops. The write
    /// changes no file on any host.
    fn refused(&mut self, reader: &dyn FeedReader, condition: Condition) -> Vec<Hop> {
        let before = (files(self.dir_a.path()), files(self.dir_b.path()));
        let error = self.write(reader).unwrap_err();
        assert_eq!(error.kind(), PipelineErrorKind::BadMoveLink, "{error}");
        let refusal = error.bad_move_link().expect("the error has the refusal");
        assert_eq!(refusal.condition, condition);
        assert_eq!(refusal.number, condition.number());
        let after = (files(self.dir_a.path()), files(self.dir_b.path()));
        assert_eq!(before, after, "a refusal of stage 1 changes no file");
        // The draft stays.
        assert!(self.collection.draft(self.feed).unwrap().is_some());
        // The check for Review gives the same refusal.
        let report =
            pipeline::check_links(&self.collection, &self.hosts, reader, self.feed).unwrap();
        assert_eq!(report.refusal.as_ref(), Some(refusal));
        refusal.hops.clone()
    }
}

fn hop(url: &str, result: HopResult) -> Hop {
    Hop {
        url: url.to_owned(),
        final_url: None,
        result,
    }
}

fn set_tag(env: &Env, tag: &str) {
    env.edit(|f| f.new_feed_url = Some(tag.to_owned()));
}

const B: &str = "https://b.example/f.xml";
const C: &str = "https://c.example/f.xml";
const D: &str = "https://d.example/f.xml";

#[test]
fn a_tag_that_names_the_own_url_passes_with_no_read() {
    let mut e = env(URL);
    set_tag(&e, URL);
    let reader = ScriptedReader::empty();
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), Vec::<String>::new());
}

#[test]
fn a_write_with_no_followed_url_and_no_move_makes_no_read_and_keeps_the_bytes() {
    let mut e = env(URL);
    e.edit(|f| f.title = Some("Album (Deluxe)".into()));
    let reader = ScriptedReader::empty();
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), Vec::<String>::new());
    let written = fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap();
    assert_eq!(
        written,
        e.original
            .replace("<title>Album</title>", "<title>Album (Deluxe)</title>")
    );
}

#[test]
fn a_loop_of_two_urls_fails() {
    // S3: the draft names B, and B names the draft.
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_feed(B, B, Some(GUID), Some(URL), Some(B));
    let hops = e.refused(&reader, Condition::Loop);
    assert_eq!(
        hops,
        [hop(B, HopResult::Feed), hop(URL, HopResult::Revisited)]
    );
}

#[test]
fn a_loop_of_three_urls_fails() {
    // S12: the draft names B, B names C, and C names the draft.
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty()
        .with_feed(B, B, Some(GUID), Some(C), Some(B))
        .with_feed(C, C, Some(GUID), Some(URL), Some(C));
    let hops = e.refused(&reader, Condition::Loop);
    assert_eq!(
        hops,
        [
            hop(B, HopResult::Feed),
            hop(C, HopResult::Feed),
            hop(URL, HopResult::Revisited)
        ]
    );
}

#[test]
fn a_loop_that_does_not_include_the_draft_fails() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty()
        .with_feed(B, B, Some(GUID), Some(C), Some(B))
        .with_feed(C, C, Some(GUID), Some(B), Some(C));
    let hops = e.refused(&reader, Condition::Loop);
    assert_eq!(hops.last(), Some(&hop(B, HopResult::Revisited)));
}

#[test]
fn a_redirect_to_a_url_of_the_chain_fails() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_feed(B, URL, Some(GUID), None, None);
    let hops = e.refused(&reader, Condition::Loop);
    assert_eq!(
        hops,
        [Hop {
            url: B.to_owned(),
            final_url: Some(URL.to_owned()),
            result: HopResult::Revisited,
        }]
    );
}

fn chain(count: usize) -> Vec<String> {
    (0..count)
        .map(|n| format!("https://chain.example/{n}.xml"))
        .collect()
}

/// A reader for a chain: each body names the next one, and the last body
/// names itself when `ends` is true.
fn chain_reader(urls: &[String], ends: bool) -> ScriptedReader {
    let mut reader = ScriptedReader::empty();
    for (n, url) in urls.iter().enumerate() {
        let next = urls.get(n + 1).map(String::as_str);
        let tag = match next {
            Some(next) => Some(next),
            None if ends => None,
            None => Some("https://chain.example/after.xml"),
        };
        reader = reader.with_feed(url, url, Some(GUID), tag, Some(url));
    }
    reader
}

#[test]
fn a_chain_of_more_than_10_hops_fails() {
    let mut e = env(URL);
    let urls = chain(12);
    set_tag(&e, &urls[0]);
    let reader = chain_reader(&urls, true);
    let hops = e.refused(&reader, Condition::Loop);
    // The write and the check of Review read 10 URLs each, and no more.
    assert_eq!(reader.reads().len(), 20);
    assert_eq!(hops.len(), 11);
    assert_eq!(hops[10], hop(&urls[10], HopResult::TooManyHops));
}

#[test]
fn a_chain_of_10_hops_with_an_end_passes() {
    let mut e = env(URL);
    let urls = chain(10);
    set_tag(&e, &urls[0]);
    let reader = chain_reader(&urls, true);
    e.write(&reader).unwrap();
    assert_eq!(reader.reads().len(), 10);
}

#[test]
fn chains_with_an_end_pass() {
    // S13: B names C, and C names C.
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty()
        .with_feed(B, B, Some(GUID), Some(C), Some(B))
        .with_feed(C, C, Some(GUID), Some(C), Some(C));
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), [B, C]);

    // S16: B names C, C names D, and D names D.
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty()
        .with_feed(B, B, Some(GUID), Some(C), None)
        .with_feed(C, C, Some(GUID), Some(D), None)
        .with_feed(D, D, Some(GUID), Some(D), Some(D));
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), [B, C, D]);

    // A body that names nothing is an end, also behind a 301.
    let mut e = env(URL);
    set_tag(&e, B);
    let final_url = "https://b.example/final.xml";
    let reader = ScriptedReader::empty().with_feed(B, final_url, Some(GUID), None, Some(final_url));
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), [B]);
}

#[test]
fn a_followed_url_that_gives_no_feed_or_fails_is_refused() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_failure(B, FailKind::NotAFeed);
    let hops = e.refused(&reader, Condition::ReadFails);
    assert_eq!(hops, [hop(B, HopResult::NotAFeed)]);

    // A URL that the reader does not serve fails with `Unreachable`.
    let hops = e.refused(&ScriptedReader::empty(), Condition::ReadFails);
    assert_eq!(hops, [hop(B, HopResult::Unreachable)]);

    let reader = ScriptedReader::empty().with_failure(B, FailKind::Gone);
    let hops = e.refused(&reader, Condition::ReadFails);
    assert_eq!(hops, [hop(B, HopResult::Gone)]);
}

#[test]
fn a_body_with_a_different_or_no_guid_is_refused() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_feed(B, B, Some("other-guid"), None, None);
    let hops = e.refused(&reader, Condition::GuidMismatch);
    assert_eq!(hops, [hop(B, HopResult::GuidMismatch)]);

    let reader = ScriptedReader::empty().with_feed(B, B, None, None, None);
    e.refused(&reader, Condition::GuidMismatch);
}

#[test]
fn a_body_with_two_different_names_is_refused() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_feed(B, B, Some(GUID), Some(C), Some(D));
    let hops = e.refused(&reader, Condition::ConflictingNames);
    assert_eq!(hops, [hop(B, HopResult::ConflictingNames)]);
}

#[test]
fn a_followed_url_that_is_not_public_is_refused() {
    let mut e = env(URL);
    set_tag(&e, B);
    let reader = ScriptedReader::empty().with_failure(B, FailKind::NotPublic);
    let hops = e.refused(&reader, Condition::NotPublic);
    assert_eq!(hops, [hop(B, HopResult::NotPublic)]);
}

#[test]
fn a_destination_that_is_not_public_is_refused_and_no_file_changes() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let reader = ScriptedReader::empty().with_failure(DEST, FailKind::NotPublic);
    let hops = e.refused(&reader, Condition::NotPublic);
    assert_eq!(hops, [hop(DEST, HopResult::NotPublic)]);
    assert!(files(e.dir_b.path()).is_empty());
}

#[test]
fn a_move_sends_one_read_to_the_destination_and_acts_only_on_not_public() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    // The destination has no feed yet, so the read fails. The write goes on.
    let reader = ScriptedReader::empty().with_failure(DEST, FailKind::Gone);
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), [DEST]);
    assert!(files(e.dir_b.path()).contains_key("album.xml"));
}

#[test]
fn a_self_link_that_the_draft_does_not_change_gives_a_notice_and_no_read() {
    // A correct feed behind a 301: the self link is not the public address.
    let old = "https://old.example/album.xml";
    let mut e = env(old);
    e.edit(|f| f.title = Some("Album (Deluxe)".into()));
    let reader = ScriptedReader::empty();
    let report = pipeline::check_links(&e.collection, &e.hosts, &reader, e.feed).unwrap();
    assert_eq!(report.refusal, None);
    assert_eq!(
        report.notices,
        [LinkNotice::SelfLinkDiffers {
            self_url: old.to_owned(),
            draft_url: URL.to_owned(),
        }]
    );
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), Vec::<String>::new());
}

#[test]
fn the_same_self_link_is_refused_when_the_draft_changes_it() {
    let mut e = env(URL);
    let other = "https://other.example/album.xml";
    e.edit(|f| {
        f.set_self_url(other);
    });
    let reader = ScriptedReader::empty();
    let hops = e.refused(&reader, Condition::SelfLinkDiffers);
    assert_eq!(hops, [hop(other, HopResult::DiffersFromDraft)]);
    assert_eq!(reader.reads(), Vec::<String>::new());

    // A change of the self link to the URL of the draft passes.
    let mut e = env("https://old.example/album.xml");
    e.edit(|f| {
        f.set_self_url(URL);
    });
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), Vec::<String>::new());
}

/// The stage 1 result of a draft with no URL (ADR 0126 item 4).
fn check_no_url(xml: &str, reader: &ScriptedReader) -> studio_core::move_link::LinkReport {
    let doc = studio_core::xml::Document::parse_bytes(xml.as_bytes()).unwrap();
    let feed = studio_core::feed::Feed::read(&doc).unwrap();
    move_link::check(
        reader,
        &DraftLinks {
            feed: &feed,
            url: None,
            self_changed: true,
            destination: None,
        },
    )
}

#[test]
fn a_draft_with_no_url_follows_a_self_link_that_it_changes() {
    // ADR 0126 item 5.
    let reader = ScriptedReader::empty().with_failure(URL, FailKind::Gone);
    let report = check_no_url(&feed_xml(URL), &reader);
    let refusal = report.refusal.unwrap();
    assert_eq!(refusal.condition, Condition::ReadFails);
    assert_eq!(refusal.hops, vec![hop(URL, HopResult::Gone)]);
    assert_eq!(reader.reads(), vec![URL.to_owned()]);
}

#[test]
fn a_draft_with_no_url_that_names_two_urls_is_refused_with_no_read() {
    // ADR 0126 item 9, condition 4.
    let xml = feed_xml(URL).replace(
        "<podcast:medium>",
        &format!("<itunes:new-feed-url xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">{DEST}</itunes:new-feed-url><podcast:medium>"),
    );
    let reader = ScriptedReader::empty();
    let report = check_no_url(&xml, &reader);
    assert_eq!(
        report.refusal.unwrap().condition,
        Condition::ConflictingNames
    );
    assert_eq!(reader.reads(), Vec::<String>::new());
}

#[test]
fn the_real_reader_gives_not_public_for_a_loopback_address() {
    // ADR 0126 item 7. The policy refuses the address before a connection.
    let failure = WebFeedReader::public()
        .read("http://127.0.0.1:9/feed.xml")
        .unwrap_err();
    assert_eq!(failure.kind, FailKind::NotPublic);
}
