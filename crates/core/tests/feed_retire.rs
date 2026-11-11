//! The state of the retire (ADR 0131 item 6). No test sends a request.

use studio_core::feed_reads::ScriptedIndexReads;
use studio_core::feed_retire::{IndexRemoval, read_removal};
use studio_core::musicindex::IndexFeed;

#[test]
fn the_read_gives_removed_not_yet_failed_or_no_guid() {
    let record = IndexFeed {
        url: Some("https://music.example/a.xml".to_owned()),
        title: None,
        last_build_date: None,
    };
    let reads = ScriptedIndexReads::empty().with_feed(Ok(None));
    assert_eq!(read_removal(Some("g"), &reads), IndexRemoval::Removed);
    let reads = ScriptedIndexReads::empty().with_feed(Ok(Some(record)));
    assert_eq!(read_removal(Some("g"), &reads), IndexRemoval::NotYet);
    let reads = ScriptedIndexReads::empty().with_feed(Err("http://index/v1/feeds/g".to_owned()));
    assert_eq!(
        read_removal(Some("g"), &reads),
        IndexRemoval::NotKnownFailed {
            address: "http://index/v1/feeds/g".to_owned()
        }
    );
    let reads = ScriptedIndexReads::empty();
    assert_eq!(read_removal(None, &reads), IndexRemoval::NotKnownNoGuid);
    assert!(reads.reads().is_empty(), "no guid sends no read");
}
