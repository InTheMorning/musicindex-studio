//! The routes of the retire and of the delete of a feed (ADR 0131). No test
//! sends a request outside the computer (ADR 0045). A server on a loopback
//! port plays the index and counts its requests.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use studio_core::feed_reads::ScriptedFileReads;
use studio_core::move_link::ScriptedReader;

use super::*;

const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";

/// A server that plays the index. It answers each request with `status` and
/// `body`, and counts the requests.
struct CountingIndex {
    port: u16,
    count: Arc<AtomicUsize>,
}

impl CountingIndex {
    fn start(status: u16, body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&count);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut buffer = [0_u8; 4096];
                let _ = stream.read(&mut buffer);
                counted.fetch_add(1, Ordering::SeqCst);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        Self { port, count }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn requests(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

fn feed_xml(title: &str, guid: &str) -> String {
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" \
         xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"><channel>\
         <title>{title}</title><language>en</language>{guid}\
         <podcast:medium>music</podcast:medium>\
         <item><guid>t-1</guid><title>One</title><itunes:duration>180</itunes:duration>\
         <enclosure url=\"https://music.example/one.mp3\" length=\"5\" type=\"audio/mpeg\"/></item>\
         </channel></rss>"
    )
}

fn guid_tag() -> String {
    format!("<podcast:guid>{GUID}</podcast:guid>")
}

/// An app with a local host named `Home`, a feed `a.xml` on it, and the
/// index at `index`. Gives the app and the ID of the feed.
async fn app_with(index: &str, xml: &str) -> (App, i64) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(index)
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(Arc::new(PassAll))
        .with_stream_check(Arc::new(StubStream))
        .with_feed_reader(Arc::new(ScriptedReader::empty()))
        .with_file_reads(Arc::new(ScriptedFileReads::empty()));
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    };
    let root = app.feeds.path().to_str().unwrap().to_owned();
    let (status, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    set_library_root(&app);
    let id = add_paid_feed(&app, connector["id"].as_i64().unwrap(), "a.xml", xml).await;
    (app, id)
}

async fn state(app: &App, id: i64) -> Value {
    let (status, body) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/retire/state"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// Write the draft with the diff that the user approved.
async fn approve_and_write(app: &App, id: i64, diff: &Value) {
    let (status, body) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn only_the_retire_state_route_reads_the_index_and_it_reads_once() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");
    let other_routes = [
        ("POST", format!("{base}/retire"), None),
        ("DELETE", format!("{base}/retire"), None),
        ("POST", format!("{base}/delete/draft"), Some(json!({}))),
        ("POST", format!("{base}/delete/restore"), None),
        ("GET", format!("{base}/diff"), None),
        ("GET", format!("{base}/check"), None),
    ];
    for (method, path, body) in other_routes {
        let _ = call(&app, method, &path, body).await;
        assert_eq!(index.requests(), 0, "{method} {path} sent an index read");
    }
    let answer = state(&app, id).await;
    assert_eq!(index.requests(), 1, "ADR 0131 item 6");
    assert_eq!(answer["removal"]["answer"], "removed");
    assert_eq!(answer["removal_done"], true);
}

#[tokio::test]
async fn the_retire_state_gives_the_index_answer_and_the_request_address() {
    // A record: not yet removed.
    let index = CountingIndex::start(
        200,
        r#"{"data":{"feed_url":"https://music.example/a.xml"}}"#,
    );
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "not_yet");
    assert_eq!(answer["removal_done"], false);
    assert_eq!(answer["ask"], "no");
    assert_eq!(answer["delete"], "not_done");
    assert_eq!(index.requests(), 1);

    // A 404 with superseded_by: the index holds the feed with a new GUID.
    let index = CountingIndex::start(404, r#"{"error":"superseded","superseded_by":"new-guid"}"#);
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "held_with_other_guid");
    assert_eq!(answer["removal"]["new_guid"], "new-guid");
    assert_eq!(answer["removal_done"], false);

    // A failed read: not known, with the address.
    let (app, id) = app_with("http://127.0.0.1:9", &feed_xml("Album", &guid_tag())).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "not_known_failed");
    assert_eq!(
        answer["removal"]["address"],
        format!("http://127.0.0.1:9/v1/feeds/{GUID}")
    );

    // No podcast:guid: not known, and no request.
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", "")).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "not_known_no_guid");
    assert_eq!(index.requests(), 0);
}

#[tokio::test]
async fn a_retire_sets_the_switch_in_the_draft_and_the_state_follows_the_write() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");

    let (status, draft) = call(&app, "POST", &format!("{base}/retire"), None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["hidden"], true);
    assert_eq!(state(&app, id).await["ask"], "draft_only");

    let (_, diff) = call(&app, "GET", &format!("{base}/diff"), None).await;
    approve_and_write(&app, id, &diff).await;
    assert_eq!(state(&app, id).await["ask"], "written");

    // "Stop Retiring" clears the switch in a new draft, and the user writes it.
    let (status, draft) = call(&app, "DELETE", &format!("{base}/retire"), None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["hidden"], false);
    assert_eq!(state(&app, id).await["ask"], "written_cleared");
    let (_, diff) = call(&app, "GET", &format!("{base}/diff"), None).await;
    approve_and_write(&app, id, &diff).await;
    assert_eq!(state(&app, id).await["ask"], "no");
}

#[tokio::test]
async fn a_read_only_feed_gives_409_for_a_retire_and_a_delete_draft() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");
    // The delete makes the feed read-only. Then both routes refuse.
    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    approve_and_write(&app, id, &draft).await;
    let (status, body) = call(&app, "POST", &format!("{base}/retire"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
}

#[tokio::test]
async fn the_delete_draft_gives_the_summary_and_the_names_and_the_write_deletes_the_file() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");

    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let deletes = draft["summary"]["deletes"].as_array().unwrap();
    assert_eq!(deletes.len(), 1, "{draft}");
    assert_eq!(deletes[0]["path"], "a.xml");
    assert_eq!(draft["summary"]["stays"], json!([]));
    assert_eq!(draft["names"]["host_name"], "Home");
    assert_eq!(draft["names"]["path"], "a.xml");
    assert_eq!(draft["names"]["confirm_text"], "Album");
    assert_eq!(draft["names"]["publishers"], json!([]));
    assert!(app.feeds.path().join("a.xml").exists());

    // The diff route gives the same diff ID.
    let (_, again) = call(&app, "GET", &format!("{base}/diff"), None).await;
    assert_eq!(again["id"], draft["id"]);

    // A diff ID that the user did not approve writes nothing.
    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/write"),
        Some(json!({ "diff_id": "not-approved" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(app.feeds.path().join("a.xml").exists());

    approve_and_write(&app, id, &draft).await;
    assert!(!app.feeds.path().join("a.xml").exists());
    assert_eq!(state(&app, id).await["delete"], "done");

    // The restore draft brings the file back through the same write route.
    let (status, restore) = call(&app, "POST", &format!("{base}/delete/restore"), None).await;
    assert_eq!(status, StatusCode::OK, "{restore}");
    approve_and_write(&app, id, &restore).await;
    assert!(app.feeds.path().join("a.xml").exists());
    assert_eq!(state(&app, id).await["delete"], "not_done");
}

#[tokio::test]
async fn a_feed_with_no_title_gives_its_file_name_to_confirm() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("", &guid_tag())).await;
    let (status, draft) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["names"]["confirm_text"], "a.xml");
}

#[tokio::test]
async fn the_option_adds_the_files_of_the_release_to_the_deletes() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    std::fs::write(app.feeds.path().join("one.mp3"), b"audio").unwrap();
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let host = connectors[0]["id"].as_i64().unwrap();
    let (status, _) = call(&app, "POST", &format!("/api/v1/hosts/{host}/scan"), None).await;
    assert_eq!(status, StatusCode::OK);
    let base = format!("/api/v1/feeds/{id}/delete/draft");

    let (_, without) = call(&app, "POST", &base, Some(json!({ "with_files": false }))).await;
    assert_eq!(without["summary"]["deletes"].as_array().unwrap().len(), 1);
    let (status, with) = call(&app, "POST", &base, Some(json!({ "with_files": true }))).await;
    assert_eq!(status, StatusCode::OK, "{with}");
    let paths: Vec<&str> = with["summary"]["deletes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["one.mp3", "a.xml"], "ADR 0131 items 11 and 12");
}
