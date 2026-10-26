//! A small Blossom server for tests. It runs in the test process.
//!
//! It has the routes of BUD-01, BUD-02 and BUD-04 that the Blossom
//! connector uses. It checks the authorization events of BUD-11 on each
//! `PUT` and `DELETE`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use nostr::prelude::{Event, Tag};
use serde_json::json;

use crate::digest::sha256_hex;

/// The kind of a Blossom authorization event (BUD-11).
const AUTH_KIND: u16 = 24_242;

#[derive(Debug, Default)]
struct State {
    /// The bytes and the content type of each blob, by SHA-256.
    blobs: BTreeMap<String, (Vec<u8>, String)>,
    deletes: Vec<String>,
    refuse_uploads: bool,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    stop: AtomicBool,
}

/// A Blossom server on a loopback port. It stops when it is dropped.
#[derive(Debug)]
pub struct TestBlossom {
    port: u16,
    shared: Arc<Shared>,
}

impl TestBlossom {
    /// Start a server.
    ///
    /// # Panics
    ///
    /// Panics when the server cannot listen on a loopback port.
    #[must_use]
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            stop: AtomicBool::new(false),
        });
        let accept = Arc::clone(&shared);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if accept.stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let connection = Arc::clone(&accept);
                thread::spawn(move || serve(&connection, stream));
            }
        });
        Self { port, shared }
    }

    /// The address of the server, for example `http://127.0.0.1:41234`.
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// When true, each `PUT` gets 403.
    pub fn refuse_uploads(&self, refuse: bool) {
        self.with(|state| state.refuse_uploads = refuse);
    }

    /// The SHA-256 of each stored blob, sorted.
    #[must_use]
    pub fn blobs(&self) -> Vec<String> {
        self.with(|state| state.blobs.keys().cloned().collect())
    }

    /// The SHA-256 of each `DELETE` request that the server got, in sequence.
    #[must_use]
    pub fn deletes(&self) -> Vec<String> {
        self.with(|state| state.deletes.clone())
    }

    fn with<T>(&self, action: impl FnOnce(&mut State) -> T) -> T {
        action(&mut self.shared.state.lock().expect("the state"))
    }
}

impl Drop for TestBlossom {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A connection wakes the accept loop so that it can end.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

/// One request. Header names are in lowercase.
struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut words = line.split_whitespace();
    let method = words.next()?.to_owned();
    let target = words.next()?;
    let path = target.split('?').next().unwrap_or(target).to_owned();
    let mut headers = BTreeMap::new();
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).ok()?;
        if header.trim().is_empty() {
            break;
        }
        let (name, value) = header.split_once(':')?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    let length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body).ok()?;
    Some(Request {
        method,
        path,
        headers,
        body,
    })
}

/// One response.
struct Reply {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn unauthorized(reason: &str) -> Self {
        let mut reply = Self::status(401);
        reply.headers.push(("X-Reason", reason.to_owned()));
        reply
    }
}

fn serve(shared: &Shared, mut stream: TcpStream) {
    let Some(request) = read_request(&stream) else {
        return;
    };
    let reply = route(shared, &request);
    let mut head = format!("HTTP/1.1 {} X\r\n", reply.status);
    for (name, value) in &reply.headers {
        let _ = write!(head, "{name}: {value}\r\n");
    }
    let _ = write!(
        head,
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        reply.body.len()
    );
    let mut bytes = head.into_bytes();
    // A HEAD response has the length of the body but no body.
    if request.method != "HEAD" {
        bytes.extend_from_slice(&reply.body);
    }
    let _ = stream.write_all(&bytes);
}

/// The SHA-256 in a path of the form `/<sha256>` or `/<sha256>.<ext>`.
fn path_hash(path: &str) -> Option<&str> {
    let name = path.strip_prefix('/')?;
    let hash = name.split('.').next()?;
    (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())).then_some(hash)
}

fn route(shared: &Shared, request: &Request) -> Reply {
    match (request.method.as_str(), request.path.as_str()) {
        ("PUT", "/upload") => {
            if refused(shared) {
                return Reply::status(403);
            }
            let kind = request
                .headers
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".to_owned());
            store(shared, request, request.body.clone(), &kind)
        }
        ("PUT", "/mirror") => {
            if refused(shared) {
                return Reply::status(403);
            }
            let fetched = serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|v| v["url"].as_str().map(str::to_owned))
                .and_then(|url| fetch(&url));
            match fetched {
                Some((bytes, kind)) => store(shared, request, bytes, &kind),
                None => Reply::status(502),
            }
        }
        ("PUT", _) => Reply::status(404),
        ("HEAD" | "GET", path) => {
            let state = shared.state.lock().expect("the state");
            let Some((bytes, kind)) = path_hash(path).and_then(|h| state.blobs.get(h)) else {
                return Reply::status(404);
            };
            Reply {
                status: 200,
                headers: vec![("Content-Type", kind.clone())],
                body: bytes.clone(),
            }
        }
        ("DELETE", path) => {
            let Some(hash) = path_hash(path) else {
                return Reply::status(404);
            };
            let mut state = shared.state.lock().expect("the state");
            state.deletes.push(hash.to_owned());
            if let Err(reason) = check_token(request, "delete", hash) {
                return Reply::unauthorized(&reason);
            }
            if state.blobs.remove(hash).is_some() {
                Reply::status(200)
            } else {
                Reply::status(404)
            }
        }
        _ => Reply::status(405),
    }
}

fn refused(shared: &Shared) -> bool {
    shared.state.lock().expect("the state").refuse_uploads
}

/// Get the blob of a mirror request. Gives its bytes and content type.
fn fetch(url: &str) -> Option<(Vec<u8>, String)> {
    let mut response = ureq::get(url).call().ok()?;
    let kind = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_owned();
    let bytes = response.body_mut().read_to_vec().ok()?;
    Some((bytes, kind))
}

/// Check the token and store a blob. The reply is the blob descriptor.
fn store(shared: &Shared, request: &Request, bytes: Vec<u8>, kind: &str) -> Reply {
    let hash = sha256_hex(&[&bytes]);
    if let Err(reason) = check_token(request, "upload", &hash) {
        return Reply::unauthorized(&reason);
    }
    let host = request
        .headers
        .get("host")
        .map_or("127.0.0.1", String::as_str);
    let size = bytes.len();
    let created = shared
        .state
        .lock()
        .expect("the state")
        .blobs
        .insert(hash.clone(), (bytes, kind.to_owned()))
        .is_none();
    let descriptor = json!({
        "url": format!("http://{host}/{hash}.{}", extension(kind)),
        "sha256": hash,
        "size": size,
        "type": kind,
        "uploaded": now(),
    });
    Reply {
        status: if created { 201 } else { 200 },
        headers: vec![("Content-Type", "application/json".to_owned())],
        body: descriptor.to_string().into_bytes(),
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The file extension for a content type.
fn extension(kind: &str) -> &'static str {
    match kind.split(';').next().unwrap_or("").trim() {
        "audio/mpeg" => "mp3",
        "audio/flac" | "audio/x-flac" => "flac",
        "audio/wav" | "audio/x-wav" => "wav",
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "text/html" => "html",
        "text/css" => "css",
        "text/plain" => "txt",
        "application/xml" | "text/xml" | "application/rss+xml" => "xml",
        "application/json" => "json",
        _ => "bin",
    }
}

/// Check the `Authorization` header of a request (BUD-11). The error is
/// the reason for the 401 response.
fn check_token(request: &Request, action: &str, hash: &str) -> Result<(), String> {
    let header = request
        .headers
        .get("authorization")
        .ok_or("no Authorization header")?;
    let token = header
        .strip_prefix("Nostr ")
        .ok_or("the scheme is not Nostr")?;
    let json = URL_SAFE_NO_PAD
        .decode(token.trim())
        .map_err(|_| "the token is not Base64url")?;
    let event: Event = serde_json::from_slice(&json).map_err(|_| "the token is not an event")?;
    if event.kind.as_u16() != AUTH_KIND {
        return Err("the event kind is not 24242".to_owned());
    }
    event
        .verify()
        .map_err(|_| "the event ID or signature is not correct")?;
    let tag = |name: &str| -> Vec<&[String]> {
        event
            .tags
            .iter()
            .map(Tag::as_slice)
            .filter(|t| t.first().is_some_and(|n| n == name))
            .collect()
    };
    let expires = tag("expiration")
        .first()
        .and_then(|t| t.get(1))
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or("no expiration tag")?;
    if expires <= now() {
        return Err("the token expired".to_owned());
    }
    if !tag("t")
        .iter()
        .any(|t| t.get(1).is_some_and(|v| v == action))
    {
        return Err(format!("the t tag is not {action}"));
    }
    if !tag("x").iter().any(|t| t.get(1).is_some_and(|v| v == hash)) {
        return Err("no x tag has the SHA-256 of the blob".to_owned());
    }
    let servers = tag("server");
    if !servers.is_empty() {
        let host = request.headers.get("host").map_or("", String::as_str);
        let name = host.rsplit_once(':').map_or(host, |(n, _)| n);
        if !servers.iter().any(|t| t.get(1).is_some_and(|v| v == name)) {
            return Err("no server tag has the host name".to_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr_events::sign_with_keys;
    use nostr::prelude::{Keys, Kind, SecretKey, Timestamp, UnsignedEvent};

    struct Seen {
        status: u16,
        headers: BTreeMap<String, String>,
        body: Vec<u8>,
    }

    fn call(method: &str, url: &str, headers: &[(&str, &str)], body: &[u8]) -> Seen {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        let mut builder = ureq::http::Request::builder().method(method).uri(url);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let response = agent
            .run(builder.body(body.to_vec()).expect("a request"))
            .expect("a response");
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(n, v)| (n.as_str().to_owned(), v.to_str().unwrap_or("").to_owned()))
            .collect();
        let body = response.into_body().read_to_vec().expect("a body");
        Seen {
            status,
            headers,
            body,
        }
    }

    fn keys() -> Keys {
        Keys::new(SecretKey::from_slice(&[1; 32]).expect("a secret key"))
    }

    /// An `Authorization` value for `action`, with these tags.
    fn token(action: &str, hash: &str, expires: u64, extra: &[Tag]) -> String {
        let keys = keys();
        let mut tags = vec![
            Tag::parse(["t", action]).expect("a tag"),
            Tag::parse(["x", hash]).expect("a tag"),
            Tag::expiration(Timestamp::from(expires)),
        ];
        tags.extend_from_slice(extra);
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(now()),
            Kind::Custom(AUTH_KIND),
            tags,
            "Studio test",
        );
        let event = sign_with_keys(unsigned, &keys).expect("a signed event");
        format!("Nostr {}", URL_SAFE_NO_PAD.encode(event.as_json()))
    }

    fn valid(action: &str, hash: &str) -> String {
        token(action, hash, now() + 3600, &[])
    }

    fn upload(server: &TestBlossom, body: &[u8], auth: &str) -> Seen {
        call(
            "PUT",
            &format!("{}/upload", server.url()),
            &[("Authorization", auth), ("Content-Type", "audio/mpeg")],
            body,
        )
    }

    #[test]
    fn an_upload_gives_201_and_then_200() {
        let server = TestBlossom::start();
        let hash = sha256_hex(&[b"abc"]);
        let auth = valid("upload", &hash);
        let first = upload(&server, b"abc", &auth);
        assert_eq!(first.status, 201);
        let descriptor: serde_json::Value = serde_json::from_slice(&first.body).expect("JSON");
        assert_eq!(descriptor["sha256"], hash.as_str());
        assert_eq!(descriptor["size"], 3);
        assert_eq!(descriptor["type"], "audio/mpeg");
        assert_eq!(
            descriptor["url"],
            format!("{}/{hash}.mp3", server.url()).as_str()
        );
        assert!(descriptor["uploaded"].as_u64().is_some());
        assert_eq!(upload(&server, b"abc", &auth).status, 200);
        assert_eq!(server.blobs(), std::slice::from_ref(&hash));
        let get = call("GET", &format!("{}/{hash}.mp3", server.url()), &[], b"");
        assert_eq!((get.status, get.body.as_slice()), (200, &b"abc"[..]));
        let head = call("HEAD", &format!("{}/{hash}", server.url()), &[], b"");
        assert_eq!(head.status, 200);
        assert_eq!(head.headers["content-length"], "3");
        assert_eq!(head.headers["content-type"], "audio/mpeg");
        let missing = format!("{}/{}", server.url(), sha256_hex(&[b"no"]));
        assert_eq!(call("GET", &missing, &[], b"").status, 404);
        assert_eq!(call("HEAD", &missing, &[], b"").status, 404);
    }

    #[test]
    fn a_bad_token_gives_401_with_a_reason() {
        let server = TestBlossom::start();
        let hash = sha256_hex(&[b"abc"]);
        let other = sha256_hex(&[b"other"]);
        let cases = [
            (token("upload", &other, now() + 3600, &[]), "x tag"),
            (token("upload", &hash, now() - 10, &[]), "expired"),
            (valid("delete", &hash), "t tag"),
            (
                token(
                    "upload",
                    &hash,
                    now() + 3600,
                    &[Tag::parse(["server", "example.org"]).expect("a tag")],
                ),
                "server tag",
            ),
            ("Bearer abc".to_owned(), "Nostr"),
        ];
        for (auth, reason) in cases {
            let seen = upload(&server, b"abc", &auth);
            assert_eq!(seen.status, 401, "{reason}");
            assert!(seen.headers["x-reason"].contains(reason), "{reason}");
        }
        let none = call("PUT", &format!("{}/upload", server.url()), &[], b"abc");
        assert_eq!(none.status, 401);
        assert_eq!(server.blobs(), Vec::<String>::new());
    }

    #[test]
    fn a_server_tag_with_the_host_name_passes() {
        let server = TestBlossom::start();
        let hash = sha256_hex(&[b"abc"]);
        let tag = Tag::parse(["server", "127.0.0.1"]).expect("a tag");
        let auth = token("upload", &hash, now() + 3600, &[tag]);
        assert_eq!(upload(&server, b"abc", &auth).status, 201);
    }

    #[test]
    fn refuse_uploads_gives_403() {
        let server = TestBlossom::start();
        server.refuse_uploads(true);
        let auth = valid("upload", &sha256_hex(&[b"abc"]));
        assert_eq!(upload(&server, b"abc", &auth).status, 403);
        assert_eq!(server.blobs(), Vec::<String>::new());
    }

    #[test]
    fn a_mirror_copies_a_blob_between_servers() {
        let source = TestBlossom::start();
        let target = TestBlossom::start();
        let hash = sha256_hex(&[b"abc"]);
        let auth = valid("upload", &hash);
        let first = upload(&source, b"abc", &auth);
        let descriptor: serde_json::Value = serde_json::from_slice(&first.body).expect("JSON");
        let body = json!({"url": descriptor["url"]}).to_string();
        let mirror = |auth: &str| {
            call(
                "PUT",
                &format!("{}/mirror", target.url()),
                &[("Authorization", auth)],
                body.as_bytes(),
            )
        };
        let seen = mirror(&auth);
        assert_eq!(seen.status, 201);
        assert_eq!(target.blobs(), std::slice::from_ref(&hash));
        let copy: serde_json::Value = serde_json::from_slice(&seen.body).expect("JSON");
        assert_eq!(copy["type"], "audio/mpeg");
        assert_eq!(mirror(&auth).status, 200);
        let wrong = mirror(&valid("upload", &sha256_hex(&[b"other"])));
        assert_eq!(wrong.status, 401);
    }

    #[test]
    fn a_delete_removes_the_blob() {
        let server = TestBlossom::start();
        let hash = sha256_hex(&[b"abc"]);
        upload(&server, b"abc", &valid("upload", &hash));
        let url = format!("{}/{hash}", server.url());
        let refused = call(
            "DELETE",
            &url,
            &[("Authorization", &valid("upload", &hash))],
            b"",
        );
        assert_eq!(refused.status, 401);
        assert_eq!(server.blobs().len(), 1);
        let auth = valid("delete", &hash);
        assert_eq!(
            call("DELETE", &url, &[("Authorization", &auth)], b"").status,
            200
        );
        assert_eq!(server.blobs(), Vec::<String>::new());
        assert_eq!(
            call("DELETE", &url, &[("Authorization", &auth)], b"").status,
            404
        );
        assert_eq!(server.deletes(), [hash.clone(), hash.clone(), hash]);
    }
}
