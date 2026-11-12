//! Media tickets and the audio route (ADR 0062 items 26 to 29).
//!
//! The audio player cannot send the access token. It asks for a ticket
//! with the usual access, and then reads `/media/audio?ticket=`. The route
//! reads the file from the release folder, sends the byte range that the
//! player asks for, and keeps no full file in memory. A ticket holds the ID of
//! a library file, never an address, and the route never reads a host
//! (ADR 0109 items 5 and 6).

use std::collections::HashMap;
use std::io::{SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::stream::unfold;
use serde::Deserialize;
use serde_json::json;
use studio_core::collection::LibraryFileId;
use studio_core::inventory;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::access::random_hex;
use crate::error::ApiError;
use crate::routes::{ApiResult, Shared, blocking};
use crate::workspace::{Workspace, lock_workspace};

/// How long a ticket works (ADR 0062 item 29).
const TICKET_LIFETIME: Duration = Duration::from_hours(1);

/// The size of one block that the route sends.
const BLOCK_BYTES: usize = 64 * 1024;

/// What a ticket opens: a library file for the player or for the download
/// to a device, or the archive of the account. A ticket for a file holds the
/// ID of the file and no address (ADR 0109 item 5, ADR 0105 item 15).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    File(i64),
    /// The archive of the account (ADR 0107 item 7). The flag says if the
    /// archive holds the audio and the images. `folder` is the folder of a
    /// collection that is not open (ADR 0176 item 7), or `None` for the
    /// collection of the workspace.
    Archive {
        include_assets: bool,
        folder: Option<PathBuf>,
    },
}

/// The media tickets of one workspace. A ticket holds one target and its
/// end time. Each user has one workspace, so a ticket belongs to one user
/// (ADR 0062 item 29). Studio keeps the tickets in memory only.
#[derive(Debug)]
pub(crate) struct Tickets {
    lifetime: Duration,
    tickets: HashMap<String, (Target, Instant)>,
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

    /// Make a ticket for one library file (ADR 0109 item 5). The ticket
    /// holds the file ID.
    pub(crate) fn issue_file(&mut self, file: i64, now: Instant) -> Result<String, ApiError> {
        self.insert(Target::File(file), now)
    }

    /// Make a ticket for the archive of the account (ADR 0107 item 7), or
    /// of the collection in `folder` (ADR 0176 item 7).
    pub(crate) fn issue_archive(
        &mut self,
        include_assets: bool,
        folder: Option<PathBuf>,
        now: Instant,
    ) -> Result<String, ApiError> {
        self.insert(
            Target::Archive {
                include_assets,
                folder,
            },
            now,
        )
    }

    /// Use the archive ticket: the call removes it, so that it works one
    /// time. Gives `include_assets` and the folder of the ticket.
    pub(crate) fn take_archive(
        &mut self,
        ticket: &str,
        now: Instant,
    ) -> Option<(bool, Option<PathBuf>)> {
        match self.tickets.get(ticket) {
            Some((Target::Archive { .. }, expires)) if now < *expires => {
                match self.tickets.remove(ticket) {
                    Some((
                        Target::Archive {
                            include_assets,
                            folder,
                        },
                        _,
                    )) => Some((include_assets, folder)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn insert(&mut self, target: Target, now: Instant) -> Result<String, ApiError> {
        self.tickets.retain(|_, (_, expires)| now < *expires);
        let ticket = random_hex::<16>()?;
        self.tickets
            .insert(ticket.clone(), (target, now + self.lifetime));
        Ok(ticket)
    }

    /// The library file of a ticket that exists and has not expired.
    pub(crate) fn file(&self, ticket: &str, now: Instant) -> Option<i64> {
        match self.tickets.get(ticket) {
            Some((Target::File(id), expires)) if now < *expires => Some(*id),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TicketRequest {
    /// The ID of a library file of the user.
    #[serde(default)]
    file: Option<i64>,
    /// An address. A ticket never holds one, so the route refuses it
    /// (ADR 0109 item 5).
    #[serde(default)]
    url: Option<String>,
}

/// Make a media ticket for one library file of the user. The route uses the
/// usual access of the API. A request with an address is refused.
pub(crate) async fn make_ticket(
    State(shared): State<Shared>,
    Json(body): Json<TicketRequest>,
) -> ApiResult {
    match (body.file, body.url) {
        (Some(file), None) => make_file_ticket(shared, file).await,
        _ => Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_ticket_request",
            "give the ID of a library file: a ticket holds no address",
        )),
    }
}

/// Make a ticket for a library file (ADR 0109 item 5).
async fn make_file_ticket(shared: Shared, file: i64) -> ApiResult {
    blocking(shared, move |w| {
        let file = w.collection.library_file(LibraryFileId(file))?;
        let ticket = w.tickets.issue_file(file.id.0, Instant::now())?;
        Ok(Json(json!({ "ticket": ticket })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct AudioQuery {
    #[serde(default)]
    ticket: String,
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
    let (path, media_type) = {
        let w = lock_workspace(&shared)?;
        let id = w
            .tickets
            .file(&query.ticket, Instant::now())
            .ok_or_else(ticket_not_valid)?;
        let file = w.collection.library_file(LibraryFileId(id))?;
        // The route sends only a file that the server holds. A file that
        // only a host has loads from the host (ADR 0109 items 5 and 6).
        if !file.local || !file.media_type.starts_with("audio/") {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "file_not_held",
                format!("{} is not an audio file that this server holds", file.path),
            ));
        }
        (local_path(&w, &file)?, file.media_type)
    };
    from_folder(&path, &media_type, ask).await
}

fn ticket_not_valid() -> ApiError {
    ApiError::new(
        StatusCode::FORBIDDEN,
        "ticket_not_valid",
        "the media ticket is unknown or expired: ask for a new ticket",
    )
}

/// The path of a library file that the server holds, in its release folder.
fn local_path(
    w: &Workspace,
    file: &studio_core::collection::LibraryFile,
) -> Result<PathBuf, ApiError> {
    let folder = w.collection.feed(file.feed_id)?.folder.ok_or_else(|| {
        ApiError::internal(format!("feed {} has no release folder", file.feed_id.0))
    })?;
    Ok(w.library()?.path(&folder, &file.path))
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

/// Where the bytes of a file download are.
enum Download {
    /// A file of the release folder.
    Folder(PathBuf),
    /// The bytes of the copy on a host, read through its connector.
    Bytes(Vec<u8>),
}

/// The value of `Content-Disposition` for a file name: `attachment` with a
/// quoted ASCII name and the full name in UTF-8 (RFC 6266).
fn attachment(name: &str) -> String {
    use std::fmt::Write as _;
    let plain: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .map(|c| if matches!(c, '"' | '\\') { '_' } else { c })
        .collect();
    let mut encoded = String::new();
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    format!("attachment; filename=\"{plain}\"; filename*=UTF-8''{encoded}")
}

/// Send the bytes of one library file for a file ticket, with
/// `Content-Disposition: attachment` (ADR 0105 item 15). The route reads the
/// local bytes, or the copy on a host through its connector. It writes no
/// file. The route is outside the access layer: the ticket is the access,
/// and a ticket belongs to the workspace of one user.
pub(crate) async fn download(
    State(shared): State<Shared>,
    Query(query): Query<AudioQuery>,
) -> Result<Response, ApiError> {
    // The ticket of an archive is single use (ADR 0107 item 7).
    let job = {
        let (shared, ticket) = (Arc::clone(&shared), query.ticket.clone());
        tokio::task::spawn_blocking(move || {
            let mut w = lock_workspace(&shared)?;
            archive_job(&mut w, &ticket)
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))??
    };
    if let Some(job) = job {
        return Ok(archive_response(job));
    }
    // A connector cannot move between threads, so one blocking task reads
    // the host. It drops the workspace lock before the read.
    let (file, source) = tokio::task::spawn_blocking(move || {
        let w = lock_workspace(&shared)?;
        let id = w
            .tickets
            .file(&query.ticket, Instant::now())
            .ok_or_else(ticket_not_valid)?;
        let file = w.collection.library_file(LibraryFileId(id))?;
        if file.local {
            let path = local_path(&w, &file)?;
            return Ok::<_, ApiError>((file, Download::Folder(path)));
        }
        let missing = |file: &studio_core::collection::LibraryFile| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "file_missing",
                format!(
                    "{} is not on this server and no file host has a copy",
                    file.path
                ),
            )
        };
        let hosts = w.hosts()?;
        let copy = inventory::copies(&w.collection, &hosts, &file)?
            .into_iter()
            .next()
            .ok_or_else(|| missing(&file))?;
        drop(w);
        let read = hosts
            .get(copy.connector_id)?
            .read(&copy.path)?
            .ok_or_else(|| missing(&file))?;
        Ok((file, Download::Bytes(read.bytes)))
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))??;
    let mut response = match source {
        Download::Folder(path) => from_folder(&path, &file.media_type, Ask::Whole).await?,
        Download::Bytes(bytes) => {
            let length = bytes.len().to_string();
            let mut response = Response::new(Body::from(bytes));
            let headers = response.headers_mut();
            set_header(headers, header::CONTENT_TYPE, &file.media_type);
            set_header(headers, header::CONTENT_LENGTH, &length);
            set_header(headers, header::CACHE_CONTROL, "no-store");
            response
        }
    };
    set_header(
        response.headers_mut(),
        header::CONTENT_DISPOSITION,
        &attachment(&file.path),
    );
    Ok(response)
}

/// What the stream of an archive needs. The job holds no lock of the
/// workspace.
struct ArchiveJob {
    folder: PathBuf,
    library: Option<PathBuf>,
    manifest: studio_core::account_archive::Manifest,
    include_assets: bool,
}

/// Use the archive ticket, and collect what the stream needs. Gives `None`
/// when the ticket is not an archive ticket.
fn archive_job(w: &mut Workspace, ticket: &str) -> Result<Option<ArchiveJob>, ApiError> {
    let Some((include_assets, closed)) = w.tickets.take_archive(ticket, Instant::now()) else {
        return Ok(None);
    };
    if let Some(folder) = closed {
        return closed_archive_job(&folder, include_assets).map(Some);
    }
    // The manifest holds the email of the account and the public key of
    // each Nostr method. It holds no secret (ADR 0107 item 6).
    let mut keys: Vec<String> = w.identity.pubkey.iter().cloned().collect();
    for method in w.vault_methods()? {
        if let studio_core::vault::VaultMethod::Nostr { pubkey } = method
            && !keys.contains(&pubkey)
        {
            keys.push(pubkey);
        }
    }
    Ok(Some(ArchiveJob {
        folder: w.data_dir.clone(),
        library: w.collection.library_root()?,
        manifest: studio_core::account_archive::Manifest::new(w.identity.email.clone(), keys),
        include_assets,
    }))
}

/// The job for a collection that is not open. The manifest has no email,
/// because the identity of the workspace belongs to the open collection.
fn closed_archive_job(
    folder: &std::path::Path,
    include_assets: bool,
) -> Result<ArchiveJob, ApiError> {
    let methods = studio_core::vault::VaultFile::load(&folder.join("vault.json"))?.methods();
    let keys = methods
        .into_iter()
        .filter_map(|method| match method {
            studio_core::vault::VaultMethod::Nostr { pubkey } => Some(pubkey),
            _ => None,
        })
        .collect();
    let root = folder.parent().and_then(std::path::Path::parent);
    let library = match (root, folder.file_name().and_then(|n| n.to_str())) {
        (Some(root), Some(name)) => Some(
            studio_core::collections::Collections::new(root)
                .archive_library(name)
                .map_err(|e| ApiError::internal(e.to_string()))?,
        ),
        _ => None,
    };
    Ok(ArchiveJob {
        folder: folder.to_path_buf(),
        library,
        manifest: studio_core::account_archive::Manifest::new(None, keys),
        include_assets,
    })
}

/// A writer that sends its bytes to the body of the response in blocks.
/// The archive never lands in a file (ADR 0107 item 7).
struct ChannelWriter {
    sender: tokio::sync::mpsc::Sender<std::io::Result<Bytes>>,
    block: Vec<u8>,
}

impl ChannelWriter {
    fn send_block(&mut self) -> std::io::Result<()> {
        if self.block.is_empty() {
            return Ok(());
        }
        let block = Bytes::from(std::mem::replace(
            &mut self.block,
            Vec::with_capacity(BLOCK_BYTES),
        ));
        // The send fails when the browser closed the connection.
        self.sender
            .blocking_send(Ok(block))
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.block.extend_from_slice(data);
        if self.block.len() >= BLOCK_BYTES {
            self.send_block()?;
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.send_block()
    }
}

/// Send the archive as a stream with `Content-Disposition: attachment`.
fn archive_response(job: ArchiveJob) -> Response {
    let (sender, receiver) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(4);
    let name = crate::account::archive_name(job.manifest.exported_at);
    tokio::task::spawn_blocking(move || {
        let mut writer = ChannelWriter {
            sender: sender.clone(),
            block: Vec::with_capacity(BLOCK_BYTES),
        };
        let written = studio_core::account_archive::write_archive_from(
            &job.folder,
            job.library.as_deref(),
            &job.manifest,
            job.include_assets,
            &mut writer,
        )
        .map_err(|e| std::io::Error::other(e.to_string()))
        .and_then(|()| writer.flush());
        if let Err(error) = written {
            // The error ends the body, so that the browser does not keep a
            // short file as a good one.
            let _ = sender.blocking_send(Err(error));
        }
    });
    let body = Body::from_stream(unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|item| (item, receiver))
    }));
    let mut response = Response::new(body);
    let headers = response.headers_mut();
    set_header(headers, header::CONTENT_TYPE, "application/x-tar");
    set_header(headers, header::CONTENT_DISPOSITION, &attachment(&name));
    set_header(headers, header::CACHE_CONTROL, "no-store");
    response
}
