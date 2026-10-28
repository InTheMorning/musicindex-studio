//! Media tickets and the audio route (ADR 0062 items 26 to 29).
//!
//! The audio player cannot send the access token. It asks for a ticket
//! with the usual access, and then reads `/media/audio?ticket=`. The route
//! reads the file from the release folder or from its host, sends the byte
//! range that the player asks for, and keeps no full file in memory.

use std::collections::HashMap;
use std::io::{Read, SeekFrom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::stream::unfold;
use serde::Deserialize;
use serde_json::json;
use studio_core::feed::is_web_address;
use studio_core::web::Web;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::access::random_hex;
use crate::error::ApiError;
use crate::routes::{ApiResult, Shared, blocking, pending_library_file};
use crate::workspace::lock_workspace;

/// How long a ticket works (ADR 0062 item 29).
const TICKET_LIFETIME: Duration = Duration::from_hours(1);

/// The size of one block that the route sends.
const BLOCK_BYTES: usize = 64 * 1024;

/// The media tickets of one workspace. A ticket holds one address and its
/// end time. Each user has one workspace, so a ticket belongs to one user
/// (ADR 0062 item 29). Studio keeps the tickets in memory only.
#[derive(Debug)]
pub(crate) struct Tickets {
    lifetime: Duration,
    tickets: HashMap<String, (String, Instant)>,
}

impl Default for Tickets {
    fn default() -> Self {
        Self::new(TICKET_LIFETIME)
    }
}

impl Tickets {
    /// Make an empty set of tickets that work for `lifetime`.
    pub(crate) fn new(lifetime: Duration) -> Self {
        Self {
            lifetime,
            tickets: HashMap::new(),
        }
    }

    /// Make a ticket for `url`. The call removes each ticket that
    /// expired.
    pub(crate) fn issue(&mut self, url: &str, now: Instant) -> Result<String, ApiError> {
        self.tickets.retain(|_, (_, expires)| now < *expires);
        let ticket = random_hex::<16>()?;
        self.tickets
            .insert(ticket.clone(), (url.to_owned(), now + self.lifetime));
        Ok(ticket)
    }

    /// The address of a ticket that exists and has not expired.
    pub(crate) fn address(&self, ticket: &str, now: Instant) -> Option<&str> {
        self.tickets
            .get(ticket)
            .filter(|(_, expires)| now < *expires)
            .map(|(url, _)| url.as_str())
    }
}

#[derive(Deserialize)]
pub(crate) struct TicketRequest {
    url: String,
}

/// Make a media ticket for one address. The address must be correct by
/// ADR 0023 item 2. The route uses the usual access of the API.
pub(crate) async fn make_ticket(
    State(shared): State<Shared>,
    Json(body): Json<TicketRequest>,
) -> ApiResult {
    if !is_web_address(&body.url) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_address",
            format!("{} is not a correct web address", body.url),
        ));
    }
    blocking(shared, move |w| {
        // A ticket holds the address with no OP3 prefix, so that the player
        // does not make a download for OP3 (ADR 0076 item 3).
        let plain = studio_core::media::plain_address(&body.url);
        let ticket = w.tickets.issue(&plain, Instant::now())?;
        Ok(Json(json!({ "ticket": ticket })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct AudioQuery {
    #[serde(default)]
    ticket: String,
}

/// Where the bytes of a ticket are.
enum Source {
    /// A file of the release folder.
    Folder { path: PathBuf, media_type: String },
    /// A file on its host. The client has the network policy of the
    /// workspace (ADR 0057).
    Host { url: String, web: Web },
}

/// A byte range of one `Range` header: the first and the last byte, or
/// the last `n` bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Range {
    From { first: u64, last: Option<u64> },
    Suffix(u64),
}

/// Read a `Range` header with one range, such as `bytes=0-99`, `bytes=100-`
/// or `bytes=-500`. Gives `None` for any other form.
fn parse_range(value: &str) -> Option<Range> {
    let spec = value.trim().strip_prefix("bytes=")?;
    let (first, last) = spec.split_once('-')?;
    let number = |text: &str| {
        (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
            .then(|| text.parse::<u64>().ok())
            .flatten()
    };
    match (number(first), number(last)) {
        (Some(first), last) if last.is_none_or(|last| last >= first) => {
            (last.is_some() || spec.ends_with('-')).then_some(Range::From { first, last })
        }
        (None, Some(count)) if first.is_empty() => Some(Range::Suffix(count)),
        _ => None,
    }
}

impl Range {
    /// The first and last byte in a file of `length` bytes. Gives `None`
    /// when the range does not fit the file.
    fn within(self, length: u64) -> Option<(u64, u64)> {
        if length == 0 {
            return None;
        }
        match self {
            Self::From { first, last } if first < length => {
                Some((first, last.map_or(length - 1, |l| l.min(length - 1))))
            }
            Self::Suffix(count) if count > 0 => Some((length.saturating_sub(count), length - 1)),
            _ => None,
        }
    }

    /// The header value for a host.
    fn header(self) -> String {
        match self {
            Self::From {
                first,
                last: Some(last),
            } => format!("bytes={first}-{last}"),
            Self::From { first, last: None } => format!("bytes={first}-"),
            Self::Suffix(count) => format!("bytes=-{count}"),
        }
    }
}

/// What the player asked for.
#[derive(Debug, Clone, Copy)]
enum Ask {
    /// The request has no `Range` header.
    Whole,
    /// One byte range.
    One(Range),
    /// A `Range` header that is not one range.
    Bad,
}

/// Send the audio of a ticket (ADR 0062 items 26, 27 and 29). The route
/// is outside the access layer: the ticket is the access.
pub(crate) async fn audio(
    State(shared): State<Shared>,
    Query(query): Query<AudioQuery>,
    request_headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    let range_text = request_headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());
    let ask = range_text.map_or(Ask::Whole, |text| {
        parse_range(text).map_or(Ask::Bad, Ask::One)
    });
    // The lock is held only to read the ticket and the library. The read of
    // the bytes occurs with no lock, so that a long play does not stop the
    // other requests.
    let source = {
        let w = lock_workspace(&shared)?;
        let url = w
            .tickets
            .address(&query.ticket, Instant::now())
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::FORBIDDEN,
                    "ticket_not_valid",
                    "the media ticket is unknown or expired: ask for a new ticket",
                )
            })?
            .to_owned();
        match pending_library_file(&w, &url, "audio/") {
            Some((path, media_type)) => Source::Folder { path, media_type },
            None => Source::Host {
                url,
                web: w.web.clone(),
            },
        }
    };
    match source {
        Source::Folder { path, media_type } => from_folder(&path, &media_type, ask).await,
        Source::Host { url, web } => from_host(url, web, ask).await,
    }
}

/// The response for a range that the file cannot give.
fn not_satisfiable(content_range: Option<String>) -> Response {
    let mut response = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
    if let Some(value) = content_range.and_then(|v| HeaderValue::from_str(&v).ok()) {
        response.headers_mut().insert(header::CONTENT_RANGE, value);
    }
    response
}

/// The block stream of a reader of the async runtime.
fn file_body(file: tokio::io::Take<tokio::fs::File>) -> Body {
    Body::from_stream(unfold(file, |mut file| async move {
        let mut block = vec![0_u8; BLOCK_BYTES];
        match file.read(&mut block).await {
            Ok(0) => None,
            Ok(count) => {
                block.truncate(count);
                Some((Ok(Bytes::from(block)), file))
            }
            Err(error) => Some((Err(error), file)),
        }
    }))
}

async fn from_folder(
    path: &std::path::Path,
    media_type: &str,
    ask: Ask,
) -> Result<Response, ApiError> {
    let io_error = |error: std::io::Error| ApiError::internal(format!("the audio file: {error}"));
    let mut file = tokio::fs::File::open(path).await.map_err(io_error)?;
    let length = file.metadata().await.map_err(io_error)?.len();
    let (status, first, last) = match ask {
        Ask::Whole => (StatusCode::OK, 0, length.saturating_sub(1)),
        Ask::One(range) => match range.within(length) {
            Some((first, last)) => (StatusCode::PARTIAL_CONTENT, first, last),
            None => return Ok(not_satisfiable(Some(format!("bytes */{length}")))),
        },
        Ask::Bad => return Ok(not_satisfiable(Some(format!("bytes */{length}")))),
    };
    let count = if length == 0 { 0 } else { last - first + 1 };
    file.seek(SeekFrom::Start(first)).await.map_err(io_error)?;
    let mut response = Response::new(file_body(file.take(count)));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    set_header(headers, header::CONTENT_TYPE, media_type);
    set_header(headers, header::CONTENT_LENGTH, &count.to_string());
    set_header(headers, header::ACCEPT_RANGES, "bytes");
    set_header(headers, header::CACHE_CONTROL, "no-store");
    if status == StatusCode::PARTIAL_CONTENT {
        set_header(
            headers,
            header::CONTENT_RANGE,
            &format!("bytes {first}-{last}/{length}"),
        );
    }
    Ok(response)
}

fn set_header(headers: &mut axum::http::HeaderMap, name: header::HeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        headers.insert(name, value);
    }
}

async fn from_host(url: String, web: Web, ask: Ask) -> Result<Response, ApiError> {
    let header_value = match ask {
        Ask::Whole => None,
        Ask::One(range) => Some(range.header()),
        // A header that is not one range cannot be satisfied.
        Ask::Bad => return Ok(not_satisfiable(None)),
    };
    let stream = tokio::task::spawn_blocking(move || web.open_range(&url, header_value.as_deref()))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))??;
    let status = StatusCode::from_u16(stream.status).unwrap_or(StatusCode::BAD_GATEWAY);
    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        return Ok(not_satisfiable(stream.content_range));
    }
    // The reader blocks, so a blocking task reads it and sends the blocks
    // through a small channel. The task ends when the player closes the
    // connection, because the send then fails.
    let (sender, receiver) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(4);
    let mut reader = stream.reader;
    let (media_type, content_length, content_range) = (
        stream.media_type,
        stream.content_length,
        stream.content_range,
    );
    tokio::task::spawn_blocking(move || {
        loop {
            let mut block = vec![0_u8; BLOCK_BYTES];
            let item = match reader.read(&mut block) {
                Ok(0) => return,
                Ok(count) => {
                    block.truncate(count);
                    Ok(Bytes::from(block))
                }
                Err(error) => Err(error),
            };
            let failed = item.is_err();
            if sender.blocking_send(item).is_err() || failed {
                return;
            }
        }
    });
    let body = Body::from_stream(unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|item| (item, receiver))
    }));
    let mut response = Response::new(body);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    if let Some(media_type) = &media_type {
        set_header(headers, header::CONTENT_TYPE, media_type);
    }
    if let Some(length) = content_length {
        set_header(headers, header::CONTENT_LENGTH, &length.to_string());
    }
    if let Some(content_range) = &content_range {
        set_header(headers, header::CONTENT_RANGE, content_range);
    }
    set_header(headers, header::ACCEPT_RANGES, "bytes");
    set_header(headers, header::CACHE_CONTROL, "no-store");
    Ok(response)
}
