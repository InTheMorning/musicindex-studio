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

mod answers {
    use studio_core::feed_retire::{
        FileDelete, IndexRemoval, RetireAsk, RetireFacts, read_removal, retire_state,
    };
    use studio_core::musicindex::MusicIndex;
    use studio_core::testing::TestServer;
    use studio_core::web::Web;

    const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
    const NEW: &str = "0b1c2d3e-0000-5cef-ace4-edaa54e1f810";

    fn facts() -> RetireFacts {
        RetireFacts {
            ask: RetireAsk::Written,
            delete: FileDelete::NotDone,
            guid: Some(GUID.to_owned()),
        }
    }

    fn answer(status: u16, body: &[u8]) -> (IndexRemoval, bool) {
        let path: &'static str = Box::leak(format!("/v1/feeds/{GUID}").into_boxed_str());
        let server = TestServer::start(vec![(path, status, body.to_vec())]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let removal = read_removal(Some(GUID), &index);
        let state = retire_state(&facts(), removal.clone());
        (removal, state.removal_done)
    }

    #[test]
    fn a_plain_404_gives_removed_and_step_2_done() {
        assert_eq!(
            answer(404, br#"{"error":"not found"}"#),
            (IndexRemoval::Removed, true)
        );
        assert_eq!(answer(404, b""), (IndexRemoval::Removed, true));
    }

    #[test]
    fn a_404_with_superseded_by_gives_the_new_guid_and_step_2_open() {
        let body = format!(r#"{{"error":"superseded","superseded_by":"{NEW}"}}"#);
        assert_eq!(
            answer(404, body.as_bytes()),
            (
                IndexRemoval::HeldWithOtherGuid {
                    new_guid: NEW.to_owned()
                },
                false
            )
        );
    }

    #[test]
    fn a_200_gives_not_yet_and_step_2_open() {
        let body = br#"{"data":{"feed_url":"https://music.example/a.xml"}}"#;
        assert_eq!(answer(200, body), (IndexRemoval::NotYet, false));
    }
}
