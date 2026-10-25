//! A small S3 server for tests. It runs in the test process.
//!
//! It has only the operations that the S3 connector uses, with the
//! conditions of AWS. It accepts path-style addresses only. It does not
//! check signatures.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// The shift of each MD5 step (RFC 1321, section 3.4).
const SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// The constants of RFC 1321, section 3.4: the integer part of
/// 2^32 * |sin(i + 1)|.
const CONSTANTS: [u32; 64] = [
    0xd76a_a478,
    0xe8c7_b756,
    0x2420_70db,
    0xc1bd_ceee,
    0xf57c_0faf,
    0x4787_c62a,
    0xa830_4613,
    0xfd46_9501,
    0x6980_98d8,
    0x8b44_f7af,
    0xffff_5bb1,
    0x895c_d7be,
    0x6b90_1122,
    0xfd98_7193,
    0xa679_438e,
    0x49b4_0821,
    0xf61e_2562,
    0xc040_b340,
    0x265e_5a51,
    0xe9b6_c7aa,
    0xd62f_105d,
    0x0244_1453,
    0xd8a1_e681,
    0xe7d3_fbc8,
    0x21e1_cde6,
    0xc337_07d6,
    0xf4d5_0d87,
    0x455a_14ed,
    0xa9e3_e905,
    0xfcef_a3f8,
    0x676f_02d9,
    0x8d2a_4c8a,
    0xfffa_3942,
    0x8771_f681,
    0x6d9d_6122,
    0xfde5_380c,
    0xa4be_ea44,
    0x4bde_cfa9,
    0xf6bb_4b60,
    0xbebf_bc70,
    0x289b_7ec6,
    0xeaa1_27fa,
    0xd4ef_3085,
    0x0488_1d05,
    0xd9d4_d039,
    0xe6db_99e5,
    0x1fa2_7cf8,
    0xc4ac_5665,
    0xf429_2244,
    0x432a_ff97,
    0xab94_23a7,
    0xfc93_a039,
    0x655b_59c3,
    0x8f0c_cc92,
    0xffef_f47d,
    0x8584_5dd1,
    0x6fa8_7e4f,
    0xfe2c_e6e0,
    0xa301_4314,
    0x4e08_11a1,
    0xf753_7e82,
    0xbd3a_f235,
    0x2ad7_d2bb,
    0xeb86_d391,
];

/// The MD5 digest of `data`, as in RFC 1321.
#[expect(clippy::many_single_char_names, reason = "the names follow RFC 1321")]
fn md5(data: &[u8]) -> [u8; 16] {
    let mut state: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64).wrapping_mul(8)).to_le_bytes());
    for block in message.as_chunks::<64>().0 {
        let words: Vec<u32> = block
            .as_chunks::<4>()
            .0
            .iter()
            .map(|w| u32::from_le_bytes(*w))
            .collect();
        let [mut a, mut b, mut c, mut d] = state;
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f
                .wrapping_add(a)
                .wrapping_add(CONSTANTS[i])
                .wrapping_add(words[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(SHIFTS[i]));
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
    let mut digest = [0_u8; 16];
    for (out, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *out = word.to_le_bytes();
    }
    digest
}

/// Lowercase hexadecimal text of `bytes`.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// The ETag of a single PUT: the MD5 of the bytes, in quotes.
fn single_etag(bytes: &[u8]) -> String {
    format!("\"{}\"", hex(&md5(bytes)))
}

/// An object in memory.
#[derive(Debug, Clone)]
struct Object {
    bytes: Vec<u8>,
    content_type: String,
    etag: String,
}

/// One part of an open multipart upload.
#[derive(Debug)]
struct Part {
    bytes: Vec<u8>,
    digest: [u8; 16],
}

/// An open multipart upload.
#[derive(Debug)]
struct Upload {
    key: String,
    content_type: String,
    parts: BTreeMap<u32, Part>,
}

/// What the server keeps and the switches of a test.
#[derive(Debug, Default)]
struct State {
    objects: BTreeMap<String, Object>,
    uploads: HashMap<String, Upload>,
    next_upload: u32,
    refuse_all: bool,
    ignore_delete_condition: bool,
    ignore_put_conditions: bool,
}

#[derive(Debug)]
struct Shared {
    bucket: String,
    state: Mutex<State>,
    stop: AtomicBool,
}

/// An S3 server on a loopback port for one bucket. It stops when it is
/// dropped.
#[derive(Debug)]
pub struct FakeS3 {
    port: u16,
    shared: Arc<Shared>,
}

impl FakeS3 {
    /// Start a server for `bucket`.
    ///
    /// # Panics
    ///
    /// Panics when the server cannot listen on a loopback port.
    #[must_use]
    pub fn start(bucket: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let shared = Arc::new(Shared {
            bucket: bucket.to_owned(),
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

    /// The address of the endpoint, for example `http://127.0.0.1:41234`.
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// When true, each request gets 403 `AccessDenied`.
    pub fn refuse_all(&self, refuse: bool) {
        self.with(|state| state.refuse_all = refuse);
    }

    /// When true, DELETE ignores `If-Match`, as R2 possibly does.
    pub fn ignore_delete_condition(&self, ignore: bool) {
        self.with(|state| state.ignore_delete_condition = ignore);
    }

    /// When true, PUT and `CompleteMultipartUpload` ignore `If-None-Match`
    /// and `If-Match`.
    pub fn ignore_put_conditions(&self, ignore: bool) {
        self.with(|state| state.ignore_put_conditions = ignore);
    }

    /// The bytes of the object at `key`, or `None`.
    #[must_use]
    pub fn object(&self, key: &str) -> Option<Vec<u8>> {
        self.with(|state| state.objects.get(key).map(|o| o.bytes.clone()))
    }

    /// The key of each object, in key sequence.
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.with(|state| state.objects.keys().cloned().collect())
    }

    /// Store an object with no request. The type is `binary/octet-stream`.
    pub fn put_object(&self, key: &str, bytes: &[u8]) {
        self.with(|state| {
            state.objects.insert(
                key.to_owned(),
                Object {
                    bytes: bytes.to_vec(),
                    content_type: DEFAULT_TYPE.to_owned(),
                    etag: single_etag(bytes),
                },
            );
        });
    }

    /// The number of multipart uploads that are not complete or aborted.
    #[must_use]
    pub fn open_uploads(&self) -> usize {
        self.with(|state| state.uploads.len())
    }

    fn with<T>(&self, action: impl FnOnce(&mut State) -> T) -> T {
        action(&mut self.shared.state.lock().expect("the state"))
    }
}

impl Drop for FakeS3 {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A connection wakes the accept loop so that it can end.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

const DEFAULT_TYPE: &str = "binary/octet-stream";

/// One request, with a decoded path.
struct Request {
    method: String,
    path: String,
    query: Vec<(String, String)>,
    /// Header names are in lowercase.
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

/// One response.
struct Reply {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn empty(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn xml(body: String) -> Self {
        Self {
            status: 200,
            headers: vec![("Content-Type", "application/xml".to_owned())],
            body: body.into_bytes(),
        }
    }

    fn error(status: u16, code: &str, message: &str) -> Self {
        let mut reply = Self::xml(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <Error><Code>{code}</Code><Message>{message}</Message></Error>"
        ));
        reply.status = status;
        reply
    }

    fn with_etag(mut self, etag: &str) -> Self {
        self.headers.push(("ETag", etag.to_owned()));
        self
    }
}

/// Decode the percent escapes of `text`.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let pair = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], pair) {
            (b'%', Some(value)) => {
                out.push(value);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Escape the characters of `text` that XML does not allow in text.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Read one request from the stream. Gives `None` for a broken request.
fn read_request(stream: &TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut words = line.split_whitespace();
    let method = words.next()?.to_owned();
    let target = words.next()?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let query = query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(key), percent_decode(value))
        })
        .collect();
    let mut headers = HashMap::new();
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
        path: percent_decode(path),
        query,
        headers,
        body,
    })
}

/// Answer the one request of a connection.
fn serve(shared: &Shared, mut stream: TcpStream) {
    let Some(request) = read_request(&stream) else {
        return;
    };
    let reply = {
        let mut state = shared.state.lock().expect("the state");
        route(&shared.bucket, &mut state, &request)
    };
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

/// Find the operation of a request and run it.
fn route(bucket: &str, state: &mut State, request: &Request) -> Reply {
    let signed =
        request.param("X-Amz-Signature").is_some() || request.header("authorization").is_some();
    if state.refuse_all || !signed {
        return Reply::error(403, "AccessDenied", "Access Denied");
    }
    let path = request.path.trim_start_matches('/');
    let (name, key) = path.split_once('/').unwrap_or((path, ""));
    if name != bucket {
        return Reply::error(404, "NoSuchBucket", "The specified bucket does not exist");
    }
    let upload = request.param("uploadId");
    match (request.method.as_str(), key.is_empty()) {
        ("GET", true) if request.param("list-type") == Some("2") => list(state, request),
        (_, true) => Reply::error(400, "InvalidRequest", "This request is not supported"),
        ("PUT", false) => match (request.param("partNumber"), upload) {
            (Some(number), Some(id)) => upload_part(state, request, id, number),
            _ => put(state, request, key),
        },
        ("GET" | "HEAD", false) => get(state, key),
        ("DELETE", false) => match upload {
            Some(id) => abort(state, id),
            None => delete(state, request, key),
        },
        ("POST", false) => match upload {
            Some(id) => complete(state, request, key, id),
            None if request.param("uploads").is_some() => create(state, request, key),
            None => Reply::error(400, "InvalidRequest", "This request is not supported"),
        },
        _ => Reply::error(405, "MethodNotAllowed", "The method is not allowed"),
    }
}

/// An ETag without its quotes.
fn bare(etag: &str) -> &str {
    etag.trim().trim_matches('"')
}

/// The error of the `If-None-Match` and `If-Match` conditions of a write,
/// or `None` when the write can go on.
fn write_condition(state: &State, request: &Request, key: &str) -> Option<Reply> {
    if state.ignore_put_conditions {
        return None;
    }
    let current = state.objects.get(key);
    if request.header("if-none-match") == Some("*") && current.is_some() {
        return Some(Reply::error(
            412,
            "PreconditionFailed",
            "At least one of the pre-conditions you specified did not hold",
        ));
    }
    let wanted = request.header("if-match")?;
    match current {
        None => Some(Reply::error(
            404,
            "NoSuchKey",
            "The specified key does not exist",
        )),
        Some(object) if bare(&object.etag) != bare(wanted) => Some(Reply::error(
            412,
            "PreconditionFailed",
            "At least one of the pre-conditions you specified did not hold",
        )),
        Some(_) => None,
    }
}

fn put(state: &mut State, request: &Request, key: &str) -> Reply {
    if let Some(error) = write_condition(state, request, key) {
        return error;
    }
    let etag = single_etag(&request.body);
    let content_type = request
        .header("content-type")
        .unwrap_or(DEFAULT_TYPE)
        .to_owned();
    state.objects.insert(
        key.to_owned(),
        Object {
            bytes: request.body.clone(),
            content_type,
            etag: etag.clone(),
        },
    );
    Reply::empty(200).with_etag(&etag)
}

fn get(state: &State, key: &str) -> Reply {
    let Some(object) = state.objects.get(key) else {
        return Reply::error(404, "NoSuchKey", "The specified key does not exist");
    };
    Reply {
        status: 200,
        headers: vec![("Content-Type", object.content_type.clone())],
        body: object.bytes.clone(),
    }
    .with_etag(&object.etag)
}

fn delete(state: &mut State, request: &Request, key: &str) -> Reply {
    let mismatch = request
        .header("if-match")
        .zip(state.objects.get(key))
        .is_some_and(|(wanted, object)| bare(&object.etag) != bare(wanted));
    if mismatch && !state.ignore_delete_condition {
        return Reply::error(
            412,
            "PreconditionFailed",
            "At least one of the pre-conditions you specified did not hold",
        );
    }
    state.objects.remove(key);
    Reply::empty(204)
}

fn list(state: &State, request: &Request) -> Reply {
    let prefix = request.param("prefix").unwrap_or("");
    let max: usize = request
        .param("max-keys")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let after = request.param("continuation-token");
    // The continuation token is the last key of the page before.
    let mut matches = state
        .objects
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .filter(|(key, _)| after.is_none_or(|token| key.as_str() > token));
    let page: Vec<_> = matches.by_ref().take(max).collect();
    let truncated = matches.next().is_some();
    let mut body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult>\
         <Name>{}</Name><Prefix>{}</Prefix><KeyCount>{}</KeyCount>\
         <MaxKeys>{max}</MaxKeys><IsTruncated>{truncated}</IsTruncated>",
        xml_escape(&request.path),
        xml_escape(prefix),
        page.len()
    );
    for (key, object) in &page {
        let _ = write!(
            body,
            "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag></Contents>",
            xml_escape(key),
            object.bytes.len(),
            xml_escape(&object.etag)
        );
    }
    if let (true, Some((last, _))) = (truncated, page.last()) {
        let _ = write!(
            body,
            "<NextContinuationToken>{}</NextContinuationToken>",
            xml_escape(last)
        );
    }
    body.push_str("</ListBucketResult>");
    Reply::xml(body)
}

fn create(state: &mut State, request: &Request, key: &str) -> Reply {
    state.next_upload += 1;
    let id = format!("upload-{}", state.next_upload);
    let content_type = request
        .header("content-type")
        .unwrap_or(DEFAULT_TYPE)
        .to_owned();
    state.uploads.insert(
        id.clone(),
        Upload {
            key: key.to_owned(),
            content_type,
            parts: BTreeMap::new(),
        },
    );
    Reply::xml(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><InitiateMultipartUploadResult>\
         <Key>{}</Key><UploadId>{id}</UploadId></InitiateMultipartUploadResult>",
        xml_escape(key)
    ))
}

fn no_upload() -> Reply {
    Reply::error(404, "NoSuchUpload", "The specified upload does not exist")
}

fn upload_part(state: &mut State, request: &Request, id: &str, number: &str) -> Reply {
    let Some(upload) = state.uploads.get_mut(id) else {
        return no_upload();
    };
    let Ok(number) = number.parse::<u32>() else {
        return Reply::error(400, "InvalidArgument", "The part number is not valid");
    };
    let digest = md5(&request.body);
    upload.parts.insert(
        number,
        Part {
            bytes: request.body.clone(),
            digest,
        },
    );
    Reply::empty(200).with_etag(&format!("\"{}\"", hex(&digest)))
}

/// The part numbers of a `CompleteMultipartUpload` body, in the sequence
/// of the body.
fn part_numbers(body: &[u8]) -> Vec<u32> {
    let text = String::from_utf8_lossy(body);
    text.split("<PartNumber>")
        .skip(1)
        .filter_map(|rest| rest.split("</PartNumber>").next()?.trim().parse().ok())
        .collect()
}

fn complete(state: &mut State, request: &Request, key: &str, id: &str) -> Reply {
    let Some(upload) = state.uploads.get(id) else {
        return no_upload();
    };
    if upload.key != key {
        return no_upload();
    }
    if let Some(error) = write_condition(state, request, key) {
        return error;
    }
    let numbers = part_numbers(&request.body);
    if numbers.is_empty() {
        return Reply::error(
            400,
            "MalformedXML",
            "The XML you provided was not well-formed",
        );
    }
    let mut bytes = Vec::new();
    let mut digests = Vec::new();
    for number in &numbers {
        let Some(part) = upload.parts.get(number) else {
            return Reply::error(400, "InvalidPart", "One or more parts could not be found");
        };
        bytes.extend_from_slice(&part.bytes);
        digests.extend_from_slice(&part.digest);
    }
    let etag = format!("\"{}-{}\"", hex(&md5(&digests)), numbers.len());
    let content_type = upload.content_type.clone();
    state.uploads.remove(id);
    state.objects.insert(
        key.to_owned(),
        Object {
            bytes,
            content_type,
            etag: etag.clone(),
        },
    );
    Reply::xml(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><CompleteMultipartUploadResult>\
         <Key>{}</Key><ETag>{}</ETag></CompleteMultipartUploadResult>",
        xml_escape(key),
        xml_escape(&etag)
    ))
    .with_etag(&etag)
}

fn abort(state: &mut State, id: &str) -> Reply {
    if state.uploads.remove(id).is_some() {
        Reply::empty(204)
    } else {
        no_upload()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTH: (&str, &str) = ("Authorization", "AWS4-HMAC-SHA256 test");

    /// What a test sees of a response.
    struct Seen {
        status: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    }

    impl Seen {
        fn header(&self, name: &str) -> &str {
            self.headers.get(name).map_or("", String::as_str)
        }

        fn text(&self) -> String {
            String::from_utf8_lossy(&self.body).into_owned()
        }
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

    fn signed(method: &str, url: &str, extra: &[(&str, &str)], body: &[u8]) -> Seen {
        let mut headers = vec![AUTH];
        headers.extend_from_slice(extra);
        call(method, url, &headers, body)
    }

    fn address(server: &FakeS3, rest: &str) -> String {
        format!("{}/{rest}", server.url())
    }

    #[test]
    fn md5_matches_the_vectors_of_rfc_1321() {
        let cases = [
            ("", "d41d8cd98f00b204e9800998ecf8427e"),
            ("a", "0cc175b9c0f1b6a831c399e269772661"),
            ("abc", "900150983cd24fb0d6963f7d28e17f72"),
            ("message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
            (
                "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                "57edf4a22be3c955ac49da2e2107b67a",
            ),
        ];
        for (text, digest) in cases {
            assert_eq!(hex(&md5(text.as_bytes())), digest, "md5 of {text:?}");
        }
    }

    #[test]
    fn put_get_head_and_delete_work() {
        let server = FakeS3::start("bk");
        let url = address(&server, "bk/a/b.xml");
        let put = signed("PUT", &url, &[("Content-Type", "text/xml")], b"abc");
        assert_eq!(put.status, 200);
        assert_eq!(put.header("etag"), "\"900150983cd24fb0d6963f7d28e17f72\"");
        let get = signed("GET", &url, &[], b"");
        assert_eq!(get.status, 200);
        assert_eq!(get.body, b"abc");
        assert_eq!(get.header("etag"), put.header("etag"));
        assert_eq!(get.header("content-type"), "text/xml");
        assert_eq!(get.header("content-length"), "3");
        let head = signed("HEAD", &url, &[], b"");
        assert_eq!(head.status, 200);
        assert_eq!(head.body.len(), 0);
        assert_eq!(head.header("etag"), put.header("etag"));
        assert_eq!(head.header("content-length"), "3");
        assert_eq!(signed("DELETE", &url, &[], b"").status, 204);
        assert_eq!(server.object("a/b.xml"), None);
        let missing = signed("GET", &url, &[], b"");
        assert_eq!(missing.status, 404);
        assert!(missing.text().contains("<Code>NoSuchKey</Code>"));
        let head = signed("HEAD", &url, &[], b"");
        assert_eq!(head.status, 404);
        assert_eq!(head.body.len(), 0);
        assert_eq!(signed("DELETE", &url, &[], b"").status, 204);
    }

    #[test]
    fn put_conditions_give_412_and_404() {
        let server = FakeS3::start("bk");
        let url = address(&server, "bk/k");
        let first = signed("PUT", &url, &[("If-None-Match", "*")], b"one");
        assert_eq!(first.status, 200);
        let again = signed("PUT", &url, &[("If-None-Match", "*")], b"two");
        assert_eq!(again.status, 412);
        assert!(again.text().contains("<Code>PreconditionFailed</Code>"));
        assert_eq!(server.object("k").as_deref(), Some(&b"one"[..]));
        let wrong = signed("PUT", &url, &[("If-Match", "\"0000\"")], b"two");
        assert_eq!(wrong.status, 412);
        let etag = first.header("etag").to_owned();
        let right = signed("PUT", &url, &[("If-Match", &etag)], b"two");
        assert_eq!(right.status, 200);
        assert_eq!(server.object("k").as_deref(), Some(&b"two"[..]));
        let none = signed(
            "PUT",
            &address(&server, "bk/new"),
            &[("If-Match", &etag)],
            b"x",
        );
        assert_eq!(none.status, 404);
        assert!(none.text().contains("<Code>NoSuchKey</Code>"));
    }

    #[test]
    fn delete_condition_gives_412_unless_ignored() {
        let server = FakeS3::start("bk");
        server.put_object("k", b"abc");
        let url = address(&server, "bk/k");
        let wrong = signed("DELETE", &url, &[("If-Match", "\"0000\"")], b"");
        assert_eq!(wrong.status, 412);
        assert!(server.object("k").is_some());
        let right = signed(
            "DELETE",
            &url,
            &[("If-Match", "\"900150983cd24fb0d6963f7d28e17f72\"")],
            b"",
        );
        assert_eq!(right.status, 204);
        assert!(server.object("k").is_none());
        server.put_object("k", b"abc");
        server.ignore_delete_condition(true);
        let ignored = signed("DELETE", &url, &[("If-Match", "\"0000\"")], b"");
        assert_eq!(ignored.status, 204);
        assert!(server.object("k").is_none());
    }

    #[test]
    fn ignore_put_conditions_lets_a_write_pass() {
        let server = FakeS3::start("bk");
        server.put_object("k", b"one");
        server.ignore_put_conditions(true);
        let url = address(&server, "bk/k");
        let put = signed("PUT", &url, &[("If-None-Match", "*")], b"two");
        assert_eq!(put.status, 200);
        let put = signed("PUT", &url, &[("If-Match", "\"0000\"")], b"three");
        assert_eq!(put.status, 200);
        assert_eq!(server.object("k").as_deref(), Some(&b"three"[..]));
    }

    fn keys(page: &Seen) -> Vec<String> {
        page.text()
            .split("<Key>")
            .skip(1)
            .filter_map(|r| r.split("</Key>").next().map(str::to_owned))
            .collect()
    }

    #[test]
    fn list_gives_pages_and_obeys_the_prefix() {
        let server = FakeS3::start("bk");
        server.put_object("feeds/b.xml", b"bb");
        server.put_object("feeds/a.xml", b"a");
        server.put_object("other/c.xml", b"ccc");
        let base = address(&server, "bk?list-type=2&max-keys=2");
        let first = signed("GET", &base, &[], b"");
        assert_eq!(first.status, 200);
        assert_eq!(keys(&first), ["feeds/a.xml", "feeds/b.xml"]);
        assert!(first.text().contains("<IsTruncated>true</IsTruncated>"));
        assert!(first.text().contains("<Size>1</Size>"));
        assert!(first.text().contains("<ETag>&quot;"));
        assert!(
            first
                .text()
                .contains("<NextContinuationToken>feeds/b.xml</NextContinuationToken>")
        );
        let second = signed(
            "GET",
            &format!("{base}&continuation-token=feeds%2Fb.xml"),
            &[],
            b"",
        );
        assert_eq!(keys(&second), ["other/c.xml"]);
        assert!(second.text().contains("<IsTruncated>false</IsTruncated>"));
        assert!(!second.text().contains("NextContinuationToken"));
        let prefixed = signed(
            "GET",
            &address(&server, "bk?list-type=2&prefix=feeds%2F"),
            &[],
            b"",
        );
        assert_eq!(keys(&prefixed), ["feeds/a.xml", "feeds/b.xml"]);
        assert!(prefixed.text().contains("<IsTruncated>false</IsTruncated>"));
    }

    #[test]
    fn a_request_needs_a_signature_and_the_bucket() {
        let server = FakeS3::start("bk");
        server.put_object("k", b"abc");
        let url = address(&server, "bk/k");
        let none = call("GET", &url, &[], b"");
        assert_eq!(none.status, 403);
        assert!(none.text().contains("<Code>AccessDenied</Code>"));
        let query = call("GET", &format!("{url}?X-Amz-Signature=abc"), &[], b"");
        assert_eq!(query.status, 200);
        let wrong = signed("GET", &address(&server, "other/k"), &[], b"");
        assert_eq!(wrong.status, 404);
        assert!(wrong.text().contains("<Code>NoSuchBucket</Code>"));
        server.refuse_all(true);
        assert_eq!(signed("GET", &url, &[], b"").status, 403);
        server.refuse_all(false);
        assert_eq!(signed("GET", &url, &[], b"").status, 200);
    }

    fn start_upload(server: &FakeS3, key: &str) -> String {
        let created = signed(
            "POST",
            &address(server, &format!("bk/{key}?uploads")),
            &[],
            b"",
        );
        assert_eq!(created.status, 200);
        created
            .text()
            .split("<UploadId>")
            .nth(1)
            .and_then(|r| r.split("</UploadId>").next())
            .expect("an upload ID")
            .to_owned()
    }

    fn complete_body(numbers: &[u32]) -> Vec<u8> {
        let mut body = String::from("<CompleteMultipartUpload>");
        for number in numbers {
            let _ = write!(
                body,
                "<Part><PartNumber>{number}</PartNumber><ETag>x</ETag></Part>"
            );
        }
        body.push_str("</CompleteMultipartUpload>");
        body.into_bytes()
    }

    #[test]
    fn multipart_upload_joins_the_parts() {
        let server = FakeS3::start("bk");
        let id = start_upload(&server, "big");
        assert_eq!(server.open_uploads(), 1);
        let part = |n: u32, bytes: &[u8]| {
            signed(
                "PUT",
                &address(&server, &format!("bk/big?partNumber={n}&uploadId={id}")),
                &[],
                bytes,
            )
        };
        let two = part(2, b"world");
        let one = part(1, b"hello ");
        assert_eq!(one.status, 200);
        assert_eq!(one.header("etag"), single_etag(b"hello "));
        let url = address(&server, &format!("bk/big?uploadId={id}"));
        let done = signed("POST", &url, &[], &complete_body(&[1, 2]));
        assert_eq!(done.status, 200);
        let mut digests = md5(b"hello ").to_vec();
        digests.extend_from_slice(&md5(b"world"));
        let expected = format!("\"{}-2\"", hex(&md5(&digests)));
        assert_eq!(done.header("etag"), expected);
        assert!(done.text().contains(&xml_escape(&expected)));
        assert_eq!(two.status, 200);
        assert_eq!(server.object("big").as_deref(), Some(&b"hello world"[..]));
        let get = signed("GET", &address(&server, "bk/big"), &[], b"");
        assert_eq!(get.body, b"hello world");
        assert_eq!(get.header("etag"), expected);
        assert_eq!(server.open_uploads(), 0);
    }

    #[test]
    fn multipart_complete_obeys_the_conditions_and_the_sequence() {
        let server = FakeS3::start("bk");
        server.put_object("big", b"old");
        let id = start_upload(&server, "big");
        for (n, bytes) in [(1, &b"a"[..]), (2, &b"b"[..])] {
            let url = address(&server, &format!("bk/big?partNumber={n}&uploadId={id}"));
            assert_eq!(signed("PUT", &url, &[], bytes).status, 200);
        }
        let url = address(&server, &format!("bk/big?uploadId={id}"));
        let body = complete_body(&[2, 1]);
        let exists = signed("POST", &url, &[("If-None-Match", "*")], &body);
        assert_eq!(exists.status, 412);
        let wrong = signed("POST", &url, &[("If-Match", "\"0000\"")], &body);
        assert_eq!(wrong.status, 412);
        assert_eq!(server.open_uploads(), 1);
        let etag = single_etag(b"old");
        let right = signed("POST", &url, &[("If-Match", &etag)], &body);
        assert_eq!(right.status, 200);
        assert_eq!(server.object("big").as_deref(), Some(&b"ba"[..]));
    }

    #[test]
    fn multipart_complete_can_ignore_the_conditions() {
        let server = FakeS3::start("bk");
        server.put_object("big", b"old");
        server.ignore_put_conditions(true);
        let id = start_upload(&server, "big");
        let part = address(&server, &format!("bk/big?partNumber=1&uploadId={id}"));
        assert_eq!(signed("PUT", &part, &[], b"new").status, 200);
        let url = address(&server, &format!("bk/big?uploadId={id}"));
        let done = signed(
            "POST",
            &url,
            &[("If-None-Match", "*")],
            &complete_body(&[1]),
        );
        assert_eq!(done.status, 200);
        assert_eq!(server.object("big").as_deref(), Some(&b"new"[..]));
    }

    #[test]
    fn multipart_abort_removes_the_parts() {
        let server = FakeS3::start("bk");
        let id = start_upload(&server, "big");
        let part = address(&server, &format!("bk/big?partNumber=1&uploadId={id}"));
        assert_eq!(signed("PUT", &part, &[], b"a").status, 200);
        let url = address(&server, &format!("bk/big?uploadId={id}"));
        assert_eq!(signed("DELETE", &url, &[], b"").status, 204);
        assert_eq!(server.open_uploads(), 0);
        let again = signed("POST", &url, &[], &complete_body(&[1]));
        assert_eq!(again.status, 404);
        assert!(again.text().contains("<Code>NoSuchUpload</Code>"));
        assert_eq!(server.object("big"), None);
    }
}
