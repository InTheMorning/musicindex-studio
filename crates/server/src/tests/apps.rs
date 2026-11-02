//! Tests of the `/app/v1/` layer of the server (task 073d, ADR 0110). The
//! tests use `studio-app-client` as their client (ADR 0110 item 30). The
//! server runs on a loopback port, so the client sends real HTTP requests.
//! No test sends a request to a host outside the computer.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use base64::Engine as _;
use sha2::{Digest, Sha256};
use studio_app_client::nostr::prelude::Tag;
use studio_app_client::{AppSigner, Client, Error as ClientError, MemorySigner, PairingText};

use super::*;

const LIVE_ORIGIN: &str = "wss://example.com";
const FEED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
    <title>Album</title><podcast:guid>g-1</podcast:guid><podcast:medium>music</podcast:medium>\
    <item><guid>t-1</guid></item></channel></rss>";

/// A server on a loopback port. `app` has the router for requests in
/// memory, with the same origin.
struct Live {
    app: App,
    origin: String,
    host: String,
}

async fn live_server(signup: Signup, rules: PublicRules) -> Live {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let origin = format!("http://{addr}");
    let app = app_with_rules(signup, rules, |options| {
        options.origin = Origin::parse(&origin).unwrap();
        options.listen = addr;
    });
    let service = crate::auth::server_router(&app.state)
        .into_make_service_with_connect_info::<std::net::SocketAddr>();
    tokio::spawn(async move {
        axum::serve(listener, service).await.unwrap();
    });
    Live {
        app,
        origin,
        host: addr.to_string(),
    }
}

/// A call for the origin of the live server.
fn lcall<'a>(live: &'a Live, method: &'a str, path: &'a str, body: Option<Value>) -> Call<'a> {
    let mut c = call(method, path, body);
    c.host = &live.host;
    c.origin = (method != "GET").then_some(live.origin.as_str());
    c
}

async fn lapi(
    live: &Live,
    who: &(String, String),
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Reply {
    let mut c = lcall(live, method, path, body);
    c.token = Some(&who.0);
    c.csrf = Some(&who.1);
    send(&live.app, c).await
}

async fn on<T: Send + 'static>(
    client: &Arc<Client>,
    work: impl FnOnce(&Client) -> T + Send + 'static,
) -> T {
    let client = Arc::clone(client);
    tokio::task::spawn_blocking(move || work(&client))
        .await
        .unwrap()
}

/// A new user of the live server (the sign-up mode is open).
async fn live_user(live: &Live, email: &str, passphrase: &str) -> (String, String) {
    let body = json!({ "email": email, "passphrase": passphrase, "invite": null });
    let reply = send(
        &live.app,
        lcall(live, "POST", "/auth/v1/signup/email", Some(body)),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.json);
    (
        reply.token(),
        reply.json["csrf"].as_str().unwrap().to_owned(),
    )
}

/// "Connect an App" for a user. Gives the ID and the pairing text.
async fn add_app(live: &Live, who: &(String, String), name: &str) -> (i64, String) {
    let reply = lapi(
        live,
        who,
        "POST",
        "/api/v1/apps",
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.json);
    (
        reply.json["id"].as_i64().unwrap(),
        reply.json["pairing_text"].as_str().unwrap().to_owned(),
    )
}

async fn pair_with(text: String, signer: MemorySigner) -> Result<Client, ClientError> {
    tokio::task::spawn_blocking(move || {
        let text = PairingText::parse(&text).unwrap();
        Client::pair(&text, signer, LIVE_ORIGIN)
    })
    .await
    .unwrap()
}

/// A user with an app that paired.
async fn user_with_app(
    live: &Live,
    email: &str,
) -> ((String, String), i64, Arc<Client>, MemorySigner) {
    let who = live_user(live, email, PASS_ONE).await;
    let (id, text) = add_app(live, &who, "Show tool").await;
    let signer = MemorySigner::generate().unwrap();
    let client = pair_with(text, signer.clone()).await.expect("a pairing");
    (who, id, Arc::new(client), signer)
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

/// The header `Nostr <base64 event>` of a request that the test signs.
fn header_of(live: &Live, signer: &MemorySigner, method: &str, path: &str, body: &[u8]) -> String {
    let tag = |name: &str, value: &str| Tag::parse([name, value]).unwrap();
    let mut tags = vec![
        tag("u", &format!("{}{path}", live.origin)),
        tag("method", method),
    ];
    if !body.is_empty() {
        tags.push(tag("payload", &hex_sha256(body)));
    }
    let mut nonce = [0_u8; 8];
    getrandom::fill(&mut nonce).unwrap();
    tags.push(tag("nonce", &format!("{nonce:?}")));
    let now = crate::accounts::now().unsigned_abs();
    let event = signer.sign(27_235, tags, now).unwrap();
    format!(
        "Nostr {}",
        base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&event).unwrap())
    )
}

async fn app_request(
    live: &Live,
    signer: &MemorySigner,
    method: &'static str,
    path: &str,
    addr: &'static str,
) -> Reply {
    let header = header_of(live, signer, method, path, b"");
    let mut c = lcall(live, method, path, None);
    c.origin = None;
    c.addr = addr;
    c.extra = vec![("authorization", header.as_str())];
    send(&live.app, c).await
}

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

/// A feed on an SFTP host with a pending show and its grant for the app.
/// Gives the feed ID, the folder of the host and the grant ID.
async fn granted_show(
    live: &Live,
    who: &(String, String),
    server: &mut studio_core::testing::sshd::TestSshd,
    app_id: i64,
) -> (i64, std::path::PathBuf, i64) {
    let mut body = sftp_body(server);
    let root = std::path::PathBuf::from(body["root"].as_str().unwrap());
    std::fs::write(root.join("album.xml"), FEED).unwrap();
    body["base_url"] = json!("https://cdn.example.com/feeds");
    let connector = lapi(live, who, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(
        connector.status,
        StatusCode::CREATED,
        "{:?}",
        connector.json
    );
    let feed = lapi(
        live,
        who,
        "POST",
        "/api/v1/feeds",
        Some(json!({ "connector_id": connector.json["id"], "path": "album.xml" })),
    )
    .await;
    assert_eq!(feed.status, StatusCode::CREATED, "{:?}", feed.json);
    let id = feed.json["id"].as_i64().unwrap();
    let base = format!("/api/v1/feeds/{id}");
    let draft = lapi(
        live,
        who,
        "POST",
        &format!("{base}/live-items"),
        Some(json!({ "title": "Show", "start": "2099-11-01T10:00:00Z" })),
    )
    .await;
    assert_eq!(draft.status, StatusCode::OK, "{:?}", draft.json);
    let guid = draft.json["model"]["live_items"][0]["item"]["guid"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut model = draft.json["model"].clone();
    model["live_items"][0]["content_links"] =
        json!([{ "href": "https://example.com/show", "label": "Watch" }]);
    let saved = lapi(
        live,
        who,
        "PUT",
        &format!("{base}/draft"),
        Some(json!({ "revision": draft.json["revision"], "model": model })),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK, "{:?}", saved.json);
    let diff = lapi(
        live,
        who,
        "PUT",
        &format!("{base}/grants/{guid}"),
        Some(json!({ "app_id": app_id })),
    )
    .await;
    assert_eq!(diff.status, StatusCode::OK, "{:?}", diff.json);
    let written = lapi(
        live,
        who,
        "POST",
        &format!("{base}/write"),
        Some(json!({ "diff_id": diff.json["id"] })),
    )
    .await;
    assert_eq!(written.status, StatusCode::OK, "{:?}", written.json);
    let grants = lapi(live, who, "GET", &format!("{base}/grants"), None).await;
    (id, root, grants.json[0]["id"].as_i64().unwrap())
}

fn rules_for(server: &studio_core::testing::sshd::TestSshd) -> PublicRules {
    PublicRules::with_test_exceptions(vec![std::net::SocketAddr::from((
        [127, 0, 0, 1],
        server.port(),
    ))])
}

// ---- Connect an app ----

#[tokio::test]
async fn connect_an_app_needs_an_unlock_in_the_last_five_minutes() {
    let app = app(Signup::Invite);
    let offset = clock_of(&app);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let ok = api(
        &app,
        &who,
        "POST",
        "/api/v1/apps",
        Some(json!({ "name": "One" })),
    )
    .await;
    assert_eq!(ok.status, StatusCode::CREATED, "{:?}", ok.json);
    offset.store(6 * 60, Ordering::SeqCst);
    let late = api(
        &app,
        &who,
        "POST",
        "/api/v1/apps",
        Some(json!({ "name": "Two" })),
    )
    .await;
    assert_eq!(
        late.status,
        StatusCode::UNAUTHORIZED,
        "ADR 0110 item 5 and ADR 0106 item 2: Connect an App must need an unlock in the last 5 minutes. Call require_recent_unlock in add_app."
    );
    assert_eq!(late.code(), "unlock_required");
    // The list and the removal do not need it.
    let list = api(&app, &who, "GET", "/api/v1/apps", None).await;
    assert_eq!(list.status, StatusCode::OK);
}

// ---- Keys ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pairing_with_a_key_that_is_active_in_another_account_gives_409_app_key_in_use() {
    let live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (_, _, _, signer) = user_with_app(&live, "one@example.org").await;
    let two = live_user(&live, "two@example.org", PASS_TWO).await;
    let (_, text) = add_app(&live, &two, "Other").await;
    let result = pair_with(text.clone(), signer).await;
    assert!(
        matches!(result, Err(ClientError::KeyInUse)),
        "ADR 0110 item 11: a key that is active in a different account must give 409 app_key_in_use. Claim the key in the index in one transaction: {result:?}"
    );
    // The refused pairing keeps no claim and does not use the code.
    let again = pair_with(text, MemorySigner::generate().unwrap()).await;
    assert!(again.is_ok(), "{again:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_sign_in_key_of_an_account_cannot_be_an_app_key() {
    let live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let keys = new_keys();
    // The sign-in with the key makes the account on the live server. The
    // challenge and the event use the origin of the live server.
    let challenge = send(&live.app, lcall(&live, "POST", "/auth/v1/challenge", None))
        .await
        .json["challenge"]
        .as_str()
        .unwrap()
        .to_owned();
    let url = format!("{}/auth/v1/nostr", live.origin);
    let event = signed_event(
        &keys,
        &[["u", &url], ["method", "POST"], ["challenge", &challenge]],
        now_secs(),
    );
    let up = send(
        &live.app,
        lcall(
            &live,
            "POST",
            "/auth/v1/nostr",
            Some(json!({ "event": event, "invite": null })),
        ),
    )
    .await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    let who = who_of(&up);
    let (_, text) = add_app(&live, &who, "Show tool").await;
    let signer = MemorySigner::from_secret_key(keys.secret_key().clone());
    let result = pair_with(text, signer).await;
    assert!(
        matches!(&result, Err(ClientError::Other { status: 400, code }) if code == "app_key_is_sign_in_key"),
        "ADR 0110 item 4: the sign-in key of an account must give 400 app_key_is_sign_in_key: {result:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_token_gets_app_key_required_and_an_unknown_key_gets_401() {
    let live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (_, _, _, _) = user_with_app(&live, "one@example.org").await;
    let mut c = lcall(&live, "GET", "/app/v1/whoami", None);
    c.origin = None;
    c.extra = vec![("authorization", "Bearer an-old-token-of-64-hex-characters")];
    let reply = send(&live.app, c).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        reply.code(),
        "app_key_required",
        "ADR 0110 item 37: an app with a token must get app_key_required"
    );
    // A key that Studio never saw: the key that HKDF derived from an old
    // token is such a key, because the migration cleared it (item 38).
    let stranger = MemorySigner::generate().unwrap();
    let reply = app_request(&live, &stranger, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.code(), "app_signature_invalid");
}

#[test]
fn the_migration_removes_the_token_hashes_of_the_index() {
    let data = tempfile::tempdir().unwrap();
    {
        let db = rusqlite::Connection::open(data.path().join("server.sqlite")).unwrap();
        // The schema before task 073: version 7 with the token index.
        for migration in crate::accounts::migrations_for_test().iter().take(7) {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", 7).unwrap();
        db.execute(
            "INSERT INTO app_tokens (token_hash, account_id, app_id) VALUES ('abc', 1, 1)",
            [],
        )
        .unwrap();
    }
    let accounts = crate::accounts::Accounts::open(data.path()).unwrap();
    drop(accounts);
    let db = rusqlite::Connection::open(data.path().join("server.sqlite")).unwrap();
    let tokens: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'app_tokens'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        tokens, 0,
        "ADR 0110 item 37: a migration must remove the index of the token hashes"
    );
}

// ---- Layers ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_app_layer_refuses_a_cookie_a_session_route_and_a_wrong_host() {
    let live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (who, _, _, signer) = user_with_app(&live, "one@example.org").await;
    // A session cookie opens no app route.
    let mut c = lcall(&live, "GET", "/app/v1/whoami", None);
    c.token = Some(&who.0);
    c.csrf = Some(&who.1);
    let reply = send(&live.app, c).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        reply.code(),
        "app_signature_invalid",
        "ADR 0110 item 16: /app/v1/ accepts no other credential than the signed request"
    );
    // The signed request opens no session route.
    for path in ["/api/v1/status", "/auth/v1/session"] {
        let header = header_of(&live, &signer, "GET", path, b"");
        let mut c = lcall(&live, "GET", path, None);
        c.extra = vec![("authorization", header.as_str())];
        let reply = send(&live.app, c).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(
            reply.code(),
            "app_credential_refused",
            "ADR 0110 item 16: {path} must refuse the Nostr header. Check it in the outer layer."
        );
    }
    // A wrong host gets the refusal of the outer layer.
    let header = header_of(&live, &signer, "GET", "/app/v1/whoami", b"");
    let mut c = call("GET", "/app/v1/whoami", None);
    c.host = "evil.example";
    c.extra = vec![("authorization", header.as_str())];
    let reply = send(&live.app, c).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.code(), "host_not_allowed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_61st_request_of_an_app_key_and_the_21st_bad_request_of_an_address_give_429() {
    let live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (_, _, _, signer) = user_with_app(&live, "one@example.org").await;
    // The pairing request counted one for the key.
    for n in 1..crate::limits::APP_LIMIT {
        let reply = app_request(&live, &signer, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "request {n}: {:?}",
            reply.json
        );
    }
    let over = app_request(&live, &signer, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
    assert_eq!(
        over.status,
        StatusCode::TOO_MANY_REQUESTS,
        "ADR 0110 item 40: the 61st request of an app key in a minute must give 429. Call allow_app with the public key."
    );
    assert_eq!(over.code(), "too_many_attempts");

    let stranger = MemorySigner::generate().unwrap();
    for n in 1..=crate::limits::UNKNOWN_TOKEN_LIMIT {
        let reply = app_request(&live, &stranger, "GET", "/app/v1/whoami", "10.0.0.9:4000").await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "bad request {n}");
    }
    let blocked = app_request(&live, &stranger, "GET", "/app/v1/whoami", "10.0.0.9:4000").await;
    assert_eq!(
        blocked.status,
        StatusCode::TOO_MANY_REQUESTS,
        "ADR 0110 item 40: the 21st bad request of an address in an hour must give 429. Call record_unknown for each 401."
    );
    // Another address still passes the layer and gets 401.
    let other = app_request(&live, &stranger, "GET", "/app/v1/whoami", "10.0.0.10:4000").await;
    assert_eq!(other.status, StatusCode::UNAUTHORIZED);
}

// ---- Shows ----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_app_goes_live_through_the_public_origin_after_the_user_signed_out() {
    let mut sshd = studio_core::testing::sshd::TestSshd::start();
    let live = live_server(Signup::Open, rules_for(&sshd)).await;
    live.app
        .state
        .users
        .pass_all_addresses
        .store(true, Ordering::SeqCst);
    let (who, app_id, client, _) = user_with_app(&live, "one@example.org").await;
    let (_, root, grant) = granted_show(&live, &who, &mut sshd, app_id).await;
    let out = send(&live.app, {
        let mut c = sign_out_call(&who);
        c.host = &live.host;
        c.origin = Some(&live.origin);
        c
    })
    .await;
    assert_eq!(out.status, StatusCode::OK);

    let shows = on(&client, Client::shows).await.unwrap();
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0].grant_id, grant);
    let result = on(&client, move |c| {
        c.set_status(grant, studio_app_client::ShowStatus::Live)
    })
    .await
    .unwrap();
    assert_eq!(result.status.as_deref(), Some("live"));
    assert!(
        std::fs::read_to_string(root.join("album.xml"))
            .unwrap()
            .contains("status=\"live\""),
        "ADR 0058: a granted write must work with the user signed out"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_app_of_one_user_cannot_see_the_shows_of_another_user() {
    let mut sshd = studio_core::testing::sshd::TestSshd::start();
    let live = live_server(Signup::Open, rules_for(&sshd)).await;
    live.app
        .state
        .users
        .pass_all_addresses
        .store(true, Ordering::SeqCst);
    let (one, one_app, _, _) = user_with_app(&live, "one@example.org").await;
    let (_, _, two_client, _) = user_with_app(&live, "two@example.org").await;
    let (_, root, grant) = granted_show(&live, &one, &mut sshd, one_app).await;
    assert_eq!(on(&two_client, Client::shows).await.unwrap().len(), 0);
    let result = on(&two_client, move |c| {
        c.set_status(grant, studio_app_client::ShowStatus::Live)
    })
    .await;
    assert!(
        matches!(result, Err(ClientError::NotGranted)),
        "ADR 0110 item 19: the app of another user must get 404: {result:?}"
    );
    assert!(
        !std::fs::read_to_string(root.join("album.xml"))
            .unwrap()
            .contains("status=\"live\"")
    );
}

// ---- The index ----

fn key_rows(app: &App) -> i64 {
    let db = rusqlite::Connection::open(app.data.path().join("server.sqlite")).unwrap();
    db.query_row(
        "SELECT (SELECT COUNT(*) FROM app_keys WHERE app_id <> 0)
              + (SELECT COUNT(*) FROM app_codes)",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_rows_follow_the_user_and_come_back_after_a_restart() {
    let mut live = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (who, app_id, _, signer) = user_with_app(&live, "one@example.org").await;
    // The key row and the code row exist.
    assert_eq!(key_rows(&live.app), 2);
    // A lost row does not stop the app: the server makes the index again at
    // the start.
    let db = rusqlite::Connection::open(live.app.data.path().join("server.sqlite")).unwrap();
    db.execute("DELETE FROM app_keys", []).unwrap();
    db.execute("DELETE FROM app_codes", []).unwrap();
    drop(db);
    let reply = app_request(&live, &signer, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    live.app = restart(live.app);
    assert_eq!(
        key_rows(&live.app),
        2,
        "ADR 0110 item 39: the server must make the index of the keys and the codes again at each start. Call rebuild_app_index in make_state."
    );
    let reply = app_request(&live, &signer, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    // The removal of the app removes its rows.
    let removed = lapi(
        &live,
        &who,
        "DELETE",
        &format!("/api/v1/apps/{app_id}"),
        None,
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(key_rows(&live.app), 0);
    let reply = app_request(&live, &signer, "GET", "/app/v1/whoami", "10.0.0.1:4000").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn after_an_import_each_app_is_connect_again_and_the_account_has_no_grant() {
    let one = live_server(Signup::Open, PublicRules::with_test_exceptions(Vec::new())).await;
    let (who, app_id, _, signer) = user_with_app(&one, "heycitizen@example.org").await;
    add_feed_to(&one.app, "heycitizen@example.org");
    let collection = user_collection(&one.app, "heycitizen@example.org");
    let feed = collection.feeds().unwrap()[0].id;
    studio_core::grant::record(&collection, feed, "guid-1", app_id, LIVE_ORIGIN, 1).unwrap();
    drop(collection);
    let archive = {
        let ticket = lapi(
            &one,
            &who,
            "POST",
            "/api/v1/account/export",
            Some(json!({})),
        )
        .await;
        let path = format!(
            "/media/download?ticket={}",
            ticket.json["ticket"].as_str().unwrap()
        );
        let mut c = lcall(&one, "GET", &path, None);
        c.token = Some(&who.0);
        send_bytes(&one.app, c).await.1
    };

    let two = app(Signup::Open);
    let (_, upload) = import_upload(&two, &archive, None).await;
    assert_eq!(import_check(&two, &upload).await.status, StatusCode::OK);
    let opened = import_open(&two, &upload, json!({ "passphrase": PASS_ONE })).await;
    assert_eq!(opened.status, StatusCode::CREATED, "{:?}", opened.json);
    let who_two = who_of(&opened);
    let list = api(&two, &who_two, "GET", "/api/v1/apps", None).await;
    assert_eq!(list.json[0]["name"], "Show tool");
    assert_eq!(
        list.json[0]["state"], "token",
        "ADR 0110 item 41: an imported app must show Connect Again. Call reset_apps_after_import in the import."
    );
    assert_eq!(list.json[0]["short_key"], Value::Null);
    let facts = api(&two, &who_two, "GET", "/api/v1/feeds/1/grants", None).await;
    assert!(
        facts.json[0]["ended_at"].is_i64(),
        "ADR 0110 item 41: an import must keep no grant: {:?}",
        facts.json
    );
    // The index of the new server holds no key of the archive.
    assert!(
        two.state
            .accounts
            .app_key_account(&signer.public_key().to_hex())
            .unwrap()
            .is_none()
    );
}
