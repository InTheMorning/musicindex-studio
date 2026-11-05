//! Stage 1 of ADR 0126: the write refuses a bad link before any file changes.
//! No test sends a request to the network (ADR 0045). A scripted reader
//! serves the bodies.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use studio_core::collection::{Collection, ConnectorId, FeedId};
use studio_core::connector::{Connector, Expected, LocalFolder};
use studio_core::discover::{FailKind, ReadFailure};
use studio_core::feed::Feed;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::move_link::{
    self, Condition, DraftLinks, FeedBody, FeedReader, Hop, HopResult, LinkNotice, Repair,
    ScriptedReader, WebFeedReader,
};
use studio_core::pipeline::{self, AddressCheck, PipelineError, PipelineErrorKind};
use studio_core::xml::Document;

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
    env_on(self_url, LocalFolder::KIND)
}

/// [`env`] with a host A of the connector kind `kind`. Host A is a local
/// folder in each case.
fn env_on(self_url: &str, kind: &str) -> Env {
    env_with_xml(feed_xml(self_url), kind)
}

/// [`env_on`] with the file `original` on host A.
fn env_with_xml(original: String, kind: &str) -> Env {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let folder_a = LocalFolder::new(dir_a.path(), Some("https://a.example".into()));
    let folder_b = LocalFolder::new(dir_b.path(), Some("https://b.example".into()));
    folder_a
        .write("album.xml", original.as_bytes(), &Expected::Absent)
        .unwrap();
    let mut collection = Collection::open(&dir_a.path().join("collection.sqlite")).unwrap();
    let a = collection
        .add_connector(kind, "A", &folder_a.config(), None)
        .unwrap();
    let b = collection
        .add_connector(LocalFolder::KIND, "B", &folder_b.config(), None)
        .unwrap();
    let library = Library::open(dir_a.path().join("library")).unwrap();
    let feed = collection
        .add_feed(Some(URL), Some((a, "album.xml")))
        .unwrap();
    pipeline::refresh(&mut collection, &folder_a, feed).unwrap();
    collection.set_feed_folder(feed, "album").unwrap();
    library
        .write_feed_xml("album", "album.xml", original.as_bytes())
        .unwrap();
    pipeline::start_draft(&collection, &folder_a, feed).unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(a, Box::new(folder_a));
    hosts.insert(b, Box::new(folder_b));
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

fn failure(kind: FailKind) -> ReadFailure {
    ReadFailure {
        kind,
        message: "scripted failure".to_owned(),
    }
}

/// The body that a destination serves when it has the draft.
fn draft_body(env: &Env, url: &str) -> FeedBody {
    let draft = env.collection.draft(env.feed).unwrap().unwrap();
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml).unwrap()).unwrap();
    FeedBody::of(&feed, url.to_owned())
}

/// A reader for a move. The read of stage 1 gets the first result, and the
/// reads of stage 2 get the others.
fn destination(results: Vec<Result<FeedBody, ReadFailure>>) -> ScriptedReader {
    ScriptedReader::empty().with_results(DEST, results)
}

fn gone() -> Result<FeedBody, ReadFailure> {
    Err(failure(FailKind::Gone))
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
fn a_move_sends_one_read_to_the_destination_in_stage_1_and_acts_only_on_not_public() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    // The destination has no feed yet, so the read of stage 1 fails. The
    // write goes on. Stage 2 reads the destination again after the write.
    let reader = ScriptedReader::empty().with_results(
        DEST,
        vec![Err(failure(FailKind::Gone)), Ok(draft_body(&e, DEST))],
    );
    e.write(&reader).unwrap();
    assert_eq!(reader.reads(), [DEST, DEST]);
    assert_eq!(reader.waits(), 0);
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

// ---- Stage 2: the read of the destination (ADR 0126 items 14 to 19) ----

/// A moved feed whose destination fails stage 2 three times. Gives the
/// environment and the refusal, after the checks that every refusal shares.
fn refused_move() -> (Env, ScriptedReader, Vec<u8>) {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let draft = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let reader = destination(vec![gone()]);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::BadMoveLink, "{error}");
    let refusal = error.bad_move_link().unwrap();
    assert_eq!(refusal.condition, Condition::ReadFails);
    assert_eq!(refusal.hops, [hop(DEST, HopResult::Gone)]);
    // One read of stage 1 and 3 reads of stage 2, with 2 waits.
    assert_eq!(reader.reads(), [DEST; 4]);
    assert_eq!(reader.waits(), 2);
    (e, reader, draft)
}

/// The state that every stage 2 refusal leaves (ADR 0126 item 18).
fn assert_kept_after_refusal(e: &Env, placed: &[u8]) {
    assert_eq!(
        fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap(),
        e.original,
        "the old host keeps its file, and it has no pointer"
    );
    assert_eq!(files(e.dir_b.path()).get("album.xml").unwrap(), placed);
    let row = e.collection.feed(e.feed).unwrap();
    assert_ne!(row.connector_id, Some(e.b), "the feed keeps its location");
    assert_eq!(row.public_url.as_deref(), Some(URL));
    assert!(e.collection.draft(e.feed).unwrap().is_some());
    assert_eq!(
        e.collection
            .draft_feed_move(e.feed)
            .unwrap()
            .unwrap()
            .connector_id,
        e.b,
        "the draft keeps its move"
    );
    assert_eq!(e.collection.snapshots(e.feed).unwrap().len(), 0);
    let a = row.connector_id.unwrap();
    assert!(!e.collection.is_feed_pointer(a, "album.xml").unwrap());
    assert!(
        e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap(),
        "the refusal records the placed file"
    );
    assert!(!e.collection.is_feed_pointer(e.b, "album.xml").unwrap());
}

#[test]
fn a_destination_that_fails_stage_2_gets_no_pointer_and_keeps_the_draft_move() {
    let (e, _, placed) = refused_move();
    assert_kept_after_refusal(&e, &placed);
}

#[test]
fn a_destination_with_a_different_guid_is_condition_3_and_gets_no_pointer() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let placed = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let mut other = draft_body(&e, DEST);
    other.podcast_guid = Some("other-guid".to_owned());
    let reader = destination(vec![Ok(other)]);
    let error = e.write(&reader).unwrap_err();
    let refusal = error.bad_move_link().unwrap();
    assert_eq!(refusal.condition, Condition::GuidMismatch);
    assert_eq!(refusal.hops, [hop(DEST, HopResult::GuidMismatch)]);
    assert_kept_after_refusal(&e, &placed);
}

#[test]
fn a_destination_with_other_names_than_the_draft_is_condition_2() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let placed = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let mut stale = draft_body(&e, DEST);
    stale.names.self_link = Some(URL.to_owned());
    let reader = destination(vec![Ok(stale.clone())]);
    let error = e.write(&reader).unwrap_err();
    let refusal = error.bad_move_link().unwrap();
    assert_eq!(refusal.condition, Condition::ReadFails);
    assert_eq!(refusal.hops, [hop(DEST, HopResult::NamesDiffer)]);
    assert_kept_after_refusal(&e, &placed);

    // A tag that the draft does not have is also another name.
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let mut tagged = draft_body(&e, DEST);
    tagged.names.tag = Some(C.to_owned());
    let error = e.write(&destination(vec![Ok(tagged)])).unwrap_err();
    assert_eq!(
        error.bad_move_link().unwrap().condition,
        Condition::ReadFails
    );
}

#[test]
fn the_pointer_is_written_only_after_stage_2_passes() {
    // A reader that records what the old host holds at each read.
    struct Watching {
        old: std::path::PathBuf,
        body: FeedBody,
        seen: std::sync::Mutex<Vec<String>>,
    }
    impl FeedReader for Watching {
        fn read(&self, _url: &str) -> Result<FeedBody, ReadFailure> {
            let on_old = fs::read_to_string(self.old.join("album.xml")).unwrap();
            self.seen.lock().unwrap().push(on_old);
            Ok(self.body.clone())
        }
        fn wait_between_reads(&self) {}
    }
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let reader = Watching {
        old: e.dir_a.path().to_path_buf(),
        body: draft_body(&e, DEST),
        seen: std::sync::Mutex::default(),
    };
    e.write(&reader).unwrap();
    let seen = reader.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2, "one read of stage 1 and one of stage 2");
    assert!(
        seen.iter().all(|text| *text == e.original),
        "the old host holds its feed at each read"
    );
    let on_old = fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap();
    assert!(on_old.contains(&format!("new-feed-url>{DEST}</")));
    assert!(
        e.collection
            .is_feed_pointer(ConnectorId(1), "album.xml")
            .unwrap()
    );
    assert!(
        !e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap()
    );
}

#[test]
fn after_a_refusal_a_repair_lets_the_next_write_complete_the_move() {
    let (mut e, _, _) = refused_move();
    e.edit(|f| f.title = Some("Album (Repaired)".into()));
    let repaired = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let reader = destination(vec![gone(), Ok(draft_body(&e, DEST))]);
    e.write(&reader).unwrap();

    assert_eq!(files(e.dir_b.path()).get("album.xml").unwrap(), &repaired);
    assert_names_own_url(&on_host(&e, e.b), DEST);
    assert!(
        !e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap(),
        "a successful move removes the row"
    );
    let on_old = fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap();
    assert!(on_old.contains(&format!("new-feed-url>{DEST}</")));
    assert!(
        e.collection
            .is_feed_pointer(ConnectorId(1), "album.xml")
            .unwrap()
    );
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(e.b));
    assert!(e.collection.draft(e.feed).unwrap().is_none());
}

#[test]
fn a_placed_file_with_a_different_guid_gives_file_exists_and_does_not_change() {
    let (mut e, _, _) = refused_move();
    // The file on the host changed to a feed of another podcast.
    let other = e
        .original
        .replace(GUID, "00000000-0000-0000-0000-000000000000");
    fs::write(e.dir_b.path().join("album.xml"), &other).unwrap();
    e.edit(|f| f.title = Some("Album (Repaired)".into()));
    let reader = destination(vec![Ok(draft_body(&e, DEST))]);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert_eq!(
        fs::read_to_string(e.dir_b.path().join("album.xml")).unwrap(),
        other
    );
    let on_old = fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap();
    assert_eq!(on_old, e.original);
}

#[test]
fn a_file_without_a_row_is_not_a_placed_file() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    // The same GUID, but Studio did not place the file.
    fs::write(e.dir_b.path().join("album.xml"), e.original.as_bytes()).unwrap();
    let reader = destination(vec![Ok(draft_body(&e, DEST))]);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert_eq!(
        fs::read_to_string(e.dir_b.path().join("album.xml")).unwrap(),
        e.original
    );
}

#[test]
fn set_feed_host_accepts_a_placed_file_of_this_feed_and_not_one_of_another_feed() {
    let (mut e, _, _) = refused_move();
    // The refusal scanned the new host, so the placed file is in the inventory.
    assert!(e.collection.host_file(e.b, "album.xml").unwrap().is_some());
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();

    let other = e.collection.add_feed(None, None).unwrap();
    e.collection
        .add_placed_feed_file(e.b, "album.xml", other)
        .unwrap();
    let error = pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
}

#[test]
fn a_delete_of_a_placed_file_by_the_user_removes_the_row_and_the_file() {
    let (mut e, _, _) = refused_move();
    // The user chooses the host of the feed again, and deletes the file.
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, ConnectorId(1)).unwrap();
    e.collection
        .add_draft_operation(
            e.feed,
            studio_core::collection::DraftOperationKind::Delete,
            None,
            e.b,
            "album.xml",
        )
        .unwrap();
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert_eq!(diff.blocked.len(), 0, "a placed file is not a pointer file");
    e.edit(|f| f.title = Some("Album (Repaired)".into()));
    let reader = ScriptedReader::empty();
    e.write(&reader).unwrap();
    assert!(!files(e.dir_b.path()).contains_key("album.xml"));
    assert!(
        !e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap()
    );
}

#[test]
fn a_slow_destination_passes_on_the_second_or_the_third_read() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let good = draft_body(&e, DEST);
    // Stage 1, then stage 2 read 1 (stale), then read 2 (correct).
    let reader = destination(vec![gone(), gone(), Ok(good.clone())]);
    e.write(&reader).unwrap();
    assert_eq!(reader.reads().len(), 3);
    assert_eq!(reader.waits(), 1);

    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let good = draft_body(&e, DEST);
    let reader = destination(vec![gone(), gone(), gone(), Ok(good)]);
    e.write(&reader).unwrap();
    assert_eq!(reader.reads().len(), 4);
    assert_eq!(reader.waits(), 2);
}

#[test]
fn a_destination_that_is_not_public_gets_no_second_read_in_stage_2() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let reader = destination(vec![gone(), Err(failure(FailKind::NotPublic))]);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(
        error.bad_move_link().unwrap().condition,
        Condition::NotPublic
    );
    assert_eq!(reader.waits(), 0);
}

#[test]
fn a_move_with_no_tag_keeps_the_names_exact_by_text() {
    // The check compares the names by exact text (ADR 0126 item 2).
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let mut body = draft_body(&e, DEST);
    body.names.self_link = body.names.self_link.map(|text| format!("{text}/"));
    let error = e.write(&destination(vec![Ok(body)])).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::BadMoveLink);
}

#[test]
fn a_move_from_an_expired_temporary_area_runs_stage_2_and_writes_no_pointer() {
    let mut e = env_on(URL, studio_core::connector::TEMPORARY_KIND);
    let a = e.collection.feed(e.feed).unwrap().connector_id.unwrap();
    // The server removed the files of the area (ADR 0122 item 10).
    fs::remove_file(e.dir_a.path().join("album.xml")).unwrap();
    e.collection
        .set_connector_config(a, "{\"expired_at\": 1}")
        .unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let placed = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;

    // Stage 2 refuses: the new file stays as a placed file, and the feed
    // keeps its location.
    let reader = destination(vec![gone()]);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::BadMoveLink, "{error}");
    assert_eq!(reader.reads().len(), 4);
    assert_eq!(files(e.dir_b.path()).get("album.xml").unwrap(), &placed);
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(a));
    assert!(e.collection.draft_feed_move(e.feed).unwrap().is_some());
    assert!(
        e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap()
    );
    assert!(!files(e.dir_a.path()).contains_key("album.xml"));

    // After a pass, the move completes with no pointer on the old host.
    let reader = destination(vec![gone(), Ok(draft_body(&e, DEST))]);
    e.write(&reader).unwrap();
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(e.b));
    assert!(!files(e.dir_a.path()).contains_key("album.xml"));
    assert!(!e.collection.is_feed_pointer(a, "album.xml").unwrap());
    assert!(
        !e.collection
            .is_placed_feed_file(e.b, "album.xml", e.feed)
            .unwrap()
    );
}

// ---- A return to an earlier host (ADR 0126 items 21 to 23) ----

const A_ID: ConnectorId = ConnectorId(1);

/// Move the feed to `target`, which serves the draft at `url`. The first
/// move uses the draft of the environment. Each later move starts a draft
/// on the host of the feed.
fn move_to(e: &mut Env, target: ConnectorId, url: &str) {
    if e.collection.draft(e.feed).unwrap().is_none() {
        let host = e.collection.feed(e.feed).unwrap().connector_id.unwrap();
        pipeline::start_draft(&e.collection, e.hosts.get(host).unwrap(), e.feed).unwrap();
    }
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, target).unwrap();
    let reader = ScriptedReader::empty().with_body(url, draft_body(e, url));
    e.write(&reader).unwrap();
}

fn move_to_b(e: &mut Env) {
    let b = e.b;
    move_to(e, b, DEST);
}

fn on_host(e: &Env, id: ConnectorId) -> String {
    let dir = if id == A_ID { &e.dir_a } else { &e.dir_b };
    fs::read_to_string(dir.path().join("album.xml")).unwrap()
}

fn is_pointer_to(text: &str, url: &str) -> bool {
    text.contains(&format!("new-feed-url>{url}</")) && text.contains(GUID)
}

fn is_feed_with_self(text: &str, url: &str) -> bool {
    names_own_url(text) == Some(url.to_owned()) && text.contains(&format!("href=\"{url}\""))
}

/// The text of `itunes:new-feed-url` in a feed file.
fn names_own_url(text: &str) -> Option<String> {
    let start = text.find("new-feed-url>")? + "new-feed-url>".len();
    let end = start + text[start..].find("</")?;
    Some(text[start..end].to_owned())
}

/// The feed on the new host names its own URL by the tag (ADR 0128 item 1).
fn assert_names_own_url(text: &str, url: &str) {
    assert_eq!(
        names_own_url(text).as_deref(),
        Some(url),
        "Stophammer ADR 0083 section 2 and ADR 0128 item 1: a moved feed names its own URL by itunes:new-feed-url, with the exact text of the URL. Fix: set the tag in set_feed_host."
    );
}

#[test]
fn a_move_from_a_to_b_and_back_to_a_completes() {
    let mut e = env(URL);
    move_to_b(&mut e);
    assert_names_own_url(&on_host(&e, e.b), DEST);
    assert!(is_pointer_to(&on_host(&e, A_ID), DEST));
    assert!(e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());

    // The diff of the return says that the pointer file becomes the feed.
    pipeline::start_draft(&e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert!(diff.feed_move.as_ref().unwrap().replaces_pointer);
    move_to(&mut e, A_ID, URL);

    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(A_ID));
    assert!(is_feed_with_self(&on_host(&e, A_ID), URL));
    assert_names_own_url(&on_host(&e, A_ID), URL);
    assert!(is_pointer_to(&on_host(&e, e.b), URL));
    assert!(!e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());
    assert!(e.collection.is_feed_pointer(e.b, "album.xml").unwrap());
    assert!(
        !e.collection
            .is_placed_feed_file(A_ID, "album.xml", e.feed)
            .unwrap()
    );
}

#[test]
fn two_returns_in_sequence_keep_the_bytes_and_the_pointer_rows_right() {
    let mut e = env(URL);
    move_to_b(&mut e);
    let rows = |e: &Env| {
        (
            e.collection.is_feed_pointer(A_ID, "album.xml").unwrap(),
            e.collection.is_feed_pointer(e.b, "album.xml").unwrap(),
        )
    };
    assert_eq!(rows(&e), (true, false));
    assert_names_own_url(&on_host(&e, e.b), DEST);
    move_to(&mut e, A_ID, URL);
    assert!(is_feed_with_self(&on_host(&e, A_ID), URL));
    assert_names_own_url(&on_host(&e, A_ID), URL);
    assert!(is_pointer_to(&on_host(&e, e.b), URL));
    assert_eq!(rows(&e), (false, true));
    move_to_b(&mut e);
    assert!(is_feed_with_self(&on_host(&e, e.b), DEST));
    assert_names_own_url(&on_host(&e, e.b), DEST);
    assert!(is_pointer_to(&on_host(&e, A_ID), DEST));
    assert_eq!(rows(&e), (true, false));
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(e.b));
}

#[test]
fn a_file_at_a_pointer_path_with_a_different_guid_gives_file_exists() {
    let mut e = env(URL);
    move_to_b(&mut e);
    pipeline::start_draft(&e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    // Somebody replaced the pointer file with a feed of another podcast.
    let other = e
        .original
        .replace(GUID, "00000000-0000-0000-0000-000000000000");
    fs::write(e.dir_a.path().join("album.xml"), &other).unwrap();
    let reader = ScriptedReader::empty().with_body(URL, draft_body(&e, URL));
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert_eq!(on_host(&e, A_ID), other);
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(e.b));
    assert!(e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());
}

#[test]
fn set_feed_host_refuses_the_pointer_path_of_a_different_feed() {
    let mut e = env(URL);
    move_to_b(&mut e);
    pipeline::start_draft(&e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    let other = e.collection.add_feed(None, None).unwrap();
    e.collection
        .add_feed_pointer(A_ID, "album.xml", other)
        .unwrap();
    let error = pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert!(
        !e.collection
            .is_pointer_of_feed(A_ID, "album.xml", e.feed)
            .unwrap()
    );
}

#[test]
fn the_diff_of_a_return_has_the_new_value_and_a_normal_move_does_not() {
    let e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let normal = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert!(!normal.feed_move.as_ref().unwrap().replaces_pointer);
    let json = serde_json::to_value(normal.feed_move.as_ref().unwrap()).unwrap();
    assert_eq!(json["replaces_pointer"], false);
    let normal_id = normal.id.clone();

    // The same move with the row of a pointer has a different diff ID.
    e.collection
        .add_feed_pointer(e.b, "album.xml", e.feed)
        .unwrap();
    fs::write(e.dir_b.path().join("album.xml"), e.original.as_bytes()).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let ret = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert!(ret.feed_move.as_ref().unwrap().replaces_pointer);
    assert_ne!(ret.id, normal_id);
    let json = serde_json::to_value(ret.feed_move.as_ref().unwrap()).unwrap();
    assert_eq!(json["replaces_pointer"], true);
}

#[test]
fn a_stage_2_refusal_on_a_return_keeps_the_state_and_a_repair_completes() {
    let mut e = env(URL);
    move_to_b(&mut e);
    pipeline::start_draft(&e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    let placed = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let b_before = on_host(&e, e.b);

    // A CDN still serves the pointer: the body has the tag of the pointer.
    let mut stale = draft_body(&e, URL);
    stale.names.tag = Some(DEST.to_owned());
    let reader = ScriptedReader::empty().with_body(URL, stale);
    let error = e.write(&reader).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::BadMoveLink, "{error}");
    // The destination holds the feed, as a placed file and not a pointer.
    assert_eq!(files(e.dir_a.path()).get("album.xml").unwrap(), &placed);
    assert!(!e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());
    assert!(
        e.collection
            .is_placed_feed_file(A_ID, "album.xml", e.feed)
            .unwrap()
    );
    // The feed keeps its location, and the old host gets no pointer.
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(e.b));
    assert_eq!(on_host(&e, e.b), b_before);
    assert!(!e.collection.is_feed_pointer(e.b, "album.xml").unwrap());
    assert!(e.collection.draft_feed_move(e.feed).unwrap().is_some());

    // A repair, and the next write replaces the placed file and completes.
    e.edit(|f| f.title = Some("Album (Repaired)".into()));
    let reader = ScriptedReader::empty().with_body(URL, draft_body(&e, URL));
    e.write(&reader).unwrap();
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(A_ID));
    assert!(on_host(&e, A_ID).contains("Album (Repaired)"));
    assert_names_own_url(&on_host(&e, A_ID), URL);
    assert!(is_pointer_to(&on_host(&e, e.b), URL));
    assert!(!e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());
    assert!(
        !e.collection
            .is_placed_feed_file(A_ID, "album.xml", e.feed)
            .unwrap()
    );
}

#[test]
fn a_stopped_write_of_a_return_continues() {
    let mut e = env(URL);
    move_to_b(&mut e);
    pipeline::start_draft(&e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    let draft = e.collection.draft(e.feed).unwrap().unwrap().draft_xml;
    let id_before = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed)
        .unwrap()
        .id;
    // The earlier write stopped after the feed XML replaced the pointer
    // file: the file has the draft bytes, and the row is still there.
    fs::write(e.dir_a.path().join("album.xml"), &draft).unwrap();
    let id_after = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed)
        .unwrap()
        .id;
    assert_eq!(id_before, id_after);
    let reader = ScriptedReader::empty().with_body(URL, draft_body(&e, URL));
    e.write(&reader).unwrap();
    assert_eq!(e.collection.feed(e.feed).unwrap().connector_id, Some(A_ID));
    assert_names_own_url(&on_host(&e, A_ID), URL);
    assert!(is_pointer_to(&on_host(&e, e.b), URL));
    assert!(!e.collection.is_feed_pointer(A_ID, "album.xml").unwrap());
}

// ---- The repairs of ADR 0126 item 20 ----

fn repairs_of(e: &Env, reader: &dyn FeedReader) -> (Vec<Repair>, Vec<Repair>) {
    let report = pipeline::check_links(&e.collection, &e.hosts, reader, e.feed).unwrap();
    let refusal = report.refusal.map(|r| r.repairs).unwrap_or_default();
    (refusal, report.repairs)
}

#[test]
fn the_report_gives_the_repairs_that_fit_each_condition() {
    // Condition 1, 2, 3 and 4 from the tag of the draft: remove the tag.
    let e = env(URL);
    set_tag(&e, B);
    let loop_reader = ScriptedReader::empty().with_feed(B, B, Some(GUID), Some(URL), Some(B));
    let remove = vec![Repair::RemoveTag];
    assert_eq!(
        repairs_of(&e, &loop_reader),
        (remove.clone(), remove.clone())
    );
    let gone = ScriptedReader::empty().with_failure(B, FailKind::Gone);
    assert_eq!(repairs_of(&e, &gone).0, remove);
    let guid = ScriptedReader::empty().with_feed(B, B, Some("other"), None, None);
    assert_eq!(repairs_of(&e, &guid).0, remove);
    let names = ScriptedReader::empty().with_feed(B, B, Some(GUID), Some(C), Some(D));
    assert_eq!(repairs_of(&e, &names).0, remove);
    let hidden = ScriptedReader::empty().with_failure(B, FailKind::NotPublic);
    assert_eq!(repairs_of(&e, &hidden).0, remove);

    // Condition 6: use the public address.
    let e = env(URL);
    e.edit(|f| {
        f.set_self_url("https://other.example/album.xml");
    });
    let none = ScriptedReader::empty();
    let use_address = vec![Repair::UsePublicAddress];
    assert_eq!(repairs_of(&e, &none), (use_address.clone(), use_address));

    // Condition 7 for the destination of a move: no repair.
    let e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let hidden = ScriptedReader::empty().with_failure(DEST, FailKind::NotPublic);
    assert_eq!(repairs_of(&e, &hidden), (vec![], vec![]));

    // The notice: use the public address, and no refusal.
    let e = env("https://old.example/album.xml");
    assert_eq!(
        repairs_of(&e, &none),
        (vec![], vec![Repair::UsePublicAddress])
    );

    // Nothing to report: no repair.
    let e = env(URL);
    assert_eq!(repairs_of(&e, &none), (vec![], vec![]));

    // A draft with no URL has no `use_public_address`.
    let report = check_no_url(
        &feed_xml(URL),
        &ScriptedReader::empty().with_failure(URL, FailKind::Gone),
    );
    assert_eq!(report.refusal.unwrap().repairs, Vec::<Repair>::new());
}

#[test]
fn a_stage_2_refusal_has_the_stage_and_the_repairs_that_apply() {
    let (e, _, _) = refused_move();
    // The tag and the self link of the placed draft fit the destination, so
    // no repair applies.
    let mut e = e;
    let reader = destination(vec![gone()]);
    let error = e.write(&reader).unwrap_err();
    let refusal = error.bad_move_link().unwrap();
    assert_eq!(refusal.stage, 2);
    assert_eq!(refusal.repairs, Vec::<Repair>::new());

    // A draft with a tag gets the repair "Remove the Tag" in stage 2.
    // The tag of the move names the URL of the draft (ADR 0128), so it is
    // not a link to repair. A tag that the user sets after the move is.
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    set_tag(&e, C);
    let reader = ScriptedReader::empty()
        .with_feed(C, C, Some(GUID), None, None)
        .with_results(
            DEST,
            vec![Err(failure(FailKind::Gone)), Err(failure(FailKind::Gone))],
        );
    let error = e.write(&reader).unwrap_err();
    let refusal = error.bad_move_link().unwrap();
    assert_eq!(refusal.stage, 2, "{error}");
    assert_eq!(refusal.repairs, [Repair::RemoveTag]);
}

/// The draft text and the ID of its diff.
fn draft_text(e: &Env) -> (String, String) {
    let draft = e.collection.draft(e.feed).unwrap().unwrap();
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    (
        String::from_utf8(draft.draft_xml).unwrap(),
        diff.id.as_str().to_owned(),
    )
}

fn revision_of(e: &Env) -> String {
    pipeline::revision(&e.collection.draft(e.feed).unwrap().unwrap())
}

#[test]
fn use_the_public_address_changes_only_the_self_link_in_the_draft_and_the_diff() {
    let old = "https://old.example/album.xml";
    let e = env(old);
    e.edit(|f| f.title = Some("Album (Deluxe)".into()));
    let (before, id_before) = draft_text(&e);
    pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::UsePublicAddress,
        &revision_of(&e),
    )
    .unwrap();
    let (after, id_after) = draft_text(&e);
    assert_eq!(after, before.replace(old, URL));
    assert_ne!(id_before, id_after);
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    let changed: Vec<&str> = diff
        .unified
        .lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("+++")
                && !l.starts_with("---")
        })
        .collect();
    assert!(changed.iter().any(|l| l.contains(old)), "{changed:?}");
    assert!(changed.iter().any(|l| l.contains(URL)), "{changed:?}");
    assert!(changed.iter().any(|l| l.contains("Deluxe")), "{changed:?}");
    // The repair applies once.
    let error = pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::UsePublicAddress,
        &revision_of(&e),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
}

#[test]
fn remove_the_tag_removes_only_the_tag_and_a_repair_that_does_not_apply_gives_an_error() {
    let e = env(URL);
    set_tag(&e, B);
    let (before, _) = draft_text(&e);
    // It does not apply: the draft has a tag, but no different self link.
    let error = pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::UsePublicAddress,
        &revision_of(&e),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
    // A stale revision changes nothing.
    let error = pipeline::apply_repair(&e.collection, &e.hosts, e.feed, Repair::RemoveTag, "stale")
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::StaleRevision);
    assert_eq!(draft_text(&e).0, before);

    pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::RemoveTag,
        &revision_of(&e),
    )
    .unwrap();
    let (after, _) = draft_text(&e);
    assert!(before.contains(B) && !after.contains(B), "{after}");
    // Only the tag element goes. The namespace declaration stays.
    let start = before.find("<itunes:new-feed-url").unwrap();
    let end = before.find("</itunes:new-feed-url>").unwrap() + "</itunes:new-feed-url>".len();
    let mut expected = before.clone();
    expected.replace_range(start..end, "");
    let normalized = |text: &str| text.split_whitespace().collect::<String>();
    assert_eq!(normalized(&after), normalized(&expected));
    let error = pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::RemoveTag,
        &revision_of(&e),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
}

// ---- The tag of the own address (ADR 0128 items 1 to 6) ----

/// The feed file of `feed_xml` with a tag, and with no self link when
/// `self_link` is false.
fn feed_with(self_link: bool, tag: Option<&str>) -> String {
    let mut xml = feed_xml(URL);
    if !self_link {
        let start = xml.find("    <atom:link").unwrap();
        let end = start + xml[start..].find('\n').unwrap() + 1;
        xml.replace_range(start..end, "");
    }
    if let Some(tag) = tag {
        xml = xml.replace(
            "    <podcast:guid>",
            &format!("    <itunes:new-feed-url>{tag}</itunes:new-feed-url>\n    <podcast:guid>"),
        );
        xml = xml.replace(
            "xmlns:atom=",
            "xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" xmlns:atom=",
        );
    }
    xml
}

fn draft_bytes(e: &Env) -> Vec<u8> {
    e.collection.draft(e.feed).unwrap().unwrap().draft_xml
}

#[test]
fn a_move_gives_the_feed_the_tag_of_its_own_url_and_the_pointer_names_the_destination() {
    let mut e = env(URL);
    move_to_b(&mut e);
    assert_names_own_url(&on_host(&e, e.b), DEST);
    assert!(is_pointer_to(&on_host(&e, A_ID), DEST));
}

#[test]
fn a_new_tag_goes_first_in_the_channel_and_a_tag_of_the_base_stays_in_its_position() {
    let mut e = env(URL);
    move_to_b(&mut e);
    let on_b = on_host(&e, e.b);
    let channel = on_b.find("<channel>").unwrap() + "<channel>".len();
    assert!(
        on_b[channel..]
            .trim_start()
            .starts_with("<itunes:new-feed-url>"),
        "ADR 0128 item 12: a new itunes:new-feed-url goes directly after <channel>. Fix: write it with text_first in Feed::write_hosted.\n{on_b}"
    );

    let with_tag = feed_xml(URL)
        .replace(
            "<rss version=\"2.0\"",
            "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\"",
        )
        .replace(
            "</channel>",
            "<itunes:new-feed-url>https://old.example/a.xml</itunes:new-feed-url></channel>",
        );
    let e = env_with_xml(with_tag, LocalFolder::KIND);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let draft = String::from_utf8(draft_bytes(&e)).unwrap();
    assert_names_own_url(&draft, DEST);
    let channel = draft.find("<channel>").unwrap() + "<channel>".len();
    assert!(
        !draft[channel..]
            .trim_start()
            .starts_with("<itunes:new-feed-url>"),
        "ADR 0128 item 12: a tag that the feed already has stays in its position.\n{draft}"
    );
}

#[test]
fn a_feed_with_no_self_link_gets_the_tag_at_a_move() {
    let mut e = env_with_xml(feed_with(false, None), LocalFolder::KIND);
    move_to_b(&mut e);
    let on_b = on_host(&e, e.b);
    assert_names_own_url(&on_b, DEST);
    assert!(!on_b.contains("rel=\"self\""), "the move adds no self link");
    // The return gives the tag too.
    move_to(&mut e, A_ID, URL);
    assert_names_own_url(&on_host(&e, A_ID), URL);
}

#[test]
fn a_tag_of_the_draft_before_the_move_gets_the_new_value() {
    let mut e = env(URL);
    set_tag(&e, C);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let draft = String::from_utf8(draft_bytes(&e)).unwrap();
    assert_names_own_url(&draft, DEST);
    assert!(!draft.contains(C));
    // Stage 1 has no URL to follow, and the move completes.
    move_to_b(&mut e);
    assert_names_own_url(&on_host(&e, e.b), DEST);
}

#[test]
fn the_diff_of_a_move_shows_the_tag_change() {
    let e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert!(
        diff.unified
            .lines()
            .any(|l| l.starts_with('+') && l.contains(&format!("new-feed-url>{DEST}</"))),
        "{}",
        diff.unified
    );
}

#[test]
fn undo_of_a_move_gives_the_bytes_of_a_base_with_no_tag() {
    let e = env(URL);
    let base = draft_bytes(&e);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    assert_ne!(draft_bytes(&e), base);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    assert_eq!(draft_bytes(&e), base, "ADR 0128 item 4");
    assert!(!pipeline::draft_changed(&e.collection, e.feed).unwrap());
}

#[test]
fn undo_of_a_move_puts_back_the_tag_of_the_base() {
    // The base names its own URL already.
    let e = env_with_xml(feed_with(true, Some(URL)), LocalFolder::KIND);
    let base = draft_bytes(&e);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    assert_names_own_url(&String::from_utf8(draft_bytes(&e)).unwrap(), DEST);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    assert_eq!(draft_bytes(&e), base);
    assert!(!pipeline::draft_changed(&e.collection, e.feed).unwrap());
}

#[test]
fn a_second_host_choice_replaces_the_tag_and_undo_still_gives_the_base() {
    let mut e = env(URL);
    let dir_c = tempfile::tempdir().unwrap();
    let folder_c = LocalFolder::new(dir_c.path(), Some("https://c.example".into()));
    let c = e
        .collection
        .add_connector(LocalFolder::KIND, "C", &folder_c.config(), None)
        .unwrap();
    e.hosts.insert(c, Box::new(folder_c));
    let base = draft_bytes(&e);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, c).unwrap();
    let draft = String::from_utf8(draft_bytes(&e)).unwrap();
    assert_names_own_url(&draft, "https://c.example/album.xml");
    assert!(!draft.contains(DEST));
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, A_ID).unwrap();
    assert_eq!(draft_bytes(&e), base);
}

#[test]
fn a_write_with_no_move_adds_no_tag() {
    let mut e = env(URL);
    e.edit(|f| f.title = Some("Album (Deluxe)".into()));
    e.write(&ScriptedReader::empty()).unwrap();
    let written = fs::read_to_string(e.dir_a.path().join("album.xml")).unwrap();
    assert!(
        !written.contains("new-feed-url"),
        "a feed that never moved gets no tag"
    );
}

#[test]
fn a_tag_that_names_the_address_gives_no_address_difference_after_a_write_and_a_refresh() {
    let mut e = env(URL);
    move_to_b(&mut e);
    assert_eq!(
        e.collection.feed(e.feed).unwrap().address_differences,
        Vec::new()
    );
    pipeline::refresh(&mut e.collection, e.hosts.get(e.b).unwrap(), e.feed).unwrap();
    let row = e.collection.feed(e.feed).unwrap();
    assert_eq!(row.public_url.as_deref(), Some(DEST));
    assert_eq!(row.address_differences, Vec::new());
    // The same after a return and a refresh.
    move_to(&mut e, A_ID, URL);
    pipeline::refresh(&mut e.collection, e.hosts.get(A_ID).unwrap(), e.feed).unwrap();
    let row = e.collection.feed(e.feed).unwrap();
    assert_eq!(row.public_url.as_deref(), Some(URL));
    assert_eq!(row.address_differences, Vec::new());
}

#[test]
fn a_stopped_write_of_a_first_move_continues_with_the_tag() {
    let mut e = env(URL);
    pipeline::set_feed_host(&e.collection, &e.hosts, None, e.feed, e.b).unwrap();
    let draft = draft_bytes(&e);
    // The earlier write stopped after the feed XML reached the new host.
    fs::write(e.dir_b.path().join("album.xml"), &draft).unwrap();
    let reader = ScriptedReader::empty().with_body(DEST, draft_body(&e, DEST));
    e.write(&reader).unwrap();
    assert_names_own_url(&on_host(&e, e.b), DEST);
    assert!(is_pointer_to(&on_host(&e, A_ID), DEST));
}

#[test]
fn a_repair_of_a_draft_gets_a_date_at_its_diff_and_keeps_the_id_of_a_second_diff() {
    // ADR 0127 item 19: a repair is a change like each other.
    let old = "https://old.example/album.xml";
    let e = env(old);
    e.edit(|f| f.title = Some("Album (Deluxe)".into()));
    pipeline::apply_repair(
        &e.collection,
        &e.hosts,
        e.feed,
        Repair::UsePublicAddress,
        &revision_of(&e),
    )
    .unwrap();
    let checked = studio_core::checked_time::CheckedTime {
        seconds: 1_793_390_400,
        computer_ahead_seconds: 0,
    };
    let state = pipeline::set_build_date(&e.collection, e.feed, Some(checked)).unwrap();
    assert!(state.set);
    let first = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert!(
        first
            .unified
            .contains("<lastBuildDate>Fri, 30 Oct 2026 20:00:00 +0000")
    );
    // The next diff keeps the date, and the revision of the editor stays valid.
    pipeline::set_build_date(&e.collection, e.feed, Some(checked)).unwrap();
    let second = pipeline::diff(&e.collection, &e.hosts, &e.library, e.feed).unwrap();
    assert_eq!(first.id, second.id);
}
