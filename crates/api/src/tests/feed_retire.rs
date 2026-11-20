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
         <podcast:publisher><podcast:remoteItem medium=\"publisher\" rel=\"artist\" feedGuid=\"artist-1\" feedUrl=\"https://music.example/artist.xml\"/></podcast:publisher>\
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

#[tokio::test]
async fn return_wait_route_reports_the_index_state_and_skips_feeds_without_a_guid() {
    let index = CountingIndex::start(200, r#"{"data":{"return_wait_until":4102444800}}"#);
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let (status, answer) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/return-wait"),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0190 item 1: return the warning state; fix the route response."
    );
    assert_eq!(
        answer["state"], "waits",
        "ADR 0190 item 2: report a future wait; fix time classification."
    );
    assert_eq!(
        answer["until"], 4_102_444_800_u64,
        "ADR 0190 item 2: give the index time; fix the response time."
    );
    assert_eq!(
        answer["too_far"], true,
        "ADR 0190 item 7: flag a wait beyond the send horizon; fix the horizon check."
    );

    let index = CountingIndex::start(200, r#"{"data":{"return_wait_until":null}}"#);
    let (app, id) = app_with(&index.url(), &feed_xml("Album", "")).await;
    let (status, answer) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/return-wait"),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0190 item 1: answer when a feed has no GUID; fix the route response."
    );
    assert_eq!(
        answer["state"], "unknown",
        "ADR 0190 item 4: report unknown when no GUID exists; fix the no-GUID state."
    );
    assert_eq!(
        index.requests(),
        0,
        "ADR 0190 item 1: skip the index read without a GUID; fix the route read."
    );
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
async fn the_retire_state_and_delete_draft_read_the_index_once() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");
    let other_routes = [
        ("POST", format!("{base}/retire"), None),
        ("DELETE", format!("{base}/retire"), None),
        ("POST", format!("{base}/delete/restore"), None),
        ("GET", format!("{base}/diff"), None),
        ("GET", format!("{base}/check"), None),
    ];
    for (method, path, body) in other_routes {
        let _ = call(&app, method, &path, body).await;
        assert_eq!(
            index.requests(),
            0,
            "ADR 0200 item 7: {method} {path} sent an index read"
        );
    }
    let _ = call(&app, "DELETE", &format!("{base}/draft"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(index.requests(), 1, "ADR 0200 item 7");
    let answer = state(&app, id).await;
    assert_eq!(index.requests(), 2, "ADR 0200 item 7");
    assert_eq!(answer["removal"]["answer"], "removed");
    assert_eq!(answer["removal_done"], true);
}

#[tokio::test]
async fn delete_draft_refuses_a_held_feed_and_a_failed_index_read() {
    // Before the guard, the held-feed case started a draft and returned 200.
    let index = CountingIndex::start(
        200,
        r#"{"data":{"feed_url":"https://music.example/a.xml"}}"#,
    );
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "index_holds_feed");

    let (app, id) = app_with("http://127.0.0.1:9", &feed_xml("Album", &guid_tag())).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "index_holds_feed");
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

// After the write of a restore, the file on the host hides the feed again.
// Step 1 of the Retire block then names the last write, not the draft
// (ADR 0198 item 11).
#[tokio::test]
async fn after_a_restore_write_the_retire_state_names_the_last_write() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let base = format!("/api/v1/feeds/{id}");
    let (status, _) = call(&app, "POST", &format!("{base}/retire"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, diff) = call(&app, "GET", &format!("{base}/diff"), None).await;
    approve_and_write(&app, id, &diff).await;
    assert_eq!(state(&app, id).await["ask"], "written");

    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    approve_and_write(&app, id, &draft).await;
    assert_eq!(state(&app, id).await["delete"], "done");

    let (status, restore) = call(&app, "POST", &format!("{base}/delete/restore"), None).await;
    assert_eq!(status, StatusCode::OK, "{restore}");
    approve_and_write(&app, id, &restore).await;
    assert!(app.feeds.path().join("a.xml").exists());
    let restored = state(&app, id).await;
    assert_eq!(restored["delete"], "not_done");
    assert_eq!(restored["ask"], "written", "{restored}");
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

#[tokio::test]
async fn feed_rows_report_a_completed_host_delete() {
    let index = CountingIndex::start(404, "{}");
    let (app, deleted_id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let host = connectors[0]["id"].as_i64().unwrap();
    let other_id = add_paid_feed(&app, host, "other.xml", &feed_xml("Other", "")).await;

    let (status, draft) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{deleted_id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0198 item 7: start the feed delete. Fix the delete draft route. {draft}"
    );
    let (status, diff) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{deleted_id}/diff"),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0213 item 22: make a diff for a feed-file deletion; fix the diff route. {diff}"
    );
    assert!(
        diff["page_refusal"].is_null(),
        "ADR 0213 item 22: give no refusal for a feed-file deletion; fix the diff route. {diff}"
    );
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{deleted_id}/write"),
        Some(json!({ "diff_id": draft["id"] })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0198 item 7: finish the feed delete. Fix the write route. {written}"
    );

    let (status, rows) = call(&app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0198 item 7: return the feed list. Fix GET /feeds."
    );
    let listed_deleted = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == deleted_id)
        .unwrap();
    let listed_other = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == other_id)
        .unwrap();
    assert_eq!(
        listed_deleted["host_deleted"], true,
        "ADR 0198 item 7: include the delete state in the feed list. Add it to FeedRow JSON."
    );
    assert_eq!(
        listed_other["host_deleted"], false,
        "ADR 0198 item 7: mark an undeleted feed as not deleted. Set host_deleted per feed."
    );
}

#[tokio::test]
async fn a_guid_at_another_address_is_a_leftover_copy_and_the_delete_draft_passes() {
    // ADR 0228 item 20: the index reads the feed from another address.
    let index = CountingIndex::start(
        200,
        r#"{"data":{"feed_url":"https://other.example/a.xml"}}"#,
    );
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "leftover_copy", "{answer}");
    assert_eq!(
        answer["removal"]["index_url"],
        "https://other.example/a.xml"
    );
    assert_eq!(answer["removal_done"], true);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // The address of this file keeps the refusal (ADR 0200 item 6).
    let index = CountingIndex::start(
        200,
        r#"{"data":{"feed_url":"HTTPS://Music.Example/a.xml"}}"#,
    );
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    let answer = state(&app, id).await;
    assert_eq!(answer["removal"]["answer"], "not_yet", "{answer}");
    assert_eq!(answer["removal_done"], false);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/delete/draft"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

/// The paths of the file list of a feed.
async fn file_paths(app: &App, id: i64) -> Vec<String> {
    let (status, files) = call(app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    let rows = files
        .as_array()
        .or_else(|| files["files"].as_array())
        .unwrap_or_else(|| panic!("the file list has an unknown shape: {files}"));
    let mut paths: Vec<String> = rows
        .iter()
        .map(|row| row["path"].as_str().unwrap_or_default().to_owned())
        .collect();
    paths.sort();
    paths
}

async fn feed_row(app: &App, id: i64) -> Value {
    let (_, rows) = call(app, "GET", "/api/v1/feeds", None).await;
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
        .clone()
}

// A plain 404 of the index lets the delete pass for a feed that is not
// hidden. The deleted feed keeps its media rows, takes no notice of a host,
// and gets its path again after the restore write (ADR 0230 items 28 to 30).
#[tokio::test]
async fn a_deleted_feed_makes_no_remote_rows_and_the_restore_gives_its_path_again() {
    let index = CountingIndex::start(404, "{}");
    let (app, id) = app_with(&index.url(), &feed_xml("Album", &guid_tag())).await;
    // A local library file of the release, as an import makes it. It
    // replaces the remote row that the first read made for the enclosure.
    {
        let collection =
            studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
                .unwrap();
        for file in collection
            .library_files(studio_core::collection::FeedId(id))
            .unwrap()
        {
            collection.remove_library_file(file.id).unwrap();
        }
    }
    plant_library_file(&app, id, "album", "one.mp3", b"audio");
    let base = format!("/api/v1/feeds/{id}");
    let before = file_paths(&app, id).await;
    let original = feed_row(&app, id).await;
    assert_eq!(original["host_deleted"], false);

    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/delete/draft"),
        Some(json!({ "with_files": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    approve_and_write(&app, id, &draft).await;

    let deleted = feed_row(&app, id).await;
    assert_eq!(deleted["host_deleted"], true);
    assert_eq!(
        deleted["host_for_address"], false,
        "ADR 0230 item 28: a feed with a done delete has no host for its address. Fix host_for_address in crates/api/src/routes.rs."
    );
    assert_eq!(state(&app, id).await["delete"], "done");
    let after = file_paths(&app, id).await;
    assert_eq!(
        after, before,
        "ADR 0230 item 29: the delete must not add remote rows. Skip feeds with a done delete in record_remote_files_with in crates/api/src/workspace.rs."
    );
    assert!(
        after.iter().all(|path| !path.contains(" 2")),
        "ADR 0230 item 29: no row has ' 2' in its name. Fix record_remote_files_with in crates/api/src/workspace.rs. {after:?}"
    );

    let (status, restore) = call(&app, "POST", &format!("{base}/delete/restore"), None).await;
    assert_eq!(status, StatusCode::OK, "{restore}");
    approve_and_write(&app, id, &restore).await;
    let restored = feed_row(&app, id).await;
    assert_eq!(restored["host_deleted"], false);
    assert_eq!(
        restored["connector_id"], original["connector_id"],
        "ADR 0230 item 30: the restore write gives the feed its host again."
    );
    assert_eq!(
        restored["path"], original["path"],
        "ADR 0230 item 30: the restore write gives the feed its path again."
    );
    assert!(
        file_paths(&app, id).await.contains(&"one.mp3".to_owned()),
        "ADR 0230 item 30: after the restore the file list keeps the original row."
    );
}
