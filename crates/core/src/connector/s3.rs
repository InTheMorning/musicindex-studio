//! The S3 connector for S3 and R2 buckets (ADR 0046).

use std::fmt;
use std::io::Read;
use std::time::Duration;

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use rusty_s3::actions::{CompleteMultipartUpload, CreateMultipartUpload, ListObjectsV2};
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use serde::{Deserialize, Serialize};

use crate::net::{self, NetPolicy, refused_host};

use super::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, LIST_MAX_DEPTH,
    LIST_MAX_FILES, ListedFile, Stamp, Version, length_error, relative_path,
};

/// How long a presigned address is valid.
const SIGNATURE_LIFETIME: Duration = Duration::from_secs(3600);
/// The largest file that one `PutObject` carries (ADR 0046 item 10).
const SINGLE_MAX: u64 = 64 * 1024 * 1024;
/// The size of each part of a multipart upload (ADR 0046 item 10).
const PART_SIZE: u64 = 16 * 1024 * 1024;
/// The most parts that S3 accepts in one multipart upload.
const MAX_PARTS: u64 = 10_000;
/// The most bytes that `read` accepts from one object.
const READ_LIMIT: u64 = 1024 * 1024 * 1024;
/// The most bytes that Studio reads from a list page or an error body.
const XML_LIMIT: u64 = 16 * 1024 * 1024;

/// The address style of a bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum S3UrlStyle {
    /// The bucket name is the first segment of the path.
    Path,
    /// The bucket name is a part of the host name.
    Virtual,
}

/// The configuration of an S3 host. It holds no secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct S3Config {
    /// The address of the endpoint, for example `https://s3.example.com`.
    pub endpoint: String,
    /// The region name that the signature uses.
    pub region: String,
    /// The name of the bucket.
    pub bucket: String,
    /// The address style of the bucket.
    pub url_style: S3UrlStyle,
    /// The key prefix. It is empty or it ends with `/`.
    #[serde(default)]
    pub prefix: String,
    /// The public address of the prefix, when the bucket has one.
    #[serde(default)]
    pub base_url: Option<String>,
}

/// The access key of an S3 host.
#[derive(Clone, Deserialize)]
pub struct S3Credentials {
    access_key_id: String,
    secret_access_key: String,
}

impl S3Credentials {
    /// Make credentials from the access key ID and the secret access key.
    #[must_use]
    pub fn new(access_key_id: impl Into<String>, secret_access_key: impl Into<String>) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
        }
    }
}

impl fmt::Debug for S3Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Credentials").finish_non_exhaustive()
    }
}

/// One bucket of an S3 host, below a key prefix.
///
/// The version marker is the `ETag`. Each write has a condition on the
/// host. A delete has a short window between its check and its request on
/// a host that ignores `If-Match` (ADR 0046 item 12).
pub struct S3Bucket {
    config: S3Config,
    bucket: Bucket,
    credentials: Credentials,
    agent: ureq::Agent,
    single_max: u64,
    part_size: u64,
    page_size: Option<usize>,
}

impl fmt::Debug for S3Bucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The credentials are not in the text.
        f.debug_struct("S3Bucket")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl S3Bucket {
    /// The connector kind in the collection.
    pub const KIND: &'static str = "s3";

    /// Make a connector for a bucket.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::InvalidConfig`] when the endpoint is not
    /// an `http` or `https` address, the bucket or the region is empty, or
    /// the prefix is not empty and does not end with `/`.
    pub fn new(config: S3Config, credentials: S3Credentials) -> Result<Self, ConnectorError> {
        let invalid =
            |message: &str| ConnectorError::new(ConnectorErrorKind::InvalidConfig, message);
        if config.bucket.is_empty() || config.region.is_empty() {
            return Err(invalid("the bucket and the region must not be empty"));
        }
        if !config.prefix.is_empty()
            && (!config.prefix.ends_with('/') || config.prefix.starts_with('/'))
        {
            return Err(invalid(
                "the prefix must be empty, or end with / and not start with /",
            ));
        }
        let style = match config.url_style {
            S3UrlStyle::Path => UrlStyle::Path,
            S3UrlStyle::Virtual => UrlStyle::VirtualHost,
        };
        let endpoint = config
            .endpoint
            .parse()
            .map_err(|_| invalid("the endpoint is not a valid address"))?;
        let bucket = Bucket::new(
            endpoint,
            style,
            config.bucket.clone(),
            config.region.clone(),
        )
        .map_err(|e| invalid(&format!("the endpoint is not usable: {e}")))?;
        Ok(Self {
            config,
            bucket,
            credentials: Credentials::new(credentials.access_key_id, credentials.secret_access_key),
            agent: agent(&NetPolicy::Open),
            single_max: SINGLE_MAX,
            part_size: PART_SIZE,
            page_size: None,
        })
    }

    /// The same connector with the network policy `policy` (ADR 0057).
    #[must_use]
    pub fn with_policy(self, policy: &NetPolicy) -> Self {
        Self {
            agent: agent(policy),
            ..self
        }
    }

    /// The same connector with other limits for a single `PutObject` and for
    /// the parts of a multipart upload. A test uses this.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_limits(self, single_max: u64, part_size: u64) -> Self {
        Self {
            single_max,
            part_size,
            ..self
        }
    }

    /// The same connector with a maximum number of keys in each list page.
    /// A test uses this.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_page_size(self, page_size: usize) -> Self {
        Self {
            page_size: Some(page_size),
            ..self
        }
    }

    /// The configuration text for the collection.
    ///
    /// # Panics
    ///
    /// Never, because the configuration is made of text.
    #[must_use]
    pub fn config(&self) -> String {
        serde_json::to_string(&self.config).expect("the configuration is JSON")
    }

    /// Make the connector from the configuration text of a row and the
    /// secret JSON of the vault.
    pub(crate) fn from_config(config: &str, secret: &[u8]) -> Result<Self, ConnectorError> {
        let config: S3Config = serde_json::from_str(config)
            .map_err(|e| ConnectorError::new(ConnectorErrorKind::InvalidConfig, e.to_string()))?;
        // The message holds no part of the secret.
        let credentials: S3Credentials = serde_json::from_slice(secret).map_err(|_| {
            ConnectorError::new(
                ConnectorErrorKind::InvalidConfig,
                "the secret of the connector must be JSON with access_key_id and secret_access_key",
            )
        })?;
        Self::new(config, credentials)
    }

    /// Test that the provider refuses a conditional write that is not
    /// correct (ADR 0046 items 4 and 5). The test uses a key below the
    /// prefix whose name starts with `.studio-probe-`. It deletes the key at
    /// the end, also after a failed step, when it can.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::NoConditions`] when the second or the
    /// third write does not give 412. Gives [`ConnectorErrorKind::Unauthorized`]
    /// when the host refuses the credentials, and
    /// [`ConnectorErrorKind::Unavailable`] when Studio cannot connect.
    pub fn test_provider(&self) -> Result<(), ConnectorError> {
        let path = format!(".studio-probe-{}", uuid::Uuid::new_v4());
        let result = self.probe(&path);
        // The delete has its own condition. A failed delete does not hide
        // the result of the steps.
        let cleanup = self
            .stat(&path)
            .and_then(|found| found.map_or(Ok(()), |file| self.delete(&path, &file.stamp)));
        result?;
        cleanup
    }

    /// The three writes of the provider test.
    fn probe(&self, path: &str) -> Result<(), ConnectorError> {
        let no_conditions = || {
            ConnectorError::new(
                ConnectorErrorKind::NoConditions,
                "the provider does not refuse a conditional write, so Studio cannot write to it safely",
            )
        };
        self.put(path, b"studio probe", &Expected::Absent)?;
        // A second write with `If-None-Match` must give 412.
        match self.put(path, b"studio probe", &Expected::Absent) {
            Err(e) if e.kind == ConnectorErrorKind::Conflict => {}
            Ok(_) => return Err(no_conditions()),
            Err(e) => return Err(e),
        }
        // A write with an `If-Match` that is not correct must give 412.
        let wrong = Expected::Version(Version::new("\"00000000000000000000000000000000\""));
        match self.put(path, b"studio probe", &wrong) {
            Err(e) if e.kind == ConnectorErrorKind::Conflict => Ok(()),
            Ok(_) => Err(no_conditions()),
            Err(e) => Err(e),
        }
    }

    /// The key of a path: the prefix and the path.
    fn key_of(&self, path: &str) -> Result<String, ConnectorError> {
        relative_path(path)?;
        if path.split('/').any(|s| s.is_empty() || s == ".") {
            return Err(ConnectorError::new(
                ConnectorErrorKind::InvalidPath,
                format!("the path {path:?} is not a relative path in the connector"),
            ));
        }
        Ok(format!("{}{path}", self.config.prefix))
    }

    /// Sign an action with extra headers. The headers are in the signature,
    /// so the request must send the same headers.
    fn sign<'a, A: S3Action<'a>>(mut action: A, headers: &[(&'a str, &'a str)]) -> String {
        for (name, value) in headers {
            action.headers_mut().insert(*name, *value);
        }
        action.sign(SIGNATURE_LIFETIME).to_string()
    }

    /// Send one request. The headers are the same as in the signature.
    fn send<B: ureq::AsSendBody>(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, &str)],
        body: B,
    ) -> Result<ureq::http::Response<ureq::Body>, ConnectorError> {
        let unavailable = |e: &dyn fmt::Display| {
            ConnectorError::new(
                ConnectorErrorKind::Unavailable,
                format!(
                    "cannot reach the S3 host: {}",
                    without_addresses(&e.to_string())
                ),
            )
        };
        let mut builder = ureq::http::Request::builder().method(method).uri(url);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let request = builder.body(body).map_err(|e| unavailable(&e))?;
        self.agent.run(request).map_err(|e| match refused_host(&e) {
            Some(host) => ConnectorError::new(
                ConnectorErrorKind::NotPublic,
                crate::net::not_public_message(host),
            ),
            None => unavailable(&e),
        })
    }

    /// The error for a response that is not a success.
    fn failure(what: &str, mut response: ureq::http::Response<ureq::Body>) -> ConnectorError {
        let status = response.status().as_u16();
        let code = response
            .body_mut()
            .with_config()
            .limit(XML_LIMIT)
            .read_to_string()
            .ok()
            .and_then(|text| error_code(&text));
        let kind = match status {
            412 => ConnectorErrorKind::Conflict,
            401 | 403 => ConnectorErrorKind::Unauthorized,
            _ => ConnectorErrorKind::Io,
        };
        let detail = code.map_or_else(String::new, |c| format!(" {c}"));
        ConnectorError::new(kind, format!("{what}: the S3 host gave {status}{detail}"))
    }

    /// The `Content-Type` of a new object, from the extension of its path.
    fn content_type_of(path: &str) -> String {
        if path.to_ascii_lowercase().ends_with(".xml") {
            return "application/rss+xml".to_owned();
        }
        crate::remote::media_type_of_name(path)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    /// The condition header of a write.
    fn condition(expected: &Expected) -> (&'static str, &str) {
        match expected {
            Expected::Absent => ("if-none-match", "*"),
            Expected::Version(version) => ("if-match", version.as_str()),
        }
    }

    /// The error for a write whose condition failed or whose key is gone.
    fn write_failure(
        path: &str,
        expected: &Expected,
        response: ureq::http::Response<ureq::Body>,
    ) -> ConnectorError {
        // An `If-Match` write on a key that is gone gives 404. The file
        // thus changed on the host.
        if response.status().as_u16() == 404 && matches!(expected, Expected::Version(_)) {
            return ConnectorError::new(
                ConnectorErrorKind::Conflict,
                format!("{path} changed on the host"),
            );
        }
        let mut error = Self::failure(path, response);
        if error.kind == ConnectorErrorKind::Conflict {
            error.message = format!("{path} changed on the host");
        }
        error
    }

    /// One `PutObject`.
    fn put(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let key = self.key_of(path)?;
        let content_type = Self::content_type_of(path);
        let (name, value) = Self::condition(expected);
        let headers = [("content-type", content_type.as_str()), (name, value)];
        let url = Self::sign(
            self.bucket.put_object(Some(&self.credentials), &key),
            &headers,
        );
        let response = self.send("PUT", &url, &headers, bytes)?;
        if !response.status().is_success() {
            return Err(Self::write_failure(path, expected, response));
        }
        etag_of(&response)
            .map(Version::new)
            .ok_or_else(|| missing_etag(path))
    }

    /// A multipart upload of `length` bytes of `reader`.
    fn put_parts(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let key = self.key_of(path)?;
        let part_size = self.part_size.max(1);
        if length.div_ceil(part_size) > MAX_PARTS {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Io,
                format!("{path} needs more than {MAX_PARTS} parts"),
            ));
        }
        let upload_id = self.create_upload(path, &key)?;
        let result = self.upload_and_complete(path, &key, &upload_id, reader, length, expected);
        if result.is_err() {
            // The abort is a best effort. The first error is the result.
            let headers: [(&str, &str); 0] = [];
            let url = Self::sign(
                self.bucket
                    .abort_multipart_upload(Some(&self.credentials), &key, &upload_id),
                &headers,
            );
            let _ = self.send("DELETE", &url, &headers, ());
        }
        result
    }

    fn create_upload(&self, path: &str, key: &str) -> Result<String, ConnectorError> {
        let content_type = Self::content_type_of(path);
        let headers = [("content-type", content_type.as_str())];
        let url = Self::sign(
            CreateMultipartUpload::new(&self.bucket, Some(&self.credentials), key),
            &headers,
        );
        let mut response = self.send("POST", &url, &headers, ())?;
        if !response.status().is_success() {
            return Err(Self::failure(path, response));
        }
        let text = read_text(&mut response)?;
        let mut id = None;
        walk(&text, |stack, text| {
            if ends_with(stack, &["InitiateMultipartUploadResult", "UploadId"]) {
                id = Some(text.to_owned());
            }
        })?;
        id.filter(|id| !id.is_empty()).ok_or_else(|| {
            ConnectorError::new(
                ConnectorErrorKind::Io,
                format!("{path}: the S3 host gave no upload ID"),
            )
        })
    }

    fn upload_and_complete(
        &self,
        path: &str,
        key: &str,
        upload_id: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let mut etags = Vec::new();
        let mut left = length;
        while left > 0 {
            let size = left.min(self.part_size.max(1));
            let part = read_exact_len(reader, size, path, length)?;
            let number = u16::try_from(etags.len() + 1).expect("at most 10000 parts");
            let headers: [(&str, &str); 0] = [];
            let url = Self::sign(
                self.bucket
                    .upload_part(Some(&self.credentials), key, number, upload_id),
                &headers,
            );
            let response = self.send("PUT", &url, &headers, part.as_slice())?;
            if !response.status().is_success() {
                return Err(Self::failure(path, response));
            }
            etags.push(etag_of(&response).ok_or_else(|| missing_etag(path))?);
            left -= size;
        }
        // The reader must have no more bytes.
        let mut extra = [0_u8; 1];
        let more = reader.read(&mut extra).map_err(|e| read_error(path, &e))?;
        if more != 0 {
            return Err(length_error(path, length));
        }
        let complete = CompleteMultipartUpload::new(
            &self.bucket,
            Some(&self.credentials),
            key,
            upload_id,
            etags.iter().map(String::as_str),
        );
        let body = complete.clone().body();
        let (name, value) = Self::condition(expected);
        let headers = [(name, value)];
        let url = Self::sign(complete, &headers);
        let mut response = self.send("POST", &url, &headers, body.as_bytes())?;
        if !response.status().is_success() {
            return Err(Self::write_failure(path, expected, response));
        }
        // S3 can give 200 with an error in the body.
        let header_etag = etag_of(&response);
        let text = read_text(&mut response)?;
        let mut code = None;
        let mut etag = None;
        walk(&text, |stack, text| {
            if ends_with(stack, &["Error", "Code"]) {
                code = Some(text.to_owned());
            } else if ends_with(stack, &["CompleteMultipartUploadResult", "ETag"]) {
                etag = Some(text.to_owned());
            }
        })?;
        if let Some(code) = code {
            let kind = if code == "PreconditionFailed" {
                ConnectorErrorKind::Conflict
            } else {
                ConnectorErrorKind::Io
            };
            return Err(ConnectorError::new(
                kind,
                format!("{path}: the S3 host gave {code}"),
            ));
        }
        etag.or(header_etag)
            .map(Version::new)
            .ok_or_else(|| missing_etag(path))
    }
}

impl Connector for S3Bucket {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        let key = self.key_of(path)?;
        let url = Self::sign(
            self.bucket.get_object(Some(&self.credentials), &key),
            &NO_COMPRESSION,
        );
        let mut response = self.send("GET", &url, &NO_COMPRESSION, ())?;
        let status = response.status().as_u16();
        if status == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Self::failure(path, response));
        }
        let version = etag_of(&response)
            .map(Version::new)
            .ok_or_else(|| missing_etag(path))?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(READ_LIMIT)
            .read_to_vec()
            .map_err(|e| read_error(path, &e))?;
        Ok(Some(FileRead { bytes, version }))
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.put(path, bytes, expected)
    }

    fn write_from(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        // The path check comes before any read of the reader.
        self.key_of(path)?;
        if length <= self.single_max {
            let mut bytes = Vec::new();
            reader
                .take(length.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(|e| read_error(path, &e))?;
            if bytes.len() as u64 != length {
                return Err(length_error(path, length));
            }
            return self.put(path, &bytes, expected);
        }
        self.put_parts(path, reader, length, expected)
    }

    fn public_url(&self, path: &str) -> Option<String> {
        let base = self.config.base_url.as_deref()?;
        Some(format!(
            "{}/{}",
            base.trim_end_matches('/'),
            crate::web::url_path(path)
        ))
    }

    fn list(&self) -> Result<FileList, ConnectorError> {
        let prefix = self.config.prefix.as_str();
        let mut list = FileList::default();
        let mut token: Option<String> = None;
        loop {
            let mut action = ListObjectsV2::new(&self.bucket, Some(&self.credentials));
            if !prefix.is_empty() {
                action.with_prefix(prefix);
            }
            if let Some(token) = token.as_deref() {
                action.with_continuation_token(token);
            }
            if let Some(size) = self.page_size {
                action.with_max_keys(size);
            }
            let headers: [(&str, &str); 0] = [];
            let url = Self::sign(action, &headers);
            let mut response = self.send("GET", &url, &headers, ())?;
            if !response.status().is_success() {
                return Err(Self::failure("the list", response));
            }
            let page = parse_list(&read_text(&mut response)?)?;
            for entry in page.entries {
                let key = if page.url_encoded {
                    percent_decode(&entry.key)
                } else {
                    entry.key
                };
                let Some(path) = key.strip_prefix(prefix) else {
                    continue;
                };
                if path.is_empty() || path.ends_with('/') {
                    continue;
                }
                let segments: Vec<&str> = path.split('/').collect();
                if segments.iter().any(|s| s.is_empty() || s.starts_with('.')) {
                    continue;
                }
                if segments.len() > LIST_MAX_DEPTH {
                    list.truncated = true;
                    continue;
                }
                if list.files.len() == LIST_MAX_FILES {
                    list.truncated = true;
                    list.files.sort_by(|a, b| a.path.cmp(&b.path));
                    return Ok(list);
                }
                list.files.push(ListedFile {
                    path: path.to_owned(),
                    size: entry.size,
                    stamp: stamp_of(&entry.etag, entry.size),
                });
            }
            match page.next_token {
                Some(next) if page.truncated => token = Some(next),
                _ => break,
            }
        }
        list.files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(list)
    }

    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        let key = self.key_of(path)?;
        let url = Self::sign(
            self.bucket.head_object(Some(&self.credentials), &key),
            &NO_COMPRESSION,
        );
        let response = self.send("HEAD", &url, &NO_COMPRESSION, ())?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Self::failure(path, response));
        }
        let etag = etag_of(&response).ok_or_else(|| missing_etag(path))?;
        let size = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| {
                ConnectorError::new(
                    ConnectorErrorKind::Io,
                    format!("{path}: the S3 host gave no size"),
                )
            })?;
        Ok(Some(ListedFile {
            path: path.to_owned(),
            size,
            stamp: stamp_of(&etag, size),
        }))
    }

    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        let key = self.key_of(path)?;
        let Some(current) = self.stat(path)? else {
            return Err(ConnectorError::new(
                ConnectorErrorKind::NotFound,
                format!("{path} does not exist"),
            ));
        };
        let changed = || {
            ConnectorError::new(
                ConnectorErrorKind::Conflict,
                format!("{path} changed on the host"),
            )
        };
        if &current.stamp != expected {
            return Err(changed());
        }
        let etag = current
            .stamp
            .as_str()
            .rsplit_once('|')
            .map_or(current.stamp.as_str(), |(etag, _)| etag);
        let headers = [("if-match", etag)];
        let url = Self::sign(
            self.bucket.delete_object(Some(&self.credentials), &key),
            &headers,
        );
        let response = self.send("DELETE", &url, &headers, ())?;
        match response.status().as_u16() {
            404 => Err(ConnectorError::new(
                ConnectorErrorKind::NotFound,
                format!("{path} does not exist"),
            )),
            _ if response.status().is_success() => Ok(()),
            _ => Err(Self::failure(path, response)),
        }
    }
}

/// The stamp of an object: its `ETag` and its size.
/// The HTTP client of a connector. It follows no redirect.
fn agent(policy: &NetPolicy) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .proxy(net::proxy(policy))
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(120)))
        .http_status_as_error(false)
        .max_redirects(0)
        .user_agent(concat!("musicindex-studio/", env!("CARGO_PKG_VERSION")))
        .build();
    net::agent(config, policy)
}

fn stamp_of(etag: &str, size: u64) -> Stamp {
    Stamp::new(format!("{etag}|{size}"))
}

/// The headers of a read and a stat. With no `Accept-Encoding`, the HTTP
/// client asks for gzip. R2 then compresses a feed and gives a weak ETag,
/// and a weak ETag does not agree with `If-Match`. The version must be the
/// strong ETag of the stored object.
const NO_COMPRESSION: [(&str, &str); 1] = [("accept-encoding", "identity")];

/// The `ETag` header of a response.
fn etag_of<B>(response: &ureq::http::Response<B>) -> Option<String> {
    response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

fn missing_etag(path: &str) -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Io,
        format!("{path}: the S3 host gave no ETag"),
    )
}

fn read_error(path: &str, error: &dyn fmt::Display) -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Io,
        format!("cannot read the data for {path}: {error}"),
    )
}

/// The text body of a response.
fn read_text(response: &mut ureq::http::Response<ureq::Body>) -> Result<String, ConnectorError> {
    response
        .body_mut()
        .with_config()
        .limit(XML_LIMIT)
        .read_to_string()
        .map_err(|e| read_error("the S3 response", &e))
}

/// Read exactly `size` bytes from `reader`. A reader that ends sooner gives
/// an error for the whole `length`.
fn read_exact_len(
    reader: &mut dyn Read,
    size: u64,
    path: &str,
    length: u64,
) -> Result<Vec<u8>, ConnectorError> {
    let mut bytes = Vec::new();
    reader
        .take(size)
        .read_to_end(&mut bytes)
        .map_err(|e| read_error(path, &e))?;
    if bytes.len() as u64 != size {
        return Err(length_error(path, length));
    }
    Ok(bytes)
}

/// True when the element path `stack` ends with `tail`.
fn ends_with(stack: &[String], tail: &[&str]) -> bool {
    stack.len() >= tail.len()
        && stack[stack.len() - tail.len()..]
            .iter()
            .zip(tail)
            .all(|(a, b)| a == b)
}

/// Walk the elements of an XML text. At the end of each element, `leaf`
/// gets the path of the element names and the text of the element.
fn walk(xml: &str, mut leaf: impl FnMut(&[String], &str)) -> Result<(), ConnectorError> {
    let bad = |e: &dyn fmt::Display| {
        ConnectorError::new(
            ConnectorErrorKind::Io,
            format!("the S3 host gave XML that Studio cannot read: {e}"),
        )
    };
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut text = String::new();
    loop {
        match reader.read_event().map_err(|e| bad(&e))? {
            Event::Start(tag) => {
                stack.push(tag.local_name().as_ref().to_owned());
                text.clear();
            }
            Event::Empty(tag) => {
                stack.push(tag.local_name().as_ref().to_owned());
                leaf(&stack, "");
                stack.pop();
                text.clear();
            }
            Event::Text(t) => text.push_str(&t.xml_content(XmlVersion::Implicit1_0)),
            Event::CData(data) => text.push_str(&data),
            Event::GeneralRef(reference) => {
                if let Some(c) = reference.resolve_char_ref().map_err(|e| bad(&e))? {
                    text.push(c);
                } else if let Some(value) = resolve_predefined_entity(&reference) {
                    text.push_str(value);
                }
            }
            Event::End(_) => {
                leaf(&stack, &text);
                stack.pop();
                text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

/// The `Code` of an S3 error body, when it has one.
fn error_code(xml: &str) -> Option<String> {
    let mut code = None;
    walk(xml, |stack, text| {
        if ends_with(stack, &["Error", "Code"]) {
            code = Some(text.to_owned());
        }
    })
    .ok()?;
    code
}

/// One key of a list page.
#[derive(Default)]
struct Entry {
    key: String,
    size: u64,
    etag: String,
}

/// One page of a list answer.
#[derive(Default)]
struct Page {
    entries: Vec<Entry>,
    truncated: bool,
    next_token: Option<String>,
    url_encoded: bool,
}

fn parse_list(xml: &str) -> Result<Page, ConnectorError> {
    let mut page = Page::default();
    let mut entry = Entry::default();
    let mut bad_size = false;
    walk(xml, |stack, text| {
        let Some(name) = stack.last() else { return };
        if stack.len() == 3 && stack[1] == "Contents" {
            match name.as_str() {
                "Key" => text.clone_into(&mut entry.key),
                "ETag" => text.clone_into(&mut entry.etag),
                "Size" => match text.trim().parse() {
                    Ok(size) => entry.size = size,
                    Err(_) => bad_size = true,
                },
                _ => {}
            }
        } else if stack.len() == 2 && name == "Contents" {
            page.entries.push(std::mem::take(&mut entry));
        } else if stack.len() == 2 {
            match name.as_str() {
                "IsTruncated" => page.truncated = text.trim() == "true",
                "NextContinuationToken" => page.next_token = Some(text.to_owned()),
                "EncodingType" => page.url_encoded = text.trim() == "url",
                _ => {}
            }
        }
    })?;
    if bad_size {
        return Err(ConnectorError::new(
            ConnectorErrorKind::Io,
            "the S3 host gave a list with a size that is not a number",
        ));
    }
    Ok(page)
}

/// Decode the percent escapes of a key that the host encoded.
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

/// The text of an error with each address removed. A presigned address
/// holds the access key ID, so no message may contain it.
fn without_addresses(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            if word.contains("://") || word.contains("X-Amz-") {
                "<address>"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::super::suite;
    use super::*;
    use crate::testing::fake_s3::FakeS3;

    fn config(server: &FakeS3, prefix: &str) -> S3Config {
        S3Config {
            endpoint: server.url(),
            region: "us-east-1".to_owned(),
            bucket: "bk".to_owned(),
            url_style: S3UrlStyle::Path,
            prefix: prefix.to_owned(),
            base_url: Some("https://cdn.example.com/feeds/".to_owned()),
        }
    }

    fn credentials() -> S3Credentials {
        S3Credentials::new("AKIDEXAMPLE", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY")
    }

    fn connector(server: &FakeS3, prefix: &str) -> S3Bucket {
        S3Bucket::new(config(server, prefix), credentials()).unwrap()
    }

    /// The `Content-Type` that a GET of `path` gives.
    fn content_type_of(bucket: &S3Bucket, path: &str) -> String {
        let key = bucket.key_of(path).unwrap();
        let headers: [(&str, &str); 0] = [];
        let url = S3Bucket::sign(
            bucket.bucket.get_object(Some(&bucket.credentials), &key),
            &headers,
        );
        let response = bucket.send("GET", &url, &headers, ()).unwrap();
        response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn an_error_text_loses_each_address() {
        let text = "bad uri http://h/b/k?X-Amz-Credential=AKIDEXAMPLE%2F2026 here";
        assert_eq!(without_addresses(text), "bad uri <address> here");
    }

    #[test]
    fn the_config_text_and_the_row_make_the_connector() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "feeds/");
        let secret = br#"{"access_key_id":"A","secret_access_key":"B"}"#;
        let again = S3Bucket::from_config(&bucket.config(), secret).unwrap();
        assert_eq!(again.config, bucket.config);
        let old = r#"{"endpoint":"https://s3.example.com","region":"r","bucket":"bk","url_style":"virtual"}"#;
        let made = S3Bucket::from_config(old, secret).unwrap();
        assert_eq!(made.config.prefix, "");
        assert_eq!(made.config.url_style, S3UrlStyle::Virtual);
        assert_eq!(made.config.base_url, None);
        for bad in [&b"{}"[..], b"not json", br#"{"access_key_id":"A"}"#] {
            let error = S3Bucket::from_config(old, bad).unwrap_err();
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        }
        let mut wrong = config(&server, "feeds");
        let error = S3Bucket::new(wrong.clone(), credentials()).unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        wrong.prefix = "/feeds/".to_owned();
        assert!(S3Bucket::new(wrong.clone(), credentials()).is_err());
        wrong.prefix = String::new();
        wrong.endpoint = "ftp://example.com".to_owned();
        assert!(S3Bucket::new(wrong, credentials()).is_err());
    }

    #[test]
    fn from_row_makes_an_s3_connector_and_needs_the_secret() {
        use crate::collection::{ConnectorId, ConnectorRow};
        use crate::vault::OpenVault;
        let server = FakeS3::start("bk");
        let mut vault = OpenVault::create("correct horse battery staple 2026").unwrap();
        vault
            .put(
                "host-1",
                br#"{"access_key_id":"A","secret_access_key":"B"}"#,
            )
            .unwrap();
        let row = |secret_ref: Option<&str>| ConnectorRow {
            id: ConnectorId(1),
            kind: S3Bucket::KIND.to_owned(),
            name: "Bucket".to_owned(),
            config: connector(&server, "").config(),
            secret_ref: secret_ref.map(str::to_owned),
        };
        let made = super::super::from_row(&row(Some("host-1")), Some(&vault)).unwrap();
        made.write("a.txt", b"x", &Expected::Absent).unwrap();
        assert_eq!(server.object("a.txt").as_deref(), Some(&b"x"[..]));
        let error = super::super::from_row(&row(None), Some(&vault))
            .err()
            .unwrap();
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
    }

    #[test]
    fn the_debug_text_does_not_show_the_secret() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "");
        let text = format!("{bucket:?} {:?}", credentials());
        assert!(!text.contains("wJalrXUtnFEMI"), "{text}");
        assert!(text.contains("S3Bucket"));
    }

    #[test]
    fn a_read_gives_the_strong_etag_so_a_write_with_it_passes() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "");
        let v1 = bucket.write("f.xml", b"one", &Expected::Absent).unwrap();
        let read = bucket.read("f.xml").unwrap().unwrap();
        assert_eq!(read.version, v1);
        let stat = bucket.stat("f.xml").unwrap().unwrap();
        assert_eq!(stat.stamp.as_str(), format!("{v1}|3"));
        bucket
            .write("f.xml", b"two", &Expected::Version(read.version))
            .unwrap();
    }

    #[test]
    fn read_write_stat_and_delete_use_the_prefix_and_the_etag() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "feeds/");
        assert_eq!(bucket.read("a/f.xml").unwrap(), None);
        assert_eq!(bucket.stat("a/f.xml").unwrap(), None);
        let v1 = bucket.write("a/f.xml", b"one", &Expected::Absent).unwrap();
        assert_eq!(server.object("feeds/a/f.xml").as_deref(), Some(&b"one"[..]));
        let read = bucket.read("a/f.xml").unwrap().unwrap();
        assert_eq!(read.bytes, b"one");
        assert_eq!(read.version, v1);
        assert!(v1.as_str().starts_with('"'));

        let stat = bucket.stat("a/f.xml").unwrap().unwrap();
        assert_eq!(stat.size, 3);
        assert_eq!(stat.stamp.as_str(), format!("{v1}|3"));

        let error = bucket
            .write("a/f.xml", b"two", &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        let error = bucket
            .write(
                "a/f.xml",
                b"two",
                &Expected::Version(Version::new("\"0000\"")),
            )
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        let error = bucket
            .write(
                "none.xml",
                b"two",
                &Expected::Version(Version::new("\"0000\"")),
            )
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        let v2 = bucket
            .write("a/f.xml", b"two", &Expected::Version(v1))
            .unwrap();
        assert_ne!(v2, read.version);

        let old = Stamp::new("\"0000\"|3");
        let error = bucket.delete("a/f.xml", &old).unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        let stamp = bucket.stat("a/f.xml").unwrap().unwrap().stamp;
        bucket.delete("a/f.xml", &stamp).unwrap();
        assert_eq!(server.object("feeds/a/f.xml"), None);
        let error = bucket.delete("a/f.xml", &stamp).unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::NotFound);
    }

    #[test]
    fn a_delete_sends_if_match_so_a_changed_file_stays() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "");
        bucket.write("a.mp3", b"one", &Expected::Absent).unwrap();
        let stamp = bucket.stat("a.mp3").unwrap().unwrap().stamp;
        // The file changes after the stat and before the delete request.
        server.put_object("a.mp3", b"other");
        let etag = stamp.as_str().rsplit_once('|').unwrap().0.to_owned();
        let headers = [("if-match", etag.as_str())];
        let url = S3Bucket::sign(
            bucket
                .bucket
                .delete_object(Some(&bucket.credentials), "a.mp3"),
            &headers,
        );
        let response = bucket.send("DELETE", &url, &headers, ()).unwrap();
        assert_eq!(response.status().as_u16(), 412);
        assert!(server.object("a.mp3").is_some());
    }

    #[test]
    fn a_path_is_a_key_below_the_prefix() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "p/");
        for bad in ["", "/a", "../a", "a/../b", "a//b", "a/./b"] {
            let error = bucket.read(bad).unwrap_err();
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidPath, "{bad}");
        }
        bucket
            .write("a b/c d.xml", b"x", &Expected::Absent)
            .unwrap();
        assert_eq!(server.object("p/a b/c d.xml").as_deref(), Some(&b"x"[..]));
        assert!(bucket.read("a b/c d.xml").unwrap().is_some());
    }

    #[test]
    fn the_list_removes_the_prefix_and_leaves_out_hidden_and_deep_keys() {
        let server = FakeS3::start("bk");
        for key in [
            "feeds/b.xml",
            "feeds/album/feed.xml",
            "feeds/.hidden/a.xml",
            "feeds/a/.b/c.xml",
            "feeds/a/b/c/d/deep.xml",
            "feeds/a/b/c/ok.xml",
            "other/x.xml",
        ] {
            server.put_object(key, b"x");
        }
        let list = connector(&server, "feeds/").list().unwrap();
        let paths: Vec<&str> = list.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["a/b/c/ok.xml", "album/feed.xml", "b.xml"]);
        assert!(list.files.iter().all(|f| f.size == 1));
        assert!(list.truncated);
        // The list stamp is the stamp of a stat.
        let stat = connector(&server, "feeds/").stat("b.xml").unwrap().unwrap();
        assert_eq!(list.files[2].stamp, stat.stamp);
        // A bucket with no prefix lists every key.
        let all = connector(&server, "").list().unwrap();
        // The keys below `feeds/` are one segment deeper without the prefix.
        assert_eq!(all.files.len(), 3);
    }

    #[test]
    fn the_list_follows_the_pages() {
        let server = FakeS3::start("bk");
        for n in 0..5 {
            server.put_object(&format!("p/{n}.xml"), b"x");
        }
        server.put_object("p/.hidden/a.xml", b"x");
        let list = connector(&server, "p/").with_page_size(2).list().unwrap();
        let paths: Vec<&str> = list.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["0.xml", "1.xml", "2.xml", "3.xml", "4.xml"]);
        assert!(!list.truncated);
    }

    #[test]
    fn the_list_stops_at_the_file_limit() {
        let server = FakeS3::start("bk");
        for n in 0..=LIST_MAX_FILES {
            server.put_object(&format!("{n:05}.xml"), b"x");
        }
        let list = connector(&server, "").with_page_size(400).list().unwrap();
        assert_eq!(list.files.len(), LIST_MAX_FILES);
        assert!(list.truncated);
    }

    #[test]
    fn a_small_file_is_one_put_and_a_large_file_is_a_multipart_upload() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "").with_limits(10, 4);
        let single = bucket
            .write_from("one.bin", &mut &b"0123456789"[..], 10, &Expected::Absent)
            .unwrap();
        assert!(!single.as_str().contains('-'), "{single}");
        assert_eq!(
            server.object("one.bin").as_deref(),
            Some(&b"0123456789"[..])
        );

        let multi = bucket
            .write_from("two.bin", &mut &b"0123456789A"[..], 11, &Expected::Absent)
            .unwrap();
        assert!(multi.as_str().ends_with("-3\""), "{multi}");
        assert_eq!(
            server.object("two.bin").as_deref(),
            Some(&b"0123456789A"[..])
        );
        assert_eq!(server.open_uploads(), 0);
        assert_eq!(bucket.read("two.bin").unwrap().unwrap().version, multi);
    }

    #[test]
    fn a_multipart_upload_with_a_condition_that_fails_is_a_conflict_and_aborts() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "").with_limits(10, 4);
        server.put_object("big.bin", b"old");
        let error = bucket
            .write_from("big.bin", &mut &b"0123456789A"[..], 11, &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        assert_eq!(server.open_uploads(), 0);
        assert_eq!(server.object("big.bin").as_deref(), Some(&b"old"[..]));

        let stale = Expected::Version(Version::new("\"0000\""));
        let error = bucket
            .write_from("big.bin", &mut &b"0123456789A"[..], 11, &stale)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        assert_eq!(server.open_uploads(), 0);

        let current = bucket.read("big.bin").unwrap().unwrap().version;
        bucket
            .write_from(
                "big.bin",
                &mut &b"0123456789A"[..],
                11,
                &Expected::Version(current),
            )
            .unwrap();
        assert_eq!(
            server.object("big.bin").as_deref(),
            Some(&b"0123456789A"[..])
        );
    }

    #[test]
    fn a_short_or_long_reader_gives_io_and_leaves_no_object_and_no_upload() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "").with_limits(10, 4);
        for (data, length) in [
            (&b"abc"[..], 5_u64),
            (&b"0123456789A"[..], 20),
            (&b"0123456789ABCDE"[..], 11),
            (&b"abcdef"[..], 3),
        ] {
            let error = bucket
                .write_from("x.bin", &mut { data }, length, &Expected::Absent)
                .unwrap_err();
            assert_eq!(error.kind(), ConnectorErrorKind::Io, "{length}");
            assert_eq!(server.object("x.bin"), None);
            assert_eq!(server.open_uploads(), 0);
        }
    }

    #[test]
    fn a_feed_gets_the_rss_type_and_an_mp3_gets_the_audio_type() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "");
        bucket.write("f.xml", b"<rss/>", &Expected::Absent).unwrap();
        bucket.write("S.MP3", b"x", &Expected::Absent).unwrap();
        bucket.write("a.flac", b"x", &Expected::Absent).unwrap();
        bucket.write("a.bin", b"x", &Expected::Absent).unwrap();
        assert_eq!(content_type_of(&bucket, "f.xml"), "application/rss+xml");
        assert_eq!(content_type_of(&bucket, "S.MP3"), "audio/mpeg");
        assert_eq!(content_type_of(&bucket, "a.flac"), "audio/flac");
        assert_eq!(
            content_type_of(&bucket, "a.bin"),
            "application/octet-stream"
        );
        // A multipart upload sets the type too.
        let small = bucket.with_limits(1, 2);
        small
            .write_from("m.mp3", &mut &b"abcde"[..], 5, &Expected::Absent)
            .unwrap();
        assert_eq!(content_type_of(&small, "m.mp3"), "audio/mpeg");
    }

    #[test]
    fn the_public_address_encodes_each_segment() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "feeds/");
        assert_eq!(
            bucket.public_url("a/01 Song (Live).mp3").as_deref(),
            Some("https://cdn.example.com/feeds/a/01%20Song%20%28Live%29.mp3")
        );
        let mut plain = config(&server, "");
        plain.base_url = None;
        let bucket = S3Bucket::new(plain, credentials()).unwrap();
        assert_eq!(bucket.public_url("a.xml"), None);
    }

    #[test]
    fn a_host_that_refuses_gives_unauthorized() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "");
        bucket.write("a.xml", b"x", &Expected::Absent).unwrap();
        server.refuse_all(true);
        let kind = |e: ConnectorError| e.kind();
        assert_eq!(
            bucket.read("a.xml").map_err(kind),
            Err(ConnectorErrorKind::Unauthorized)
        );
        assert_eq!(
            bucket.stat("a.xml").map_err(kind),
            Err(ConnectorErrorKind::Unauthorized)
        );
        assert_eq!(
            bucket.list().map_err(kind),
            Err(ConnectorErrorKind::Unauthorized)
        );
        assert_eq!(
            bucket.write("b.xml", b"x", &Expected::Absent).map_err(kind),
            Err(ConnectorErrorKind::Unauthorized)
        );
        let large = bucket.with_limits(1, 2);
        assert_eq!(
            large
                .write_from("c.bin", &mut &b"abcde"[..], 5, &Expected::Absent)
                .map_err(kind),
            Err(ConnectorErrorKind::Unauthorized)
        );
    }

    #[test]
    fn the_provider_test_passes_on_a_correct_provider_and_leaves_no_key() {
        let server = FakeS3::start("bk");
        let bucket = connector(&server, "feeds/");
        bucket.test_provider().unwrap();
        assert!(server.keys().is_empty(), "{:?}", server.keys());
    }

    #[test]
    fn the_provider_test_gives_no_conditions_and_leaves_no_key() {
        let server = FakeS3::start("bk");
        server.ignore_put_conditions(true);
        let bucket = connector(&server, "");
        let error = bucket.test_provider().unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::NoConditions);
        assert!(error.to_string().contains("safely"), "{error}");
        assert!(server.keys().is_empty(), "{:?}", server.keys());
    }

    #[test]
    fn the_provider_test_gives_unauthorized_and_unavailable() {
        let server = FakeS3::start("bk");
        server.refuse_all(true);
        let bucket = connector(&server, "");
        assert_eq!(
            bucket.test_provider().unwrap_err().kind(),
            ConnectorErrorKind::Unauthorized
        );
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let mut config = config(&server, "");
        config.endpoint = format!("http://127.0.0.1:{port}");
        let bucket = S3Bucket::new(config, credentials()).unwrap();
        assert_eq!(
            bucket.test_provider().unwrap_err().kind(),
            ConnectorErrorKind::Unavailable
        );
    }

    #[test]
    fn an_endpoint_with_no_server_gives_unavailable() {
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let config = S3Config {
            endpoint: format!("http://127.0.0.1:{port}"),
            region: "us-east-1".to_owned(),
            bucket: "bk".to_owned(),
            url_style: S3UrlStyle::Path,
            prefix: String::new(),
            base_url: None,
        };
        let bucket = S3Bucket::new(config, credentials()).unwrap();
        let error = bucket.read("a.xml").unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
        let error = bucket.list().unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    #[test]
    fn another_status_gives_io_with_the_status_and_the_code() {
        let server = FakeS3::start("other");
        let bucket = connector(&server, "");
        // The fake gives 404 NoSuchBucket for a list of another bucket.
        let error = bucket.list().unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
        assert!(error.to_string().contains("404"), "{error}");
        assert!(error.to_string().contains("NoSuchBucket"), "{error}");
    }

    /// Run a check of the shared suite on a new fake server.
    fn suite_check(check: fn(&dyn Fn() -> Box<dyn Connector>)) {
        let servers = std::cell::RefCell::new(Vec::new());
        check(&|| {
            let server = FakeS3::start("bk");
            let bucket = connector(&server, "feeds/");
            servers.borrow_mut().push(server);
            Box::new(bucket)
        });
    }

    #[test]
    fn the_suite_runs_all_checks() {
        suite_check(suite::run_all);
    }

    #[test]
    fn suite_read_of_an_absent_file_gives_none() {
        suite_check(suite::read_of_an_absent_file_gives_none);
    }

    #[test]
    fn suite_write_then_read_gives_the_bytes_and_the_version() {
        suite_check(suite::write_then_read_gives_the_bytes_and_the_version);
    }

    #[test]
    fn suite_write_over_a_file_with_absent_is_a_conflict() {
        suite_check(suite::write_over_a_file_with_absent_is_a_conflict);
    }

    #[test]
    fn suite_write_with_an_old_version_is_a_conflict() {
        suite_check(suite::write_with_an_old_version_is_a_conflict);
    }

    #[test]
    fn suite_write_with_the_current_version_replaces_the_file() {
        suite_check(suite::write_with_the_current_version_replaces_the_file);
    }

    #[test]
    fn suite_write_from_gives_the_same_file_as_write() {
        suite_check(suite::write_from_gives_the_same_file_as_write);
    }

    #[test]
    fn suite_write_from_with_a_short_reader_writes_nothing() {
        suite_check(suite::write_from_with_a_short_reader_writes_nothing);
    }

    #[test]
    fn suite_stat_gives_the_size_and_a_new_stamp_after_a_change() {
        suite_check(suite::stat_gives_the_size_and_a_new_stamp_after_a_change);
    }

    #[test]
    fn suite_list_gives_each_file_in_path_sequence() {
        suite_check(suite::list_gives_each_file_in_path_sequence);
    }

    #[test]
    fn suite_delete_with_the_current_stamp_removes_the_file() {
        suite_check(suite::delete_with_the_current_stamp_removes_the_file);
    }

    #[test]
    fn suite_delete_with_an_old_stamp_is_a_conflict() {
        suite_check(suite::delete_with_an_old_stamp_is_a_conflict);
    }

    #[test]
    fn suite_delete_of_an_absent_file_is_not_found() {
        suite_check(suite::delete_of_an_absent_file_is_not_found);
    }

    #[test]
    fn suite_a_path_out_of_the_host_is_invalid() {
        suite_check(suite::a_path_out_of_the_host_is_invalid);
    }

    #[test]
    fn suite_sha256_when_given_is_the_sha256_of_the_bytes() {
        suite_check(suite::sha256_when_given_is_the_sha256_of_the_bytes);
    }

    #[test]
    fn suite_set_redirect_without_the_ability_is_unsupported() {
        suite_check(suite::set_redirect_without_the_ability_is_unsupported);
    }

    #[test]
    fn suite_finish_keeps_each_change() {
        suite_check(suite::finish_keeps_each_change);
    }
}
