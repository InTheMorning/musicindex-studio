//! Contract tests of the local API (task 007).

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
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let workspace = Workspace::open(data.path()).unwrap().with_musicindex(index);
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

/// Make the vault, a local connector and a feed. Gives the feed ID.
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
    feed["id"].as_i64().unwrap()
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

    for path in ["/api/v1/status", "/api/v1/connectors", "/api/v1/feeds"] {
        let (status, _, text) = send(&app, request("GET", path, None)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        for forbidden in [secret, PASSPHRASE, vault_key.as_str()] {
            assert!(!text.contains(forbidden), "{path} contains a secret");
        }
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
        assert!(policy.starts_with("default-src 'self'"));
        assert!(!policy.contains("unsafe"));
    }
    let request = Request::builder()
        .uri("/ui/../Cargo.toml")
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&app, request).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
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
    assert_eq!(body[0]["feed_guid"], "b-1");
    assert_eq!(body[0]["artist"], "Ann");
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
async fn each_draft_route_of_a_read_only_feed_gives_read_only() {
    let app = app();
    set_up(&app).await;
    let (_, feed) = call(&app, "POST", "/api/v1/feeds", Some(json!({}))).await;
    let id = feed["id"].as_i64().unwrap();
    let model = serde_json::to_value(studio_core::feed::Feed::default()).unwrap();
    let body = json!({ "revision": "r", "model": model });
    for (method, path, body) in [
        ("GET", format!("/api/v1/feeds/{id}/draft"), None),
        ("PUT", format!("/api/v1/feeds/{id}/draft"), Some(body)),
        ("DELETE", format!("/api/v1/feeds/{id}/draft"), None),
        ("GET", format!("/api/v1/feeds/{id}/diff"), None),
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
