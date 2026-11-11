//! The routes of the feed check (ADR 0174). No test sends a request outside
//! the computer (ADR 0045). Scripted readers serve the reads.

use std::sync::Arc;

use studio_core::feed_reads::ScriptedFileReads;
use studio_core::move_link::ScriptedReader;

use super::*;

const PUBLIC: &str = "https://music.example/a.xml";
const ONE: &str = "http://files.example/one.mp3";
const TWO: &str = "http://files.example/two.mp3";

fn feed_xml(extra: &str) -> String {
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" \
         xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"><channel>\
         <title>Album</title><language>en</language>{extra}\
         <podcast:guid>ga</podcast:guid><podcast:medium>music</podcast:medium>\
         <item><guid>t-1</guid><title>One</title><itunes:duration>180</itunes:duration>\
         <enclosure url=\"{ONE}\" length=\"500\" type=\"audio/mpeg\"/></item>\
         <item><guid>t-2</guid><title>Two</title><itunes:duration>180</itunes:duration>\
         <enclosure url=\"{TWO}\" type=\"audio/mpeg\"/></item>\
         </channel></rss>"
    )
}

/// An app with a connector for the feeds folder, and a feed on it with the
/// public address `PUBLIC`. Gives the app and the ID of the feed.
async fn app_with_feed(reader: ScriptedReader, files: ScriptedFileReads) -> (App, i64) {
    app_with_scripted_index(reader, files, "http://127.0.0.1:9", &feed_xml("")).await
}

/// The same as `app_with_feed`, with the Music Index API at `index` and the
/// feed file `xml`.
async fn app_with_scripted_index(
    reader: ScriptedReader,
    files: ScriptedFileReads,
    index: &str,
    xml: &str,
) -> (App, i64) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(index)
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(Arc::new(PassAll))
        .with_stream_check(Arc::new(StubStream))
        .with_feed_reader(Arc::new(reader))
        .with_file_reads(Arc::new(files));
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

fn kinds(check: &Value) -> Vec<String> {
    check["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["kind"].as_str().unwrap().to_owned())
        .collect()
}

fn finding<'a>(check: &'a Value, kind: &str, entry: Option<&str>) -> &'a Value {
    check["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["kind"] == kind && f["entry"].as_str() == entry)
        .unwrap_or_else(|| panic!("no finding {kind} in {check}"))
}

fn same_file_files() -> ScriptedFileReads {
    ScriptedFileReads::empty()
        .with_size(ONE, 700)
        .with_file(ONE, 700, b"abc")
        .with_file("https://files.example/one.mp3", 700, b"abc")
}

#[tokio::test]
async fn the_check_route_gives_the_local_findings_and_the_reads_add_the_network_findings() {
    let (app, id) = app_with_feed(ScriptedReader::empty(), same_file_files()).await;
    let path = format!("/api/v1/feeds/{id}/check");

    let (status, check) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{check}");
    assert_eq!(kinds(&check), ["no_self_link", "no_file_size"]);
    assert_eq!(check["read"], false);
    assert_eq!(check["read_only"], false);
    let link = finding(&check, "no_self_link", None);
    assert_eq!(link["level"], "notice");
    assert_eq!(link["repair"], "add_the_link");
    assert_eq!(
        finding(&check, "no_file_size", Some("t-2"))["repair"],
        "read_the_size"
    );

    // The reads add the findings that need the network. The second track
    // has no script for its size and its file: Studio could not read it.
    let (status, read) = call(&app, "POST", &format!("{path}/reads"), None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["read"], true);
    let http = finding(&read, "http_file", Some("t-1"));
    assert_eq!(http["repair"], "use_the_https_address");
    assert_eq!(http["url"], ONE);
    let wrong = finding(&read, "wrong_file_size", Some("t-1"));
    assert_eq!(wrong["repair"], "read_the_size");
    // The index at the dead address also gives a failed read.
    assert_eq!(
        read["unreadable"],
        json!([TWO, "http://127.0.0.1:9/v1/feeds/a"])
    );

    // The facts stay in memory: a route with no read gives the same result.
    let (status, again) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{again}");
    assert_eq!(again["findings"], read["findings"]);
    assert_eq!(again["read"], true);
}

#[tokio::test]
async fn the_repair_route_changes_the_draft_and_gives_the_new_findings() {
    let reader = ScriptedReader::empty().with_feed(PUBLIC, PUBLIC, Some("ga"), None, None);
    let files = same_file_files().with_size(TWO, 4321);
    let (app, id) = app_with_feed(reader, files).await;
    let path = format!("/api/v1/feeds/{id}/check");
    let (_, read) = call(&app, "POST", &format!("{path}/reads"), None).await;
    let repair = |kind: &str, entry: Option<&str>, url: Option<&str>, revision: &Value| json!({ "kind": kind, "entry": entry, "url": url, "revision": revision });

    // Add the Link.
    let (status, done) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(repair("no_self_link", None, None, &read["revision"])),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert!(draft_bytes(&app, id).contains(&format!("href=\"{PUBLIC}\"")));
    assert!(!kinds(&done["check"]).contains(&"no_self_link".to_owned()));
    assert_eq!(done["draft"]["has_host"], true);
    assert_ne!(done["check"]["revision"], read["revision"]);

    // The old revision is stale.
    let (status, stale) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(repair("no_file_size", Some("t-2"), None, &read["revision"])),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
    assert_eq!(stale["error"]["code"], "stale_revision");

    // Read the Size.
    let revision = done["check"]["revision"].clone();
    let (status, done) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(repair("no_file_size", Some("t-2"), None, &revision)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert!(draft_bytes(&app, id).contains("length=\"4321\""));

    // Use the https Address changes only the named address.
    let revision = done["check"]["revision"].clone();
    let before = draft_bytes(&app, id);
    let (status, done) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(repair("http_file", Some("t-1"), Some(ONE), &revision)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert!(!kinds(&done["check"]).contains(&"http_file".to_owned()));
    assert_eq!(
        draft_bytes(&app, id),
        before.replace(ONE, "https://files.example/one.mp3")
    );

    // A finding that the check does not give has no repair.
    let revision = done["check"]["revision"].clone();
    let (status, none) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(repair("http_file", Some("t-2"), Some(TWO), &revision)),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{none}");
    assert_eq!(none["error"]["code"], "no_such_finding");
}

#[tokio::test]
async fn add_the_link_with_a_body_that_has_another_guid_changes_no_draft_and_gives_the_reason() {
    let reader = ScriptedReader::empty().with_feed(PUBLIC, PUBLIC, Some("other"), None, None);
    let (app, id) = app_with_feed(reader, ScriptedFileReads::empty()).await;
    let path = format!("/api/v1/feeds/{id}/check");
    let (_, check) = call(&app, "GET", &path, None).await;
    let before = draft_bytes(&app, id);
    let (status, refused) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(json!({ "kind": "no_self_link", "revision": check["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["error"]["code"], "repair_refused");
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("podcast:guid"),
        "{refused}"
    );
    assert_eq!(draft_bytes(&app, id), before);
}

#[tokio::test]
async fn a_read_only_feed_gives_its_findings_with_no_repair_and_the_repair_route_refuses_it() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let path = format!("/api/v1/feeds/{id}/check");
    let (status, check) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{check}");
    assert_eq!(check["read_only"], true);
    let found = check["findings"].as_array().unwrap();
    assert!(found.iter().any(|f| f["kind"] == "no_self_link"), "{check}");
    assert!(found.iter().all(|f| f["repair"].is_null()), "{check}");

    let (status, body) = call(
        &app,
        "POST",
        &format!("{path}/repair"),
        Some(json!({ "kind": "no_self_link", "revision": check["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
}

#[tokio::test]
async fn the_reads_run_with_no_lock_of_the_workspace() {
    let reader = ScriptedReader::empty().with_feed(
        "https://other.example/f.xml",
        "https://other.example/f.xml",
        Some("ga"),
        None,
        Some("https://other.example/f.xml"),
    );
    let (app, connector, free) = app_with_lock_probe(reader).await;
    let tag = "<itunes:new-feed-url>https://other.example/f.xml</itunes:new-feed-url>";
    let id = add_paid_feed(&app, connector, "a.xml", &feed_xml(tag)).await;
    let (status, read) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/check/reads"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert!(
        free.load(std::sync::atomic::Ordering::SeqCst),
        "the read held the lock of the workspace"
    );
}

const INDEX_GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";

fn index_feed_xml() -> String {
    feed_xml("").replace("<podcast:guid>ga<", &format!("<podcast:guid>{INDEX_GUID}<"))
}

#[tokio::test]
async fn the_reads_give_the_two_index_findings_the_report_and_the_line_from_a_scripted_index() {
    let server = studio_core::testing::TestServer::start(vec![
        (
            "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
            200,
            br#"{"data":{"feed_url":"https://old.example/a.xml","title":"A","last_build_date":1}}"#
                .to_vec(),
        ),
        (
            "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/copies",
            200,
            br#"{"data":[
                {"url":"https://mirror.example/a.xml","open":true,"differs_recipients":true},
                {"url":"https://same.example/a.xml","open":true,"differs_recipients":false}
            ],"pagination":{"cursor":null,"has_more":false}}"#
                .to_vec(),
        ),
    ]);
    let (app, id) = app_with_scripted_index(
        ScriptedReader::empty(),
        same_file_files(),
        &server.url(""),
        &index_feed_xml(),
    )
    .await;
    let path = format!("/api/v1/feeds/{id}/check");

    // No read yet: no index finding and no line.
    let (_, before) = call(&app, "GET", &path, None).await;
    assert!(!kinds(&before).contains(&"index_address_differs".to_owned()));
    assert_eq!(before["index_line"], Value::Null);

    let (status, read) = call(&app, "POST", &format!("{path}/reads"), None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    let differs = finding(&read, "index_address_differs", None);
    assert_eq!(differs["url"], "https://old.example/a.xml");
    assert_eq!(differs["level"], "notice");
    let copy = finding(&read, "copy_other_recipients", None);
    assert_eq!(copy["level"], "warning");
    assert_eq!(copy["url"], "https://mirror.example/a.xml");
    let report = copy["report"].as_str().unwrap();
    assert!(report.contains(INDEX_GUID) && report.contains(PUBLIC));
    assert!(report.contains("https://mirror.example/a.xml"));
    assert_eq!(
        kinds(&read)
            .iter()
            .filter(|k| *k == "copy_other_recipients")
            .count(),
        1
    );
    assert!(finding(&read, "no_self_link", None).get("report").is_none());
    assert_eq!(read["index_line"], Value::Null);
    assert_eq!(read["unreadable"], json!([TWO]));

    // The facts stay in memory.
    let (_, again) = call(&app, "GET", &path, None).await;
    assert_eq!(again["findings"], read["findings"]);
}

#[tokio::test]
async fn an_unknown_feed_gives_the_line_and_a_failed_index_read_gives_the_address() {
    // A 404: the index does not hold the feed.
    let server = studio_core::testing::TestServer::start(Vec::new());
    let (app, id) = app_with_scripted_index(
        ScriptedReader::empty(),
        same_file_files(),
        &server.url(""),
        &index_feed_xml(),
    )
    .await;
    let path = format!("/api/v1/feeds/{id}/check/reads");
    let (status, read) = call(&app, "POST", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(
        read["index_line"],
        "Music Index does not hold this feed yet."
    );
    assert!(!kinds(&read).contains(&"index_address_differs".to_owned()));

    // A refused connection: a failed read with the request address.
    let (app, id) = app_with_scripted_index(
        ScriptedReader::empty(),
        same_file_files(),
        "http://127.0.0.1:9",
        &index_feed_xml(),
    )
    .await;
    let (status, read) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/check/reads"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    let address = format!("http://127.0.0.1:9/v1/feeds/{INDEX_GUID}");
    assert!(
        read["unreadable"]
            .as_array()
            .unwrap()
            .contains(&json!(address))
    );
    assert_eq!(read["index_line"], Value::Null);
}
