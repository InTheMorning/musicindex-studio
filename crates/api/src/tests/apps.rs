//! Tests of the `/app/v1/` routes and the connected apps (task 073c,
//! ADR 0110). The tests use `studio-app-client` as the client (ADR 0110
//! item 30). The API runs on a loopback port, so the client sends real
//! HTTP requests. No test sends a request to a host outside the computer.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use sha2::{Digest, Sha256};
use studio_app_client::nostr::prelude::Tag;
use studio_app_client::{AppSigner, Client, Error as ClientError, MemorySigner, PairingText};

use super::*;
use crate::{WallClock, WorkspaceHandle};

const LIVE_ORIGIN: &str = "wss://example.com";

/// An API on a loopback port, with a clock that the test can move.
struct Live {
    app: App,
    handle: WorkspaceHandle,
    origin: String,
    offset: Arc<AtomicI64>,
}

fn real_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// The workspace of a data folder with the public origin and the clock.
fn workspace_for(data: &std::path::Path, origin: &str, offset: &Arc<AtomicI64>) -> Workspace {
    let offset = Arc::clone(offset);
    Workspace::open(data)
        .unwrap()
        .with_musicindex("http://127.0.0.1:9")
        .with_lookup_relays(vec![UNUSED_RELAY.to_owned()])
        .with_address_check(Arc::new(PassAll))
        .with_public_origin(origin)
        .with_wall_clock(WallClock::new(move || {
            u64::try_from(real_now() + offset.load(Ordering::SeqCst)).unwrap_or(0)
        }))
}

/// A router for requests in memory. The host is `HOST` (port 4100).
fn memory_router(routes: &Router) -> Router {
    crate::ui_router()
        .merge(routes.clone())
        .layer(axum::middleware::from_fn_with_state(
            Access::with_token(TOKEN, 4100),
            crate::access::check,
        ))
}

async fn live_app() -> Live {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let origin = format!("http://127.0.0.1:{port}");
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    std::fs::write(feeds.path().join("album.xml"), FEED).unwrap();
    let offset = Arc::new(AtomicI64::new(0));
    let workspace = workspace_for(data.path(), &origin, &offset);
    let (routes, handle) = crate::workspace_router(workspace);
    let network =
        crate::ui_router()
            .merge(routes.clone())
            .layer(axum::middleware::from_fn_with_state(
                Access::with_token(TOKEN, port),
                crate::access::check,
            ));
    tokio::spawn(async move {
        let service = network.into_make_service_with_connect_info::<std::net::SocketAddr>();
        axum::serve(listener, service).await.unwrap();
    });
    Live {
        app: App {
            router: memory_router(&routes),
            data,
            feeds,
        },
        handle,
        origin,
        offset,
    }
}

/// Run a blocking call of the client on a blocking thread.
async fn on<T: Send + 'static>(
    client: &Arc<Client>,
    work: impl FnOnce(&Client) -> T + Send + 'static,
) -> T {
    let client = Arc::clone(client);
    tokio::task::spawn_blocking(move || work(&client))
        .await
        .unwrap()
}

/// "Connect an App": the pairing text of a new waiting app.
async fn add_app(live: &Live, name: &str) -> (i64, String) {
    let (status, body) = call(
        &live.app,
        "POST",
        "/api/v1/apps",
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    (
        body["id"].as_i64().unwrap(),
        body["pairing_text"].as_str().unwrap().to_owned(),
    )
}

/// Pair an app with the client. Gives the client, the ID of the app and the
/// signer, which the test can use for events of its own.
async fn paired(live: &Live, name: &str) -> (Arc<Client>, i64, MemorySigner) {
    make_vault(&live.app).await;
    let (id, text) = add_app(live, name).await;
    let signer = MemorySigner::generate().unwrap();
    let given = signer.clone();
    let client = tokio::task::spawn_blocking(move || {
        let text = PairingText::parse(&text).unwrap();
        Client::pair(&text, given, LIVE_ORIGIN)
    })
    .await
    .unwrap()
    .expect("ADR 0110 item 10: a pairing with a correct code must work");
    (Arc::new(client), id, signer)
}

fn hex_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut text, b| {
            let _ = write!(text, "{b:02x}");
            text
        })
}

/// The tags of a signed request.
struct Signed<'a> {
    url: String,
    method: &'a str,
    payload: Option<String>,
    created_at: i64,
}

/// The header `Nostr <base64 event>` of an event that the test signs.
fn header_of(signer: &MemorySigner, request: &Signed<'_>) -> String {
    use base64::Engine as _;
    let tag = |name: &str, value: &str| Tag::parse([name, value]).unwrap();
    let mut tags = vec![tag("u", &request.url), tag("method", request.method)];
    if let Some(payload) = &request.payload {
        tags.push(tag("payload", payload));
    }
    // A new nonce makes a new event ID for each call.
    let mut nonce = [0_u8; 8];
    getrandom::fill(&mut nonce).unwrap();
    tags.push(tag("nonce", &format!("{nonce:?}")));
    let event = signer
        .sign(27_235, tags, u64::try_from(request.created_at).unwrap())
        .unwrap();
    let json = serde_json::to_vec(&event).unwrap();
    format!(
        "Nostr {}",
        base64::engine::general_purpose::STANDARD.encode(json)
    )
}

/// A request to an `/app/v1/` route in memory, with the given header.
async fn raw(
    live: &Live,
    method: &str,
    path: &str,
    authorization: Option<String>,
    body: &[u8],
) -> (StatusCode, Value, String) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, HOST);
    if let Some(authorization) = authorization {
        builder = builder.header(header::AUTHORIZATION, authorization);
    }
    send(
        &live.app,
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_vec()))
            .unwrap(),
    )
    .await
}

fn signed(live: &Live, method: &'static str, path: &str, body: &[u8]) -> Signed<'static> {
    Signed {
        url: format!("{}{path}", live.origin),
        method,
        payload: (!body.is_empty()).then(|| hex_sha256(body)),
        created_at: real_now() + live.offset.load(Ordering::SeqCst),
    }
}

/// Write a pending show with its draft grant for `app_id`. Gives the
/// `guid` of the show.
async fn granted_show(app: &App, id: i64, app_id: i64, live_xml: &dyn Fn() -> String) -> String {
    let base = format!("/api/v1/feeds/{id}");
    // The start is far in the future, so that the grant is active at the
    // time of the test.
    let (status, draft) = call(
        app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2099-11-01T10:00:00Z" })),
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
    let (status, _) = call(
        app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let grant_path = format!("{base}/grants/{guid}");
    let (status, body) = call(app, "PUT", &grant_path, Some(json!({ "app_id": app_id }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, diff) = call(app, "PUT", &grant_path, Some(json!({ "app_id": app_id }))).await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["grants"], json!([guid]));
    assert_eq!(diff["grant_apps"][&guid], app_id);
    let (status, body) = call(
        app,
        "POST",
        &format!("{base}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(live_xml().contains("status=\"pending\""));
    guid
}

fn album_xml(live: &Live) -> String {
    std::fs::read_to_string(live.app.feeds.path().join("album.xml")).unwrap()
}

// ---- Pairing ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pairing_with_a_correct_code_makes_an_active_app_and_the_signed_requests_work() {
    let live = live_app().await;
    let (client, id, signer) = paired(&live, "Show tool").await;
    let me = on(&client, Client::whoami).await.unwrap();
    assert_eq!(me.id, id);
    assert_eq!(me.name, "Show tool");
    assert_eq!(me.state.as_deref(), Some("active"));
    assert_eq!(me.live_value_origin.as_deref(), Some(LIVE_ORIGIN));
    let short = me.short_key.unwrap();
    assert!(short.starts_with("npub1"), "{short}");

    let (status, list) = call(&live.app, "GET", "/api/v1/apps", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert_eq!(list[0]["state"], "active");
    assert_eq!(list[0]["short_key"], short);
    assert_eq!(list[0]["live_value_origin"], LIVE_ORIGIN);
    assert!(list[0]["last_used_at"].is_i64(), "{list}");
    assert_eq!(list[0]["last_address"], "127.0.0.1");
    assert_eq!(list[0]["shows"], json!([]));
    assert_eq!(list[0]["code_expires_at"], Value::Null);
    let text = list.to_string();
    assert!(
        !text.contains(&signer.public_key().to_hex()),
        "ADR 0110 item 18: the app list must show only the short key"
    );

    // The state of one app, for the notice of the pairing.
    let (status, one) = call(&live.app, "GET", &format!("/api/v1/apps/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(one["state"], "active");
    let (status, _) = call(&live.app, "GET", "/api/v1/apps/999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_response_of_the_add_route_has_the_pairing_text_and_the_list_has_no_code() {
    let live = live_app().await;
    make_vault(&live.app).await;
    let (status, body) = call(
        &live.app,
        "POST",
        "/api/v1/apps",
        Some(json!({ "name": "Show tool" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let code = body["code"].as_str().unwrap().to_owned();
    assert_eq!(code.len(), 9);
    assert_eq!(
        body["pairing_text"],
        format!("{}#pair={code}", live.origin),
        "ADR 0110 item 6: the pairing text is the public origin of the shell and the code"
    );
    assert_eq!(body["state"], "waiting");
    assert!(body["code_expires_at"].is_i64());
    assert!(body.get("token").is_none());
    let id = body["id"].as_i64().unwrap();
    for path in ["/api/v1/apps".to_owned(), format!("/api/v1/apps/{id}")] {
        let (_, _, text) = send(&live.app, request("GET", &path, None)).await;
        assert!(
            !text.contains(&code) && !text.contains("code_hash"),
            "ADR 0110 item 7: {path} must not hold the pairing code"
        );
    }
    // Done before the pairing removes the app and its code (item 8).
    let (status, _) = call(&live.app, "DELETE", &format!("/api/v1/apps/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&live.app, "GET", &format!("/api/v1/apps/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let signer = MemorySigner::generate().unwrap();
    let text = PairingText::parse(&format!("{}#pair={code}", live.origin)).unwrap();
    let result = tokio::task::spawn_blocking(move || Client::pair(&text, signer, LIVE_ORIGIN))
        .await
        .unwrap();
    assert!(
        matches!(result, Err(ClientError::CodeInvalid)),
        "ADR 0110 item 11: a code after Done must give app_code_invalid. Remove the code with the app."
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_used_code_an_unknown_code_and_an_expired_code_fail() {
    let live = live_app().await;
    make_vault(&live.app).await;
    let (_, text) = add_app(&live, "One").await;
    let parsed = PairingText::parse(&text).unwrap();
    let again = parsed.clone();
    let first = tokio::task::spawn_blocking(move || {
        Client::pair(&parsed, MemorySigner::generate().unwrap(), LIVE_ORIGIN)
    })
    .await
    .unwrap();
    assert!(first.is_ok());
    let second = tokio::task::spawn_blocking(move || {
        Client::pair(&again, MemorySigner::generate().unwrap(), LIVE_ORIGIN)
    })
    .await
    .unwrap();
    assert!(
        matches!(second, Err(ClientError::CodeUsed)),
        "ADR 0110 item 11: a code that an app used in the last 15 minutes must give app_code_used: {second:?}"
    );

    let unknown = PairingText::parse(&format!("{}#pair=ZZZZ-ZZZZ", live.origin)).unwrap();
    let result = tokio::task::spawn_blocking(move || {
        Client::pair(&unknown, MemorySigner::generate().unwrap(), LIVE_ORIGIN)
    })
    .await
    .unwrap();
    assert!(matches!(result, Err(ClientError::CodeInvalid)));

    // The 15 minutes end.
    let (_, text) = add_app(&live, "Two").await;
    let connection =
        rusqlite::Connection::open(live.app.data.path().join("collection.sqlite")).unwrap();
    connection
        .execute(
            "UPDATE connected_apps SET code_expires_at = ?1 WHERE name = 'Two'",
            [real_now() - 1],
        )
        .unwrap();
    let expired = PairingText::parse(&text).unwrap();
    let result = tokio::task::spawn_blocking(move || {
        Client::pair(&expired, MemorySigner::generate().unwrap(), LIVE_ORIGIN)
    })
    .await
    .unwrap();
    assert!(
        matches!(result, Err(ClientError::CodeInvalid)),
        "ADR 0110 item 8: an expired code must give app_code_invalid: {result:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pairing_with_a_wrong_origin_gives_400_and_another_route_refuses_a_key_that_no_app_has() {
    let live = live_app().await;
    make_vault(&live.app).await;
    let (_, text) = add_app(&live, "One").await;
    let parsed = PairingText::parse(&text).unwrap();
    let result = tokio::task::spawn_blocking(move || {
        Client::pair(
            &parsed,
            MemorySigner::generate().unwrap(),
            "wss://example.com",
        )
    })
    .await
    .unwrap();
    assert!(result.is_ok());
    // A live value origin that is not https or wss gives 400.
    let (_, text) = add_app(&live, "Two").await;
    let code = text.split("#pair=").nth(1).unwrap().to_owned();
    let body = json!({ "code": code, "live_value_origin": "http://example.com" }).to_string();
    let signer = MemorySigner::generate().unwrap();
    let header = header_of(
        &signer,
        &signed(&live, "POST", "/app/v1/pair", body.as_bytes()),
    );
    let (status, error, _) =
        raw(&live, "POST", "/app/v1/pair", Some(header), body.as_bytes()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
    assert_eq!(
        error["error"]["code"], "live_value_origin_invalid",
        "ADR 0110 item 24: an origin that is not https or wss must give 400"
    );
    // A route other than the pairing refuses a key that no app has.
    let stranger = MemorySigner::generate().unwrap();
    let path = "/app/v1/whoami";
    let header = header_of(&stranger, &signed(&live, "GET", path, b""));
    let (status, body, _) = raw(&live, "GET", path, Some(header), b"").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body["error"]["code"], "app_signature_invalid",
        "ADR 0110 item 13: a key that no app has must give app_signature_invalid"
    );
}

// ---- The signed request ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn each_condition_of_item_13_gives_401_app_signature_invalid() {
    let live = live_app().await;
    let (client, _, signer) = paired(&live, "Show tool").await;
    let path = "/app/v1/whoami";
    let ok = |s: Signed<'static>| header_of(&signer, &s);
    let expect_refused = |name: &'static str, status: StatusCode, body: Value| {
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{name}: {body}");
        assert_eq!(
            body["error"]["code"], "app_signature_invalid",
            "ADR 0110 item 13: {name} must give 401 app_signature_invalid. Refuse it in app_check."
        );
    };
    // A correct request passes.
    let (status, body, _) = raw(
        &live,
        "GET",
        path,
        Some(ok(signed(&live, "GET", path, b""))),
        b"",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // The time is more than 60 seconds off.
    for shift in [-61, 61] {
        let mut s = signed(&live, "GET", path, b"");
        s.created_at += shift;
        let (status, body, _) = raw(&live, "GET", path, Some(ok(s)), b"").await;
        expect_refused("a time 61 seconds off", status, body);
    }
    // The u tag is not the address of the request.
    let mut s = signed(&live, "GET", path, b"");
    s.url = format!("{}/app/v1/shows", live.origin);
    let (status, body, _) = raw(&live, "GET", path, Some(ok(s)), b"").await;
    expect_refused("a u tag of another route", status, body);
    let mut s = signed(&live, "GET", path, b"");
    s.url = format!("https://other.example{path}");
    let (status, body, _) = raw(&live, "GET", path, Some(ok(s)), b"").await;
    expect_refused("a u tag of another origin", status, body);
    // The method tag is not the method.
    let s = signed(&live, "POST", path, b"");
    let (status, body, _) = raw(&live, "GET", path, Some(ok(s)), b"").await;
    expect_refused("a method tag of another method", status, body);
    // A body with no payload tag, and a wrong payload tag.
    let route = "/app/v1/shows/1/status";
    let body_bytes = br#"{"status":"live","grant_key":"x"}"#;
    let mut s = signed(&live, "POST", route, body_bytes);
    s.payload = None;
    let (status, body, _) = raw(&live, "POST", route, Some(ok(s)), body_bytes).await;
    expect_refused("a body with no payload tag", status, body);
    let mut s = signed(&live, "POST", route, body_bytes);
    s.payload = Some(hex_sha256(b"another body"));
    let (status, body, _) = raw(&live, "POST", route, Some(ok(s)), body_bytes).await;
    expect_refused("a payload tag of another body", status, body);
    // The event is not correct: the content changed after the signature.
    let header = ok(signed(&live, "GET", path, b""));
    let tampered = {
        use base64::Engine as _;
        let encoded = header.strip_prefix("Nostr ").unwrap();
        let mut event: Value = serde_json::from_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
        )
        .unwrap();
        event["content"] = json!("changed");
        format!(
            "Nostr {}",
            base64::engine::general_purpose::STANDARD.encode(event.to_string())
        )
    };
    let (status, body, _) = raw(&live, "GET", path, Some(tampered), b"").await;
    expect_refused("a changed event", status, body);
    // No header, and a header that is not an event.
    let (status, body, _) = raw(&live, "GET", path, None, b"").await;
    expect_refused("no header", status, body);
    let (status, body, _) = raw(&live, "GET", path, Some("Nostr !!".to_owned()), b"").await;
    expect_refused("a header that is not an event", status, body);
    // The key of the client still works after the refusals.
    assert!(on(&client, Client::whoami).await.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_bearer_header_gives_401_app_key_required_and_the_app_layer_refuses_a_big_body() {
    let live = live_app().await;
    let (_, _, signer) = paired(&live, "Show tool").await;
    for token in ["an-old-token", TOKEN] {
        let (status, body, text) = raw(
            &live,
            "GET",
            "/app/v1/whoami",
            Some(format!("Bearer {token}")),
            b"",
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            body["error"]["code"], "app_key_required",
            "ADR 0110 item 37: a Bearer header must give app_key_required"
        );
        assert!(!text.contains(token));
    }
    // A body over 64 KiB is refused before the hash.
    let route = "/app/v1/shows/1/status";
    let big = vec![b'a'; 64 * 1024 + 1];
    let header = header_of(&signer, &signed(&live, "POST", route, &big));
    let (status, body, _) = raw(&live, "POST", route, Some(header), &big).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(body["error"]["code"], "body_too_large");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_api_and_auth_routes_refuse_the_nostr_header() {
    let live = live_app().await;
    let (_, _, signer) = paired(&live, "Show tool").await;
    let header = header_of(&signer, &signed(&live, "GET", "/api/v1/status", b""));
    for path in ["/api/v1/status", "/auth/v1/session"] {
        let (status, body, _) = raw(&live, "GET", path, Some(header.clone()), b"").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(
            body["error"]["code"], "app_credential_refused",
            "ADR 0110 item 16: {path} must refuse the Nostr header. Check it in access::check."
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_repeated_event_is_refused_also_after_the_workspace_loads_again() {
    let live = live_app().await;
    let (_, _, signer) = paired(&live, "Show tool").await;
    let path = "/app/v1/whoami";
    let header = header_of(&signer, &signed(&live, "GET", path, b""));
    let (status, _, _) = raw(&live, "GET", path, Some(header.clone()), b"").await;
    assert_eq!(status, StatusCode::OK);
    let (status, body, _) = raw(&live, "GET", path, Some(header.clone()), b"").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body["error"]["code"], "app_signature_invalid",
        "ADR 0110 item 15: a repeated event must be refused. Record the event ID before the route runs."
    );
    // A restart: a new workspace on the same folder has the same store.
    let workspace = workspace_for(live.app.data.path(), &live.origin, &live.offset);
    let (routes, _handle) = crate::workspace_router(workspace);
    let restarted = Live {
        app: App {
            router: memory_router(&routes),
            data: tempfile::tempdir().unwrap(),
            feeds: tempfile::tempdir().unwrap(),
        },
        handle: live.handle.clone(),
        origin: live.origin.clone(),
        offset: Arc::clone(&live.offset),
    };
    let (status, _, _) = raw(&restarted, "GET", path, Some(header), b"").await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "ADR 0110 item 15: the replay store must live in the collection, not in memory"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn after_the_server_clock_goes_back_an_accepted_event_is_refused() {
    let live = live_app().await;
    let (_, _, signer) = paired(&live, "Show tool").await;
    let path = "/app/v1/whoami";
    let header = header_of(&signer, &signed(&live, "GET", path, b""));
    let (status, _, _) = raw(&live, "GET", path, Some(header.clone()), b"").await;
    assert_eq!(status, StatusCode::OK);
    live.offset.store(-3600, Ordering::SeqCst);
    let (status, body, _) = raw(&live, "GET", path, Some(header), b"").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "app_signature_invalid");
    // An event of the new time is older than the latest accepted event by
    // more than 60 seconds, so it is stale (ADR 0110 item 13).
    let fresh = header_of(&signer, &signed(&live, "GET", path, b""));
    let (status, body, _) = raw(&live, "GET", path, Some(fresh), b"").await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "ADR 0110 item 13: an event that is more than 60 seconds earlier than the latest accepted event must be refused: {body}"
    );
}

// ---- Grants ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_grant_names_one_active_app_and_the_review_sheet_gets_the_apps() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    // No app: 422 app_required.
    let base = format!("/api/v1/feeds/{id}");
    let (status, draft) = call(
        &live.app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2099-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let guid = draft["model"]["live_items"][0]["item"]["guid"]
        .as_str()
        .unwrap()
        .to_owned();
    let grant_path = format!("{base}/grants/{guid}");
    for body in [None, Some(json!({})), Some(json!({ "app_id": 99 }))] {
        let (status, error) = call(&live.app, "PUT", &grant_path, body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
        assert_eq!(
            error["error"]["code"], "app_required",
            "ADR 0110 item 19: a grant with no active app must give 422 app_required"
        );
    }
    // A waiting app is not active.
    let (waiting, _) = add_app(&live, "Waiting").await;
    let (status, error) = call(
        &live.app,
        "PUT",
        &grant_path,
        Some(json!({ "app_id": waiting })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["code"], "app_required");

    let (_client, app_id, signer) = paired(&live, "Show tool").await;
    let (status, diff) = call(
        &live.app,
        "PUT",
        &grant_path,
        Some(json!({ "app_id": app_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{diff}");
    assert_eq!(diff["grant_apps"][&guid], app_id);
    let apps = diff["apps"].as_array().unwrap();
    assert_eq!(apps.len(), 1, "the waiting app is not in the list: {diff}");
    assert_eq!(apps[0]["id"], app_id);
    assert_eq!(apps[0]["name"], "Show tool");
    assert_eq!(apps[0]["live_value_origin"], LIVE_ORIGIN);
    assert_eq!(
        apps[0]["short_key"],
        studio_core::apps::short_key(&signer.public_key().to_hex())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_approval_of_a_grant_fails_when_its_app_is_not_active() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (_, app_id, _) = paired(&live, "Show tool").await;
    let base = format!("/api/v1/feeds/{id}");
    let (_, draft) = call(
        &live.app,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2099-11-01T10:00:00Z" })),
    )
    .await;
    let guid = draft["model"]["live_items"][0]["item"]["guid"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut model = draft["model"].clone();
    model["live_items"][0]["content_links"] =
        json!([{ "href": "https://example.com/show", "label": "Watch" }]);
    call(
        &live.app,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    let (_, diff) = call(
        &live.app,
        "PUT",
        &format!("{base}/grants/{guid}"),
        Some(json!({ "app_id": app_id })),
    )
    .await;
    // The app goes back to the state token (an import does this) before the
    // write.
    let connection =
        rusqlite::Connection::open(live.app.data.path().join("collection.sqlite")).unwrap();
    connection
        .execute(
            "UPDATE connected_apps SET state = 'token', pubkey = NULL",
            [],
        )
        .unwrap();
    let (status, body) = call(
        &live.app,
        "POST",
        &format!("{base}/write"),
        Some(json!({ "diff_id": diff["id"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(
        body["error"]["code"], "app_not_active",
        "ADR 0110 item 19: the approval of a grant with an app that is not active must fail, not skip the grant"
    );
    assert!(!album_xml(&live).contains("Show"), "the write must not run");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_granted_write_asks_the_time_with_the_lock_free_and_sets_the_build_date() {
    // ADR 0127 item 21. The servers say the time of the computer.
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    let guid = granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let log = TimeLog {
        free: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        seen: Arc::default(),
    };
    let probe = TimeProbe {
        shared: live.handle.shared_for_test(),
        free: Arc::clone(&log.free),
        seen: Arc::clone(&log.seen),
        answer: Some(real_now()),
    };
    live.handle
        .with_workspace(move |w| {
            w.server_time = crate::workspace::ServerTimeSource(Arc::new(probe));
        })
        .await;
    assert!(!album_xml(&live).contains("lastBuildDate"));
    let grant_id = on(&client, Client::shows).await.unwrap()[0].grant_id;
    assert_ne!(guid, "");
    on(&client, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Live)
    })
    .await
    .unwrap();
    assert!(album_xml(&live).contains("<lastBuildDate>"));
    assert!(album_xml(&live).contains("status=\"live\""));
    assert_eq!(log.requests().len(), 2, "{:?}", log.requests());
    assert!(
        log.lock_was_free(),
        "ADR 0127 item 21: the time request of a granted write ran while the workspace was locked. Ask the servers in granted_change_timed."
    );
    // The next granted write in 10 minutes sends no new request.
    on(&client, move |c| c.remove_live_value(grant_id))
        .await
        .unwrap();
    assert_eq!(log.requests().len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_connected_app_starts_a_show_sets_its_live_value_and_ends_it() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    let guid = granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;

    let shows = on(&client, Client::shows).await.unwrap();
    assert_eq!(shows.len(), 1);
    let show = &shows[0];
    assert_eq!(show.feed_id, id);
    assert_eq!(show.feed_title.as_deref(), Some("Album"));
    assert_eq!(show.item_guid, guid);
    assert_eq!(show.title.as_deref(), Some("Show"));
    assert_eq!(show.status.as_deref(), Some("pending"));
    assert_eq!(show.start.as_deref(), Some("2099-11-01T10:00:00Z"));
    assert_eq!(show.live_value_origin.as_deref(), Some(LIVE_ORIGIN));
    let grant_id = show.grant_id;

    let result = on(&client, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Live)
    })
    .await
    .unwrap();
    assert_eq!(result.status.as_deref(), Some("live"));
    assert!(album_xml(&live).contains("status=\"live\""));
    let result = on(&client, move |c| {
        c.set_live_value(grant_id, "wss://example.com/value", "socket.io")
    })
    .await
    .unwrap();
    assert_eq!(result.live_value.unwrap().uri, "wss://example.com/value");
    assert!(album_xml(&live).contains("uri=\"wss://example.com/value\""));
    let result = on(&client, move |c| c.remove_live_value(grant_id))
        .await
        .unwrap();
    assert!(result.live_value.is_none());
    assert!(!album_xml(&live).contains("liveValue"));
    let result = on(&client, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Ended)
    })
    .await
    .unwrap();
    assert_eq!(result.status.as_deref(), Some("ended"));
    assert!(!client.holds_grant_key(grant_id));
    assert_eq!(on(&client, Client::shows).await.unwrap().len(), 0);

    let (status, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{grants}");
    assert_eq!(grants[0]["app_id"], app_id);
    let changes: Vec<&str> = grants[0]["writes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["change"].as_str().unwrap())
        .collect();
    assert_eq!(
        changes,
        ["go_live", "set_live_value", "remove_live_value", "end"]
    );
    // The app row shows the use.
    let (_, one) = call(&live.app, "GET", &format!("/api/v1/apps/{app_id}"), None).await;
    assert!(one["last_used_at"].is_i64());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_granted_live_value_on_another_origin_with_http_or_a_wrong_protocol_gives_400() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let grant_id = on(&client, Client::shows).await.unwrap()[0].grant_id;
    let before = album_xml(&live);
    let long = format!("wss://example.com/{}", "a".repeat(2049));
    for (uri, protocol) in [
        (
            "wss://other.example/value".to_owned(),
            "socket.io".to_owned(),
        ),
        ("ws://example.com/value".to_owned(), "socket.io".to_owned()),
        (
            "wss://example.com/value".to_owned(),
            "bad protocol".to_owned(),
        ),
        ("wss://example.com/value".to_owned(), "p".repeat(33)),
        (long, "socket.io".to_owned()),
        (
            "wss://example.com/va\nlue".to_owned(),
            "socket.io".to_owned(),
        ),
    ] {
        let result = on(&client, move |c| {
            c.set_live_value(grant_id, &uri, &protocol)
        })
        .await;
        assert!(
            matches!(result, Err(ClientError::LiveValueInvalid)),
            "ADR 0110 item 26: a live value that is not on the origin of the grant must give 400 live_value_invalid: {result:?}"
        );
    }
    assert_eq!(album_xml(&live), before);
    // The user page keeps its own code for the same rule (task 072d).
    let (status, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let causes: Vec<&str> = grants[0]["writes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["cause"].as_str().unwrap())
        .collect();
    assert_eq!(causes, ["live_value_invalid"; 6]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_app_that_the_grant_does_not_name_gets_404_and_no_wrapper() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (_, app_id, _) = paired(&live, "One").await;
    let read = live.app.feeds.path().join("album.xml");
    granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let (_, text) = add_app(&live, "Two").await;
    let two_signer = MemorySigner::generate().unwrap();
    let given = two_signer.clone();
    let two = tokio::task::spawn_blocking(move || {
        let text = PairingText::parse(&text).unwrap();
        Client::pair(&text, given, LIVE_ORIGIN)
    })
    .await
    .unwrap()
    .unwrap();
    let two = Arc::new(two);
    assert_eq!(on(&two, Client::shows).await.unwrap().len(), 0);
    let (_, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    let grant_id = grants[0]["id"].as_i64().unwrap();
    let result = on(&two, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Live)
    })
    .await;
    assert!(
        matches!(result, Err(ClientError::NotGranted)),
        "ADR 0110 item 19: an app that the grant does not name must get 404: {result:?}"
    );
    assert!(!album_xml(&live).contains("status=\"live\""));
    // The wrapper route gives 404 too, and no wrapper.
    let route = format!("/app/v1/shows/{grant_id}/wrapper");
    let header = header_of(&two_signer, &signed(&live, "GET", &route, b""));
    let (status, body, text) = raw(&live, "GET", &route, Some(header), b"").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert!(!text.contains("ciphertext"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_granted_change_with_a_changed_draft_is_refused_and_listed_with_the_feed_state() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let grant_id = on(&client, Client::shows).await.unwrap()[0].grant_id;
    // No draft change: the feed state has no notice.
    let (_, draft) = call(&live.app, "GET", &format!("/api/v1/feeds/{id}/draft"), None).await;
    assert_eq!(draft["grant_blocked_by_draft"], Value::Null);
    let mut model = draft["model"].clone();
    model["title"] = json!("Album (Deluxe)");
    let (status, edited) = call(
        &live.app,
        "PUT",
        &format!("/api/v1/feeds/{id}/draft"),
        Some(json!({ "revision": draft["revision"], "model": model })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let block = &edited["grant_blocked_by_draft"];
    assert_eq!(
        block["state"], "grant_blocked_by_draft",
        "ADR 0110 item 42: the feed state must give the notice. Call grant_blocked_by_draft in draft_json."
    );
    assert_eq!(block["app_name"], "Show tool");
    assert_eq!(block["show_title"], "Show");
    assert_eq!(block["start"], "2099-11-01T10:00:00Z");
    let before = album_xml(&live);
    let result = on(&client, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Live)
    })
    .await;
    assert!(matches!(
        result,
        Err(ClientError::Other { status: 409, .. })
    ));
    assert_eq!(album_xml(&live), before);
    let (_, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    assert_eq!(grants[0]["writes"][0]["cause"], "draft_with_changes");
    assert_eq!(grants[0]["writes"][0]["app_name"], "Show tool");
    assert_eq!(
        draft_model(&live.app, id).await["title"],
        "Album (Deluxe)",
        "the draft of the user must stay"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopping_a_grant_ends_it_for_the_connected_app() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    let guid = granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let grant_id = on(&client, Client::shows).await.unwrap()[0].grant_id;
    let (status, body) = call(
        &live.app,
        "POST",
        &format!("/api/v1/feeds/{id}/grants/{guid}/end"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = on(&client, move |c| {
        c.set_status(grant_id, studio_app_client::ShowStatus::Live)
    })
    .await;
    assert!(
        matches!(
            result,
            Err(ClientError::NotGranted | ClientError::Other { status: 409, .. })
        ),
        "{result:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_removal_of_an_app_ends_its_grants_and_the_app_gets_401() {
    let live = live_app().await;
    let id = set_up(&live.app).await;
    let (client, app_id, _) = paired(&live, "Show tool").await;
    let read = live.app.feeds.path().join("album.xml");
    granted_show(&live.app, id, app_id, &|| {
        std::fs::read_to_string(&read).unwrap()
    })
    .await;
    let (status, _) = call(&live.app, "DELETE", &format!("/api/v1/apps/{app_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let result = on(&client, Client::whoami).await;
    assert!(
        matches!(result, Err(ClientError::SignatureInvalid)),
        "ADR 0110 item 21: a removed app must get 401: {result:?}"
    );
    let (_, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    assert!(grants[0]["ended_at"].is_i64());
}

// ---- Grant vaults ----

struct SftpShow {
    root: std::path::PathBuf,
    connector: i64,
    client: Arc<Client>,
    signer: MemorySigner,
    app_id: i64,
    grant: i64,
}

async fn sftp_show(
    live: &Live,
    server: &mut studio_core::testing::sshd::TestSshd,
) -> (i64, SftpShow) {
    let (client, app_id, signer) = paired(live, "Show tool").await;
    let mut body = sftp_body(server);
    let root = std::path::PathBuf::from(body["root"].as_str().unwrap());
    std::fs::write(root.join("album.xml"), FEED).unwrap();
    body["base_url"] = json!("https://cdn.example.com/feeds");
    let (status, connector) = call(&live.app, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{connector}");
    let (status, feed) = call(
        &live.app,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector["id"], "path": "album.xml" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{feed}");
    set_library_root(&live.app);
    let id = feed["id"].as_i64().unwrap();
    let read = root.clone();
    granted_show(&live.app, id, app_id, &move || {
        std::fs::read_to_string(read.join("album.xml")).unwrap()
    })
    .await;
    let (_, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    (
        id,
        SftpShow {
            root,
            connector: connector["id"].as_i64().unwrap(),
            client,
            signer,
            app_id,
            grant: grants[0]["id"].as_i64().unwrap(),
        },
    )
}

fn grant_files(app: &App) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(app.data.path().join("grants"))
        .map(|dir| {
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

async fn unlock(app: &App) {
    let (status, body) = call(
        app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASSPHRASE })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn live_status(show: &SftpShow, status: &'static str) -> Result<(), ClientError> {
    let grant = show.grant;
    let status = if status == "live" {
        studio_app_client::ShowStatus::Live
    } else {
        studio_app_client::ShowStatus::Ended
    };
    on(&show.client, move |c| c.set_status(grant, status))
        .await
        .map(|_| ())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_connected_app_writes_through_the_grant_vault_while_the_user_vault_is_locked() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (_, show) = sftp_show(&live, &mut server).await;
    assert_eq!(grant_files(&live.app), [format!("{}.json", show.grant)]);
    lock(&live.app).await;
    let host_xml = || std::fs::read_to_string(show.root.join("album.xml")).unwrap();
    live_status(&show, "live").await.unwrap();
    assert!(host_xml().contains("status=\"live\""));
    let grant = show.grant;
    on(&show.client, move |c| {
        c.set_live_value(grant, "wss://example.com/value", "socket.io")
    })
    .await
    .unwrap();
    assert!(host_xml().contains("uri=\"wss://example.com/value\""));
    assert_eq!(grant_files(&live.app), [format!("{}.json", show.grant)]);
    live_status(&show, "ended").await.unwrap();
    assert!(host_xml().contains("status=\"ended\""));
    // After the end, the grant vault file is gone at once.
    assert_eq!(grant_files(&live.app), Vec::<String>::new());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_wrong_grant_vault_key_gives_423_and_the_list_has_the_cause() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (id, show) = sftp_show(&live, &mut server).await;
    // A signed request with a key that does not open the grant vault.
    let route = format!("/app/v1/shows/{}/status", show.grant);
    let body = br#"{"status":"live","grant_key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="}"#;
    let header = header_of(&show.signer, &signed(&live, "POST", &route, body));
    let (status, error, _) = raw(&live, "POST", &route, Some(header), body).await;
    assert_eq!(status, StatusCode::LOCKED, "{error}");
    assert_eq!(
        error["error"]["code"], "vault_locked",
        "ADR 0110 item 23: a wrong grant vault key must give 423 vault_locked"
    );
    assert!(
        !std::fs::read_to_string(show.root.join("album.xml"))
            .unwrap()
            .contains("status=\"live\"")
    );
    let (_, grants) = call(
        &live.app,
        "GET",
        &format!("/api/v1/feeds/{id}/grants"),
        None,
    )
    .await;
    assert_eq!(grants[0]["writes"][0]["cause"], "vault_not_open");
    // The right key still works.
    live_status(&show, "live").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_granted_write_does_not_use_the_open_user_vault() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (id, show) = sftp_show(&live, &mut server).await;
    // The user vault is open but has lost the secret of the host. In memory
    // only: the saved vault is not changed.
    let connector = show.connector;
    live.handle
        .with_workspace(move |w| {
            let secret_ref = w
                .collection
                .connector(studio_core::collection::ConnectorId(connector))
                .unwrap()
                .secret_ref
                .unwrap();
            assert!(w.vault.as_mut().unwrap().remove(&secret_ref));
        })
        .await;
    live_status(&show, "live").await.unwrap();
    assert!(
        std::fs::read_to_string(show.root.join("album.xml"))
            .unwrap()
            .contains("status=\"live\"")
    );
    let (status, _) = call(&live.app, "GET", &format!("/api/v1/feeds/{id}/diff"), None).await;
    assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_new_secret_of_the_host_reaches_the_grant_vault_after_update_connector() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (_, show) = sftp_show(&live, &mut server).await;
    let connector = show.connector;
    let set_secret = |secret: Option<String>| {
        let handle = live.handle.clone();
        async move {
            handle
                .with_workspace(move |w| {
                    let secret_ref = w
                        .collection
                        .connector(studio_core::collection::ConnectorId(connector))
                        .unwrap()
                        .secret_ref
                        .unwrap();
                    let vault = w.vault.as_mut().unwrap();
                    match secret {
                        Some(new) => {
                            vault.put(&secret_ref, new.as_bytes()).unwrap();
                            None
                        }
                        None => Some(
                            String::from_utf8(vault.get(&secret_ref).unwrap().unwrap().to_vec())
                                .unwrap(),
                        ),
                    }
                })
                .await
        }
    };
    let good = set_secret(None).await.unwrap();
    set_secret(Some(
        json!({ "private_key": "not a key", "passphrase": null }).to_string(),
    ))
    .await;
    let (status, _) = call(
        &live.app,
        "PUT",
        &format!("/api/v1/connectors/{connector}"),
        Some(json!({ "name": "Server 2" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(live_status(&show, "live").await.is_err());

    set_secret(Some(good)).await;
    let (status, _) = call(
        &live.app,
        "PUT",
        &format!("/api/v1/connectors/{connector}"),
        Some(json!({ "name": "Server 3" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    lock(&live.app).await;
    // The grant vault key changed, so the client reads the new wrapper after
    // the first refusal.
    let mut result = live_status(&show, "live").await;
    if result.is_err() {
        result = live_status(&show, "live").await;
    }
    assert!(
        result.is_ok(),
        "ADR 0058 item 7: update_connector must make the grant vault again with the new secret: {result:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_file_and_no_row_has_a_grant_vault_key_or_a_private_key() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (_, show) = sftp_show(&live, &mut server).await;
    let grant = show.grant;
    let key = live
        .handle
        .with_workspace(move |w| {
            let record = format!("grant-vault/{grant}");
            let bytes = w.vault.as_ref().unwrap().get(&record).unwrap().unwrap();
            String::from_utf8(bytes.to_vec()).unwrap()
        })
        .await;
    live_status(&show, "live").await.unwrap();
    let mut files = 0;
    let mut pending = vec![live.app.data.path().to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            assert!(
                !bytes.windows(key.len()).any(|w| w == key.as_bytes()),
                "ADR 0110 item 23: {} must not hold the grant vault key. Keep the key in the user vault only",
                path.display()
            );
            files += 1;
        }
    }
    assert!(files >= 3, "the scan found {files} files");
    // The private key of the app is not on the disk: only a hex scan of its
    // public key would find it, and Studio never sees the private key.
    let (_, list) = call(&live.app, "GET", "/api/v1/apps", None).await;
    assert!(list[0].get("secret").is_none() && list[0].get("private_key").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_app_lists_its_shows_while_the_user_vault_is_locked() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (_, show) = sftp_show(&live, &mut server).await;
    lock(&live.app).await;
    let shows = on(&show.client, Client::shows).await.unwrap();
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0].grant_id, show.grant);
    assert_eq!(shows[0].live_value_origin.as_deref(), Some(LIVE_ORIGIN));
    // The first write works, and then the list is the same.
    live_status(&show, "live").await.unwrap();
    unlock(&live.app).await;
    let shows = on(&show.client, Client::shows).await.unwrap();
    assert_eq!(shows.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn after_the_removal_of_an_app_and_an_unlock_its_grant_vault_key_opens_no_grant_vault() {
    let mut server = studio_core::testing::sshd::TestSshd::start();
    let live = live_app().await;
    let (_, show) = sftp_show(&live, &mut server).await;
    let grant = show.grant;
    let key = live
        .handle
        .with_workspace(move |w| {
            let bytes = w
                .vault
                .as_ref()
                .unwrap()
                .get(&format!("grant-vault/{grant}"))
                .unwrap()
                .unwrap();
            String::from_utf8(bytes.to_vec()).unwrap()
        })
        .await;
    let path = live
        .app
        .data
        .path()
        .join("grants")
        .join(format!("{grant}.json"));
    let file = studio_core::vault::VaultFile::load(&path).unwrap();
    assert!(studio_core::grant_vault::open_with_key(file, &key).is_ok());
    let (status, _) = call(
        &live.app,
        "DELETE",
        &format!("/api/v1/apps/{}", show.app_id),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    lock(&live.app).await;
    unlock(&live.app).await;
    let opens = studio_core::vault::VaultFile::load(&path)
        .is_ok_and(|file| studio_core::grant_vault::open_with_key(file, &key).is_ok());
    assert!(
        !opens,
        "ADR 0110 item 21: the old grant vault key must open no current grant vault. Remove the grant vault of an ended grant"
    );
}

// ---- Logs ----

#[test]
fn no_log_line_of_the_app_routes_holds_a_body_a_header_a_pairing_text_or_a_key() {
    // The API has no log capture. The code logs with `eprintln!` only. This
    // test reads the source of the app layer and its routes, and refuses a
    // log macro there (ADR 0110 item 17). The manual check for the rest:
    // `grep -n "eprintln!\|println!\|dbg!" crates/api/src crates/server/src`
    // must show no argument that is a body, an Authorization header, an
    // event, a pairing text or a grant vault key.
    let source = include_str!("../routes.rs");
    let start = source.find("fn app_json").unwrap();
    let end = source
        .find("// ---- The NIP-46 signer of the user")
        .unwrap();
    let section = &source[start..end];
    for macro_name in ["println!", "eprintln!", "dbg!", "tracing::", "log::"] {
        assert!(
            !section.contains(macro_name),
            "ADR 0110 item 17: the routes of the connected apps must not log ({macro_name}). Return an error code and write nothing to a log."
        );
    }
}
