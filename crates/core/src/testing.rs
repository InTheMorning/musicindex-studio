//! A small HTTP server for tests. It needs no network.
//!
//! The `testing` feature makes it available to the tests of other crates.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

/// One response of the test server: a path, a status and a body.
pub type Route = (&'static str, u16, Vec<u8>);

type Routes = Arc<Mutex<HashMap<String, (u16, Vec<u8>)>>>;

/// An HTTP server on a loopback port. It answers each `GET` with the route
/// for the path, without the query, or with status 404.
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
        let routes: HashMap<String, (u16, Vec<u8>)> = routes
            .into_iter()
            .map(|(path, status, body)| (path.to_owned(), (status, body)))
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
                while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                    header.clear();
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/");
                let path = path.split('?').next().unwrap_or(path).to_owned();
                let (status, body) = shared
                    .lock()
                    .ok()
                    .and_then(|r| r.get(&path).cloned())
                    .unwrap_or((404, b"not found".to_vec()));
                let head = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
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
        self.routes
            .lock()
            .expect("the route table")
            .insert(path.to_owned(), (status, body));
    }
}
