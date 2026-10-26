//! Tests of the accounts, the sessions and the layers (task 034c). Each
//! test uses the router in memory and a temporary data folder. No test sends
//! a request to a host outside the computer.

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use studio_core::net::PublicRules;
use tower::ServiceExt;

use crate::config::{Origin, ServeOptions, Signup};
use crate::session::Shared;

const ORIGIN: &str = "http://127.0.0.1:8080";
const HOST: &str = "127.0.0.1:8080";
const PASS_ONE: &str = "correct horse battery";
const PASS_TWO: &str = "staple orange umbrella river";

struct App {
    router: Router,
    state: Shared,
    data: tempfile::TempDir,
}

fn app(signup: Signup) -> App {
    app_with(signup, |_| {})
}

fn app_with(signup: Signup, tweak: impl FnOnce(&mut ServeOptions)) -> App {
    let data = tempfile::tempdir().unwrap();
    let mut options = ServeOptions {
        data_dir: data.path().to_path_buf(),
        listen: "127.0.0.1:8080".parse().unwrap(),
        origin: Origin::parse(ORIGIN).unwrap(),
        signup,
        client_address_header: None,
        session_idle: 7 * 86_400,
        session_max: 30 * 86_400,
        vault_idle: std::time::Duration::from_secs(600),
        storage_limit: 10 * 1024 * 1024 * 1024,
        argon2_concurrency: 2,
        argon2_wait: std::time::Duration::from_secs(10),
        // Port 9 answers nothing, so no test reaches a real host.
        musicindex: Some("http://127.0.0.1:9".to_owned()),
        hub: Some("http://127.0.0.1:9/pubnotify".to_owned()),
    };
    tweak(&mut options);
    let state = crate::make_state(&options, PublicRules::with_test_exceptions(Vec::new())).unwrap();
    App {
        router: crate::auth::server_router(&state),
        state,
        data,
    }
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
    text: String,
}

impl Reply {
    fn code(&self) -> &str {
        self.json["error"]["code"].as_str().unwrap_or("")
    }

    fn message(&self) -> &str {
        self.json["error"]["message"].as_str().unwrap_or("")
    }

    /// The session token in the cookie of the response.
    fn token(&self) -> String {
        let cookie = self
            .headers
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        cookie
            .strip_prefix("__Host-studio-session=")
            .and_then(|rest| rest.split(';').next())
            .unwrap()
            .to_owned()
    }
}

struct Call<'a> {
    method: &'a str,
    path: &'a str,
    body: Option<Value>,
    host: &'a str,
    origin: Option<&'a str>,
    token: Option<&'a str>,
    csrf: Option<&'a str>,
    content_type: &'a str,
    /// The address of the connection.
    addr: &'a str,
    /// More headers.
    extra: Vec<(&'a str, &'a str)>,
}

fn call<'a>(method: &'a str, path: &'a str, body: Option<Value>) -> Call<'a> {
    Call {
        method,
        path,
        body,
        host: HOST,
        origin: (method != "GET").then_some(ORIGIN),
        token: None,
        csrf: None,
        content_type: "application/json",
        addr: "10.0.0.1:4000",
        extra: Vec::new(),
    }
}

async fn send(app: &App, call: Call<'_>) -> Reply {
    let mut builder = Request::builder()
        .method(call.method)
        .uri(call.path)
        .header(header::HOST, call.host);
    if let Some(origin) = call.origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    if let Some(token) = call.token {
        builder = builder.header(header::COOKIE, format!("__Host-studio-session={token}"));
    }
    if let Some(csrf) = call.csrf {
        builder = builder.header("x-studio-csrf", csrf);
    }
    for (name, value) in &call.extra {
        builder = builder.header(*name, *value);
    }
    let body = match call.body {
        Some(body) => {
            builder = builder.header(header::CONTENT_TYPE, call.content_type);
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let mut request = builder.body(body).unwrap();
    request.extensions_mut().insert(axum::extract::ConnectInfo(
        call.addr.parse::<std::net::SocketAddr>().unwrap(),
    ));
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        text: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn sign_up(app: &App, email: &str, passphrase: &str, invite: Option<&str>) -> Reply {
    let body = json!({ "email": email, "passphrase": passphrase, "invite": invite });
    send(app, call("POST", "/auth/v1/signup/email", Some(body))).await
}

/// Sign up with an invite code, and give the token and the CSRF token.
async fn new_user(app: &App, email: &str, passphrase: &str) -> (String, String) {
    let invite = app.state.accounts.create_invite().unwrap();
    let reply = sign_up(app, email, passphrase, Some(&invite)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.json);
    (
        reply.token(),
        reply.json["csrf"].as_str().unwrap().to_owned(),
    )
}

async fn api(
    app: &App,
    who: &(String, String),
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Reply {
    let mut c = call(method, path, body);
    c.token = Some(&who.0);
    c.csrf = Some(&who.1);
    send(app, c).await
}

#[tokio::test]
async fn two_users_do_not_see_the_feeds_of_each_other() {
    let app = app(Signup::Invite);
    let one = new_user(&app, "one@example.org", PASS_ONE).await;
    let two = new_user(&app, "two@example.org", PASS_TWO).await;
    let added = api(&app, &one, "POST", "/api/v1/feeds", Some(json!({}))).await;
    assert_eq!(added.status, StatusCode::CREATED);
    let id = added.json["id"].as_i64().unwrap();
    let own = api(&app, &one, "GET", "/api/v1/feeds", None).await;
    assert_eq!(own.json.as_array().unwrap().len(), 1);
    let other = api(&app, &two, "GET", "/api/v1/feeds", None).await;
    assert_eq!(other.status, StatusCode::OK);
    assert_eq!(other.json.as_array().unwrap().len(), 0);
    let hidden = api(
        &app,
        &two,
        "GET",
        &format!("/api/v1/feeds/{id}/snapshots"),
        None,
    )
    .await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND, "{:?}", hidden.text);
    let gone = api(&app, &two, "DELETE", &format!("/api/v1/feeds/{id}"), None).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let still = api(&app, &one, "GET", "/api/v1/feeds", None).await;
    assert_eq!(still.json.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_new_user_has_a_folder_with_a_library_and_an_open_vault() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "One@Example.org ", PASS_ONE).await;
    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.status, StatusCode::OK);
    assert_eq!(status.json["vault"], "open");
    let folders = app.state.accounts.folders().unwrap();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].1.len(), 32);
    let folder = app.data.path().join("users").join(&folders[0].1);
    assert!(folder.join("vault.json").exists());
    let library = api(&app, &who, "GET", "/api/v1/library", None).await;
    assert!(
        library
            .json
            .to_string()
            .contains(&format!("{}/library", folder.display()))
    );
    let session = send(&app, {
        let mut c = call("GET", "/auth/v1/session", None);
        c.token = Some(&who.0);
        c
    })
    .await;
    assert_eq!(session.json["email"], "one@example.org");
    assert_eq!(session.json["signed_in"], true);
    assert_eq!(session.json["csrf"], who.1.as_str());
}

#[tokio::test]
async fn the_cookie_has_the_attributes_of_the_task() {
    let app = app(Signup::Open);
    let reply = sign_up(&app, "a@example.org", PASS_ONE, None).await;
    let cookie = reply
        .headers
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    let token = reply.token();
    assert_eq!(token.len(), 64);
    assert_eq!(
        cookie,
        format!(
            "__Host-studio-session={token}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=2592000"
        )
    );
}

#[tokio::test]
async fn the_api_layer_needs_a_session_and_the_csrf_header() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let none = send(&app, call("GET", "/api/v1/status", None)).await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
    assert_eq!(none.code(), "session_required");
    let mut no_header = call("GET", "/api/v1/status", None);
    no_header.token = Some(&who.0);
    let reply = send(&app, no_header).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.code(), "csrf_required");
    let mut wrong = call("GET", "/api/v1/status", None);
    wrong.token = Some(&who.0);
    wrong.csrf = Some("0000");
    assert_eq!(send(&app, wrong).await.code(), "csrf_required");
    let mut other_host = call("GET", "/api/v1/status", None);
    other_host.host = "evil.example.org";
    other_host.token = Some(&who.0);
    other_host.csrf = Some(&who.1);
    let reply = send(&app, other_host).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.code(), "host_not_allowed");
    let mut other_origin = call("POST", "/api/v1/vault/lock", None);
    other_origin.origin = Some("https://evil.example.org");
    other_origin.token = Some(&who.0);
    other_origin.csrf = Some(&who.1);
    let reply = send(&app, other_origin).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.code(), "origin_not_allowed");
    let mut no_origin = call("POST", "/api/v1/vault/lock", None);
    no_origin.origin = None;
    no_origin.token = Some(&who.0);
    no_origin.csrf = Some(&who.1);
    assert_eq!(send(&app, no_origin).await.code(), "origin_not_allowed");
    let mut app_route = call("GET", "/app/v1/whoami", None);
    app_route.token = Some(&who.0);
    assert_eq!(send(&app, app_route).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sign_up_needs_an_unused_invite_in_the_mode_invite() {
    let app = app(Signup::Invite);
    let none = sign_up(&app, "a@example.org", PASS_ONE, None).await;
    assert_eq!(none.status, StatusCode::FORBIDDEN);
    assert_eq!(none.code(), "invite_required");
    let wrong = sign_up(&app, "a@example.org", PASS_ONE, Some("abc")).await;
    assert_eq!(wrong.code(), "invite_required");
    let invite = app.state.accounts.create_invite().unwrap();
    let weak = sign_up(&app, "a@example.org", "short", Some(&invite)).await;
    assert_eq!(weak.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(weak.code(), "passphrase_too_short");
    let first = sign_up(&app, "a@example.org", PASS_ONE, Some(&invite)).await;
    assert_eq!(first.status, StatusCode::CREATED);
    let used = sign_up(&app, "b@example.org", PASS_ONE, Some(&invite)).await;
    assert_eq!(used.status, StatusCode::FORBIDDEN);
    assert_eq!(used.code(), "invite_required");
    let other = app.state.accounts.create_invite().unwrap();
    let taken = sign_up(&app, " A@example.org", PASS_ONE, Some(&other)).await;
    assert_eq!(taken.status, StatusCode::CONFLICT);
    assert_eq!(taken.code(), "email_in_use");
    let again = sign_up(&app, "b@example.org", PASS_ONE, Some(&other)).await;
    assert_eq!(again.status, StatusCode::CREATED);
}

#[tokio::test]
async fn sign_up_works_with_no_invite_in_the_mode_open() {
    let app = app(Signup::Open);
    let session = send(&app, call("GET", "/auth/v1/session", None)).await;
    assert_eq!(
        session.json,
        json!({ "mode": "server", "signup": "open", "signed_in": false })
    );
    let reply = sign_up(&app, "a@example.org", PASS_ONE, None).await;
    assert_eq!(reply.status, StatusCode::CREATED);
}

#[tokio::test]
async fn a_failed_sign_up_leaves_no_account_and_the_recovery_key_comes_once() {
    let app = app(Signup::Open);
    let weak = sign_up(&app, "a@example.org", "password1234", None).await;
    assert_eq!(weak.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(app.state.accounts.folders().unwrap().len(), 0);
    let body = json!({ "email": "a@example.org", "passphrase": PASS_ONE, "recovery_key": true });
    let reply = send(&app, call("POST", "/auth/v1/signup/email", Some(body))).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    assert!(reply.json["recovery_key"].is_string());
}

#[tokio::test]
async fn an_unknown_email_and_a_wrong_passphrase_give_the_same_error() {
    let app = app(Signup::Invite);
    new_user(&app, "a@example.org", PASS_ONE).await;
    let signin = |email: &str, pass: &str| {
        call(
            "POST",
            "/auth/v1/signin/email",
            Some(json!({ "email": email, "passphrase": pass })),
        )
    };
    let unknown = send(&app, signin("nobody@example.org", PASS_ONE)).await;
    let wrong = send(&app, signin("a@example.org", PASS_TWO)).await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.status, wrong.status);
    assert_eq!(unknown.code(), "sign_in_failed");
    assert_eq!(unknown.code(), wrong.code());
    assert_eq!(unknown.message(), wrong.message());
    let good = send(&app, signin(" A@Example.org", PASS_ONE)).await;
    assert_eq!(good.status, StatusCode::OK);
    assert_eq!(good.json["csrf"].as_str().unwrap().len(), 64);
    let who = (good.token(), good.json["csrf"].as_str().unwrap().to_owned());
    assert_eq!(
        api(&app, &who, "GET", "/api/v1/status", None).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn after_sign_out_the_cookie_gets_401() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let mut out = call("POST", "/auth/v1/signout", Some(json!({})));
    out.token = Some(&who.0);
    let reply = send(&app, out).await;
    let cookie = reply
        .headers
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("Max-Age=0"));
    let after = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
    assert_eq!(after.code(), "session_required");
}

#[tokio::test]
async fn a_session_idle_for_over_7_days_ends() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let old = accounts_now() - 8 * 86_400;
    let db = rusqlite::Connection::open(app.data.path().join("server.sqlite")).unwrap();
    db.execute("UPDATE sessions SET last_seen = ?1", [old])
        .unwrap();
    let ended = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(ended.status, StatusCode::UNAUTHORIZED);
    assert_eq!(ended.code(), "session_ended");
    let rows: i64 = db
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 0);
    let after = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(after.code(), "session_required");
}

#[tokio::test]
async fn a_session_older_than_30_days_ends() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let old = accounts_now() - 31 * 86_400;
    let db = rusqlite::Connection::open(app.data.path().join("server.sqlite")).unwrap();
    db.execute("UPDATE sessions SET created_at = ?1", [old])
        .unwrap();
    let ended = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(ended.code(), "session_ended");
}

fn accounts_now() -> i64 {
    crate::accounts::now()
}

#[tokio::test]
async fn a_body_with_a_different_content_type_gets_415() {
    let app = app(Signup::Open);
    let mut c = call(
        "POST",
        "/auth/v1/signup/email",
        Some(json!({ "email": "a@example.org", "passphrase": PASS_ONE })),
    );
    c.content_type = "text/plain";
    assert_eq!(
        send(&app, c).await.status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

#[tokio::test]
async fn server_sqlite_holds_no_token_no_invite_and_no_passphrase() {
    let app = app(Signup::Invite);
    let invite = app.state.accounts.create_invite().unwrap();
    let reply = sign_up(&app, "a@example.org", PASS_ONE, Some(&invite)).await;
    let token = reply.token();
    let csrf = reply.json["csrf"].as_str().unwrap().to_owned();
    let bytes = std::fs::read(app.data.path().join("server.sqlite")).unwrap();
    let has = |needle: &str| bytes.windows(needle.len()).any(|w| w == needle.as_bytes());
    assert!(has("a@example.org"));
    assert!(!has(&token));
    assert!(!has(&csrf));
    assert!(!has(&invite));
    assert!(!has(PASS_ONE));
}

#[tokio::test]
async fn the_ui_files_are_served_without_a_session() {
    let app = app(Signup::Invite);
    let root = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::HOST, HOST)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(root.status(), StatusCode::OK);
}

// ---- The vault window, the limits and the account deletion (task 034d) ----

fn s3_body() -> Value {
    json!({
        "kind": "s3",
        "name": "Bucket",
        "endpoint": "http://127.0.0.1:9",
        "region": "us-east-1",
        "bucket": "bk",
        "url_style": "path",
        "prefix": "feeds",
        "base_url": "https://cdn.example.com/feeds",
        "access_key_id": "KEYID",
        "secret_access_key": "SECRET",
    })
}

fn sign_in_call<'a>(email: &str, passphrase: &str) -> Call<'a> {
    call(
        "POST",
        "/auth/v1/signin/email",
        Some(json!({ "email": email, "passphrase": passphrase })),
    )
}

#[tokio::test]
async fn the_idle_task_locks_the_vault_after_the_idle_limit() {
    let app = app_with(Signup::Invite, |o| {
        o.vault_idle = std::time::Duration::from_secs(1);
    });
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
    crate::idle::sweep(&app.state).await;
    assert_eq!(
        api(&app, &who, "GET", "/api/v1/status", None).await.json["vault"],
        "open"
    );
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    crate::idle::sweep(&app.state).await;
    let reply = api(&app, &who, "POST", "/api/v1/connectors", Some(s3_body())).await;
    assert_eq!(reply.status, StatusCode::LOCKED, "{:?}", reply.json);
    assert_eq!(reply.code(), "vault_locked");
    // The session stays, and the unlock opens the vault again.
    let unlock = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASS_ONE })),
    )
    .await;
    assert_eq!(unlock.status, StatusCode::OK, "{:?}", unlock.json);
    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
}

fn sign_out_call(who: &(String, String)) -> Call<'_> {
    let mut c = call("POST", "/auth/v1/signout", Some(json!({})));
    c.token = Some(&who.0);
    c
}

#[tokio::test]
async fn sign_out_of_the_last_session_locks_the_vault() {
    let app = app(Signup::Invite);
    let first = new_user(&app, "a@example.org", PASS_ONE).await;
    let second = send(&app, sign_in_call("a@example.org", PASS_ONE)).await;
    let second = (
        second.token(),
        second.json["csrf"].as_str().unwrap().to_owned(),
    );
    send(&app, sign_out_call(&first)).await;
    let status = api(&app, &second, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
    send(&app, sign_out_call(&second)).await;
    let id = app.state.accounts.folders().unwrap()[0].0;
    let user = app.state.users.peek(id).await.unwrap();
    assert_eq!(
        user.handle.vault_state().await.unwrap(),
        studio_api::VaultState::Locked
    );
}

#[tokio::test]
async fn an_ended_last_session_locks_the_vault() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let old = accounts_now() - 8 * 86_400;
    let db = rusqlite::Connection::open(app.data.path().join("server.sqlite")).unwrap();
    db.execute("UPDATE sessions SET last_seen = ?1", [old])
        .unwrap();
    let ended = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(ended.code(), "session_ended");
    let id = app.state.accounts.folders().unwrap()[0].0;
    let user = app.state.users.peek(id).await.unwrap();
    assert_eq!(
        user.handle.vault_state().await.unwrap(),
        studio_api::VaultState::Locked
    );
}

#[tokio::test]
async fn the_idle_task_closes_a_user_with_no_session_and_no_work() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let id = app.state.accounts.folders().unwrap()[0].0;
    crate::idle::sweep(&app.state).await;
    assert!(app.state.users.peek(id).await.is_some());
    let hash = crate::accounts::sha256_hex(&who.0);
    app.state.accounts.delete_session(&hash).unwrap();
    crate::idle::sweep(&app.state).await;
    assert!(app.state.users.peek(id).await.is_none());
}

#[tokio::test]
async fn the_idle_task_keeps_a_user_with_a_queued_download() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let (id, folder) = app.state.accounts.folders().unwrap().remove(0);
    let mut collection = studio_core::collection::Collection::open(
        &app.state
            .users
            .folder_path(&folder)
            .join("collection.sqlite"),
    )
    .unwrap();
    let feed = collection.add_feed(None, None).unwrap();
    let file = collection
        .add_library_file(
            feed,
            "a.mp3",
            5,
            None,
            "audio/mpeg",
            false,
            Some("http://127.0.0.1:9/a.mp3"),
            None,
        )
        .unwrap();
    collection.queue_download(file).unwrap();
    let hash = crate::accounts::sha256_hex(&who.0);
    app.state.accounts.delete_session(&hash).unwrap();
    assert!(app.state.users.is_busy_folder(&folder).await);
    crate::idle::sweep(&app.state).await;
    assert!(app.state.users.peek(id).await.is_some());
    // The vault of the user is locked, also when the user stays open.
    let user = app.state.users.peek(id).await.unwrap();
    assert_eq!(
        user.handle.vault_state().await.unwrap(),
        studio_api::VaultState::Locked
    );
    collection.remove_download(file).unwrap();
    crate::idle::sweep(&app.state).await;
    assert!(app.state.users.peek(id).await.is_none());
}

#[tokio::test]
async fn a_sign_in_gets_503_busy_when_no_permit_comes() {
    let app = app_with(Signup::Invite, |o| {
        o.argon2_concurrency = 1;
        o.argon2_wait = std::time::Duration::from_millis(150);
    });
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let permit = std::sync::Arc::clone(&app.state.argon2)
        .acquire_owned()
        .await
        .unwrap();
    let busy = send(&app, sign_in_call("a@example.org", PASS_ONE)).await;
    assert_eq!(busy.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(busy.code(), "busy");
    let unknown = send(&app, sign_in_call("nobody@example.org", PASS_ONE)).await;
    assert_eq!(unknown.code(), "busy");
    let unlock = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASS_ONE })),
    )
    .await;
    assert_eq!(unlock.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(unlock.code(), "busy");
    let invite = app.state.accounts.create_invite().unwrap();
    let up = sign_up(&app, "b@example.org", PASS_TWO, Some(&invite)).await;
    assert_eq!(up.code(), "busy");
    // The failed sign-up did not use the invite.
    drop(permit);
    let up = sign_up(&app, "b@example.org", PASS_TWO, Some(&invite)).await;
    assert_eq!(up.status, StatusCode::CREATED);
    let good = send(&app, sign_in_call("a@example.org", PASS_ONE)).await;
    assert_eq!(good.status, StatusCode::OK);
}

#[tokio::test]
async fn ten_failed_sign_ins_block_the_account_also_for_the_correct_passphrase() {
    let app = app(Signup::Invite);
    new_user(&app, "a@example.org", PASS_ONE).await;
    for _ in 0..10 {
        let reply = send(&app, sign_in_call("a@example.org", PASS_TWO)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.code(), "sign_in_failed");
    }
    // A different address of the client does not help.
    let mut eleventh = sign_in_call("a@example.org", PASS_ONE);
    eleventh.addr = "10.0.0.99:4000";
    let reply = send(&app, eleventh).await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(reply.code(), "too_many_attempts");
    // Another account is free.
    new_user(&app, "b@example.org", PASS_TWO).await;
    let mut other = sign_in_call("b@example.org", PASS_TWO);
    other.addr = "10.0.0.99:4000";
    assert_eq!(send(&app, other).await.status, StatusCode::OK);
}

#[tokio::test]
async fn thirty_failed_sign_ins_block_the_client_address() {
    let app = app_with(Signup::Invite, |o| {
        o.client_address_header = Some("x-real-ip".to_owned());
    });
    new_user(&app, "a@example.org", PASS_ONE).await;
    for i in 0..30 {
        let mut c = sign_in_call(&format!("nobody{i}@example.org"), PASS_ONE);
        c.extra = vec![("x-real-ip", "203.0.113.5")];
        assert_eq!(send(&app, c).await.code(), "sign_in_failed");
    }
    let mut blocked = sign_in_call("a@example.org", PASS_ONE);
    blocked.extra = vec![("x-real-ip", "203.0.113.5")];
    assert_eq!(send(&app, blocked).await.code(), "too_many_attempts");
    // The header names the client, not the address of the connection.
    let mut free = sign_in_call("a@example.org", PASS_ONE);
    free.extra = vec![("x-real-ip", "203.0.113.6")];
    assert_eq!(send(&app, free).await.status, StatusCode::OK);
}

#[tokio::test]
async fn account_deletion_removes_the_folder_the_rows_and_the_session() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    new_user(&app, "b@example.org", PASS_TWO).await;
    let (id, folder) = app.state.accounts.folders().unwrap().remove(0);
    let path = app.state.users.folder_path(&folder);
    assert!(path.join("vault.json").exists());
    let delete = |body: Value| {
        let mut c = call("POST", "/auth/v1/account/delete", Some(body));
        c.token = Some(&who.0);
        c.csrf = Some(&who.1);
        c
    };
    let wrong = send(&app, delete(json!({ "confirm": "b@example.org" }))).await;
    assert_eq!(wrong.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(wrong.code(), "confirm_mismatch");
    assert!(path.exists());
    let mut no_csrf = call(
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": "a@example.org" })),
    );
    no_csrf.token = Some(&who.0);
    assert_eq!(send(&app, no_csrf).await.code(), "csrf_required");
    // A locked vault blocks the deletion.
    api(&app, &who, "POST", "/api/v1/vault/lock", None).await;
    let locked = send(&app, delete(json!({ "confirm": "a@example.org" }))).await;
    assert_eq!(locked.status, StatusCode::LOCKED);
    assert_eq!(locked.code(), "vault_locked");
    assert!(path.exists());
    api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({ "passphrase": PASS_ONE })),
    )
    .await;
    let done = send(&app, delete(json!({ "confirm": " A@Example.org " }))).await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.text);
    let cookie = done
        .headers
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("Max-Age=0"));
    assert!(!path.exists());
    assert!(app.state.accounts.folder_of(id).unwrap().is_none());
    assert!(app.state.accounts.email_of(id).unwrap().is_none());
    assert!(app.state.users.peek(id).await.is_none());
    let db = rusqlite::Connection::open(app.data.path().join("server.sqlite")).unwrap();
    let rows: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE account_id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 0);
    let after = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
    // The email address is free again, and the other account stays.
    assert_eq!(app.state.accounts.folders().unwrap().len(), 1);
    let again = send(&app, sign_in_call("a@example.org", PASS_ONE)).await;
    assert_eq!(again.code(), "sign_in_failed");
}

#[tokio::test]
async fn a_user_folder_refuses_a_local_host_on_the_server() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let body =
        json!({ "kind": "local", "name": "Folder", "root": app.data.path().to_str().unwrap() });
    let reply = api(&app, &who, "POST", "/api/v1/connectors", Some(body)).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{:?}", reply.json);
    assert_eq!(reply.code(), "not_on_server");
    let other = json!({ "root": app.data.path().join("elsewhere").to_str().unwrap() });
    let reply = api(&app, &who, "PUT", "/api/v1/library", Some(other)).await;
    assert_eq!(reply.code(), "not_on_server");
}
