//! A small HTTP server on the loopback address for the tests. No request
//! leaves the computer.

#![allow(dead_code, missing_docs)]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use nostr::prelude::Event;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Req {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Req {
    /// The event of the `Authorization` header.
    pub fn event(&self) -> Option<Event> {
        let value = self.headers.get("authorization")?;
        let encoded = value.strip_prefix("Nostr ")?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }
}

pub struct Resp {
    pub status: u16,
    pub date: Option<i64>,
    pub body: String,
}

impl Resp {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            date: None,
            body: body.to_owned(),
        }
    }

    pub fn error(status: u16, code: &str) -> Self {
        Self::json(
            status,
            &format!(r#"{{"error":{{"code":"{code}","message":"x"}}}}"#),
        )
    }

    pub fn with_date(mut self, unix: i64) -> Self {
        self.date = Some(unix);
        self
    }
}

pub fn now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap()
}

pub struct Server {
    pub origin: String,
    requests: Arc<Mutex<Vec<Req>>>,
}

impl Server {
    /// Start a server. The handler gets the request and its index, and gives
    /// the answer.
    pub fn start(mut handler: impl FnMut(&Req, usize) -> Resp + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                let Some(req) = read_request(&mut stream) else {
                    continue;
                };
                let index = {
                    let mut seen = seen.lock().unwrap();
                    seen.push(req.clone());
                    seen.len() - 1
                };
                let resp = handler(&req, index);
                write_response(&mut stream, &resp);
            }
        });
        Self { origin, requests }
    }

    pub fn requests(&self) -> Vec<Req> {
        self.requests.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

fn read_request(stream: &mut TcpStream) -> Option<Req> {
    let mut data = Vec::new();
    let mut chunk = [0_u8; 4096];
    let head_end = loop {
        if let Some(at) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break at;
        }
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 {
            return None;
        }
        data.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&data[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?.to_owned();
    let path = first.next()?.to_owned();
    let headers: HashMap<String, String> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    let length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut body = data[head_end + 4..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Some(Req {
        method,
        path,
        headers,
        body,
    })
}

fn http_date(unix: i64) -> String {
    // Days since 1970-01-01 to a civil date.
    let days = unix.div_euclid(86_400);
    let secs = unix.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let weekday = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"]
        [usize::try_from(days.rem_euclid(7)).unwrap()];
    format!(
        "{weekday}, {day:02} {} {year} {:02}:{:02}:{:02} GMT",
        names[usize::try_from(month - 1).unwrap()],
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

fn write_response(stream: &mut TcpStream, resp: &Resp) {
    let date = resp
        .date
        .map(|d| format!("Date: {}\r\n", http_date(d)))
        .unwrap_or_default();
    let text = format!(
        "HTTP/1.1 {} X\r\n{date}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        resp.status,
        resp.body.len(),
        resp.body
    );
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

/// Check a request like Studio does (ADR 0110 item 13), against the server
/// time. Gives the reason when the request is not correct.
pub fn verify(req: &Req, origin: &str, server_time: i64) -> Result<Event, &'static str> {
    let event = req.event().ok_or("no event")?;
    if event.kind.as_u16() != 27_235 {
        return Err("kind");
    }
    if event.verify().is_err() {
        return Err("signature");
    }
    if i64::try_from(event.created_at.as_secs())
        .unwrap()
        .abs_diff(server_time)
        > 60
    {
        return Err("time");
    }
    let tag = |name: &str| {
        event.tags.iter().find_map(|t| match t.as_slice() {
            [n, v, ..] if n == name => Some(v.clone()),
            _ => None,
        })
    };
    if tag("u").as_deref() != Some(&format!("{origin}{}", req.path)) {
        return Err("u");
    }
    if tag("method").as_deref() != Some(req.method.as_str()) {
        return Err("method");
    }
    if !req.body.is_empty() {
        let hash = Sha256::digest(&req.body)
            .iter()
            .fold(String::new(), |mut text, b| {
                use std::fmt::Write as _;
                let _ = write!(text, "{b:02x}");
                text
            });
        if tag("payload").as_deref() != Some(hash.as_str()) {
            return Err("payload");
        }
    }
    Ok(event)
}
