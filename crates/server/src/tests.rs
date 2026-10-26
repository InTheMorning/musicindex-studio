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
    let data = tempfile::tempdir().unwrap();
    let options = ServeOptions {
        data_dir: data.path().to_path_buf(),
        listen: "127.0.0.1:8080".parse().unwrap(),
        origin: Origin::parse(ORIGIN).unwrap(),
        signup,
        client_address_header: None,
        // Port 9 answers nothing, so no test reaches a real host.
        musicindex: Some("http://127.0.0.1:9".to_owned()),
        hub: Some("http://127.0.0.1:9/pubnotify".to_owned()),
    };
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
    let body = match call.body {
        Some(body) => {
            builder = builder.header(header::CONTENT_TYPE, call.content_type);
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
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
