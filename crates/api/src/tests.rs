//! Contract tests of the local API (task 007).

mod apps;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{Access, Workspace, local_router};

const TOKEN: &str = "test-token";
const HOST: &str = "127.0.0.1:4100";
const PASSPHRASE: &str = "correct horse battery";
const FEED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
    <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
    <item><guid>t-1</guid></item></channel></rss>";
/// A feed with a channel `podcast:value` block, for the recipient list test.
const FEED_WITH_RECIPIENTS: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
    <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
    <podcast:value type=\"lightning\" method=\"keysend\">\
    <podcast:valueRecipient name=\"Ann\" address=\"ann@example.com\" split=\"90\" type=\"lnaddress\"/>\
    <podcast:valueRecipient name=\"Bob\" address=\"bob@example.com\" split=\"10\" type=\"lnaddress\"/>\
    </podcast:value>\
    <item><guid>t-1</guid></item></channel></rss>";

/// An address check that passes each address, so that no test sends a
/// request to the network.
struct PassAll;

impl studio_core::pipeline::AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

/// The lookup relay of a test that does not read a key list. Port 9
/// answers nothing. No test uses the default lookup relays.
const UNUSED_RELAY: &str = "ws://127.0.0.1:9";

struct App {
    router: Router,
    data: tempfile::TempDir,
    feeds: tempfile::TempDir,
}

fn app() -> App {
    app_with_index("http://127.0.0.1:9")
}

/// An app whose musicindex API is at `index`. Port 9 answers nothing.
fn app_with_index(index: &str) -> App {
    app_with_keysend(index, None)
}

/// An app with a scheme for the keysend lookups. `None` sends no lookup.
fn app_with_keysend(index: &str, keysend: Option<&'static str>) -> App {
    app_with_reader(index, keysend, None)
}

/// [`app_with_keysend`] with a reader of public feeds. The reader serves the
/// destination of a move in stage 2 of ADR 0126.
fn app_with_reader(
    index: &str,
    keysend: Option<&'static str>,
    reader: Option<studio_core::move_link::ScriptedReader>,
) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let mut workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(index)
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(keysend)
        .with_address_check(std::sync::Arc::new(PassAll));
    if let Some(reader) = reader {
        workspace = workspace.with_feed_reader(std::sync::Arc::new(reader));
    }
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

/// An app with the rules of the server (task 034d). The user folder is the
/// data folder, and the library folder is `library/` in it.
fn app_with_rules(storage_limit: u64) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_server_rules(crate::ServerRules {
            storage_limit,
            folder: data.path().to_path_buf(),
            library_root: data.path().join("library"),
            hold_assets: true,
            vault_idle: std::time::Duration::from_secs(600),
            vault_idle_max: std::time::Duration::from_secs(3600),
        })
        .unwrap();
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

async fn send(app: &App, request: Request<Body>) -> (StatusCode, Value, String) {
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, value, text)
}

fn request(method: &str, path: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, HOST)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"));
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn call(app: &App, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let (status, value, _) = send(app, request(method, path, body)).await;
    (status, value)
}

/// Send a request and give its raw body, for a response that is not JSON.
async fn send_raw(
    app: &App,
    request: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, headers, bytes)
}

/// Percent-encode a value for a query string, for a test URL that has a
/// scheme, a host and a port.
fn url_encode(value: &str) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

/// Give the workspace a library folder, through a second connection to
/// the collection file (ADR 0036 item 1). Task 021f adds the route that
/// sets this folder through the API.
fn set_library_root(app: &App) {
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let root = app.data.path().join("library");
    collection
        .set_setting("library_root", &json!(root.to_str().unwrap()))
        .unwrap();
}

/// Make the vault, a local connector, a feed and a library folder. Gives
/// the feed ID.
async fn set_up(app: &App) -> i64 {
    let (status, _) = call(
        app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let root = app.feeds.path().to_str().unwrap();
    let (status, connector) = call(
        app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, feed) = call(
        app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    set_library_root(app);
    feed["id"].as_i64().unwrap()
}

/// The path of a test media file of task 018b, in `crates/core/tests/media`.
fn media_path(file: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/media")
        .join(file)
}

/// Send one file to a feed in parts, and finish the upload (ADR 0059).
/// Gives the status and the body of the first failed step, or of `finish`.
async fn upload(app: &App, id: i64, name: &str, bytes: Vec<u8>) -> (StatusCode, Value) {
    upload_with(app, id, name, bytes, false).await
}

/// Like [`upload`], with the `replace` flag of the start.
async fn upload_with(
    app: &App,
    id: i64,
    name: &str,
    bytes: Vec<u8>,
    replace: bool,
) -> (StatusCode, Value) {
    let (status, started) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/uploads"),
        Some(json!({ "name": name, "size": bytes.len(), "replace": replace })),
    )
    .await;
    if status != StatusCode::CREATED {
        return (status, started);
    }
    let upload = started["upload"].as_str().unwrap().to_owned();
    let part_size = usize::try_from(started["part_size"].as_u64().unwrap()).unwrap();
    for (n, part) in bytes.chunks(part_size).enumerate() {
        let (status, body) = put_part(app, &upload, n, part.to_vec()).await;
        if status != StatusCode::OK {
            return (status, body);
        }
    }
    call(
        app,
        "POST",
        &format!("/api/v1/uploads/{upload}/finish"),
        None,
    )
    .await
}

/// Send one part of an upload.
async fn put_part(app: &App, upload: &str, n: usize, bytes: Vec<u8>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/uploads/{upload}/parts/{n}"))
        .header(header::HOST, HOST)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::from(bytes))
        .unwrap();
    let (status, body, _) = send(app, request).await;
    (status, body)
}

/// True when the temporary upload folder has no file, or does not exist
/// (task 018c).
fn incoming_is_empty(app: &App) -> bool {
    match std::fs::read_dir(app.data.path().join("incoming")) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    }
}

#[tokio::test]
async fn status_gives_the_vault_state_and_the_counts() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/status", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["vault"], "absent");
    assert_eq!(body["connectors"], 0);
    assert_eq!(body["feeds"], 0);
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn the_vault_is_made_locked_and_unlocked() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE, "recovery_key": true })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["recovery_key"].as_str().unwrap().len(), 39);
    assert!(app.data.path().join("vault.json").exists());

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_exists");

    let (_, body) = call(&app, "POST", "/api/v1/vault/lock", None).await;
    assert_eq!(body["vault"], "locked");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": "not the passphrase" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "wrong_key");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["vault"], "open");
}

#[tokio::test]
async fn the_strength_route_gives_the_score_and_no_part_of_the_passphrase() {
    let app = app();
    let (status, body, text) = send(
        &app,
        request(
            "POST",
            "/api/v1/vault/strength",
            Some(json!({ "passphrase": "monkeymonkey1" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["acceptable"], false);
    assert_eq!(body["chars"], 13);
    assert!(body["score"].as_u64().unwrap() < 3);
    assert!(!text.contains("monkey"));
    let (_, body) = call(
        &app,
        "POST",
        "/api/v1/vault/strength",
        Some(json!({ "passphrase": "Kx9#mQ2!vL7@pR4z" })),
    )
    .await;
    assert_eq!(body["acceptable"], true);
    assert_eq!(body["score"], 4);
}

#[tokio::test]
async fn a_weak_passphrase_gives_422() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": "passwordpassword" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "passphrase_too_weak");
    assert!(!app.data.path().join("vault.json").exists());
}

#[tokio::test]
async fn the_passphrase_route_replaces_the_passphrase_and_applies_the_vault_rule() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": "staple orange umbrella river" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_required");
    call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": "passwordpassword" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "passphrase_too_weak");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": "short" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "passphrase_too_short");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": "staple orange umbrella river" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["vault"], "open");
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": "staple orange umbrella river" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn a_short_passphrase_gives_422() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": "short" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "passphrase_too_short");
}

#[tokio::test]
async fn connectors_and_feeds_are_added_listed_and_read() {
    let app = app();
    let id = set_up(&app).await;

    let (status, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(connectors[0]["kind"], "local");
    assert_eq!(connectors[0]["name"], "Home");

    let (status, feeds) = call(&app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feeds[0]["path"], "album.xml");
    assert_eq!(feeds[0]["public_url"], "https://music.example/album.xml");
    assert_eq!(feeds[0]["title"], Value::Null);

    let (status, feed) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["title"], "Album");
    assert_eq!(feed["podcast_guid"], "g-1");
    assert_eq!(feed["medium"], "music");
    assert!(
        feed["last_version"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
}

#[tokio::test]
async fn the_host_lists_its_feed_files_and_marks_the_ones_in_the_collection() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("single.xml"),
        FEED.replace("Album", "Single"),
    )
    .unwrap();
    std::fs::write(app.feeds.path().join("notes.txt"), "not a feed").unwrap();
    set_up(&app).await;
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let path = format!("/api/v1/connectors/{}/feeds", connectors[0]["id"]);
    let (status, body) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["truncated"], false);
    let feeds = body["feeds"].as_array().unwrap();
    assert_eq!(feeds.len(), 2);
    assert_eq!(feeds[0]["path"], "album.xml");
    assert_eq!(feeds[0]["in_collection"], true);
    assert_eq!(feeds[1]["path"], "single.xml");
    assert_eq!(feeds[1]["title"], "Single");
    assert_eq!(feeds[1]["medium"], "music");
    assert_eq!(feeds[1]["in_collection"], false);
}

#[tokio::test]
async fn a_folder_that_starts_with_a_tilde_is_in_the_home_folder() {
    let app = app();
    let (status, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": "~" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let config: Value = serde_json::from_str(connector["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["root"], std::env::var("HOME").unwrap());
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "x", "root": "~/no-such-folder-for-studio-tests" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "folder_not_found");
    let home = std::env::var("HOME").unwrap();
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with(home.trim_end_matches('/'))
    );
}

#[tokio::test]
async fn a_relative_folder_and_a_half_feed_give_422() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "x", "root": "relative/dir" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_config");
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "path": "a.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_new_release_is_made_as_a_draft_with_no_hosted_file_and_then_written() {
    let app = app();
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    set_library_root(&app);
    let (status, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({
            "connector_id": connector["id"],
            "path": "my-album.xml",
            "title": "New Album",
            "author": "New Artist",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(feed["title"], "New Album");
    assert_eq!(feed["medium"], "music");
    assert_eq!(feed["public_url"], "https://music.example/my-album.xml");
    assert!(!app.feeds.path().join("my-album.xml").exists());

    let draft_path = format!("/api/v1/feeds/{}/draft", feed["id"]);
    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(draft["model"]["title"], "New Album");
    assert_eq!(draft["model"]["author"], "New Artist");
    assert_eq!(draft["model"]["medium"], "music");
    assert_eq!(draft["model"]["language"], "en");
    assert_eq!(
        draft["model"]["podcast_guid"],
        studio_core::guid::podcast_guid_for_url("https://music.example/my-album.xml")
    );

    let (_, diff) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{}/diff", feed["id"]),
        None,
    )
    .await;
    assert!(diff["unified"].as_str().unwrap().contains("+<rss"));
    // The diff also gives the field changes, the lines and the guard count
    // (ADR 0114).
    let title_group = diff["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["name"] == "title")
        .expect("the new feed adds the title");
    assert_eq!(title_group["kind"], "field");
    assert_eq!(title_group["changes"][0]["new"], "New Album");
    let lines = diff["lines"].as_array().unwrap();
    assert_ne!(lines.len(), 0);
    assert!(lines.iter().all(|line| line["kind"] == "added"));
    assert!(diff["uncovered"].is_u64());

    // On 2026-10-23 the review of task 018d found that a discard of this
    // draft left a release that Studio could not edit: no draft and no
    // hosted file. The draft of a release with no hosted file stays.
    let (status, body) = call(&app, "DELETE", &draft_path, None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "no_hosted_file");
    let (status, _) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{}/write", feed["id"]),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(written["title"], "New Album");
    assert!(app.feeds.path().join("my-album.xml").exists());
}

#[tokio::test]
async fn a_new_release_on_a_folder_with_no_public_address_gets_a_placeholder_guid_409_for_a_file_and_422_for_an_empty_field()
 {
    let app = app();
    let root = app.feeds.path().to_str().unwrap();
    let (_, no_address) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "No address", "root": root })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({ "connector_id": no_address["id"], "path": "a.xml", "title": "A", "author": "B" })),
    )
    .await;
    // ADR 0122 item 5: the feed is made, with no public address and a
    // placeholder GUID.
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["public_url"].is_null());
    let (_, draft) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{}/draft", body["id"]),
        None,
    )
    .await;
    let guid = draft["model"]["podcast_guid"].as_str().unwrap_or_default();
    assert!(guid.starts_with("pending-"), "{draft}");

    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    std::fs::write(app.feeds.path().join("taken.xml"), "<rss/>").unwrap();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({ "connector_id": connector["id"], "path": "taken.xml", "title": "A", "author": "B" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "file_exists");

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({ "connector_id": connector["id"], "path": "empty.xml", "title": "  ", "author": "B" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_feed");
}

#[tokio::test]
async fn a_placeholder_guid_feed_gets_its_real_guid_when_its_folder_gets_a_public_address() {
    let app = app();
    let root = app.feeds.path().to_str().unwrap().to_owned();
    let (_, folder) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Folder", "root": root })),
    )
    .await;
    set_library_root(&app);
    let (status, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({ "connector_id": folder["id"], "path": "a.xml", "title": "A", "author": "B" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = feed["id"].as_i64().unwrap();
    let listed = |rows: &Value| {
        rows.as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()
            .clone()
    };

    let (_, rows) = call(&app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(listed(&rows)["placeholder_guid"], true);
    // Feed facts shows the placeholder before the first write.
    assert!(
        listed(&rows)["podcast_guid"]
            .as_str()
            .is_some_and(|guid| guid.starts_with("pending-")),
        "the feed row gives the placeholder GUID of the draft"
    );

    // ADR 0122 item 8: the folder gets a public address, and the feed gets
    // its real GUID in its draft.
    let (status, _) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{}", folder["id"]),
        Some(json!({ "base_url": "https://music.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, rows) = call(&app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(listed(&rows)["placeholder_guid"], false);
    assert_eq!(listed(&rows)["public_url"], "https://music.example/a.xml");
    let (_, draft) = call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(
        draft["model"]["podcast_guid"],
        studio_core::guid::podcast_guid_for_url("https://music.example/a.xml")
    );
}

#[tokio::test]
async fn the_practice_publish_route_is_gone() {
    let app = app();
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/practice/publish",
        Some(json!({ "from": 1, "to": 2 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_bucket_with_no_public_address_gives_a_placeholder_guid_for_a_new_release() {
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    let app = app();
    make_vault(&app).await;
    let mut body = s3_body(&server);
    body.as_object_mut().unwrap().remove("base_url");
    let (status, connector) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{connector}");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({ "connector_id": connector["id"], "path": "a.xml", "title": "A", "author": "B" })),
    )
    .await;
    // ADR 0122 item 5: any file host with no public address gives a
    // placeholder GUID.
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert!(body["public_url"].is_null());
    let (_, draft) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{}/draft", body["id"]),
        None,
    )
    .await;
    let guid = draft["model"]["podcast_guid"].as_str().unwrap_or_default();
    assert!(guid.starts_with("pending-"), "{draft}");
}

/// An app with a connected folder at a public address, and a library
/// folder. Gives the app and the connector ID.
async fn publisher_app() -> (App, i64) {
    let app = app();
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
    (app, connector["id"].as_i64().unwrap())
}

/// The body of `POST /feeds/new`, with its optional medium and role.
fn new_feed_body(
    connector: i64,
    path: &str,
    title: &str,
    medium: Option<&str>,
    role: Option<&str>,
) -> Value {
    let mut body = json!({
        "connector_id": connector,
        "path": path,
        "title": title,
        "author": title,
    });
    if let Some(medium) = medium {
        body["medium"] = json!(medium);
    }
    if let Some(role) = role {
        body["role"] = json!(role);
    }
    body
}

async fn make_feed(app: &App, body: Value) -> Value {
    let (status, feed) = call(app, "POST", "/api/v1/feeds/new", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    feed
}

/// The model of the draft of a feed. The route starts the draft when the
/// feed has none.
async fn draft_model(app: &App, id: i64) -> Value {
    let (status, draft) = call(app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    draft["model"].clone()
}

#[tokio::test]
async fn a_publisher_feed_is_made_with_its_role_and_a_bad_medium_or_role_gives_422() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    assert_eq!(label["medium"], "publisher");
    assert_eq!(label["publisher_roles"], json!(["label"]));
    assert_eq!(label["roles_confirmed"], true);
    assert_eq!(
        draft_model(&app, label["id"].as_i64().unwrap()).await["medium"],
        "publisher"
    );

    // Hosting is a first-class role of the form (ADR 0083 item 3).
    let hosting = make_feed(
        &app,
        new_feed_body(
            connector,
            "hosting.xml",
            "Host",
            Some("publisher"),
            Some("hosting"),
        ),
    )
    .await;
    assert_eq!(hosting["publisher_roles"], json!(["hosting"]));

    // A music feed ignores the role.
    let music = make_feed(
        &app,
        new_feed_body(connector, "music.xml", "Music", None, Some("band")),
    )
    .await;
    assert_eq!(music["medium"], "music");
    assert_eq!(music["publisher_roles"], json!([]));

    for (path, medium, role) in [
        ("p1.xml", Some("publisher"), None),
        ("p2.xml", Some("publisher"), Some("band")),
        ("p5.xml", Some("publisher"), Some("producer")),
        ("p3.xml", Some("video"), None),
    ] {
        let body = new_feed_body(connector, path, "Label", medium, role);
        let (status, response) = call(&app, "POST", "/api/v1/feeds/new", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path}");
        assert_eq!(response["error"]["code"], "invalid_feed", "{path}");
    }
    let (status, response) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(new_feed_body(
            connector,
            "p4.xml",
            " ",
            Some("publisher"),
            Some("label"),
        )),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("publisher feed")
    );
    // A refusal makes no file and no feed.
    assert!(!app.feeds.path().join("p1.xml").exists());
}

#[tokio::test]
async fn the_feed_list_gives_the_link_of_each_feed() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let release = make_feed(
        &app,
        new_feed_body(connector, "release.xml", "Release", None, None),
    )
    .await;
    let label_id = label["id"].as_i64().unwrap();
    let release_id = release["id"].as_i64().unwrap();
    assert_eq!(listed(&app, release_id).await["publisher_ids"], json!([]));
    let path = format!("/api/v1/feeds/{label_id}/links");
    let (status, body) = call(&app, "POST", &path, Some(json!({ "release": release_id }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // ADR 0085 item 2: the core gives the links of each feed.
    assert_eq!(
        listed(&app, label_id).await["release_ids"],
        json!([release_id])
    );
    assert_eq!(
        listed(&app, release_id).await["publisher_ids"],
        json!([label_id])
    );
    // ADR 0087 item 1: the release gives the role of the label feed.
    assert_eq!(listed(&app, label_id).await["roles_linked"], json!(true));
}

#[tokio::test]
async fn a_link_changes_both_drafts_and_a_role_change_and_a_removal_follow_it() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let release = make_feed(
        &app,
        new_feed_body(connector, "release.xml", "Release", None, None),
    )
    .await;
    let label_id = label["id"].as_i64().unwrap();
    let release_id = release["id"].as_i64().unwrap();
    let label_guid = draft_model(&app, label_id).await["podcast_guid"].clone();
    let release_guid = draft_model(&app, release_id).await["podcast_guid"].clone();

    let link_path = format!("/api/v1/feeds/{label_id}/links");
    let (status, body) = call(
        &app,
        "POST",
        &link_path,
        Some(json!({ "release": release_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["changed"],
        json!([{ "id": label_id, "title": "Label" }, { "id": release_id, "title": "Release" }])
    );

    let label_model = draft_model(&app, label_id).await;
    let label_item = &label_model["remote_items"][0];
    assert_eq!(label_item["medium"], "music");
    assert_eq!(label_item["rel"], "label");
    assert_eq!(label_item["feed_guid"], release_guid);
    assert_eq!(label_item["feed_url"], "https://music.example/release.xml");

    let release_model = draft_model(&app, release_id).await;
    let release_item = &release_model["publisher"]["remote_items"][0];
    assert_eq!(release_item["medium"], "publisher");
    assert_eq!(release_item["rel"], "label");
    assert_eq!(release_item["feed_guid"], label_guid);
    assert_eq!(release_item["feed_url"], "https://music.example/label.xml");

    let role_path = format!("/api/v1/feeds/{label_id}/role");
    let (status, body) = call(
        &app,
        "PUT",
        &role_path,
        Some(json!({ "roles": ["artist"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["changed"],
        json!([{ "id": label_id, "title": "Label" }, { "id": release_id, "title": "Release" }])
    );
    assert_eq!(
        draft_model(&app, label_id).await["remote_items"][0]["rel"],
        "artist"
    );
    assert_eq!(
        draft_model(&app, release_id).await["publisher"]["remote_items"][0]["rel"],
        "artist"
    );
    // A custom set writes its roles in their sequence (ADR 0082 item 8).
    let (status, body) = call(
        &app,
        "PUT",
        &role_path,
        Some(json!({ "roles": ["producer", "artist"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        draft_model(&app, release_id).await["publisher"]["remote_items"][0]["rel"],
        "artist producer"
    );

    let (status, body) = call(
        &app,
        "DELETE",
        &format!("/api/v1/feeds/{label_id}/links/{release_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["changed"],
        json!([{ "id": label_id, "title": "Label" }, { "id": release_id, "title": "Release" }])
    );
    assert_eq!(draft_model(&app, label_id).await["remote_items"], json!([]));
    assert!(draft_model(&app, release_id).await["publisher"].is_null());

    // A role of a feed with no link changes no other draft.
    let (status, body) = call(&app, "PUT", &role_path, Some(json!({ "roles": ["label"] }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["changed"], json!([]));
    let (status, body) = call(&app, "PUT", &role_path, Some(json!({ "roles": ["band"] }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_feed");
}

#[tokio::test]
async fn a_link_starts_the_draft_of_a_written_release_with_no_draft() {
    let app = app();
    let release_id = set_up(&app).await;
    // The medium of a feed comes from its last read.
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{release_id}/refresh"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let connector = connectors[0]["id"].as_i64().unwrap();
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let label_id = label["id"].as_i64().unwrap();
    let album_before = std::fs::read(app.feeds.path().join("album.xml")).unwrap();

    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{label_id}/links"),
        Some(json!({ "release": release_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["changed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|feed| feed["id"] == release_id),
        "{body}"
    );
    let release_model = draft_model(&app, release_id).await;
    assert_eq!(
        release_model["publisher"]["remote_items"][0]["rel"],
        "label"
    );
    assert_eq!(
        std::fs::read(app.feeds.path().join("album.xml")).unwrap(),
        album_before,
        "a draft does not write the hosted file"
    );
}

#[tokio::test]
async fn a_link_gives_422_for_a_music_feed_and_409_for_a_read_only_feed() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let music = make_feed(
        &app,
        new_feed_body(connector, "release.xml", "Release", None, None),
    )
    .await;
    let label_id = label["id"].as_i64().unwrap();
    let music_id = music["id"].as_i64().unwrap();

    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{music_id}/links"),
        Some(json!({ "release": label_id })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "not_publisher");

    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{music_id}/role"),
        Some(json!({ "roles": ["artist"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "not_publisher");

    let (_, read_only) = call(&app, "POST", "/api/v1/feeds", Some(json!({}))).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{label_id}/links"),
        Some(json!({ "release": read_only["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
}

#[tokio::test]
async fn each_publisher_route_needs_the_token() {
    let app = app();
    let routes = [
        (
            "POST",
            "/api/v1/feeds/1/links",
            Some(json!({ "release": 2 })),
        ),
        ("DELETE", "/api/v1/feeds/1/links/2", None),
        (
            "PUT",
            "/api/v1/feeds/1/role",
            Some(json!({ "roles": ["label"] })),
        ),
    ];
    for (method, path, body) in routes {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, HOST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
            .unwrap();
        let (status, value, _) = send(&app, request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
        assert_eq!(value["error"]["code"], "token_required", "{method} {path}");
    }
}

#[tokio::test]
async fn a_feed_without_an_address_is_read_only_and_a_missing_file_gives_404() {
    let app = app();
    set_up(&app).await;
    let (_, feed) = call(&app, "POST", "/api/v1/feeds", Some(json!({}))).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{}/refresh", feed["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "read_only");

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let (_, missing) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connectors[0]["id"], "path": "missing.xml" })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{}/refresh", missing["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn a_feed_and_then_its_connector_go_out_of_the_collection() {
    let app = app();
    let id = set_up(&app).await;
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let connector = connectors[0]["id"].as_i64().unwrap();

    let (status, body) = call(
        &app,
        "DELETE",
        &format!("/api/v1/connectors/{connector}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "in_use");

    let (status, _) = call(&app, "DELETE", &format!("/api/v1/feeds/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = call(&app, "DELETE", &format!("/api/v1/feeds/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert!(app.feeds.path().join("album.xml").exists());

    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/connectors/{connector}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
}

#[tokio::test]
async fn an_api_request_without_the_token_gives_401() {
    let app = app();
    for auth in [None, Some("Bearer wrong-token"), Some("test-token")] {
        let mut builder = Request::builder()
            .uri("/api/v1/status")
            .header(header::HOST, HOST);
        if let Some(auth) = auth {
            builder = builder.header(header::AUTHORIZATION, auth);
        }
        let (status, body, _) = send(&app, builder.body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{auth:?}");
        assert_eq!(body["error"]["code"], "token_required");
    }
}

#[tokio::test]
async fn a_host_that_is_not_loopback_gives_403() {
    let app = app();
    for host in ["evil.example:4100", "127.0.0.1:9999", "127.0.0.1", ""] {
        let request = Request::builder()
            .uri("/api/v1/status")
            .header(header::HOST, host)
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(&app, request).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{host}");
        assert_eq!(body["error"]["code"], "host_not_allowed");
    }
    for host in ["localhost:4100", "[::1]:4100"] {
        let request = request("GET", "/api/v1/status", None);
        let (mut parts, body) = request.into_parts();
        parts.headers.insert(header::HOST, host.parse().unwrap());
        let (status, _, _) = send(&app, Request::from_parts(parts, body)).await;
        assert_eq!(status, StatusCode::OK, "{host}");
    }
}

#[tokio::test]
async fn a_body_that_is_not_json_gives_415() {
    let app = app();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/vault")
        .header(header::HOST, HOST)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("passphrase=correct+horse+battery"))
        .unwrap();
    let (status, _, _) = send(&app, request).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(!app.data.path().join("vault.json").exists());
}

#[tokio::test]
async fn no_get_response_contains_a_secret() {
    let app = app();
    let id = set_up(&app).await;
    call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    // Put a connector secret in the vault, and a connector that refers to it.
    let secret = "sftp-password-1f9e2c";
    let vault_key = {
        let file =
            studio_core::vault::VaultFile::load(&app.data.path().join("vault.json")).unwrap();
        let mut vault = file.open_with_passphrase(PASSPHRASE).unwrap();
        vault.put("connector:secret", secret.as_bytes()).unwrap();
        vault
            .file()
            .save(&app.data.path().join("vault.json"))
            .unwrap();
        vault.key_for_signer().to_string()
    };
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    collection
        .add_connector(
            "sftp",
            "Server",
            r#"{"host":"example.com"}"#,
            Some("connector:secret"),
        )
        .unwrap();
    drop(collection);
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;

    for path in [
        "/api/v1/status",
        "/api/v1/connectors",
        "/api/v1/feeds",
        "/api/v1/recipients",
    ] {
        let (status, _, text) = send(&app, request("GET", path, None)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        for forbidden in [secret, PASSPHRASE, vault_key.as_str()] {
            assert!(!text.contains(forbidden), "{path} contains a secret");
        }
    }
}

/// The page rule of ADR 0109 item 1. Scripts, styles, fonts and frames come
/// only from Studio. Only `img-src` and `media-src` accept `https:`.
fn check_page_rule(policy: &str) {
    const FIX: &str =
        "Change SECURITY_HEADERS in crates/api/src/ui.rs back to the rule of ADR 0109 item 1.";
    assert_eq!(
        policy,
        "default-src 'self'; img-src 'self' data: https:; media-src 'self' https:; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        "ADR 0109 item 1: the page rule is not the rule of the decision. {FIX}"
    );
    assert!(
        !policy.contains("unsafe"),
        "ADR 0016: the page rule has no unsafe source. {FIX}"
    );
    for directive in policy.split(';').map(str::trim) {
        let (name, sources) = directive.split_once(' ').unwrap_or((directive, ""));
        if matches!(name, "img-src" | "media-src") {
            continue;
        }
        let allowed: &[&str] = match name {
            "default-src" | "script-src" | "style-src" | "font-src" | "frame-src"
            | "connect-src" => &["'self'"],
            _ => &["'none'"],
        };
        assert!(
            sources.split(' ').all(|source| allowed.contains(&source)),
            "ADR 0109 item 2: {name} may load only from Studio, and `https:` is only for img-src and media-src. {FIX}"
        );
    }
}

#[tokio::test]
async fn the_ui_files_load_without_the_token_and_with_a_strict_policy() {
    let app = app();
    let paths =
        std::iter::once("/".to_owned()).chain(crate::ui::paths().map(|p| format!("/ui/{p}")));
    for path in paths {
        let request = Request::builder()
            .uri(&path)
            .header(header::HOST, HOST)
            .body(Body::empty())
            .unwrap();
        let response = app.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let policy = response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap();
        check_page_rule(policy);
    }
    let request = Request::builder()
        .uri("/ui/../Cargo.toml")
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&app, request).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_manifest_and_its_icons_load_without_the_token() {
    let app = app();
    let get = |path: String| {
        let router = app.router.clone();
        async move {
            let request = Request::builder()
                .uri(&path)
                .header(header::HOST, HOST)
                .body(Body::empty())
                .unwrap();
            router.oneshot(request).await.unwrap()
        }
    };
    let response = get("/manifest.webmanifest".to_owned()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/manifest+json"
    );
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(manifest["display"], "standalone");
    let icons = manifest["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 3);
    for icon in icons {
        let src = icon["src"].as_str().unwrap();
        let response = get(src.to_owned()).await;
        assert_eq!(response.status(), StatusCode::OK, "{src}");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
    }
}

#[test]
fn the_ui_names_no_other_server_and_sends_the_token() {
    for (name, text) in crate::ui::text_files() {
        // The vendored code and the license texts are not UI code.
        if name.starts_with("vendor/") || name.to_ascii_lowercase().ends_with(".txt") {
            continue;
        }
        for scheme in ["http://", "https://"] {
            let found: Vec<&str> = text
                .match_indices(scheme)
                .map(|(i, _)| &text[i..(i + 40).min(text.len())])
                // A scheme with no host name after it, as in an instruction
                // to the user, names no server.
                .filter(|s| {
                    s[scheme.len()..]
                        .chars()
                        .next()
                        .is_some_and(char::is_alphanumeric)
                })
                .filter(|s| !s.contains("www.w3.org/2000/svg") && !s.contains("music.example"))
                // The link that the user opens to the OP3 page (ADR 0076
                // item 11). The UI does not load it.
                .filter(|s| !s.starts_with("https://op3.dev/show/"))
                // The prefix that `media.js` removes from an address, so that
                // the player makes no download for OP3 (ADR 0076 item 3).
                .filter(|s| !s.starts_with("https://op3.dev/e"))
                // The link that the user opens to a feed on musicindex.org
                // (ADR 0117 item 8). The UI does not load it.
                .filter(|s| !s.starts_with("https://musicindex.org/#feed="))
                .collect();
            assert!(
                found.is_empty(),
                "ADR 0016: {name} names another server: {found:?}"
            );
        }
    }
    let app_js = crate::ui::text_files()
        .find(|(name, _)| *name == "app.js")
        .unwrap()
        .1;
    assert_eq!(
        app_js.matches("fetch(").count(),
        1,
        "app.js must call the API through api()"
    );
    for (name, text) in crate::ui::text_files() {
        if name != "app.js" && !name.starts_with("vendor/") {
            assert!(
                !text.contains("fetch("),
                "ADR 0017: {name} calls fetch(). Import api() from app.js, so that each request sends the token."
            );
        }
    }
    assert!(app_js.contains("Authorization: `Bearer ${token}`"));
}

/// On 2026-10-27 the files `cover.js` and `picker.js` were in the UI folder
/// and imported by `editor.js`, but `ui.rs` did not serve them. Each script
/// of the UI folder must be served.
#[test]
fn each_ui_script_of_the_ui_folder_is_served() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let served: Vec<&str> = crate::ui::paths().collect();
    for entry in std::fs::read_dir(folder).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if std::path::Path::new(&name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("js"))
        {
            assert!(
                served.contains(&name.as_str()),
                "ADR 0016: crates/api/ui/{name} is not served. Add it to FILES in crates/api/src/ui.rs."
            );
        }
    }
}

#[test]
fn each_ui_address_in_the_ui_files_is_served() {
    // On 2026-10-22 the lockup image was in app.js but not in the served
    // files, and the page showed a broken image.
    let served: Vec<&str> = crate::ui::paths().collect();
    for (name, text) in crate::ui::text_files() {
        for (index, _) in text.match_indices("/ui/") {
            let rest = &text[index + 4..];
            let end = rest.find(['"', '\'', ')', '`', ' ']).unwrap_or(rest.len());
            let path = &rest[..end];
            if path.is_empty() || path.contains("${") {
                continue;
            }
            assert!(
                served.contains(&path),
                "ADR 0016: {name} refers to /ui/{path}, but crates/api/src/ui.rs does not serve it. Add it to FILES."
            );
        }
    }
}

/// A test server with a publisher feed, two releases and a musicindex API.
fn web_feeds() -> studio_core::testing::TestServer {
    use studio_core::testing::TestServer;
    let server = TestServer::start(Vec::new());
    let feed = |medium: &str, guid: &str, title: &str, extra: &str| {
        format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>{title}</title>\
             <podcast:guid>{guid}</podcast:guid><podcast:medium>{medium}</podcast:medium>{extra}</channel></rss>"
        )
        .into_bytes()
    };
    let a = server.url("/a.xml");
    server.set(
        "/publisher.xml",
        200,
        feed("publisher", "p-1", "Label", &format!(
            "<podcast:remoteItem feedGuid=\"g-1\" feedUrl=\"{a}\"/><podcast:remoteItem feedGuid=\"b-1\"/>"
        )),
    );
    // The same GUID as the local FEED, so that a folder match can find it.
    server.set("/a.xml", 200, FEED.as_bytes().to_vec());
    server.set("/b.xml", 200, feed("music", "b-1", "Album B", ""));
    server.set(
        "/v1/feeds/b-1",
        200,
        format!("{{\"data\":{{\"feed_url\":\"{}\"}}}}", server.url("/b.xml")).into_bytes(),
    );
    server.set(
        "/v1/search",
        200,
        br#"{"data":[{"entity_type":"feed","entity_id":"b-1","title":"Album B","release_artist":"Ann"}]}"#.to_vec(),
    );
    server
}

#[tokio::test]
async fn discovery_follows_a_publisher_and_import_adds_read_only_feeds() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/discover",
        Some(json!({ "addresses": [server.url("/publisher.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let feeds = body["feeds"].as_array().unwrap();
    let titles: Vec<&str> = feeds.iter().map(|f| f["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Label", "Album", "Album B"]);
    assert_eq!(feeds[1]["via"], "release");
    assert_eq!(feeds[0]["host"], "127.0.0.1");
    assert_eq!(feeds[0]["service"], Value::Null);
    assert!(feeds.iter().all(|f| f["in_collection"] == false));

    let urls: Vec<&str> = feeds.iter().map(|f| f["url"].as_str().unwrap()).collect();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": urls })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["added"].as_array().unwrap().len(), 3);
    assert!(
        body["added"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["connector_id"].is_null())
    );

    // A second import adds nothing, and a second discovery marks the feeds.
    let (_, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": urls })),
    )
    .await;
    assert_eq!(body["added"], json!([]));
    let (_, body) = call(
        &app,
        "POST",
        "/api/v1/discover",
        Some(json!({ "addresses": [server.url("/publisher.xml")] })),
    )
    .await;
    assert!(
        body["feeds"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["in_collection"] == true)
    );
    // The vault is not necessary for read-only feeds (ADR 0020).
    let (_, status_body) = call(&app, "GET", "/api/v1/status", None).await;
    assert_eq!(status_body["vault"], "absent");
}

#[tokio::test]
async fn a_read_only_feed_refreshes_from_its_address() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    let (_, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/b.xml")] })),
    )
    .await;
    let id = body["added"][0]["id"].as_i64().unwrap();
    server.set(
        "/b.xml",
        200,
        FEED.replace("Album", "Album B2").into_bytes(),
    );
    let (status, feed) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["title"], "Album B2");
    server.set("/b.xml", 500, Vec::new());
    let (status, body) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "network");
}

#[tokio::test]
async fn a_connected_folder_takes_the_read_only_feeds_that_it_holds() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/a.xml"), server.url("/b.xml")] })),
    )
    .await;
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/connectors/{}/match", connector["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let linked = body["linked"].as_array().unwrap();
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0]["path"], "album.xml");
    assert_eq!(linked[0]["connector_id"], connector["id"]);
    assert!(
        body["inventory"]["new"].as_u64().unwrap() >= 1,
        "ADR 0036 item 6: Studio lists a host when the user adds it"
    );
}

/// The example of ADR 0021: a folder with no public address holds a feed that
/// discovery found and a feed that only the folder has.
#[tokio::test]
async fn a_folder_match_finds_the_folder_address_from_the_feeds() {
    let server = web_feeds();
    server.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let app = app_with_index(&server.url(""));
    std::fs::write(
        app.feeds.path().join("other.xml"),
        FEED.replace("g-1", "g-2"),
    )
    .unwrap();
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/album.xml")] })),
    )
    .await;
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/connectors/{}/match", connector["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["folder_address"],
        json!({ "state": "inferred", "url": server.url(""), "new": true })
    );
    assert_eq!(body["linked"][0]["address_source"], "read");

    let (_, other) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "other.xml" })),
    )
    .await;
    assert_eq!(other["public_url"], server.url("/other.xml"));
    assert_eq!(other["address_source"], "folder");
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors[0]["disagreeing_feeds"], json!([]));
    assert!(
        connectors[0]["config"]
            .as_str()
            .unwrap()
            .contains("\"base_url_inferred\":true")
    );
}

#[tokio::test]
async fn search_gives_feed_hits_from_musicindex() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    let (status, body) = call(&app, "GET", "/api/v1/search?q=album%20b", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["feeds"][0]["feed_guid"], "b-1");
    assert_eq!(body["feeds"][0]["artist"], "Ann");
    let (_, body) = call(
        &app,
        "POST",
        "/api/v1/discover",
        Some(json!({ "guids": ["b-1"] })),
    )
    .await;
    assert_eq!(body["feeds"][0]["title"], "Album B");
}

#[tokio::test]
async fn an_msp_backup_needs_the_vault_and_keeps_its_token_there() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    let token = "msp-edit-token-5e8c1a";
    let backup = json!({ "feedId": "abc", "editToken": token, "feedUrl": server.url("/b.xml") });

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/import/msp",
        Some(json!({ "backups": [backup] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_required");

    call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    let (status, _, text) = send(
        &app,
        request(
            "POST",
            "/api/v1/import/msp",
            Some(json!({ "backups": [backup, { "feedId": "x" }] })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(!text.contains(token));
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["added"][0]["title"], "Album B");
    assert_eq!(body["failed"].as_array().unwrap().len(), 1);

    let vault_file = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert!(!vault_file.contains(token));
    let vault = studio_core::vault::VaultFile::from_json(&vault_file)
        .unwrap()
        .open_with_passphrase(PASSPHRASE)
        .unwrap();
    assert_eq!(
        vault.get("msp:abc").unwrap().unwrap().as_slice(),
        token.as_bytes()
    );
    for path in ["/api/v1/feeds", "/api/v1/connectors", "/api/v1/status"] {
        let (_, _, text) = send(&app, request("GET", path, None)).await;
        assert!(!text.contains(token), "{path}");
    }
}

const S3_KEY_ID: &str = "AKIDEXAMPLE";
const S3_SECRET: &str = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";

/// The body of a new S3 host on a fake server.
fn s3_body(server: &studio_core::testing::fake_s3::FakeS3) -> Value {
    json!({
        "kind": "s3",
        "name": "Bucket",
        "endpoint": server.url(),
        "region": "us-east-1",
        "bucket": "bk",
        "url_style": "path",
        "prefix": "feeds",
        "base_url": "https://cdn.example.com/feeds",
        "access_key_id": S3_KEY_ID,
        "secret_access_key": S3_SECRET,
    })
}

async fn make_vault(app: &App) {
    call(
        app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
}

/// The text of each file in the data folder of the app.
fn data_files_text(app: &App) -> String {
    let mut text = String::new();
    for entry in std::fs::read_dir(app.data.path()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            text.push_str(&String::from_utf8_lossy(&std::fs::read(path).unwrap()));
        }
    }
    text
}

#[tokio::test]
async fn an_s3_host_is_added_after_the_provider_test_and_keeps_its_key_in_the_vault() {
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    let app = app();
    make_vault(&app).await;
    let (status, _, text) = send(
        &app,
        request("POST", "/api/v1/connectors", Some(s3_body(&server))),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert!(!text.contains(S3_SECRET) && !text.contains(S3_KEY_ID));
    let row: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(row["kind"], "s3");
    let secret_ref = row["secret_ref"].as_str().unwrap();
    assert!(secret_ref.starts_with("connector-"), "{secret_ref}");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["prefix"], "feeds/");
    assert_eq!(config["bucket"], "bk");
    // The provider test leaves no key on the host.
    assert!(server.keys().is_empty(), "{:?}", server.keys());

    let (_, _, list) = send(&app, request("GET", "/api/v1/connectors", None)).await;
    assert!(!list.contains(S3_SECRET) && !list.contains(S3_KEY_ID));
    assert!(!data_files_text(&app).contains(S3_SECRET));
    let vault_file = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert!(!vault_file.contains(S3_SECRET));
    let vault = studio_core::vault::VaultFile::from_json(&vault_file)
        .unwrap()
        .open_with_passphrase(PASSPHRASE)
        .unwrap();
    let secret: Value = serde_json::from_slice(&vault.get(secret_ref).unwrap().unwrap()).unwrap();
    assert_eq!(
        secret,
        json!({ "access_key_id": S3_KEY_ID, "secret_access_key": S3_SECRET })
    );
}

#[tokio::test]
async fn an_s3_host_needs_the_vault() {
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    let app = app();
    let (status, body) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&server))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_required");
    make_vault(&app).await;
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, body) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&server))).await;
    assert_eq!(status, StatusCode::LOCKED);
    assert_eq!(body["error"]["code"], "vault_locked");
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
}

#[tokio::test]
async fn an_s3_host_that_fails_the_provider_test_changes_neither_vault_nor_collection() {
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    let app = app();
    make_vault(&app).await;
    let vault_before = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();

    server.ignore_put_conditions(true);
    let (status, body) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&server))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "no_conditions");
    assert!(server.keys().is_empty(), "{:?}", server.keys());

    server.ignore_put_conditions(false);
    server.refuse_all(true);
    let (status, body) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&server))).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "host_unauthorized");

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
    let vault_after = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert_eq!(vault_before, vault_after);
}

#[tokio::test]
async fn an_s3_host_with_a_bad_field_gives_422_and_no_request_to_the_host() {
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    let app = app();
    make_vault(&app).await;
    let cases = [
        ("endpoint", json!("ftp://s3.example.com"), "invalid_address"),
        ("base_url", json!("cdn.example.com"), "invalid_address"),
        ("bucket", json!(" "), "invalid_config"),
        ("access_key_id", json!(""), "invalid_config"),
        ("secret_access_key", json!(""), "invalid_config"),
        ("region", json!(""), "invalid_config"),
    ];
    for (field, value, code) in cases {
        let mut body = s3_body(&server);
        body[field] = value;
        let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}");
        assert_eq!(answer["error"]["code"], code, "{field}");
    }
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
}

#[tokio::test]
async fn an_s3_host_gets_a_new_public_address_and_the_match_route_links_its_feeds() {
    let web = web_feeds();
    web.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let server = studio_core::testing::fake_s3::FakeS3::start("bk");
    server.put_object("feeds/album.xml", FEED.as_bytes());
    let app = app_with_index(&web.url(""));
    make_vault(&app).await;
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [web.url("/album.xml")] })),
    )
    .await;
    let (_, connector) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&server))).await;
    let id = connector["id"].as_i64().unwrap();

    let (status, row) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{id}"),
        Some(json!({ "base_url": "https://new.example.com/m" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["base_url"], "https://new.example.com/m");
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{id}"),
        Some(json!({ "base_url": "new.example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_address");

    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/connectors/{id}/match"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["linked"][0]["path"], "album.xml");
    assert_eq!(
        body["folder_address"],
        json!({ "state": "typed", "url": "https://new.example.com/m" })
    );
}

/// The row of a feed in the list of `GET /api/v1/feeds`.
async fn listed(app: &App, id: i64) -> Value {
    let (status, rows) = call(app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(status, StatusCode::OK);
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
        .clone()
}

#[tokio::test]
async fn the_feed_list_gives_the_draft_state_and_the_cover_image() {
    let app = app();
    let root = app.feeds.path().to_str().unwrap();
    let (status, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    set_library_root(&app);

    // A feed with no draft. Its draft starts only when Studio asks for it.
    let (status, read_only) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let read_only_id = read_only["id"].as_i64().unwrap();
    let row = listed(&app, read_only_id).await;
    assert_eq!(row["draft_state"], "none");
    assert_eq!(row["image"], Value::Null);
    assert_eq!(row["rss_image_url"], Value::Null);
    assert!(row["added_at"].as_i64().unwrap() > 0);

    // A new release has a draft and no hosted file.
    let feed = make_feed(
        &app,
        json!({
            "connector_id": connector["id"],
            "path": "my-album.xml",
            "title": "New Album",
            "author": "New Artist",
        }),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    assert_eq!(listed(&app, id).await["draft_state"], "new");

    // After its write, the draft is the same as the hosted file.
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // The route starts a draft when the feed has none after the write.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let _ = call(&app, "GET", &draft_path, None).await;
    assert_eq!(listed(&app, id).await["draft_state"], "unchanged");

    // A feed in the DeMu form gives the address of its RSS image (ADR 0060).
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["image"] = Value::Null;
    model["rss_image"] = json!({ "url": "https://music.example/demu.png" });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let row = listed(&app, id).await;
    assert_eq!(row["image"], Value::Null);
    assert_eq!(row["rss_image_url"], "https://music.example/demu.png");

    // An empty address removes both cover tags (ADR 0060, ADR 0065 item 6).
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["image"] = json!("");
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let row = listed(&app, id).await;
    assert_eq!(row["image"], Value::Null);
    assert_eq!(row["rss_image_url"], Value::Null);

    // An edit of the cover address changes the draft and its image.
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["image"] = json!("https://music.example/cover.jpg");
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let row = listed(&app, id).await;
    assert_eq!(row["draft_state"], "changed");
    assert_eq!(row["image"], "https://music.example/cover.jpg");
    assert_eq!(row["rss_image_url"], Value::Null);
}

/// The live file of the feed that `set_up` adds.
fn live(app: &App) -> String {
    std::fs::read_to_string(app.feeds.path().join("album.xml")).unwrap()
}

#[tokio::test]
async fn a_draft_is_edited_examined_written_and_rolled_back() {
    let app = app();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(draft["changed"], false);
    assert_eq!(draft["model"]["items"][0]["key"], 0);

    let mut model = draft["model"].clone();
    model["title"] = json!("Album (remaster)");
    let (status, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(draft["changed"], true);
    assert_eq!(live(&app), FEED);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert!(
        diff["unified"]
            .as_str()
            .unwrap()
            .contains("+<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>Album (remaster)</title>")
    );
    let write_path = format!("/api/v1/feeds/{id}/write");
    let (status, body) = call(
        &app,
        "POST",
        &write_path,
        Some(json!({ "diff_id": "other" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "diff_not_approved");
    let (status, feed) = call(
        &app,
        "POST",
        &write_path,
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["title"], "Album (remaster)");
    assert!(live(&app).contains("Album (remaster)"));

    let (_, snapshots) = call(&app, "GET", &format!("/api/v1/feeds/{id}/snapshots"), None).await;
    let snapshot = snapshots[0]["id"].as_i64().unwrap();
    let (status, draft) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/snapshots/{snapshot}/draft"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(draft["model"]["title"], "Album");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    call(
        &app,
        "POST",
        &write_path,
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(live(&app), FEED);
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/snapshots/999/draft"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// ADR 0064 item 8: a track that uses the album credits has no persons. A
/// save with an empty list removes each item `podcast:person` and keeps the
/// channel persons.
#[tokio::test]
async fn an_empty_persons_list_of_a_track_removes_its_persons_and_keeps_the_channel_persons() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
        <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
        <podcast:person>Chan</podcast:person>\
        <item><guid>t-1</guid><podcast:person>Bob</podcast:person>\
        <podcast:person>Cy</podcast:person></item></channel></rss>",
    )
    .unwrap();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    assert_eq!(model["items"][0]["persons"].as_array().unwrap().len(), 2);
    model["items"][0]["persons"] = json!([]);
    let (status, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["model"]["items"][0]["persons"], json!([]));
    assert_eq!(draft["model"]["persons"].as_array().unwrap().len(), 1);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let unified = diff["unified"].as_str().unwrap();
    let removed = unified
        .lines()
        .filter(|l| l.starts_with('-') && !l.starts_with("---"))
        .collect::<String>();
    let added = unified
        .lines()
        .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        .collect::<String>();
    assert!(
        removed.contains("Bob") && removed.contains("Cy"),
        "{unified}"
    );
    assert!(!added.contains("Bob") && !added.contains("Cy"), "{unified}");
    assert!(added.contains("Chan"), "{unified}");
}

#[tokio::test]
async fn a_stale_or_invalid_model_and_a_changed_file_give_errors() {
    let app = app();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["key"] = json!(5);
    let (status, body) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_model");
    let (status, body) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": "old", "model": draft["model"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "stale_revision");

    let mut model = draft["model"].clone();
    model["title"] = json!("Changed");
    call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    std::fs::write(
        app.feeds.path().join("album.xml"),
        FEED.replace("t-1", "t-9"),
    )
    .unwrap();
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "conflict");
    assert!(live(&app).contains("t-9"));

    let (status, _) = call(&app, "DELETE", &draft_path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(draft["changed"], false);
    assert_eq!(draft["model"]["items"][0]["guid"], "t-9");
}

#[tokio::test]
async fn each_host_route_of_a_read_only_feed_with_no_folder_gives_an_error() {
    let app = app();
    set_up(&app).await;
    let (_, feed) = call(&app, "POST", "/api/v1/feeds", Some(json!({}))).await;
    let id = feed["id"].as_i64().unwrap();
    let model = serde_json::to_value(studio_core::feed::Feed::default()).unwrap();
    let body = json!({ "revision": "r", "model": model });
    // A feed with no release folder has no draft to start (ADR 0067 item 3).
    for (method, path, body) in [
        ("GET", format!("/api/v1/feeds/{id}/draft"), None),
        ("PUT", format!("/api/v1/feeds/{id}/draft"), Some(body)),
        ("GET", format!("/api/v1/feeds/{id}/diff"), None),
    ] {
        let (status, response) = call(&app, method, &path, body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}");
        assert_eq!(response["error"]["code"], "not_found", "{method} {path}");
    }
    for (method, path, body) in [
        (
            "POST",
            format!("/api/v1/feeds/{id}/write"),
            Some(json!({ "diff_id": "d" })),
        ),
        ("GET", format!("/api/v1/feeds/{id}/snapshots"), None),
        (
            "POST",
            format!("/api/v1/feeds/{id}/snapshots/1/draft"),
            None,
        ),
    ] {
        let (status, response) = call(&app, method, &path, body).await;
        assert_eq!(status, StatusCode::CONFLICT, "{method} {path}");
        assert_eq!(response["error"]["code"], "read_only", "{method} {path}");
    }
}

#[tokio::test]
async fn the_draft_and_the_save_give_bad_keys() {
    let app = app();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(draft["bad_keys"], json!([]));
    let mut model = draft["model"].clone();
    model["persons"] = json!([{ "name": "Ann", "npub": "npub1xyz" }]);
    let (_, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(
        saved["bad_keys"],
        json!([{ "field": "persons/0/npub", "value": "npub1xyz" }])
    );
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "bad_key");
}

#[tokio::test]
async fn the_draft_and_the_save_give_the_op3_track_counts() {
    let app = app();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": "https://music.example/one.mp3",
        "length": "1",
        "media_type": "audio/mpeg",
    });
    let (_, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(draft["model"]["op3"], json!(false));
    assert_eq!(draft["op3_tracks"], json!({ "prefixed": 0, "tracks": 1 }));
    let mut model = draft["model"].clone();
    model["op3"] = json!(true);
    let (_, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(saved["model"]["op3"], json!(true));
    assert_eq!(saved["op3_tracks"], json!({ "prefixed": 1, "tracks": 1 }));
    assert_eq!(
        saved["model"]["items"][0]["enclosure"]["url"],
        json!("https://op3.dev/e,pg=g-1/music.example/one.mp3")
    );
}

#[tokio::test]
async fn a_bad_address_is_listed_with_the_draft_and_stops_the_write() {
    let app = app();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(draft["bad_addresses"], json!([]));
    let mut model = draft["model"].clone();
    model["link"] = json!("www.example.com");
    let (_, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(
        draft["bad_addresses"],
        json!([{ "field": "link", "value": "www.example.com" }])
    );
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "bad_address");
    assert!(body["error"]["message"].as_str().unwrap().contains("link"));
    assert_eq!(live(&app), FEED);
}

#[tokio::test]
async fn the_media_route_gives_the_size_and_the_type_of_a_file() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set_file("/a.mp3", vec![0; 4321], "audio/mpeg", false);
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/media/facts",
        Some(json!({ "url": server.url("/a.mp3") })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "length": 4321, "media_type": "audio/mpeg" }));
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/media/facts",
        Some(json!({ "url": "none" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_address");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/media/facts",
        Some(json!({ "url": server.url("/gone.mp3") })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "network");
}

/// A held image comes from the release folder (ADR 0109 item 5). The planned
/// address `https://music.example/...` has no web server in this test, so
/// only the library can give the bytes.
#[tokio::test]
async fn the_image_route_gives_a_held_image_by_its_file_id() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("image.png")).unwrap();
    let (status, file) = upload(&app, id, "image.png", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    assert_eq!(file["on_host"], false);
    let file_id = file["library_file_id"].as_i64().unwrap();

    let path = format!("/api/v1/images?file={file_id}");
    let (status, headers, body) = send_raw(&app, request("GET", &path, None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "image/png");
    assert_eq!(body, bytes);

    // The held list gives the planned address of the file.
    let (status, held) = call(&app, "GET", "/api/v1/media/held", None).await;
    assert_eq!(status, StatusCode::OK);
    let row = &held["files"][0];
    assert_eq!(row["file"], file_id);
    assert_eq!(row["url"], file["url"]);
}

#[tokio::test]
async fn the_image_route_reads_no_address_and_no_file_that_is_not_an_image() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set_file("/cover.png", vec![1, 2, 3, 4], "image/png", true);
    let app = app();
    let id = set_up(&app).await;
    // An address is not a file ID: the route refuses it, and the host
    // sees no request.
    let (status, _) = call(
        &app,
        "GET",
        &format!(
            "/api/v1/images?url={}",
            url_encode(&server.url("/cover.png"))
        ),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "ADR 0109 item 5: the image route accepts a file ID, not an address"
    );
    let (status, _) = call(&app, "GET", "/api/v1/images?file=9999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let (status, body) = call(
        &app,
        "GET",
        &format!(
            "/api/v1/images?file={}",
            file["library_file_id"].as_i64().unwrap()
        ),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "file_not_held");
}

#[tokio::test]
async fn an_api_request_with_no_token_to_images_gives_401() {
    let app = app();
    let request = Request::builder()
        .uri("/api/v1/images?file=1")
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(&app, request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_required");
}

/// A request to the audio route. It carries no access token: the ticket is
/// the access (ADR 0062 item 29).
fn audio_request(ticket: &str, range: Option<&str>) -> Request<Body> {
    let builder = Request::builder()
        .uri(format!("/media/audio?ticket={ticket}"))
        .header(header::HOST, HOST);
    match range {
        Some(range) => builder.header(header::RANGE, range),
        None => builder,
    }
    .body(Body::empty())
    .unwrap()
}

async fn ticket_for(app: &App, file: i64) -> String {
    let (status, body) = call(
        app,
        "POST",
        "/api/v1/media/tickets",
        Some(json!({ "file": file })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let ticket = body["ticket"].as_str().unwrap().to_owned();
    assert_eq!(ticket.len(), 32);
    assert!(ticket.bytes().all(|b| b.is_ascii_hexdigit()));
    ticket
}

#[tokio::test]
async fn the_audio_route_gives_a_range_of_a_file_of_the_release_folder() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    assert!(bytes.len() > 100);
    let (status, file) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let ticket = ticket_for(&app, file["library_file_id"].as_i64().unwrap()).await;

    let (status, headers, body) = send_raw(&app, audio_request(&ticket, Some("bytes=0-99"))).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body.len(), 100);
    assert_eq!(body, bytes[..100]);
    assert_eq!(
        headers[header::CONTENT_RANGE],
        format!("bytes 0-99/{}", bytes.len()).as_str()
    );
    assert_eq!(headers[header::CONTENT_TYPE], "audio/mpeg");
    assert_eq!(headers[header::ACCEPT_RANGES], "bytes");

    // A range with no end and a range of the last bytes.
    let (status, _, body) = send_raw(&app, audio_request(&ticket, Some("bytes=100-"))).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body, bytes[100..]);
    let (status, _, body) = send_raw(&app, audio_request(&ticket, Some("bytes=-10"))).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body, bytes[bytes.len() - 10..]);

    // No range gives the whole file with 200.
    let (status, headers, body) = send_raw(&app, audio_request(&ticket, None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, bytes);
    assert_eq!(
        headers[header::CONTENT_LENGTH],
        bytes.len().to_string().as_str()
    );
}

/// The audio route sends only an audio file of the release folder (ADR 0109
/// items 5 and 6).
#[tokio::test]
async fn the_audio_route_refuses_a_file_that_is_not_audio() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("image.png")).unwrap();
    let (_, file) = upload(&app, id, "image.png", bytes).await;
    let ticket = ticket_for(&app, file["library_file_id"].as_i64().unwrap()).await;
    let (status, body, _) = send(&app, audio_request(&ticket, None)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "file_not_held");
}

#[tokio::test]
async fn the_audio_route_gives_416_for_a_range_outside_the_file() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    let ticket = ticket_for(&app, file["library_file_id"].as_i64().unwrap()).await;
    for range in [
        format!("bytes={}-", bytes.len()),
        "bytes=99999999-".to_owned(),
        "bytes=-0".to_owned(),
        "bytes=5-1".to_owned(),
        "bytes=0-1,5-6".to_owned(),
        "lines=0-1".to_owned(),
    ] {
        let (status, headers, _) = send_raw(&app, audio_request(&ticket, Some(&range))).await;
        assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE, "{range}");
        if range.starts_with("bytes=9") || range == format!("bytes={}-", bytes.len()) {
            assert_eq!(
                headers[header::CONTENT_RANGE],
                format!("bytes */{}", bytes.len()).as_str()
            );
        }
    }
}

#[tokio::test]
async fn the_audio_route_refuses_an_unknown_an_expired_and_a_missing_ticket() {
    let app = app();
    let (status, body, _) = send(&app, audio_request(&"0".repeat(32), None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "ticket_not_valid");
    let (status, _, _) = send(
        &app,
        Request::builder()
            .uri("/media/audio")
            .header(header::HOST, HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // A ticket that is valid for no time is expired at its first use.
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_ticket_lifetime(std::time::Duration::ZERO);
    let expired = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds: tempfile::tempdir().unwrap(),
    };
    let id = set_up(&expired).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&expired, id, "tagged.mp3", bytes).await;
    let ticket = ticket_for(&expired, file["library_file_id"].as_i64().unwrap()).await;
    let (status, body, _) = send(&expired, audio_request(&ticket, None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "ticket_not_valid");
}

#[tokio::test]
async fn the_ticket_route_needs_the_token_and_a_file_id_and_never_an_address() {
    let app = app();
    let no_token = Request::builder()
        .method("POST")
        .uri("/api/v1/media/tickets")
        .header(header::HOST, HOST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "file": 1 }).to_string()))
        .unwrap();
    let (status, body, _) = send(&app, no_token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_required");
    for request in [
        json!({ "url": "https://example.com/a.mp3" }),
        json!({}),
        json!({ "url": "https://example.com/a.mp3", "file": 1 }),
    ] {
        let (status, body) =
            call(&app, "POST", "/api/v1/media/tickets", Some(request.clone())).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "ADR 0109 item 5: a ticket holds a library file ID, not an address {request}"
        );
        assert_eq!(body["error"]["code"], "invalid_ticket_request");
    }
    // A file of no user is refused.
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/media/tickets",
        Some(json!({ "file": 9999 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn recipients_are_listed_saved_and_removed() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        FEED_WITH_RECIPIENTS.as_bytes(),
    )
    .unwrap();
    let id = set_up(&app).await;
    call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;

    let (status, body) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK);
    let recipients = body.as_array().unwrap();
    assert_eq!(recipients.len(), 2);
    assert_eq!(recipients[0]["name"], "Ann");
    assert_eq!(recipients[0]["address"], "ann@example.com");
    assert_eq!(recipients[0]["feeds"], 1);
    assert_eq!(recipients[0]["saved_id"], Value::Null);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/recipients/saved",
        Some(json!({ "name": "Cara", "recipient_type": "node", "address": "cara@example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let saved_id = body["id"].as_i64().unwrap();

    let (_, body) = call(&app, "GET", "/api/v1/recipients", None).await;
    let recipients = body.as_array().unwrap();
    assert_eq!(recipients.len(), 3);
    let cara = recipients
        .iter()
        .find(|r| r["name"] == "Cara")
        .expect("Cara is in the list");
    assert_eq!(cara["saved_id"], saved_id);

    // A second save of the same key gives the same person (ADR 0098).
    let (status, again) = call(
        &app,
        "POST",
        "/api/v1/recipients/saved",
        Some(json!({ "name": "Cara 2", "recipient_type": "node", "address": "cara@example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(again["id"], saved_id);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/recipients/saved",
        Some(json!({ "name": "No Address" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_recipient");

    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/recipients/saved/{saved_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = call(
        &app,
        "DELETE",
        &format!("/api/v1/recipients/saved/{saved_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn an_api_request_with_no_token_to_recipients_gives_401() {
    let app = app();
    let request = Request::builder()
        .uri("/api/v1/recipients")
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(&app, request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_required");
}

/// The payment routes of musicindex, for the recipient picker (ADR 0027
/// item 6). The track GUID has hex digits only. `MusicIndex` keeps only the
/// hex digits and the hyphens of a GUID in its request path.
#[tokio::test]
async fn the_musicindex_routes_give_the_payment_routes_of_a_feed_and_a_track() {
    let server = web_feeds();
    server.set(
        "/v1/feeds/b-1",
        200,
        br#"{"data":{"title":"Album B","payment_routes":[{"recipient_name":"Ann","route_type":"lnaddress","address":"ann@example.com","split":90,"fee":false}],"tracks":[{"track_guid":"aa11","title":"Track One"}]}}"#.to_vec(),
    );
    server.set(
        "/v1/feeds/b-1/tracks/track%20one",
        200,
        br#"{"data":{"title":"Track One","payment_routes":[{"recipient_name":"Bob","route_type":"lnaddress","address":"bob@example.com","split":10,"fee":false}]}}"#.to_vec(),
    );
    let app = app_with_index(&server.url(""));

    let (status, body) = call(&app, "GET", "/api/v1/musicindex/feeds/b-1/recipients", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Album B");
    assert_eq!(body["recipients"][0]["address"], "ann@example.com");
    assert_eq!(body["tracks"][0]["track_guid"], "aa11");

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/musicindex/feeds/b-1/tracks/track%20one/recipients",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Track One");
    assert_eq!(body["recipients"][0]["address"], "bob@example.com");
    assert_eq!(body["tracks"], json!([]));

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/musicindex/feeds/00000000-0000-0000-0000-000000000000/recipients",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

/// ADR 0028: on 2026-10-22 the recipients and the people of the editor were
/// lists of open entries, and the operator scrolled far and closed them by
/// hand.
#[test]
fn each_details_element_is_a_named_entry_or_a_section() {
    for (name, text) in crate::ui::text_files() {
        let is_script = std::path::Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("js"));
        if name.starts_with("vendor/") || !is_script {
            continue;
        }
        for (index, _) in text.match_indices("<details") {
            let end = text[index..].find('>').map_or(text.len(), |e| index + e);
            let tag = &text[index..end];
            assert!(
                tag.contains("name=") || tag.contains("disclosure"),
                "ADR 0028: {name} has a details element that is not a list entry with a name, \
                 and not a section with the class disclosure: {tag}. Give the entries of a list \
                 one name, so that one entry is open at a time."
            );
        }
    }
}

/// ADR 0029: on 2026-10-22 the operator found six shades of gray hard to
/// read. The editor keeps three background levels, each on a named token.
#[test]
fn each_background_uses_a_token() {
    let (_, css) = crate::ui::text_files()
        .find(|(name, _)| *name == "styles.css")
        .unwrap();
    for keyword in ["background:", "background-color:"] {
        for (index, _) in css.match_indices(keyword) {
            let rest = &css[index..];
            let end = rest.find(';').map_or(rest.len(), |e| e + 1);
            let declaration = &rest[..end];
            let value = &declaration[keyword.len()..];
            let is_literal_color = value.contains('#')
                || value.contains("rgb(")
                || value.contains("rgba(")
                || value.contains("hsl(");
            let is_bare_surface = value.contains("var(--bg)")
                || value.contains("var(--surface)")
                || value.contains("var(--surface2)");
            assert!(
                !is_literal_color && !is_bare_surface,
                "ADR 0029 §Background Levels: styles.css sets a background that is not a level, \
                 fill or named token: {declaration}. Use --level-1, --level-2, --level-3 or a \
                 fill token."
            );
        }
    }
}

/// ADR 0039 item 4: no page is a placeholder. The text "Select a feed" was
/// the placeholder of the editor area, so no UI file may show it again.
#[test]
fn no_ui_file_has_the_select_a_feed_placeholder() {
    for (name, text) in crate::ui::text_files() {
        assert!(
            !text.contains("Select a feed"),
            "ADR 0039 item 4: {name} shows the placeholder \"Select a feed\". Show the \
             Releases page, or the feed that is open, instead."
        );
    }
}

/// musicindex ADR 0041 R-SCALE-4: on 2026-10-24 Studio gave each control
/// 44 px with a mouse. A control takes its height from the `--target`
/// token: 28 px with a mouse, and 44 px with a coarse pointer.
#[test]
fn each_control_height_uses_the_target_token() {
    let (_, css) = crate::ui::text_files()
        .find(|(name, _)| *name == "styles.css")
        .unwrap();
    let touch = "@media (pointer: coarse) {\n  :root { --target: 44px; --target-min: 28px; --row: 52px; --leading: 1.45; }\n}";
    assert!(
        css.contains("  --target: 28px;\n  --target-min: 20px;\n") && css.contains(touch),
        "musicindex ADR 0041 R-SCALE-4: styles.css must set --target to 28 px and \
         --target-min to 20 px in :root, and to 44 px and 28 px for a coarse pointer."
    );
    let rest = css.replacen(touch, "", 1);
    assert!(
        !rest.contains("44px"),
        "musicindex ADR 0041 R-SCALE-4: styles.css has a fixed 44px. Use var(--target), \
         which is 28 px with a mouse and 44 px with touch."
    );
}

/// ADR 0037 item 5: on 2026-10-23 the operator asked for button roles that
/// show what each button does. The class "button quiet" has no fill, so
/// it is only for an icon button, such as the close button of a sheet.
#[test]
fn each_quiet_button_is_an_icon_button() {
    for (name, text) in crate::ui::text_files() {
        let is_script = std::path::Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("js"));
        if name.starts_with("vendor/") || !is_script {
            continue;
        }
        for (index, _) in text.match_indices("class=\"button quiet") {
            let quote_start = index + "class=\"".len();
            let rest = &text[quote_start..];
            let end = rest.find('"').unwrap_or(rest.len());
            let classes = &rest[..end];
            assert!(
                classes.contains("icon-button"),
                "ADR 0037 §Button Roles item 5: {name} has a quiet button with no icon-button \
                 class: \"{classes}\". Give this button a role, such as action or destructive, \
                 or add the class icon-button when it has an icon and no text label."
            );
        }
    }
}

#[tokio::test]
async fn an_upload_of_an_mp3_gives_its_release_address_and_facts_with_no_change_of_the_draft() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();

    let (status, body) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["path"], "album/tagged.mp3");
    assert_eq!(body["url"], "https://music.example/album/tagged.mp3");
    assert_eq!(body["media_type"], "audio/mpeg");
    assert_eq!(body["kind"], "audio");
    assert_eq!(body["title"], "Test Title");
    assert_eq!(body["track_number"], 3);
    assert_eq!(body["duration"], "00:00:01");
    assert_eq!(body["converted_from_wav"], false);
    assert_eq!(body["file_name"], "tagged.mp3");
    assert!(body["size"].as_u64().unwrap() > 0);
    // An MP3 import gives no lossless file (ADR 0033 item 4).
    assert_eq!(body["lossless"], Value::Null);
    assert_eq!(body["alternate_enclosures"], json!([]));
    assert!(incoming_is_empty(&app));

    // The route does not change the model of the draft, so the diff lists
    // no file: the draft does not refer to it yet (ADR 0032 item 10).
    let (status, _) = call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["files"], json!([]));
}

/// The name of an upload comes from the browser. A file goes only into
/// the folder of its release, so the route keeps only the last part of the
/// name, and refuses a name that is no file name.
#[tokio::test]
async fn an_upload_keeps_only_the_last_part_of_its_name() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();

    let (status, body) = upload(&app, id, "../../sub/tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["path"], "album/tagged.mp3");

    for name in ["..", "", "sub/"] {
        let (status, body) = upload(&app, id, name, bytes.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{name}: {body}");
        assert_eq!(body["error"]["code"], "invalid_name");
    }
    assert!(incoming_is_empty(&app));
}

/// An upload of a WAV file gives an MP3 enclosure and a lossless FLAC
/// file, as two pending files of the draft (ADR 0033 items 1, 3 and 5).
#[tokio::test]
async fn an_upload_of_a_wav_file_gives_an_mp3_file_and_a_lossless_flac_file() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.wav")).unwrap();

    let (status, body) = upload(&app, id, "tagged.wav", bytes).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["path"], "album/tagged.mp3");
    assert_eq!(body["url"], "https://music.example/album/tagged.mp3");
    assert_eq!(body["media_type"], "audio/mpeg");
    assert_eq!(body["converted_from_wav"], true);
    assert_eq!(body["file_name"], "tagged.mp3");
    assert!(body["size"].as_u64().unwrap() > 0);
    assert_eq!(body["lossless"]["path"], "album/tagged.flac");
    // The core gives the two alternative enclosures (ADR 0065 item 10).
    let alternates = body["alternate_enclosures"].as_array().unwrap();
    assert_eq!(alternates.len(), 2);
    assert_eq!(alternates[0]["media_type"], "audio/mpeg");
    assert_eq!(alternates[0]["default"], "true");
    assert_eq!(alternates[0]["sources"][0]["uri"], body["url"]);
    assert_eq!(alternates[1]["media_type"], "audio/flac");
    assert_eq!(alternates[1]["default"], Value::Null);
    assert_eq!(alternates[1]["sources"][0]["uri"], body["lossless"]["url"]);
    assert_eq!(
        body["lossless"]["url"],
        "https://music.example/album/tagged.flac"
    );
    assert_eq!(body["lossless"]["media_type"], "audio/flac");
    assert_eq!(body["lossless"]["file_name"], "tagged.flac");
    assert!(body["lossless"]["size"].as_u64().unwrap() > 0);
    assert_eq!(body["on_host"], false);
    assert_eq!(body["lossless"]["on_host"], false);
    assert!(body["library_file_id"].is_i64());
    assert!(body["lossless"]["library_file_id"].is_i64());
    assert!(incoming_is_empty(&app));

    // Both files are library files, and no draft refers to either yet.
    let delete_mp3 = format!("/api/v1/feeds/{id}/files?file={}", body["library_file_id"]);
    let delete_flac = format!(
        "/api/v1/feeds/{id}/files?file={}",
        body["lossless"]["library_file_id"]
    );
    let (status, body) = call(&app, "DELETE", &delete_mp3, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (status, body) = call(&app, "DELETE", &delete_flac, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

/// An upload of a FLAC file gives an MP3 enclosure, and keeps the FLAC
/// file as the lossless file (ADR 0033 items 1, 3 and 5).
#[tokio::test]
async fn an_upload_of_a_flac_file_gives_an_mp3_file_and_keeps_the_flac_as_lossless() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.flac")).unwrap();

    let (status, body) = upload(&app, id, "tagged.flac", bytes).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["path"], "album/tagged.mp3");
    assert_eq!(body["media_type"], "audio/mpeg");
    assert_eq!(body["converted_from_wav"], false);
    assert_eq!(body["file_name"], "tagged.mp3");
    assert_eq!(body["lossless"]["path"], "album/tagged.flac");
    assert_eq!(body["lossless"]["media_type"], "audio/flac");
    assert_eq!(body["lossless"]["file_name"], "tagged.flac");
    assert!(body["lossless"]["size"].as_u64().unwrap() > 0);
    assert!(incoming_is_empty(&app));
}

/// Give a feed a release folder, and plant a library file row with
/// `bytes` at `name` in it, through a second connection to the
/// collection file (ADR 0036 items 1, 4 and 5). This simulates a file
/// that an earlier import put there, with no call to the upload route.
fn plant_library_file(app: &App, id: i64, folder: &str, name: &str, bytes: &[u8]) {
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let id = studio_core::collection::FeedId(id);
    collection.set_feed_folder(id, folder).unwrap();
    let library = studio_core::library::Library::open(app.data.path().join("library")).unwrap();
    let source_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join(name);
    std::fs::write(&source, bytes).unwrap();
    let (size, sha256) = library.put_file(folder, name, &source).unwrap();
    collection
        .add_library_file(
            id,
            name,
            size,
            Some(&sha256),
            "audio/flac",
            true,
            None,
            None,
        )
        .unwrap();
}

/// When a different file is already in the library at the release name
/// of the FLAC file, the upload of a WAV file gives 409 `file_exists`,
/// and the route adds no library file for the MP3 file either
/// (ADR 0033 item 3, ADR 0036 item 5).
#[tokio::test]
async fn when_the_flac_name_of_a_wav_upload_is_in_the_library_no_file_is_added() {
    let app = app();
    let id = set_up(&app).await;
    plant_library_file(&app, id, "album", "tagged.flac", b"a different flac file");

    let bytes = std::fs::read(media_path("tagged.wav")).unwrap();
    let (status, body) = upload(&app, id, "tagged.wav", bytes).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "file_exists");
    assert!(incoming_is_empty(&app));

    // The MP3 file did not stay in the library either.
    assert!(!app.data.path().join("library/album/tagged.mp3").exists());
}

/// A second import at a release name that the library has already, with
/// different bytes, gives 409 `file_exists`. Studio does not replace a
/// library file (ADR 0034, ADR 0036 item 5).
#[tokio::test]
async fn a_second_import_with_different_bytes_at_the_release_name_gives_file_exists() {
    let app = app();
    let id = set_up(&app).await;
    let mp3_bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, _) = upload(&app, id, "tagged.mp3", mp3_bytes).await;
    assert_eq!(status, StatusCode::CREATED);

    let wav_bytes = std::fs::read(media_path("tagged.wav")).unwrap();
    let (status, body) = upload(&app, id, "tagged.wav", wav_bytes).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "file_exists");
    assert!(incoming_is_empty(&app));
}

/// After a write of a WAV track, with its MP3 enclosure and its FLAC
/// alternative enclosure in the draft, a second upload of the same WAV
/// file finds both files on the host already, through the host
/// inventory, and adds no new library file row (ADR 0034, ADR 0036
/// items 6 and 9).
#[tokio::test]
async fn a_second_upload_of_the_same_wav_file_after_a_write_gives_on_host_true() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.wav")).unwrap();

    let (status, body) = upload(&app, id, "tagged.wav", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["on_host"], false);
    assert_eq!(body["lossless"]["on_host"], false);
    let main_id = body["library_file_id"].as_i64().unwrap();
    let lossless_id = body["lossless"]["library_file_id"].as_i64().unwrap();

    // Give the track the MP3 enclosure and the FLAC file as an
    // alternative enclosure, and write the draft, so the two files go on
    // the host.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": body["url"],
        "length": body["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    model["items"][0]["alternate_enclosures"] = json!([{
        "media_type": "audio/flac",
        "length": body["lossless"]["size"].to_string(),
        "bitrate": null,
        "title": "Maximum fidelity (lossless)",
        "rel": null,
        "default": null,
        "sources": [{ "uri": body["lossless"]["url"], "content_type": null }],
    }]);
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert!(app.feeds.path().join("album/tagged.flac").exists());

    // A second upload of the same WAV file finds both files on the host
    // already, and adds no new library file row.
    let (status, body) = upload(&app, id, "tagged.wav", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["on_host"], true);
    assert_eq!(body["lossless"]["on_host"], true);
    assert_eq!(body["library_file_id"].as_i64().unwrap(), main_id);
    assert_eq!(
        body["lossless"]["library_file_id"].as_i64().unwrap(),
        lossless_id
    );
    assert!(incoming_is_empty(&app));
}

/// A replacement of the audio of a written track: the new MP3 file has
/// the name of the file from before with different bytes, so it gets the
/// hash name in the library and no 409. After the draft refers to the new
/// files, `delete_unused` puts a deletion of the copy from before into
/// the draft, one time only (ADR 0040 items 3 and 6).
#[tokio::test]
async fn a_replacement_gets_the_hash_name_and_deletes_the_unused_copy() {
    let app = app();
    let id = set_up(&app).await;
    let mp3 = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, first) = upload(&app, id, "tagged.mp3", mp3).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": first["url"],
        "length": first["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let guid = model["items"][0]["guid"].clone();
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");

    // The WAV file gives `tagged.mp3` with different bytes, and
    // `tagged.flac`.
    let wav = std::fs::read(media_path("tagged.wav")).unwrap();
    let (status, replaced) = upload_with(&app, id, "tagged.wav", wav, true).await;
    assert_eq!(status, StatusCode::CREATED, "{replaced}");
    let new_path = replaced["path"].as_str().unwrap();
    let short = new_path
        .strip_prefix("album/tagged.")
        .and_then(|rest| rest.strip_suffix(".mp3"))
        .unwrap_or_default();
    assert!(
        short.len() == 8 && short.chars().all(|c| c.is_ascii_hexdigit()),
        "{new_path}"
    );
    assert_eq!(replaced["lossless"]["path"], "album/tagged.flac");
    assert!(app.data.path().join("library/album/tagged.mp3").exists());
    assert!(incoming_is_empty(&app));

    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": replaced["url"],
        "length": replaced["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["model"]["items"][0]["guid"], guid);

    let operation = json!({ "op": "delete_unused", "urls": [first["url"], replaced["url"]] });
    let operations_path = format!("/api/v1/feeds/{id}/operations");
    let (status, diff) = call(&app, "POST", &operations_path, Some(operation.clone())).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let (status, diff) = call(&app, "POST", &operations_path, Some(operation)).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let deletions: Vec<&Value> = diff["operations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|op| op["kind"] == "delete")
        .collect();
    assert_eq!(deletions.len(), 1, "{diff}");
    assert_eq!(deletions[0]["path"], "album/tagged.mp3");
}

#[tokio::test]
async fn a_text_file_gives_415_and_a_repeated_name_reuses_the_same_library_file() {
    let app = app();
    let id = set_up(&app).await;
    let text = std::fs::read(media_path("not-media.txt")).unwrap();

    let (status, body) = upload(&app, id, "not-media.bin", text).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(body["error"]["code"], "unsupported_media");
    assert!(incoming_is_empty(&app));

    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, first) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(incoming_is_empty(&app));

    // A track of the draft refers to the file.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": first["url"],
        "length": first["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // An import of the same bytes at the same name again reuses the same
    // library file row, even while a track of the draft refers to it
    // (ADR 0036 item 5).
    let (status, second) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{second}");
    assert_eq!(second["library_file_id"], first["library_file_id"]);
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn an_upload_to_a_folder_with_no_public_address_gives_a_placeholder_address() {
    let app = app();
    set_library_root(&app);
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "No address", "root": root })),
    )
    .await;
    let (_, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();

    // ADR 0122 item 12: the file gets a placeholder address, and the file
    // is not on the host yet.
    let (status, body) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["on_host"], false);
    let url = body["url"].as_str().unwrap();
    assert!(url.starts_with("pending:"), "{url}");
    assert_eq!(url, format!("pending:{}", body["path"].as_str().unwrap()));
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn the_held_list_gives_a_placeholder_file_before_and_after_the_write() {
    let app = app();
    set_library_root(&app);
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "No address", "root": root })),
    )
    .await;
    let (status, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(json!({
            "connector_id": connector["id"],
            "path": "my-album.xml",
            "title": "New Album",
            "author": "New Artist",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    let id = feed["id"].as_i64().unwrap();
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, file) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let file_id = file["library_file_id"].as_i64().unwrap();
    let url = file["url"].as_str().unwrap().to_owned();
    assert!(url.starts_with("pending:"), "{url}");

    // ADR 0109 item 5 and ADR 0122 item 14: the held list gives the
    // placeholder before the write, so the player can play the file.
    let (_, held) = call(&app, "GET", "/api/v1/media/held", None).await;
    assert_eq!(held["files"][0]["file"], file_id, "{held}");
    assert_eq!(held["files"][0]["url"], url.as_str());

    // The draft uses the placeholder, and the write puts the file on the folder.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    let item = studio_core::feed::Item {
        title: Some("Song".to_owned()),
        enclosure: Some(studio_core::feed::Enclosure {
            url: Some(url.clone()),
            length: Some(file["size"].to_string()),
            media_type: Some("audio/mpeg".to_owned()),
        }),
        ..studio_core::feed::Item::default()
    };
    model["items"] = json!([serde_json::to_value(item).unwrap()]);
    let (status, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");
    assert!(
        app.feeds
            .path()
            .join(file["path"].as_str().unwrap())
            .exists()
    );

    // After the write the file is on the folder, and the placeholder is still held.
    let (_, held) = call(&app, "GET", "/api/v1/media/held", None).await;
    assert_eq!(held["files"][0]["file"], file_id, "{held}");
    assert_eq!(held["files"][0]["url"], url.as_str());
    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    let row = files["files"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["id"] == file_id))
        .unwrap_or_else(|| panic!("{files}"));
    // The row keeps the placeholder address after the write.
    assert_eq!(row["url"], url.as_str(), "{row}");
}

#[tokio::test]
async fn a_library_file_is_removed_and_a_missing_one_gives_404() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, body) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED);
    let file_id = body["library_file_id"].as_i64().unwrap();

    let delete_path = format!("/api/v1/feeds/{id}/files?file={file_id}");
    let (status, _) = call(&app, "DELETE", &delete_path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // The row is already gone, so a second removal gives 404.
    let (status, body) = call(&app, "DELETE", &delete_path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    // The file is also gone from the release folder, so the same name
    // imports again, not the 409 conflict of a name that the library
    // has already.
    let (status, body) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

/// Studio refuses to remove a library file while a track or an image of
/// the draft refers to it (ADR 0036 item 11).
#[tokio::test]
async fn a_delete_of_a_library_file_that_the_draft_refers_to_gives_file_in_use() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, file) = upload(&app, id, "tagged.mp3", bytes).await;
    assert_eq!(status, StatusCode::CREATED);

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    let delete_path = format!("/api/v1/feeds/{id}/files?file={}", file["library_file_id"]);
    let (status, body) = call(&app, "DELETE", &delete_path, None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "file_in_use");
}

/// Add a connector with a public address, for the host of a copy or a
/// move (task 021f).
async fn add_host(app: &App, name: &str, root: &std::path::Path, base_url: &str) -> i64 {
    let (_, connector) = call(
        app,
        "POST",
        "/api/v1/connectors",
        Some(json!({
            "kind": "local",
            "name": name,
            "root": root.to_str().unwrap(),
            "base_url": base_url,
        })),
    )
    .await;
    connector["id"].as_i64().unwrap()
}

#[tokio::test]
async fn the_library_route_moves_the_collection_and_the_files_route_lists_its_files() {
    let app = app();
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    let (_, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();

    let (status, library) = call(&app, "GET", "/api/v1/library", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(library["root"], Value::Null);
    assert_eq!(
        library["download_max_file_bytes"].as_u64().unwrap(),
        2 * 1024 * 1024 * 1024
    );

    let library_root = app.data.path().join("library");
    let (status, report) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": library_root.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["release_folders"], 1);
    assert_eq!(report["feed_files"], 1);
    assert_eq!(report["errors"], json!([]));

    let (_, library) = call(&app, "GET", "/api/v1/library", None).await;
    assert_eq!(library["root"], library_root.to_str().unwrap());

    let (status, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    assert_eq!(files["folder"], "album");
    assert_eq!(files["files"], json!([]));
    assert_eq!(files["other_host_files"], json!([]));

    // A change of an existing root is not available in milestone 7a.
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": app.data.path().join("elsewhere").to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "library_set");
}

#[tokio::test]
async fn a_library_root_that_studio_cannot_write_gives_422() {
    use std::os::unix::fs::PermissionsExt;
    let app = app();
    let parent = app.data.path().join("locked");
    std::fs::create_dir_all(&parent).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o500)).unwrap();
    let root = parent.join("library");

    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": root.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "library_not_writable");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains(root.to_str().unwrap())
    );

    // Restore the permission, so the test folder can be removed.
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[tokio::test]
async fn a_copy_operation_to_a_second_host_gives_a_copy_in_the_diff() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();

    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    // The draft of the feed, with no reference to the file yet.
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "copy", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let operations = diff["operations"].as_array().unwrap();
    assert_eq!(operations.len(), 1, "{operations:?}");
    assert_eq!(operations[0]["kind"], "copy");
    assert_eq!(operations[0]["connector_id"], backup_id);
    assert_eq!(operations[0]["path"], "album/tagged.mp3");
    assert_eq!(
        operations[0]["url"],
        "https://backup.example/album/tagged.mp3"
    );
}

/// ADR 0036 item 12: a move is a copy to the new host, a reference of
/// the new copy in the draft, and a delete of the copy on the old host.
#[tokio::test]
async fn a_move_operation_gives_a_copy_a_reference_and_a_delete() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();
    let from_url = file["url"].as_str().unwrap().to_owned();

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let home_id = connectors[0]["id"].as_i64().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    // The track refers to the file on its first host.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": from_url,
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "move", "file": file_id, "from": home_id, "to": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        operations
            .iter()
            .any(|op| op["kind"] == "copy" && op["connector_id"] == backup_id),
        "{operations:?}"
    );
    // ADR 0148 item 7: the old copy stays, so the draft has no deletion.
    assert!(
        operations
            .iter()
            .any(|op| op["kind"] == "hold" && op["connector_id"] == home_id),
        "{operations:?}"
    );
    assert!(
        !operations.iter().any(|op| op["kind"] == "delete"),
        "{operations:?}"
    );

    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(
        draft["model"]["items"][0]["enclosure"]["url"],
        "https://backup.example/album/tagged.mp3"
    );
}

/// ADR 0036 item 12: a reference to a host with no copy of the file puts
/// the address of that host in the draft, and adds a copy, so the new
/// address has a file after the write. The draft then shows a change
/// also after a reload.
#[tokio::test]
async fn a_reference_to_a_host_with_no_copy_adds_a_copy() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "reference", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        operations
            .iter()
            .any(|op| op["kind"] == "copy" && op["connector_id"] == backup_id),
        "{operations:?}"
    );
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(
        draft["model"]["items"][0]["enclosure"]["url"],
        "https://backup.example/album/tagged.mp3"
    );
    assert_eq!(draft["changed"], true);
}

/// ADR 0036 item 13: Studio refuses a deletion while a feed refers to
/// the address of the file.
#[tokio::test]
async fn a_delete_operation_of_a_file_in_use_gives_a_blocked_entry() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let home_id = connectors[0]["id"].as_i64().unwrap();
    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "delete", "connector_id": home_id, "path": "album/tagged.mp3" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    // The draft still needs the upload of the file itself. The blocked
    // deletion of its only copy is not among the operations to do.
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        !operations.iter().any(|op| op["kind"] == "delete"),
        "{operations:?}"
    );
    let blocked = diff["blocked"].as_array().unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0]["connector_id"], home_id);
    assert_eq!(blocked[0]["path"], "album/tagged.mp3");
    assert_eq!(blocked[0]["feed_titles"], json!([format!("feed {id}")]));
}

/// ADR 0036 item 12: an enclosure with a default alternate enclosure
/// keeps its track playable after a dereference of its own address.
#[tokio::test]
async fn dereference_of_an_enclosure_with_a_default_alternate_enclosure_promotes_its_address() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.wav")).unwrap();
    let (_, file) = upload(&app, id, "tagged.wav", bytes).await;

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    model["items"][0]["alternate_enclosures"] = json!([{
        "media_type": "audio/flac",
        "length": file["lossless"]["size"].to_string(),
        "bitrate": null,
        "title": "Maximum fidelity (lossless)",
        "rel": null,
        "default": "true",
        "sources": [{ "uri": file["lossless"]["url"], "content_type": null }],
    }]);
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "dereference", "url": file["url"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");

    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(
        draft["model"]["items"][0]["enclosure"]["url"],
        file["lossless"]["url"]
    );
}

/// `DELETE /operations/{op}` removes one operation by its ID, so the
/// user can take back a copy or a deletion before the write.
#[tokio::test]
async fn an_operation_is_removed_by_its_id() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();

    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    // The draft of the feed, with no reference to the file yet.
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "copy", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["operations"].as_array().unwrap().len(), 1);

    // The diff gives the ID of each draft operation, so the UI can remove
    // it.
    let operation_id = diff["operations"][0]["operation_id"].as_i64().unwrap();

    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/feeds/{id}/operations/{operation_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["operations"], json!([]));
}

/// ADR 0036 items 10 and 11: after a write, the copy that the enclosure
/// refers to is in use, and a copy that it does not refer to is not.
#[tokio::test]
async fn after_a_write_the_referenced_copy_is_in_use_and_the_other_copy_is_not() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();
    let home_url = file["url"].as_str().unwrap().to_owned();

    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": home_url,
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "copy", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, write_result) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{write_result}");

    // The write removes the draft. Studio starts a fresh one from the
    // live file, which still has the enclosure on the home address.
    call(&app, "GET", &draft_path, None).await;

    let (status, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    let copies = files["files"][0]["copies"].as_array().unwrap();
    assert_eq!(copies.len(), 2, "{copies:?}");
    let home_copy = copies
        .iter()
        .find(|c| c["url"] == home_url)
        .expect("the home copy");
    assert_eq!(home_copy["in_use"], true);
    let backup_copy = copies
        .iter()
        .find(|c| c["connector_id"] == backup_id)
        .expect("the backup copy");
    assert_eq!(backup_copy["in_use"], false);
}

/// ADR 0036 items 6, 11 and 12: a delete operation on a host file that
/// is not a copy of a library file gives that file a pending mark, with
/// the operation ID.
#[tokio::test]
async fn a_delete_operation_marks_an_other_host_file_as_pending() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    upload(&app, id, "tagged.mp3", bytes).await;

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let home_id = connectors[0]["id"].as_i64().unwrap();

    // A file that Studio did not put there: not a copy of the library
    // file, so it is an "other" file of the release folder.
    std::fs::create_dir_all(app.feeds.path().join("album")).unwrap();
    std::fs::write(app.feeds.path().join("album/notes.txt"), b"notes").unwrap();
    let (status, _) = call(&app, "POST", &format!("/api/v1/hosts/{home_id}/scan"), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    let other = files["other_host_files"].as_array().unwrap();
    let notes = other
        .iter()
        .find(|f| f["path"] == "album/notes.txt")
        .expect("the extra file is in the list");
    assert_eq!(notes["in_use"], false);
    assert_eq!(notes["pending"], Value::Null);

    // The draft of the feed, with no operation yet.
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "delete", "connector_id": home_id, "path": "album/notes.txt" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let operation_id = diff["operations"][0]["operation_id"].as_i64().unwrap();

    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    let other = files["other_host_files"].as_array().unwrap();
    let notes = other
        .iter()
        .find(|f| f["path"] == "album/notes.txt")
        .expect("the extra file is in the list");
    assert_eq!(
        notes["pending"],
        json!({ "kind": "delete", "operation_id": operation_id })
    );
}

/// ADR 0036 items 4 and 12: a copy operation gives its library file a
/// pending copy, with the host, the path and the operation ID.
#[tokio::test]
async fn a_copy_operation_gives_the_library_file_a_pending_copy() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(&app, id, "tagged.mp3", bytes).await;
    let file_id = file["library_file_id"].as_i64().unwrap();

    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;

    // The draft of the feed, with no reference to the file yet.
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "copy", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let operation_id = diff["operations"][0]["operation_id"].as_i64().unwrap();

    let (status, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    assert_eq!(
        files["files"][0]["pending_copies"],
        json!([{
            "connector_id": backup_id,
            "path": "album/tagged.mp3",
            "operation_id": operation_id,
        }])
    );
    // The file has no host of its own, so it uses the default host of the
    // release (ADR 0036 item 8).
    assert_eq!(files["files"][0]["own_connector_id"], Value::Null);
}

/// ADR 0021 and ADR 0036 item 11: `PUT /connectors/{id}` changes the
/// name and the public address of a local folder host, and the address
/// of each library file changes with it, as after a folder match.
#[tokio::test]
async fn put_connector_changes_the_name_and_the_address_of_a_local_folder() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    upload(&app, id, "tagged.mp3", bytes).await;

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let home_id = connectors[0]["id"].as_i64().unwrap();

    let (status, connector) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{home_id}"),
        Some(json!({ "name": "Renamed", "base_url": "https://new.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{connector}");

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors[0]["name"], "Renamed");
    let config = connectors[0]["config"].as_str().unwrap().to_owned();
    assert!(config.contains("https://new.example"), "{config}");
    assert!(config.contains("\"base_url_inferred\":false"), "{config}");

    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(
        files["files"][0]["url"],
        "https://new.example/album/tagged.mp3"
    );

    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{home_id}"),
        Some(json!({ "base_url": "ftp://x" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_address");

    // The rejected update did not change the address.
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let config = connectors[0]["config"].as_str().unwrap();
    assert!(config.contains("https://new.example"), "{config}");
}

/// A feed with two enclosures at other servers, for the tests of the
/// remote file records (ADR 0038).
fn feed_with_enclosures(title: &str, second: bool) -> String {
    let item = |guid: &str, name: &str, length: u32| {
        format!(
            "<item><guid>{guid}</guid><enclosure url=\"https://cdn.example/{name}.mp3\" \
             length=\"{length}\" type=\"audio/mpeg\"/></item>"
        )
    };
    let second = if second {
        item("t-2", "b", 200)
    } else {
        String::new()
    };
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>{title}</title><podcast:guid>g-9</podcast:guid>\
         <podcast:medium>music</podcast:medium>{}{second}</channel></rss>",
        item("t-1", "a", 100)
    )
}

#[tokio::test]
async fn a_published_feed_gets_its_folder_its_xml_and_remote_rows_and_a_refresh_keeps_them_current()
{
    let server = studio_core::testing::TestServer::start(Vec::new());
    let xml = feed_with_enclosures("Album C", true);
    server.set("/c.xml", 200, xml.clone().into_bytes());
    let app = app();
    set_library_root(&app);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/c.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["added"][0]["id"].as_i64().unwrap();
    assert_eq!(body["added"][0]["folder"], "c");
    let stored = std::fs::read(app.data.path().join("library/c/c.xml")).unwrap();
    assert_eq!(stored, xml.as_bytes());

    let files_path = format!("/api/v1/feeds/{id}/files");
    let (status, files) = call(&app, "GET", &files_path, None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    let rows = files["files"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row["local"] == false));
    assert_eq!(rows[0]["source_url"], "https://cdn.example/a.mp3");
    assert_eq!(rows[0]["path"], "a.mp3");
    assert_eq!(rows[0]["size"], 100);
    // The feed has no draft, so the route compares with the hosted feed
    // in the release folder.
    assert_eq!(rows[0]["length_in_feed"], 100);
    assert_eq!(rows[0]["in_use"], true);
    assert_eq!(rows[0]["copies"], json!([]));
    assert_eq!(files["remote_count"], 2);
    assert_eq!(files["remote_bytes"], 300);

    // A refresh that removes an enclosure removes its remote row.
    server.set(
        "/c.xml",
        200,
        feed_with_enclosures("Album C", false).into_bytes(),
    );
    let (status, _) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, files) = call(&app, "GET", &files_path, None).await;
    let rows = files["files"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["source_url"], "https://cdn.example/a.mp3");
    assert_eq!(files["remote_count"], 1);
    assert_eq!(files["remote_bytes"], 100);
}

#[tokio::test]
async fn a_published_feed_added_with_no_library_gets_no_folder_and_no_rows() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/c.xml",
        200,
        feed_with_enclosures("Album C", true).into_bytes(),
    );
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/c.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["added"][0]["folder"], Value::Null);
}

#[tokio::test]
async fn a_write_keeps_the_remote_rows_in_agreement_with_the_written_feed() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        feed_with_enclosures("Album", true),
    )
    .unwrap();
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    let (_, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let library_root = app.data.path().join("library");
    let (status, report) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": library_root.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{report}");

    // The start of the library gives a remote row for each address.
    let files_path = format!("/api/v1/feeds/{id}/files");
    let (_, files) = call(&app, "GET", &files_path, None).await;
    assert_eq!(files["remote_count"], 2);

    // The user moves the second enclosure to a new address and writes.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][1]["enclosure"]["url"] = json!("https://cdn.example/c.mp3");
    let (status, draft) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    // The draft reports the length of the enclosure at each address.
    let (_, files) = call(&app, "GET", &files_path, None).await;
    let a = files["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["source_url"] == "https://cdn.example/a.mp3")
        .unwrap();
    assert_eq!(a["length_in_feed"], 100);
    assert_eq!(a["in_use"], true);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, files) = call(&app, "GET", &files_path, None).await;
    let mut urls: Vec<&str> = files["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["source_url"].as_str().unwrap())
        .collect();
    urls.sort_unstable();
    assert_eq!(
        urls,
        ["https://cdn.example/a.mp3", "https://cdn.example/c.mp3"]
    );
}

#[tokio::test]
async fn a_feed_added_from_a_host_folder_gets_its_release_folder_at_its_first_read() {
    let app = app();
    let id = set_up(&app).await;
    std::fs::write(
        app.feeds.path().join("album.xml"),
        feed_with_one_enclosure("https://music.example/album/a.mp3", 10),
    )
    .unwrap();
    let (status, body) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["folder"], "album");
    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(files["remote_count"], 1, "{files}");
    assert!(app.data.path().join("library/album/album.xml").exists());
}

// ---- The download queue (task 023c) ----

/// A feed with one enclosure at `url`, whose length is `length`.
fn feed_with_one_enclosure(url: &str, length: usize) -> String {
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>Album D</title><podcast:guid>g-8</podcast:guid>\
         <podcast:medium>music</podcast:medium>\
         <item><guid>t-1</guid><enclosure url=\"{url}\" length=\"{length}\" \
         type=\"audio/mpeg\"/></item></channel></rss>"
    )
}

/// Ask for the downloads of a feed until the list is empty.
async fn wait_for_downloads(app: &App, id: i64) {
    for _ in 0..200 {
        let (status, body) = call(app, "GET", &format!("/api/v1/feeds/{id}/downloads"), None).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let list = body["downloads"].as_array().unwrap();
        assert!(
            list.iter().all(|d| d["state"] != "failed"),
            "a download failed: {body}"
        );
        if list.is_empty() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("the downloads did not end");
}

/// Add a published feed with one enclosure to a new app, and give the
/// feed ID with the enclosure bytes.
async fn app_with_remote_file(
    length: usize,
) -> (App, i64, studio_core::testing::TestServer, Vec<u8>) {
    let server = studio_core::testing::TestServer::start(Vec::new());
    let bytes: Vec<u8> = (0..5000_u32).map(|i| (i % 253) as u8).collect();
    server.set("/a.mp3", 200, bytes.clone());
    server.set(
        "/d.xml",
        200,
        feed_with_one_enclosure(&server.url("/a.mp3"), length).into_bytes(),
    );
    let app = app();
    set_library_root(&app);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/d.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["added"][0]["id"].as_i64().unwrap();
    (app, id, server, bytes)
}

#[tokio::test]
async fn a_download_over_http_puts_the_file_in_the_library_with_its_sha256() {
    let (app, id, _server, bytes) = app_with_remote_file(5000).await;
    let (status, started) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/downloads"),
        Some(json!({ "all": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(started["queued"].as_array().unwrap().len(), 1);
    wait_for_downloads(&app, id).await;

    let stored = std::fs::read(app.data.path().join("library/d/a.mp3")).unwrap();
    assert_eq!(stored, bytes);
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let file = &collection
        .library_files(studio_core::collection::FeedId(id))
        .unwrap()[0];
    assert!(file.local);
    assert_eq!(file.size, 5000);
    let sha256 = file.sha256.as_deref().unwrap();
    assert_eq!(sha256.len(), 64);
    assert!(sha256.bytes().all(|b| b.is_ascii_hexdigit()));
    assert!(file.stamp.is_some());
    assert_eq!(collection.downloads().unwrap(), Vec::new());

    // The feed has no host. The files route gives the downloaded file
    // with no planned address, and the feed still uses it at its address.
    let (status, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{files}");
    assert_eq!(files["files"][0]["local"], true);
    assert_eq!(files["files"][0]["in_use"], true);
    assert_eq!(files["remote_count"], 0);

    // A file in the library does nothing.
    let (status, again) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/downloads"),
        Some(json!({ "all": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["queued"], json!([]));
}

#[tokio::test]
async fn a_file_larger_than_the_limit_gives_422_download_limit_and_names_the_setting() {
    let (app, id, _server, _) = app_with_remote_file(5000).await;
    let (status, _) = call(
        &app,
        "PUT",
        "/api/v1/library/limits",
        Some(json!({ "download_max_file_bytes": 1000, "download_min_free_bytes": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/downloads"),
        Some(json!({ "all": true })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "download_limit");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("download_max_file_bytes")
    );
    let (_, list) = call(&app, "GET", &format!("/api/v1/feeds/{id}/downloads"), None).await;
    assert_eq!(list["downloads"], json!([]));
}

#[tokio::test]
async fn a_download_stops_and_its_record_goes_out() {
    let (app, id, _server, _) = app_with_remote_file(5000).await;
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let file = collection
        .library_files(studio_core::collection::FeedId(id))
        .unwrap()[0]
        .id
        .0;
    // A file of another feed gives 404.
    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/feeds/{}/downloads/{file}", id + 1),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/feeds/{id}/downloads/{file}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(collection.downloads().unwrap(), Vec::new());
}

#[tokio::test]
async fn a_copy_of_a_remote_file_waits_for_its_download_and_the_write_refuses_it() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    let bytes: Vec<u8> = (0..3000_u32).map(|i| (i % 199) as u8).collect();
    // The file is not on the server yet, so the first download fails.
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        feed_with_one_enclosure(&server.url("/a.mp3"), 3000),
    )
    .unwrap();
    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": "https://music.example" })),
    )
    .await;
    let (_, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let library_root = app.data.path().join("library");
    let (status, _) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": library_root.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    let file_id = files["files"][0]["id"].as_i64().unwrap();
    assert_eq!(files["files"][0]["local"], false, "{files}");

    let backup_dir = tempfile::tempdir().unwrap();
    let backup_id = add_host(&app, "Backup", backup_dir.path(), "https://backup.example").await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "copy", "file": file_id, "connector_id": backup_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let copy = &diff["operations"][0];
    assert_eq!(copy["kind"], "copy");
    // The route queued the download, and the first download fails. The
    // write refuses the draft, and no file changes on the host.
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["operations"][0]["waiting"], true, "{diff}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "waiting_for_download");
    assert!(!backup_dir.path().join("album/a.mp3").exists());

    // The file appears on the server. The user starts the download again.
    server.set("/a.mp3", 200, bytes.clone());
    let (status, started) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/downloads"),
        Some(json!({ "files": [file_id] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    wait_for_downloads(&app, id).await;

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["operations"][0]["waiting"], false, "{diff}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        std::fs::read(backup_dir.path().join("album/a.mp3")).unwrap(),
        bytes
    );
}

#[tokio::test]
async fn a_download_row_from_before_a_restart_is_in_the_queue_again() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set("/a.mp3", 200, vec![9; 2000]);
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let file = {
        let mut collection =
            studio_core::collection::Collection::open(&data.path().join("collection.sqlite"))
                .unwrap();
        let root = data.path().join("library");
        collection
            .set_setting("library_root", &json!(root.to_str().unwrap()))
            .unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        collection.set_feed_folder(feed, "d").unwrap();
        let file = collection
            .add_library_file(
                feed,
                "a.mp3",
                2000,
                None,
                "audio/mpeg",
                false,
                Some(&server.url("/a.mp3")),
                None,
            )
            .unwrap();
        collection.queue_download(file).unwrap();
        (feed, file)
    };
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_address_check(std::sync::Arc::new(PassAll));
    workspace.resume_downloads();
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    };
    wait_for_downloads(&app, file.0.0).await;
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    assert!(collection.library_file(file.1).unwrap().local);
    assert_eq!(
        std::fs::read(app.data.path().join("library/d/a.mp3")).unwrap(),
        vec![9; 2000]
    );
}

// ---- The folder scan (task 023e) ----

/// A feed with one library file, `tagged.mp3`, and the home host. Gives
/// the feed ID, the library file ID, the home host ID and the file bytes.
async fn app_with_library_file() -> (App, i64, i64, i64, Vec<u8>) {
    let app = app();
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, file) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    let home = connectors[0]["id"].as_i64().unwrap();
    (
        app,
        id,
        file["library_file_id"].as_i64().unwrap(),
        home,
        bytes,
    )
}

/// Put `bytes` on the home host, at the path of the release, and let
/// Studio list the host.
async fn put_on_home(app: &App, home: i64, name: &str, bytes: &[u8]) {
    std::fs::create_dir_all(app.feeds.path().join("album")).unwrap();
    std::fs::write(app.feeds.path().join("album").join(name), bytes).unwrap();
    let (status, _) = call(app, "POST", &format!("/api/v1/hosts/{home}/scan"), None).await;
    assert_eq!(status, StatusCode::OK);
}

fn library_path(app: &App, name: &str) -> std::path::PathBuf {
    app.data.path().join("library/album").join(name)
}

async fn change(app: &App, id: i64, body: Value) -> (StatusCode, Value) {
    call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/library-changes"),
        Some(body),
    )
    .await
}

fn collection_of(app: &App) -> studio_core::collection::Collection {
    studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite")).unwrap()
}

#[tokio::test]
async fn the_scan_route_gives_a_new_a_missing_and_a_changed_file_with_the_feed_title() {
    let (app, id, file, _home, mut bytes) = app_with_library_file().await;
    call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    let (status, body) = call(&app, "POST", "/api/v1/library/scan", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["differences"], json!([]));

    std::fs::write(library_path(&app, "new.mp3"), b"new").unwrap();
    bytes.push(0);
    std::fs::write(library_path(&app, "tagged.mp3"), &bytes).unwrap();
    let (_, body) = call(&app, "POST", "/api/v1/library/scan", None).await;
    let list = body["differences"].as_array().unwrap();
    assert_eq!(list.len(), 2, "{body}");
    let new = list.iter().find(|d| d["kind"] == "new").unwrap();
    assert_eq!(new["path"], "new.mp3");
    assert_eq!(new["feed_id"], id);
    assert_eq!(new["feed_title"], "Album");
    let changed = list.iter().find(|d| d["kind"] == "changed").unwrap();
    assert_eq!(changed["file"], file);
    assert_eq!(changed["path"], "tagged.mp3");

    std::fs::remove_file(library_path(&app, "tagged.mp3")).unwrap();
    let (_, body) = call(&app, "POST", "/api/v1/library/scan", None).await;
    let missing = body["differences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["kind"] == "missing")
        .cloned()
        .unwrap();
    assert_eq!(missing["file"], file);
}

#[tokio::test]
async fn add_gives_a_library_file_that_the_feed_does_not_use_and_a_repeat_gives_409() {
    let (app, id, _file, _home, _bytes) = app_with_library_file().await;
    std::fs::write(library_path(&app, "extra.mp3"), b"extra").unwrap();

    let (status, body) = change(&app, id, json!({ "action": "add", "path": "extra.mp3" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["differences"], json!([]));
    let files = collection_of(&app)
        .library_files(studio_core::collection::FeedId(id))
        .unwrap();
    let extra = files.iter().find(|f| f.path == "extra.mp3").unwrap();
    assert!(extra.local);
    assert_eq!(extra.size, 5);
    assert_eq!(extra.media_type, "audio/mpeg");
    let (_, listed) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    let entry = listed["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "extra.mp3")
        .unwrap();
    assert_eq!(entry["in_use"], false);

    let (status, body) = change(&app, id, json!({ "action": "add", "path": "extra.mp3" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "no_change");
}

#[tokio::test]
async fn forget_keeps_the_row_of_a_file_with_a_copy_and_removes_the_row_of_a_file_with_none() {
    let (app, id, file, home, bytes) = app_with_library_file().await;
    std::fs::write(library_path(&app, "lone.mp3"), b"lone").unwrap();
    change(&app, id, json!({ "action": "add", "path": "lone.mp3" })).await;
    put_on_home(&app, home, "tagged.mp3", &bytes).await;
    std::fs::remove_file(library_path(&app, "tagged.mp3")).unwrap();
    std::fs::remove_file(library_path(&app, "lone.mp3")).unwrap();

    let (status, body) = change(&app, id, json!({ "action": "forget", "file": file })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let files = collection_of(&app)
        .library_files(studio_core::collection::FeedId(id))
        .unwrap();
    let kept = files.iter().find(|f| f.path == "tagged.mp3").unwrap();
    assert!(!kept.local);
    assert!(files.iter().any(|f| f.path == "lone.mp3"));

    let lone = files.iter().find(|f| f.path == "lone.mp3").unwrap().id.0;
    let (status, body) = change(&app, id, json!({ "action": "forget", "file": lone })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let files = collection_of(&app)
        .library_files(studio_core::collection::FeedId(id))
        .unwrap();
    assert!(files.iter().all(|f| f.path != "lone.mp3"));
}

#[tokio::test]
async fn download_and_restore_put_the_recorded_bytes_back_from_a_copy_on_a_host() {
    let (app, id, file, home, bytes) = app_with_library_file().await;
    put_on_home(&app, home, "tagged.mp3", &bytes).await;

    // A missing file comes back.
    std::fs::remove_file(library_path(&app, "tagged.mp3")).unwrap();
    let (status, body) = change(&app, id, json!({ "action": "download", "file": file })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    wait_for_downloads(&app, id).await;
    assert_eq!(
        std::fs::read(library_path(&app, "tagged.mp3")).unwrap(),
        bytes
    );
    assert!(
        collection_of(&app)
            .library_file(studio_core::collection::LibraryFileId(file))
            .unwrap()
            .local
    );

    // A changed file goes back to the recorded bytes.
    std::fs::write(library_path(&app, "tagged.mp3"), b"other bytes").unwrap();
    let (_, scan) = call(&app, "POST", "/api/v1/library/scan", None).await;
    assert_eq!(scan["differences"][0]["kind"], "changed");
    let (status, body) = change(&app, id, json!({ "action": "restore", "file": file })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    wait_for_downloads(&app, id).await;
    assert_eq!(
        std::fs::read(library_path(&app, "tagged.mp3")).unwrap(),
        bytes
    );
    let (_, scan) = call(&app, "POST", "/api/v1/library/scan", None).await;
    assert_eq!(scan["differences"], json!([]));
}

#[tokio::test]
async fn restore_with_no_copy_gives_409_and_changes_nothing() {
    let (app, id, file, _home, _bytes) = app_with_library_file().await;
    std::fs::write(library_path(&app, "tagged.mp3"), b"other bytes").unwrap();

    let (status, body) = change(&app, id, json!({ "action": "restore", "file": file })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "no_copy");
    assert_eq!(
        std::fs::read(library_path(&app, "tagged.mp3")).unwrap(),
        b"other bytes"
    );
    assert!(
        collection_of(&app)
            .library_file(studio_core::collection::LibraryFileId(file))
            .unwrap()
            .local
    );

    // A decision for a difference that is not there gives 409 too.
    let (status, body) = change(&app, id, json!({ "action": "forget", "file": file })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "no_change");
}

#[tokio::test]
async fn use_new_changes_the_draft_to_the_hash_name_and_the_write_leaves_only_the_new_file() {
    let (app, id, file, home, bytes) = app_with_library_file().await;
    let old_url = "https://music.example/album/tagged.mp3";

    // The draft uses the file, and the write puts it on the host.
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": old_url,
        "length": bytes.len().to_string(),
        "media_type": "audio/mpeg",
    });
    call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");
    assert_eq!(
        std::fs::read(app.feeds.path().join("album/tagged.mp3")).unwrap(),
        bytes
    );
    let (status, _) = call(&app, "POST", &format!("/api/v1/hosts/{home}/scan"), None).await;
    assert_eq!(status, StatusCode::OK);

    // The user replaces the file in the library folder.
    let mut new_bytes = bytes.clone();
    new_bytes.extend_from_slice(b"more");
    std::fs::write(library_path(&app, "tagged.mp3"), &new_bytes).unwrap();

    let (status, body) = change(&app, id, json!({ "action": "use_new", "file": file })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["differences"], json!([]));

    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let enclosure = &draft["model"]["items"][0]["enclosure"];
    let new_url = enclosure["url"].as_str().unwrap();
    assert!(
        new_url.starts_with("https://music.example/album/tagged.") && new_url != old_url,
        "{new_url}"
    );
    assert_eq!(enclosure["length"], new_bytes.len().to_string());

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        operations.iter().any(|op| op["kind"] == "upload"
            && op["path"].as_str().unwrap().starts_with("album/tagged.")
            && op["path"] != "album/tagged.mp3"),
        "{operations:?}"
    );
    assert!(
        operations
            .iter()
            .any(|op| op["kind"] == "delete" && op["path"] == "album/tagged.mp3"),
        "{operations:?}"
    );
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");

    // The host has only the new file.
    let on_host: Vec<_> = std::fs::read_dir(app.feeds.path().join("album"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(on_host.len(), 1, "{on_host:?}");
    assert_ne!(on_host[0], "tagged.mp3");
    assert_eq!(
        std::fs::read(app.feeds.path().join("album").join(&on_host[0])).unwrap(),
        new_bytes
    );
}

#[tokio::test]
async fn the_feed_host_route_puts_a_feed_move_into_the_diff() {
    let app = app();
    let id = set_up(&app).await;
    std::fs::write(
        app.feeds.path().join("album.xml"),
        "<rss version=\"2.0\"><channel><title>Album</title></channel></rss>\n",
    )
    .unwrap();
    let new_dir = tempfile::tempdir().unwrap();
    let new_id = add_host(&app, "New", new_dir.path(), "https://new.example").await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;

    let (status, diff) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": new_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["feed_move"]["connector_id"], new_id);
    assert_eq!(diff["feed_move"]["path"], "album.xml");
    assert_eq!(diff["feed_move"]["url"], "https://new.example/album.xml");
    assert_eq!(
        diff["feed_move"]["from_url"],
        "https://music.example/album.xml"
    );
    assert_eq!(diff["feed_move"]["method"], "pointer");
    assert_eq!(
        diff["announcements"][0]["url"],
        "https://new.example/album.xml"
    );
    assert_eq!(diff["announce_skip"], Value::Null);

    // A host that has a file at the path gives 409.
    let busy_dir = tempfile::tempdir().unwrap();
    std::fs::write(busy_dir.path().join("album.xml"), "other").unwrap();
    let busy_id = add_host(&app, "Busy", busy_dir.path(), "https://busy.example").await;
    let (status, _) = call(&app, "POST", &format!("/api/v1/hosts/{busy_id}/scan"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": busy_id })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "file_exists");
}

/// The body of a new SFTP host on a test server. The login is the plain key.
fn sftp_body(server: &mut studio_core::testing::sshd::TestSshd) -> Value {
    json!({
        "kind": "sftp",
        "name": "Server",
        "host": "127.0.0.1",
        "port": server.port(),
        "user": server.user(),
        "root": server.root().to_str().unwrap(),
        "base_url": "https://cdn.example.com/feeds",
        "login": "key",
        "private_key": std::fs::read_to_string(server.key_path()).unwrap(),
        "host_key": server.host_key_fingerprint(),
    })
}

#[tokio::test]
async fn the_fingerprint_route_gives_the_host_key_without_a_vault() {
    let server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/fingerprint",
        Some(json!({ "host": "127.0.0.1", "port": server.port() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["fingerprint"], server.host_key_fingerprint());
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/fingerprint",
        Some(json!({ "host": " ", "port": 22 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_config");
    let closed = {
        let other = studio_core::testing::sshd::TestSshd::start();
        other.port()
    };
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/fingerprint",
        Some(json!({ "host": "127.0.0.1", "port": closed })),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "host_unavailable");
}

#[tokio::test]
async fn an_sftp_host_is_added_after_the_connection_and_keeps_its_key_in_the_vault() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    make_vault(&app).await;
    let key = std::fs::read_to_string(server.key_path()).unwrap();
    let body = sftp_body(&mut server);
    let (status, _, text) = send(&app, request("POST", "/api/v1/connectors", Some(body))).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert!(!text.contains(key.trim()));
    let row: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(row["kind"], "sftp");
    let secret_ref = row["secret_ref"].as_str().unwrap();
    assert!(secret_ref.starts_with("connector-"), "{secret_ref}");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["host_key"], server.host_key_fingerprint());
    assert_eq!(config["login"], "key");
    assert!(config.get("private_key").is_none());

    let (_, _, list) = send(&app, request("GET", "/api/v1/connectors", None)).await;
    assert!(!list.contains(key.trim()));
    let key_body = key
        .lines()
        .find(|l| l.len() > 20 && !l.starts_with("-----"))
        .unwrap();
    assert!(!data_files_text(&app).contains(key_body));
    let vault_file = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert!(!vault_file.contains(key_body));
    let vault = studio_core::vault::VaultFile::from_json(&vault_file)
        .unwrap()
        .open_with_passphrase(PASSPHRASE)
        .unwrap();
    let secret: Value = serde_json::from_slice(&vault.get(secret_ref).unwrap().unwrap()).unwrap();
    assert_eq!(secret["private_key"], key);
}

#[tokio::test]
async fn an_sftp_host_with_a_wrong_host_key_or_root_changes_neither_vault_nor_collection() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    make_vault(&app).await;
    let vault_before = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    let data_before = data_files_text(&app);

    let mut body = sftp_body(&mut server);
    body["host_key"] = json!("SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(answer["error"]["code"], "host_unauthorized");
    assert!(
        answer["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&server.host_key_fingerprint())
    );

    // A root that does not exist, and a root that is a file.
    let mut missing = sftp_body(&mut server);
    let root = missing["root"].as_str().unwrap().to_owned();
    std::fs::write(format!("{root}/file.txt"), b"x").unwrap();
    for bad in [format!("{root}/missing"), format!("{root}/file.txt")] {
        missing["root"] = json!(bad);
        let (status, answer) =
            call(&app, "POST", "/api/v1/connectors", Some(missing.clone())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
        assert_eq!(answer["error"]["code"], "folder_not_found", "{bad}");
    }

    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
    let vault_after = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert_eq!(vault_before, vault_after);
    assert_eq!(data_before.len(), data_files_text(&app).len());
}

#[tokio::test]
async fn an_sftp_host_with_a_closed_port_gives_503() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    make_vault(&app).await;
    let mut body = sftp_body(&mut server);
    drop(server);
    body["port"] = json!(body["port"].as_u64().unwrap());
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer["error"]["code"], "host_unavailable");
}

#[tokio::test]
async fn an_sftp_host_needs_the_vault_for_a_key_but_not_for_the_agent_checks() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    let body = sftp_body(&mut server);
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(answer["error"]["code"], "vault_required");
    make_vault(&app).await;
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body.clone())).await;
    assert_eq!(status, StatusCode::LOCKED);
    assert_eq!(answer["error"]["code"], "vault_locked");

    // The agent login needs no vault, so the field checks come first.
    let mut agent = body;
    agent["login"] = json!("agent");
    agent["user"] = json!("");
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(agent)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(answer["error"]["code"], "invalid_config");
}

#[tokio::test]
async fn an_sftp_host_with_a_bad_field_gives_422_before_any_connection() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    make_vault(&app).await;
    let cases = [
        ("host", json!(" "), "invalid_config"),
        ("user", json!(""), "invalid_config"),
        ("root", json!(""), "invalid_config"),
        ("host_key", json!(""), "invalid_config"),
        ("private_key", json!(""), "invalid_config"),
        ("base_url", json!("cdn.example.com"), "invalid_address"),
    ];
    for (field, value, code) in cases {
        let mut body = sftp_body(&mut server);
        body[field] = value;
        let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}");
        assert_eq!(answer["error"]["code"], code, "{field}");
    }
    let mut body = sftp_body(&mut server);
    body["login"] = json!("password");
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(answer["error"]["code"], "invalid_config");
    let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
}

#[tokio::test]
async fn an_sftp_host_accepts_a_new_host_key_only_when_the_server_gives_it() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app();
    make_vault(&app).await;
    let body = sftp_body(&mut server);
    let (_, row) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    let id = row["id"].as_i64().unwrap();
    let old = server.host_key_fingerprint();
    server.with_host_key_change();
    let new = server.host_key_fingerprint();
    assert_ne!(old, new);
    let path = format!("/api/v1/connectors/{id}");

    let (status, answer) = call(&app, "PUT", &path, Some(json!({ "host_key": old }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(answer["error"]["code"], "host_key_mismatch");
    let (_, rows) = call(&app, "GET", "/api/v1/connectors", None).await;
    let config: Value = serde_json::from_str(rows[0]["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["host_key"], old);

    let (status, answer) = call(&app, "PUT", &path, Some(json!({ "host_key": new }))).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let config: Value = serde_json::from_str(answer["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["host_key"], new);
    // A connection works with the new key.
    let (status, answer) = call(&app, "GET", &format!("{path}/feeds"), None).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
}

#[tokio::test]
async fn an_sftp_host_gets_a_new_public_address_and_the_match_route_links_its_feeds() {
    let web = web_feeds();
    web.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let app = app_with_index(&web.url(""));
    make_vault(&app).await;
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [web.url("/album.xml")] })),
    )
    .await;
    let body = sftp_body(&mut server);
    std::fs::write(
        format!("{}/album.xml", body["root"].as_str().unwrap()),
        FEED,
    )
    .unwrap();
    let (_, connector) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    let id = connector["id"].as_i64().unwrap();
    let path = format!("/api/v1/connectors/{id}");
    let (status, row) = call(
        &app,
        "PUT",
        &path,
        Some(json!({ "base_url": "https://new.example.com/m" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["base_url"], "https://new.example.com/m");
    let (status, answer) = call(
        &app,
        "PUT",
        &path,
        Some(json!({ "base_url": "new.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(answer["error"]["code"], "invalid_address");

    let (status, answer) = call(&app, "POST", &format!("{path}/match"), None).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["linked"][0]["path"], "album.xml");
    assert_eq!(
        answer["folder_address"],
        json!({ "state": "typed", "url": "https://new.example.com/m" })
    );
}

// ---- The signing plan of an nsite host with "My signer" (ADR 0050) ----

/// A feed on an nsite host with "My signer", on two test relays and one
/// test server, with a draft that changes the title.
struct SignerSite {
    app: App,
    id: i64,
    keys: nostr::prelude::Keys,
    // The servers stop when these values drop.
    relays: [studio_core::testing::relay::TestRelay; 2],
    _blossom: studio_core::testing::blossom::TestBlossom,
}

async fn signer_site() -> SignerSite {
    signer_site_for(&"01".repeat(32)).await
}

/// The same as `signer_site`, for the key with this secret.
async fn signer_site_for(secret_hex: &str) -> SignerSite {
    use studio_core::connector::{Connector, Expected, NsiteHost};
    use studio_core::nostr_events::Signing;
    use studio_core::testing::{blossom::TestBlossom, relay::TestRelay};
    let relays = [TestRelay::start(), TestRelay::start()];
    let blossom = TestBlossom::start();
    let keys = studio_core::nostr_events::parse_secret_key(secret_hex).unwrap();
    let config_for = |mode: &str| {
        json!({
            "public_key": keys.public_key().to_hex(),
            "d": "album",
            "relays": relays.iter().map(studio_core::testing::relay::TestRelay::url).collect::<Vec<_>>(),
            "servers": [blossom.url()],
            "signing": mode,
        })
        .to_string()
    };
    let secret = json!({ "secret_key": secret_hex }).to_string().into_bytes();
    let seed =
        NsiteHost::from_config(&config_for("vault"), Some(&secret), Signing::default()).unwrap();
    seed.write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    seed.finish().unwrap();

    let app = app();
    let mut collection = collection_of(&app);
    let connector = collection
        .add_connector(NsiteHost::KIND, "Site", &config_for("signer"), None)
        .unwrap();
    let feed = collection
        .add_feed(
            seed.public_url("album.xml").as_deref(),
            Some((connector, "album.xml")),
        )
        .unwrap();
    set_library_root(&app);
    let id = feed.0;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let mut model = draft["model"].clone();
    model["title"] = json!("Album (remaster)");
    let (status, body) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    SignerSite {
        app,
        id,
        keys,
        relays,
        _blossom: blossom,
    }
}

/// Sign the events of a diff with the key of the site, as JSON.
fn sign_diff_events(keys: &nostr::prelude::Keys, diff: &Value) -> Vec<Value> {
    diff["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let event = &entry["event"];
            let tags: Vec<nostr::prelude::Tag> = event["tags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|tag| {
                    let parts: Vec<String> = tag
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|part| part.as_str().unwrap().to_owned())
                        .collect();
                    nostr::prelude::Tag::parse(parts).unwrap()
                })
                .collect();
            let unsigned = nostr::prelude::UnsignedEvent::new(
                keys.public_key(),
                nostr::prelude::Timestamp::from_secs(event["created_at"].as_u64().unwrap()),
                nostr::prelude::Kind::from_u16(
                    u16::try_from(event["kind"].as_u64().unwrap()).unwrap(),
                ),
                tags,
                event["content"].as_str().unwrap(),
            );
            let signed = studio_core::nostr_events::sign_with_keys(unsigned, keys).unwrap();
            assert_eq!(signed.id.to_hex(), event["id"].as_str().unwrap());
            serde_json::from_str(&signed.as_json()).unwrap()
        })
        .collect()
}

#[tokio::test]
async fn the_diff_of_an_nsite_host_with_a_signer_gives_the_events_to_sign() {
    let site = signer_site().await;
    let (status, diff) = call(
        &site.app,
        "GET",
        &format!("/api/v1/feeds/{}/diff", site.id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let events = diff["events"].as_array().unwrap();
    assert_ne!(events.len(), 0);
    for entry in events {
        assert_eq!(entry["user_signs"], true);
        assert!(entry["connector_id"].is_i64());
        let event = &entry["event"];
        assert_eq!(event["pubkey"], site.keys.public_key().to_hex());
        assert!(event["created_at"].is_u64());
        assert!(event["kind"].is_u64());
        assert!(event["tags"].is_array());
        assert!(event["content"].is_string());
        assert_eq!(event["id"].as_str().unwrap().len(), 64);
    }
}

#[tokio::test]
async fn a_write_with_the_signed_events_works_and_a_write_with_none_gives_409() {
    let site = signer_site().await;
    let id = site.id;
    let (_, diff) = call(&site.app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let write_path = format!("/api/v1/feeds/{id}/write");
    let (status, body) = call(
        &site.app,
        "POST",
        &write_path,
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "signature_required");

    let signed = sign_diff_events(&site.keys, &diff);
    let (status, body) = call(
        &site.app,
        "POST",
        &write_path,
        Some(json!({ "diff_id": diff["id"], "signed_events": signed })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["title"], "Album (remaster)");
}

#[tokio::test]
async fn a_signed_event_that_does_not_parse_gives_422_invalid_event() {
    let site = signer_site().await;
    let id = site.id;
    let (_, diff) = call(&site.app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &site.app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"], "signed_events": [{ "id": "abc" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_event");
}

// ---- The nsite and Blossom host routes (ADR 0048, ADR 0044) ----

/// An app whose lookup relays are `relays`.
fn app_with_lookup(relays: Vec<String>) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_lookup_relays(relays)
        .with_address_check(std::sync::Arc::new(PassAll));
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

/// The address of a relay that no server answers.
fn dead_relay() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("ws://127.0.0.1:{port}")
}

const NOSTR_SECRET_HEX: &str = "0101010101010101010101010101010101010101010101010101010101010101";

fn nostr_public_hex() -> String {
    studio_core::nostr_events::parse_secret_key(NOSTR_SECRET_HEX)
        .unwrap()
        .public_key()
        .to_hex()
}

/// The servers of one nsite host test: two relays and one Blossom server.
struct NsiteServers {
    relays: [studio_core::testing::relay::TestRelay; 2],
    blossom: studio_core::testing::blossom::TestBlossom,
}

fn nsite_servers() -> NsiteServers {
    use studio_core::testing::{blossom::TestBlossom, relay::TestRelay};
    NsiteServers {
        relays: [TestRelay::start(), TestRelay::start()],
        blossom: TestBlossom::start(),
    }
}

impl NsiteServers {
    fn relay_urls(&self) -> Vec<String> {
        self.relays
            .iter()
            .map(studio_core::testing::relay::TestRelay::url)
            .collect()
    }

    /// The body of a new nsite host.
    fn body(&self, signing: &str) -> Value {
        let mut body = json!({
            "kind": "nsite",
            "name": "Site",
            "d": "album",
            "relays": self.relay_urls(),
            "servers": [self.blossom.url()],
            "signing": signing,
        });
        if signing == "vault" {
            body["secret_key"] = json!(NOSTR_SECRET_HEX);
        } else {
            body["public_key"] = json!(nostr_public_hex());
        }
        body
    }
}

async fn assert_no_host(app: &App, vault_before: Option<&str>) {
    let (_, connectors) = call(app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(connectors, json!([]));
    if let Some(before) = vault_before {
        let after = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
        assert_eq!(before, after);
    }
}

#[tokio::test]
async fn the_lookup_route_gives_the_relays_and_the_servers_of_a_key() {
    use nostr::prelude::{Kind, Tag, Timestamp, UnsignedEvent};
    let relay = studio_core::testing::relay::TestRelay::start();
    let keys = studio_core::nostr_events::parse_secret_key(NOSTR_SECRET_HEX).unwrap();
    let event = |kind: u16, tag: Vec<&str>| {
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from_secs(100),
            Kind::Custom(kind),
            [Tag::parse(tag).unwrap()],
            "",
        );
        studio_core::nostr_events::sign_with_keys(unsigned, &keys).unwrap()
    };
    relay.insert(event(10002, vec!["r", "wss://relay.example"]));
    relay.insert(event(10063, vec!["server", "https://blob.example"]));
    let app = app_with_lookup(vec![relay.url()]);
    let public = nostr_public_hex();
    let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key()).unwrap();
    let nsec = nostr::nips::nip19::ToBech32::to_bech32(keys.secret_key()).unwrap();
    for key in [public.clone(), npub.clone(), nsec.clone()] {
        let (status, _, text) = send(
            &app,
            request(
                "POST",
                "/api/v1/connectors/nostr-lists",
                Some(json!({ "key": key })),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{text}");
        assert!(!text.contains(NOSTR_SECRET_HEX) && !text.contains(&nsec));
        let body: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(body["public_key"], public);
        assert_eq!(body["npub"], npub);
        assert_eq!(body["relays"], json!(["wss://relay.example"]));
        assert_eq!(body["servers"], json!(["https://blob.example"]));
    }

    let other = studio_core::nostr_events::parse_secret_key(&"02".repeat(32))
        .unwrap()
        .public_key()
        .to_hex();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/nostr-lists",
        Some(json!({ "key": other })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["relays"], json!([]));
    assert_eq!(body["servers"], json!([]));

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/nostr-lists",
        Some(json!({ "key": "not a key" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_config");
}

#[tokio::test]
async fn the_lookup_route_gives_503_when_no_lookup_relay_answers() {
    let app = app_with_lookup(vec![dead_relay()]);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/nostr-lists",
        Some(json!({ "key": nostr_public_hex() })),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert_eq!(body["error"]["code"], "host_unavailable");
}

#[tokio::test]
async fn an_nsite_host_with_a_vault_key_keeps_the_key_in_the_vault_only() {
    let servers = nsite_servers();
    let app = app();
    make_vault(&app).await;
    let (status, _, text) = send(
        &app,
        request("POST", "/api/v1/connectors", Some(servers.body("vault"))),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert!(!text.contains(NOSTR_SECRET_HEX));
    let row: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(row["kind"], "nsite");
    let secret_ref = row["secret_ref"].as_str().unwrap();
    assert!(secret_ref.starts_with("connector-"), "{secret_ref}");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(config["public_key"], nostr_public_hex());
    assert_eq!(config["d"], "album");
    assert_eq!(config["gateway"], "nsite.lol");
    assert_eq!(config["signing"], "vault");

    let (_, _, list) = send(&app, request("GET", "/api/v1/connectors", None)).await;
    assert!(!list.contains(NOSTR_SECRET_HEX));
    assert!(!data_files_text(&app).contains(NOSTR_SECRET_HEX));
    let vault_file = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    assert!(!vault_file.contains(NOSTR_SECRET_HEX));
    let vault = studio_core::vault::VaultFile::from_json(&vault_file)
        .unwrap()
        .open_with_passphrase(PASSPHRASE)
        .unwrap();
    let secret: Value = serde_json::from_slice(&vault.get(secret_ref).unwrap().unwrap()).unwrap();
    assert_eq!(secret, json!({ "secret_key": NOSTR_SECRET_HEX }));
}

#[tokio::test]
async fn an_nsite_host_and_a_blossom_host_with_a_signer_need_no_vault() {
    let servers = nsite_servers();
    let app = app();
    let (status, row) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("signer")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{row}");
    assert_eq!(row["secret_ref"], Value::Null);
    let (status, row) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({
            "kind": "blossom",
            "name": "Media",
            "public_key": nostr_public_hex(),
            "servers": [servers.blossom.url()],
            "signing": "signer",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{row}");
    assert_eq!(row["kind"], "blossom");
    assert_eq!(row["secret_ref"], Value::Null);
}

#[tokio::test]
async fn a_blossom_host_with_a_vault_key_keeps_the_key_in_the_vault_only() {
    let servers = nsite_servers();
    let app = app();
    make_vault(&app).await;
    let (status, _, text) = send(
        &app,
        request(
            "POST",
            "/api/v1/connectors",
            Some(json!({
                "kind": "blossom",
                "name": "Media",
                "servers": [servers.blossom.url()],
                "signing": "vault",
                "secret_key": NOSTR_SECRET_HEX,
            })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert!(!text.contains(NOSTR_SECRET_HEX));
    let row: Value = serde_json::from_str(&text).unwrap();
    assert!(row["secret_ref"].is_string());
    assert!(!data_files_text(&app).contains(NOSTR_SECRET_HEX));
}

#[tokio::test]
async fn an_nsite_host_with_a_vault_key_needs_the_vault() {
    let servers = nsite_servers();
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("vault")),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_required");
    make_vault(&app).await;
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("vault")),
    )
    .await;
    assert_eq!(status, StatusCode::LOCKED);
    assert_eq!(body["error"]["code"], "vault_locked");
    assert_no_host(&app, None).await;
}

#[tokio::test]
async fn a_secret_key_of_another_public_key_gives_422_key_mismatch_and_changes_nothing() {
    let servers = nsite_servers();
    let app = app();
    make_vault(&app).await;
    let before = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    let mut body = servers.body("vault");
    body["public_key"] = json!(
        studio_core::nostr_events::parse_secret_key(&"02".repeat(32))
            .unwrap()
            .public_key()
            .to_hex()
    );
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert_eq!(answer["error"]["code"], "key_mismatch");
    assert!(!answer.to_string().contains(NOSTR_SECRET_HEX));
    assert_no_host(&app, Some(&before)).await;
}

#[tokio::test]
async fn an_nsite_host_with_a_relay_that_has_no_server_gives_503_and_changes_nothing() {
    let servers = nsite_servers();
    let app = app();
    make_vault(&app).await;
    let before = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    let mut body = servers.body("vault");
    body["relays"] = json!([dead_relay()]);
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{answer}");
    assert_eq!(answer["error"]["code"], "host_unavailable");
    assert_no_host(&app, Some(&before)).await;

    // A Blossom server with no answer gives the same result.
    let mut body = servers.body("vault");
    let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    body["servers"] = json!([format!("http://{}", dead.local_addr().unwrap())]);
    drop(dead);
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{answer}");
    assert_eq!(answer["error"]["code"], "host_unavailable");
    assert_no_host(&app, Some(&before)).await;
}

#[tokio::test]
async fn a_new_nsite_host_with_a_bad_field_gives_422_and_changes_nothing() {
    let servers = nsite_servers();
    let app = app();
    make_vault(&app).await;
    let before = std::fs::read_to_string(app.data.path().join("vault.json")).unwrap();
    let cases = [
        ("secret_key", json!("not a key"), "invalid_config"),
        ("secret_key", json!(null), "invalid_config"),
        ("servers", json!([]), "invalid_config"),
        ("servers", json!(["ftp://blob.example"]), "invalid_address"),
        ("relays", json!([]), "invalid_config"),
        ("relays", json!(["https://relay.example"]), "invalid_config"),
        ("d", json!("Not Valid"), "invalid_config"),
        ("d", json!(null), "invalid_config"),
        ("gateway", json!("not a host"), "invalid_config"),
        ("public_key", json!("xyz"), "invalid_config"),
    ];
    for (field, value, code) in cases {
        let mut body = servers.body("vault");
        body[field] = value;
        let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field} {answer}");
        assert_eq!(answer["error"]["code"], code, "{field}");
    }
    // A signer host needs a public key and takes no secret key.
    let mut body = servers.body("signer");
    body["secret_key"] = json!(NOSTR_SECRET_HEX);
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert_eq!(answer["error"]["code"], "invalid_config");
    let mut body = servers.body("signer");
    body["public_key"] = Value::Null;
    let (status, answer) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert_eq!(answer["error"]["code"], "invalid_config");
    assert_no_host(&app, Some(&before)).await;
}

#[tokio::test]
async fn an_nsite_host_with_no_d_and_a_feed_gets_the_d_of_the_feed_guid() {
    let web = web_feeds();
    web.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let servers = nsite_servers();
    let app = app_with_index(&web.url(""));
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [web.url("/album.xml")] })),
    )
    .await;
    let (_, feeds) = call(&app, "GET", "/api/v1/feeds", None).await;
    let feed_id = feeds[0]["id"].as_i64().unwrap();
    let guid = feeds[0]["podcast_guid"].as_str().unwrap().to_owned();
    let mut body = servers.body("signer");
    body.as_object_mut().unwrap().remove("d");
    body["feed_id"] = json!(feed_id);
    let (status, row) = call(&app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{row}");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(
        config["d"],
        studio_core::nostr_events::site_name_from_guid(&guid)
    );
}

/// An nsite host with "My signer" on the test servers, and the connector ID.
async fn added_nsite(app: &App, servers: &NsiteServers) -> i64 {
    let (status, row) = call(
        app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("signer")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{row}");
    row["id"].as_i64().unwrap()
}

#[tokio::test]
async fn the_site_route_gives_consistent_false_for_a_relay_that_dropped_the_manifest() {
    use studio_core::connector::{Connector, Expected, NsiteHost};
    let servers = nsite_servers();
    let app = app();
    let id = added_nsite(&app, &servers).await;
    let path = format!("/api/v1/connectors/{id}/site");
    let (status, body) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["consistent"], true);
    assert_eq!(body["relays"].as_array().unwrap().len(), 2);
    assert_eq!(body["relays"][0]["id"], Value::Null);
    assert_eq!(body["relays"][0]["error"], Value::Null);

    // The second relay does not keep the manifest of the write.
    servers.relays[1].drop_events(true);
    let config = json!({
        "public_key": nostr_public_hex(),
        "d": "album",
        "relays": servers.relay_urls(),
        "servers": [servers.blossom.url()],
        "signing": "vault",
    })
    .to_string();
    let secret = json!({ "secret_key": NOSTR_SECRET_HEX })
        .to_string()
        .into_bytes();
    let seed = NsiteHost::from_config(
        &config,
        Some(&secret),
        studio_core::nostr_events::Signing::default(),
    )
    .unwrap();
    seed.write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    // The check after the publication finds the difference. The manifest
    // stays on the first relay.
    let _ = seed.finish();
    let (status, body) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["consistent"], false, "{body}");
    assert!(body["relays"][0]["id"].is_string());
    assert!(body["relays"][0]["created_at"].is_u64());
    assert_eq!(body["relays"][1]["id"], Value::Null);
}

#[tokio::test]
async fn the_site_route_refuses_another_kind_and_a_locked_vault() {
    let servers = nsite_servers();
    let app = app();
    let (_, local) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(
            json!({ "kind": "local", "name": "Home", "root": app.feeds.path().to_str().unwrap() }),
        ),
    )
    .await;
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{}/site", local["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_config");

    make_vault(&app).await;
    let (_, row) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("vault")),
    )
    .await;
    let path = format!("/api/v1/connectors/{}/site", row["id"]);
    let (status, _) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK);
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, body) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::LOCKED, "{body}");
    assert_eq!(body["error"]["code"], "vault_locked");
}

#[tokio::test]
async fn a_put_with_new_servers_relays_and_gateway_changes_the_row() {
    let servers = nsite_servers();
    let other = studio_core::testing::blossom::TestBlossom::start();
    let app = app();
    let id = added_nsite(&app, &servers).await;
    let path = format!("/api/v1/connectors/{id}");
    let (status, row) = call(
        &app,
        "PUT",
        &path,
        Some(json!({
            "servers": [servers.blossom.url(), other.url()],
            "relays": [servers.relays[0].url()],
            "gateway": "Gateway.Example",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{row}");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert_eq!(
        config["servers"],
        json!([servers.blossom.url(), other.url()])
    );
    assert_eq!(config["relays"], json!([servers.relays[0].url()]));
    assert_eq!(config["gateway"], "gateway.example");
    assert_eq!(config["d"], "album");

    // A bad field changes nothing.
    for body in [
        json!({ "servers": [] }),
        json!({ "servers": ["ftp://x.example"] }),
        json!({ "relays": ["https://x.example"] }),
        json!({ "gateway": "not a host" }),
    ] {
        let (status, _) = call(&app, "PUT", &path, Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (_, list) = call(&app, "GET", "/api/v1/connectors", None).await;
    let kept: Value = serde_json::from_str(list[0]["config"].as_str().unwrap()).unwrap();
    assert_eq!(kept, config);
}

#[tokio::test]
async fn a_put_field_for_another_kind_gives_422() {
    let servers = nsite_servers();
    let app = app();
    let (_, blossom) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({
            "kind": "blossom",
            "name": "Media",
            "public_key": nostr_public_hex(),
            "servers": [servers.blossom.url()],
            "signing": "signer",
        })),
    )
    .await;
    let path = format!("/api/v1/connectors/{}", blossom["id"]);
    for body in [
        json!({ "relays": [servers.relays[0].url()] }),
        json!({ "gateway": "gateway.example" }),
    ] {
        let (status, answer) = call(&app, "PUT", &path, Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
        assert_eq!(answer["error"]["code"], "invalid_config");
    }
    let (status, row) = call(
        &app,
        "PUT",
        &path,
        Some(json!({ "servers": [servers.blossom.url()] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{row}");

    let (_, local) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(
            json!({ "kind": "local", "name": "Home", "root": app.feeds.path().to_str().unwrap() }),
        ),
    )
    .await;
    let (status, answer) = call(
        &app,
        "PUT",
        &format!("/api/v1/connectors/{}", local["id"]),
        Some(json!({ "servers": [servers.blossom.url()] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
}

#[tokio::test]
async fn the_match_route_links_a_feed_to_its_file_on_an_nsite_host() {
    use studio_core::connector::{Connector, Expected, NsiteHost};
    let web = web_feeds();
    web.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let servers = nsite_servers();
    let app = app_with_index(&web.url(""));
    call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [web.url("/album.xml")] })),
    )
    .await;
    let id = added_nsite(&app, &servers).await;

    // The site has the feed XML at the path album.xml.
    let config = json!({
        "public_key": nostr_public_hex(),
        "d": "album",
        "relays": servers.relay_urls(),
        "servers": [servers.blossom.url()],
        "signing": "vault",
    })
    .to_string();
    let secret = json!({ "secret_key": NOSTR_SECRET_HEX })
        .to_string()
        .into_bytes();
    let seed = NsiteHost::from_config(
        &config,
        Some(&secret),
        studio_core::nostr_events::Signing::default(),
    )
    .unwrap();
    seed.write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    seed.finish().unwrap();

    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/connectors/{id}/match"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["linked"][0]["path"], "album.xml");
    assert_eq!(body["linked"][0]["connector_id"], id);
    assert_eq!(body["folder_address"]["state"], "typed");
    let url = body["folder_address"]["url"].as_str().unwrap();
    assert!(
        url.starts_with("https://") && url.ends_with("album.nsite.lol"),
        "{url}"
    );
}

// ---- The announcements (task 031c) ----

/// An app whose hub is `hub`, with the announcement times `timing`. It also
/// gives the worker, so that a test can wake it.
fn announce_app(
    hub: &studio_core::testing::TestHub,
    timing: studio_core::announce::Timing,
) -> (App, std::sync::Arc<crate::announcer::Announcer>) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_hub(&hub.url())
        .with_announce_timing(timing);
    let announcer = std::sync::Arc::clone(&workspace.announcer);
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    };
    (app, announcer)
}

/// Write the feed once, with the diff of its draft.
async fn write_once(app: &App, id: i64) {
    call(app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    let (_, diff) = call(app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// Make each waiting announcement due and wake the worker.
fn announce_now(app: &App, announcer: &crate::announcer::Announcer) {
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    studio_core::announce::make_due(&collection, crate::announcer::now()).unwrap();
    announcer.wake();
}

/// The announcements of a feed, when `ready` accepts them. Waits for a
/// maximum of 5 seconds.
async fn announcements_when(app: &App, id: i64, ready: impl Fn(&[Value]) -> bool) -> Vec<Value> {
    let mut list = Vec::new();
    for _ in 0..100 {
        let (status, body) = call(
            app,
            "GET",
            &format!("/api/v1/feeds/{id}/announcements"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        list = body.as_array().unwrap().clone();
        if ready(&list) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    list
}

#[tokio::test]
async fn a_write_sends_one_announcement_to_the_hub() {
    let hub = studio_core::testing::TestHub::start();
    let (app, announcer) = announce_app(&hub, studio_core::announce::Timing::DEFAULT);
    let id = set_up(&app).await;
    write_once(&app, id).await;
    announce_now(&app, &announcer);
    let list = announcements_when(&app, id, |l| l.iter().any(|a| a["state"] == "sent")).await;
    assert_eq!(list.len(), 1, "{list:?}");
    assert_eq!(list[0]["state"], "sent");
    let requests = hub.requests();
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(
        requests[0].url.as_deref(),
        Some("https://music.example/album.xml")
    );
    assert_eq!(requests[0].podping.as_deref(), Some("1"));
}

#[tokio::test]
async fn the_choice_off_sends_no_announcement() {
    let hub = studio_core::testing::TestHub::start();
    let (app, announcer) = announce_app(&hub, studio_core::announce::Timing::DEFAULT);
    let id = set_up(&app).await;
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/settings/announcements",
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    write_once(&app, id).await;
    announce_now(&app, &announcer);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    assert_eq!(hub.requests().len(), 0);
    let (_, list) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/announcements"),
        None,
    )
    .await;
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn a_failed_request_waits_and_then_succeeds() {
    let hub = studio_core::testing::TestHub::start();
    hub.fail_next(1);
    let timing = studio_core::announce::Timing {
        retry_waits: [2; 4],
        ..studio_core::announce::Timing::DEFAULT
    };
    let (app, announcer) = announce_app(&hub, timing);
    let id = set_up(&app).await;
    write_once(&app, id).await;
    announce_now(&app, &announcer);
    // A new announcement is also "waiting", with no attempt.
    let list = announcements_when(&app, id, |l| {
        l.iter()
            .any(|a| a["state"] == "waiting" && a["attempts"] == 1)
    })
    .await;
    assert_eq!(list[0]["state"], "waiting", "{list:?}");
    assert_eq!(list[0]["attempts"], 1);
    assert!(
        list[0]["last_error"]
            .as_str()
            .is_some_and(|t| !t.is_empty())
    );
    let list = announcements_when(&app, id, |l| l.iter().any(|a| a["state"] == "sent")).await;
    assert_eq!(list[0]["state"], "sent", "{list:?}");
    assert_eq!(hub.requests().len(), 2);
}

#[tokio::test]
async fn a_wallet_changes_through_its_route_and_only_a_wallet_can() {
    let app = app();
    let (_, person) = call(
        &app,
        "POST",
        "/api/v1/people",
        Some(json!({ "name": "Ann" })),
    )
    .await;
    let id = person["id"].as_i64().unwrap();
    let wallets = format!("/api/v1/people/{id}/wallets");
    let (_, made) = call(
        &app,
        "POST",
        &wallets,
        Some(json!({ "address": "02abc", "recipient_type": "node" })),
    )
    .await;
    let wallet = made["id"].as_i64().unwrap();
    let (status, body) = call(
        &app,
        "PUT",
        &format!("{wallets}/{wallet}"),
        Some(json!({ "address": "ann@example.com", "recipient_type": "node" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, person) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(person["wallets"][0]["key"]["address"], "ann@example.com");
    assert_eq!(person["wallets"][0]["recipient_type"], "lnaddress");
    let (status, _) = call(
        &app,
        "PUT",
        &format!("/api/v1/people/{id}/keys/1"),
        Some(json!({})),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
}

#[tokio::test]
async fn the_desktop_has_never_as_the_start_of_the_lock_time() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/settings/lock", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "minutes": 0, "choices": [0, 5, 10, 30, 60] }));
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/settings/lock",
        Some(json!({ "minutes": 30 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["minutes"], 30);
    let (_, body) = call(&app, "GET", "/api/v1/settings/lock", None).await;
    assert_eq!(body["minutes"], 30);
    let (status, _) = call(
        &app,
        "PUT",
        "/api/v1/settings/lock",
        Some(json!({ "minutes": 7 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_stored_never_from_a_desktop_does_not_count_on_a_server() {
    let app = app_with_rules(1 << 30);
    collection_of(&app)
        .set_setting("vault_lock_minutes", &json!(0))
        .unwrap();
    let (_, body) = call(&app, "GET", "/api/v1/settings/lock", None).await;
    assert_eq!(body, json!({ "minutes": 10, "choices": [5, 10, 30, 60] }));
}

#[tokio::test]
async fn the_announcement_settings_are_on_in_a_new_workspace() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/settings/announcements", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "enabled": true }));
    let (status, _) = call(
        &app,
        "PUT",
        "/api/v1/settings/announcements",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = call(&app, "GET", "/api/v1/feeds/999/announcements", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn resuming_fails_a_waiting_announcement_older_than_a_day() {
    let app = app();
    let id = set_up(&app).await;
    let workspace = Workspace::open(app.data.path()).unwrap();
    let now = crate::announcer::now();
    studio_core::announce::queue(
        &workspace.collection,
        studio_core::collection::FeedId(id),
        &[studio_core::announce::PlannedAnnouncement {
            url: "https://music.example/album.xml".to_owned(),
            podping: true,
        }],
        &studio_core::announce::Timing::DEFAULT,
        now - 2 * 86_400,
    )
    .unwrap();
    workspace.resume_announcements();
    let rows =
        studio_core::announce::recent(&workspace.collection, studio_core::collection::FeedId(id))
            .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state.as_str(), "failed");
}

#[tokio::test]
async fn a_playlist_is_made_with_the_musicl_medium_and_an_empty_name_gives_422() {
    let (app, connector) = publisher_app().await;
    let (status, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(new_feed_body(
            connector,
            "list.xml",
            "Mix",
            Some("musicL"),
            None,
        )),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    assert_eq!(feed["medium"], "musicL");
    assert_eq!(
        draft_model(&app, feed["id"].as_i64().unwrap()).await["medium"],
        "musicL"
    );

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/new",
        Some(new_feed_body(
            connector,
            "list2.xml",
            " ",
            Some("musicL"),
            None,
        )),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["message"], "give the name of the playlist");
}

#[tokio::test]
async fn the_playlist_tracks_route_gives_the_tracks_of_the_collection() {
    let app = app();
    let id = set_up(&app).await;
    let (status, _) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(&app, "GET", "/api/v1/playlist/tracks", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!([{
            "feed_guid": "g-1",
            "feed_url": "https://music.example/album.xml",
            "feed_title": "Album",
            "item_guid": "t-1",
            "title": null,
        }])
    );
}

#[tokio::test]
async fn a_credit_name_with_no_key_can_be_added_and_changed() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/people",
        Some(json!({ "name": "Ann" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let person = body["id"].as_i64().unwrap();
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{person}/credit-names"),
        Some(json!({ "name": "Ann", "href": "", "npub": null })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "the form sends a null key: {body}"
    );
    let row = body["id"].as_i64().unwrap();
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/people/{person}/credit-names/{row}"),
        Some(json!({ "name": "Anna", "href": "https://a.example", "npub": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ADR 0146 item 6: {body}");
    assert_eq!(body["id"], row);
    let (_, body) = call(&app, "GET", &format!("/api/v1/people/{person}"), None).await;
    assert_eq!(body["credit_names"][0]["name"], "Anna");
}

#[tokio::test]
async fn a_route_of_musicindex_names_the_provider_of_a_shared_node() {
    let server = web_feeds();
    server.set(
        "/v1/feeds/b-1",
        200,
        br#"{"data":{"title":"Album B","payment_routes":[
            {"recipient_name":"Ann","route_type":"node","address":"03b6f613e88bd874177c28c6ad83b3baba43c4c656f56be1f8df84669556054b79","split":90,"custom_key":"112111100","custom_value":"abc","fee":false},
            {"recipient_name":"Bob","route_type":"node","address":"02aa","split":10,"fee":false}]}}"#
            .to_vec(),
    );
    let app = app_with_index(&server.url(""));
    let (status, body) = call(&app, "GET", "/api/v1/musicindex/feeds/b-1/recipients", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["recipients"][0]["provider"], "fountain.fm",
        "ADR 0145 item 9"
    );
    assert_eq!(body["recipients"][0]["custom_value"], "abc");
    assert!(
        body["recipients"][1]["provider"].is_null(),
        "ADR 0145 item 9"
    );
}

#[tokio::test]
async fn the_one_search_gives_artists_labels_and_other_feeds() {
    let server = web_feeds();
    server.set(
        "/v1/search",
        200,
        br#"{"data":[{"entity_type":"publisher","entity_id":"c0ffee-01","title":"Ann"},
            {"entity_type":"feed","entity_id":"c0ffee-03","title":"Ann Album","release_artist":"Ann"}]}"#
            .to_vec(),
    );
    server.set(
        "/v1/feeds/c0ffee-01",
        200,
        br#"{"data":{"title":"Ann","image_url":"https://img.example/a.png",
            "source_links":[{"link_type":"website","url":"https://ann.example"}],
            "source_ids":[],"payment_routes":[{"recipient_name":"Ann","route_type":"lnaddress","address":"ann@x.example","split":100,"fee":false}]}}"#
            .to_vec(),
    );
    let app = app_with_index(&server.url(""));
    let (status, body) = call(&app, "GET", "/api/v1/musicindex/search?q=ann", None).await;
    assert_eq!(status, StatusCode::OK, "ADR 0145 item 3: {body}");
    assert_eq!(body["rows"][0]["title"], "Ann");
    assert_eq!(body["feeds"][0]["title"], "Ann Album", "ADR 0145 item 3");
    assert_eq!(body["feeds"][0]["artist"], "Ann");
    let (status, body) = call(&app, "GET", "/api/v1/musicindex/values/c0ffee-01", None).await;
    assert_eq!(status, StatusCode::OK, "ADR 0145 item 3: {body}");
    assert_eq!(body["website"], "https://ann.example");
    assert_eq!(body["routes"][0]["address"], "ann@x.example");
    let (status, _) = call(&app, "GET", "/api/v1/musicindex/values/c0ffee-02", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_musicindex_tracks_route_gives_the_address_and_the_tracks() {
    let server = web_feeds();
    server.set(
        "/v1/feeds/b-1",
        200,
        br#"{"data":{"title":"Album B","feed_url":"https://b.example/feed.xml","payment_routes":[],"tracks":[{"track_guid":"aa11","title":"Track One"}]}}"#.to_vec(),
    );
    let app = app_with_index(&server.url(""));
    let (status, body) = call(&app, "GET", "/api/v1/musicindex/feeds/b-1/tracks", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({
            "feed_guid": "b-1",
            "feed_url": "https://b.example/feed.xml",
            "title": "Album B",
            "tracks": [{ "track_guid": "aa11", "title": "Track One" }],
        })
    );

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/musicindex/feeds/00000000-0000-0000-0000-000000000000/tracks",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_in_index");
}

#[tokio::test]
async fn a_playlist_with_two_remote_items_is_written_with_no_item() {
    let (app, connector) = publisher_app().await;
    let feed = make_feed(
        &app,
        new_feed_body(connector, "list.xml", "Mix", Some("musicL"), None),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["remote_items"] = json!([
        { "feed_guid": "f-1", "feed_url": "https://a.example/a.xml", "item_guid": "i-1",
          "medium": "music", "title": "First" },
        { "feed_guid": "f-2", "feed_url": "https://b.example/b.xml", "item_guid": "i-2",
          "medium": "music", "title": "Second" },
    ]);
    let (status, body) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let xml = std::fs::read_to_string(app.feeds.path().join("list.xml")).unwrap();
    assert!(!xml.contains("<item>"), "{xml}");
    assert_eq!(xml.matches("<podcast:remoteItem").count(), 2, "{xml}");
    let first = xml.find("itemGuid=\"i-1\"").unwrap();
    let second = xml.find("itemGuid=\"i-2\"").unwrap();
    assert!(first < second);
    for needle in [
        "feedGuid=\"f-1\"",
        "feedUrl=\"https://a.example/a.xml\"",
        "title=\"First\"",
        "feedGuid=\"f-2\"",
        "title=\"Second\"",
    ] {
        assert!(xml.contains(needle), "{needle} in {xml}");
    }
    assert_eq!(xml.matches("medium=\"music\"").count(), 2, "{xml}");
}

#[tokio::test]
async fn a_live_item_goes_live_and_ends_through_three_writes() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": " ", "start": "2026-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_live_item");
    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2026-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(live(&app), FEED);
    let entry = &draft["model"]["live_items"][0];
    assert_eq!(entry["status"], "pending");
    let guid = entry["item"]["guid"].as_str().unwrap().to_owned();

    // A show with no content link does not pass the write.
    let mut model = draft["model"].clone();
    model["live_items"][0]["content_links"] =
        json!([{ "href": "https://example.com/show", "label": "Watch" }]);
    let (status, draft) = call(
        &app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    write_once(&app, id).await;
    assert!(live(&app).contains("status=\"pending\""));

    for (next, word) in [("live", "live"), ("ended", "ended")] {
        let (status, draft) = call(
            &app,
            "POST",
            &format!("{base}/live-items/{guid}/status"),
            Some(json!({ "status": next })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{draft}");
        assert_eq!(draft["model"]["live_items"][0]["status"], word);
        // The route does not write.
        assert!(!live(&app).contains(&format!("status=\"{word}\"")));
        write_once(&app, id).await;
        let xml = live(&app);
        assert!(xml.contains(&format!("status=\"{word}\"")), "{xml}");
        assert!(xml.contains(&guid), "{xml}");
    }
}

/// The recording of an ended show: the file goes through the upload route
/// with `replace=true`, the draft puts its enclosure on the live item in
/// place of the stream, and the write puts the file on the host (task 033c).
#[tokio::test]
async fn an_ended_live_item_gets_the_enclosure_of_an_uploaded_recording() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2026-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let guid = draft["model"]["live_items"][0]["item"]["guid"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut model = draft["model"].clone();
    model["live_items"][0]["content_links"] =
        json!([{ "href": "https://example.com/show", "label": "Watch" }]);
    model["live_items"][0]["item"]["enclosure"] = json!({
        "url": "https://example.com/stream.m3u8",
        "length": "0",
        "media_type": "application/x-mpegURL",
    });
    let (status, _) = call(
        &app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    for next in ["live", "ended"] {
        let (status, body) = call(
            &app,
            "POST",
            &format!("{base}/live-items/{guid}/status"),
            Some(json!({ "status": next })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    write_once(&app, id).await;
    assert!(live(&app).contains("stream.m3u8"));

    let mp3 = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, facts) = upload_with(&app, id, "tagged.mp3", mp3, true).await;
    assert_eq!(status, StatusCode::CREATED, "{facts}");
    assert_eq!(facts["kind"], "audio");

    let (_, draft) = call(&app, "GET", &format!("{base}/draft"), None).await;
    let mut model = draft["model"].clone();
    model["live_items"][0]["item"]["enclosure"] = json!({
        "url": facts["url"],
        "length": facts["size"].to_string(),
        "media_type": facts["media_type"],
    });
    let (status, saved) = call(
        &app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    write_once(&app, id).await;
    let xml = live(&app);
    assert!(
        xml.contains("https://music.example/album/tagged.mp3"),
        "{xml}"
    );
    assert!(!xml.contains("stream.m3u8"), "{xml}");
    assert!(xml.contains("status=\"ended\""), "{xml}");
    // The write puts the recording on the host (ADR 0054 item 6).
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
}

#[tokio::test]
async fn a_live_status_cannot_go_back() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (_, draft) = call(
        &app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2026-11-01T10:00:00Z" })),
    )
    .await;
    let guid = draft["model"]["live_items"][0]["item"]["guid"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("{base}/live-items/{guid}/status");
    let (status, body) = call(&app, "POST", &path, Some(json!({ "status": "ended" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "bad_status");
    call(&app, "POST", &path, Some(json!({ "status": "live" }))).await;
    call(&app, "POST", &path, Some(json!({ "status": "ended" }))).await;
    let (status, body) = call(&app, "POST", &path, Some(json!({ "status": "live" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "bad_status");
}

#[tokio::test]
async fn the_listing_route_hides_and_shows_the_draft_and_writes_nothing() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/listing"),
        Some(json!({ "hidden": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["hidden"], true);
    assert_eq!(draft["model"]["itunes_block"], "Yes");
    assert_eq!(live(&app), FEED);
    let (_, diff) = call(&app, "GET", &format!("{base}/diff"), None).await;
    let unified = diff["unified"].as_str().unwrap();
    assert!(
        unified.contains("<itunes:block>Yes</itunes:block>"),
        "{unified}"
    );
    assert!(
        unified.contains("<podcast:block>yes</podcast:block>"),
        "{unified}"
    );

    let (_, again) = call(&app, "GET", &format!("{base}/draft"), None).await;
    assert_eq!(again["hidden"], true);

    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/listing"),
        Some(json!({ "hidden": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["hidden"], false);
    let (_, diff) = call(&app, "GET", &format!("{base}/diff"), None).await;
    let unified = diff["unified"].as_str().unwrap();
    assert!(
        unified.contains("<itunes:block>No</itunes:block>"),
        "{unified}"
    );
    assert!(
        unified.contains("<podcast:block>no</podcast:block>"),
        "{unified}"
    );
}

#[tokio::test]
async fn the_platform_listing_route_changes_only_the_block_of_that_platform() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (_, draft) = call(&app, "GET", &format!("{base}/draft"), None).await;
    let mut model = draft["model"].clone();
    model["blocks"] = json!([
        { "value": "no", "id": null },
        { "value": "no", "id": "spotify" },
        { "value": "no", "id": "apple" },
    ]);
    let (status, draft) = call(
        &app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");

    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/listing/spotify"),
        Some(json!({ "hidden": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let values: Vec<_> = draft["model"]["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["value"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(values, ["no", "yes", "no"]);
    assert_eq!(draft["hidden"], false);

    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/listing/tidal"),
        Some(json!({ "hidden": true })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn the_apps_routes_add_list_and_remove_an_app_and_only_the_post_gives_the_code() {
    let app = app();
    make_vault(&app).await;
    let (status, body) = call(&app, "POST", "/api/v1/apps", Some(json!({"name": " "}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_name");

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/apps",
        Some(json!({"name": "Show tool"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let code = body["code"].as_str().unwrap().to_owned();
    assert_eq!(code.len(), 9);
    assert_eq!(body["state"], "waiting");
    assert!(body.get("token").is_none());
    let id = body["id"].as_i64().unwrap();

    let (status, _, text) = send(&app, request("GET", "/api/v1/apps", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !text.contains(&code),
        "ADR 0110 item 7: the list of apps must not hold the pairing code"
    );
    let list: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(list[0]["name"], "Show tool");
    assert_eq!(list[0]["state"], "waiting");
    assert!(list[0].get("code").is_none());
    assert!(list[0].get("code_hash").is_none());
    assert!(list[0].get("token_hash").is_none());

    let (status, _) = call(&app, "DELETE", &format!("/api/v1/apps/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, "DELETE", &format!("/api/v1/apps/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// An app route refuses each request until 073c builds the signed request.
#[tokio::test]
async fn an_app_route_gives_401_app_key_required_for_a_bearer_header() {
    let app = app();
    let (status, body, _) = send(
        &app,
        Request::builder()
            .uri("/app/v1/whoami")
            .header(header::HOST, HOST)
            .header(header::AUTHORIZATION, "Bearer an-old-token")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body["error"]["code"], "app_key_required",
        "ADR 0110 item 37: an app route must refuse a token with app_key_required"
    );
}

/// The live value has one rule in the core, for the draft save (task 072d).
#[tokio::test]
async fn the_core_refuses_a_long_or_control_live_value_in_a_draft_save() {
    let app = app();
    let id = set_up(&app).await;
    let base = format!("/api/v1/feeds/{id}");
    let (status, draft) = call(
        &app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2099-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let long = "a".repeat(2049);
    let mut model = draft["model"].clone();
    model["live_items"][0]["live_value"] = json!({ "uri": long, "protocol": "socket.io" });
    let (status, body) = call(
        &app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        body["error"]["code"], "invalid_live_value",
        "ADR 0054 item 13: the draft save must refuse a long or control live value. Call live::check_live_values in replace_model."
    );
}

#[tokio::test]
async fn the_session_route_gives_the_local_mode_with_a_loopback_host_and_no_token() {
    let app = app();
    let get_session = |host: &str| {
        Request::builder()
            .uri("/auth/v1/session")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap()
    };
    let (status, value, _) = send(&app, get_session(HOST)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value, json!({ "mode": "local", "collections": false }));
    let (status, value, _) = send(&app, get_session("example.com:4100")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(value["error"]["code"], "host_not_allowed");
}

#[tokio::test]
async fn the_workspace_router_has_no_access_layer() {
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path()).unwrap();
    let (router, _handle) = crate::workspace_router(workspace);
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn the_handle_makes_locks_and_unlocks_the_vault_and_a_vault_with_no_wrapper_stays_in_memory()
{
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path()).unwrap();
    let (_router, handle) = crate::workspace_router(workspace);
    assert_eq!(
        handle.vault_state().await.unwrap(),
        crate::VaultState::Absent
    );
    let recovery = handle.create_vault(Some(PASSPHRASE), true).await.unwrap();
    assert!(recovery.is_some());
    assert_eq!(handle.vault_state().await.unwrap(), crate::VaultState::Open);
    let error = handle
        .create_vault(Some(PASSPHRASE), false)
        .await
        .unwrap_err();
    assert_eq!(error.status(), StatusCode::CONFLICT);
    handle.lock().await.unwrap();
    assert_eq!(
        handle.vault_state().await.unwrap(),
        crate::VaultState::Locked
    );
    handle.unlock_with_passphrase(PASSPHRASE).await.unwrap();
    assert_eq!(handle.vault_state().await.unwrap(), crate::VaultState::Open);

    let other = tempfile::tempdir().unwrap();
    let (_router, handle) = crate::workspace_router(Workspace::open(other.path()).unwrap());
    assert_eq!(handle.create_vault(None, false).await.unwrap(), None);
    assert_eq!(handle.vault_state().await.unwrap(), crate::VaultState::Open);
    assert!(!other.path().join("vault.json").exists());
    handle.lock().await.unwrap();
    assert_eq!(
        handle.vault_state().await.unwrap(),
        crate::VaultState::Absent
    );
}

/// An app with the network policy `Public` and no exception (ADR 0057).
fn public_app() -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_net_policy(studio_core::net::NetPolicy::public());
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

fn assert_not_public(status: StatusCode, body: &Value, host: &str) {
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "address_not_public", "{body}");
    assert_eq!(
        body["error"]["message"],
        format!("Studio on a server cannot connect to {host}, because it is not a public address.")
    );
}

#[tokio::test]
async fn with_the_policy_public_a_feed_and_a_media_address_on_loopback_give_422() {
    let server = web_feeds();
    server.set_file("/a.mp3", vec![0; 10], "audio/mpeg", false);
    let app = public_app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/discover",
        Some(json!({ "addresses": [server.url("/publisher.xml")] })),
    )
    .await;
    assert_not_public(status, &body, "127.0.0.1");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/media/facts",
        Some(json!({ "url": server.url("/a.mp3") })),
    )
    .await;
    assert_not_public(status, &body, "127.0.0.1");
}

#[tokio::test]
async fn with_the_policy_public_an_s3_host_and_an_sftp_host_on_loopback_give_422() {
    let s3 = studio_core::testing::fake_s3::FakeS3::start("bk");
    let mut sshd = studio_core::testing::sshd::TestSshd::start();
    let app = public_app();
    make_vault(&app).await;
    let (status, body) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&s3))).await;
    assert_not_public(status, &body, "127.0.0.1");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(sftp_body(&mut sshd)),
    )
    .await;
    assert_not_public(status, &body, "127.0.0.1");
    let (status, list) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn with_the_policy_public_an_nsite_host_and_the_fingerprint_route_on_loopback_give_422() {
    let servers = nsite_servers();
    let sshd = studio_core::testing::sshd::TestSshd::start();
    let app = public_app();
    make_vault(&app).await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(servers.body("vault")),
    )
    .await;
    assert_not_public(status, &body, "127.0.0.1");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors/fingerprint",
        Some(json!({ "host": "127.0.0.1", "port": sshd.port() })),
    )
    .await;
    assert_not_public(status, &body, "127.0.0.1");
}

// ---- The rules of the server (task 034d) ----

#[tokio::test]
async fn server_rules_refuse_a_local_host_and_a_different_library_folder() {
    let app = app_with_rules(1 << 30);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({
            "kind": "local",
            "name": "Folder",
            "root": app.feeds.path().to_str().unwrap(),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "not_on_server");
    let (_, list) = call(&app, "GET", "/api/v1/connectors", None).await;
    assert_eq!(list, json!([]));

    let other = app.feeds.path().join("other");
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": other.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "not_on_server");
    assert!(!other.exists());

    let fixed = app.data.path().join("library");
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": fixed.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_desktop_keeps_the_local_host() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({
            "kind": "local",
            "name": "Folder",
            "root": app.feeds.path().to_str().unwrap(),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

#[tokio::test]
async fn an_upload_over_the_storage_limit_gives_507_and_leaves_no_partial_file() {
    let app = app_with_rules(1024 * 1024);
    let (status, body) = upload(&app, 1, "big.mp3", vec![7_u8; 2 * 1024 * 1024]).await;
    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE, "{body}");
    assert_eq!(body["error"]["code"], "storage_full");
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn an_upload_starts_only_when_the_folder_is_under_the_limit() {
    // The collection file alone is larger than 1 byte.
    let app = app_with_rules(1);
    let (status, body) = upload(&app, 1, "small.mp3", vec![7_u8; 10]).await;
    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE, "{body}");
    assert_eq!(body["error"]["code"], "storage_full");
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn a_download_over_the_storage_limit_records_storage_full() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set("/a.mp3", 200, vec![1_u8; 5000]);
    server.set(
        "/d.xml",
        200,
        feed_with_one_enclosure(&server.url("/a.mp3"), 5000).into_bytes(),
    );
    let app = app_with_rules(1);
    set_library_root(&app);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/d.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["added"][0]["id"].as_i64().unwrap();
    call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/downloads"),
        Some(json!({ "all": true })),
    )
    .await;
    for _ in 0..200 {
        let (_, body) = call(&app, "GET", &format!("/api/v1/feeds/{id}/downloads"), None).await;
        let list = body["downloads"].as_array().unwrap();
        if let Some(failed) = list.iter().find(|d| d["state"] == "failed") {
            assert!(
                failed["message"]
                    .as_str()
                    .unwrap()
                    .starts_with("storage_full"),
                "{failed}"
            );
            assert!(!app.data.path().join("library/d/a.mp3").exists());
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("the download did not fail");
}

#[tokio::test]
async fn the_handle_reports_the_storage_and_locks_an_idle_vault() {
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_server_rules(crate::ServerRules {
            storage_limit: 1 << 30,
            folder: data.path().to_path_buf(),
            library_root: data.path().join("library"),
            hold_assets: true,
            vault_idle: std::time::Duration::from_secs(600),
            vault_idle_max: std::time::Duration::from_secs(3600),
        })
        .unwrap();
    let (_router, handle) = crate::workspace_router(workspace);
    handle.create_vault(Some(PASSPHRASE), false).await.unwrap();
    assert!(handle.storage_used().await > 0);
    let long = std::time::Duration::from_secs(3600);
    assert!(!handle.lock_if_idle(long).await);
    assert_eq!(handle.vault_state().await.unwrap(), crate::VaultState::Open);
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    assert!(
        handle
            .lock_if_idle(std::time::Duration::from_millis(10))
            .await
    );
    assert_eq!(
        handle.vault_state().await.unwrap(),
        crate::VaultState::Locked
    );
    assert!(!handle.lock_if_idle(std::time::Duration::ZERO).await);
    handle.delete_folder().await.unwrap();
    assert!(!data.path().exists());
}

#[tokio::test]
async fn the_nostr_wrapper_routes_add_read_and_open_a_vault() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "no_nostr_wrapper");
    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "vault_required");

    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = call(&app, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "no_nostr_wrapper");

    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(status, StatusCode::OK);
    let key = body["key"].as_str().unwrap().to_owned();
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(json!({ "pubkey": "aa", "ciphertext": "test-ciphertext" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(&app, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pubkey"], "aa");
    assert_eq!(body["ciphertext"], "test-ciphertext");

    // The wrapper routes work while the vault is closed, and the closed
    // vault refuses the two routes that need it.
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, _) = call(&app, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(status, StatusCode::LOCKED);
    assert!(!body.to_string().contains(&key));
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(json!({ "pubkey": "bb", "ciphertext": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::LOCKED);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock-key",
        Some(json!({ "key": key })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["vault"], "open");
}

#[tokio::test]
async fn unlock_key_with_an_incorrect_key_gives_401_and_the_vault_stays_closed() {
    let app = app();
    call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    for key in [
        "not base64 !",
        "AAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
    ] {
        let (status, body) = call(
            &app,
            "POST",
            "/api/v1/vault/unlock-key",
            Some(json!({ "key": key })),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{key}");
        assert_eq!(body["error"]["code"], "wrong_key");
    }
    let (_, body) = call(&app, "GET", "/api/v1/status", None).await;
    assert_eq!(body["vault"], "locked");
}

// ---- The NIP-46 signer signs the planned events (task 035d) ----

/// A test signer on its own relay, with the key `0x31` repeated.
fn test_signer() -> (
    studio_core::testing::relay::TestRelay,
    studio_core::testing::signer::TestSigner,
) {
    let relay = studio_core::testing::relay::TestRelay::start();
    let signer = studio_core::testing::signer::TestSigner::start(&relay.url());
    (relay, signer)
}

#[tokio::test]
async fn a_nip46_signer_signs_the_planned_events_of_an_nsite_write() {
    let (_signer_relay, signer) = test_signer();
    let site = signer_site_for(&"31".repeat(32)).await;
    let (status, state) = call(&site.app, "GET", "/api/v1/signer", None).await;
    assert_eq!(status, StatusCode::OK, "{state}");
    assert_eq!(state["state"], "none");

    let (status, state) = call(
        &site.app,
        "POST",
        "/api/v1/signer/connect",
        Some(json!({ "bunker": signer.bunker_uri() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{state}");
    assert_eq!(state["state"], "connected");
    assert_eq!(state["pubkey"], signer.public_key().to_hex());
    let (_, state) = call(&site.app, "GET", "/api/v1/signer", None).await;
    assert_eq!(state["state"], "connected");

    let id = site.id;
    let (_, diff) = call(&site.app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let mut events_out = Vec::new();
    for entry in diff["events"].as_array().unwrap() {
        let (status, event) = call(
            &site.app,
            "POST",
            "/api/v1/signer/sign",
            Some(json!({ "event": entry["event"] })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{event}");
        assert_eq!(event["id"], entry["event"]["id"]);
        assert_eq!(event["pubkey"], signer.public_key().to_hex());
        events_out.push(event);
    }
    assert_ne!(events_out.len(), 0);
    let (status, body) = call(
        &site.app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"], "signed_events": events_out })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["title"], "Album (remaster)");
    // The manifest is a replaceable event, so the relay keeps the new one.
    let manifests: Vec<String> = site.relays[0]
        .events()
        .iter()
        .filter(|event| event.kind.as_u16() == 35128)
        .map(|event| event.id.to_hex())
        .collect();
    assert_eq!(manifests.len(), 1);
    assert!(
        events_out
            .iter()
            .any(|event| event["id"].as_str() == Some(manifests[0].as_str()))
    );

    let (status, state) = call(&site.app, "DELETE", "/api/v1/signer", None).await;
    assert_eq!(status, StatusCode::OK, "{state}");
    assert_eq!(state["state"], "none");
}

#[tokio::test]
async fn the_sign_route_refuses_a_wrong_kind_a_different_key_and_no_signer() {
    let (_signer_relay, signer) = test_signer();
    let site = signer_site_for(&"31".repeat(32)).await;
    let key = signer.public_key().to_hex();
    let event = |kind: u64, pubkey: &str| {
        json!({
            "pubkey": pubkey,
            "created_at": 1_700_000_000,
            "kind": kind,
            "tags": [],
            "content": "",
        })
    };
    let sign = |event: Value| {
        call(
            &site.app,
            "POST",
            "/api/v1/signer/sign",
            Some(json!({ "event": event })),
        )
    };
    let (status, body) = sign(event(35128, &key)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "signer_not_connected");

    let (status, body) = call(
        &site.app,
        "POST",
        "/api/v1/signer/connect",
        Some(json!({ "bunker": signer.bunker_uri() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let requests = signer.methods().len();

    let (status, body) = sign(event(1, &key)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "kind_not_permitted");
    let other = studio_core::nostr_events::parse_secret_key(&"02".repeat(32))
        .unwrap()
        .public_key()
        .to_hex();
    let (status, body) = sign(event(35128, &other)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "signer_key_differs");
    let (status, body) = sign(json!({ "kind": 35128 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_event");
    // No refused event went to the signer.
    assert_eq!(signer.methods().len(), requests);
}

#[tokio::test]
async fn a_nostrconnect_address_connects_a_signer_through_the_workspace() {
    let relay = studio_core::testing::relay::TestRelay::start();
    let signer = studio_core::testing::signer::TestSigner::start(&relay.url());
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_nip46_relays(vec![relay.url()]);
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    };
    let (status, started) = call(&app, "POST", "/api/v1/signer/connect", Some(json!({}))).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(started["state"], "pending");
    let uri = started["uri"].as_str().unwrap().to_owned();
    assert!(uri.starts_with("nostrconnect://"));
    assert!(started["qr_svg"].as_str().unwrap().contains("<svg"));
    let (_, state) = call(&app, "GET", "/api/v1/signer", None).await;
    assert_eq!(state["state"], "pending");
    signer.connect(&uri);
    let mut state = Value::Null;
    for _ in 0..100 {
        (_, state) = call(&app, "GET", "/api/v1/signer", None).await;
        if state["state"] == "connected" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert_eq!(state["state"], "connected", "{state}");
    assert_eq!(state["pubkey"], signer.public_key().to_hex());
}

/// Start an upload and give its ID.
async fn start_upload(app: &App, id: i64, name: &str, size: usize) -> String {
    let (status, started) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/uploads"),
        Some(json!({ "name": name, "size": size })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    assert_eq!(started["part_size"], 8 * 1024 * 1024);
    started["upload"].as_str().unwrap().to_owned()
}

/// A file of 20 MiB: a valid MP3 file with silent padding after it.
fn big_mp3() -> Vec<u8> {
    let mut bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    bytes.resize(20 * 1024 * 1024, 0);
    bytes
}

#[tokio::test]
async fn parts_in_any_sequence_with_a_part_twice_give_the_bytes_of_the_source() {
    let app = app();
    let id = set_up(&app).await;
    let bytes = big_mp3();
    let upload = start_upload(&app, id, "tagged.mp3", bytes.len()).await;
    let part = |n: usize| bytes[n * (8 << 20)..bytes.len().min((n + 1) * (8 << 20))].to_vec();
    for n in [2, 0, 1, 1] {
        let (status, body) = put_part(&app, &upload, n, part(n)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, state) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["received"], json!([0, 1, 2]));
    assert_eq!(state["size"], bytes.len());
    assert_eq!(state["feed"], id);
    let (_, listed) = call(&app, "GET", &format!("/api/v1/feeds/{id}/uploads"), None).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let (status, facts) = call(
        &app,
        "POST",
        &format!("/api/v1/uploads/{upload}/finish"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{facts}");
    assert_eq!(facts["size"], bytes.len());
    let stored = std::fs::read(app.data.path().join("library/album/tagged.mp3")).unwrap();
    assert!(stored == bytes, "the stored file differs from the source");
    assert!(incoming_is_empty(&app));
    let (_, listed) = call(&app, "GET", &format!("/api/v1/feeds/{id}/uploads"), None).await;
    assert_eq!(listed, json!([]));
    let (status, _) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn finish_with_a_missing_part_gives_409_and_keeps_the_upload() {
    let app = app();
    let id = set_up(&app).await;
    let upload = start_upload(&app, id, "tagged.mp3", 9 << 20).await;
    let (status, _) = put_part(&app, &upload, 0, vec![0; 8 << 20]).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/uploads/{upload}/finish"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "upload_incomplete");
    let (status, state) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["received"], json!([0]));
}

#[tokio::test]
async fn a_part_of_the_wrong_size_or_outside_the_file_gives_422() {
    let app = app();
    let id = set_up(&app).await;
    let upload = start_upload(&app, id, "tagged.mp3", 9 << 20).await;
    for (n, size) in [
        (0, 100),
        (0, (8 << 20) - 1),
        (1, 8 << 20),
        (1, (1 << 20) - 1),
        (2, 1 << 20),
    ] {
        let (status, body) = put_part(&app, &upload, n, vec![0; size]).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{n} {size} {body}"
        );
        assert_eq!(body["error"]["code"], "bad_part");
    }
    // The body limit of the route is the part size.
    let (status, _) = put_part(&app, &upload, 0, vec![0; (8 << 20) + 1]).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    let (_, state) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(state["received"], json!([]));
}

#[tokio::test]
async fn a_start_checks_the_name_the_size_and_the_feed() {
    let app = app();
    let id = set_up(&app).await;
    let path = format!("/api/v1/feeds/{id}/uploads");
    let (status, body) = call(
        &app,
        "POST",
        &path,
        Some(json!({ "name": "a.mp3", "size": (2_u64 << 30) + 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(body["error"]["code"], "too_large");
    let (status, body) = call(
        &app,
        "POST",
        &path,
        Some(json!({ "name": "..", "size": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/feeds/999/uploads",
        Some(json!({ "name": "a.mp3", "size": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn an_unknown_upload_gives_404_on_each_upload_route() {
    let app = app();
    let id = "00".repeat(16);
    let (status, _) = put_part(&app, &id, 0, vec![1]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    for (method, path) in [
        ("GET", format!("/api/v1/uploads/{id}")),
        ("POST", format!("/api/v1/uploads/{id}/finish")),
        ("DELETE", format!("/api/v1/uploads/{id}")),
    ] {
        let (status, _) = call(&app, method, &path, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}");
    }
}

#[tokio::test]
async fn a_delete_removes_the_row_and_the_file() {
    let app = app();
    let id = set_up(&app).await;
    let upload = start_upload(&app, id, "tagged.mp3", 100).await;
    assert!(!incoming_is_empty(&app));
    let (status, _) = call(&app, "DELETE", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(incoming_is_empty(&app));
    let (status, _) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn two_parts_at_the_same_time_are_both_recorded() {
    let app = app();
    let id = set_up(&app).await;
    let upload = start_upload(&app, id, "tagged.mp3", 40 << 20).await;
    let part = |n: usize| put_part(&app, &upload, n, vec![7; 8 << 20]);
    let results = tokio::join!(part(0), part(1), part(2), part(3), part(4));
    for (status, body) in [results.0, results.1, results.2, results.3, results.4] {
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (_, state) = call(&app, "GET", &format!("/api/v1/uploads/{upload}"), None).await;
    assert_eq!(state["received"], json!([0, 1, 2, 3, 4]));
}

#[tokio::test]
async fn a_start_over_the_storage_limit_gives_507_and_open_uploads_count() {
    let app = app_with_rules(3 << 20);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/1/uploads",
        Some(json!({ "name": "big.mp3", "size": 4 << 20 })),
    )
    .await;
    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE, "{body}");
    assert_eq!(body["error"]["code"], "storage_full");
    assert!(incoming_is_empty(&app));
}

#[tokio::test]
async fn uploads_that_are_open_together_cannot_go_over_the_storage_limit() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/d.xml",
        200,
        feed_with_one_enclosure(&server.url("/a.mp3"), 5000).into_bytes(),
    );
    let app = app_with_rules(10 << 20);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/d.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["added"][0]["id"].as_i64().unwrap();
    let path = format!("/api/v1/feeds/{id}/uploads");
    let mut statuses = Vec::new();
    for _ in 0..4 {
        let (status, _) = call(
            &app,
            "POST",
            &path,
            Some(json!({ "name": "a.mp3", "size": 4 << 20 })),
        )
        .await;
        statuses.push(status);
    }
    assert_eq!(statuses[0], StatusCode::CREATED, "{statuses:?}");
    assert_eq!(statuses[1], StatusCode::CREATED, "{statuses:?}");
    assert_eq!(
        statuses[2],
        StatusCode::INSUFFICIENT_STORAGE,
        "{statuses:?}"
    );
    assert_eq!(
        statuses[3],
        StatusCode::INSUFFICIENT_STORAGE,
        "{statuses:?}"
    );
}

/// Many starts at the same time cannot go over the storage limit (task 072b,
/// ADR 0055 item 21).
#[tokio::test]
async fn starts_at_the_same_time_cannot_go_over_the_storage_limit() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/d.xml",
        200,
        feed_with_one_enclosure(&server.url("/a.mp3"), 5000).into_bytes(),
    );
    let app = app_with_rules(10 << 20);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/feeds/import",
        Some(json!({ "urls": [server.url("/d.xml")] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["added"][0]["id"].as_i64().unwrap();
    let path = format!("/api/v1/feeds/{id}/uploads");
    let start = || {
        call(
            &app,
            "POST",
            &path,
            Some(json!({ "name": "a.mp3", "size": 4 << 20 })),
        )
    };
    let results = tokio::join!(
        start(),
        start(),
        start(),
        start(),
        start(),
        start(),
        start(),
        start()
    );
    let statuses = [
        results.0.0,
        results.1.0,
        results.2.0,
        results.3.0,
        results.4.0,
        results.5.0,
        results.6.0,
        results.7.0,
    ];
    let created = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    assert!(
        created <= 2,
        "ADR 0055 item 21: the check of the room and the staging file must be one step under the lock of the workspace. {created} of 8 starts passed a limit of 2 files: {statuses:?}"
    );
    assert_eq!(created, 2, "{statuses:?}");
}

/// An upload with no new part for 24 hours has no row and no file after
/// the clean step (ADR 0059 item 4).
#[tokio::test]
async fn an_upload_older_than_24_hours_has_no_row_and_no_file_after_the_clean_step() {
    let app = app();
    let id = set_up(&app).await;
    let old = start_upload(&app, id, "old.mp3", 100).await;
    let fresh = start_upload(&app, id, "new.mp3", 100).await;
    let path = app.data.path().join("collection.sqlite");
    let collection = studio_core::collection::Collection::open(&path).unwrap();
    let now = crate::announcer::now();
    collection
        .record_upload_part(&old, 0, now - 25 * 3600)
        .unwrap();
    collection
        .record_upload_part(&fresh, 0, now - 23 * 3600)
        .unwrap();
    drop(collection);

    let workspace = Workspace::open(app.data.path()).unwrap();
    assert_eq!(crate::uploads::clean_expired(&workspace, now).unwrap(), 1);
    let (status, _) = call(&app, "GET", &format!("/api/v1/uploads/{old}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app, "GET", &format!("/api/v1/uploads/{fresh}"), None).await;
    assert_eq!(status, StatusCode::OK);
    let incoming = app.data.path().join("incoming");
    assert!(!incoming.join(format!("{old}.part")).exists());
    assert!(incoming.join(format!("{fresh}.part")).exists());
}

async fn lock(app: &App) {
    let (status, _) = call(app, "POST", "/api/v1/vault/lock", None).await;
    assert_eq!(status, StatusCode::OK);
}

// ---- The file sources of the picker (task 039a, ADR 0062) ----

/// Give the app a release library, so that a feed has a release folder.
async fn set_library(app: &App) {
    let root = app.data.path().join("library");
    let (status, _) = call(
        app,
        "PUT",
        "/api/v1/library",
        Some(json!({ "root": root.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

fn names(list: &Value, key: &str) -> Vec<String> {
    let mut names: Vec<String> = list["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f[key].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn the_release_picker_lists_only_the_files_of_this_feed_of_the_asked_kind() {
    let app = app();
    set_library(&app).await;
    let id = set_up(&app).await;
    for (name, file) in [("image.png", "image.png"), ("tagged.mp3", "tagged.mp3")] {
        let bytes = std::fs::read(media_path(file)).unwrap();
        let (status, body) = upload(&app, id, name, bytes).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    let (_, images) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/picker/release?kind=image"),
        None,
    )
    .await;
    assert_eq!(names(&images, "name"), vec!["image.png"], "{images}");
    let image = &images["files"][0];
    assert_eq!(image["local"], true);
    assert_eq!(image["media_type"], "image/png");
    assert_eq!(image["in_use"], false);
    assert_eq!(image["address"], "https://music.example/album/image.png");
    assert!(image["size"].as_u64().unwrap() > 0);
    let (_, audio) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/picker/release?kind=audio"),
        None,
    )
    .await;
    assert_eq!(names(&audio, "name"), vec!["tagged.mp3"], "{audio}");
    assert_eq!(audio["files"][0]["media_type"], "audio/mpeg");
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/picker/release?kind=video"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn the_release_picker_lists_only_the_lyrics_files_for_lyrics() {
    let app = app();
    set_library(&app).await;
    let id = set_up(&app).await;
    let png = std::fs::read(media_path("image.png")).unwrap();
    let (status, body) = upload(&app, id, "image.png", png).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, body) = upload(
        &app,
        id,
        "words.srt",
        b"1\n00:00:01,000 --> 00:00:02,000\nHi\n".to_vec(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, lyrics) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/picker/release?kind=lyrics"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{lyrics}");
    assert_eq!(names(&lyrics, "name"), vec!["words.srt"], "{lyrics}");
    assert_eq!(lyrics["files"][0]["media_type"], "application/srt");
}

#[tokio::test]
async fn an_upload_of_a_vtt_file_gives_the_lyrics_kind_and_its_media_type() {
    let app = app();
    set_library(&app).await;
    let id = set_up(&app).await;
    let (status, body) = upload(&app, id, "words.vtt", b"WEBVTT\n".to_vec()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["kind"], "lyrics", "{body}");
    assert_eq!(body["media_type"], "text/vtt", "{body}");
    assert_eq!(body["title"], Value::Null);
    assert_eq!(body["track_number"], Value::Null);
    assert_eq!(body["duration"], Value::Null);
}

#[tokio::test]
async fn the_host_picker_lists_only_the_lyrics_files_for_lyrics() {
    let app = app();
    let host = tempfile::tempdir().unwrap();
    fill_host(host.path());
    std::fs::write(host.path().join("words.vtt"), b"WEBVTT\n").unwrap();
    let id = add_host(&app, "Files", host.path(), "https://files.example").await;
    let (status, lyrics) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{id}/picker?kind=lyrics"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{lyrics}");
    assert_eq!(names(&lyrics, "path"), vec!["notes.txt", "words.vtt"]);
    let vtt = lyrics["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "words.vtt")
        .unwrap();
    assert_eq!(vtt["media_type"], "text/vtt");
}

#[tokio::test]
async fn the_release_picker_gives_a_remote_file_its_source_address() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" \
         xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"><channel>\
         <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
         <itunes:image href=\"https://music.example/cover.png\"/>\
         <item><guid>t-1</guid></item></channel></rss>",
    )
    .unwrap();
    set_library(&app).await;
    let id = set_up(&app).await;
    let (status, _) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, images) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{id}/picker/release?kind=image"),
        None,
    )
    .await;
    // The file is beside the feed on the host, not in the planned folder
    // of the release. The library holds no bytes of it, so the write
    // cannot put it in that folder (ADR 0062 item 7).
    let image = &images["files"][0];
    assert_eq!(image["local"], false, "{images}");
    assert_eq!(
        image["address"], "https://music.example/cover.png",
        "{images}"
    );
}

/// Fill a host folder: images, audio files, an XML file and a hidden
/// folder.
fn fill_host(root: &std::path::Path) {
    for name in ["a.jpg", "b.PNG", "c.mp3", "d.flac", "feed.xml", "notes.txt"] {
        std::fs::write(root.join(name), b"x").unwrap();
    }
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("sub/e.webp"), b"x").unwrap();
    std::fs::create_dir_all(root.join(".hidden")).unwrap();
    std::fs::write(root.join(".hidden/f.jpg"), b"x").unwrap();
}

#[tokio::test]
async fn the_host_picker_lists_only_the_asked_kind_and_no_hidden_folder() {
    let app = app();
    let host = tempfile::tempdir().unwrap();
    fill_host(host.path());
    let id = add_host(&app, "Files", host.path(), "https://files.example").await;
    let (status, images) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{id}/picker?kind=image"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{images}");
    assert_eq!(names(&images, "path"), vec!["a.jpg", "b.PNG", "sub/e.webp"]);
    assert_eq!(images["truncated"], false);
    let first = images["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "a.jpg")
        .unwrap();
    assert_eq!(first["address"], "https://files.example/a.jpg");
    assert_eq!(first["size"], 1);
    assert_eq!(first["media_type"], "image/jpeg");
    let png = images["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "b.PNG")
        .unwrap();
    assert_eq!(png["media_type"], "image/png");
    let (_, audio) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{id}/picker?kind=audio"),
        None,
    )
    .await;
    assert_eq!(names(&audio, "path"), vec!["c.mp3", "d.flac"]);
    let mp3 = audio["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "c.mp3")
        .unwrap();
    assert_eq!(mp3["media_type"], "audio/mpeg");
}

#[tokio::test]
async fn the_host_picker_is_truncated_at_the_limit_of_adr_0019() {
    let app = app();
    let host = tempfile::tempdir().unwrap();
    for n in 0..studio_core::connector::LIST_MAX_FILES + 5 {
        std::fs::write(host.path().join(format!("{n:05}.jpg")), b"x").unwrap();
    }
    let id = add_host(&app, "Files", host.path(), "https://files.example").await;
    let (status, images) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{id}/picker?kind=image"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(images["truncated"], true, "truncated");
    assert_eq!(
        images["files"].as_array().unwrap().len(),
        studio_core::connector::LIST_MAX_FILES
    );
}

#[tokio::test]
async fn a_host_with_no_public_address_gives_null_addresses_with_a_reason() {
    let app = app();
    let host = tempfile::tempdir().unwrap();
    fill_host(host.path());
    let (status, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Private", "root": host.path().to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{connector}");
    let id = connector["id"].as_i64().unwrap();
    let (_, images) = call(
        &app,
        "GET",
        &format!("/api/v1/connectors/{id}/picker?kind=image"),
        None,
    )
    .await;
    let files = images["files"].as_array().unwrap();
    assert_eq!(files.len(), 3, "{images}");
    for file in files {
        assert_eq!(file["address"], Value::Null, "{file}");
        assert_eq!(file["reason"], "no_public_address", "{file}");
    }
}

/// A host address in a track image gives a diff with no upload. After the
/// write, the library has a row for the address with `local` false (ADR
/// 0038 items 1 and 3, ADR 0062 item 10).
#[tokio::test]
async fn a_host_address_in_the_draft_uploads_nothing_and_gets_a_remote_row_after_the_write() {
    let app = app();
    set_library(&app).await;
    let id = set_up(&app).await;
    // The first read gives the feed its release folder (ADR 0038 item 4).
    let (status, _) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK);
    let address = "https://other.example/art/cover.jpg";
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["image"] = json!(address);
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let uploads = diff["operations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|op| op["kind"] == "upload")
        .count();
    assert_eq!(uploads, 0, "{diff}");
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");
    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    let row = files["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["source_url"] == address)
        .unwrap_or_else(|| panic!("no library row for the address: {files}"));
    assert_eq!(row["local"], false, "{row}");
}

/// Replace the image of a track and send `delete_unused` with the address
/// from before. When `shared` is true, a second feed of the collection
/// uses the address from before in its draft or in its live file. Gives
/// the number of deletions in the diff.
async fn replace_image_and_delete_unused(shared: bool, in_draft: bool) -> usize {
    let app = app();
    set_library(&app).await;
    let id = set_up(&app).await;
    let bytes = std::fs::read(media_path("image.png")).unwrap();
    let (status, first) = upload(&app, id, "image.png", bytes).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    let before = first["url"].as_str().unwrap().to_owned();

    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["image"] = json!(before);
    call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, written) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{written}");

    if shared {
        // A second feed of the collection uses the address from before.
        let second = FEED.replace("g-1", "g-2").replace(
            "<item><guid>t-1</guid>",
            &format!("<item><guid>t-1</guid><itunes:image href=\"{before}\"/>"),
        );
        let second = second.replace(
            "<rss ",
            "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" ",
        );
        std::fs::write(app.feeds.path().join("second.xml"), second).unwrap();
        let (_, connectors) = call(&app, "GET", "/api/v1/connectors", None).await;
        let connector = connectors[0]["id"].clone();
        let (status, feed) = call(
            &app,
            "POST",
            "/api/v1/feeds",
            Some(json!({ "connector_id": connector, "path": "second.xml" })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{feed}");
        // The first read gives the second feed its release folder with
        // its live file (ADR 0038 item 4).
        let other = feed["id"].as_i64().unwrap();
        let (status, _) = call(
            &app,
            "POST",
            &format!("/api/v1/feeds/{other}/refresh"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        if in_draft {
            // The live file of the second feed has no image. Its draft has.
            let other = feed["id"].as_i64().unwrap();
            let other_path = format!("/api/v1/feeds/{other}/draft");
            let (_, other_draft) = call(&app, "GET", &other_path, None).await;
            let mut other_model = other_draft["model"].clone();
            other_model["items"][0]["image"] = json!(before);
            call(
                &app,
                "PUT",
                &other_path,
                Some(json!({ "revision": other_draft["revision"], "model": other_model })),
            )
            .await;
        }
    }

    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["image"] = json!("https://other.example/new.png");
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "delete_unused", "urls": [before] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    diff["operations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|op| op["kind"] == "delete")
        .count()
}

#[tokio::test]
async fn a_replaced_track_image_gets_one_deletion() {
    assert_eq!(replace_image_and_delete_unused(false, false).await, 1);
}

#[tokio::test]
async fn a_replaced_track_image_that_a_second_feed_uses_in_its_live_file_gets_no_deletion() {
    assert_eq!(replace_image_and_delete_unused(true, false).await, 0);
}

#[tokio::test]
async fn a_replaced_track_image_that_a_second_feed_uses_in_its_draft_gets_no_deletion() {
    assert_eq!(replace_image_and_delete_unused(true, true).await, 0);
}

/// A feed with a channel image whose `purpose` has a token that the UI does
/// not offer, for the test of the artwork list.
const FEED_WITH_ARTWORK: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
    <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
    <podcast:image href=\"https://music.example/banner.png\" alt=\"Old text\" purpose=\"artwork hero banner\" width=\"3000\" height=\"1000\"/>\
    <item><guid>t-1</guid></item></channel></rss>";

#[tokio::test]
async fn a_change_of_the_alt_of_an_image_with_an_unknown_purpose_token_changes_only_the_alt() {
    let app = app();
    std::fs::write(
        app.feeds.path().join("album.xml"),
        FEED_WITH_ARTWORK.as_bytes(),
    )
    .unwrap();
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        draft["model"]["images"][0]["purpose"],
        "artwork hero banner"
    );
    let mut model = draft["model"].clone();
    model["images"][0]["alt"] = json!("New text");
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        live(&app),
        FEED_WITH_ARTWORK.replace("alt=\"Old text\"", "alt=\"New text\"")
    );
}

// ---- A read-only feed with a release folder (ADR 0067) ----

/// Add a read-only feed that reads `address`, and give it a release folder
/// that holds the feed XML that the address serves. Gives the feed ID.
fn read_only_with_folder(app: &App, address: &str) -> i64 {
    let mut collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let id = collection.add_feed(Some(address), None).unwrap();
    let web = studio_core::web::Web::new();
    studio_core::pipeline::refresh_public(&mut collection, &web, id).unwrap();
    collection.set_feed_folder(id, "album").unwrap();
    let library = studio_core::library::Library::open(app.data.path().join("library")).unwrap();
    library
        .write_feed_xml("album", "a.xml", FEED.as_bytes())
        .unwrap();
    id.0
}

#[tokio::test]
async fn the_draft_of_a_read_only_feed_has_no_host_and_a_saved_edit_gives_a_diff() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let draft_path = format!("/api/v1/feeds/{id}/draft");

    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["model"]["title"], "Album");
    assert_eq!(draft["has_host"], false);

    let mut model = draft["model"].clone();
    model["title"] = json!("Album (remaster)");
    let (status, saved) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["changed"], true);
    let (status, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert!(
        diff["unified"]
            .as_str()
            .unwrap()
            .contains("+    <title>Album (remaster)</title>")
            || diff["unified"]
                .as_str()
                .unwrap()
                .contains("Album (remaster)"),
        "{diff}"
    );

    // A write with no host gives 409 `read_only`.
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
}

#[tokio::test]
async fn a_host_choice_of_a_read_only_draft_gives_a_feed_move_and_a_write_moves_the_feed() {
    let server = web_feeds();
    // The new host serves the draft at its new address (ADR 0126 item 15).
    // The feed has no self link, so the draft names itself only by the tag
    // of the move (ADR 0128 item 1).
    let new_url = "https://new.example/album/a.xml";
    let reader = studio_core::move_link::ScriptedReader::empty().with_feed(
        new_url,
        new_url,
        Some("g-1"),
        Some(new_url),
        None,
    );
    let app = app_with_reader(&server.url(""), None, Some(reader));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let host_path = format!("/api/v1/feeds/{id}/feed-host");
    let new_dir = tempfile::tempdir().unwrap();
    let new_id = add_host(&app, "New", new_dir.path(), "https://new.example").await;
    call(&app, "GET", &draft_path, None).await;

    let (status, diff) = call(
        &app,
        "PUT",
        &host_path,
        Some(json!({ "connector_id": new_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["feed_move"]["connector_id"], new_id);
    assert_eq!(diff["feed_move"]["method"], "none");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(draft["has_host"], true);

    // `null` removes the move.
    let (status, diff) = call(
        &app,
        "PUT",
        &host_path,
        Some(json!({ "connector_id": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["feed_move"], Value::Null);
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(draft["has_host"], false);

    // A write of the moved draft gives the feed its host.
    let (_, diff) = call(
        &app,
        "PUT",
        &host_path,
        Some(json!({ "connector_id": new_id })),
    )
    .await;
    let (status, feed) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{feed}");
    assert_eq!(feed["connector_id"], new_id);
    assert_eq!(feed["path"], "album/a.xml");
    assert!(new_dir.path().join("album/a.xml").exists());
}

#[tokio::test]
async fn a_null_host_for_a_feed_with_a_host_gives_422() {
    let app = app();
    let id = set_up(&app).await;
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": null })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn a_folder_match_keeps_the_draft_of_a_read_only_feed() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["title"] = json!("Album (kept)");
    call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;

    let root = app.feeds.path().to_str().unwrap();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/connectors/{}/match", connector["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["linked"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["id"] == id)
    );
    let (status, draft) = call(&app, "GET", &draft_path, None).await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    assert_eq!(draft["model"]["title"], "Album (kept)");
    assert_eq!(draft["has_host"], true);
}

#[tokio::test]
async fn a_link_of_a_publisher_feed_and_a_read_only_release_with_a_folder_gives_409() {
    let server = web_feeds();
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let read_only = read_only_with_folder(&app, &server.url("/a.xml"));
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{}/links", label["id"]),
        Some(json!({ "release": read_only })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
}

#[tokio::test]
async fn with_the_policy_public_the_write_of_a_read_only_draft_on_loopback_gives_the_policy_error()
{
    let server = web_feeds();
    let app = public_app();
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let new_dir = tempfile::tempdir().unwrap();
    let new_id = add_host(&app, "New", new_dir.path(), "https://new.example").await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    let (status, diff) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": new_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    // The core reports the policy error as a failed web read.
    assert_eq!(status, StatusCode::BAD_GATEWAY, "{body}");
    assert_eq!(body["error"]["code"], "network", "{body}");
    assert_eq!(
        body["error"]["message"],
        "Studio on a server cannot connect to 127.0.0.1, because it is not a public address."
    );
    assert!(!new_dir.path().join("album/a.xml").exists());
}

#[tokio::test]
async fn an_upload_for_a_feed_with_no_host_gives_409_read_only() {
    let server = web_feeds();
    let app = app_with_index(&server.url(""));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let bytes = std::fs::read(media_path("image.png")).unwrap();
    let (status, body) = upload(&app, id, "image.png", bytes).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "read_only");
    assert_eq!(
        body["error"]["message"],
        "choose a file host for this feed first, in Review"
    );
}

// ---- The Nostr profile route and the search relay setting (ADR 0074) ----

fn nostr_keys(seed: u8) -> nostr::prelude::Keys {
    nostr::prelude::Keys::new(nostr::prelude::SecretKey::from_slice(&[seed; 32]).unwrap())
}

fn nostr_npub(seed: u8) -> String {
    nostr::nips::nip19::ToBech32::to_bech32(&nostr_keys(seed).public_key()).unwrap()
}

/// A kind 0 event of the key `seed`, with this JSON content.
fn profile_event(seed: u8, time: u64, content: &Value) -> nostr::prelude::Event {
    use nostr::prelude::{Kind, Timestamp, UnsignedEvent};
    let keys = nostr_keys(seed);
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::from_secs(time),
        Kind::Metadata,
        Vec::new(),
        content.to_string(),
    );
    studio_core::nostr_events::sign_with_keys(unsigned, &keys).unwrap()
}

/// An app for the profile route. NIP-05 requests use `http`, so that a
/// test server on loopback can answer them.
fn nostr_app(index: &str, lookup: Vec<String>, public: bool) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let mut workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(index)
        .with_lookup_relays(lookup)
        .with_plain_nip05()
        .with_address_check(std::sync::Arc::new(PassAll));
    if public {
        workspace = workspace.with_net_policy(studio_core::net::NetPolicy::public());
    }
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

/// A server that answers `/.well-known/nostr.json` with these names.
fn nip05_server(names: &Value) -> studio_core::testing::TestServer {
    studio_core::testing::TestServer::start(vec![(
        "/.well-known/nostr.json",
        200,
        json!({ "names": names }).to_string().into_bytes(),
    )])
}

fn nostr_hex(seed: u8) -> String {
    nostr_keys(seed).public_key().to_hex()
}

async fn profiles(app: &App, query: &str) -> Value {
    let path = format!("/api/v1/nostr/profiles?q={}", url_encode(query));
    let (status, body) = call(app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[tokio::test]
async fn resolve_gives_the_profiles_of_the_keys_and_refuses_51_keys() {
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(1, 100, &json!({ "display_name": "Ann" })));
    let app = nostr_app("http://127.0.0.1:9", vec![relay.url()], false);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/nostr/resolve",
        Some(json!({ "npubs": [nostr_npub(1), nostr_npub(2)] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body["profiles"].as_array().unwrap();
    assert_eq!(list.len(), 2, "{body}");
    assert_eq!(list[0]["npub"], nostr_npub(1));
    assert_eq!(list[0]["name"], "Ann");
    assert_eq!(list[1]["name"], Value::Null);
    let many: Vec<String> = (0..51).map(|_| nostr_npub(1)).collect();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/nostr/resolve",
        Some(json!({ "npubs": many })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_query");
}

async fn set_search_relay(app: &App, relay: &str) {
    let (status, body) = call(
        app,
        "PUT",
        "/api/v1/settings/nostr",
        Some(json!({ "search_relay": relay })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_nostr_settings_give_the_start_value_and_refuse_a_bad_relay() {
    let app = app();
    let (_, body) = call(&app, "GET", "/api/v1/settings/nostr", None).await;
    assert_eq!(body["search_relay"], "wss://search.nos.today");
    let (status, body) = call(
        &app,
        "PUT",
        "/api/v1/settings/nostr",
        Some(json!({ "search_relay": "https://search.example" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_relay");
    set_search_relay(&app, "ws://127.0.0.1:9").await;
    let (_, body) = call(&app, "GET", "/api/v1/settings/nostr", None).await;
    assert_eq!(body["search_relay"], "ws://127.0.0.1:9");
    set_search_relay(&app, "").await;
    let (_, body) = call(&app, "GET", "/api/v1/settings/nostr", None).await;
    assert_eq!(body["search_relay"], "");
}

#[tokio::test]
async fn an_empty_query_gives_422() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/nostr/profiles?q=%20", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_query");
}

#[tokio::test]
async fn a_key_gives_its_profile_from_the_lookup_relay() {
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(
        1,
        100,
        &json!({ "name": "old", "display_name": "Old Name" }),
    ));
    relay.insert(profile_event(
        1,
        200,
        &json!({ "name": "ann", "display_name": "Ann Lee", "picture": "https://img.example/a.png" }),
    ));
    let app = nostr_app("http://127.0.0.1:9", vec![relay.url()], false);
    let npub = nostr_npub(1);
    for text in [
        npub.clone(),
        nostr_hex(1),
        format!("nostr:{npub}"),
        format!("https://njump.me/{npub}"),
    ] {
        let body = profiles(&app, &text).await;
        assert_eq!(body["failed"], json!([]), "{text}");
        assert_eq!(body["profiles"].as_array().unwrap().len(), 1, "{text}");
        let profile = &body["profiles"][0];
        assert_eq!(profile["npub"], npub);
        assert_eq!(profile["name"], "Ann Lee");
        assert_eq!(profile["picture"], "https://img.example/a.png");
        assert_eq!(profile["nip05_ok"], false);
        assert_eq!(profile["sources"], json!([]));
    }
    // A key with no kind 0 still gives a profile with the key only.
    let body = profiles(&app, &nostr_npub(9)).await;
    assert_eq!(body["profiles"][0]["npub"], nostr_npub(9));
    assert_eq!(body["profiles"][0]["name"], Value::Null);
}

#[tokio::test]
async fn a_nip05_address_gives_its_profile_and_a_different_key_gives_nip05_ok_false() {
    let server = nip05_server(&json!({ "ann": nostr_hex(1), "bob": nostr_hex(2) }));
    let domain = format!("127.0.0.1:{}", server.port());
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(
        1,
        100,
        &json!({ "display_name": "Ann Lee", "nip05": format!("ann@{domain}") }),
    ));
    // The profile of key 3 claims the name "bob", but the domain gives key 2.
    relay.insert(profile_event(
        3,
        100,
        &json!({ "display_name": "Fake Bob", "nip05": format!("bob@{domain}") }),
    ));
    let app = nostr_app("http://127.0.0.1:9", vec![relay.url()], false);

    let body = profiles(&app, &format!("ann@{domain}")).await;
    assert_eq!(body["profiles"].as_array().unwrap().len(), 1);
    assert_eq!(body["profiles"][0]["npub"], nostr_npub(1));
    assert_eq!(body["profiles"][0]["name"], "Ann Lee");
    assert_eq!(body["profiles"][0]["nip05"], format!("ann@{domain}"));
    assert_eq!(body["profiles"][0]["nip05_ok"], true);

    // The domain gives key 2 for "bob", so the claim of key 3 fails.
    let body = profiles(&app, &nostr_npub(3)).await;
    assert_eq!(body["profiles"][0]["name"], "Fake Bob");
    assert_eq!(body["profiles"][0]["nip05"], format!("bob@{domain}"));
    assert_eq!(body["profiles"][0]["nip05_ok"], false);

    // A name that the domain does not give has no profile.
    let body = profiles(&app, &format!("zed@{domain}")).await;
    assert_eq!(body["profiles"], json!([]));
}

#[tokio::test]
async fn a_nip05_answer_after_a_redirect_gives_no_key() {
    // NIP-05: a client ignores each HTTP redirect of `nostr.json`.
    let target = nip05_server(&json!({ "ann": nostr_hex(1) }));
    let server = studio_core::testing::TestServer::start(vec![(
        "/.well-known/nostr.json",
        302,
        target.url("/.well-known/nostr.json").into_bytes(),
    )]);
    let domain = format!("127.0.0.1:{}", server.port());
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(
        1,
        100,
        &json!({ "display_name": "Ann Lee", "nip05": format!("ann@{domain}") }),
    ));
    let app = nostr_app("http://127.0.0.1:9", vec![relay.url()], false);

    let body = profiles(&app, &format!("ann@{domain}")).await;
    assert_eq!(body["profiles"], json!([]));
    let body = profiles(&app, &nostr_npub(1)).await;
    assert_eq!(body["profiles"][0]["nip05_ok"], false);
}

/// The musicindex test server of a name search: one feed with a feed key
/// and two contributors.
fn nostr_index() -> studio_core::testing::TestServer {
    let guid = "a1b2c3d4-0000-0000-0000-000000000001";
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/v1/search",
        200,
        json!({ "data": [{ "entity_type": "feed", "entity_id": guid, "title": "Songs", "release_artist": "Ann Lee" }] })
            .to_string()
            .into_bytes(),
    );
    server.set(
        &format!("/v1/feeds/{guid}"),
        200,
        json!({ "data": {
            "release_artist": "Ann Lee",
            "source_ids": [
                { "scheme": "nostr_npub", "value": nostr_npub(1) },
                { "scheme": "other", "value": "x" },
            ],
            "source_contributors": [
                { "name": "Ann Other", "npub": nostr_npub(2) },
                { "name": "Cy Unrelated", "npub": nostr_npub(5) },
                { "name": "Ann NoKey", "npub": null },
            ],
        } })
        .to_string()
        .into_bytes(),
    );
    server
}

#[tokio::test]
async fn a_name_gives_profiles_from_three_sources_merged_by_key() {
    let index = nostr_index();
    let lookup = studio_core::testing::relay::TestRelay::start();
    lookup.insert(profile_event(1, 100, &json!({ "display_name": "Ann Lee" })));
    let search = studio_core::testing::relay::TestRelay::start();
    search.insert(profile_event(
        1,
        300,
        &json!({ "display_name": "Ann Lee (relay)" }),
    ));
    search.insert(profile_event(3, 100, &json!({ "name": "ann third" })));
    search.insert(profile_event(
        4,
        100,
        &json!({ "name": "unrelated person" }),
    ));
    let app = nostr_app(&index.url(""), vec![lookup.url()], false);
    set_search_relay(&app, &search.url()).await;
    // A person with the key 1 in a feed of the user.
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["persons"] = json!([
        { "name": "Ann Lee", "npub": nostr_npub(1) },
        { "name": "Bob", "npub": nostr_npub(7) },
    ]);
    call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;

    let body = profiles(&app, "ann").await;
    assert_eq!(body["failed"], json!([]));
    let list = body["profiles"].as_array().unwrap();
    let by_key = |seed: u8| {
        list.iter()
            .find(|p| p["npub"] == nostr_npub(seed))
            .unwrap_or_else(|| panic!("no profile for key {seed}: {body}"))
            .clone()
    };
    assert_eq!(list.len(), 3, "{body}");
    // Key 1 has the three sources. The newest event wins for the name.
    assert_eq!(
        by_key(1)["sources"],
        json!([
            { "kind": "collection" },
            { "kind": "music_index", "feeds": 1 },
            { "kind": "relay" },
        ])
    );
    assert_eq!(by_key(1)["name"], "Ann Lee (relay)");
    assert_eq!(
        by_key(2)["sources"],
        json!([{ "kind": "music_index", "feeds": 1 }])
    );
    assert_eq!(by_key(2)["name"], "Ann Other");
    assert_eq!(by_key(3)["sources"], json!([{ "kind": "relay" }]));
    assert_eq!(by_key(3)["name"], "ann third");
    // The profile with the most sources is first.
    assert_eq!(list[0]["npub"], nostr_npub(1));
    // A name that no source has gives nothing from the collection.
    let body = profiles(&app, "Bob").await;
    assert_eq!(body["profiles"][0]["npub"], nostr_npub(7));
    assert_eq!(
        body["profiles"][0]["sources"],
        json!([{ "kind": "collection" }])
    );
}

#[tokio::test]
async fn a_source_that_does_not_answer_gives_the_other_results_and_is_named() {
    let search = studio_core::testing::relay::TestRelay::start();
    search.insert(profile_event(3, 100, &json!({ "name": "ann third" })));
    // musicindex does not answer (port 9).
    let app = nostr_app("http://127.0.0.1:9", vec![UNUSED_RELAY.to_owned()], false);
    set_search_relay(&app, &search.url()).await;
    let body = profiles(&app, "ann").await;
    assert_eq!(body["failed"], json!(["musicindex"]));
    assert_eq!(body["profiles"][0]["npub"], nostr_npub(3));

    // The relay does not answer, and musicindex does.
    let index = nostr_index();
    let app = nostr_app(&index.url(""), vec![UNUSED_RELAY.to_owned()], false);
    set_search_relay(&app, &dead_relay()).await;
    let body = profiles(&app, "ann").await;
    assert_eq!(body["failed"], json!(["relay"]));
    assert_eq!(body["profiles"].as_array().unwrap().len(), 2, "{body}");
}

#[tokio::test]
async fn domain_match_is_true_only_for_the_domain_of_the_feed_link() {
    let server = nip05_server(&json!({ "ann": nostr_hex(1) }));
    let domain = format!("127.0.0.1:{}", server.port());
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(
        1,
        100,
        &json!({ "display_name": "Ann Lee", "nip05": format!("ann@{domain}") }),
    ));
    let app = nostr_app("http://127.0.0.1:9", vec![relay.url()], false);
    let id = set_up(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut revision = draft["revision"].clone();
    let model = draft["model"].clone();
    let query = format!(
        "/api/v1/nostr/profiles?q={}&feed={id}",
        url_encode(&nostr_npub(1))
    );
    for (link, expected) in [
        ("http://www.127.0.0.1/music", true),
        ("https://example.org/", false),
    ] {
        let mut changed = model.clone();
        changed["link"] = json!(link);
        let (_, saved) = call(
            &app,
            "PUT",
            &draft_path,
            Some(json!({ "revision": revision, "model": changed })),
        )
        .await;
        revision = saved["revision"].clone();
        let (status, body) = call(&app, "GET", &query, None).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["profiles"][0]["nip05_ok"], true, "{link}");
        assert_eq!(body["profiles"][0]["domain_match"], expected, "{link}");
    }
    // With no feed, there is no domain match.
    let body = profiles(&app, &nostr_npub(1)).await;
    assert_eq!(body["profiles"][0]["domain_match"], false);
}

#[tokio::test]
async fn an_empty_search_relay_setting_makes_no_relay_request() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let app = nostr_app("http://127.0.0.1:9", vec![UNUSED_RELAY.to_owned()], false);
    // The listener was the search relay. The empty setting replaces it.
    let relay = format!("ws://{}", listener.local_addr().unwrap());
    set_search_relay(&app, &relay).await;
    set_search_relay(&app, "").await;
    let body = profiles(&app, "ann").await;
    assert_eq!(body["failed"], json!(["musicindex"]));
    assert_eq!(body["profiles"], json!([]));
    assert!(listener.accept().is_err(), "a connection came in");
}

#[tokio::test]
async fn with_the_policy_public_a_nip05_domain_and_a_relay_on_loopback_get_no_request() {
    let nip05 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    nip05.set_nonblocking(true).unwrap();
    let domain = format!("127.0.0.1:{}", nip05.local_addr().unwrap().port());
    let search = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    search.set_nonblocking(true).unwrap();
    let lookup = studio_core::testing::relay::TestRelay::start();
    lookup.insert(profile_event(1, 100, &json!({ "display_name": "Ann Lee" })));
    let app = nostr_app("http://127.0.0.1:9", vec![lookup.url()], true);
    set_search_relay(&app, &format!("ws://{}", search.local_addr().unwrap())).await;

    let body = profiles(&app, &format!("ann@{domain}")).await;
    assert_eq!(body["profiles"], json!([]));
    assert!(nip05.accept().is_err(), "the NIP-05 server got a request");

    // The relay search and the lookup relay are refused too.
    let body = profiles(&app, "ann").await;
    assert_eq!(body["failed"], json!(["musicindex", "relay"]));
    assert!(search.accept().is_err(), "the search relay got a request");
    let body = profiles(&app, &nostr_npub(1)).await;
    assert_eq!(body["profiles"][0]["name"], Value::Null);
}

/// The OP3 form of an address of the local test server (ADR 0076). The
/// test server is the only host that these tests reach.
fn op3_address(plain: &str) -> String {
    format!("https://op3.dev/e,pg=abc/{plain}")
}

#[tokio::test]
async fn the_media_route_reads_the_plain_address_of_a_prefixed_enclosure() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set_file("/a.mp3", vec![0; 4321], "audio/mpeg", false);
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/media/facts",
        Some(json!({ "url": op3_address(&server.url("/a.mp3")) })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({ "length": 4321, "media_type": "audio/mpeg" }));
}

#[tokio::test]
async fn a_new_album_with_from_is_a_copy_with_new_guids_and_leaves_the_source_as_it_was() {
    let (app, connector) = publisher_app().await;
    let source = make_feed(
        &app,
        new_feed_body(connector, "source.xml", "Source", None, None),
    )
    .await;
    let source_id = source["id"].as_i64().unwrap();
    let source_path = format!("/api/v1/feeds/{source_id}/draft");
    let before = call(&app, "GET", &source_path, None).await.1;

    let mut body = new_feed_body(connector, "copy.xml", "Copy", None, None);
    body["from"] = json!(source_id);
    let copy = make_feed(&app, body).await;
    assert_eq!(copy["title"], "Copy");
    assert_eq!(copy["medium"], "music");
    assert_eq!(copy["public_url"], "https://music.example/copy.xml");
    assert!(!app.feeds.path().join("copy.xml").exists());

    let model = draft_model(&app, copy["id"].as_i64().unwrap()).await;
    assert_eq!(model["title"], "Copy");
    assert_eq!(
        model["podcast_guid"],
        studio_core::guid::podcast_guid_for_url("https://music.example/copy.xml")
    );
    assert_ne!(model["podcast_guid"], before["model"]["podcast_guid"]);

    let after = call(&app, "GET", &source_path, None).await.1;
    assert_eq!(after, before);
}

#[tokio::test]
async fn from_with_a_publisher_feed_or_a_playlist_gives_422() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let playlist = make_feed(
        &app,
        new_feed_body(connector, "list.xml", "List", Some("musicL"), None),
    )
    .await;
    let album = make_feed(
        &app,
        new_feed_body(connector, "mine.xml", "Album", None, None),
    )
    .await;
    for (source, medium, path) in [
        (&label, None, "c1.xml"),
        (&playlist, None, "c2.xml"),
        (&album, Some("musicL"), "c3.xml"),
        (&album, Some("publisher"), "c4.xml"),
    ] {
        let mut body = new_feed_body(connector, path, "Copy", medium, Some("label"));
        body["from"] = source["id"].clone();
        let (status, response) = call(&app, "POST", "/api/v1/feeds/new", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path}");
        assert_eq!(response["error"]["code"], "invalid_feed", "{path}");
    }
}

// ---- Change sets (ADR 0091, task 054b) ----

/// A feed XML with value blocks. `channel` and `item` give the recipients
/// of the channel block and of the block of the one track, each as
/// (name, address, split). `extra` goes into the channel block.
fn paid_feed(
    title: &str,
    guid: &str,
    channel: &[(&str, &str, &str)],
    item: &[(&str, &str, &str)],
    extra: &str,
) -> String {
    use std::fmt::Write as _;
    let recipients = |list: &[(&str, &str, &str)]| -> String {
        if list.is_empty() {
            return String::new();
        }
        let mut inner = String::new();
        for (name, address, split) in list {
            let _ = write!(
                inner,
                "<podcast:valueRecipient name=\"{name}\" address=\"{address}\" \
                 split=\"{split}\" type=\"lnaddress\"/>"
            );
        }
        format!("<podcast:value type=\"lightning\" method=\"keysend\">{inner}</podcast:value>")
    };
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>{title}</title><podcast:guid>{guid}</podcast:guid>\
         <podcast:medium>music</podcast:medium>{channel_value}{extra}\
         <item><guid>{guid}-t1</guid><title>Track</title>{item_value}</item>\
         </channel></rss>",
        channel_value = recipients(channel),
        item_value = recipients(item),
    )
}

/// Add a feed file to the connected folder and to the collection. Gives
/// the feed ID.
async fn add_paid_feed(app: &App, connector: i64, path: &str, xml: &str) -> i64 {
    std::fs::write(app.feeds.path().join(path), xml).unwrap();
    let (status, feed) = call(
        app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector, "path": path })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    let id = feed["id"].as_i64().unwrap();
    // The first read gives the feed its release folder.
    let (status, body) = call(app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    id
}

fn ann_key() -> Value {
    json!({ "address": "ann@example.com" })
}

/// The row of a feed in the response of a change set.
fn change_row(set: &Value, id: i64) -> Value {
    set["feeds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["feed_id"] == id)
        .unwrap_or_else(|| panic!("feed {id} is not in {set}"))
        .clone()
}

const ANN: (&str, &str, &str) = ("Ann", "ann@example.com", "90");
const BOB: (&str, &str, &str) = ("Bob", "bob@example.com", "10");

#[tokio::test]
async fn recipient_uses_gives_the_places_of_a_recipient_and_not_a_value_time_split() {
    let (app, connector) = publisher_app().await;
    let split = "<podcast:value type=\"lightning\" method=\"keysend\">\
        <podcast:valueRecipient name=\"Bob\" address=\"bob@example.com\" split=\"100\" type=\"lnaddress\"/>\
        <podcast:valueTimeSplit startTime=\"0\" duration=\"10\">\
        <podcast:valueRecipient name=\"Ann\" address=\"ann@example.com\" split=\"100\" type=\"lnaddress\"/>\
        </podcast:valueTimeSplit></podcast:value>";
    let a = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[ANN, BOB], ""),
    )
    .await;
    let b = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[], &[ANN, BOB], ""),
    )
    .await;
    let c = add_paid_feed(
        &app,
        connector,
        "c.xml",
        &paid_feed("C", "gc", &[], &[], split),
    )
    .await;

    let (status, body) = call(&app, "POST", "/api/v1/recipients/uses", Some(ann_key())).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let feeds = body["feeds"].as_array().unwrap();
    assert_eq!(feeds.len(), 2, "{body}");
    let row = |id: i64| feeds.iter().find(|r| r["feed_id"] == id).unwrap().clone();
    assert_eq!(row(a)["channel"], true);
    assert_eq!(row(a)["items"], 1);
    assert_eq!(row(a)["live_items"], 0);
    assert_eq!(row(a)["writable"], true);
    assert_eq!(row(a)["title"], "A");
    assert_eq!(row(b)["channel"], false);
    assert_eq!(row(b)["items"], 1);
    assert!(feeds.iter().all(|r| r["feed_id"] != c), "{body}");
}

#[tokio::test]
async fn a_change_set_gives_a_changed_feed_a_not_changed_feed_and_a_read_only_feed() {
    let (app, connector) = publisher_app().await;
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/ro.xml",
        200,
        paid_feed("Read only", "gro", &[ANN, BOB], &[], "").into_bytes(),
    );
    let changed = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[], ""),
    )
    .await;
    let same = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[BOB], &[], ""),
    )
    .await;
    let read_only = {
        let mut collection =
            studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
                .unwrap();
        let id = collection
            .add_feed(Some(&server.url("/ro.xml")), None)
            .unwrap();
        studio_core::pipeline::refresh_public(&mut collection, &studio_core::web::Web::new(), id)
            .unwrap();
        collection.set_feed_folder(id, "ro").unwrap();
        let library = studio_core::library::Library::open(app.data.path().join("library")).unwrap();
        library
            .write_feed_xml(
                "ro",
                "ro.xml",
                paid_feed("Read only", "gro", &[ANN, BOB], &[], "").as_bytes(),
            )
            .unwrap();
        id.0
    };

    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": { "kind": "rename", "name": "Anna" },
            "feed_ids": [changed, same, read_only],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    assert_eq!(set["open"], true);
    assert_eq!(set["operation"]["kind"], "rename");
    let row = change_row(&set, changed);
    assert_eq!(row["outcome"]["outcome"], "changed");
    assert_eq!(row["outcome"]["channel"], true);
    assert_eq!(row["blocked"], Value::Null);
    let row = change_row(&set, same);
    assert_eq!(row["outcome"]["outcome"], "not_changed");
    assert_eq!(row["outcome"]["cause"]["cause"], "no_recipient");
    assert_eq!(row["blocked"], Value::Null);
    let row = change_row(&set, read_only);
    assert_eq!(row["outcome"]["outcome"], "changed");
    assert_eq!(row["blocked"], "read_only_no_host");
    // The unchanged feed has no draft. The changed feeds have one.
    let collection = collection_of(&app);
    assert!(
        collection
            .draft(studio_core::collection::FeedId(same))
            .unwrap()
            .is_none()
    );
    assert!(
        collection
            .draft(studio_core::collection::FeedId(changed))
            .unwrap()
            .is_some()
    );
    let draft = draft_model(&app, changed).await;
    assert_eq!(draft["values"][0]["recipients"][0]["name"], "Anna");

    // The list and the detail route give the same set.
    let (_, list) = call(&app, "GET", "/api/v1/change-sets", None).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let id = set["id"].as_i64().unwrap();
    let (status, detail) = call(&app, "GET", &format!("/api/v1/change-sets/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail, set);
    let (status, body) = call(&app, "GET", "/api/v1/change-sets/9999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn the_write_of_a_changed_feed_gives_its_written_at_in_the_change_set() {
    let (app, connector) = publisher_app().await;
    let a = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[], ""),
    )
    .await;
    let b = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[ANN, BOB], &[], ""),
    )
    .await;
    let (_, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": { "kind": "rename", "name": "Anna" },
            "feed_ids": [a, b],
        })),
    )
    .await;
    let id = set["id"].as_i64().unwrap();
    assert_eq!(change_row(&set, a)["written_at"], Value::Null);

    let (status, diff) = call(&app, "GET", &format!("/api/v1/feeds/{a}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{a}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (_, set) = call(&app, "GET", &format!("/api/v1/change-sets/{id}"), None).await;
    assert!(change_row(&set, a)["written_at"].is_i64(), "{set}");
    assert_eq!(change_row(&set, b)["written_at"], Value::Null);
    assert_eq!(set["open"], true);
}

#[tokio::test]
async fn discard_puts_back_the_draft_byte_for_byte_and_a_feed_with_no_draft_has_none() {
    let (app, connector) = publisher_app().await;
    let with_draft = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[], ""),
    )
    .await;
    let without = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[ANN, BOB], &[], ""),
    )
    .await;
    // An earlier edit makes the draft of the first feed.
    let draft_path = format!("/api/v1/feeds/{with_draft}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["title"] = json!("A (edited)");
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let before = collection_of(&app)
        .draft(studio_core::collection::FeedId(with_draft))
        .unwrap()
        .unwrap()
        .draft_xml;

    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": { "kind": "remove" },
            "feed_ids": [with_draft, without],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    let id = set["id"].as_i64().unwrap();
    let collection = collection_of(&app);
    assert_ne!(
        collection
            .draft(studio_core::collection::FeedId(with_draft))
            .unwrap()
            .unwrap()
            .draft_xml,
        before
    );
    assert!(
        collection
            .draft(studio_core::collection::FeedId(without))
            .unwrap()
            .is_some()
    );

    let (status, _) = call(&app, "DELETE", &format!("/api/v1/change-sets/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let collection = collection_of(&app);
    assert_eq!(
        collection
            .draft(studio_core::collection::FeedId(with_draft))
            .unwrap()
            .unwrap()
            .draft_xml,
        before
    );
    assert!(
        collection
            .draft(studio_core::collection::FeedId(without))
            .unwrap()
            .is_none()
    );
    let (status, _) = call(&app, "GET", &format!("/api/v1/change-sets/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app, "DELETE", &format!("/api/v1/change-sets/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rename_sets_the_name_of_the_person_and_change_address_keeps_its_wallet() {
    let (app, connector) = publisher_app().await;
    let a = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[], ""),
    )
    .await;
    let (_, saved) = call(
        &app,
        "POST",
        "/api/v1/recipients/saved",
        Some(json!({ "name": "Ann", "recipient_type": "lnaddress", "address": "ann@example.com" })),
    )
    .await;
    let saved_id = saved["id"].as_i64().unwrap();
    let saved_row = |list: &Value| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|r| r["saved_id"] == saved_id)
            .unwrap()
            .clone()
    };

    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": { "kind": "rename", "name": "Anna" },
            "feed_ids": [a],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    let (_, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(saved_row(&list)["name"], "Anna");
    // Discard puts back the feed, and it keeps the new name of the person.
    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/change-sets/{}", set["id"]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(saved_row(&list)["name"], "Anna");

    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": {
                "kind": "change_address",
                "address": "anna@new.example",
                "custom_key": null,
                "custom_value": null,
                "recipient_type": "lnaddress",
            },
            "feed_ids": [a],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    let (_, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(saved_row(&list)["address"], "ann@example.com");
}

/// Add a recipient to the draft of the feed with the ID `id`. The recipient
/// is then only in the draft, not in the written feed.
async fn add_draft_only_recipient(app: &App, id: i64, name: &str, address: &str) {
    {
        let (status, set) = call(
            app,
            "POST",
            "/api/v1/change-sets",
            Some(json!({
                "recipient": { "address": address },
                "operation": {
                    "kind": "add",
                    "name": name,
                    "recipient_type": "lnaddress",
                    "address": address,
                    "custom_key": null,
                    "custom_value": null,
                    "percent": 5,
                },
                "feed_ids": [id],
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{set}");
    }
}

#[tokio::test]
async fn the_list_and_the_uses_count_a_recipient_in_the_same_feeds() {
    let (app, connector) = publisher_app().await;
    let a = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[ANN, BOB], &[], ""),
    )
    .await;
    let b = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[ANN, BOB], &[], ""),
    )
    .await;
    let c = add_paid_feed(
        &app,
        connector,
        "c.xml",
        &paid_feed("C", "gc", &[BOB], &[], ""),
    )
    .await;
    add_draft_only_recipient(&app, c, "Ann", "ann@example.com").await;

    let (status, uses) = call(&app, "POST", "/api/v1/recipients/uses", Some(ann_key())).await;
    assert_eq!(status, StatusCode::OK, "{uses}");
    let ids: Vec<i64> = uses["feeds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["feed_id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![a, b, c], "{uses}");

    let (status, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK);
    let ann = list
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["address"] == "ann@example.com")
        .unwrap();
    assert_eq!(ann["feeds"], 3, "{list}");
}

#[tokio::test]
async fn a_recipient_that_only_a_draft_has_is_in_the_list() {
    let (app, connector) = publisher_app().await;
    let c = add_paid_feed(
        &app,
        connector,
        "c.xml",
        &paid_feed("C", "gc", &[BOB], &[], ""),
    )
    .await;
    add_draft_only_recipient(&app, c, "Dan", "dan@example.com").await;

    let (status, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK);
    let dan = list
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["address"] == "dan@example.com")
        .unwrap_or_else(|| panic!("the list has no Dan: {list}"));
    assert_eq!(dan["name"], "Dan");
    assert_eq!(dan["recipient_type"], "lnaddress");
    assert_eq!(dan["feeds"], 1);
    assert_eq!(dan["saved_id"], Value::Null);
}

#[tokio::test]
async fn the_level_of_a_publisher_feed_gives_its_releases_and_an_album_gives_422() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await;
    let release = make_feed(
        &app,
        new_feed_body(connector, "release.xml", "Release", None, None),
    )
    .await;
    let label_id = label["id"].as_i64().unwrap();
    let release_id = release["id"].as_i64().unwrap();
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{label_id}/links"),
        Some(json!({ "release": release_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, level) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{label_id}/level"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{level}");
    assert_eq!(level["feed_ids"], json!([label_id, release_id]));

    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/feeds/{release_id}/level"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "not_publisher");
}

#[tokio::test]
async fn each_change_set_route_needs_the_token() {
    let app = app();
    let routes = [
        ("POST", "/api/v1/recipients/uses"),
        ("POST", "/api/v1/change-sets"),
        ("GET", "/api/v1/change-sets"),
        ("GET", "/api/v1/change-sets/1"),
        ("DELETE", "/api/v1/change-sets/1"),
        ("GET", "/api/v1/feeds/1/level"),
    ];
    for (method, path) in routes {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, HOST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let (status, value, _) = send(&app, request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
        assert_eq!(value["error"]["code"], "token_required", "{method} {path}");
    }
}

// ---- Wallet recipients (ADR 0095, task 060c) ----

const WALLET_KEYSEND: &str = r#"{"status":"OK","tag":"keysend","pubkey":"02abc","customData":[{"customKey":"696969","customValue":"x1"}]}"#;
const OTHER_KEYSEND: &str = r#"{"status":"OK","tag":"keysend","pubkey":"02def"}"#;

/// A feed XML whose channel has one recipient of the given type.
fn wallet_feed(guid: &str, recipient_type: &str, address: &str, custom: &str) -> String {
    format!(
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>{guid}</title><podcast:guid>{guid}</podcast:guid>\
         <podcast:medium>music</podcast:medium>\
         <podcast:value type=\"lightning\" method=\"keysend\">\
         <podcast:valueRecipient name=\"Ann\" address=\"{address}\" split=\"100\" \
         type=\"{recipient_type}\" {custom}/></podcast:value>\
         <item><guid>{guid}-t1</guid><title>Track</title></item>\
         </channel></rss>"
    )
}

/// An app with a connector, a keysend scheme and the test server of the
/// wallet. The server answers for `ann`, `bob` and `cy`.
async fn wallet_app(keysend: Option<&'static str>) -> (App, i64, studio_core::testing::TestServer) {
    let app = app_with_keysend("http://127.0.0.1:9", keysend);
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
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/.well-known/keysend/ann",
        200,
        WALLET_KEYSEND.as_bytes().to_vec(),
    );
    server.set(
        "/.well-known/keysend/bob",
        200,
        OTHER_KEYSEND.as_bytes().to_vec(),
    );
    server.set("/.well-known/keysend/cy", 404, b"no".to_vec());
    (app, connector["id"].as_i64().unwrap(), server)
}

/// Two feeds: one pays the node key of the wallet, one pays the Lightning
/// address of the wallet. Gives the two feed IDs and the address.
async fn wallet_feeds(
    app: &App,
    connector: i64,
    server: &studio_core::testing::TestServer,
) -> (i64, i64, String) {
    let address = format!("ann@127.0.0.1:{}", server.port());
    let node = add_paid_feed(
        app,
        connector,
        "node.xml",
        &wallet_feed(
            "gn",
            "node",
            "02abc",
            "customKey=\"696969\" customValue=\"x1\"",
        ),
    )
    .await;
    let ln = add_paid_feed(
        app,
        connector,
        "ln.xml",
        &wallet_feed("gl", "lnaddress", &address, ""),
    )
    .await;
    (node, ln, address)
}

fn wallet_key_json() -> Value {
    json!({ "address": "02abc", "custom_key": "696969", "custom_value": "x1" })
}

#[tokio::test]
async fn a_node_and_its_lightning_address_are_one_recipient() {
    let (app, connector, server) = wallet_app(Some("http")).await;
    let (_, _, address) = wallet_feeds(&app, connector, &server).await;
    let (status, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1, "{list:?}");
    assert_eq!(list[0]["feeds"], 2);
    assert_eq!(list[0]["address"], "02abc");
    assert_eq!(list[0]["custom_key"], "696969");
    let addresses = list[0]["addresses"].as_array().unwrap();
    assert_eq!(addresses.len(), 2);
    assert_eq!(addresses[0]["address"], "02abc");
    assert_eq!(addresses[0]["lookup"], Value::Null);
    assert_eq!(addresses[1]["address"], address.as_str());
    assert_eq!(addresses[1]["lookup"], "found");
}

#[tokio::test]
async fn a_node_address_gives_the_provider_when_two_addresses_share_its_key() {
    let (app, connector, server) = wallet_app(Some("http")).await;
    wallet_feeds(&app, connector, &server).await;
    let (_, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    let addresses = &list[0]["addresses"];
    assert_eq!(addresses[0]["provider"], Value::Null, "one address: {list}");

    let domain = format!("127.0.0.1:{}", server.port());
    server.set(
        "/.well-known/keysend/dan",
        200,
        br#"{"status":"OK","tag":"keysend","pubkey":"02abc","customData":[{"customKey":"696969","customValue":"x2"}]}"#.to_vec(),
    );
    add_paid_feed(
        &app,
        connector,
        "dan.xml",
        &wallet_feed("gd", "lnaddress", &format!("dan@{domain}"), ""),
    )
    .await;
    let (_, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    let wallet = list
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["custom_value"] == "x1")
        .unwrap();
    let addresses = wallet["addresses"].as_array().unwrap();
    assert_eq!(addresses[0]["address"], "02abc");
    assert_eq!(addresses[0]["provider"], domain.as_str(), "{list}");
    assert_eq!(addresses[1]["provider"], Value::Null, "{list}");
}

#[tokio::test]
async fn with_no_lookup_scheme_the_same_feeds_give_two_recipients() {
    let (app, connector, server) = wallet_app(None).await;
    wallet_feeds(&app, connector, &server).await;
    let (status, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert_eq!(list.as_array().unwrap().len(), 2, "{list}");
    let rows = collection_of(&app).keysend_lookups().unwrap();
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
async fn recipient_uses_and_a_rename_act_on_each_member_of_the_wallet() {
    let (app, connector, server) = wallet_app(Some("http")).await;
    let (node, ln, _) = wallet_feeds(&app, connector, &server).await;
    call(&app, "GET", "/api/v1/recipients", None).await;
    let (status, uses) = call(
        &app,
        "POST",
        "/api/v1/recipients/uses",
        Some(wallet_key_json()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{uses}");
    assert_eq!(uses["feeds"].as_array().unwrap().len(), 2, "{uses}");
    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": wallet_key_json(),
            "operation": { "kind": "rename", "name": "Anna" },
            "feed_ids": [node, ln],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    assert_eq!(change_row(&set, node)["outcome"]["outcome"], "changed");
    assert_eq!(change_row(&set, ln)["outcome"]["outcome"], "changed");
    assert_eq!(
        draft_model(&app, ln).await["values"][0]["recipients"][0]["name"],
        "Anna"
    );
}

#[tokio::test]
async fn lnaddress_suggestions_give_a_checked_address_of_the_wallet() {
    let (app, connector, server) = wallet_app(Some("http")).await;
    let (_, _, address) = wallet_feeds(&app, connector, &server).await;
    let before = collection_of(&app).keysend_lookups().unwrap().len();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/recipients/lnaddress-suggestions",
        Some(wallet_key_json()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["suggestions"],
        json!([{ "address": address, "sources": ["collection"] }])
    );
    assert_eq!(collection_of(&app).keysend_lookups().unwrap().len(), before);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/recipients/lnaddress-suggestions",
        Some(json!({ "address": address })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_recipient");
}

#[tokio::test]
async fn use_lnaddress_checks_the_lookup_of_the_address() {
    let (app, connector, server) = wallet_app(Some("http")).await;
    let (node, _, _) = wallet_feeds(&app, connector, &server).await;
    let port = server.port();
    let try_address = |address: String| {
        let app = &app;
        async move {
            call(
                app,
                "POST",
                "/api/v1/change-sets",
                Some(json!({
                    "recipient": wallet_key_json(),
                    "operation": { "kind": "use_lnaddress", "address": address },
                    "feed_ids": [node],
                })),
            )
            .await
        }
    };
    let (status, body) = try_address(format!("bob@127.0.0.1:{port}")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "lnaddress_mismatch");
    let (status, body) = try_address(format!("cy@127.0.0.1:{port}")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "lnaddress_no_keysend");
    let (status, body) = try_address("no-at-sign".to_owned()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_recipient");
    let (status, set) = try_address(format!("ann@127.0.0.1:{port}")).await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    let draft = draft_model(&app, node).await;
    let recipient = &draft["values"][0]["recipients"][0];
    assert_eq!(recipient["recipient_type"], "lnaddress");
    assert_eq!(draft["values"][0]["method"], "lnaddress");
}

#[tokio::test]
async fn change_address_with_no_type_gives_422() {
    let (app, connector, server) = wallet_app(None).await;
    let (node, _, _) = wallet_feeds(&app, connector, &server).await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": wallet_key_json(),
            "operation": { "kind": "change_address", "address": "new@example.com" },
            "feed_ids": [node],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_recipient");
}

// ---- People (ADR 0098, task 063d) ----

/// The ID of a new person that has the given name.
async fn new_person(app: &App, name: &str) -> i64 {
    let (status, body) = call(app, "POST", "/api/v1/people", Some(json!({ "name": name }))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_i64().unwrap()
}

#[tokio::test]
async fn the_role_list_gives_the_music_group_first() {
    let app = app();
    let (status, body) = call(&app, "GET", "/api/v1/roles", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let groups = body["groups"].as_array().unwrap();
    assert_eq!(
        groups[0]["name"], "Music",
        "ADR 0112 item 3: GET /roles must give the group Music first. Check studio_core::roles::groups."
    );
    assert_eq!(groups[0]["roles"][1], "Vocalist");
    assert_eq!(groups.len(), 12);
}

#[tokio::test]
async fn a_person_is_added_read_changed_and_removed() {
    let app = app();
    let id = new_person(&app, " Ann ").await;

    let (status, body) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "Ann");
    assert_eq!(body["default_address"], Value::Null);
    assert_eq!(body["wallets"], json!([]));
    assert_eq!(body["pays"], json!([]));
    assert_eq!(body["credited"], json!([]));

    let (status, list) = call(&app, "GET", "/api/v1/people", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], id);
    assert_eq!(list[0]["pays"], 0);
    assert_eq!(list[0]["credited"], 0);

    let (status, _) = call(
        &app,
        "PATCH",
        &format!("/api/v1/people/{id}"),
        Some(json!({ "name": "Ann B", "usual_role": "vocals", "usual_group": "music" })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(body["name"], "Ann B");
    assert_eq!(body["usual_role"], "vocals");
    assert_eq!(body["usual_group"], "music");

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/people",
        Some(json!({ "name": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_person");

    let (status, _) = call(&app, "DELETE", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for method in ["GET", "DELETE"] {
        let (status, body) = call(&app, method, &format!("/api/v1/people/{id}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {body}");
        assert_eq!(body["error"]["code"], "not_found");
    }
}

#[tokio::test]
async fn one_save_changes_a_person() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await["id"]
        .as_i64()
        .unwrap();
    let id = new_person(&app, "Ann").await;
    let base = format!("/api/v1/people/{id}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/save"),
        Some(json!({
            "fields": { "name": "Ann B", "img": "https://x.example/a.png" },
            "members": [label],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, body) = call(&app, "GET", &base, None).await;
    assert_eq!(body["name"], "Ann B");
    assert_eq!(body["img"], "https://x.example/a.png");
    assert_eq!(body["members"], json!([label]));
    assert_eq!(body["linked_credits"], json!([]));

    let (status, body) = call(
        &app,
        "POST",
        &format!("{base}/save"),
        Some(json!({ "fields": { "name": "  " } })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_person");

    let (status, _) = call(
        &app,
        "POST",
        &format!("{base}/save"),
        Some(json!({ "members": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, "GET", &base, None).await;
    assert_eq!(body["members"], json!([]));
    assert_eq!(body["name"], "Ann B");
}

#[tokio::test]
async fn each_kind_of_link_is_added_and_removed() {
    let (app, connector) = publisher_app().await;
    let label = make_feed(
        &app,
        new_feed_body(
            connector,
            "label.xml",
            "Label",
            Some("publisher"),
            Some("label"),
        ),
    )
    .await["id"]
        .as_i64()
        .unwrap();
    let id = new_person(&app, "Ann").await;
    let base = format!("/api/v1/people/{id}");

    let bodies = [
        (
            "wallets",
            json!({ "address": "ann@example.com", "recipient_type": "lnaddress" }),
            "wallets",
        ),
        ("keys", json!({ "npub": "npub1ann" }), "keys"),
        (
            "credit-names",
            json!({ "name": "Ann Artist" }),
            "credit_names",
        ),
        (
            "support-links",
            json!({ "label": "Shop", "url": "https://shop.example" }),
            "support_links",
        ),
        ("members", json!({ "feed_id": label }), "members"),
    ];
    for (kind, body, field) in bodies {
        let (status, added) =
            call(&app, "POST", &format!("{base}/{kind}"), Some(body.clone())).await;
        assert_eq!(status, StatusCode::CREATED, "{kind} {added}");
        let link = added["id"].as_i64().unwrap();
        let (_, person) = call(&app, "GET", &base, None).await;
        assert_eq!(person[field].as_array().unwrap().len(), 1, "{kind}");

        // A second add of the same link gives the same ID. A support link
        // has no key, so each add makes a new link.
        if kind != "support-links" {
            let (_, again) = call(&app, "POST", &format!("{base}/{kind}"), Some(body)).await;
            assert_eq!(again["id"], link, "{kind}");
        }

        let (status, _) = call(&app, "DELETE", &format!("{base}/{kind}/{link}"), None).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{kind}");
        let (_, person) = call(&app, "GET", &base, None).await;
        assert_eq!(person[field], json!([]), "{kind}");
        let (status, _) = call(&app, "DELETE", &format!("{base}/{kind}/{link}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{kind}");
    }

    let (status, body) = call(&app, "POST", &format!("{base}/pets"), Some(json!({}))).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn the_default_address_is_set_and_cleared() {
    let app = app();
    let id = new_person(&app, "Ann").await;
    let base = format!("/api/v1/people/{id}");
    for address in ["a@example.com", "b@example.com"] {
        call(
            &app,
            "POST",
            &format!("{base}/wallets"),
            Some(json!({ "address": address })),
        )
        .await;
    }
    // With no choice, the first wallet is the default address.
    let (_, person) = call(&app, "GET", &base, None).await;
    assert_eq!(person["default_address"]["address"], "a@example.com");

    let (status, _) = call(
        &app,
        "PUT",
        &format!("{base}/default-address"),
        Some(json!({ "address": "b@example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, person) = call(&app, "GET", &base, None).await;
    assert_eq!(person["default_address"]["address"], "b@example.com");

    let (status, _) = call(
        &app,
        "PUT",
        &format!("{base}/default-address"),
        Some(Value::Null),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, person) = call(&app, "GET", &base, None).await;
    assert_eq!(person["default_address"]["address"], "a@example.com");
}

#[tokio::test]
async fn a_wallet_gives_its_members_and_a_member_can_be_the_default_address() {
    let app = app();
    collection_of(&app)
        .set_keysend_lookup(&studio_core::keysend::Lookup {
            address: "ann@example.com".into(),
            pubkey: Some("02abc".into()),
            custom_key: Some("696969".into()),
            custom_value: Some("x1".into()),
            status: studio_core::keysend::LookupStatus::Found,
            checked_at: 1,
        })
        .unwrap();
    let id = new_person(&app, "Ann").await;
    let base = format!("/api/v1/people/{id}");
    let (status, _) = call(
        &app,
        "POST",
        &format!("{base}/wallets"),
        Some(json!({ "address": "ann@example.com", "recipient_type": "lnaddress" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, person) = call(&app, "GET", &base, None).await;
    let members = person["wallets"][0]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2, "{person}");
    assert_eq!(members[0]["key"]["address"], "02abc");
    assert_eq!(members[0]["key"]["custom_value"], "x1");
    assert_eq!(members[0]["recipient_type"], "node");
    assert_eq!(members[1]["key"]["address"], "ann@example.com");
    assert_eq!(members[1]["recipient_type"], "lnaddress");

    let (status, _) = call(
        &app,
        "PUT",
        &format!("{base}/default-address"),
        Some(members[0]["key"].clone()),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, person) = call(&app, "GET", &base, None).await;
    assert_eq!(person["default_address"]["address"], "02abc");
}

#[tokio::test]
async fn a_wallet_of_a_different_person_gives_409() {
    let app = app();
    let first = new_person(&app, "Ann").await;
    let second = new_person(&app, "Bob").await;
    let wallet = json!({ "address": "ann@example.com" });
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{first}/wallets"),
        Some(wallet.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{second}/wallets"),
        Some(wallet),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "link_in_other_person");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&format!("person {first}")),
        "{body}"
    );
}

/// A feed with one credit, and a feed with one recipient. Both have the
/// name Ann. Gives the two feed IDs.
async fn credit_and_payee_feeds(app: &App, connector: i64) -> (i64, i64) {
    let credit = add_paid_feed(
        app,
        connector,
        "credit.xml",
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>One</title><podcast:guid>g-one</podcast:guid>\
         <podcast:medium>music</podcast:medium>\
         <podcast:person role=\"vocals\">Ann</podcast:person>\
         <item><guid>one-t1</guid><title>Track</title></item></channel></rss>",
    )
    .await;
    let payee = add_paid_feed(
        app,
        connector,
        "payee.xml",
        &wallet_feed("g-two", "lnaddress", "ann@example.com", ""),
    )
    .await;
    (credit, payee)
}

#[tokio::test]
async fn a_suggestion_makes_a_person_that_the_feeds_use() {
    let (app, connector) = publisher_app().await;
    let (credit, payee) = credit_and_payee_feeds(&app, connector).await;

    let (status, list) = call(&app, "GET", "/api/v1/people/suggestions", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert_eq!(list.as_array().unwrap().len(), 1, "{list}");
    let suggestion = &list[0];
    assert_eq!(suggestion["name"], "Ann");
    assert_eq!(suggestion["credit_feeds"], 1);
    assert_eq!(suggestion["payee_feeds"], 1);

    let credits = &suggestion["credits"][0];
    let recipient = &suggestion["recipients"][0];
    let (status, made) = call(
        &app,
        "POST",
        "/api/v1/people/suggestions/make",
        Some(json!({
            "name": "Ann",
            "credits": [[credits["name"], credits["href"], credits["npub"]]],
            "recipients": [[recipient["key"], recipient["recipient_type"]]],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{made}");
    let id = made["id"].as_i64().unwrap();

    let (_, list) = call(&app, "GET", "/api/v1/people/suggestions", None).await;
    assert_eq!(list, json!([]));

    let (status, person) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "{person}");
    assert_eq!(person["pays"], json!([payee]));
    assert_eq!(person["credited"], json!([credit]));
    // ADR 0099 item 5: the places give the address and the role of each feed.
    let places = person["places"].as_array().unwrap();
    assert_eq!(places.len(), 2, "{person}");
    assert!(
        places.iter().all(|p| p.get("feed_id").is_some()),
        "{person}"
    );
    let paid = places
        .iter()
        .find(|p| p["feed_id"] == json!(payee))
        .unwrap();
    assert_eq!(paid["pays"][0]["address"], "ann@example.com");
    assert_eq!(paid["roles"], json!([]));
    let credited = places
        .iter()
        .find(|p| p["feed_id"] == json!(credit))
        .unwrap();
    assert_eq!(credited["pays"], json!([]));
    assert_eq!(credited["roles"], json!(["vocals"]));
    let (_, list) = call(&app, "GET", "/api/v1/people", None).await;
    assert_eq!(list[0]["pays"], 1);
    assert_eq!(list[0]["credited"], 1);
    assert_eq!(list[0]["feeds"], 2);
    assert!(list[0].get("places").is_none(), "{list}");
}

#[tokio::test]
async fn items_are_added_to_a_person_and_a_conflict_gives_409() {
    let app = app();
    let ann = new_person(&app, "Ann").await;
    let bob = new_person(&app, "Bob").await;
    let body = json!({
        "credits": [["Ann", "", "npub1ann"]],
        "recipients": [[{ "address": "ann@example.com" }, "lnaddress"]],
    });
    let (status, added) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{ann}/items"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{added}");
    assert_eq!(added["added"], 3);
    let (_, added) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{ann}/items"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(added["added"], 0);
    let (status, error) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{bob}/items"),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["code"], "link_in_other_person");
    let (status, error) = call(
        &app,
        "POST",
        "/api/v1/people/999/items",
        Some(json!({ "keys": ["npub1x"] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
    assert_eq!(error["error"]["code"], "not_found");
    let (status, error) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{bob}/items"),
        Some(json!({ "keys": ["not-a-key"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["code"], "invalid_person");
}

#[tokio::test]
async fn the_check_route_gives_the_state_of_each_feed() {
    let (app, connector) = publisher_app().await;
    let (credit, payee) = credit_and_payee_feeds(&app, connector).await;
    let ann = new_person(&app, "Ann").await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{ann}/keys"),
        Some(json!({ "npub": "npub1ann" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, check) = call(&app, "GET", &format!("/api/v1/people/{ann}/check"), None).await;
    assert_eq!(status, StatusCode::OK, "{check}");
    let feeds = check["feeds"].as_array().unwrap();
    let credited = feeds
        .iter()
        .find(|f| f["feed_id"] == json!(credit))
        .unwrap();
    assert_eq!(credited["state"], "needs_correction", "{check}");
    assert_eq!(credited["findings"][0]["kind"], "credit_key", "{check}");
    // The payee feed has no wallet of the person, so the check lists no
    // payee finding for it.
    assert!(
        feeds.iter().all(|f| f["feed_id"] != json!(payee)),
        "{check}"
    );
    let (_, person) = call(&app, "GET", &format!("/api/v1/people/{ann}"), None).await;
    assert_eq!(person["needs_correction"], 1, "{person}");
    let (status, _) = call(&app, "GET", "/api/v1/people/999/check", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_check_route_gives_the_places_and_the_values_of_a_finding() {
    let (app, connector) = publisher_app().await;
    let (credit, _) = credit_and_payee_feeds(&app, connector).await;
    let ann = new_person(&app, "Ann").await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{ann}/keys"),
        Some(json!({ "npub": "npub1ann" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, check) = call(&app, "GET", &format!("/api/v1/people/{ann}/check"), None).await;
    assert_eq!(status, StatusCode::OK, "{check}");
    let credited = check["feeds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["feed_id"] == json!(credit))
        .unwrap();
    // The credit is in the channel, and it has no key of the person.
    assert_eq!(
        credited["findings"][0],
        json!({
            "kind": "credit_key",
            "role": "vocals",
            "group": "",
            "npub": "",
            "places": [{ "kind": "channel" }],
        }),
        "ADR 0120 item 6: a finding gives its values and its places. {check}"
    );
}

#[tokio::test]
async fn the_more_route_gives_the_items_of_the_feeds_that_match_a_person() {
    let (app, connector) = publisher_app().await;
    credit_and_payee_feeds(&app, connector).await;
    let ann = new_person(&app, "Ann").await;
    let (status, more) = call(&app, "GET", &format!("/api/v1/people/{ann}/more"), None).await;
    assert_eq!(status, StatusCode::OK, "{more}");
    let items = more["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "{more}");
    assert_eq!(items[1]["kind"], "payee");
    assert_eq!(items[1]["cause"], "same_name");
    assert_eq!(items[0]["kind"], "credit");
    assert_eq!(items[0]["name"], "Ann");
    assert_eq!(items[0]["cause"], "same_name");
    assert_eq!(items[0]["roles"], json!(["vocals"]));
    let (status, error) = call(&app, "GET", "/api/v1/people/999/more", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
    assert_eq!(error["error"]["code"], "not_found");
}

#[tokio::test]
async fn a_merge_moves_the_links_and_deletes_the_other_person() {
    let app = app();
    let keep = new_person(&app, "Ann").await;
    let other = new_person(&app, "Annie").await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{other}/wallets"),
        Some(json!({ "address": "annie@example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{keep}/merge"),
        Some(json!({ "other": other })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, person) = call(&app, "GET", &format!("/api/v1/people/{keep}"), None).await;
    assert_eq!(person["name"], "Ann");
    assert_eq!(person["wallets"][0]["key"]["address"], "annie@example.com");
    let (status, _) = call(&app, "GET", &format!("/api/v1/people/{other}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, error) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{keep}/merge"),
        Some(json!({ "other": keep })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["code"], "invalid_person");
    let (status, error) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{keep}/merge"),
        Some(json!({ "other": 999 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
}

#[tokio::test]
async fn a_dismissed_suggestion_leaves_the_list() {
    let (app, connector) = publisher_app().await;
    credit_and_payee_feeds(&app, connector).await;
    let (_, list) = call(&app, "GET", "/api/v1/people/suggestions", None).await;
    let suggestion = &list[0];
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/people/suggestions/dismiss",
        Some(json!({
            "identity": suggestion["identity"],
            "items": suggestion["item_keys"],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, list) = call(&app, "GET", "/api/v1/people/suggestions", None).await;
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn each_people_route_needs_the_token() {
    let app = app();
    let routes = [
        ("GET", "/api/v1/people"),
        ("POST", "/api/v1/people"),
        ("GET", "/api/v1/people/1"),
        ("PATCH", "/api/v1/people/1"),
        ("DELETE", "/api/v1/people/1"),
        ("PUT", "/api/v1/people/1/default-address"),
        ("POST", "/api/v1/people/1/save"),
        ("GET", "/api/v1/people/1/musicindex"),
        ("GET", "/api/v1/people/1/musicindex/c0ffee-01"),
        ("POST", "/api/v1/people/1/wallets"),
        ("DELETE", "/api/v1/people/1/wallets/1"),
        ("GET", "/api/v1/people/suggestions"),
        ("POST", "/api/v1/people/suggestions/make"),
        ("POST", "/api/v1/people/suggestions/dismiss"),
        ("POST", "/api/v1/people/1/changes"),
        ("POST", "/api/v1/people/1/changes/preview"),
        ("GET", "/api/v1/people/1/check"),
        ("GET", "/api/v1/people/1/detail-suggestions"),
        ("POST", "/api/v1/people/1/profile-addresses/dismiss"),
        ("POST", "/api/v1/people/1/items"),
        ("GET", "/api/v1/people/1/more"),
        ("POST", "/api/v1/people/1/merge"),
        ("GET", "/api/v1/feeds/1/members"),
    ];
    for (method, path) in routes {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, HOST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let (status, value, _) = send(&app, request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
        assert_eq!(value["error"]["code"], "token_required", "{method} {path}");
    }
}

// ---- The profile addresses and the export of the people (ADR 0098, task 064c) ----

/// An app with a lookup relay and the keysend scheme `http`, so that a test
/// server on loopback can answer the lookups.
fn profile_address_app(relay: &studio_core::testing::relay::TestRelay) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![relay.url()])
        .with_keysend(Some("http"))
        .with_address_check(std::sync::Arc::new(PassAll));
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

/// A person that has the Nostr key 1, whose profile gives a Lightning address
/// that has keysend data. Gives the app, the person and the address.
async fn person_with_profile_address() -> (
    App,
    i64,
    String,
    (
        studio_core::testing::relay::TestRelay,
        studio_core::testing::TestServer,
    ),
) {
    let server = studio_core::testing::TestServer::start(Vec::new());
    server.set(
        "/.well-known/keysend/ann",
        200,
        WALLET_KEYSEND.as_bytes().to_vec(),
    );
    let address = format!("ann@127.0.0.1:{}", server.port());
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(1, 100, &json!({ "lud16": address })));
    let app = profile_address_app(&relay);
    let id = new_person(&app, "Ann").await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/keys"),
        Some(json!({ "npub": nostr_npub(1) })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    (app, id, address, (relay, server))
}

#[tokio::test]
async fn a_profile_address_is_a_suggestion_and_the_user_adds_it_with_the_evidence_nostr() {
    let (app, id, address, _servers) = person_with_profile_address().await;
    let path = format!("/api/v1/people/{id}/profile-addresses");
    let (status, body) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body["suggestions"].as_array().unwrap();
    assert_eq!(list.len(), 1, "{body}");
    assert_eq!(list[0]["address"], address);
    assert_eq!(list[0]["wallet"]["address"], "02abc");
    assert_eq!(list[0]["in_person_wallet"], false);
    // The suggestion keeps no lookup.
    assert_eq!(collection_of(&app).keysend_lookups().unwrap().len(), 0);

    let (status, body) = call(&app, "POST", &path, Some(json!({ "address": address }))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert!(body["id"].as_i64().is_some(), "{body}");
    let (_, person) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    let wallets = person["wallets"].as_array().unwrap();
    assert_eq!(wallets.len(), 1, "{person}");
    assert_eq!(wallets[0]["evidence"], "nostr");
    assert_eq!(wallets[0]["recipient_type"], "node");
    assert_eq!(wallets[0]["key"]["address"], "02abc");
    assert_eq!(collection_of(&app).keysend_lookups().unwrap().len(), 1);

    // The address is a member of the wallet now, so it is no suggestion.
    let (_, body) = call(&app, "GET", &path, None).await;
    assert_eq!(body["suggestions"], json!([]));
}

/// A test server of the musicindex API with one artist feed and one feed
/// with no link, and the keysend data of the Lightning address of the artist
/// feed. It gives the server and that address.
fn musicindex_lookup_server() -> (studio_core::testing::TestServer, String) {
    let server = studio_core::testing::TestServer::start(Vec::new());
    let address = format!("ann@127.0.0.1:{}", server.port());
    server.set(
        "/v1/search",
        200,
        br#"{"data":[
            {"entity_type":"publisher","entity_id":"c0ffee-01","title":"Ann Lee","release_artist":"Ann Lee","release_artist_source":"itunes_author","two_way_link_count":2,"agreed_roles":["artist"],"confirmed_release_artists":["Ann Lee"]},
            {"entity_type":"publisher","entity_id":"c0ffee-02","title":"Ann Band","two_way_link_count":0,"agreed_roles":[],"confirmed_release_artists":[]}
        ]}"#
            .to_vec(),
    );
    server.set(
        "/v1/feeds/c0ffee-01",
        200,
        format!(
            r#"{{"data":{{"title":"Ann Lee","feed_url":"https://feed.example/ann.xml","image_url":"https://img.example/ann.png",
                "source_links":[{{"link_type":"website","url":"https://ann.example"}}],
                "source_ids":[{{"scheme":"nostr_npub","value":"npub1ann"}}],
                "payment_routes":[{{"recipient_name":"Ann","route_type":"lnaddress","address":"{address}","split":100,"fee":false}}]}}}}"#
        )
        .into_bytes(),
    );
    server.set(
        "/.well-known/keysend/ann",
        200,
        br#"{"pubkey":"02ln","customData":[]}"#.to_vec(),
    );
    (server, address)
}

/// The musicindex lookup of a person (ADR 0117). The search gives the rows
/// in three levels, and the values give the state of each value of a feed.
#[tokio::test]
async fn the_musicindex_lookup_gives_the_levels_and_the_states_of_a_feed() {
    let (server, address) = musicindex_lookup_server();
    let app = app_with_keysend(&server.url(""), Some("http"));
    let id = new_person(&app, "Ann Lee").await;

    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/musicindex?q=Ann%20Lee"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{body}");
    assert_eq!(rows[0]["feed_guid"], "c0ffee-01");
    assert_eq!(rows[0]["level"], "role");
    assert_eq!(
        rows[0]["kind"],
        json!({"type": "role", "roles": ["artist"]})
    );
    assert_eq!(rows[0]["albums"], 2);
    assert_eq!(rows[0]["host"], "feed.example");
    assert_eq!(rows[0]["has"]["wallet"], true);
    assert_eq!(rows[1]["feed_guid"], "c0ffee-02");
    assert_eq!(rows[1]["level"], "name_only");
    assert_eq!(rows[1]["kind"], json!({"type": "unverified"}));
    assert_eq!(rows[1]["host"], Value::Null);

    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/musicindex/c0ffee-01"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["picture"],
        json!({"value": "https://img.example/ann.png", "state": "new"})
    );
    assert_eq!(body["website"]["value"], "https://ann.example");
    assert_eq!(body["npub"]["value"], "npub1ann");
    assert_eq!(body["wallets"][0]["lookup"], "found");
    assert_eq!(body["wallets"][0]["key"]["address"], address.as_str());
    assert_eq!(body["wallets"][0]["state"], "new");

    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/musicindex/00000000-0000-0000-0000-000000000000"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "not_in_index");
    let (status, body) = call(&app, "GET", "/api/v1/people/999/musicindex?q=Ann", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    let (status, body) = call(&app, "GET", "/api/v1/people/999/musicindex/c0ffee-01", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

/// A save of a Lightning address from the musicindex lookup links the
/// address, and stores its lookup: the wallet then has the node of the
/// address too (ADR 0095, ADR 0117 item 7).
#[tokio::test]
async fn a_saved_lightning_address_from_musicindex_keeps_its_node() {
    let (server, address) = musicindex_lookup_server();
    let app = app_with_keysend(&server.url(""), Some("http"));
    let id = new_person(&app, "Ann Lee").await;
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/musicindex/c0ffee-01"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let wallet = body["wallets"][0]["key"].clone();
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/save"),
        Some(json!({ "wallets": [{ "key": wallet, "recipient_type": "lnaddress" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, body) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(
        body["wallets"][0]["key"]["address"],
        address.as_str(),
        "{body}"
    );
    let members: Vec<&str> = body["wallets"][0]["members"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["key"]["address"].as_str())
        .collect();
    assert!(members.contains(&"02ln"), "{body}");
}

#[tokio::test]
async fn the_detail_suggestions_give_img_and_href_from_the_profile_and_the_credits() {
    let relay = studio_core::testing::relay::TestRelay::start();
    relay.insert(profile_event(
        1,
        100,
        &json!({ "picture": "https://p.example/a.png", "website": "https://w.example" }),
    ));
    let app = profile_address_app(&relay);
    let id = new_person(&app, "Ann").await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/keys"),
        Some(json!({ "npub": nostr_npub(1) })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    rusqlite::Connection::open(app.data.path().join("collection.sqlite"))
        .unwrap()
        .execute_batch(
            "INSERT INTO feeds (id, medium, created_at) VALUES (1, 'music', 0);
             INSERT INTO feed_credits (feed_id, name, role, href, npub, img, seen_at)
             VALUES (1, 'Ann', 'vocals', 'https://c.example', '', 'https://c.example/a.png', 0);",
        )
        .unwrap();
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/detail-suggestions"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["img"][0]["value"], "https://p.example/a.png",
        "ADR 0113 item 1: the route must give img with the profile first. {body}"
    );
    assert_eq!(body["img"][0]["sources"][0]["kind"], "profile");
    assert_eq!(body["img"][0]["sources"][0]["npub"], nostr_npub(1));
    assert_eq!(body["img"][1]["sources"][0]["kind"], "credit");
    assert_eq!(body["img"][1]["sources"][0]["feed_ids"], json!([1]));
    assert_eq!(body["href"][0]["value"], "https://w.example");
    assert_eq!(body["href"][1]["value"], "https://c.example");
    let (status, _) = call(&app, "GET", "/api/v1/people/999/detail-suggestions", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_dismissed_profile_address_does_not_come_back_and_a_merge_keeps_the_dismissal() {
    let (app, id, address, _servers) = person_with_profile_address().await;
    let path = format!("/api/v1/people/{id}/profile-addresses");
    let (status, body) = call(
        &app,
        "POST",
        &format!("{path}/dismiss"),
        Some(json!({ "address": address })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, body) = call(&app, "GET", &path, None).await;
    assert_eq!(
        body["suggestions"],
        json!([]),
        "ADR 0098 item 18: a dismissed address must not come back. Check list_profile_addresses."
    );
    // A merge moves the dismissal to the person that it keeps.
    let keep = new_person(&app, "Keep").await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{keep}/merge"),
        Some(json!({ "other": id })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{keep}/profile-addresses"),
        None,
    )
    .await;
    assert_eq!(
        body["suggestions"],
        json!([]),
        "ADR 0098 item 18: a merge must keep the dismissal. Check merge_people."
    );
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/people/999/profile-addresses/dismiss",
        Some(json!({ "address": address })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_profile_address_with_no_keysend_data_is_refused() {
    let (app, id, _, (_relay, server)) = person_with_profile_address().await;
    let none = format!("cy@127.0.0.1:{}", server.port());
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/profile-addresses"),
        Some(json!({ "address": none })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_person");
    let (_, person) = call(&app, "GET", &format!("/api/v1/people/{id}"), None).await;
    assert_eq!(person["wallets"], json!([]));
}

#[tokio::test]
async fn with_no_keysend_scheme_the_profile_addresses_are_empty() {
    let app = app();
    let id = new_person(&app, "Ann").await;
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/people/{id}/profile-addresses"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["suggestions"], json!([]));
}

#[tokio::test]
async fn the_export_gives_each_link_with_its_evidence_and_the_keysend_addresses_and_no_secret() {
    let (app, id, address, _servers) = person_with_profile_address().await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/profile-addresses"),
        Some(json!({ "address": address })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, headers, bytes) =
        send_raw(&app, request("GET", "/api/v1/people/export", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers[header::CONTENT_DISPOSITION],
        "attachment; filename=\"studio-people.json\""
    );
    let text = String::from_utf8(bytes).unwrap();
    for secret in ["secret", "passphrase", "password", "token"] {
        assert!(!text.contains(secret), "the export has the text {secret}");
    }
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["format"], "studio-people");
    assert_eq!(body["version"], 1);
    assert!(body["exported_at"].as_i64().unwrap() > 0);
    let people = body["people"].as_array().unwrap();
    assert_eq!(people.len(), 1, "{body}");
    let person = &people[0];
    assert_eq!(person["name"], "Ann");
    assert_eq!(person["keys"][0]["evidence"], "user");
    assert!(person["keys"][0]["linked_at"].as_i64().unwrap() > 0);
    let wallet = &person["wallets"][0];
    assert_eq!(wallet["evidence"], "nostr");
    assert!(wallet["linked_at"].as_i64().unwrap() > 0);
    assert_eq!(wallet["addresses"][0]["address"], address);
    assert_eq!(wallet["addresses"][0]["evidence"], "keysend");
    assert!(wallet["addresses"][0]["checked_at"].as_i64().unwrap() > 0);
    assert_eq!(person["memberships"], json!([]));
    assert_eq!(person["default_address"]["address"], "02abc");
}

// ---- The change sets of a person (ADR 0098 item 20, task 064b) ----

/// A person Ann with the wallet `ann@example.com`, and two feeds that pay
/// only Bob. Gives the app, the person and the two feed IDs.
async fn person_and_two_feeds() -> (App, i64, [i64; 2]) {
    let (app, connector) = publisher_app().await;
    let a = add_paid_feed(
        &app,
        connector,
        "a.xml",
        &paid_feed("A", "ga", &[BOB], &[], ""),
    )
    .await;
    let b = add_paid_feed(
        &app,
        connector,
        "b.xml",
        &paid_feed("B", "gb", &[BOB], &[], ""),
    )
    .await;
    let id = new_person(&app, "Ann").await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/wallets"),
        Some(json!({ "address": "ann@example.com", "recipient_type": "lnaddress" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    (app, id, [a, b])
}

#[tokio::test]
async fn the_preview_of_a_person_operation_changes_no_draft_and_the_change_set_lists_the_person() {
    let (app, id, feeds) = person_and_two_feeds().await;
    let body = json!({
        "operation": {
            "kind": "add_to_releases",
            "payee_percent": 10,
            "credit": true,
            "support_link": null,
        },
        "feed_ids": feeds,
    });
    let (status, preview) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/changes/preview"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    let rows = preview["feeds"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{preview}");
    for row in rows {
        assert_eq!(row["outcome"]["outcome"], "changed", "{preview}");
        assert_eq!(row["outcome"]["parts"].as_array().unwrap().len(), 2);
    }
    let collection = collection_of(&app);
    for feed in feeds {
        assert!(
            collection
                .draft(studio_core::collection::FeedId(feed))
                .unwrap()
                .is_none(),
            "ADR 0098 item 20: a preview must make no draft."
        );
    }
    assert_eq!(collection.change_set_rows().unwrap().len(), 0);

    let (status, set) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{id}/changes"),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    assert_eq!(set["person_id"], id);
    let row = change_row(&set, feeds[0]);
    assert_eq!(row["outcome"]["outcome"], "changed");
    assert_eq!(row["outcome"]["parts"][0]["part"], "payee");
    assert_eq!(row["outcome"]["parts"][1]["part"], "credit");
    let (_, list) = call(&app, "GET", "/api/v1/change-sets", None).await;
    assert_eq!(list[0]["person_id"], id, "{list}");
}

/// A publisher feed with the credit of `Ann` in its channel, and an album.
#[tokio::test]
async fn the_members_route_gives_each_membership_and_each_channel_credit() {
    let (app, connector) = publisher_app().await;
    let band_xml = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
        <title>Band</title><podcast:guid>band-081</podcast:guid>\
        <podcast:medium>publisher</podcast:medium>\
        <podcast:person role=\"vocalist\" npub=\"npub1ann\">Ann</podcast:person>\
        <item><guid>band-081-t1</guid><title>Release</title></item></channel></rss>";
    let band = add_paid_feed(&app, connector, "band.xml", band_xml).await;
    let album = add_paid_feed(
        &app,
        connector,
        "album.xml",
        &paid_feed("Album", "ga", &[], &[], ""),
    )
    .await;
    let ann = new_person(&app, "Ann").await;
    let bob = new_person(&app, "Bob").await;
    for person in [ann, bob] {
        let (status, body) = call(
            &app,
            "POST",
            &format!("/api/v1/people/{person}/members"),
            Some(json!({ "feed_id": band })),
        )
        .await;
        assert!(status.is_success(), "{body}");
    }

    let (status, body) = call(&app, "GET", &format!("/api/v1/feeds/{band}/members"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let members = body["members"].as_array().unwrap();
    assert_eq!(members.len(), 2, "{body}");
    assert_eq!(members[0]["name"], "Ann", "{body}");
    assert_eq!(members[0]["member"], true, "{body}");
    assert_eq!(members[0]["in_credits"], true, "{body}");
    assert_eq!(members[1]["name"], "Bob", "{body}");
    assert_eq!(members[1]["member"], true, "{body}");
    assert_eq!(members[1]["in_credits"], false, "{body}");

    let (status, body) = call(&app, "GET", &format!("/api/v1/feeds/{album}/members"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["members"], json!([]), "{body}");

    // ADR 0118 item 5: a channel credit needs a publisher feed.
    let credit = json!({
        "operation": { "kind": "credit_feed", "role": "Vocalist", "group": "" },
        "feed_ids": [album],
    });
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{bob}/changes"),
        Some(credit.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let credit = json!({
        "operation": { "kind": "credit_feed", "role": "Vocalist", "group": "" },
        "feed_ids": [band],
    });
    let (status, set) = call(
        &app,
        "POST",
        &format!("/api/v1/people/{bob}/changes"),
        Some(credit),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    assert_eq!(
        change_row(&set, band)["outcome"]["outcome"],
        "changed",
        "{set}"
    );
}

#[tokio::test]
async fn a_person_operation_that_is_not_valid_gives_422() {
    let (app, id, feeds) = person_and_two_feeds().await;
    let path = format!("/api/v1/people/{id}/changes");
    let empty = json!({
        "operation": { "kind": "remove", "payee": true, "credit": false },
        "feed_ids": [],
    });
    let (status, body) = call(&app, "POST", &path, Some(empty)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_change_set");
    let nothing = json!({
        "operation": { "kind": "remove", "payee": false, "credit": false },
        "feed_ids": feeds,
    });
    for route in [path.clone(), format!("{path}/preview")] {
        let (status, body) = call(&app, "POST", &route, Some(nothing.clone())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(body["error"]["code"], "invalid_person_operation");
    }
    let stranger = json!({
        "operation": {
            "kind": "use_address",
            "key": { "address": "stranger@example.com" },
        },
        "feed_ids": feeds,
    });
    let (status, body) = call(&app, "POST", &path, Some(stranger)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "invalid_person_operation");
    let (_, list) = call(&app, "GET", "/api/v1/change-sets", None).await;
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn the_recipient_list_marks_a_fee() {
    let (app, connector) = publisher_app().await;
    add_paid_feed(
        &app,
        connector,
        "fee.xml",
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
         <title>Fee</title><podcast:guid>g-fee</podcast:guid>\
         <podcast:medium>music</podcast:medium>\
         <podcast:value type=\"lightning\" method=\"keysend\">\
         <podcast:valueRecipient name=\"Artist\" address=\"02aa\" split=\"90\" type=\"node\"/>\
         <podcast:valueRecipient name=\"Host\" address=\"03bb\" split=\"10\" type=\"node\" fee=\"True\"/>\
         </podcast:value>\
         <item><guid>fee-t1</guid><title>Track</title></item></channel></rss>",
    )
    .await;
    let (status, list) = call(&app, "GET", "/api/v1/recipients", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let entry = |address: &str| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|e| e["address"] == address)
            .cloned()
            .unwrap()
    };
    assert_eq!(entry("03bb")["fee"], true, "{list}");
    assert_eq!(entry("03bb")["addresses"][0]["fee"], true, "{list}");
    assert_eq!(entry("02aa")["fee"], false, "{list}");
}

#[cfg(unix)]
fn mode_of(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[cfg(unix)]
#[test]
fn a_new_workspace_folder_and_grants_have_the_mode_0700() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    Workspace::open(&data).unwrap();
    for path in [data.clone(), data.join("grants")] {
        assert_eq!(
            mode_of(&path),
            0o700,
            "ADR 0104 item 2: {} must have the mode 0700. Make it with private_dir() in Workspace::open.",
            path.display()
        );
    }
}

#[cfg(unix)]
#[test]
fn an_open_repairs_the_modes_of_an_older_installation() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o755)).unwrap();
    let vault = data.join("vault.json");
    std::fs::write(&vault, "{}").unwrap();
    std::fs::set_permissions(&vault, std::fs::Permissions::from_mode(0o644)).unwrap();
    Workspace::open(&data).unwrap();
    assert_eq!(
        mode_of(&data),
        0o700,
        "ADR 0104 item 2: Workspace::open must set the mode 0700 on an existing data folder."
    );
    assert_eq!(
        mode_of(&vault),
        0o600,
        "ADR 0104 item 2: Workspace::open must set the mode 0600 on an existing vault.json."
    );
}

/// A clock that a test moves. The offset is in seconds after the start.
fn test_clock() -> (crate::Clock, std::sync::Arc<std::sync::atomic::AtomicU64>) {
    let start = std::time::Instant::now();
    let offset = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let shared = std::sync::Arc::clone(&offset);
    let clock = crate::Clock::new(move || {
        start + std::time::Duration::from_secs(shared.load(std::sync::atomic::Ordering::SeqCst))
    });
    (clock, offset)
}

fn app_with_clock(clock: crate::Clock) -> (App, crate::WorkspaceHandle) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_public_origin("http://127.0.0.1:4100")
        .with_clock(clock);
    let (workspace_routes, handle) = crate::workspace_router(workspace);
    let router =
        crate::ui_router()
            .merge(workspace_routes)
            .layer(axum::middleware::from_fn_with_state(
                Access::with_token(TOKEN, 4100),
                crate::access::check,
            ));
    (
        App {
            router,
            data,
            feeds,
        },
        handle,
    )
}

const SENSITIVE: [(&str, &str); 5] = [
    ("POST", "/api/v1/vault/signer-key"),
    ("POST", "/api/v1/vault/passphrase"),
    ("POST", "/api/v1/apps"),
    ("POST", "/api/v1/vault/recovery-key"),
    ("POST", "/api/v1/account/export"),
];

fn sensitive_body(path: &str) -> Option<Value> {
    match path {
        "/api/v1/vault/passphrase" => Some(json!({ "passphrase": "staple orange umbrella river" })),
        "/api/v1/apps" => Some(json!({ "name": "Show tool" })),
        "/api/v1/account/export" => Some(json!({})),
        _ => None,
    }
}

#[tokio::test]
async fn a_sensitive_change_needs_an_unlock_in_the_last_five_minutes() {
    let (clock, offset) = test_clock();
    let (app, _handle) = app_with_clock(clock);
    make_vault(&app).await;
    for (method, path) in SENSITIVE {
        let (status, body) = call(&app, method, path, sensitive_body(path)).await;
        assert!(
            status.is_success(),
            "ADR 0106 item 2: {path} must work right after the unlock: {body}"
        );
    }
    // Six minutes later the vault is still open, but the unlock is old.
    offset.store(6 * 60, std::sync::atomic::Ordering::SeqCst);
    for (method, path) in SENSITIVE {
        let (status, body) = call(&app, method, path, sensitive_body(path)).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "ADR 0106 item 2: {path} must give 401 unlock_required with an unlock older than 5 minutes. Call require_recent_unlock in the route."
        );
        assert_eq!(body["error"]["code"], "unlock_required", "{path}");
    }
    // A new unlock makes the changes work again. The passphrase is the
    // current one here, because the first loop changed it.
    lock(&app).await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": "staple orange umbrella river" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for (method, path) in SENSITIVE {
        let (status, body) = call(&app, method, path, sensitive_body(path)).await;
        assert!(
            status.is_success(),
            "ADR 0106 item 2: {path} must work after a new unlock: {body}"
        );
    }
    // Four minutes after that unlock the change still works.
    offset.store(10 * 60 - 1, std::sync::atomic::Ordering::SeqCst);
    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert!(status.is_success(), "ADR 0106 item 2: {body}");
}

#[tokio::test]
async fn a_vault_creation_and_a_recovery_unlock_each_count_as_a_recent_unlock() {
    let (clock, offset) = test_clock();
    let (app, handle) = app_with_clock(clock);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE, "recovery_key": true })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let recovery = body["recovery_key"].as_str().unwrap().to_owned();
    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0106 item 1: the creation of a vault is an unlock. Mark the unlock in create_vault: {body}"
    );
    lock(&app).await;
    offset.store(3600, std::sync::atomic::Ordering::SeqCst);
    handle.unlock_with_recovery_key(&recovery).await.unwrap();
    let (status, body) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "ADR 0106 item 1: a recovery unlock is an unlock. Mark the unlock in unlock_with_recovery_key: {body}"
    );
    // A lock clears the time, so a lock and no unlock gives 423.
    lock(&app).await;
    let (status, _) = call(&app, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(status, StatusCode::LOCKED);
}

// ---- A server that holds no assets (task 070, ADR 0105) ----

/// An app with the rules of a server. `hold` is `--hold-assets on`.
fn app_assets(
    hold: bool,
    ticket_lifetime: Option<std::time::Duration>,
) -> (App, crate::WorkspaceHandle) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let mut workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_server_rules(crate::ServerRules {
            storage_limit: 1 << 30,
            folder: data.path().to_path_buf(),
            library_root: data.path().join("library"),
            hold_assets: hold,
            vault_idle: std::time::Duration::from_secs(600),
            vault_idle_max: std::time::Duration::from_secs(3600),
        })
        .unwrap();
    if let Some(lifetime) = ticket_lifetime {
        workspace = workspace.with_ticket_lifetime(lifetime);
    }
    let (workspace_routes, handle) = crate::workspace_router(workspace);
    let router =
        crate::ui_router()
            .merge(workspace_routes)
            .layer(axum::middleware::from_fn_with_state(
                Access::with_token(TOKEN, 4100),
                crate::access::check,
            ));
    (
        App {
            router,
            data,
            feeds,
        },
        handle,
    )
}

/// Like [`set_up`], for a server app. The server has no local folder host
/// in its API, so the test adds the connector through the collection file.
async fn set_up_server(app: &App) -> i64 {
    let (status, _) = call(
        app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let folder = studio_core::connector::LocalFolder::new(
        app.feeds.path(),
        Some("https://music.example".to_owned()),
    );
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let connector = collection
        .add_connector(
            studio_core::connector::LocalFolder::KIND,
            "Home",
            &folder.config(),
            None,
        )
        .unwrap();
    drop(collection);
    let (status, feed) = call(
        app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector.0, "path": "album.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    set_library_root(app);
    feed["id"].as_i64().unwrap()
}

/// Put the uploaded file in the draft as the enclosure of the first track.
async fn use_as_enclosure(app: &App, id: i64, file: &Value) {
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, saved) = call(
        app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
}

/// Write the draft of a feed with the ID of its current diff.
async fn write_now(app: &App, id: i64) -> (StatusCode, Value) {
    let (_, diff) = call(app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await
}

async fn release_file(app: &App, id: i64, path: &str) -> Value {
    let (status, body) = call(app, "GET", &format!("/api/v1/feeds/{id}/files"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["files"]
        .as_array()
        .and_then(|files| files.iter().find(|f| f["path"] == path))
        .cloned()
        .unwrap_or_else(|| panic!("no file {path} in {body}"))
}

fn download_request(ticket: &str) -> Request<Body> {
    Request::builder()
        .uri(format!("/media/download?ticket={ticket}"))
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap()
}

async fn file_ticket(app: &App, file: i64) -> String {
    let (status, body) = call(
        app,
        "POST",
        "/api/v1/media/tickets",
        Some(json!({ "file": file })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["ticket"].as_str().unwrap().to_owned()
}

/// The path of each file under `root`, for a comparison before and after.
fn files_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(&folder).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[tokio::test]
async fn status_gives_hold_assets_false_with_off_and_true_with_on_and_on_the_desktop() {
    let (off, _) = app_assets(false, None);
    let (status, body) = call(&off, "GET", "/api/v1/status", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["hold_assets"], false,
        "ADR 0105 item 1: status gives hold_assets false with `off`"
    );
    let (on, _) = app_assets(true, None);
    let (_, body) = call(&on, "GET", "/api/v1/status", None).await;
    assert_eq!(body["hold_assets"], true);
    let (_, body) = call(&app(), "GET", "/api/v1/status", None).await;
    assert_eq!(
        body["hold_assets"], true,
        "ADR 0105 item 2: the desktop holds assets"
    );
    assert!(
        body["storage"].is_null(),
        "the desktop has no storage rules"
    );
    let (_, body) = call(&off, "GET", "/api/v1/status", None).await;
    assert!(
        body["storage"]["used"].is_u64() && body["storage"]["limit"].is_u64(),
        "ADR 0105 item 17: a server gives the space in use and the limit"
    );
}

#[tokio::test]
async fn with_off_the_download_routes_and_the_folder_scan_give_403_not_on_server() {
    let (app, _) = app_assets(false, None);
    let id = set_up_server(&app).await;
    let routes = [
        ("GET", format!("/api/v1/feeds/{id}/downloads"), None),
        (
            "POST",
            format!("/api/v1/feeds/{id}/downloads"),
            Some(json!({ "all": true })),
        ),
        ("DELETE", format!("/api/v1/feeds/{id}/downloads/1"), None),
        ("POST", "/api/v1/library/scan".to_owned(), None),
        (
            "POST",
            format!("/api/v1/feeds/{id}/library-changes"),
            Some(json!({ "action": "forget", "file": 1 })),
        ),
    ];
    for (method, path, body) in routes {
        let (status, body) = call(&app, method, &path, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "ADR 0105 item 14: {method} {path} gives 403 with `off`. Call require_held_assets first. {body}"
        );
        assert_eq!(body["error"]["code"], "not_on_server", "{method} {path}");
    }
    // With `on`, the routes work as before.
    let (on, _) = app_assets(true, None);
    let id = set_up_server(&on).await;
    let (status, _) = call(&on, "GET", &format!("/api/v1/feeds/{id}/downloads"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(&on, "POST", "/api/v1/library/scan", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn the_download_route_sends_a_file_from_its_host_and_writes_no_file() {
    let (app, _) = app_assets(false, None);
    let id = set_up_server(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, file) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let file_id = file["library_file_id"].as_i64().unwrap();

    // A pending file comes from the release folder.
    let ticket = file_ticket(&app, file_id).await;
    let (status, headers, body) = send_raw(&app, download_request(&ticket)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, bytes);
    assert_eq!(
        headers[header::CONTENT_DISPOSITION],
        "attachment; filename=\"tagged.mp3\"; filename*=UTF-8''tagged.mp3"
    );

    // After the write the server has no bytes. The route reads the copy on
    // the host through its connector.
    use_as_enclosure(&app, id, &file).await;
    let (status, written) = write_now(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{written}");
    let row = release_file(&app, id, "tagged.mp3").await;
    assert_eq!(
        row["local"], false,
        "ADR 0105 item 5: the write removes the bytes of the sent asset"
    );
    assert!(
        !app.data.path().join("library/album/tagged.mp3").exists(),
        "ADR 0105 item 5: the release folder has no audio after the write"
    );
    let before = files_under(app.data.path());
    let ticket = file_ticket(&app, file_id).await;
    // The audio route reads no host: a file that only a host has loads from
    // the host (ADR 0109 item 6).
    let (status, body, _) = send(&app, audio_request(&ticket, None)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "ADR 0109 item 6: the audio route sends only a held file. Remove the read from a host. {body}"
    );
    assert_eq!(body["error"]["code"], "file_not_held");
    let (_, held) = call(&app, "GET", "/api/v1/media/held", None).await;
    assert_eq!(held["files"], json!([]), "a file on a host is not held");
    let (status, headers, body) = send_raw(&app, download_request(&ticket)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body, bytes,
        "ADR 0105 item 15: the route sends the bytes of the copy on the host"
    );
    assert_eq!(headers[header::CONTENT_TYPE], "audio/mpeg");
    assert!(
        headers[header::CONTENT_DISPOSITION]
            .to_str()
            .unwrap()
            .starts_with("attachment;")
    );
    assert_eq!(
        files_under(app.data.path()),
        before,
        "ADR 0105 item 15: the download route writes no file on the server"
    );
}

#[tokio::test]
async fn the_download_route_refuses_an_unknown_ticket_a_ticket_of_another_user_and_an_expired_ticket()
 {
    let (first, _) = app_assets(false, None);
    let id = set_up_server(&first).await;
    let (status, file) = upload(
        &first,
        id,
        "tagged.mp3",
        std::fs::read(media_path("tagged.mp3")).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let file_id = file["library_file_id"].as_i64().unwrap();
    let ticket = file_ticket(&first, file_id).await;

    // The workspace of another user does not know the ticket.
    let (second, _) = app_assets(false, None);
    let second_id = set_up_server(&second).await;
    let (status, _) = upload(
        &second,
        second_id,
        "tagged.mp3",
        std::fs::read(media_path("tagged.mp3")).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    for (name, app, ticket) in [
        ("another user", &second, ticket.as_str()),
        (
            "an unknown ticket",
            &first,
            "0123456789abcdef0123456789abcdef",
        ),
    ] {
        let (status, body, _) = send(app, download_request(ticket)).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "ADR 0105 item 15: {name} gives 403. The ticket store belongs to one workspace. {body}"
        );
        assert_eq!(body["error"]["code"], "ticket_not_valid");
    }
    // A ticket that is valid for no time is expired at its first use.
    let (expired, _) = app_assets(false, Some(std::time::Duration::ZERO));
    let expired_id = set_up_server(&expired).await;
    let (status, file) = upload(
        &expired,
        expired_id,
        "tagged.mp3",
        std::fs::read(media_path("tagged.mp3")).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    let ticket = file_ticket(&expired, file["library_file_id"].as_i64().unwrap()).await;
    let (status, body, _) = send(&expired, download_request(&ticket)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "ADR 0105 item 15: an expired ticket gives 403 {body}"
    );
    assert_eq!(body["error"]["code"], "ticket_not_valid");

    // A ticket for a file that does not exist is refused when it is made.
    let (status, _) = call(
        &first,
        "POST",
        "/api/v1/media/tickets",
        Some(json!({ "file": 9999 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A pending file at 6 days keeps its bytes. At 8 days the hourly step
/// removes them. "Add the File Again" gives the row its bytes and clears
/// `removed_at`, and the write then succeeds (ADR 0105 item 10).
#[tokio::test]
#[expect(clippy::too_many_lines, reason = "the test follows one user flow")]
async fn add_the_file_again_gives_a_removed_file_its_bytes_and_the_write_then_succeeds() {
    let (app, handle) = app_assets(false, None);
    let id = set_up_server(&app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (status, file) = upload(&app, id, "tagged.mp3", bytes.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{file}");
    use_as_enclosure(&app, id, &file).await;
    let folder_file = app.data.path().join("library/album/tagged.mp3");
    assert!(folder_file.exists());
    let row = release_file(&app, id, "tagged.mp3").await;
    assert_eq!(row["local"], true);
    let imported = row["imported_at"].as_i64().unwrap();
    assert_eq!(
        row["expires_at"].as_i64(),
        Some(imported + 7 * 86_400),
        "ADR 0105 item 8: a pending file expires 7 days after its import"
    );

    let day = 86_400;
    handle
        .with_workspace(move |w| crate::uploads::sweep_pending(w, imported + 6 * day).unwrap())
        .await;
    assert!(
        folder_file.exists(),
        "ADR 0105 item 8: a pending file at 6 days keeps its bytes"
    );
    handle
        .with_workspace(move |w| crate::uploads::sweep_pending(w, imported + 8 * day).unwrap())
        .await;
    assert!(
        !folder_file.exists(),
        "ADR 0105 item 10: no bytes at 8 days"
    );
    let row = release_file(&app, id, "tagged.mp3").await;
    assert_eq!(row["local"], false);
    assert_eq!(row["removed_at"].as_i64(), Some(imported + 8 * day));
    assert_eq!(
        row["in_use"], true,
        "ADR 0105 item 10: the draft still uses a removed file, so the Files page counts it as missing"
    );

    // The draft keeps the address, and the write refuses it.
    let (status, body) = write_now(&app, id).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "file_missing");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("tagged.mp3")
    );

    // A wrong file, or a file that Studio did not remove, does not work.
    let (status, started) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/uploads"),
        Some(json!({ "name": "tagged.mp3", "size": 5, "replace": "other.mp3" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{started}");
    assert_eq!(started["error"]["code"], "not_removed");
    let (status, started) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/uploads"),
        Some(json!({ "name": "tagged.mp3", "size": 5, "replace": "tagged.mp3" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let wrong = started["upload"].as_str().unwrap().to_owned();
    let (status, _) = put_part(&app, &wrong, 0, b"wrong".to_vec()).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/uploads/{wrong}/finish"),
        None,
    )
    .await;
    assert_ne!(status, StatusCode::CREATED, "{body}");
    assert!(!folder_file.exists());

    // The same file again, with `replace` for its path.
    let (status, started) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/uploads"),
        Some(json!({ "name": "tagged.mp3", "size": bytes.len(), "replace": "tagged.mp3" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let upload_id = started["upload"].as_str().unwrap().to_owned();
    let (status, _) = put_part(&app, &upload_id, 0, bytes.clone()).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/uploads/{upload_id}/finish"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(
        body["library_file_id"], row["id"],
        "ADR 0105 item 10: the file keeps its row"
    );
    assert_eq!(std::fs::read(&folder_file).unwrap(), bytes);
    let row = release_file(&app, id, "tagged.mp3").await;
    assert_eq!(
        row["local"], true,
        "ADR 0105 item 10: the row has its bytes"
    );
    assert_eq!(row["removed_at"], Value::Null);
    assert!(row["imported_at"].as_i64().unwrap() >= imported);
    assert!(incoming_is_empty(&app));

    let (status, written) = write_now(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{written}");
    assert_eq!(
        std::fs::read(app.feeds.path().join("album/tagged.mp3")).unwrap(),
        bytes
    );
}

/// ADR 0055 item 16: the hourly step and the step at the start do not use
/// the vault, so they do not extend the vault window.
#[tokio::test]
async fn the_asset_sweeps_do_not_change_the_last_use_of_the_vault() {
    let (app, handle) = app_assets(false, None);
    let id = set_up_server(&app).await;
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    // A host that has a secret in the vault.
    collection
        .add_connector(
            studio_core::connector::LocalFolder::KIND,
            "Secret",
            "{}",
            Some("secret-1"),
        )
        .unwrap();
    drop(collection);
    let _ = id;
    let before = handle.with_workspace(|w| w.vault_last_used()).await;
    std::thread::sleep(std::time::Duration::from_millis(20));
    let after = handle
        .with_workspace(|w| {
            crate::uploads::sweep_pending(w, i64::MAX / 2).unwrap();
            crate::uploads::sweep_hosted(w).unwrap();
            w.vault_last_used()
        })
        .await;
    assert_eq!(
        before, after,
        "ADR 0055 item 16: a sweep must not touch the vault. Use hosts_without_secrets."
    );
}

// ---- Export and import of an account (task 071b, 071c, ADR 0107) ----

/// Make an export ticket and read the archive of the ticket. Gives the
/// ticket response, and the status, the headers and the body of the stream.
async fn export_archive(
    app: &App,
    body: Value,
) -> (Value, StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let (status, ticket) = call(app, "POST", "/api/v1/account/export", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{ticket}");
    let name = ticket["ticket"].as_str().unwrap().to_owned();
    let (status, headers, bytes) = send_raw(app, download_request(&name)).await;
    (ticket, status, headers, bytes)
}

fn tar_names(bytes: &[u8]) -> Vec<String> {
    let mut archive = tar::Archive::new(bytes);
    archive
        .entries()
        .unwrap()
        .map(|e| e.unwrap().path().unwrap().display().to_string())
        .collect()
}

#[tokio::test]
async fn a_new_recovery_key_opens_the_vault_and_needs_a_recent_unlock() {
    let (clock, offset) = test_clock();
    let (app, handle) = app_with_clock(clock);
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = call(&app, "POST", "/api/v1/vault/recovery-key", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let key = body["recovery_key"].as_str().unwrap().to_owned();
    assert_eq!(key.len(), 39, "32 digits in 8 groups: {key}");
    lock(&app).await;
    handle.unlock_with_recovery_key(&key).await.unwrap();
    let (_, status_body) = call(&app, "GET", "/api/v1/status", None).await;
    assert_eq!(
        status_body["vault"], "open",
        "ADR 0107 item 4: the new recovery key must open the vault. Save the vault in new_recovery_key."
    );
    offset.store(6 * 60, std::sync::atomic::Ordering::SeqCst);
    let (status, body) = call(&app, "POST", "/api/v1/vault/recovery-key", None).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "ADR 0107 item 4: the recovery key route needs a recent unlock: {body}"
    );
    assert_eq!(body["error"]["code"], "unlock_required");
    assert!(
        !body.to_string().contains(&key),
        "the error must not repeat a key"
    );
}

#[tokio::test]
async fn the_export_facts_give_the_counts_and_if_the_vault_can_move() {
    let (app, handle) = app_assets(true, None);
    set_up_server(&app).await;
    let (status, facts) = call(&app, "GET", "/api/v1/account/export-facts", None).await;
    assert_eq!(status, StatusCode::OK, "{facts}");
    assert_eq!(facts["feeds"], 1);
    assert_eq!(facts["people"], 0);
    assert_eq!(
        facts["movable"], true,
        "ADR 0107 item 4: a passphrase vault can move"
    );
    assert_eq!(facts["methods"], json!(["passphrase"]));
    assert!(facts["archive_bytes"].as_u64().unwrap() > 0, "{facts}");
    assert_eq!(facts["holds_assets"], true);
    drop(handle);
    // A vault with only a passkey wrapper cannot open on a different server.
    let (app, handle) = app_assets(true, None);
    handle.create_vault(None, false).await.unwrap();
    handle
        .add_passkey_wrapper("cred", [1; 32], zeroize::Zeroizing::new([2; 32]))
        .await
        .unwrap();
    let (_, facts) = call(&app, "GET", "/api/v1/account/export-facts", None).await;
    assert_eq!(
        facts["movable"], false,
        "ADR 0107 item 4: a vault with only a passkey wrapper must not be movable"
    );
    assert_eq!(facts["methods"], json!(["passkey"]));
    let (status, body) = call(&app, "POST", "/api/v1/account/export", Some(json!({}))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(
        body["error"]["code"], "vault_not_movable",
        "ADR 0107 item 4: the export of a vault with only passkeys needs a recovery key first"
    );
    // The recovery key makes the vault movable.
    let (status, _) = call(&app, "POST", "/api/v1/vault/recovery-key", None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, facts) = call(&app, "GET", "/api/v1/account/export-facts", None).await;
    assert_eq!(facts["movable"], true);
}

#[tokio::test]
async fn the_export_ticket_streams_an_archive_and_writes_no_copy() {
    let (app, _) = app_assets(true, None);
    set_up_server(&app).await;
    // A loose file in the library, and a file the export must never see.
    std::fs::create_dir_all(app.data.path().join("library")).unwrap();
    std::fs::write(app.data.path().join("library/loose.mp3"), b"audio").unwrap();
    let before = files_under(app.data.path());
    let (ticket, status, headers, bytes) =
        export_archive(&app, json!({ "include_assets": true })).await;
    assert_eq!(status, StatusCode::OK);
    let disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        disposition.starts_with("attachment;") && disposition.contains(".tar"),
        "ADR 0107 item 7: the archive must come as an attachment: {disposition}"
    );
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "application/x-tar"
    );
    assert_eq!(
        ticket["file_name"].as_str().unwrap().len(),
        "studio-2026-11-01.tar".len()
    );
    let manifest = studio_core::account_archive::check_archive(&bytes[..])
        .unwrap_or_else(|e| panic!("ADR 0107 item 11: the export must pass check_archive: {e}"));
    assert_eq!(manifest.email, None, "the desktop has no email");
    let names = tar_names(&bytes);
    assert!(names.contains(&"library/loose.mp3".to_owned()), "{names:?}");
    assert!(names.contains(&"vault.json".to_owned()));
    assert_eq!(
        files_under(app.data.path()),
        before,
        "ADR 0107 item 7: the export must write no copy of the archive and leave no temporary file"
    );
    // The ticket works one time.
    let (status, _, _) = send_raw(&app, download_request(ticket["ticket"].as_str().unwrap())).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "ADR 0107 item 7: an archive ticket must work one time. Use take_archive."
    );
    // Without the audio, the loose file stays out.
    let (_, status, _, bytes) = export_archive(&app, json!({ "include_assets": false })).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!tar_names(&bytes).contains(&"library/loose.mp3".to_owned()));
}

#[tokio::test]
async fn a_server_that_holds_no_assets_exports_the_files_that_it_keeps() {
    let (app, _) = app_assets(false, None);
    set_up_server(&app).await;
    std::fs::create_dir_all(app.data.path().join("library")).unwrap();
    std::fs::write(app.data.path().join("library/waiting.mp3"), b"audio").unwrap();
    let (ticket, _, _, bytes) = export_archive(&app, json!({ "include_assets": false })).await;
    assert_eq!(ticket["include_assets"], true);
    assert!(
        tar_names(&bytes).contains(&"library/waiting.mp3".to_owned()),
        "ADR 0107 item 6: with --hold-assets off the archive must hold the files that the server keeps, because they have no other copy"
    );
}

#[tokio::test]
async fn a_library_outside_the_data_folder_is_in_the_archive() {
    let (app, _) = app_assets(true, None);
    set_up_server(&app).await;
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("song.mp3"), b"audio").unwrap();
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    collection
        .set_setting("library_root", &json!(outside.path().to_str().unwrap()))
        .unwrap();
    drop(collection);
    // Reopen the app on the changed collection: the workspace reads the
    // setting at each request.
    let (_, _, _, bytes) = export_archive(&app, json!({})).await;
    assert!(
        tar_names(&bytes).contains(&"library/song.mp3".to_owned()),
        "ADR 0107 item 6: a library outside the data folder must go into library/ of the archive"
    );
}

/// Send the archive to the import routes in parts, and check it. Gives the
/// upload ID and the response of the check.
async fn import_check(app: &App, bytes: &[u8]) -> (String, StatusCode, Value) {
    let (status, started) = call(
        app,
        "POST",
        "/api/v1/import",
        Some(json!({ "size": bytes.len() })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let upload = started["upload"].as_str().unwrap().to_owned();
    let part = usize::try_from(started["part_size"].as_u64().unwrap()).unwrap();
    for (n, chunk) in bytes.chunks(part).enumerate() {
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/api/v1/import/{upload}/parts/{n}"))
            .header(header::HOST, HOST)
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::from(chunk.to_vec()))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, facts) = call(app, "POST", &format!("/api/v1/import/{upload}/check"), None).await;
    (upload, status, facts)
}

/// An app that paired, with a grant, in the collection of `app` (ADR 0110
/// item 41).
fn add_app_with_grant(app: &App) {
    let collection =
        studio_core::collection::Collection::open(&app.data.path().join("collection.sqlite"))
            .unwrap();
    let (connected, code) = studio_core::apps::start_pairing(&collection, "Show tool").unwrap();
    let signer = studio_app_client::MemorySigner::generate().unwrap();
    let key = studio_app_client::AppSigner::public_key(&signer).to_hex();
    studio_core::apps::pair(&collection, &code, &key, "wss://example.com").unwrap();
    let feed = collection.feeds().unwrap()[0].id;
    studio_core::grant::record(
        &collection,
        feed,
        "guid-1",
        connected.id,
        "wss://example.com",
        1,
    )
    .unwrap();
}

/// After an import, each app shows Connect Again and no grant stays
/// (ADR 0110 item 41).
async fn assert_apps_reset(target: &App, feeds: &Value) {
    let (_, apps_after) = call(target, "GET", "/api/v1/apps", None).await;
    assert_eq!(apps_after[0]["name"], "Show tool");
    assert_eq!(
        apps_after[0]["state"], "token",
        "ADR 0110 item 41: an imported app must show Connect Again. Call reset_apps_after_import in replace_account"
    );
    let feed_id = feeds[0]["id"].as_i64().unwrap();
    let (_, grants_after) = call(
        target,
        "GET",
        &format!("/api/v1/feeds/{feed_id}/grants"),
        None,
    )
    .await;
    assert!(
        grants_after[0]["ended_at"].is_i64(),
        "ADR 0110 item 41: an import must keep no grant: {grants_after}"
    );
}

#[tokio::test]
async fn an_export_imports_into_an_empty_desktop_and_opens_with_the_passphrase() {
    let (source, _) = app_assets(true, None);
    set_up_server(&source).await;
    std::fs::create_dir_all(source.data.path().join("library")).unwrap();
    std::fs::write(source.data.path().join("library/loose.mp3"), b"audio").unwrap();
    add_app_with_grant(&source);
    let (_, _, _, bytes) = export_archive(&source, json!({})).await;
    let vault_before = std::fs::read(source.data.path().join("vault.json")).unwrap();

    let target = app();
    // The desktop made a vault of its own before the import.
    make_vault(&target).await;
    let (upload, status, facts) = import_check(&target, &bytes).await;
    assert_eq!(status, StatusCode::OK, "{facts}");
    assert_eq!(facts["counts"]["feeds"], 1);
    assert_eq!(facts["methods"], json!(["passphrase"]));
    assert_eq!(facts["needs_email"], true);
    let open_path = format!("/api/v1/import/{upload}/open");
    let (status, body) = call(
        &target,
        "POST",
        &open_path,
        Some(json!({ "passphrase": "wrong wrong wrong wrong" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "wrong_key");
    let (status, body) = call(
        &target,
        "POST",
        &open_path,
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["vault"], "open");
    assert_eq!(body["feeds"], 1);
    let (_, feeds) = call(&target, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.as_array().unwrap().len(), 1, "{feeds}");
    assert_apps_reset(&target, &feeds).await;
    assert_eq!(
        std::fs::read(target.data.path().join("vault.json")).unwrap(),
        vault_before,
        "ADR 0107 item 13: the import must keep the vault as it is"
    );
    let collection =
        studio_core::collection::Collection::open(&target.data.path().join("collection.sqlite"))
            .unwrap();
    assert_eq!(
        collection.library_root().unwrap(),
        Some(target.data.path().join("library")),
        "ADR 0107 item 1: the library must be in the new data folder"
    );
    assert_eq!(
        std::fs::read(target.data.path().join("library/loose.mp3")).unwrap(),
        b"audio"
    );
    let (status, _) = call(&target, "GET", &format!("/api/v1/import/{upload}"), None).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "the import ends after the open"
    );
    assert!(
        !target
            .data
            .path()
            .join("import-staging")
            .read_dir()
            .is_ok_and(|mut d| d.next().is_some()),
        "ADR 0107 item 9: the import must leave no staging file"
    );
}

#[tokio::test]
async fn a_hostile_library_root_does_not_leave_the_data_folder_on_the_desktop() {
    let (source, _) = app_assets(true, None);
    set_up_server(&source).await;
    let (_, _, _, bytes) = export_archive(&source, json!({})).await;
    // Repack the archive with a collection that points outside.
    let work = tempfile::tempdir().unwrap();
    studio_core::account_archive::unpack_archive(&bytes[..], &work.path().join("a")).unwrap();
    let db = rusqlite::Connection::open(work.path().join("a/collection.sqlite")).unwrap();
    db.execute(
        "UPDATE settings SET value = ?1 WHERE key = 'library_root'",
        [json!("/etc").to_string()],
    )
    .unwrap();
    drop(db);
    let mut hostile = Vec::new();
    studio_core::account_archive::write_archive(
        &work.path().join("a"),
        &studio_core::account_archive::Manifest::new(None, Vec::new()),
        true,
        &mut hostile,
    )
    .unwrap();
    let target = app();
    let (upload, status, facts) = import_check(&target, &hostile).await;
    assert_eq!(status, StatusCode::OK, "{facts}");
    let (status, body) = call(
        &target,
        "POST",
        &format!("/api/v1/import/{upload}/open"),
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let collection =
        studio_core::collection::Collection::open(&target.data.path().join("collection.sqlite"))
            .unwrap();
    assert_eq!(
        collection.library_root().unwrap(),
        Some(target.data.path().join("library")),
        "ADR 0107 item 1: an absolute library_root in an archive must become the library of the data folder. Call reset_library_root after the unpack."
    );
}

#[tokio::test]
async fn a_desktop_import_into_a_collection_with_a_feed_gives_collection_not_empty() {
    let source = app();
    set_up(&source).await;
    let target = app();
    set_up(&target).await;
    for (method, path, body) in [
        ("POST", "/api/v1/import", Some(json!({ "size": 10 }))),
        ("POST", "/api/v1/import/x/check", None),
        (
            "POST",
            "/api/v1/import/x/open",
            Some(json!({ "passphrase": PASSPHRASE })),
        ),
    ] {
        let (status, body) = call(&target, method, path, body).await;
        assert_eq!(status, StatusCode::CONFLICT, "{path}: {body}");
        assert_eq!(
            body["error"]["code"], "collection_not_empty",
            "ADR 0107 item 9: {path} must refuse a collection with a feed. Call require_empty."
        );
    }
}

#[tokio::test]
async fn the_import_refuses_a_file_that_is_not_an_archive_and_the_server_has_no_desktop_import() {
    let target = app();
    let (upload, status, body) = import_check(&target, &vec![7_u8; 3000]).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        body["error"]["code"], "archive_invalid",
        "ADR 0107 item 11: a bad archive must give 422 archive_invalid"
    );
    let (status, _) = call(&target, "GET", &format!("/api/v1/import/{upload}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // An incomplete upload is not checked.
    let (_, started) = call(
        &target,
        "POST",
        "/api/v1/import",
        Some(json!({ "size": 10 })),
    )
    .await;
    let upload = started["upload"].as_str().unwrap();
    let (status, body) = call(
        &target,
        "POST",
        &format!("/api/v1/import/{upload}/check"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "upload_incomplete");
    let (status, _) = call(
        &target,
        "POST",
        &format!("/api/v1/import/{upload}/open"),
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "open needs a check first");
    // A server user imports at sign-up, not here.
    let (server, _) = app_assets(true, None);
    let (status, body) = call(
        &server,
        "POST",
        "/api/v1/import",
        Some(json!({ "size": 10 })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "not_on_server");
}

#[tokio::test]
async fn a_desktop_import_over_an_existing_vault_needs_a_recent_unlock() {
    let (source, _) = app_assets(true, None);
    set_up_server(&source).await;
    let (_, _, _, bytes) = export_archive(&source, json!({})).await;

    let (clock, offset) = test_clock();
    let (target, _handle) = app_with_clock(clock);
    make_vault(&target).await;
    let (upload, status, facts) = import_check(&target, &bytes).await;
    assert_eq!(status, StatusCode::OK, "{facts}");
    let open_path = format!("/api/v1/import/{upload}/open");
    let body = json!({ "passphrase": PASSPHRASE });
    offset.store(6 * 60, std::sync::atomic::Ordering::SeqCst);
    let (status, answer) = call(&target, "POST", &open_path, Some(body.clone())).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "ADR 0106 item 2 and ADR 0107 item 9: an import over an existing vault needs a recent unlock. Call require_recent_unlock in open_import: {answer}"
    );
    assert_eq!(answer["error"]["code"], "unlock_required");
    let (_, feeds) = call(&target, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.as_array().map(Vec::len), Some(0), "nothing changed");
    let (status, answer) = call(
        &target,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let (status, answer) = call(&target, "POST", &open_path, Some(body)).await;
    assert_eq!(status, StatusCode::OK, "a fresh unlock passes: {answer}");
}

#[tokio::test]
async fn the_queues_of_an_archive_do_not_come_in_on_the_desktop() {
    let (source, _) = app_assets(true, None);
    set_up_server(&source).await;
    let db = rusqlite::Connection::open(source.data.path().join("collection.sqlite")).unwrap();
    db.pragma_update(None, "foreign_keys", false).unwrap();
    db.execute(
        "INSERT INTO announcements (feed_id, url, podping, state, created_at, next_at)
         VALUES (1, 'https://example.org/a.xml', 1, 'waiting', 0, 4000000000)",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO downloads (library_file_id, state, added_at) VALUES (1, 'queued', 0)",
        [],
    )
    .unwrap();
    drop(db);
    let (_, _, _, bytes) = export_archive(&source, json!({})).await;
    let work = tempfile::tempdir().unwrap();
    studio_core::account_archive::unpack_archive(&bytes[..], &work.path().join("a")).unwrap();
    let count = |dir: &std::path::Path, sql: &str| {
        rusqlite::Connection::open(dir.join("collection.sqlite"))
            .unwrap()
            .query_row(sql, [], |r| r.get::<_, i64>(0))
            .unwrap()
    };
    let waiting = "SELECT (SELECT COUNT(*) FROM announcements WHERE state = 'waiting')
                   + (SELECT COUNT(*) FROM downloads WHERE state = 'queued')";
    assert_eq!(
        count(&work.path().join("a"), waiting),
        2,
        "the archive has both queues"
    );

    let target = app();
    let (upload, status, facts) = import_check(&target, &bytes).await;
    assert_eq!(status, StatusCode::OK, "{facts}");
    let (status, body) = call(
        &target,
        "POST",
        &format!("/api/v1/import/{upload}/open"),
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        count(target.data.path(), waiting),
        0,
        "ADR 0107 item 13: an import must not bring a waiting announcement or a queued download. Call clear_queues in the check."
    );
}

#[test]
fn the_default_host_folder_is_made_and_no_other_missing_folder_is_made() {
    let home = tempfile::tempdir().unwrap();
    let default = home.path().join("musicindex").join("pages");
    let other = home.path().join("typing-error");
    crate::routes::make_default_host_folder(other.to_str().unwrap(), Some(&default)).unwrap();
    assert!(!other.exists(), "a typing error must not make a folder");
    assert!(!default.exists(), "only the default folder is made");
    crate::routes::make_default_host_folder(default.to_str().unwrap(), Some(&default)).unwrap();
    assert!(default.is_dir());
    // A folder that is not the default gives no change, when no default exists.
    crate::routes::make_default_host_folder(other.to_str().unwrap(), None).unwrap();
    assert!(!other.exists());
}

#[tokio::test]
async fn a_missing_folder_that_is_not_the_default_gives_422_and_is_not_made() {
    let app = app();
    let missing = app.data.path().join("no-such-folder");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "x", "root": missing.to_str().unwrap() })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "folder_not_found");
    assert!(!missing.exists());
}

const DAY_SECS: u64 = 24 * 60 * 60;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A server app with a handle to its workspace, for a test of the sweep of
/// the temporary area.
fn server_app_with_handle() -> (App, crate::WorkspaceHandle) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_server_rules(crate::ServerRules {
            storage_limit: 1 << 30,
            folder: data.path().to_path_buf(),
            library_root: data.path().join("library"),
            hold_assets: true,
            vault_idle: std::time::Duration::from_secs(600),
            vault_idle_max: std::time::Duration::from_secs(3600),
        })
        .unwrap();
    let (router, handle) = crate::workspace_router(workspace);
    (
        App {
            router,
            data,
            feeds,
        },
        handle,
    )
}

/// Make the temporary area of a server. Gives its connector ID.
async fn make_temporary_area(app: &App) -> i64 {
    set_library_root(app);
    let (status, connector) = call(
        app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "temporary" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{connector}");
    connector["id"].as_i64().unwrap()
}

/// Write the draft of a feed with its own diff. Gives the response.
async fn write_with_own_diff(app: &App, id: i64) -> (StatusCode, Value) {
    let (_, diff) = call(app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await
}

#[tokio::test]
async fn a_server_makes_one_temporary_area_in_the_user_folder() {
    let app = app_with_rules(1 << 30);
    let id = make_temporary_area(&app).await;
    let folder = app.data.path().join("temporary");
    assert!(folder.is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&folder).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }
    let (_, rows) = call(&app, "GET", "/api/v1/connectors", None).await;
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap();
    assert_eq!(row["kind"], "temporary");
    let config: Value = serde_json::from_str(row["config"].as_str().unwrap()).unwrap();
    assert!(config.get("root").is_none(), "{config}");

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "temporary" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "temporary_exists");
}

#[tokio::test]
async fn the_temporary_area_is_refused_on_the_desktop() {
    let app = app();
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "temporary" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "not_on_desktop");
}

#[tokio::test]
async fn a_new_album_on_the_temporary_area_has_a_placeholder_and_writes_its_file() {
    let app = app_with_rules(1 << 30);
    let connector = make_temporary_area(&app).await;
    let feed = make_feed(
        &app,
        new_feed_body(connector, "album.xml", "Album", None, None),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let guid = draft_model(&app, id).await["podcast_guid"].clone();
    assert!(guid.as_str().unwrap().starts_with("pending-"), "{guid}");
    let (status, body) = write_with_own_diff(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(app.data.path().join("temporary/album.xml").is_file());
}

#[tokio::test]
async fn the_sweep_keeps_the_drafts_and_expires_the_area_after_7_days() {
    let (app, handle) = server_app_with_handle();
    let connector = make_temporary_area(&app).await;
    let feed = make_feed(
        &app,
        new_feed_body(connector, "album.xml", "Album", None, None),
    )
    .await;
    let id = feed["id"].as_i64().unwrap();
    let (status, body) = write_with_own_diff(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let file = app.data.path().join("temporary/album.xml");
    let now = now_secs();
    let set_age = |days: u64| {
        let time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(now - days * DAY_SECS);
        std::fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(time)
            .unwrap();
    };

    // Six days after the last write, nothing changes.
    set_age(6);
    handle
        .with_workspace(move |w| crate::temporary::sweep(w, now))
        .await
        .unwrap();
    assert!(file.is_file());

    // Seven days after the last write, the files go and the drafts stay.
    set_age(7);
    handle
        .with_workspace(move |w| crate::temporary::sweep(w, now))
        .await
        .unwrap();
    assert!(!file.exists());
    assert!(app.data.path().join("temporary").is_dir());
    let has_draft = handle
        .with_workspace(move |w| {
            w.collection
                .draft(studio_core::collection::FeedId(id))
                .unwrap()
                .is_some()
        })
        .await;
    assert!(has_draft);
    assert_eq!(listed(&app, id).await["temporary_expired"], true);

    // A write to the expired area gives 409.
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "temporary_expired");
}

#[tokio::test]
async fn a_feed_of_an_expired_area_moves_to_a_public_host_and_gets_its_real_guid() {
    let (app, handle) = server_app_with_handle();
    // The bucket serves the draft at its new address (ADR 0126 item 15).
    let new_url = "https://cdn.example.com/feeds/album.xml";
    let reader = studio_core::move_link::ScriptedReader::empty().with_feed(
        new_url,
        new_url,
        Some(&studio_core::guid::podcast_guid_for_url(new_url)),
        Some(new_url),
        Some(new_url),
    );
    handle
        .with_workspace(move |w| {
            w.link_reader = crate::workspace::LinkReader(std::sync::Arc::new(reader));
        })
        .await;
    let area = make_temporary_area(&app).await;
    let feed = make_feed(&app, new_feed_body(area, "album.xml", "Album", None, None)).await;
    let id = feed["id"].as_i64().unwrap();
    let (status, body) = write_with_own_diff(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let now = now_secs();
    let file = app.data.path().join("temporary/album.xml");
    let time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(now - 8 * DAY_SECS);
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(time)
        .unwrap();
    handle
        .with_workspace(move |w| crate::temporary::sweep(w, now))
        .await
        .unwrap();

    // A server has no local folder host (ADR 0055 item 24), so the file
    // host with the public address is a bucket.
    let bucket = studio_core::testing::fake_s3::FakeS3::start("bk");
    make_vault(&app).await;
    let (status, host) = call(&app, "POST", "/api/v1/connectors", Some(s3_body(&bucket))).await;
    assert_eq!(status, StatusCode::CREATED, "{host}");
    let (status, moved) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": host["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{moved}");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": moved["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, draft) = call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(
        draft["model"]["podcast_guid"],
        studio_core::guid::podcast_guid_for_url("https://cdn.example.com/feeds/album.xml")
    );
}

// ---- Stage 1 of ADR 0126: a bad move link ----

/// An app whose reader of public feeds serves `reader`, with a connector
/// for the feeds folder. Gives the app and the connector ID. No request
/// leaves the computer (ADR 0045).
async fn app_with_link_reader(reader: studio_core::move_link::ScriptedReader) -> (App, i64) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_feed_reader(std::sync::Arc::new(reader));
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
    let id = connector["id"].as_i64().unwrap();
    (app, id)
}

const TAG_GONE: &str = "<itunes:new-feed-url xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">https://other.example/f.xml</itunes:new-feed-url>";

/// Write the draft of a feed through the routes. Gives the status and the
/// body of the write.
async fn write_through_routes(app: &App, id: i64) -> (StatusCode, Value) {
    let (status, diff) = call(app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await
}

#[tokio::test]
async fn a_refused_write_gives_422_bad_move_link_with_each_hop_and_changes_no_file() {
    // The feed names https://other.example/f.xml. That feed names the
    // address of this feed again: a loop of two URLs (scenario S3).
    let reader = studio_core::move_link::ScriptedReader::empty().with_feed(
        "https://other.example/f.xml",
        "https://other.example/f.xml",
        Some("ga"),
        Some("https://music.example/a.xml"),
        Some("https://other.example/f.xml"),
    );
    let (app, connector) = app_with_link_reader(reader).await;
    let xml = paid_feed("A", "ga", &[ANN, BOB], &[], TAG_GONE);
    let id = add_paid_feed(&app, connector, "a.xml", &xml).await;
    let (status, _) = call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = write_through_routes(&app, id).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "bad_move_link");
    let details = &body["error"]["details"];
    assert_eq!(details["condition"], "loop");
    assert_eq!(details["number"], 1);
    assert_eq!(details["stage"], 1);
    assert_eq!(details["repairs"], json!(["remove_tag"]));
    assert_eq!(
        details["hops"],
        json!([
            { "url": "https://other.example/f.xml", "result": "feed" },
            { "url": "https://music.example/a.xml", "result": "revisited" },
        ])
    );
    // No file changed on the host, and the draft stays.
    assert_eq!(
        std::fs::read_to_string(app.feeds.path().join("a.xml")).unwrap(),
        xml
    );
    assert!(
        collection_of(&app)
            .draft(studio_core::collection::FeedId(id))
            .unwrap()
            .is_some()
    );
}

/// A reader that records whether the lock of the workspace was free during
/// a read, and then serves the scripted bodies.
struct LockProbe {
    shared: std::sync::Arc<std::sync::Mutex<Workspace>>,
    free: std::sync::Arc<std::sync::atomic::AtomicBool>,
    inner: studio_core::move_link::ScriptedReader,
}

impl studio_core::move_link::FeedReader for LockProbe {
    fn read(
        &self,
        url: &str,
    ) -> Result<studio_core::move_link::FeedBody, studio_core::discover::ReadFailure> {
        // Try the lock for up to two seconds. A held lock fails the test.
        let limit = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let free = loop {
            if self.shared.try_lock().is_ok() {
                break true;
            }
            if std::time::Instant::now() > limit {
                break false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        self.free.store(free, std::sync::atomic::Ordering::SeqCst);
        self.inner.read(url)
    }

    fn wait_between_reads(&self) {
        self.inner.wait_between_reads();
    }
}

/// An app with a connector for the feeds folder, whose reader of public
/// feeds is a [`LockProbe`] over `reader`. Gives the app, the connector ID
/// and the flag of the probe.
async fn app_with_lock_probe(
    reader: studio_core::move_link::ScriptedReader,
) -> (App, i64, std::sync::Arc<std::sync::atomic::AtomicBool>) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll));
    let (router, handle) = crate::workspace_router(workspace);
    let free = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let probe = LockProbe {
        shared: handle.shared_for_test(),
        free: std::sync::Arc::clone(&free),
        inner: reader,
    };
    handle
        .with_workspace(move |w| {
            w.link_reader = crate::workspace::LinkReader(std::sync::Arc::new(probe));
        })
        .await;
    let app = App {
        router,
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
    let id = connector["id"].as_i64().unwrap();
    (app, id, free)
}

const OLD_SELF: &str = "<atom:link xmlns:atom=\"http://www.w3.org/2005/Atom\" rel=\"self\" href=\"https://old.example/b.xml\"/>";

/// Give a feed a public address that differs from its self link, as a feed
/// behind a redirect has (ADR 0021 item 1).
fn set_public_url(app: &App, id: i64, url: &str) {
    rusqlite::Connection::open(app.data.path().join("collection.sqlite"))
        .unwrap()
        .execute("UPDATE feeds SET public_url = ?2 WHERE id = ?1", (id, url))
        .unwrap();
}

fn draft_bytes(app: &App, id: i64) -> String {
    let draft = collection_of(app)
        .draft(studio_core::collection::FeedId(id))
        .unwrap()
        .unwrap();
    String::from_utf8(draft.draft_xml).unwrap()
}

#[tokio::test]
async fn the_link_check_route_gives_the_refusal_the_notice_and_the_repairs_with_no_lock() {
    let reader = studio_core::move_link::ScriptedReader::empty().with_feed(
        "https://other.example/f.xml",
        "https://other.example/f.xml",
        Some("ga"),
        Some("https://music.example/a.xml"),
        Some("https://other.example/f.xml"),
    );
    let (app, connector, free) = app_with_lock_probe(reader).await;
    let a_xml = paid_feed("A", "ga", &[ANN, BOB], &[], TAG_GONE);
    let a = add_paid_feed(&app, connector, "a.xml", &a_xml).await;
    let b_xml = paid_feed("B", "gb", &[ANN, BOB], &[], OLD_SELF);
    let b = add_paid_feed(&app, connector, "b.xml", &b_xml).await;
    set_public_url(&app, b, "https://music.example/b.xml");
    call(&app, "GET", &format!("/api/v1/feeds/{a}/draft"), None).await;
    call(&app, "GET", &format!("/api/v1/feeds/{b}/draft"), None).await;

    // A refusal, with its repair. The reader found the lock free.
    let (status, report) = call(&app, "GET", &format!("/api/v1/feeds/{a}/link-check"), None).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["refusal"]["condition"], "loop");
    assert_eq!(report["refusal"]["stage"], 1);
    assert_eq!(report["refusal"]["repairs"], json!(["remove_tag"]));
    assert_eq!(report["notices"], json!([]));
    assert_eq!(report["repairs"], json!(["remove_tag"]));
    assert!(
        free.load(std::sync::atomic::Ordering::SeqCst),
        "the read ran while the workspace was locked"
    );

    // A notice, with its repair, and no read.
    let (status, report) = call(&app, "GET", &format!("/api/v1/feeds/{b}/link-check"), None).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["refusal"], Value::Null);
    assert_eq!(
        report["notices"],
        json!([{
            "notice": "self_link_differs",
            "self_url": "https://old.example/b.xml",
            "draft_url": "https://music.example/b.xml",
        }])
    );
    assert_eq!(report["repairs"], json!(["use_public_address"]));
}

#[tokio::test]
async fn each_repair_route_changes_only_its_one_value_and_refuses_when_it_does_not_apply() {
    let reader = studio_core::move_link::ScriptedReader::empty();
    let (app, connector) = app_with_link_reader(reader).await;
    let a_xml = paid_feed("A", "ga", &[ANN, BOB], &[], TAG_GONE);
    let a = add_paid_feed(&app, connector, "a.xml", &a_xml).await;
    let b_xml = paid_feed("B", "gb", &[ANN, BOB], &[], OLD_SELF);
    let b = add_paid_feed(&app, connector, "b.xml", &b_xml).await;
    set_public_url(&app, b, "https://music.example/b.xml");
    let (_, draft_a) = call(&app, "GET", &format!("/api/v1/feeds/{a}/draft"), None).await;
    let (_, draft_b) = call(&app, "GET", &format!("/api/v1/feeds/{b}/draft"), None).await;

    // A repair that does not apply gives 422 and changes nothing.
    let before_a = draft_bytes(&app, a);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{a}/draft/use-public-address"),
        Some(json!({ "revision": draft_a["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    // The core kind InvalidOperation has the code of the change sets.
    assert_eq!(body["error"]["code"], "invalid_change_set");
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{b}/draft/remove-tag"),
        Some(json!({ "revision": draft_b["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(draft_bytes(&app, a), before_a);

    // A stale revision gives 409.
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{a}/draft/remove-tag"),
        Some(json!({ "revision": "stale" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"]["code"], "stale_revision");
    assert_eq!(draft_bytes(&app, a), before_a);

    // "Remove the Tag" removes the tag and no other byte.
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{a}/draft/remove-tag"),
        Some(json!({ "revision": draft_a["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["model"]["new_feed_url"], Value::Null);
    assert_eq!(draft_bytes(&app, a), before_a.replace(TAG_GONE, ""));

    // "Use the Public Address" changes the self link and no other byte.
    let before_b = draft_bytes(&app, b);
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{b}/draft/use-public-address"),
        Some(json!({ "revision": draft_b["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        draft_bytes(&app, b),
        before_b.replace("https://old.example/b.xml", "https://music.example/b.xml")
    );
    // The repair applied once, so it does not apply again.
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{b}/draft/use-public-address"),
        Some(json!({ "revision": body["revision"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn a_stage_two_refusal_gives_the_stage_and_keeps_the_feed_at_its_address() {
    let server = web_feeds();
    let new_url = "https://new.example/album/a.xml";
    // The destination serves a feed with a different podcast:guid.
    let reader = studio_core::move_link::ScriptedReader::empty().with_feed(
        new_url,
        new_url,
        Some("other-guid"),
        None,
        None,
    );
    let app = app_with_reader(&server.url(""), None, Some(reader));
    set_library_root(&app);
    let id = read_only_with_folder(&app, &server.url("/a.xml"));
    let new_dir = tempfile::tempdir().unwrap();
    let new_id = add_host(&app, "New", new_dir.path(), "https://new.example").await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    let (_, diff) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": new_id })),
    )
    .await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "bad_move_link");
    let details = &body["error"]["details"];
    assert_eq!(details["stage"], 2);
    assert_eq!(details["condition"], "guid_mismatch");
    assert_eq!(details["hops"][0]["url"], new_url);
    assert_eq!(details["hops"][0]["result"], "guid_mismatch");
    assert_eq!(details["repairs"], json!([]));
    // The feed keeps its host, and the draft keeps its move.
    let (_, feed) = call(&app, "GET", &format!("/api/v1/feeds/{id}"), None).await;
    assert_eq!(feed["connector_id"], Value::Null);
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["feed_move"]["connector_id"], new_id);
}

#[tokio::test]
async fn a_change_set_writes_the_other_feeds_when_one_feed_is_refused() {
    let (app, connector) =
        app_with_link_reader(studio_core::move_link::ScriptedReader::empty()).await;
    // Feed A names a URL that nothing serves. Feed B has no tag.
    let a_xml = paid_feed("A", "ga", &[ANN, BOB], &[], TAG_GONE);
    let a = add_paid_feed(&app, connector, "a.xml", &a_xml).await;
    let b_xml = paid_feed("B", "gb", &[ANN, BOB], &[], "");
    let b = add_paid_feed(&app, connector, "b.xml", &b_xml).await;
    let (status, set) = call(
        &app,
        "POST",
        "/api/v1/change-sets",
        Some(json!({
            "recipient": ann_key(),
            "operation": { "kind": "rename", "name": "Anna" },
            "feed_ids": [a, b],
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    let id = set["id"].as_i64().unwrap();

    let (status, body) = write_through_routes(&app, a).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "bad_move_link");
    assert_eq!(body["error"]["details"]["condition"], "read_fails");
    assert_eq!(
        body["error"]["details"]["hops"],
        json!([{ "url": "https://other.example/f.xml", "result": "unreachable" }])
    );
    assert_eq!(
        std::fs::read_to_string(app.feeds.path().join("a.xml")).unwrap(),
        a_xml
    );

    // The refusal of feed A does not stop feed B.
    let (status, body) = write_through_routes(&app, b).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        std::fs::read_to_string(app.feeds.path().join("b.xml"))
            .unwrap()
            .contains("Anna")
    );
    let (_, set) = call(&app, "GET", &format!("/api/v1/change-sets/{id}"), None).await;
    assert_eq!(change_row(&set, a)["written_at"], Value::Null, "{set}");
    assert!(change_row(&set, b)["written_at"].is_i64(), "{set}");
    // Feed A keeps its draft.
    assert!(
        collection_of(&app)
            .draft(studio_core::collection::FeedId(a))
            .unwrap()
            .is_some()
    );
}

// ---- The build date (ADR 0127 item 13, task 092b) ----

/// 2026-10-30 20:00:00 UTC.
const BUILD_T0: i64 = 1_793_390_400;

/// A source of server time that answers `answer` for each URL. It records
/// each URL, and it records whether the lock of the workspace was free
/// during the request.
struct TimeProbe {
    shared: std::sync::Arc<std::sync::Mutex<Workspace>>,
    free: std::sync::Arc<std::sync::atomic::AtomicBool>,
    seen: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    answer: Option<i64>,
}

impl studio_core::checked_time::ServerTime for TimeProbe {
    fn server_time(&self, url: &str) -> Option<i64> {
        let limit = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let free = loop {
            if self.shared.try_lock().is_ok() {
                break true;
            }
            if std::time::Instant::now() > limit {
                break false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        if !free {
            self.free.store(false, std::sync::atomic::Ordering::SeqCst);
        }
        self.seen.lock().unwrap().push(url.to_owned());
        self.answer
    }
}

/// What the test reads from a [`TimeProbe`].
struct TimeLog {
    free: std::sync::Arc<std::sync::atomic::AtomicBool>,
    seen: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl TimeLog {
    fn requests(&self) -> Vec<String> {
        self.seen.lock().unwrap().clone()
    }

    fn lock_was_free(&self) -> bool {
        self.free.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// An app with a connector for the feeds folder. The computer clock is
/// `BUILD_T0`, and the source of server time is a [`TimeProbe`] with
/// `answer`.
async fn app_with_time_probe(answer: Option<i64>) -> (App, i64, TimeLog) {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll))
        .with_wall_clock(crate::WallClock::new(|| {
            u64::try_from(BUILD_T0).unwrap_or(0)
        }));
    let (router, handle) = crate::workspace_router(workspace);
    let log = TimeLog {
        free: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
        seen: std::sync::Arc::default(),
    };
    let probe = TimeProbe {
        shared: handle.shared_for_test(),
        free: std::sync::Arc::clone(&log.free),
        seen: std::sync::Arc::clone(&log.seen),
        answer,
    };
    handle
        .with_workspace(move |w| {
            w.server_time = crate::workspace::ServerTimeSource(std::sync::Arc::new(probe));
        })
        .await;
    let app = App {
        router,
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
    let id = connector["id"].as_i64().unwrap();
    (app, id, log)
}

/// Change the title of the draft of a feed through the route.
async fn rename_through_route(app: &App, id: i64, title: &str) {
    let path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(app, "GET", &path, None).await;
    let mut model = draft["model"].clone();
    model["title"] = json!(title);
    let (status, body) = call(
        app,
        "PUT",
        &path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_time_requests_run_with_the_lock_free_and_a_second_diff_sends_no_new_request() {
    let (app, connector, log) = app_with_time_probe(Some(BUILD_T0)).await;
    let xml = paid_feed("A", "ga", &[ANN, BOB], &[], "");
    let id = add_paid_feed(&app, connector, "a.xml", &xml).await;
    let path = format!("/api/v1/feeds/{id}");

    // A draft with no change needs no time, and a save sends no request.
    let (_, draft) = call(&app, "GET", &format!("{path}/draft"), None).await;
    let (status, diff) = call(&app, "GET", &format!("{path}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["changed"], false);
    assert_eq!(diff["build_date"]["set"], false);
    assert_eq!(diff["build_date"]["no_server"], false);
    rename_through_route(&app, id, "A (Deluxe)").await;
    assert_eq!(log.requests(), Vec::<String>::new());
    assert!(!draft_bytes(&app, id).contains("lastBuildDate"));
    let revision = call(&app, "GET", &format!("{path}/draft"), None).await.1["revision"].clone();
    assert_ne!(revision, draft["revision"]);

    // The diff of the change asks the musicindex API and the host of the
    // feed, and the lock is free during the requests.
    let (status, first) = call(&app, "GET", &format!("{path}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(
        log.requests(),
        ["http://127.0.0.1:9", "https://music.example/a.xml"]
    );
    assert!(
        log.lock_was_free(),
        "ADR 0127 item 13: the time request ran while the workspace was locked. Ask the servers outside the lock, in with_checked_time."
    );
    assert_eq!(first["build_date"]["set"], true);
    assert_eq!(first["build_date"]["no_server"], false);
    assert_eq!(first["build_date"]["later_base"], Value::Null);
    assert_eq!(first["build_date"]["clock_ahead_seconds"], Value::Null);
    assert!(draft_bytes(&app, id).contains("<lastBuildDate>Fri, 30 Oct 2026 20:00:00 +0000<"));

    // A second diff in 10 minutes sends no new request and has the same ID.
    let (_, second) = call(&app, "GET", &format!("{path}/diff"), None).await;
    assert_eq!(second["id"], first["id"]);
    assert_eq!(log.requests().len(), 2);

    // The date does not make the revision of the editor stale.
    let (_, current) = call(&app, "GET", &format!("{path}/draft"), None).await;
    assert_eq!(current["revision"], revision);
    rename_through_route(&app, id, "A (Remastered)").await;
}

#[tokio::test]
async fn a_server_that_does_not_answer_gives_the_state_no_server_and_the_write_completes() {
    let (app, connector, log) = app_with_time_probe(None).await;
    let xml = paid_feed("A", "ga", &[ANN, BOB], &[], "");
    let id = add_paid_feed(&app, connector, "a.xml", &xml).await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    rename_through_route(&app, id, "A (Deluxe)").await;
    let (status, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["build_date"]["no_server"], true);
    assert_eq!(diff["build_date"]["set"], false);
    assert_eq!(log.requests().len(), 2);
    let (status, body) = write_through_routes(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let written = std::fs::read_to_string(app.feeds.path().join("a.xml")).unwrap();
    assert!(written.contains("A (Deluxe)"));
    assert!(!written.contains("lastBuildDate"));
}

#[tokio::test]
async fn a_clock_ahead_of_the_servers_gives_the_server_time_and_the_notice_in_the_diff() {
    // The computer clock is BUILD_T0, and both servers say two years earlier.
    let two_years = 2 * 365 * 86_400;
    let (app, connector, _) = app_with_time_probe(Some(BUILD_T0 - two_years)).await;
    let xml = paid_feed("A", "ga", &[ANN, BOB], &[], "");
    let id = add_paid_feed(&app, connector, "a.xml", &xml).await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    rename_through_route(&app, id, "A (Deluxe)").await;
    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["build_date"]["set"], true);
    assert_eq!(diff["build_date"]["clock_ahead_seconds"], two_years);
    assert!(draft_bytes(&app, id).contains("<lastBuildDate>Wed, 30 Oct 2024 20:00:00 +0000<"));
}

#[tokio::test]
async fn the_file_operation_route_gives_the_build_date_in_its_diff() {
    let (app, connector, log) = app_with_time_probe(Some(BUILD_T0)).await;
    let xml = paid_feed("A", "ga", &[ANN, BOB], &[], "");
    let id = add_paid_feed(&app, connector, "a.xml", &xml).await;
    call(&app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    rename_through_route(&app, id, "A (Deluxe)").await;
    // A route that returns a diff after a change gives the date.
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "delete_unused", "urls": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["build_date"]["set"], true);
    assert!(log.lock_was_free());
}

/// An app whose reader gives the feed at the two addresses of a move.
fn move_app() -> App {
    move_app_with_index("http://127.0.0.1:9")
}

fn move_app_with_index(index: &str) -> App {
    let mut reader = studio_core::move_link::ScriptedReader::empty();
    for url in [
        "https://backup.example/album.xml",
        "https://music.example/album.xml",
    ] {
        reader = reader.with_feed(url, url, Some("g-1"), Some(url), None);
    }
    app_with_reader(index, None, Some(reader))
}

/// An album with one written track on the file host "Home", and a second
/// file host "Backup". Gives the feed, the library file, the two file hosts
/// and the folder of "Backup".
async fn album_on_home(app: &App) -> (i64, i64, i64, i64, tempfile::TempDir) {
    let id = set_up(app).await;
    let bytes = std::fs::read(media_path("tagged.mp3")).unwrap();
    let (_, file) = upload(app, id, "tagged.mp3", bytes).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["items"][0]["enclosure"] = json!({
        "url": file["url"],
        "length": file["size"].to_string(),
        "media_type": "audio/mpeg",
    });
    let (status, _) = call(
        app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    write_the_draft(app, id).await;
    let (_, connectors) = call(app, "GET", "/api/v1/connectors", None).await;
    let home = connectors[0]["id"].as_i64().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let backup = add_host(app, "Backup", folder.path(), "https://backup.example").await;
    (
        id,
        file["library_file_id"].as_i64().unwrap(),
        home,
        backup,
        folder,
    )
}

async fn write_the_draft(app: &App, id: i64) {
    let (_, diff) = call(app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    let (status, body) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // The route starts a draft when the feed has none after the write.
    call(app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
}

async fn move_release(app: &App, id: i64, to: i64, delete_later: bool, wait: u32) -> Value {
    let (status, diff) = call(
        app,
        "POST",
        &format!("/api/v1/feeds/{id}/move"),
        Some(json!({ "connector_id": to, "delete_later": delete_later, "wait_days": wait })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    diff
}

async fn waiting(app: &App, id: i64) -> Vec<Value> {
    let (status, body) = call(app, "GET", &format!("/api/v1/feeds/{id}/moves"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["moves"].as_array().unwrap().clone()
}

/// ADR 0148 items 4 and 7: a write of a move sends the new copy, deletes no
/// old copy and records it with the waiting time of the request.
#[tokio::test]
async fn a_written_move_keeps_the_old_file_and_records_it() {
    let app = move_app();
    let (id, _, home, backup, folder) = album_on_home(&app).await;
    let diff = move_release(&app, id, backup, true, 7).await;
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        operations.iter().any(|op| op["kind"] == "hold"),
        "{operations:?}"
    );
    assert!(
        !operations.iter().any(|op| op["kind"] == "delete"),
        "{operations:?}"
    );
    assert!(
        waiting(&app, id).await.is_empty(),
        "no record before the write"
    );

    write_the_draft(&app, id).await;
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert!(folder.path().join("album/tagged.mp3").exists());
    let moves = waiting(&app, id).await;
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0]["connector_id"], home);
    assert_eq!(moves[0]["to_connector_id"], backup);
    assert_eq!(moves[0]["wait_days"], 7);
    assert_eq!(moves[0]["records"].as_array().unwrap().len(), 1);
    assert_eq!(moves[0]["ready"], false);
    assert_eq!(moves[0]["index"], "unknown");
}

/// ADR 0148 item 7: **Move to** on one file has 30 days.
#[tokio::test]
async fn move_to_on_one_file_waits_for_30_days() {
    let app = move_app();
    let (id, file, home, backup, _folder) = album_on_home(&app).await;
    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/operations"),
        Some(json!({ "op": "move", "file": file, "from": home, "to": backup })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    write_the_draft(&app, id).await;
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert_eq!(waiting(&app, id).await[0]["wait_days"], 30);
}

/// ADR 0148 item 5: with the check box off, Studio keeps no record.
#[tokio::test]
async fn a_move_with_the_box_off_keeps_no_record() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    let diff = move_release(&app, id, backup, false, 30).await;
    let operations = diff["operations"].as_array().unwrap();
    assert!(
        !operations.iter().any(|op| op["kind"] == "hold"),
        "{operations:?}"
    );
    write_the_draft(&app, id).await;
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert_eq!(waiting(&app, id).await, Vec::<Value>::new());
}

/// ADR 0148 item 6: a waiting time must be one of the choices.
#[tokio::test]
async fn a_waiting_time_that_is_not_a_choice_is_refused() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/move"),
        Some(json!({ "connector_id": backup, "delete_later": true, "wait_days": 3 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

/// ADR 0148 items 10 and 12: the route adds one deletion for each record.
/// Only the written diff removes the record and the file.
#[tokio::test]
async fn only_a_written_diff_deletes_an_old_copy() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    move_release(&app, id, backup, true, 30).await;
    write_the_draft(&app, id).await;
    let moves = waiting(&app, id).await;
    let records = moves[0]["records"].clone();

    let (status, diff) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/moves/delete"),
        Some(json!({ "records": records })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    let deletions: Vec<_> = diff["operations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|op| op["kind"] == "delete")
        .collect();
    assert_eq!(deletions.len(), 1, "{diff}");
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert_eq!(waiting(&app, id).await.len(), 1);

    write_the_draft(&app, id).await;
    assert!(!app.feeds.path().join("album/tagged.mp3").exists());
    assert_eq!(waiting(&app, id).await, Vec::<Value>::new());
}

/// ADR 0148 item 11: "Keep the Files" removes the record, and no file changes.
#[tokio::test]
async fn keep_the_files_removes_only_the_record() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    move_release(&app, id, backup, true, 30).await;
    write_the_draft(&app, id).await;
    let records = waiting(&app, id).await[0]["records"].clone();
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/feeds/{id}/moves/keep"),
        Some(json!({ "records": records })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(waiting(&app, id).await, Vec::<Value>::new());
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
}

/// ADR 0148 item 13: a later write of a file to the path of a record removes
/// that record.
#[tokio::test]
async fn a_write_to_the_path_of_a_record_removes_the_record() {
    let app = move_app();
    let (id, _, home, backup, _folder) = album_on_home(&app).await;
    move_release(&app, id, backup, true, 30).await;
    write_the_draft(&app, id).await;
    assert_eq!(waiting(&app, id).await[0]["connector_id"], home);

    move_release(&app, id, home, true, 30).await;
    write_the_draft(&app, id).await;
    let moves = waiting(&app, id).await;
    assert_eq!(moves.len(), 1, "{moves:?}");
    assert_eq!(moves[0]["connector_id"], backup);
}

/// ADR 0148 items 2 and 3: the plan gives the feed file and the files that
/// move, and it names the file that is not in the library.
#[tokio::test]
async fn the_plan_names_what_moves_and_what_stays() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    let draft_path = format!("/api/v1/feeds/{id}/draft");
    let (_, draft) = call(&app, "GET", &draft_path, None).await;
    let mut model = draft["model"].clone();
    model["image"] = json!("https://other.example/cover.jpg");
    let (status, _) = call(
        &app,
        "PUT",
        &draft_path,
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let plan_path = format!("/api/v1/feeds/{id}/move/plan?connector_id={backup}");
    let (status, plan) = call(&app, "GET", &plan_path, None).await;
    assert_eq!(status, StatusCode::OK, "{plan}");
    assert_eq!(plan["feed"]["moves"], true);
    assert_eq!(plan["audio"]["count"], 1);
    assert_eq!(plan["on_target"], 0);
    assert_eq!(plan["nothing_to_move"], false);
    assert!(
        plan["stays"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "https://other.example/cover.jpg"
                && s["reason"] == "not_in_library"),
        "{plan}"
    );

    // After the move, each file is on the target.
    move_release(&app, id, backup, true, 30).await;
    let (_, plan) = call(&app, "GET", &plan_path, None).await;
    assert_eq!(plan["audio"]["count"], 0, "{plan}");
    assert_eq!(plan["on_target"], 1, "{plan}");
    assert_eq!(plan["nothing_to_move"], true, "{plan}");
}

/// ADR 0148 item 15: the files route counts the files that stay behind a
/// draft feed move.
#[tokio::test]
async fn the_files_route_counts_the_files_that_stay_behind_a_feed_move() {
    let app = move_app();
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    let files_path = format!("/api/v1/feeds/{id}/files");
    let (_, files) = call(&app, "GET", &files_path, None).await;
    assert_eq!(files["files_stay"], 0);
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": backup })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, files) = call(&app, "GET", &files_path, None).await;
    assert_eq!(files["files_stay"], 1);
    move_release(&app, id, backup, true, 30).await;
    let (_, files) = call(&app, "GET", &files_path, None).await;
    assert_eq!(files["files_stay"], 0);
}

/// ADR 0148 item 9: the route gives the answer of musicindex about the
/// address of the feed.
#[tokio::test]
async fn a_waiting_move_gives_the_answer_of_musicindex() {
    let server = studio_core::testing::TestServer::start(Vec::new());
    let answer = |url: &str| {
        json!({ "data": { "feed_url": url, "title": "Album" } })
            .to_string()
            .into_bytes()
    };
    server.set(
        "/v1/feeds/-1",
        200,
        answer("https://music.example/album.xml"),
    );
    let app = move_app_with_index(&server.url(""));
    let (id, _, _, backup, _folder) = album_on_home(&app).await;
    move_release(&app, id, backup, true, 7).await;
    write_the_draft(&app, id).await;
    let moves = waiting(&app, id).await;
    assert_eq!(moves[0]["index"], "different", "{moves:?}");
    server.set(
        "/v1/feeds/-1",
        200,
        answer("https://backup.example/album.xml"),
    );
    assert_eq!(waiting(&app, id).await[0]["index"], "follows");
}

/// A second album on "Home", with one more file host "Spare".
async fn second_feed_on_home(app: &App, home: i64) -> i64 {
    std::fs::write(
        app.feeds.path().join("other.xml"),
        FEED.replace("g-1", "g-2"),
    )
    .unwrap();
    let (status, feed) = call(
        app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": home, "path": "other.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    feed["id"].as_i64().unwrap()
}

fn move_all_app() -> App {
    let mut reader = studio_core::move_link::ScriptedReader::empty();
    for (url, guid) in [
        ("https://backup.example/album.xml", "g-1"),
        ("https://backup.example/other.xml", "g-2"),
    ] {
        reader = reader.with_feed(url, url, Some(guid), Some(url), None);
    }
    app_with_reader("http://127.0.0.1:9", None, Some(reader))
}

async fn move_all(app: &App, from: i64, to: i64) -> Value {
    let (status, set) = call(
        app,
        "POST",
        "/api/v1/change-sets/move",
        Some(json!({ "from": from, "to": to, "delete_later": true, "wait_days": 7 })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{set}");
    set
}

/// ADR 0148 items 16 and 17: a change set has one row for each release on
/// the file host. Each feed writes with its own diff, and the old copies stay.
#[tokio::test]
async fn move_all_releases_gives_a_change_set_with_a_row_for_each_release() {
    let app = move_all_app();
    let (id, _, home, backup, folder) = album_on_home(&app).await;
    let other = second_feed_on_home(&app, home).await;
    let set = move_all(&app, home, backup).await;
    assert_eq!(set["operation"]["kind"], "move_releases");
    assert_eq!(set["operation"]["to_name"], "Backup");
    let feeds = set["feeds"].as_array().unwrap();
    assert_eq!(feeds.len(), 2, "{feeds:?}");
    let row = |feed: i64| feeds.iter().find(|f| f["feed_id"] == feed).unwrap();
    assert_eq!(row(id)["outcome"]["outcome"], "changed");
    assert_eq!(row(id)["outcome"]["channel"], true);
    assert_eq!(row(id)["outcome"]["items"], 1);
    assert_eq!(row(other)["outcome"]["items"], 0);

    write_the_draft(&app, id).await;
    write_the_draft(&app, other).await;
    assert!(app.feeds.path().join("album/tagged.mp3").exists());
    assert!(folder.path().join("album/tagged.mp3").exists());
    let moves = waiting(&app, id).await;
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0]["wait_days"], 7);
}

/// ADR 0148 item 18: Discard removes the host operations of the move and
/// puts back the draft of each feed from before.
#[tokio::test]
async fn discard_of_a_move_change_set_removes_its_host_operations() {
    let app = move_all_app();
    let (id, _, home, backup, _folder) = album_on_home(&app).await;
    let other = second_feed_on_home(&app, home).await;
    let set = move_all(&app, home, backup).await;
    let set_id = set["id"].as_i64().unwrap();
    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/v1/change-sets/{set_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, diff) = call(&app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_eq!(diff["operations"], json!([]), "{diff}");
    assert_eq!(diff["feed_move"], Value::Null, "{diff}");
    let (status, _) = call(&app, "GET", &format!("/api/v1/feeds/{other}/diff"), None).await;
    assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let (_, files) = call(&app, "GET", &format!("/api/v1/feeds/{other}/files"), None).await;
    assert_eq!(files["feed_move"], Value::Null, "{files}");
}

/// ADR 0148 item 19: a draft that already moves the feed file stays out, and
/// its row gives the cause.
#[tokio::test]
async fn a_draft_with_a_feed_move_stays_out_of_a_move_of_all_releases() {
    let app = move_all_app();
    let (id, _, home, backup, folder) = album_on_home(&app).await;
    let spare = add_host(&app, "Spare", folder.path(), "https://spare.example").await;
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/v1/feeds/{id}/feed-host"),
        Some(json!({ "connector_id": spare })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let set = move_all(&app, home, backup).await;
    let row = &set["feeds"][0];
    assert_eq!(row["outcome"]["outcome"], "not_changed", "{set}");
    assert_eq!(row["outcome"]["cause"]["cause"], "has_feed_move", "{set}");
}

/// ADR 0148 item 16: a file host with no release gives no change set.
#[tokio::test]
async fn move_all_releases_needs_a_release_on_the_file_host() {
    let app = move_all_app();
    let (_, _, home, backup, _folder) = album_on_home(&app).await;
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/change-sets/move",
        Some(json!({ "from": backup, "to": home, "delete_later": true, "wait_days": 7 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

/// An app with a local file host at the address of a test server, and a feed
/// on it. The test server gives the public address and the musicindex answer.
async fn outside_app() -> (App, studio_core::testing::TestServer, i64) {
    let server = studio_core::testing::TestServer::start(vec![]);
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    server.set("/album.xml", 200, FEED.as_bytes().to_vec());
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(&server.url("/index"))
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_keysend(None)
        .with_address_check(std::sync::Arc::new(PassAll));
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    };
    let root = app.feeds.path().to_str().unwrap().to_owned();
    let (_, connector) = call(
        &app,
        "POST",
        "/api/v1/connectors",
        Some(json!({ "kind": "local", "name": "Home", "root": root, "base_url": server.url("") })),
    )
    .await;
    let (status, feed) = call(
        &app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    let id = feed["id"].as_i64().unwrap();
    let (status, body) = call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    (app, server, id)
}

#[tokio::test]
async fn the_outside_check_finds_a_change_and_a_move_and_keeps_a_result_after_a_failed_read() {
    let (app, server, id) = outside_app().await;
    let path = format!("/api/v1/feeds/{id}/outside");
    let (status, same) = call(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{same}");
    assert_eq!(same["enabled"], json!(true));
    assert_eq!(same["state"], Value::Null, "ADR 0150 item 2");

    let changed = FEED.replace("</channel>", "<description>new</description></channel>");
    server.set("/album.xml", 200, changed.into_bytes());
    let (_, found) = call(&app, "GET", &path, None).await;
    assert_eq!(found["state"], json!("changed"), "ADR 0150 item 2");

    let moved = FEED
        .replace(
            "<rss ",
            "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" ",
        )
        .replace(
            "</channel>",
            "<itunes:new-feed-url>https://other.example/a.xml</itunes:new-feed-url></channel>",
        );
    server.set("/album.xml", 200, moved.clone().into_bytes());
    let (_, found) = call(&app, "GET", &path, None).await;
    assert_eq!(found["state"], json!("moved"), "ADR 0150 item 3: {found}");
    assert_eq!(found["new_url"], json!("https://other.example/a.xml"));

    server.set("/album.xml", 500, Vec::new());
    let (_, kept) = call(&app, "GET", &path, None).await;
    assert_eq!(kept["state"], json!("moved"), "ADR 0150 item 7");

    server.set("/album.xml", 200, moved.into_bytes());
    let (status, _) = call(&app, "POST", &format!("{path}/keep"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, hidden) = call(&app, "GET", &path, None).await;
    assert_eq!(hidden["state"], Value::Null, "ADR 0150 item 4");

    // The check writes nothing and makes no draft (ADR 0150 item 8).
    assert_eq!(
        std::fs::read_to_string(app.feeds.path().join("album.xml")).unwrap(),
        FEED
    );
    let (_, list) = call(&app, "GET", "/api/v1/feeds", None).await;
    assert_eq!(list[0]["id"], json!(id));
}

#[tokio::test]
async fn the_outside_check_compares_the_address_of_musicindex_only_on_request() {
    let (app, server, id) = outside_app().await;
    let path = format!("/api/v1/feeds/{id}/outside");
    let (_, list) = call(&app, "GET", "/api/v1/feeds", None).await;
    let own = list[0]["public_url"].as_str().unwrap().to_owned();
    let answer = |url: &str| {
        serde_json::to_vec(&json!({ "data": { "feed_url": url, "title": "A" } })).unwrap()
    };
    // The client keeps only the hexadecimal digits of the GUID `g-1`.
    let route = "/index/v1/feeds/-1";

    // No entry gives nothing.
    server.set(route, 404, Vec::new());
    let (_, none) = call(&app, "GET", &format!("{path}?index=true"), None).await;
    assert_eq!(none["index_url"], Value::Null, "ADR 0150 item 5");

    // The same address gives nothing.
    server.set(route, 200, answer(&own));
    let (_, same) = call(&app, "GET", &format!("{path}?index=true"), None).await;
    assert_eq!(same["index_url"], Value::Null, "ADR 0150 item 5: {same}");

    // A different address gives both.
    server.set(route, 200, answer("https://other.example/a.xml"));
    let (_, plain) = call(&app, "GET", &path, None).await;
    assert_eq!(
        plain["index_url"],
        Value::Null,
        "no request without index=true"
    );
    let (_, found) = call(&app, "GET", &format!("{path}?index=true"), None).await;
    assert_eq!(found["index_url"], json!("https://other.example/a.xml"));
    assert_eq!(found["own_url"], json!(own));

    // No answer gives nothing.
    server.set(route, 500, Vec::new());
    let (_, failed) = call(&app, "GET", &format!("{path}?index=true"), None).await;
    assert_eq!(failed["index_url"], Value::Null, "ADR 0150 item 5");
}

#[tokio::test]
async fn the_cleared_check_box_stops_each_request_of_the_outside_check() {
    let (app, server, id) = outside_app().await;
    let path = format!("/api/v1/feeds/{id}/outside");
    let (_, on) = call(&app, "GET", "/api/v1/settings/outside-check", None).await;
    assert_eq!(on["enabled"], json!(true), "ADR 0150 item 10");
    call(
        &app,
        "PUT",
        "/api/v1/settings/outside-check",
        Some(json!({ "enabled": false })),
    )
    .await;
    let changed = FEED.replace("</channel>", "<description>new</description></channel>");
    server.set("/album.xml", 200, changed.into_bytes());
    let (_, off) = call(&app, "GET", &path, None).await;
    assert_eq!(off["enabled"], json!(false));
    assert_eq!(off["state"], Value::Null);
    call(
        &app,
        "PUT",
        "/api/v1/settings/outside-check",
        Some(json!({ "enabled": true })),
    )
    .await;
    let (_, on) = call(&app, "GET", &path, None).await;
    assert_eq!(on["state"], json!("changed"));
}

#[tokio::test]
async fn a_read_and_a_write_give_the_outside_check_a_new_start() {
    let (app, server, id) = outside_app().await;
    let path = format!("/api/v1/feeds/{id}/outside");
    // A new read takes the new bytes.
    let changed = FEED.replace("</channel>", "<description>new</description></channel>");
    std::fs::write(app.feeds.path().join("album.xml"), &changed).unwrap();
    server.set("/album.xml", 200, changed.clone().into_bytes());
    let (_, found) = call(&app, "GET", &path, None).await;
    assert_eq!(found["state"], json!("changed"));
    call(&app, "POST", &format!("/api/v1/feeds/{id}/refresh"), None).await;
    let (_, read) = call(&app, "GET", &path, None).await;
    assert_eq!(
        read["state"],
        Value::Null,
        "ADR 0150 item 2: a read ends the result."
    );
    // A write of Studio is not a change from outside.
    call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    set_library_root(&app);
    rename_through_route(&app, id, "Album 2").await;
    let (status, body) = write_through_routes(&app, id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, stale) = call(&app, "GET", &path, None).await;
    assert_eq!(
        stale["state"],
        Value::Null,
        "the file from before the write"
    );
    let written = std::fs::read(app.feeds.path().join("album.xml")).unwrap();
    server.set("/album.xml", 200, written);
    let (_, new) = call(&app, "GET", &path, None).await;
    assert_eq!(
        new["state"],
        Value::Null,
        "ADR 0150: a write of Studio is not a change."
    );
}

/// A local index that holds its first answer until the test releases it. The
/// answer gives one feed, and each later request gets the same answer at once.
struct HeldIndex {
    url: String,
    received: std::sync::mpsc::Receiver<()>,
    release: std::sync::mpsc::Sender<()>,
}

fn held_index(requests: usize) -> HeldIndex {
    use std::io::{Read as _, Write as _};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (received_sender, received) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let body = br#"{"data":[{"entity_type":"feed","entity_id":"b-1","title":"Album B","release_artist":"Ann"}]}"#;
        for number in 0..requests {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut request = Vec::new();
            let mut part = [0_u8; 1024];
            while !request.windows(4).any(|end| end == b"\r\n\r\n") {
                match stream.read(&mut part) {
                    Ok(count) if count > 0 => request.extend_from_slice(&part[..count]),
                    _ => return,
                }
            }
            if number == 0 {
                let _ = received_sender.send(());
                let _ = released.recv_timeout(std::time::Duration::from_secs(30));
            }
            let head = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body);
        }
    });
    HeldIndex {
        url,
        received,
        release,
    }
}

/// Send one search, take the workspace lock while the index holds its answer,
/// and give the body of the response that arrives with the lock still held.
#[expect(
    clippy::await_holding_lock,
    reason = "the test holds the lock during the search on purpose"
)]
async fn search_with_the_lock_held(path: &'static str, requests: usize) -> Value {
    use axum::routing::get;
    let time_limit = std::time::Duration::from_secs(20);
    let index = held_index(requests);
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_musicindex(&index.url);
    let shared = std::sync::Arc::new(std::sync::Mutex::new(workspace));
    let router = Router::new()
        .route("/search", get(crate::routes::search))
        .route("/musicindex/search", get(crate::people::search_index))
        .with_state(std::sync::Arc::clone(&shared));
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    let search = tokio::spawn(router.oneshot(request));

    let received = index.received;
    tokio::task::spawn_blocking(move || received.recv_timeout(time_limit))
        .await
        .unwrap()
        .unwrap_or_else(|e| panic!("{path}: the index got no request: {e}"));

    let held = shared.try_lock().unwrap_or_else(|e| {
        panic!(
            "ADR 0033 item 1: {path} holds the workspace lock during its index request ({e}). \
             Read the index under the lock, then send the request in blocking_unlocked, as nostr_profiles does."
        )
    });
    index.release.send(()).unwrap();
    let response = tokio::time::timeout(time_limit, search)
        .await
        .unwrap_or_else(|_| panic!("ADR 0033 item 1: {path} waits for the workspace lock."))
        .unwrap()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    drop(held);
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn the_feed_search_completes_while_the_workspace_lock_is_held() {
    let body = search_with_the_lock_held("/search?q=ann", 1).await;
    assert_eq!(body["feeds"][0]["feed_guid"], "b-1");
    assert_eq!(body["feeds"][0]["artist"], "Ann");
}

#[tokio::test]
async fn the_picker_search_completes_while_the_workspace_lock_is_held() {
    let body = search_with_the_lock_held("/musicindex/search?q=ann", 2).await;
    assert_eq!(body["feeds"][0]["feed_guid"], "b-1");
    assert_eq!(body["rows"], json!([]));
}

/// A local index that records the request line of each search and gives one
/// answer to each request.
fn recording_index(body: &'static [u8]) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    use std::io::{Read as _, Write as _};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = std::sync::Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let mut request = Vec::new();
            let mut part = [0_u8; 1024];
            while !request.windows(4).any(|end| end == b"\r\n\r\n") {
                match stream.read(&mut part) {
                    Ok(count) if count > 0 => request.extend_from_slice(&part[..count]),
                    _ => break,
                }
            }
            let text = String::from_utf8_lossy(&request);
            log.lock()
                .unwrap()
                .push(text.lines().next().unwrap_or_default().to_owned());
            let head = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body);
        }
    });
    (url, seen)
}

const ONE_FEED: &[u8] =
    br#"{"data":[{"entity_type":"feed","entity_id":"b-1","title":"Album B","release_artist":"Ann"}]}"#;

/// The app of a search test, with the path of the search of one person.
async fn search_app(
    body: &'static [u8],
) -> (App, std::sync::Arc<std::sync::Mutex<Vec<String>>>, String) {
    let (url, seen) = recording_index(body);
    let app = app_with_index(&url);
    let (_, person) = call(
        &app,
        "POST",
        "/api/v1/people",
        Some(json!({ "name": "Ann" })),
    )
    .await;
    let path = format!("/api/v1/people/{}/musicindex", person["id"]);
    (app, seen, path)
}

#[tokio::test]
async fn a_kind_field_gives_the_kind_parameters_at_the_index() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    for path in ["/api/v1/search", "/api/v1/musicindex/search"] {
        seen.lock().unwrap().clear();
        let (status, body) = call(&app, "GET", &format!("{path}?q=kind%3Asingle"), None).await;
        assert_eq!(status, StatusCode::OK, "{path}: {body}");
        assert_eq!(body["feeds"][0]["feed_guid"], "b-1", "{path}");
        let lines = seen.lock().unwrap().clone();
        assert_eq!(
            lines.len(),
            1,
            "ADR 0151: {path} sends one request: {lines:?}"
        );
        let line = &lines[0];
        assert!(
            line.contains("type=feed")
                && line.contains("kind=unknown")
                && line.contains("tracks=1"),
            "musicindex ADR 0049 R-QUERY-6: {path}: {line}"
        );
        assert!(
            !line.contains("q=kind"),
            "R-QUERY-1: field syntax in q: {line}"
        );
        assert!(!line.contains("sort="), "ADR 0151 item 8: {line}");
    }
    // A publisher row takes no kind, so the search of a person sends nothing.
    seen.lock().unwrap().clear();
    let (status, body) = call(&app, "GET", &format!("{person}?q=kind%3Asingle"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["rows"], json!([]));
    assert!(seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_word_with_a_field_sends_only_the_word_to_the_artist_and_label_search() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    let (status, body) = call(&app, "GET", &format!("{person}?q=ann%20artist%3Abob"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let lines = seen.lock().unwrap().clone();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].contains("q=ann&")
            && lines[0].contains("artist=bob")
            && lines[0].contains("type=publisher"),
        "ADR 0151: {}",
        lines[0]
    );
}

#[tokio::test]
async fn a_field_error_gives_its_message_at_each_route_and_sends_nothing() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    for path in [
        "/api/v1/search",
        "/api/v1/musicindex/search",
        person.as_str(),
    ] {
        let (status, body) = call(&app, "GET", &format!("{path}?q=kind%3Anope"), None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path}: {body}");
        assert_eq!(
            body["error"]["code"], "query_field",
            "ADR 0151 item 5: {path}"
        );
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .starts_with("kind:"),
            "{path}: {body}"
        );
    }
    assert!(seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_library_field_gives_the_note_and_no_rows_at_each_route() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    for path in [
        "/api/v1/search",
        "/api/v1/musicindex/search",
        person.as_str(),
    ] {
        let (status, body) = call(&app, "GET", &format!("{path}?q=ann%20is%3Aready"), None).await;
        assert_eq!(status, StatusCode::OK, "{path}: {body}");
        assert_eq!(
            body["note"]["code"], "library_only",
            "R-QUERY-2: {path}: {body}"
        );
        assert_eq!(body.get("feeds").or(body.get("rows")).unwrap(), &json!([]));
    }
    assert!(
        seen.lock().unwrap().is_empty(),
        "R-QUERY-2: no Index request"
    );
}

#[tokio::test]
async fn an_answer_that_cannot_decode_gives_an_error_at_each_route() {
    let (app, _seen, person) = search_app(b"not json").await;
    for path in [
        "/api/v1/search",
        "/api/v1/musicindex/search",
        person.as_str(),
    ] {
        let (status, body) = call(&app, "GET", &format!("{path}?q=ann"), None).await;
        assert_eq!(
            status,
            StatusCode::BAD_GATEWAY,
            "ADR 0151 item 7: {path}: {body}"
        );
        assert_eq!(body["error"]["code"], "index_answer", "{path}");
    }
}

#[tokio::test]
async fn a_plain_start_text_gives_no_field_at_each_route() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    for path in [
        "/api/v1/search",
        "/api/v1/musicindex/search",
        person.as_str(),
    ] {
        seen.lock().unwrap().clear();
        let (status, body) =
            call(&app, "GET", &format!("{path}?q=In%3AFlux&plain=true"), None).await;
        assert_eq!(status, StatusCode::OK, "ADR 0151 item 3: {path}: {body}");
        let lines = seen.lock().unwrap().clone();
        assert!(!lines.is_empty(), "{path}");
        assert!(
            lines.iter().all(|line| line.contains("q=In%3AFlux")),
            "{path}: {lines:?}"
        );
        let (status, _) = call(&app, "GET", &format!("{path}?q=In%3AFlux"), None).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "the edited text reads: {path}"
        );
    }
}

#[tokio::test]
async fn a_field_with_no_request_for_artists_gives_the_note_and_sends_nothing() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    for (text, token) in [
        ("kind%3Asingle", "kind:"),
        ("able%20kind%3Aep", "kind:"),
        ("able%20after%3A2026-01", "after:"),
        ("able%20before%3A2026-01", "before:"),
        ("able%20duration%3A%3E5m", "duration:"),
        ("able%20release%3Ax", "release:"),
    ] {
        let (status, body) = call(&app, "GET", &format!("{person}?q={text}"), None).await;
        assert_eq!(status, StatusCode::OK, "{text}: {body}");
        assert_eq!(body["rows"], json!([]), "{text}");
        assert_eq!(body["searched"], false, "ADR 0151 item 9: {text}");
        assert_eq!(body["note"]["code"], "not_for_publishers", "{text}");
        assert_eq!(
            body["note"]["message"],
            format!("{token} does not apply to artist and label feeds"),
            "ADR 0151 item 9: {text}"
        );
    }
    assert!(
        seen.lock().unwrap().is_empty(),
        "ADR 0151 item 9: the search sends nothing"
    );
}

#[tokio::test]
async fn a_request_with_no_rows_gives_no_note_and_is_marked_as_a_search() {
    let (app, seen, person) = search_app(br#"{"data":[]}"#).await;
    let (status, body) = call(&app, "GET", &format!("{person}?q=zzz"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["rows"], json!([]));
    assert_eq!(body["note"], Value::Null);
    assert_eq!(body["searched"], true, "ADR 0151 item 9: a request ran");
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_text_with_nothing_to_search_is_not_marked_as_a_search() {
    let (app, seen, person) = search_app(ONE_FEED).await;
    let (status, body) = call(&app, "GET", &format!("{person}?q=in%3Aindex"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["note"], Value::Null);
    assert_eq!(body["searched"], false, "ADR 0151 item 9");
    assert!(seen.lock().unwrap().is_empty());
}
