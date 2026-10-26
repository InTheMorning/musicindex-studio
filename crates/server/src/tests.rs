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
    app_with_rules(signup, PublicRules::with_test_exceptions(Vec::new()), tweak)
}

fn app_with_rules(
    signup: Signup,
    rules: PublicRules,
    tweak: impl FnOnce(&mut ServeOptions),
) -> App {
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
        nip46_relays: vec!["wss://relay.nsec.app".to_owned()],
        nip46_timeout: std::time::Duration::from_secs(60),
    };
    tweak(&mut options);
    let state = crate::make_state(&options, rules).unwrap();
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

#[tokio::test]
async fn the_manifest_and_its_icons_are_served_without_a_session() {
    let app = app(Signup::Invite);
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
    let icons = manifest["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 3);
    for icon in icons {
        let src = icon["src"].as_str().unwrap();
        assert_eq!(get(src.to_owned()).await.status(), StatusCode::OK, "{src}");
    }
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
async fn the_strength_route_gives_the_score_of_the_desktop_route() {
    let app = app(Signup::Open);
    for passphrase in ["monkeymonkey1", "Kx9#mQ2!vL7@pR4z", "short"] {
        let body = json!({ "passphrase": passphrase });
        let reply = send(&app, call("POST", "/auth/v1/strength", Some(body))).await;
        assert_eq!(reply.status, StatusCode::OK);
        let expected = serde_json::to_value(studio_core::vault::passphrase_strength(passphrase));
        assert_eq!(reply.json, expected.unwrap());
        assert!(!reply.text.contains(passphrase));
    }
}

#[tokio::test]
async fn the_strength_route_refuses_a_blocked_client_address() {
    let app = app_with(Signup::Open, |o| {
        o.client_address_header = Some("x-real-ip".to_owned());
    });
    for i in 0..30 {
        let mut c = sign_in_call(&format!("nobody{i}@example.org"), PASS_ONE);
        c.extra = vec![("x-real-ip", "203.0.113.9")];
        send(&app, c).await;
    }
    let body = json!({ "passphrase": PASS_ONE });
    let mut blocked = call("POST", "/auth/v1/strength", Some(body));
    blocked.extra = vec![("x-real-ip", "203.0.113.9")];
    assert_eq!(send(&app, blocked).await.code(), "too_many_attempts");
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

// Task 034e: the NIP-07 sign-in.

use nostr::prelude::{Keys, Kind, SecretKey, Tag, Timestamp, ToBech32, UnsignedEvent};

const NOSTR_URL: &str = "http://127.0.0.1:8080/auth/v1/nostr";

async fn new_challenge(app: &App) -> String {
    let reply = send(app, call("POST", "/auth/v1/challenge", None)).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    reply.json["challenge"].as_str().unwrap().to_owned()
}

/// A new key pair.
fn new_keys() -> Keys {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).unwrap();
    Keys::new(SecretKey::from_slice(&bytes).unwrap())
}

/// A signed event of a kind, with the given tags and time.
fn signed_kind(keys: &Keys, kind: u16, tags: &[[&str; 2]], created_at: u64) -> Value {
    let tags: Vec<Tag> = tags
        .iter()
        .map(|tag| Tag::parse(tag.to_vec()).unwrap())
        .collect();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::from(created_at),
        Kind::from_u16(kind),
        tags,
        "",
    );
    let event = studio_core::nostr_events::sign_with_keys(unsigned, keys).unwrap();
    serde_json::to_value(&event).unwrap()
}

/// A signed sign-in event with the given tags and time.
fn signed_event(keys: &Keys, tags: &[[&str; 2]], created_at: u64) -> Value {
    signed_kind(keys, 27_235, tags, created_at)
}

fn now_secs() -> u64 {
    crate::accounts::now().unsigned_abs()
}

fn good_event(keys: &Keys, challenge: &str) -> Value {
    signed_event(
        keys,
        &[
            ["u", NOSTR_URL],
            ["method", "POST"],
            ["challenge", challenge],
        ],
        now_secs(),
    )
}

async fn nostr_call(app: &App, event: Value, invite: Option<&str>) -> Reply {
    let body = json!({ "event": event, "invite": invite });
    send(app, call("POST", "/auth/v1/nostr", Some(body))).await
}

async fn nostr_sign_in(app: &App, keys: &Keys) -> Reply {
    let challenge = new_challenge(app).await;
    nostr_call(app, good_event(keys, &challenge), None).await
}

fn who_of(reply: &Reply) -> (String, String) {
    (
        reply.token(),
        reply.json["csrf"].as_str().unwrap().to_owned(),
    )
}

#[tokio::test]
async fn a_nostr_user_signs_up_adds_a_wrapper_signs_in_and_opens_the_vault_with_the_key() {
    let app = app(Signup::Open);
    let keys = new_keys();
    let up = nostr_sign_in(&app, &keys).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    assert_eq!(up.json["new_account"], true);
    assert!(up.json["recovery_key"].is_null());
    let who = who_of(&up);

    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
    let key = api(&app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(key.status, StatusCode::OK);
    let key = key.json["key"].as_str().unwrap().to_owned();
    let pubkey = keys.public_key().to_hex();
    let add = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(json!({ "pubkey": pubkey, "ciphertext": "test-nip44-ciphertext" })),
    )
    .await;
    assert_eq!(add.status, StatusCode::OK, "{:?}", add.json);

    let out = send(&app, sign_out_call(&who)).await;
    assert_eq!(out.status, StatusCode::OK);
    let again = nostr_sign_in(&app, &keys).await;
    assert_eq!(again.status, StatusCode::OK, "{:?}", again.json);
    assert_eq!(again.json["new_account"], false);
    // The sign-in does not open the vault.
    assert_eq!(again.json["vault"], "locked");
    let who = who_of(&again);
    let locked = api(&app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(locked.status, StatusCode::LOCKED);
    let wrapper = api(&app, &who, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(wrapper.json["pubkey"], pubkey.as_str());
    assert_eq!(wrapper.json["ciphertext"], "test-nip44-ciphertext");
    let wrong = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/unlock-key",
        Some(json!({ "key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" })),
    )
    .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let open = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/unlock-key",
        Some(json!({ "key": key })),
    )
    .await;
    assert_eq!(open.status, StatusCode::OK, "{:?}", open.json);
    assert_eq!(open.json["vault"], "open");
    let session = send(&app, {
        let mut c = call("GET", "/auth/v1/session", None);
        c.token = Some(&who.0);
        c
    })
    .await;
    assert_eq!(session.json["signed_in"], true);
    assert_eq!(session.json["pubkey"], pubkey.as_str());
}

#[tokio::test]
async fn an_invite_server_needs_an_invite_for_a_new_nostr_key() {
    let app = app(Signup::Invite);
    let keys = new_keys();
    let challenge = new_challenge(&app).await;
    let none = nostr_call(&app, good_event(&keys, &challenge), None).await;
    assert_eq!(none.status, StatusCode::FORBIDDEN);
    assert_eq!(none.code(), "invite_required");
    let invite = app.state.accounts.create_invite().unwrap();
    let challenge = new_challenge(&app).await;
    let up = nostr_call(&app, good_event(&keys, &challenge), Some(&invite)).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
}

#[tokio::test]
async fn a_challenge_works_one_time_and_an_unknown_or_expired_one_never() {
    let app = app(Signup::Open);
    let keys = new_keys();
    let challenge = new_challenge(&app).await;
    let first = nostr_call(&app, good_event(&keys, &challenge), None).await;
    assert_eq!(first.status, StatusCode::CREATED);
    let used = nostr_call(&app, good_event(&keys, &challenge), None).await;
    assert_eq!(used.status, StatusCode::UNAUTHORIZED);
    assert_eq!(used.code(), "challenge_invalid");

    let unknown = nostr_call(&app, good_event(&keys, &"ab".repeat(32)), None).await;
    assert_eq!(unknown.code(), "challenge_invalid");

    let old = "cd".repeat(32);
    app.state
        .accounts
        .insert_challenge(&old, crate::accounts::now() - 1);
    let expired = nostr_call(&app, good_event(&keys, &old), None).await;
    assert_eq!(expired.status, StatusCode::UNAUTHORIZED);
    assert_eq!(expired.code(), "challenge_invalid");

    let no_tag = signed_event(&keys, &[["u", NOSTR_URL], ["method", "POST"]], now_secs());
    assert_eq!(
        nostr_call(&app, no_tag, None).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn two_requests_with_one_challenge_give_one_session() {
    let app = app(Signup::Open);
    let challenge = new_challenge(&app).await;
    let one = good_event(&new_keys(), &challenge);
    let two = good_event(&new_keys(), &challenge);
    let (a, b) = tokio::join!(nostr_call(&app, one, None), nostr_call(&app, two, None));
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::UNAUTHORIZED]);
    assert_eq!(app.state.accounts.folders().unwrap().len(), 1);
}

#[tokio::test]
async fn a_wrong_event_is_refused_and_does_not_use_the_challenge() {
    let app = app(Signup::Open);
    let keys = new_keys();
    let challenge = new_challenge(&app).await;
    let tag = |name: &'static str, value: &str| -> Value {
        let mut tags = vec![
            ["u", NOSTR_URL],
            ["method", "POST"],
            ["challenge", challenge.as_str()],
        ];
        for t in &mut tags {
            if t[0] == name {
                t[1] = value;
            }
        }
        signed_event(&keys, &tags, now_secs())
    };
    let other_url = tag("u", "http://127.0.0.1:8080/auth/v1/other");
    let other_method = tag("method", "GET");
    let old = signed_event(
        &keys,
        &[
            ["u", NOSTR_URL],
            ["method", "POST"],
            ["challenge", &challenge],
        ],
        now_secs() - 120,
    );
    let mut bad_signature = good_event(&keys, &challenge);
    let other_signature = good_event(&new_keys(), &challenge);
    bad_signature["sig"] = other_signature["sig"].clone();
    let mut bad_id = good_event(&keys, &challenge);
    bad_id["content"] = json!("changed");
    let wrong_kind = signed_kind(
        &keys,
        1,
        &[
            ["u", NOSTR_URL],
            ["method", "POST"],
            ["challenge", &challenge],
        ],
        now_secs(),
    );
    for (name, event) in [
        ("other url", other_url),
        ("other method", other_method),
        ("old event", old),
        ("bad signature", bad_signature),
        ("bad id", bad_id),
        ("wrong kind", wrong_kind),
        ("not an event", json!({ "kind": 1 })),
    ] {
        let reply = nostr_call(&app, event, None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{name}");
        assert_ne!(reply.code(), "", "{name}");
    }
    assert_eq!(app.state.accounts.folders().unwrap().len(), 0);
    // No refusal used the challenge.
    let good = nostr_call(&app, good_event(&keys, &challenge), None).await;
    assert_eq!(good.status, StatusCode::CREATED, "{:?}", good.json);
}

#[tokio::test]
async fn an_account_with_no_vault_file_gets_a_new_vault_at_the_next_sign_in() {
    let app = app(Signup::Open);
    let keys = new_keys();
    let up = nostr_sign_in(&app, &keys).await;
    let who = who_of(&up);
    let folder = app.state.accounts.folders().unwrap().remove(0).1;
    let vault_file = app
        .data
        .path()
        .join("users")
        .join(folder)
        .join("vault.json");
    assert!(!vault_file.exists());
    let out = send(&app, sign_out_call(&who)).await;
    assert_eq!(out.status, StatusCode::OK);
    let again = nostr_sign_in(&app, &keys).await;
    assert_eq!(again.status, StatusCode::OK, "{:?}", again.json);
    assert_eq!(again.json["vault"], "open");
    let who = who_of(&again);
    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
    assert!(!vault_file.exists());
}

#[tokio::test]
async fn the_failures_of_a_nostr_key_are_counted_and_the_key_is_blocked() {
    let app = app(Signup::Open);
    let keys = new_keys();
    for _ in 0..crate::limits::ACCOUNT_LIMIT {
        let reply = nostr_call(&app, good_event(&keys, &"00".repeat(32)), None).await;
        assert_eq!(reply.code(), "challenge_invalid");
    }
    let challenge = new_challenge(&app).await;
    let blocked = nostr_call(&app, good_event(&keys, &challenge), None).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn an_event_with_a_wrong_signature_does_not_block_its_public_key() {
    let app = app(Signup::Open);
    let keys = new_keys();
    for _ in 0..crate::limits::ACCOUNT_LIMIT {
        let mut event = good_event(&keys, &"00".repeat(32));
        event["sig"] = json!("00".repeat(64));
        let reply = nostr_call(&app, event, None).await;
        assert_eq!(reply.code(), "event_invalid");
    }
    let reply = nostr_sign_in(&app, &keys).await;
    assert_eq!(reply.status, StatusCode::CREATED);
}

#[tokio::test]
async fn a_nostr_account_can_delete_itself_with_its_public_key() {
    let app = app(Signup::Open);
    let keys = new_keys();
    let who = who_of(&nostr_sign_in(&app, &keys).await);
    let wrong = api(
        &app,
        &who,
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": "nobody" })),
    )
    .await;
    assert_eq!(wrong.code(), "confirm_mismatch");
    let gone = api(
        &app,
        &who,
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": keys.public_key().to_bech32().unwrap() })),
    )
    .await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT, "{:?}", gone.json);
    assert_eq!(app.state.accounts.folders().unwrap().len(), 0);
}

const NEW_PASS: &str = "river umbrella orange staple";

/// Sign up an email account with a recovery key. Gives the key.
async fn user_with_recovery(app: &App, email: &str, passphrase: &str) -> String {
    let invite = app.state.accounts.create_invite().unwrap();
    let body = json!({
        "email": email, "passphrase": passphrase, "invite": invite, "recovery_key": true,
    });
    let up = send(app, call("POST", "/auth/v1/signup/email", Some(body))).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    let key = up.json["recovery_key"].as_str().unwrap().to_owned();
    send(app, sign_out_call(&who_of(&up))).await;
    key
}

fn recovery_call<'a>(account: &str, key: &str) -> Call<'a> {
    call(
        "POST",
        "/auth/v1/recovery",
        Some(json!({ "account": account, "recovery_key": key })),
    )
}

#[tokio::test]
async fn a_recovery_sign_in_is_limited_until_a_new_passphrase() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let reply = send(&app, recovery_call("A@Example.org ", &key)).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    assert_eq!(reply.json["recovery"], true);
    assert_eq!(reply.json["vault"], "open");
    let who = who_of(&reply);

    let mut session = call("GET", "/auth/v1/session", None);
    session.token = Some(&who.0);
    assert_eq!(send(&app, session).await.json["recovery"], true);
    let status = api(&app, &who, "GET", "/api/v1/status", None).await;
    assert_eq!(status.status, StatusCode::OK);
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.status, StatusCode::FORBIDDEN);
    assert_eq!(feeds.code(), "method_required");
    let weak = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": "passwordpassword" })),
    )
    .await;
    assert_eq!(weak.code(), "passphrase_too_weak");
    let still = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(still.code(), "method_required");
    let set = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/passphrase",
        Some(json!({ "passphrase": NEW_PASS })),
    )
    .await;
    assert_eq!(set.status, StatusCode::OK, "{:?}", set.json);
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.status, StatusCode::OK);
    let mut session = call("GET", "/auth/v1/session", None);
    session.token = Some(&who.0);
    assert_eq!(send(&app, session).await.json["recovery"], false);

    send(&app, sign_out_call(&who)).await;
    let new = send(&app, sign_in_call("a@example.org", NEW_PASS)).await;
    assert_eq!(new.status, StatusCode::OK);
    let old = send(&app, sign_in_call("a@example.org", PASS_ONE)).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_nostr_account_signs_in_with_its_npub_and_a_new_wrapper_lifts_the_limit() {
    use nostr::prelude::ToBech32;
    let app = app(Signup::Open);
    let keys = new_keys();
    let challenge = new_challenge(&app).await;
    let body = json!({ "event": good_event(&keys, &challenge), "recovery_key": true });
    let up = send(&app, call("POST", "/auth/v1/nostr", Some(body))).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    let key = up.json["recovery_key"].as_str().unwrap().to_owned();
    send(&app, sign_out_call(&who_of(&up))).await;

    let npub = keys.public_key().to_bech32().unwrap();
    let reply = send(&app, recovery_call(&npub, &key)).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    let who = who_of(&reply);
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");
    let delete = api(
        &app,
        &who,
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": npub })),
    )
    .await;
    assert_eq!(delete.code(), "method_required");
    let vault_key = api(&app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(vault_key.status, StatusCode::OK);
    let add = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(json!({ "pubkey": keys.public_key().to_hex(), "ciphertext": "test-ciphertext" })),
    )
    .await;
    assert_eq!(add.status, StatusCode::OK, "{:?}", add.json);
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.status, StatusCode::OK);
}

#[tokio::test]
async fn a_failed_nostr_wrapper_keeps_the_limit() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let who = who_of(&send(&app, recovery_call("a@example.org", &key)).await);
    let bad = api(
        &app,
        &who,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(json!({ "pubkey": "", "ciphertext": "" })),
    )
    .await;
    assert!(!bad.status.is_success());
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");
}

#[tokio::test]
async fn a_restricted_session_must_add_a_method_of_its_own_account() {
    let email_app = app(Signup::Invite);
    let key = user_with_recovery(&email_app, "a@example.org", PASS_ONE).await;
    let who = who_of(&send(&email_app, recovery_call("a@example.org", &key)).await);
    // An email account cannot sign in with a Nostr wrapper.
    let keys = new_keys();
    let wrapper = json!({ "pubkey": keys.public_key().to_hex(), "ciphertext": "test-ciphertext" });
    let add = api(
        &email_app,
        &who,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(wrapper),
    )
    .await;
    assert_eq!(add.code(), "method_not_usable");
    let feeds = api(&email_app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");

    // A Nostr account cannot sign in with a passphrase, or with the
    // wrapper of a different key.
    let nostr_app = app(Signup::Open);
    let challenge = new_challenge(&nostr_app).await;
    let body = json!({ "event": good_event(&keys, &challenge), "recovery_key": true });
    let up = send(&nostr_app, call("POST", "/auth/v1/nostr", Some(body))).await;
    let key = up.json["recovery_key"].as_str().unwrap().to_owned();
    send(&nostr_app, sign_out_call(&who_of(&up))).await;
    let npub = keys.public_key().to_bech32().unwrap();
    let who = who_of(&send(&nostr_app, recovery_call(&npub, &key)).await);
    let passphrase = json!({ "passphrase": NEW_PASS });
    let set = api(
        &nostr_app,
        &who,
        "POST",
        "/api/v1/vault/passphrase",
        Some(passphrase),
    )
    .await;
    assert_eq!(set.code(), "method_not_usable");
    let other =
        json!({ "pubkey": new_keys().public_key().to_hex(), "ciphertext": "test-ciphertext" });
    let add = api(
        &nostr_app,
        &who,
        "POST",
        "/api/v1/vault/nostr-wrapper",
        Some(other),
    )
    .await;
    assert_eq!(add.code(), "method_not_usable");
    let feeds = api(&nostr_app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");
}

#[tokio::test]
async fn an_unknown_account_a_wrong_key_and_no_vault_file_give_the_same_error() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    // An account with no vault file.
    new_user(&app, "b@example.org", PASS_TWO).await;
    let (_, folder) = app
        .state
        .accounts
        .account_by_email("b@example.org")
        .unwrap()
        .unwrap();
    app.state.users.forget(2).await;
    std::fs::remove_file(app.state.users.folder_path(&folder).join("vault.json")).unwrap();

    let wrong = "0000-0000-0000-0000-0000-0000-0000-0000";
    let replies = [
        send(&app, recovery_call("nobody@example.org", &key)).await,
        send(&app, recovery_call("a@example.org", wrong)).await,
        send(&app, recovery_call("b@example.org", &key)).await,
        send(&app, recovery_call("not an account", &key)).await,
        send(&app, recovery_call("npub1invalid", &key)).await,
    ];
    for reply in &replies {
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{:?}", reply.json);
        assert_eq!(reply.code(), "sign_in_failed");
        assert_eq!(reply.message(), replies[0].message());
    }
}

#[tokio::test]
async fn ten_failed_recovery_sign_ins_block_the_account_also_for_the_correct_key() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let wrong = "0000-0000-0000-0000-0000-0000-0000-0000";
    for _ in 0..10 {
        let reply = send(&app, recovery_call("a@example.org", wrong)).await;
        assert_eq!(reply.code(), "sign_in_failed");
    }
    let mut next = recovery_call("a@example.org", &key);
    next.addr = "10.0.0.99:4000";
    let reply = send(&app, next).await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(reply.code(), "too_many_attempts");
}

#[tokio::test]
async fn a_restricted_session_cannot_delete_the_account() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let who = who_of(&send(&app, recovery_call("a@example.org", &key)).await);
    let delete = api(
        &app,
        &who,
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": "a@example.org" })),
    )
    .await;
    assert_eq!(delete.status, StatusCode::FORBIDDEN);
    assert_eq!(delete.code(), "method_required");
    assert_eq!(app.state.accounts.folders().unwrap().len(), 1);
}

// ---- Passkeys (task 035b) ----

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use p256::ecdsa::signature::Signer as _;
use p256::ecdsa::{Signature, SigningKey};
use p256::pkcs8::EncodePublicKey as _;
use sha2::{Digest, Sha256};

/// A virtual authenticator: one key, one credential and one PRF secret.
struct Device {
    key: SigningKey,
    id: Vec<u8>,
    secret: [u8; 32],
}

impl Device {
    fn new(seed: u8) -> Self {
        Self {
            key: SigningKey::from_slice(&[seed; 32]).unwrap(),
            id: vec![seed; 16],
            secret: [seed ^ 0x55; 32],
        }
    }

    fn credential_id(&self) -> String {
        B64.encode(&self.id)
    }

    /// The PRF output for a salt (base64url).
    fn prf(&self, salt: &str) -> String {
        let mut hash = Sha256::new();
        hash.update(self.secret);
        hash.update(B64.decode(salt).unwrap());
        B64.encode(hash.finalize())
    }

    fn client_data(kind: &str, challenge: &str, origin: &str) -> Vec<u8> {
        json!({ "type": kind, "challenge": challenge, "origin": origin, "crossOrigin": false })
            .to_string()
            .into_bytes()
    }

    fn authenticator_data(&self, rp_id: &str, flags: u8, counter: u32, attested: bool) -> Vec<u8> {
        let mut data = Sha256::digest(rp_id.as_bytes()).to_vec();
        data.push(flags);
        data.extend_from_slice(&counter.to_be_bytes());
        if attested {
            data.extend_from_slice(&[0; 16]);
            data.extend_from_slice(&u16::try_from(self.id.len()).unwrap().to_be_bytes());
            data.extend_from_slice(&self.id);
            data.extend_from_slice(&[0xa5, 0x01, 0x02]);
        }
        data
    }

    fn registration(&self, challenge: &str) -> Value {
        let spki = self.key.verifying_key().to_public_key_der().unwrap();
        json!({
            "client_data": B64.encode(Self::client_data("webauthn.create", challenge, ORIGIN)),
            "authenticator_data": B64.encode(self.authenticator_data("127.0.0.1", 0x45, 0, true)),
            "public_key": B64.encode(spki.as_bytes()),
            "algorithm": -7,
        })
    }

    fn assertion_with(&self, challenge: &str, origin: &str, counter: u32, flags: u8) -> Value {
        let client = Self::client_data("webauthn.get", challenge, origin);
        let auth = self.authenticator_data("127.0.0.1", flags, counter, false);
        let mut signed = auth.clone();
        signed.extend_from_slice(&Sha256::digest(&client));
        let signature: Signature = self.key.sign(&signed);
        json!({
            "credential_id": self.credential_id(),
            "client_data": B64.encode(client),
            "authenticator_data": B64.encode(auth),
            "signature": B64.encode(signature.to_der().as_bytes()),
        })
    }

    fn assertion(&self, challenge: &str, counter: u32) -> Value {
        self.assertion_with(challenge, ORIGIN, counter, 0x05)
    }
}

/// Add a passkey to a signed-in account. Gives the reply of the add call.
async fn add_passkey(app: &App, who: &(String, String), device: &Device, name: &str) -> Reply {
    let options = api(
        app,
        who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    assert_eq!(options.status, StatusCode::OK, "{:?}", options.json);
    let salt = options.json["prf_salt"].as_str().unwrap();
    let body = json!({
        "name": name,
        "registration": device.registration(options.json["challenge"].as_str().unwrap()),
        "assertion": device.assertion(options.json["assertion_challenge"].as_str().unwrap(), 0),
        "prf_output": device.prf(salt),
    });
    api(app, who, "POST", "/auth/v1/passkeys", Some(body)).await
}

/// The options of a sign-in: the challenge and the salt.
async fn passkey_options(app: &App, who: Option<&(String, String)>) -> Reply {
    let mut c = call("POST", "/auth/v1/passkey/options", Some(json!({})));
    if let Some(who) = who {
        c.token = Some(&who.0);
        c.csrf = Some(&who.1);
    }
    let reply = send(app, c).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    reply
}

/// Sign in with a passkey. The caller gives the counter of the assertion.
async fn passkey_sign_in(app: &App, device: &Device, counter: u32) -> Reply {
    let options = passkey_options(app, None).await;
    let challenge = options.json["challenge"].as_str().unwrap();
    let body = json!({
        "assertion": device.assertion(challenge, counter),
        "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
    });
    send(app, call("POST", "/auth/v1/passkey", Some(body))).await
}

#[tokio::test]
async fn a_passkey_is_added_signs_in_and_opens_the_vault() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    let added = add_passkey(&app, &who, &device, "Phone").await;
    assert_eq!(added.status, StatusCode::CREATED, "{:?}", added.json);
    assert_eq!(added.json["credential_id"], device.credential_id().as_str());
    let list = api(&app, &who, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.json[0]["name"], "Phone");
    assert_eq!(list.json.as_array().unwrap().len(), 1);

    send(&app, sign_out_call(&who)).await;
    let reply = passkey_sign_in(&app, &device, 0).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    assert_eq!(reply.json["vault"], "open");
    let again = who_of(&reply);
    let status = api(&app, &again, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");

    // The vault locks, and the passkey unlocks it in the same session.
    let lock = api(&app, &again, "POST", "/api/v1/vault/lock", Some(json!({}))).await;
    assert_eq!(lock.json["vault"], "locked");
    let options = passkey_options(&app, Some(&again)).await;
    assert_eq!(options.json["allow"][0], device.credential_id().as_str());
    let body = json!({
        "assertion": device.assertion(options.json["challenge"].as_str().unwrap(), 0),
        "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
    });
    let mut c = call("POST", "/auth/v1/passkey", Some(body));
    c.token = Some(&again.0);
    let unlocked = send(&app, c).await;
    assert_eq!(unlocked.status, StatusCode::OK, "{:?}", unlocked.json);
    assert_eq!(unlocked.json["vault"], "open");
    assert!(unlocked.headers.get(header::SET_COOKIE).is_none());
    let status = api(&app, &again, "GET", "/api/v1/status", None).await;
    assert_eq!(status.json["vault"], "open");
}

#[tokio::test]
async fn a_passkey_wrapper_is_in_the_vault_and_the_server_keeps_no_prf_output() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(2);
    let added = add_passkey(&app, &who, &device, "Phone").await;
    assert_eq!(added.status, StatusCode::CREATED);
    let probe = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    let prf = device.prf(probe.json["prf_salt"].as_str().unwrap());
    assert!(!added.text.contains(&prf));
    let (_, folder) = app
        .state
        .accounts
        .account_by_email("a@example.org")
        .unwrap()
        .unwrap();
    let vault =
        std::fs::read_to_string(app.state.users.folder_path(&folder).join("vault.json")).unwrap();
    assert!(vault.contains(&device.credential_id()));
    assert!(!vault.contains(&prf));
    let database = std::fs::read(app.data.path().join("server.sqlite")).unwrap();
    assert!(!database.windows(prf.len()).any(|w| w == prf.as_bytes()));
}

#[tokio::test]
async fn a_registration_with_no_prf_output_gets_422_passkey_no_prf() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    let options = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    let body = json!({
        "name": "Phone",
        "registration": device.registration(options.json["challenge"].as_str().unwrap()),
        "assertion": device.assertion(options.json["assertion_challenge"].as_str().unwrap(), 0),
    });
    let reply = api(&app, &who, "POST", "/auth/v1/passkeys", Some(body.clone())).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.code(), "passkey_no_prf");
    // A PRF output of the wrong length is the same error.
    let mut short = body;
    short["prf_output"] = json!(B64.encode([1_u8; 16]));
    let reply = api(&app, &who, "POST", "/auth/v1/passkeys", Some(short)).await;
    assert_eq!(reply.code(), "passkey_no_prf");
    let list = api(&app, &who, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_registration_challenge_works_one_time_and_only_for_its_account() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let other = new_user(&app, "b@example.org", PASS_TWO).await;
    let device = Device::new(1);
    let options = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    let salt = options.json["prf_salt"].as_str().unwrap();
    let body = json!({
        "name": "Phone",
        "registration": device.registration(options.json["challenge"].as_str().unwrap()),
        "assertion": device.assertion(options.json["assertion_challenge"].as_str().unwrap(), 0),
        "prf_output": device.prf(salt),
    });
    let stolen = api(
        &app,
        &other,
        "POST",
        "/auth/v1/passkeys",
        Some(body.clone()),
    )
    .await;
    assert_eq!(stolen.code(), "challenge_invalid");
    let first = api(&app, &who, "POST", "/auth/v1/passkeys", Some(body.clone())).await;
    assert_eq!(first.status, StatusCode::CREATED);
    let again = api(&app, &who, "POST", "/auth/v1/passkeys", Some(body)).await;
    assert_eq!(again.code(), "challenge_invalid");
}

#[tokio::test]
async fn a_wrong_registration_or_assertion_adds_no_passkey() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    let intruder = Device::new(9);
    let cases = ["origin", "challenge", "uv", "other_credential", "swapped"];
    for case in cases {
        let options = api(
            &app,
            &who,
            "POST",
            "/auth/v1/passkeys/options",
            Some(json!({})),
        )
        .await;
        let challenge = options.json["challenge"].as_str().unwrap();
        let assertion_challenge = options.json["assertion_challenge"].as_str().unwrap();
        let salt = options.json["prf_salt"].as_str().unwrap();
        let (registration, assertion) = match case {
            "origin" => (
                device.registration(challenge),
                device.assertion_with(assertion_challenge, "https://evil.example.org", 0, 0x05),
            ),
            // The assertion uses the challenge of the registration.
            "challenge" => (
                device.registration(challenge),
                device.assertion(challenge, 0),
            ),
            "uv" => (
                device.registration(challenge),
                device.assertion_with(assertion_challenge, ORIGIN, 0, 0x01),
            ),
            // The assertion comes from a different credential.
            "other_credential" => (
                device.registration(challenge),
                intruder.assertion(assertion_challenge, 0),
            ),
            _ => (
                device.registration(assertion_challenge),
                device.assertion(challenge, 0),
            ),
        };
        let body = json!({
            "name": "Phone", "registration": registration, "assertion": assertion,
            "prf_output": device.prf(salt),
        });
        let reply = api(&app, &who, "POST", "/auth/v1/passkeys", Some(body)).await;
        assert!(!reply.status.is_success(), "{case}: {:?}", reply.json);
    }
    let list = api(&app, &who, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn the_passkey_routes_need_a_session_the_csrf_header_and_an_open_vault() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let none = send(
        &app,
        call("POST", "/auth/v1/passkeys/options", Some(json!({}))),
    )
    .await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
    let mut no_csrf = call("POST", "/auth/v1/passkeys/options", Some(json!({})));
    no_csrf.token = Some(&who.0);
    assert_eq!(send(&app, no_csrf).await.code(), "csrf_required");
    api(&app, &who, "POST", "/api/v1/vault/lock", Some(json!({}))).await;
    let locked = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    assert_eq!(locked.status, StatusCode::LOCKED);
}

#[tokio::test]
async fn the_last_method_cannot_be_removed_and_another_one_can() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let id = app
        .state
        .accounts
        .account_by_email("a@example.org")
        .unwrap()
        .unwrap()
        .0;
    let one = Device::new(1);
    let two = Device::new(2);
    add_passkey(&app, &who, &one, "One").await;
    add_passkey(&app, &who, &two, "Two").await;
    let path = |d: &Device| format!("/auth/v1/passkeys/{}", d.credential_id());
    let removed = api(&app, &who, "DELETE", &path(&one), None).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    // The wrapper is gone: the first passkey cannot sign in any more.
    send(&app, sign_out_call(&who)).await;
    assert_eq!(
        passkey_sign_in(&app, &one, 0).await.status,
        StatusCode::UNAUTHORIZED
    );
    let reply = passkey_sign_in(&app, &two, 0).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    let who = who_of(&reply);
    // Without the email address, the second passkey is the last method.
    app.state.accounts.delete_email(id);
    let last = api(&app, &who, "DELETE", &path(&two), None).await;
    assert_eq!(last.status, StatusCode::CONFLICT);
    assert_eq!(last.code(), "last_method");
    let missing = api(&app, &who, "DELETE", &path(&one), None).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_passkey_of_one_account_cannot_be_removed_by_another() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let other = new_user(&app, "b@example.org", PASS_TWO).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "One").await;
    let path = format!("/auth/v1/passkeys/{}", device.credential_id());
    let reply = api(&app, &other, "DELETE", &path, None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let list = api(&app, &other, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_passkey_challenge_works_one_time_and_a_replayed_assertion_fails() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "Phone").await;
    send(&app, sign_out_call(&who)).await;
    let options = passkey_options(&app, None).await;
    let body = json!({
        "assertion": device.assertion(options.json["challenge"].as_str().unwrap(), 0),
        "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
    });
    let first = send(&app, call("POST", "/auth/v1/passkey", Some(body.clone()))).await;
    assert_eq!(first.status, StatusCode::OK);
    let replay = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    assert_eq!(replay.code(), "challenge_invalid");
    // A challenge that the server never gave.
    let fake = B64.encode([5_u8; 32]);
    let body = json!({ "assertion": device.assertion(&fake, 0), "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()) });
    let unknown = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    assert_eq!(unknown.code(), "challenge_invalid");
}

#[tokio::test]
async fn the_counter_must_increase() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "Phone").await;
    send(&app, sign_out_call(&who)).await;
    assert_eq!(
        passkey_sign_in(&app, &device, 5).await.status,
        StatusCode::OK
    );
    let same = passkey_sign_in(&app, &device, 5).await;
    assert_eq!(same.status, StatusCode::UNAUTHORIZED);
    assert_eq!(same.code(), "passkey_invalid");
    assert_eq!(
        passkey_sign_in(&app, &device, 3).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        passkey_sign_in(&app, &device, 6).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_sign_in_with_a_wrong_part_gives_no_session() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "Phone").await;
    send(&app, sign_out_call(&who)).await;
    let unknown = Device::new(7);
    let forger = Device::new(1);
    // The forger has the credential ID of the device but another key.
    let forger = Device {
        key: SigningKey::from_slice(&[8_u8; 32]).unwrap(),
        ..forger
    };
    for (name, who_signs, origin, flags) in [
        ("unknown", &unknown, ORIGIN, 0x05),
        ("forged", &forger, ORIGIN, 0x05),
        ("origin", &device, "https://evil.example.org", 0x05),
        ("uv", &device, ORIGIN, 0x01),
    ] {
        let options = passkey_options(&app, None).await;
        let challenge = options.json["challenge"].as_str().unwrap();
        let body = json!({
            "assertion": who_signs.assertion_with(challenge, origin, 1, flags),
            "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
        });
        let reply = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{name}");
        assert!(reply.headers.get(header::SET_COOKIE).is_none(), "{name}");
    }
}

#[tokio::test]
async fn a_wrong_prf_output_gives_no_session_and_no_prf_gives_422() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "Phone").await;
    send(&app, sign_out_call(&who)).await;
    let options = passkey_options(&app, None).await;
    let challenge = options.json["challenge"].as_str().unwrap();
    let body = json!({
        "assertion": device.assertion(challenge, 0),
        "prf_output": B64.encode([3_u8; 32]),
    });
    let wrong = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    assert!(wrong.headers.get(header::SET_COOKIE).is_none());
    let options = passkey_options(&app, None).await;
    let body = json!({
        "assertion": device.assertion(options.json["challenge"].as_str().unwrap(), 0),
    });
    let none = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    assert_eq!(none.code(), "passkey_no_prf");
}

#[tokio::test]
async fn false_assertions_do_not_block_the_account_of_the_credential() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    add_passkey(&app, &who, &device, "Phone").await;
    send(&app, sign_out_call(&who)).await;
    let forger = Device {
        key: SigningKey::from_slice(&[8_u8; 32]).unwrap(),
        ..Device::new(1)
    };
    for i in 0..15 {
        let options = passkey_options(&app, None).await;
        let challenge = options.json["challenge"].as_str().unwrap();
        let body = json!({
            "assertion": forger.assertion(challenge, 0),
            "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
        });
        let mut c = call("POST", "/auth/v1/passkey", Some(body));
        // Each false assertion comes from a different address.
        let addr = format!("10.1.0.{}:4000", i + 1);
        c.addr = &addr;
        assert_eq!(send(&app, c).await.status, StatusCode::UNAUTHORIZED);
    }
    let reply = passkey_sign_in(&app, &device, 0).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
}

#[tokio::test]
async fn false_assertions_from_one_address_block_that_address() {
    let app = app(Signup::Invite);
    let forger = Device::new(4);
    for _ in 0..30 {
        let options = passkey_options(&app, None).await;
        let body = json!({
            "assertion": forger.assertion(options.json["challenge"].as_str().unwrap(), 0),
            "prf_output": forger.prf(options.json["prf_salt"].as_str().unwrap()),
        });
        send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    }
    let options = passkey_options(&app, None).await;
    let body = json!({
        "assertion": forger.assertion(options.json["challenge"].as_str().unwrap(), 0),
    });
    let reply = send(&app, call("POST", "/auth/v1/passkey", Some(body))).await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn a_recovery_session_adds_a_passkey_and_loses_its_limit() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let who = who_of(&send(&app, recovery_call("a@example.org", &key)).await);
    // A restricted session cannot list the passkeys.
    let list = api(&app, &who, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.code(), "method_required");
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");
    let device = Device::new(1);
    let added = add_passkey(&app, &who, &device, "Phone").await;
    assert_eq!(added.status, StatusCode::CREATED, "{:?}", added.json);
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.status, StatusCode::OK);
    let list = api(&app, &who, "GET", "/auth/v1/passkeys", None).await;
    assert_eq!(list.status, StatusCode::OK);
}

#[tokio::test]
async fn a_failed_passkey_add_keeps_the_limit_of_a_recovery_session() {
    let app = app(Signup::Invite);
    let key = user_with_recovery(&app, "a@example.org", PASS_ONE).await;
    let who = who_of(&send(&app, recovery_call("a@example.org", &key)).await);
    let device = Device::new(1);
    let options = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    let body = json!({
        "name": "Phone",
        "registration": device.registration(options.json["challenge"].as_str().unwrap()),
        "assertion": device.assertion(options.json["challenge"].as_str().unwrap(), 0),
        "prf_output": device.prf(options.json["prf_salt"].as_str().unwrap()),
    });
    let reply = api(&app, &who, "POST", "/auth/v1/passkeys", Some(body)).await;
    assert!(!reply.status.is_success());
    let feeds = api(&app, &who, "GET", "/api/v1/feeds", None).await;
    assert_eq!(feeds.code(), "method_required");
}

#[tokio::test]
async fn a_nostr_account_can_add_a_passkey_and_sign_in_with_it() {
    let app = app(Signup::Open);
    let up = nostr_sign_in(&app, &new_keys()).await;
    let who = who_of(&up);
    let device = Device::new(3);
    let added = add_passkey(&app, &who, &device, "Phone").await;
    assert_eq!(added.status, StatusCode::CREATED, "{:?}", added.json);
    let options = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    assert!(
        options.json["user_name"]
            .as_str()
            .unwrap()
            .starts_with("nostr:")
    );
    send(&app, sign_out_call(&who)).await;
    let reply = passkey_sign_in(&app, &device, 0).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    assert_eq!(reply.json["vault"], "open");
}

#[tokio::test]
async fn the_options_name_the_relying_party_and_exclude_known_passkeys() {
    let app = app(Signup::Invite);
    let who = new_user(&app, "a@example.org", PASS_ONE).await;
    let device = Device::new(1);
    let first = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    assert_eq!(first.json["rp_id"], "127.0.0.1");
    assert_eq!(first.json["user_name"], "a@example.org");
    assert_eq!(
        B64.decode(first.json["user_handle"].as_str().unwrap())
            .unwrap()
            .len(),
        16
    );
    assert_eq!(
        B64.decode(first.json["prf_salt"].as_str().unwrap())
            .unwrap()
            .len(),
        32
    );
    assert_ne!(first.json["challenge"], first.json["assertion_challenge"]);
    assert_eq!(first.json["exclude"].as_array().unwrap().len(), 0);
    add_passkey(&app, &who, &device, "Phone").await;
    let second = api(
        &app,
        &who,
        "POST",
        "/auth/v1/passkeys/options",
        Some(json!({})),
    )
    .await;
    assert_eq!(second.json["exclude"][0], device.credential_id().as_str());
    assert_eq!(second.json["user_handle"], first.json["user_handle"]);
}

// Tests of the NIP-46 sign-in (task 035c). The relay and the signer are test
// servers on the computer.

use studio_core::testing::relay::TestRelay;
use studio_core::testing::signer::{Behavior, TestSigner};

struct Remote {
    app: App,
    // The relay and the signer stop when the value goes out of scope.
    relay: TestRelay,
    signer: TestSigner,
}

fn remote_app(signup: Signup, timeout_ms: u64) -> Remote {
    let relay = TestRelay::start();
    let port: u16 = relay.url().rsplit(':').next().unwrap().parse().unwrap();
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let rules = PublicRules::with_test_exceptions(vec![address]);
    let relay_url = relay.url();
    let app = app_with_rules(signup, rules, |options| {
        options.nip46_relays = vec![relay_url];
        options.nip46_timeout = std::time::Duration::from_millis(timeout_ms);
    });
    let signer = TestSigner::start(&relay.url());
    Remote { app, relay, signer }
}

async fn bunker_connect(remote: &Remote) -> String {
    let reply = send(
        &remote.app,
        call(
            "POST",
            "/auth/v1/nip46/connect",
            Some(json!({ "bunker": remote.signer.bunker_uri() })),
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.json);
    assert_eq!(
        reply.json["pubkey"],
        remote.signer.public_key().to_hex().as_str()
    );
    reply.json["connection"].as_str().unwrap().to_owned()
}

async fn nip46_sign_in(app: &App, connection: &str, invite: Option<&str>) -> Reply {
    let path = format!("/auth/v1/nip46/{connection}/signin");
    send(app, call("POST", &path, Some(json!({ "invite": invite })))).await
}

async fn session_json(app: &App, who: &(String, String)) -> Value {
    let mut c = call("GET", "/auth/v1/session", None);
    c.token = Some(&who.0);
    send(app, c).await.json
}

/// The state of the NIP-46 signer in the workspace of an account, asked
/// through the router of the workspace and not through a session.
async fn open_user_of(app: &App, pubkey: &str) -> crate::users::OpenUser {
    let (id, _) = app
        .state
        .accounts
        .account_by_pubkey(pubkey)
        .unwrap()
        .unwrap();
    app.state.users.peek(id).await.expect("an open user")
}

async fn workspace_signer_state(user: &crate::users::OpenUser) -> Value {
    let request = Request::builder()
        .uri("/api/v1/signer")
        .body(Body::empty())
        .unwrap();
    let response = user.router.clone().oneshot(request).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn the_signer_of_a_nip46_sign_in_is_ready_for_signing_and_goes_with_the_session() {
    let remote = remote_app(Signup::Open, 10_000);
    let app = &remote.app;
    let pubkey = remote.signer.public_key().to_hex();
    let connection = bunker_connect(&remote).await;
    let up = nip46_sign_in(app, &connection, None).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    let who = who_of(&up);
    let connects = || {
        remote
            .signer
            .methods()
            .iter()
            .filter(|m| *m == "connect")
            .count()
    };
    let user = open_user_of(app, &pubkey).await;
    let before = connects();
    let state = api(app, &who, "GET", "/api/v1/signer", None).await;
    assert_eq!(state.status, StatusCode::OK, "{:?}", state.json);
    assert_eq!(state.json["state"], "connected");
    assert_eq!(state.json["pubkey"], pubkey.as_str());
    // The signer connected one time only.
    assert_eq!(connects(), before);

    // A sign-out removes the signer from the session map and the workspace.
    let out = send(app, sign_out_call(&who)).await;
    assert_eq!(out.status, StatusCode::OK);
    assert_eq!(app.state.nip46.counts(), (0, 0));
    assert_eq!(workspace_signer_state(&user).await["state"], "none");

    // The sweep of an ended session removes it, too.
    let connection = bunker_connect(&remote).await;
    let again = nip46_sign_in(app, &connection, None).await;
    assert_eq!(again.status, StatusCode::OK, "{:?}", again.json);
    assert_eq!(workspace_signer_state(&user).await["state"], "connected");
    let who = who_of(&again);
    let hash = crate::accounts::sha256_hex(&who.0);
    app.state.accounts.delete_session(&hash).unwrap();
    crate::idle::sweep(&app.state).await;
    assert_eq!(app.state.nip46.counts(), (0, 0));
    assert_eq!(workspace_signer_state(&user).await["state"], "none");
}

#[tokio::test]
async fn a_nip46_sign_up_makes_a_nostr_wrapper_and_a_later_sign_in_and_unlock_work() {
    let remote = remote_app(Signup::Open, 10_000);
    let app = &remote.app;
    let pubkey = remote.signer.public_key().to_hex();
    let connection = bunker_connect(&remote).await;
    let state = send(
        app,
        call("GET", &format!("/auth/v1/nip46/{connection}"), None),
    )
    .await;
    assert_eq!(state.json["state"], "connected");
    assert_eq!(state.json["pubkey"], pubkey.as_str());

    let up = nip46_sign_in(app, &connection, None).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
    assert_eq!(up.json["new_account"], true);
    // The vault stays open after a sign-up.
    assert_eq!(up.json["vault"], "open");
    let who = who_of(&up);
    let wrapper = api(app, &who, "GET", "/api/v1/vault/nostr-wrapper", None).await;
    assert_eq!(wrapper.status, StatusCode::OK, "{:?}", wrapper.json);
    assert_eq!(wrapper.json["pubkey"], pubkey.as_str());
    let ciphertext = wrapper.json["ciphertext"].as_str().unwrap();
    // The signer made the ciphertext, and the vault key is not in it as text.
    let key = api(app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    let key = key.json["key"].as_str().unwrap().to_owned();
    assert!(!ciphertext.contains(&key));
    assert_eq!(session_json(app, &who).await["signer"], true);
    // The connection ID works one time. The signer is now in the session.
    let used = send(
        app,
        call("GET", &format!("/auth/v1/nip46/{connection}"), None),
    )
    .await;
    assert_eq!(used.status, StatusCode::NOT_FOUND);
    assert_eq!(app.state.nip46.counts(), (0, 1));

    // A sign-out removes the signer from memory.
    let out = send(app, sign_out_call(&who)).await;
    assert_eq!(out.status, StatusCode::OK);
    assert_eq!(app.state.nip46.counts(), (0, 0));

    let connection = bunker_connect(&remote).await;
    let again = nip46_sign_in(app, &connection, None).await;
    assert_eq!(again.status, StatusCode::OK, "{:?}", again.json);
    assert_eq!(again.json["new_account"], false);
    assert_eq!(again.json["vault"], "locked");
    let who = who_of(&again);
    let locked = api(app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(locked.status, StatusCode::LOCKED);
    let open = api(app, &who, "POST", "/auth/v1/nip46/unlock", None).await;
    assert_eq!(open.status, StatusCode::OK, "{:?}", open.json);
    assert_eq!(open.json["vault"], "open");
    let key_again = api(app, &who, "POST", "/api/v1/vault/signer-key", None).await;
    assert_eq!(key_again.json["key"], key.as_str());
    let methods = remote.signer.methods();
    for method in [
        "connect",
        "get_public_key",
        "sign_event",
        "nip44_encrypt",
        "nip44_decrypt",
    ] {
        assert!(methods.contains(&method.to_owned()), "{method} {methods:?}");
    }
}

#[tokio::test]
async fn a_nostrconnect_address_connects_a_signer_and_gives_a_qr_code() {
    let remote = remote_app(Signup::Open, 10_000);
    let app = &remote.app;
    let start = send(app, call("POST", "/auth/v1/nip46/connect", Some(json!({})))).await;
    assert_eq!(start.status, StatusCode::OK, "{:?}", start.json);
    let uri = start.json["uri"].as_str().unwrap();
    assert!(uri.starts_with("nostrconnect://"));
    assert!(start.json["qr_svg"].as_str().unwrap().contains("<svg"));
    let connection = start.json["connection"].as_str().unwrap();
    // The signer has not answered, so the state is pending.
    let path = format!("/auth/v1/nip46/{connection}");
    let waiting = send(app, call("GET", &path, None)).await;
    assert_eq!(waiting.json["state"], "pending");
    remote.signer.connect(uri);
    let mut state = Value::Null;
    for _ in 0..10 {
        state = send(app, call("GET", &path, None)).await.json;
        if state["state"] != "pending" {
            break;
        }
    }
    assert_eq!(state["state"], "connected", "{state:?}");
    assert_eq!(
        state["pubkey"],
        remote.signer.public_key().to_hex().as_str()
    );
    let up = nip46_sign_in(app, connection, None).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
}

#[tokio::test]
async fn a_stopped_connection_goes_away_and_an_old_one_expires() {
    let remote = remote_app(Signup::Open, 10_000);
    let app = &remote.app;
    let start = send(app, call("POST", "/auth/v1/nip46/connect", Some(json!({})))).await;
    let connection = start.json["connection"].as_str().unwrap().to_owned();
    let path = format!("/auth/v1/nip46/{connection}");
    assert_eq!(app.state.nip46.counts().0, 1);
    let stop = send(app, call("DELETE", &path, None)).await;
    assert_eq!(stop.status, StatusCode::NO_CONTENT);
    assert_eq!(
        send(app, call("GET", &path, None)).await.status,
        StatusCode::NOT_FOUND
    );
    // A connection with no use for five minutes goes away.
    let connection = bunker_connect(&remote).await;
    app.state.nip46.age_all(std::time::Duration::from_secs(301));
    let path = format!("/auth/v1/nip46/{connection}");
    assert_eq!(
        send(app, call("GET", &path, None)).await.status,
        StatusCode::NOT_FOUND
    );
    let gone = nip46_sign_in(app, &connection, None).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_eq!(app.state.nip46.counts(), (0, 0));
}

#[tokio::test]
async fn the_policy_public_refuses_a_loopback_relay_with_422() {
    // The default test app has no exception for the relay.
    let relay = TestRelay::start();
    let relay_url = relay.url();
    let app = app_with(Signup::Open, |options| {
        options.nip46_relays = vec![relay_url];
    });
    let signer = TestSigner::start(&relay.url());
    let start = send(
        &app,
        call("POST", "/auth/v1/nip46/connect", Some(json!({}))),
    )
    .await;
    assert_eq!(
        start.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{:?}",
        start.json
    );
    assert_eq!(start.code(), "address_not_public");
    let bunker = send(
        &app,
        call(
            "POST",
            "/auth/v1/nip46/connect",
            Some(json!({ "bunker": signer.bunker_uri() })),
        ),
    )
    .await;
    assert_eq!(bunker.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bunker.code(), "address_not_public");
    assert_eq!(app.state.nip46.counts(), (0, 0));
}

#[tokio::test]
async fn a_signer_that_does_not_answer_gives_504() {
    let remote = remote_app(Signup::Open, 700);
    let app = &remote.app;
    remote.signer.set_behavior(Behavior::Silent);
    let bunker = send(
        app,
        call(
            "POST",
            "/auth/v1/nip46/connect",
            Some(json!({ "bunker": remote.signer.bunker_uri() })),
        ),
    )
    .await;
    assert_eq!(
        bunker.status,
        StatusCode::GATEWAY_TIMEOUT,
        "{:?}",
        bunker.json
    );
    assert_eq!(bunker.code(), "signer_timeout");
    // A signer that stops later gives 504 at the sign-in, and the account
    // is not made.
    remote.signer.set_behavior(Behavior::Normal);
    let connection = bunker_connect(&remote).await;
    remote.signer.set_behavior(Behavior::Silent);
    let timed_out = nip46_sign_in(app, &connection, None).await;
    assert_eq!(
        timed_out.status,
        StatusCode::GATEWAY_TIMEOUT,
        "{:?}",
        timed_out.json
    );
    assert_eq!(timed_out.code(), "signer_timeout");
    assert!(
        app.state
            .accounts
            .account_by_pubkey(&remote.signer.public_key().to_hex())
            .unwrap()
            .is_none()
    );
    // The connection is free again, and the next sign-in works.
    remote.signer.set_behavior(Behavior::Normal);
    let up = nip46_sign_in(app, &connection, None).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
}

#[tokio::test]
async fn a_wrong_event_from_the_signer_gives_no_session() {
    let remote = remote_app(Signup::Open, 5_000);
    let app = &remote.app;
    let connection = bunker_connect(&remote).await;
    remote.signer.set_behavior(Behavior::BadSignature);
    let bad = nip46_sign_in(app, &connection, None).await;
    assert_eq!(bad.status, StatusCode::BAD_GATEWAY, "{:?}", bad.json);
    assert_eq!(bad.code(), "signer_invalid");
    remote.signer.set_behavior(Behavior::SignWithOtherKey);
    let other = nip46_sign_in(app, &connection, None).await;
    assert_eq!(other.code(), "signer_invalid");
    assert!(bad.headers.get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn an_invite_server_asks_for_the_invite_and_the_connection_works_again() {
    let remote = remote_app(Signup::Invite, 5_000);
    let app = &remote.app;
    let connection = bunker_connect(&remote).await;
    let none = nip46_sign_in(app, &connection, None).await;
    assert_eq!(none.status, StatusCode::FORBIDDEN);
    assert_eq!(none.code(), "invite_required");
    let invite = app.state.accounts.create_invite().unwrap();
    let up = nip46_sign_in(app, &connection, Some(&invite)).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
}

#[tokio::test]
async fn a_different_session_cannot_use_the_signer_of_a_session() {
    let remote = remote_app(Signup::Open, 5_000);
    let app = &remote.app;
    let connection = bunker_connect(&remote).await;
    let up = nip46_sign_in(app, &connection, None).await;
    let who = who_of(&up);
    // The lock of the first session does not matter. A second user has no signer.
    let other = new_user_open(app, "other@example.org").await;
    api(app, &who, "POST", "/api/v1/vault/lock", None).await;
    let refused = api(app, &other, "POST", "/auth/v1/nip46/unlock", None).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(refused.code(), "signer_not_connected");
    // The old connection ID gives nothing to a second sign-in.
    let reuse = nip46_sign_in(app, &connection, None).await;
    assert_eq!(reuse.status, StatusCode::NOT_FOUND);
    // The unlock needs a session and the CSRF header.
    let none = send(app, call("POST", "/auth/v1/nip46/unlock", None)).await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
    let mut no_csrf = call("POST", "/auth/v1/nip46/unlock", None);
    no_csrf.token = Some(&who.0);
    assert_eq!(send(app, no_csrf).await.status, StatusCode::FORBIDDEN);
}

async fn new_user_open(app: &App, email: &str) -> (String, String) {
    new_user(app, email, PASS_ONE).await
}

#[tokio::test]
async fn an_account_deletion_removes_the_signer_and_the_sweep_removes_an_ended_session() {
    let remote = remote_app(Signup::Open, 5_000);
    let app = &remote.app;
    let connection = bunker_connect(&remote).await;
    let up = nip46_sign_in(app, &connection, None).await;
    let who = who_of(&up);
    assert_eq!(app.state.nip46.counts(), (0, 1));
    // A session that the database lost loses its signer at the next sweep.
    app.state
        .accounts
        .delete_session(&crate::accounts::sha256_hex(&who.0))
        .unwrap();
    crate::idle::sweep(&app.state).await;
    assert_eq!(app.state.nip46.counts(), (0, 0));
    let connection = bunker_connect(&remote).await;
    let again = nip46_sign_in(app, &connection, None).await;
    let who = who_of(&again);
    let npub = {
        use nostr::prelude::ToBech32;
        remote.signer.public_key().to_bech32().unwrap()
    };
    let open = api(app, &who, "POST", "/auth/v1/nip46/unlock", None).await;
    assert_eq!(open.status, StatusCode::OK, "{:?}", open.json);
    let deleted = api(
        app,
        &who,
        "POST",
        "/auth/v1/account/delete",
        Some(json!({ "confirm": npub })),
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{:?}", deleted.json);
    assert_eq!(app.state.nip46.counts(), (0, 0));
    drop(remote.relay);
}

#[tokio::test]
async fn a_wrong_decrypt_at_nip46_sign_up_leaves_no_account_and_the_invite_works_again() {
    let remote = remote_app(Signup::Invite, 5_000);
    let app = &remote.app;
    let connection = bunker_connect(&remote).await;
    let invite = app.state.accounts.create_invite().unwrap();
    remote.signer.set_behavior(Behavior::WrongDecrypt);
    let bad = nip46_sign_in(app, &connection, Some(&invite)).await;
    assert_eq!(bad.status, StatusCode::BAD_GATEWAY, "{:?}", bad.json);
    assert_eq!(bad.code(), "signer_invalid");
    assert!(bad.headers.get(header::SET_COOKIE).is_none());
    let pubkey = remote.signer.public_key().to_hex();
    assert!(
        app.state
            .accounts
            .account_by_pubkey(&pubkey)
            .unwrap()
            .is_none()
    );
    assert_eq!(app.state.nip46.counts().1, 0);
    // The invite is free again, and the same connection signs up.
    remote.signer.set_behavior(Behavior::Normal);
    let up = nip46_sign_in(app, &connection, Some(&invite)).await;
    assert_eq!(up.status, StatusCode::CREATED, "{:?}", up.json);
}

#[tokio::test]
async fn the_fifth_pending_connection_of_one_address_gets_429() {
    let remote = remote_app(Signup::Open, 5_000);
    let app = &remote.app;
    let mut ids = Vec::new();
    for _ in 0..4 {
        let ok = send(app, call("POST", "/auth/v1/nip46/connect", Some(json!({})))).await;
        assert_eq!(ok.status, StatusCode::OK, "{:?}", ok.json);
        ids.push(ok.json["connection"].as_str().unwrap().to_owned());
    }
    let fifth = send(app, call("POST", "/auth/v1/nip46/connect", Some(json!({})))).await;
    assert_eq!(fifth.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(fifth.code(), "too_many_attempts");
    let mut other = call("POST", "/auth/v1/nip46/connect", Some(json!({})));
    other.addr = "10.0.0.2:4000";
    let ok = send(app, other).await;
    assert_eq!(ok.status, StatusCode::OK, "{:?}", ok.json);
    ids.push(ok.json["connection"].as_str().unwrap().to_owned());
    // End the waits, so that no blocking thread stays.
    for id in ids {
        send(app, call("DELETE", &format!("/auth/v1/nip46/{id}"), None)).await;
    }
}
