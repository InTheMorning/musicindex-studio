//! A small Podcast Index hub for tests. It runs in the test process.
//!
//! It answers `GET /pubnotify` and records the query values of each request
//! (ADR 0052 item 1).

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// The body of a successful answer.
const ACCEPTED: &str = r#"{"status":"true","description":"Feed marked for immediate update."}"#;

/// The body of a refusal.
const REFUSED: &str = r#"{"status":"false","description":"refused by the test hub"}"#;

/// The decoded query values of one request to the hub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubRequest {
    /// The `url` parameter, when the request has it.
    pub url: Option<String>,
    /// The `podping` parameter, when the request has it.
    pub podping: Option<String>,
}

#[derive(Debug, Default)]
struct State {
    requests: Vec<HubRequest>,
    fail: u32,
    refuse: u32,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    stop: AtomicBool,
}

/// A hub on a loopback port. It stops when it is dropped.
#[derive(Debug)]
pub struct TestHub {
    port: u16,
    shared: Arc<Shared>,
}

impl TestHub {
    /// Start a hub.
    ///
    /// # Panics
    ///
    /// Panics when the hub cannot listen on a loopback port.
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

    /// The address of the hub, for example `http://127.0.0.1:41234/pubnotify`.
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/pubnotify", self.port)
    }

    /// The requests that the hub got, in sequence.
    #[must_use]
    pub fn requests(&self) -> Vec<HubRequest> {
        self.with(|state| state.requests.clone())
    }

    /// The next `count` requests get the status 500.
    pub fn fail_next(&self, count: u32) {
        self.with(|state| state.fail = count);
    }

    /// The next `count` requests get the status 200 and a refusal.
    pub fn refuse_next(&self, count: u32) {
        self.with(|state| state.refuse = count);
    }

    fn with<T>(&self, action: impl FnOnce(&mut State) -> T) -> T {
        action(&mut self.shared.state.lock().expect("the state"))
    }
}

impl Drop for TestHub {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A connection wakes the accept loop so that it can end.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

/// Decode the percent escapes and the plus signs of a query value.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => out.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let pair = std::str::from_utf8(&bytes[index + 1..index + 3])
                    .ok()
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok());
                if let Some(byte) = pair {
                    out.push(byte);
                    index += 2;
                } else {
                    out.push(b'%');
                }
            }
            other => out.push(other),
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Read the request line and the headers. Give the target of a `GET`.
fn read_target(stream: &TcpStream) -> Option<(String, String)> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut words = line.split_whitespace();
    let method = words.next()?.to_owned();
    let target = words.next()?.to_owned();
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).ok()?;
        if header.trim().is_empty() {
            break;
        }
    }
    Some((method, target))
}

fn serve(shared: &Shared, mut stream: TcpStream) {
    let Some((method, target)) = read_target(&stream) else {
        return;
    };
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let (status, body) = if method != "GET" || path != "/pubnotify" {
        (404, "{}")
    } else {
        let mut request = HubRequest {
            url: None,
            podping: None,
        };
        for pair in query.split('&').filter(|pair| !pair.is_empty()) {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            match name {
                "url" => request.url = Some(decode(value)),
                "podping" => request.podping = Some(decode(value)),
                _ => {}
            }
        }
        let mut state = shared.state.lock().expect("the state");
        state.requests.push(request);
        if state.fail > 0 {
            state.fail -= 1;
            (500, "{}")
        } else if state.refuse > 0 {
            state.refuse -= 1;
            (200, REFUSED)
        } else {
            (200, ACCEPTED)
        }
    };
    let reply = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::Web;

    fn get(hub: &TestHub, query: &[(&str, &str)]) -> Result<String, crate::web::WebError> {
        let file = Web::new().get_with_query(&hub.url(), query, 4096)?;
        Ok(String::from_utf8(file.bytes).unwrap())
    }

    #[test]
    fn the_hub_accepts_a_request_and_records_the_decoded_values() {
        let hub = TestHub::start();
        let body = get(
            &hub,
            &[
                ("url", "https://m.example/a b.xml?x=1&y=2"),
                ("podping", "1"),
            ],
        )
        .unwrap();
        assert!(body.contains(r#""status":"true""#));
        get(&hub, &[("url", "https://m.example/b.xml")]).unwrap();
        assert_eq!(
            hub.requests(),
            [
                HubRequest {
                    url: Some("https://m.example/a b.xml?x=1&y=2".to_owned()),
                    podping: Some("1".to_owned()),
                },
                HubRequest {
                    url: Some("https://m.example/b.xml".to_owned()),
                    podping: None,
                },
            ]
        );
    }

    #[test]
    fn fail_next_gives_the_status_500_for_the_next_requests_only() {
        let hub = TestHub::start();
        hub.fail_next(2);
        for _ in 0..2 {
            let error = get(&hub, &[("url", "https://m.example/a.xml")]).unwrap_err();
            assert_eq!(error.status(), Some(500));
        }
        get(&hub, &[("url", "https://m.example/a.xml")]).unwrap();
        assert_eq!(hub.requests().len(), 3);
    }

    #[test]
    fn refuse_next_gives_a_refusal_with_the_status_200_for_the_next_requests_only() {
        let hub = TestHub::start();
        hub.refuse_next(1);
        let body = get(&hub, &[("url", "https://m.example/a.xml")]).unwrap();
        assert!(body.contains(r#""status":"false""#));
        assert!(body.contains("refused by the test hub"));
        let body = get(&hub, &[("url", "https://m.example/a.xml")]).unwrap();
        assert!(body.contains(r#""status":"true""#));
    }
}
