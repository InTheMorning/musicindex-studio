//! Round-trip tests over the corpus of published feeds (ADR 0012).
//!
//! `corpus/manifest.json` records the source URL, the fetch time and the
//! SHA-256 of each file.

use std::fs;
use std::path::PathBuf;

use studio_core::feed::Feed;
use studio_core::xml::Document;

fn corpus() -> Vec<(String, Vec<u8>)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("corpus directory")
        .map(|e| e.expect("corpus entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "xml"))
        .collect();
    files.sort();
    assert!(files.len() >= 20, "the corpus has {} files", files.len());
    files
        .into_iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (name, fs::read(&p).expect("corpus file"))
        })
        .collect()
}

#[test]
fn each_corpus_feed_writes_the_same_bytes_after_a_read() {
    for (name, bytes) in corpus() {
        let doc = Document::parse_bytes(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            doc.write().as_bytes() == bytes,
            "{name}: tree round trip changed bytes"
        );
    }
}

#[test]
fn each_corpus_feed_writes_the_same_bytes_after_a_projection() {
    for (name, bytes) in corpus() {
        let mut doc = Document::parse_bytes(&bytes).unwrap();
        let feed = Feed::read(&doc).unwrap_or_else(|e| panic!("{name}: {e}"));
        feed.write(&mut doc).unwrap();
        assert!(
            doc.write().as_bytes() == bytes,
            "{name}: model round trip changed bytes"
        );
    }
}

#[test]
fn each_corpus_feed_writes_the_same_bytes_after_a_json_round_trip() {
    for (name, bytes) in corpus() {
        let mut doc = Document::parse_bytes(&bytes).unwrap();
        let draft = Feed::read(&doc).unwrap();
        let json = serde_json::to_string(&draft).unwrap();
        let mut sent: Feed = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{name}: {e}"));
        sent.adopt(&draft).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(sent, draft, "{name}: the JSON model is different");
        sent.write(&mut doc).unwrap();
        assert!(
            doc.write().as_bytes() == bytes,
            "{name}: JSON round trip changed bytes"
        );
    }
}

/// The evidence of ADR 0023: one corpus feed has an address with no scheme.
#[test]
fn one_corpus_feed_has_a_bad_address() {
    let bad: Vec<(String, Vec<String>)> = corpus()
        .into_iter()
        .map(|(name, bytes)| {
            let feed = Feed::read(&Document::parse_bytes(&bytes).unwrap()).unwrap();
            let fields = feed.bad_addresses().into_iter().map(|b| b.field).collect();
            (name, fields)
        })
        .filter(|(_, fields): &(String, Vec<String>)| !fields.is_empty())
        .collect();
    assert_eq!(
        bad,
        [(
            "musicsideproject-publisher-strange-love.xml".to_owned(),
            vec!["link".to_owned()]
        )]
    );
}

#[test]
fn the_corpus_has_value_time_splits_live_items_and_publishers() {
    let feeds: Vec<Feed> = corpus()
        .iter()
        .map(|(_, b)| Feed::read(&Document::parse_bytes(b).unwrap()).unwrap())
        .collect();
    let splits = feeds
        .iter()
        .flat_map(|f| {
            f.items
                .iter()
                .map(|i| &i.value)
                .chain(f.live_items.iter().map(|l| &l.item))
        })
        .flat_map(|i| i.values.iter())
        .map(|v| v.time_splits.len())
        .sum::<usize>();
    assert!(splits > 0, "no podcast:valueTimeSplit in the corpus");
    assert!(feeds.iter().any(|f| !f.live_items.is_empty()));
    assert!(feeds.iter().any(|f| f.publisher.is_some()));
    assert!(feeds.iter().any(|f| f.medium.as_deref() == Some("musicL")));
}

#[test]
fn the_corpus_has_persons_transcripts_fundings_and_self_links() {
    let feeds: Vec<Feed> = corpus()
        .iter()
        .map(|(_, b)| Feed::read(&Document::parse_bytes(b).unwrap()).unwrap())
        .collect();
    let items = || feeds.iter().flat_map(|f| f.items.iter());
    assert!(feeds.iter().any(|f| !f.persons.is_empty()));
    assert!(feeds.iter().any(|f| !f.fundings.is_empty()));
    assert!(feeds.iter().any(|f| f.self_url().is_some()));
    assert!(items().any(|i| !i.transcripts.is_empty()));
    assert!(items().any(|i| i.enclosure.is_some() && i.guid.is_some()));
}
