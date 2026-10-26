//! A small HTTP server for tests. It needs no network.
//!
//! The `testing` feature makes it available to the tests of other crates.

pub mod blossom;
pub mod fake_s3;
pub mod hub;
pub mod relay;
pub mod sshd;

pub use hub::{HubRequest, TestHub};

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

/// One response of the test server: a path, a status and a body.
pub type Route = (&'static str, u16, Vec<u8>);

/// The response of one route: a status, a body, and the additional headers.
#[derive(Debug, Clone)]
struct Response {
    status: u16,
    body: Vec<u8>,
    content_type: Option<String>,
    /// When false, a `HEAD` response has no `Content-Length`, and a `GET`
    /// with a `Range` gets a 206 response with `Content-Range`.
    head_length: bool,
}

type Routes = Arc<Mutex<HashMap<String, Response>>>;

/// An HTTP server on a loopback port. It answers each `GET` and `HEAD` with
/// the route for the path, without the query, or with status 404. A route
/// with a 3xx status is a redirect, and its body is the `Location` address.
#[derive(Debug)]
pub struct TestServer {
    port: u16,
    routes: Routes,
}

impl TestServer {
    /// Start a server with these routes.
    ///
    /// # Panics
    ///
    /// Panics when the server cannot listen on a loopback port.
    #[must_use]
    pub fn start(routes: Vec<Route>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let routes: HashMap<String, Response> = routes
            .into_iter()
            .map(|(path, status, body)| {
                let response = Response {
                    status,
                    body,
                    content_type: None,
                    head_length: true,
                };
                (path.to_owned(), response)
            })
            .collect();
        let routes = Arc::new(Mutex::new(routes));
        let shared = Arc::clone(&routes);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                // Read the headers to the empty line.
                let mut header = String::new();
                let mut range = false;
                while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                    range |= header.to_ascii_lowercase().starts_with("range:");
                    header.clear();
                }
                let method = line.split_whitespace().next().unwrap_or("GET").to_owned();
                let path = line.split_whitespace().nth(1).unwrap_or("/");
                let path = path.split('?').next().unwrap_or(path).to_owned();
                let route = shared
                    .lock()
                    .ok()
                    .and_then(|r| r.get(&path).cloned())
                    .unwrap_or(Response {
                        status: 404,
                        body: b"not found".to_vec(),
                        content_type: None,
                        head_length: true,
                    });
                let _ = stream.write_all(&route.answer(&method, range));
            }
        });
        Self { port, routes }
    }

    /// The full address of a path on this server.
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    /// Add or replace a route while the server runs.
    ///
    /// # Panics
    ///
    /// Panics when the route table lock is poisoned.
    pub fn set(&self, path: &str, status: u16, body: Vec<u8>) {
        self.put(
            path,
            Response {
                status,
                body,
                content_type: None,
                head_length: true,
            },
        );
    }

    /// Add a file route with a `Content-Type`. When `head_length` is false,
    /// a `HEAD` response has no `Content-Length`, and only a range request
    /// gives the size.
    pub fn set_file(&self, path: &str, body: Vec<u8>, content_type: &str, head_length: bool) {
        self.put(
            path,
            Response {
                status: 200,
                body,
                content_type: Some(content_type.to_owned()),
                head_length,
            },
        );
    }

    fn put(&self, path: &str, response: Response) {
        self.routes
            .lock()
            .expect("the route table")
            .insert(path.to_owned(), response);
    }
}

impl Response {
    /// The bytes of the HTTP response to `method`.
    fn answer(&self, method: &str, range: bool) -> Vec<u8> {
        let mut head = String::new();
        let mut status = self.status;
        let mut body = self.body.clone();
        if (300..400).contains(&status) {
            let _ = write!(head, "Location: {}\r\n", String::from_utf8_lossy(&body));
            body.clear();
        }
        if let Some(content_type) = &self.content_type {
            let _ = write!(head, "Content-Type: {content_type}\r\n");
        }
        let total = body.len();
        if range && !self.head_length && status == 200 && total > 0 {
            status = 206;
            let _ = write!(head, "Content-Range: bytes 0-0/{total}\r\n");
            body.truncate(1);
        }
        if method == "HEAD" {
            if self.head_length {
                let _ = write!(head, "Content-Length: {total}\r\n");
            }
            body.clear();
        } else {
            let _ = write!(head, "Content-Length: {}\r\n", body.len());
        }
        let mut bytes =
            format!("HTTP/1.1 {status} X\r\n{head}Connection: close\r\n\r\n").into_bytes();
        bytes.extend_from_slice(&body);
        bytes
    }
}

/// What [`RangeServer`] records and does.
#[derive(Debug, Default)]
struct RangeState {
    /// The number of bytes after which the next answer ends, with no more
    /// bytes. The answer still states its full length.
    cut_after: Option<usize>,
    /// The `Range` header of each request, in order.
    ranges: Vec<Option<String>>,
}

/// An HTTP server on a loopback port for one file. It answers a range
/// request with 206, and it can end an answer early, to test a download
/// that continues.
#[derive(Debug)]
pub struct RangeServer {
    port: u16,
    state: Arc<Mutex<RangeState>>,
}

impl RangeServer {
    /// Start a server that serves `body` at each path.
    ///
    /// # Panics
    ///
    /// Panics when the server cannot listen on a loopback port.
    #[must_use]
    pub fn start(body: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let state = Arc::new(Mutex::new(RangeState::default()));
        let shared = Arc::clone(&state);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                let mut range = None;
                let mut header = String::new();
                while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                    if header.to_ascii_lowercase().starts_with("range:") {
                        range = header.split_once(':').map(|(_, v)| v.trim().to_owned());
                    }
                    header.clear();
                }
                let start = range
                    .as_deref()
                    .and_then(|r| r.strip_prefix("bytes="))
                    .and_then(|r| r.strip_suffix('-'))
                    .and_then(|r| r.parse::<usize>().ok());
                let cut = {
                    let mut state = shared.lock().expect("the state");
                    state.ranges.push(range);
                    state.cut_after
                };
                let total = body.len();
                let first = start.unwrap_or(0).min(total);
                let mut head = if start.is_some() {
                    format!(
                        "HTTP/1.1 206 X\r\nContent-Range: bytes {first}-{}/{total}\r\n",
                        total.saturating_sub(1)
                    )
                } else {
                    "HTTP/1.1 200 X\r\n".to_owned()
                };
                let _ = write!(
                    head,
                    "Content-Length: {}\r\nConnection: close\r\n\r\n",
                    total - first
                );
                let mut bytes = head.into_bytes();
                let end = cut.map_or(total, |n| (first + n).min(total));
                bytes.extend_from_slice(&body[first..end]);
                let _ = stream.write_all(&bytes);
            }
        });
        Self { port, state }
    }

    /// The full address of a path on this server.
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    /// End each later answer after `bytes` bytes of the body, or serve the
    /// whole body again with `None`.
    ///
    /// # Panics
    ///
    /// Panics when the state lock is poisoned.
    pub fn cut_after(&self, bytes: Option<usize>) {
        self.state.lock().expect("the state").cut_after = bytes;
    }

    /// The `Range` header of each request so far, in order.
    ///
    /// # Panics
    ///
    /// Panics when the state lock is poisoned.
    #[must_use]
    pub fn ranges(&self) -> Vec<Option<String>> {
        self.state.lock().expect("the state").ranges.clone()
    }
}
