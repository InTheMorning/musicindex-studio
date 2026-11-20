//! The state of the retire (ADR 0131 item 6). No test sends a request.

use studio_core::feed_reads::ScriptedIndexReads;
use studio_core::feed_retire::{
    FileDelete, IndexRemoval, RetireAsk, RetireFacts, read_removal, retire_state,
};
use studio_core::musicindex::IndexFeed;
use studio_core::outside::LastWrittenMove;

fn removal_at(
    index_url: Option<&str>,
    own: Option<&str>,
    movement: Option<&LastWrittenMove>,
) -> IndexRemoval {
    let record = IndexFeed {
        url: index_url.map(str::to_owned),
        title: None,
        last_build_date: None,
    };
    let reads = ScriptedIndexReads::empty().with_feed(Ok(Some(record)));
    read_removal(Some("g"), own, movement, &reads)
}

#[test]
fn a_guid_at_a_different_address_gives_a_leftover_copy_with_step_2_done() {
    let removal = removal_at(
        Some("https://other.example/a.xml"),
        Some("https://music.example/a.xml"),
        None,
    );
    assert_eq!(
        removal,
        IndexRemoval::LeftoverCopy {
            index_url: "https://other.example/a.xml".to_owned()
        },
        "ADR 0228 item 20: a record at another address makes this file a leftover copy."
    );
    let facts = RetireFacts {
        ask: RetireAsk::No,
        delete: FileDelete::NotDone,
        guid: Some("g".to_owned()),
        public_address: Some("https://music.example/a.xml".to_owned()),
        last_move: None,
    };
    assert!(retire_state(&facts, removal).removal_done);
}

#[test]
fn the_same_address_a_spelling_difference_or_an_unknown_address_gives_not_yet() {
    let own = "https://music.example/a.xml";
    for (index, own) in [
        (Some(own), Some(own)),
        (Some("HTTPS://Music.Example/a.xml"), Some(own)),
        (Some("https://music.example:443/a.xml"), Some(own)),
        (Some("https://op3.dev/e/music.example/a.xml"), Some(own)),
        (Some("https://other.example/a.xml"), None),
        (None, Some(own)),
    ] {
        assert_eq!(
            removal_at(index, own, None),
            IndexRemoval::NotYet,
            "ADR 0228 items 20 and 22: {index:?} with {own:?} gives no leftover copy."
        );
    }
}

#[test]
fn the_source_address_of_a_studio_move_gives_not_yet() {
    let movement = LastWrittenMove {
        from_url: Some("https://old.example/a.xml".to_owned()),
        moved_at: 100,
        announcements: Vec::new(),
    };
    for index in [
        "https://old.example/a.xml",
        "https://op3.dev/e/old.example/a.xml",
    ] {
        assert_eq!(
            removal_at(
                Some(index),
                Some("https://music.example/a.xml"),
                Some(&movement)
            ),
            IndexRemoval::NotYet,
            "ADR 0228 item 21: Music Index reads the earlier address until it follows the move."
        );
    }
}

#[test]
fn the_read_gives_removed_not_yet_failed_or_no_guid() {
    let record = IndexFeed {
        url: Some("https://music.example/a.xml".to_owned()),
        title: None,
        last_build_date: None,
    };
    let reads = ScriptedIndexReads::empty().with_feed(Ok(None));
    assert_eq!(
        read_removal(Some("g"), None, None, &reads),
        IndexRemoval::Removed
    );
    let reads = ScriptedIndexReads::empty().with_feed(Ok(Some(record)));
    assert_eq!(
        read_removal(Some("g"), None, None, &reads),
        IndexRemoval::NotYet
    );
    let reads = ScriptedIndexReads::empty().with_feed(Err("http://index/v1/feeds/g".to_owned()));
    assert_eq!(
        read_removal(Some("g"), None, None, &reads),
        IndexRemoval::NotKnownFailed {
            address: "http://index/v1/feeds/g".to_owned()
        }
    );
    let reads = ScriptedIndexReads::empty();
    assert_eq!(
        read_removal(None, None, None, &reads),
        IndexRemoval::NotKnownNoGuid
    );
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
            public_address: None,
            last_move: None,
        }
    }

    fn answer(status: u16, body: &[u8]) -> (IndexRemoval, bool) {
        let path: &'static str = Box::leak(format!("/v1/feeds/{GUID}").into_boxed_str());
        let server = TestServer::start(vec![(path, status, body.to_vec())]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let removal = read_removal(Some(GUID), None, None, &index);
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
