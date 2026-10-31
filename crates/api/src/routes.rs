//! The JSON routes of the local API.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use axum::Json;
use axum::extract::{Extension, Path, Query, Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::address::AddressSource;
use studio_core::announce;
use studio_core::apps::{self, ConnectedApp};
use studio_core::change_set::{self, RecipientKey, RecipientOperation, Wallet};
use studio_core::collection::{
    Collection, CollectionErrorKind, ConnectorId, ConnectorRow, DraftOperation, DraftOperationId,
    DraftOperationKind, FeedId, FeedRow, LibraryFile, LibraryFileId, PublisherRole, SnapshotId,
};
use studio_core::connector::{
    self, ConnectorErrorKind, LocalFolder, S3Bucket, S3Config, S3Credentials, S3UrlStyle,
    SftpConfig, SftpCredentials, SftpHost, SftpLogin,
};
use studio_core::discover::{FailKind, PublicFeed, ReadFailure};
use studio_core::download;
use studio_core::feed::{Entry, Feed, Recipient};
use studio_core::folder_scan;
use studio_core::grant;
use studio_core::inventory::{self, Hosts};
use studio_core::keysend::{self, Lookup, LookupStatus};
use studio_core::library::{self, Library};
use studio_core::live::{self, LiveStatus};
use studio_core::media;
use studio_core::migrate;
use studio_core::nostr_events::Signing;
use studio_core::pipeline;
use studio_core::playlist;
use studio_core::profiles;
use studio_core::publisher;
use studio_core::remote;
use zeroize::Zeroizing;

use crate::error::ApiError;
use crate::workspace::Workspace;

/// The shared workspace of the local API.
pub(crate) type Shared = Arc<Mutex<Workspace>>;

pub(crate) type ApiResult = Result<Json<Value>, ApiError>;

/// Run blocking core work on a blocking thread (ADR 0015).
pub(crate) async fn blocking<T, F>(shared: Shared, work: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(&mut Workspace) -> Result<T, ApiError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut workspace = lock(&shared)?;
        work(&mut workspace)
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
}

/// Run blocking core work on a blocking thread, with no lock of the
/// workspace. An MP3 encode of a WAV file or a FLAC file takes some
/// seconds, and another request must not wait for it (ADR 0015, ADR 0033
/// item 1).
pub(crate) async fn blocking_unlocked<T, F>(work: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ApiError> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
}

fn lock(shared: &Shared) -> Result<MutexGuard<'_, Workspace>, ApiError> {
    crate::workspace::lock_workspace(shared)
}

pub(crate) fn to_json(value: impl serde::Serialize) -> ApiResult {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

pub(crate) async fn status(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        Ok(Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "vault": w.vault_state(),
            "connectors": w.collection.connectors()?.len(),
            "feeds": w.collection.feeds()?.len(),
        })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct CreateVault {
    passphrase: String,
    #[serde(default)]
    recovery_key: bool,
}

pub(crate) async fn create_vault(
    State(shared): State<Shared>,
    Json(body): Json<CreateVault>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let recovery = w.create_vault(Some(&body.passphrase), body.recovery_key)?;
        // The recovery key is in this one response only. Studio cannot show
        // it again (ADR 0013).
        Ok((
            StatusCode::CREATED,
            Json(json!({ "recovery_key": recovery })),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Strength {
    passphrase: String,
}

/// The strength of a passphrase (ADR 0018). The response contains no part
/// of the passphrase.
pub(crate) async fn passphrase_strength(Json(body): Json<Strength>) -> ApiResult {
    to_json(studio_core::vault::passphrase_strength(&body.passphrase))
}

#[derive(Deserialize)]
pub(crate) struct Unlock {
    passphrase: String,
}

pub(crate) async fn unlock_vault(
    State(shared): State<Shared>,
    Json(body): Json<Unlock>,
) -> ApiResult {
    blocking(shared, move |w| {
        w.unlock_with_passphrase(&body.passphrase)?;
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct NewPassphrase {
    passphrase: String,
}

/// Replace the passphrase of the open vault (ADR 0018). The rule for the
/// passphrase is the rule of the vault creation. The response contains no
/// part of the passphrase.
pub(crate) async fn set_passphrase(
    State(shared): State<Shared>,
    Json(body): Json<NewPassphrase>,
) -> ApiResult {
    blocking(shared, move |w| {
        w.set_passphrase(&body.passphrase)?;
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

pub(crate) async fn lock_vault(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        w.lock();
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct UnlockKey {
    key: String,
}

/// Open the vault with the vault key that the signer of the user decrypted
/// (ADR 0056 item 10). The route does no key derivation. A key that does
/// not open the vault gives 401, and the vault stays closed.
pub(crate) async fn unlock_vault_with_key(
    State(shared): State<Shared>,
    Json(body): Json<UnlockKey>,
) -> ApiResult {
    blocking(shared, move |w| {
        w.unlock_with_key(&body.key).map_err(|error| {
            if error.code() == "wrong_key" {
                ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "wrong_key",
                    "the key does not open the vault",
                )
            } else {
                error
            }
        })?;
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

/// The first Nostr wrapper of the vault file. The vault can be closed.
pub(crate) async fn get_nostr_wrapper(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        if w.vault_state() == crate::workspace::VaultState::Absent {
            return Err(no_nostr_wrapper());
        }
        let file = w.vault_file()?;
        let (pubkey, ciphertext) = file.first_nostr_wrapper().ok_or_else(no_nostr_wrapper)?;
        Ok(Json(json!({ "pubkey": pubkey, "ciphertext": ciphertext })))
    })
    .await
}

fn no_nostr_wrapper() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "no_nostr_wrapper",
        "the vault has no Nostr wrapper",
    )
}

#[derive(Deserialize)]
pub(crate) struct NostrWrapper {
    pubkey: String,
    ciphertext: String,
}

/// Add a Nostr wrapper to the open vault, and save the vault.
pub(crate) async fn add_nostr_wrapper(
    State(shared): State<Shared>,
    Json(body): Json<NostrWrapper>,
) -> ApiResult {
    blocking(shared, move |w| {
        if body.pubkey.is_empty() || body.ciphertext.is_empty() {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_wrapper",
                "give a public key and a ciphertext",
            ));
        }
        require_open_vault(w)?;
        if let Some(vault) = w.vault.as_mut() {
            vault.add_nostr_wrapper(&body.pubkey, &body.ciphertext);
        }
        w.save_vault()?;
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

/// The vault key for the signer of the user, as base64. This is the only
/// route that gives the vault key. The signer encrypts it with NIP-44.
pub(crate) async fn signer_key(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        require_open_vault(w)?;
        let vault = w
            .vault
            .as_ref()
            .ok_or_else(|| ApiError::internal("the vault closed"))?;
        w.touch_vault();
        Ok(Json(json!({ "key": vault.key_for_signer().as_str() })))
    })
    .await
}

fn require_open_vault(w: &Workspace) -> Result<(), ApiError> {
    if w.vault.is_some() {
        return Ok(());
    }
    Err(match w.vault_state() {
        crate::workspace::VaultState::Absent => ApiError::vault_required(),
        _ => ApiError::vault_locked(),
    })
}

/// The connectors. Each connector lists the linked feeds with an address
/// that does not agree with the folder (ADR 0021 item 8).
pub(crate) async fn list_connectors(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let feeds = w.collection.feeds()?;
        let mut list = Vec::new();
        for row in w.collection.connectors()? {
            let linked: Vec<_> = feeds
                .iter()
                .filter(|f| f.connector_id == Some(row.id))
                .cloned()
                .collect();
            let has_address = studio_core::address::folder_url(&row).is_some();
            let mut value = serde_json::to_value(&row).unwrap_or(Value::Null);
            value["disagreeing_feeds"] = json!(studio_core::address::folder_disagreements(
                has_address,
                &linked
            ));
            list.push(value);
        }
        Ok(Json(Value::Array(list)))
    })
    .await
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum NewConnector {
    Local {
        name: String,
        root: String,
        base_url: Option<String>,
    },
    S3(NewS3),
    Sftp(NewSftp),
    Nsite(NewNsite),
    Blossom(NewBlossom),
}

/// Who signs the events of a new nsite or Blossom host (ADR 0044 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum NewSigning {
    /// "Key in the vault".
    Vault,
    /// "My signer".
    Signer,
}

/// The fields of a new nsite site (ADR 0048).
#[derive(Deserialize)]
pub(crate) struct NewNsite {
    name: String,
    public_key: Option<String>,
    d: Option<String>,
    feed_id: Option<i64>,
    relays: Vec<String>,
    servers: Vec<String>,
    gateway: Option<String>,
    title: Option<String>,
    signing: NewSigning,
    secret_key: Option<String>,
}

/// The fields of a new Blossom host (ADR 0048).
#[derive(Deserialize)]
pub(crate) struct NewBlossom {
    name: String,
    public_key: Option<String>,
    servers: Vec<String>,
    signing: NewSigning,
    secret_key: Option<String>,
}

/// The gateway of an nsite host that has none in its fields.
const DEFAULT_GATEWAY: &str = "nsite.lol";

/// The fields of a new S3 host (ADR 0046).
#[derive(Deserialize)]
pub(crate) struct NewS3 {
    name: String,
    endpoint: String,
    region: String,
    bucket: String,
    url_style: S3UrlStyle,
    #[serde(default)]
    prefix: String,
    base_url: Option<String>,
    access_key_id: String,
    secret_access_key: String,
}

/// The fields of a new SFTP host (ADR 0047).
#[derive(Deserialize)]
pub(crate) struct NewSftp {
    name: String,
    host: String,
    #[serde(default = "default_sftp_port")]
    port: u16,
    user: String,
    root: String,
    base_url: Option<String>,
    login: SftpLogin,
    private_key: Option<String>,
    passphrase: Option<String>,
    password: Option<String>,
    host_key: String,
}

fn default_sftp_port() -> u16 {
    22
}

/// The fields of a host key request.
#[derive(Deserialize)]
pub(crate) struct FingerprintRequest {
    host: String,
    #[serde(default = "default_sftp_port")]
    port: u16,
}

/// Give the fingerprint of the host key of an SFTP server (ADR 0047
/// item 3). The route needs no vault. It does not log in.
pub(crate) async fn connector_fingerprint(
    State(shared): State<Shared>,
    Json(body): Json<FingerprintRequest>,
) -> ApiResult {
    let policy = lock(&shared)?.net_policy.clone();
    blocking_unlocked(move || {
        let host = body.host.trim().to_owned();
        if host.is_empty() || body.port == 0 {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_config",
                "the host and the port must not be empty",
            ));
        }
        let fingerprint = SftpHost::fingerprint_of(&host, body.port, &policy)?;
        Ok(Json(json!({ "fingerprint": fingerprint })))
    })
    .await
}

/// Add an SFTP host (ADR 0047). The steps are in this sequence:
/// - the vault,
/// - the field checks,
/// - the connection with the root check,
/// - the secret in the vault,
/// - the collection row.
///
/// When one step fails, the next steps do not occur.
fn add_sftp_connector(
    w: &mut Workspace,
    body: &NewSftp,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let needs_secret = body.login != SftpLogin::Agent;
    if needs_secret && w.vault.is_none() {
        return Err(match w.vault_state() {
            crate::workspace::VaultState::Absent => ApiError::vault_required(),
            _ => ApiError::vault_locked(),
        });
    }
    let invalid =
        |message: &str| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid_config", message);
    let host_name = body.host.trim().to_owned();
    let user = body.user.trim().to_owned();
    let root = body.root.trim().to_owned();
    let host_key = body.host_key.trim().to_owned();
    if host_name.is_empty() || user.is_empty() || root.is_empty() || host_key.is_empty() {
        return Err(invalid(
            "the host, the user, the folder and the host key must not be empty",
        ));
    }
    let base_url = body
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(str::to_owned);
    if let Some(base_url) = &base_url {
        check_http_address(base_url, "the public address")?;
    }
    let (credentials, secret) = sftp_credentials(body)?;
    let config = SftpConfig {
        host: host_name,
        port: body.port,
        user,
        root,
        base_url,
        login: body.login,
        host_key,
    };
    let host = SftpHost::new(config, credentials)?.with_policy(&w.net_policy);
    // A missing root is a field error. The other errors keep their codes.
    host.check_root().map_err(|error| {
        if error.kind() == ConnectorErrorKind::NotFound {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "folder_not_found",
                error.to_string(),
            )
        } else {
            ApiError::from(error)
        }
    })?;
    let secret_ref = secret
        .as_ref()
        .map(|_| format!("connector-{}", uuid::Uuid::new_v4()));
    if let (Some(secret), Some(secret_ref)) = (&secret, &secret_ref) {
        let vault = w
            .vault
            .as_mut()
            .ok_or_else(|| ApiError::internal("the vault closed"))?;
        vault.put(secret_ref, secret.as_bytes())?;
    }
    let added = w.save_vault().and_then(|()| {
        w.collection
            .add_connector(
                SftpHost::KIND,
                &body.name,
                &host.config(),
                secret_ref.as_deref(),
            )
            .map_err(ApiError::from)
    });
    let id = match added {
        Ok(id) => id,
        Err(error) => {
            // The secret does not stay in the vault without a row.
            if let (Some(vault), Some(secret_ref)) = (w.vault.as_mut(), &secret_ref) {
                vault.remove(secret_ref);
                let _ = w.save_vault();
            }
            return Err(error);
        }
    };
    Ok((StatusCode::CREATED, to_json(w.collection.connector(id)?)?))
}

/// The credentials of a new SFTP host and the secret JSON for the vault.
/// The agent login has no secret.
fn sftp_credentials(body: &NewSftp) -> Result<(SftpCredentials, Option<String>), ApiError> {
    let invalid =
        |message: &str| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid_config", message);
    let secret = match body.login {
        SftpLogin::Key => {
            let key = body
                .private_key
                .as_deref()
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| invalid("the private key must not be empty"))?;
            let passphrase = body.passphrase.as_deref().filter(|p| !p.is_empty());
            json!({ "private_key": key, "passphrase": passphrase }).to_string()
        }
        SftpLogin::Password => {
            let password = body
                .password
                .as_deref()
                .filter(|p| !p.is_empty())
                .ok_or_else(|| invalid("the password must not be empty"))?;
            json!({ "password": password }).to_string()
        }
        SftpLogin::Agent => return Ok((SftpCredentials::Agent, None)),
    };
    let credentials = SftpCredentials::from_secret(body.login, Some(secret.as_bytes()))?;
    Ok((credentials, Some(secret)))
}

/// Check that an address starts with `http://` or `https://`. Give 422
/// `invalid_address` when it does not.
fn check_http_address(address: &str, what: &str) -> Result<(), ApiError> {
    let lower = address.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Ok(());
    }
    Err(ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_address",
        format!("{what} must start with http:// or https://"),
    ))
}

/// Add an S3 host (ADR 0046). The steps are in this sequence: the vault,
/// the field checks, the provider test, the secret in the vault, and the
/// collection row. When one step fails, the next steps do not occur.
fn add_s3_connector(
    w: &mut Workspace,
    body: &NewS3,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if w.vault.is_none() {
        return Err(match w.vault_state() {
            crate::workspace::VaultState::Absent => ApiError::vault_required(),
            _ => ApiError::vault_locked(),
        });
    }
    let invalid =
        |message: &str| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid_config", message);
    let endpoint = body.endpoint.trim().to_owned();
    check_http_address(&endpoint, "the endpoint")?;
    let base_url = body
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(str::to_owned);
    if let Some(base_url) = &base_url {
        check_http_address(base_url, "the public address")?;
    }
    let bucket = body.bucket.trim().to_owned();
    if bucket.is_empty() {
        return Err(invalid("the bucket must not be empty"));
    }
    if body.access_key_id.trim().is_empty() || body.secret_access_key.is_empty() {
        return Err(invalid("the access key must not be empty"));
    }
    let mut prefix = body.prefix.trim().to_owned();
    if !prefix.is_empty() && !prefix.ends_with('/') {
        prefix.push('/');
    }
    let config = S3Config {
        endpoint,
        region: body.region.trim().to_owned(),
        bucket,
        url_style: body.url_style,
        prefix,
        base_url,
    };
    let secret = json!({
        "access_key_id": body.access_key_id.trim(),
        "secret_access_key": body.secret_access_key,
    })
    .to_string();
    let host = S3Bucket::new(
        config,
        S3Credentials::new(body.access_key_id.trim(), body.secret_access_key.as_str()),
    )?
    .with_policy(&w.net_policy);
    host.test_provider()?;
    let secret_ref = format!("connector-{}", uuid::Uuid::new_v4());
    let vault = w
        .vault
        .as_mut()
        .ok_or_else(|| ApiError::internal("the vault closed"))?;
    vault.put(&secret_ref, secret.as_bytes())?;
    let added = w.save_vault().and_then(|()| {
        w.collection
            .add_connector(
                S3Bucket::KIND,
                &body.name,
                &host.config(),
                Some(&secret_ref),
            )
            .map_err(ApiError::from)
    });
    let id = match added {
        Ok(id) => id,
        Err(error) => {
            // The secret does not stay in the vault without a row.
            if let Some(vault) = w.vault.as_mut() {
                vault.remove(&secret_ref);
            }
            let _ = w.save_vault();
            return Err(error);
        }
    };
    Ok((StatusCode::CREATED, to_json(w.collection.connector(id)?)?))
}

fn invalid_config(message: &str) -> ApiError {
    ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid_config", message)
}

/// The fields of the request of `POST /api/v1/connectors/nostr-lists`.
#[derive(Deserialize)]
pub(crate) struct NostrListsRequest {
    key: String,
}

/// Give the relay list and the server list of a key (ADR 0048 item 3).
/// The route needs no vault. The key is an `npub`, an `nsec` or 64
/// hexadecimal characters. With an `nsec`, the route keeps only the public
/// key.
pub(crate) async fn nostr_lists(
    State(shared): State<Shared>,
    Json(body): Json<NostrListsRequest>,
) -> ApiResult {
    let lookup_relays = lock(&shared)?.lookup_relays.clone();
    blocking_unlocked(move || {
        let text = body.key.trim();
        let public_key = if text.starts_with("nsec1") {
            studio_core::nostr_events::secret_key_parts(text)?.public_key
        } else {
            text.to_owned()
        };
        let ids = studio_core::nostr_events::key_ids(&public_key)?;
        let key = studio_core::nostr_events::parse_public_key(&ids.public_key)?;
        let lists = connector::key_lists(&key, &lookup_relays)?;
        Ok(Json(json!({
            "public_key": ids.public_key,
            "npub": ids.npub,
            "relays": lists.relays,
            "servers": lists.servers,
        })))
    })
    .await
}

/// The servers of a request, as http or https addresses with no `/` at the
/// end. An empty list gives 422 `invalid_config`. An address with another
/// scheme gives 422 `invalid_address`.
fn clean_servers(servers: &[String]) -> Result<Vec<String>, ApiError> {
    if servers.is_empty() {
        return Err(invalid_config("the file host needs one server"));
    }
    servers
        .iter()
        .map(|server| {
            let server = server.trim().trim_end_matches('/');
            check_http_address(server, "a server address")?;
            Ok(server.to_owned())
        })
        .collect()
}

/// The relays of a request, as ws or wss addresses. An empty list or an
/// address with another scheme gives 422 `invalid_config`.
fn clean_relays(relays: &[String]) -> Result<Vec<String>, ApiError> {
    if relays.is_empty() {
        return Err(invalid_config("the site needs one relay"));
    }
    relays
        .iter()
        .map(|relay| {
            let relay = relay.trim();
            let lower = relay.to_ascii_lowercase();
            if lower.starts_with("ws://") || lower.starts_with("wss://") {
                Ok(relay.to_owned())
            } else {
                Err(invalid_config(
                    "a relay address must start with ws:// or wss://",
                ))
            }
        })
        .collect()
}

/// The gateway of a request, as a host name in lowercase.
fn clean_gateway(gateway: &str) -> Result<String, ApiError> {
    let gateway = gateway.trim().to_ascii_lowercase();
    let ok = !gateway.is_empty()
        && !gateway.starts_with(['.', '-'])
        && !gateway.ends_with(['.', '-'])
        && gateway
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.');
    if ok {
        Ok(gateway)
    } else {
        Err(invalid_config("the gateway must be a host name"))
    }
}

/// The fields that nsite and Blossom hosts share.
struct NostrHost<'a> {
    name: &'a str,
    public_key: Option<&'a str>,
    signing: NewSigning,
    secret_key: Option<&'a str>,
    servers: Vec<String>,
}

/// The key of a new nsite or Blossom host, after the field checks.
struct HostKey {
    /// The public key as hexadecimal text.
    public_key: String,
    /// The secret key, with the signing `vault` only.
    parts: Option<studio_core::nostr_events::SecretKeyParts>,
}

/// Check the key fields of a new nsite or Blossom host (step 2 of ADR 0048).
///
/// With `vault`, `secret_key` must parse. A `public_key` that is not its
/// key gives 422 `key_mismatch`. With `signer`, `public_key` must parse
/// and `secret_key` must be absent.
fn check_host_key(host: &NostrHost<'_>) -> Result<HostKey, ApiError> {
    use studio_core::nostr_events::{key_ids, secret_key_parts};
    let given = match host.public_key.map(str::trim).filter(|k| !k.is_empty()) {
        Some(text) => Some(key_ids(text)?.public_key),
        None => None,
    };
    match host.signing {
        NewSigning::Vault => {
            let text = host
                .secret_key
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| invalid_config("the secret key must not be empty"))?;
            let parts = secret_key_parts(text)?;
            if given.is_some_and(|key| key != parts.public_key) {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "key_mismatch",
                    "the secret key is not the key of the public key",
                ));
            }
            Ok(HostKey {
                public_key: parts.public_key.clone(),
                parts: Some(parts),
            })
        }
        NewSigning::Signer => {
            if host.secret_key.is_some() {
                return Err(invalid_config("a signer file host takes no secret key"));
            }
            let public_key =
                given.ok_or_else(|| invalid_config("the public key must not be empty"))?;
            Ok(HostKey {
                public_key,
                parts: None,
            })
        }
    }
}

/// Add an nsite site (ADR 0048, ADR 0044). The steps are in this sequence:
/// the vault, the field checks, the connection, the secret in the vault,
/// and the collection row. When one step fails, the next steps do not
/// occur.
fn add_nsite_connector(
    w: &mut Workspace,
    body: &NewNsite,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    require_vault_for(w, body.signing)?;
    let host = NostrHost {
        name: &body.name,
        public_key: body.public_key.as_deref(),
        signing: body.signing,
        secret_key: body.secret_key.as_deref(),
        servers: clean_servers(&body.servers)?,
    };
    let key = check_host_key(&host)?;
    let relays = clean_relays(&body.relays)?;
    let d = match (body.d.as_deref().map(str::trim), body.feed_id) {
        (Some(d), _) if !d.is_empty() => d.to_owned(),
        (_, Some(feed_id)) => {
            let guid = w.collection.feed(FeedId(feed_id))?.podcast_guid;
            let guid = guid.ok_or_else(|| invalid_config("the feed has no GUID"))?;
            studio_core::nostr_events::site_name_from_guid(&guid)
        }
        _ => return Err(invalid_config("give a site name or a feed")),
    };
    if !studio_core::nostr_events::valid_site_name(&d) {
        return Err(invalid_config("the site name is not valid"));
    }
    let gateway = clean_gateway(body.gateway.as_deref().unwrap_or(DEFAULT_GATEWAY))?;
    let title = body
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    let config = json!({
        "public_key": key.public_key,
        "d": d,
        "relays": relays,
        "servers": host.servers,
        "gateway": gateway,
        "signing": signing_text(body.signing),
        "title": title,
    })
    .to_string();
    // The host checks the configuration, and then it reads the manifest.
    let secret = key
        .parts
        .as_ref()
        .map(studio_core::nostr_events::SecretKeyParts::vault_json);
    let made = studio_core::connector::NsiteHost::from_config(
        &config,
        secret.as_deref().map(String::as_bytes),
        Signing::default(),
    )?
    .with_policy(&w.net_policy);
    let state = made.site_state()?;
    if state.relays.iter().all(|r| r.newest.is_err()) {
        // The policy refused each relay (ADR 0057).
        if let Some(refused) = state
            .relays
            .iter()
            .filter_map(|r| r.newest.as_ref().err())
            .find(|e| e.kind() == ConnectorErrorKind::NotPublic)
        {
            return Err(refused.clone().into());
        }
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "host_unavailable",
            "no relay answered",
        ));
    }
    connector::check_blossom_servers(&host.servers, &w.net_policy)?;
    add_nostr_row(
        w,
        studio_core::connector::NsiteHost::KIND,
        host.name,
        &config,
        secret.as_deref().map(String::as_bytes),
    )
}

/// Add a Blossom host (ADR 0048, ADR 0044). The steps are the steps of
/// [`add_nsite_connector`], with no relays and no manifest.
fn add_blossom_connector(
    w: &mut Workspace,
    body: &NewBlossom,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    require_vault_for(w, body.signing)?;
    let host = NostrHost {
        name: &body.name,
        public_key: body.public_key.as_deref(),
        signing: body.signing,
        secret_key: body.secret_key.as_deref(),
        servers: clean_servers(&body.servers)?,
    };
    let key = check_host_key(&host)?;
    let config = json!({
        "public_key": key.public_key,
        "servers": host.servers,
        "signing": signing_text(body.signing),
    })
    .to_string();
    let secret = key
        .parts
        .as_ref()
        .map(studio_core::nostr_events::SecretKeyParts::vault_json);
    studio_core::connector::BlossomHost::from_config(
        &config,
        secret.as_deref().map(String::as_bytes),
        Vec::new(),
        Signing::default(),
    )?;
    connector::check_blossom_servers(&host.servers, &w.net_policy)?;
    add_nostr_row(
        w,
        studio_core::connector::BlossomHost::KIND,
        host.name,
        &config,
        secret.as_deref().map(String::as_bytes),
    )
}

fn signing_text(signing: NewSigning) -> &'static str {
    match signing {
        NewSigning::Vault => "vault",
        NewSigning::Signer => "signer",
    }
}

/// Step 1 of the add routes of nsite and Blossom: the signing `vault`
/// needs the open vault.
fn require_vault_for(w: &Workspace, signing: NewSigning) -> Result<(), ApiError> {
    if signing == NewSigning::Vault && w.vault.is_none() {
        return Err(match w.vault_state() {
            crate::workspace::VaultState::Absent => ApiError::vault_required(),
            _ => ApiError::vault_locked(),
        });
    }
    Ok(())
}

/// Steps 4 and 5 of the add routes of nsite and Blossom: the secret goes
/// into the vault, and then the row goes into the collection. When a step
/// fails, the secret does not stay in the vault.
fn add_nostr_row(
    w: &mut Workspace,
    kind: &str,
    name: &str,
    config: &str,
    secret: Option<&[u8]>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let secret_ref = secret.map(|_| format!("connector-{}", uuid::Uuid::new_v4()));
    if let (Some(secret), Some(secret_ref)) = (secret, &secret_ref) {
        let vault = w
            .vault
            .as_mut()
            .ok_or_else(|| ApiError::internal("the vault closed"))?;
        vault.put(secret_ref, secret)?;
    }
    let added = w.save_vault().and_then(|()| {
        w.collection
            .add_connector(kind, name, config, secret_ref.as_deref())
            .map_err(ApiError::from)
    });
    let id = match added {
        Ok(id) => id,
        Err(error) => {
            // The secret does not stay in the vault without a row.
            if let (Some(vault), Some(secret_ref)) = (w.vault.as_mut(), &secret_ref) {
                vault.remove(secret_ref);
                let _ = w.save_vault();
            }
            return Err(error);
        }
    };
    Ok((StatusCode::CREATED, to_json(w.collection.connector(id)?)?))
}

/// Give the state of each relay of an nsite site (ADR 0044 item 6). The
/// answer has `consistent` and one entry for each relay.
pub(crate) async fn connector_site(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.kind != studio_core::connector::NsiteHost::KIND {
            return Err(invalid_config("only an nsite file host has a site state"));
        }
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let state = connector::nsite_site_state(&row, w.vault.as_ref(), &w.net_policy)?;
        let relays: Vec<Value> = state
            .relays
            .iter()
            .map(|relay| match &relay.newest {
                Ok(manifest) => json!({
                    "url": relay.address,
                    "id": manifest.as_ref().map(|m| m.id.clone()),
                    "created_at": manifest.as_ref().map(|m| m.created_at),
                    "error": Value::Null,
                }),
                Err(error) => json!({
                    "url": relay.address,
                    "id": Value::Null,
                    "created_at": Value::Null,
                    "error": error.to_string(),
                }),
            })
            .collect();
        Ok(Json(
            json!({ "consistent": state.consistent, "relays": relays }),
        ))
    })
    .await
}

/// The configuration of an nsite or Blossom row with the new relays,
/// servers and gateway of a change (ADR 0048 item 3). Each field is
/// checked before the row changes. A field for a different kind gives 422
/// `invalid_config`.
fn changed_nostr_config(
    row: &ConnectorRow,
    body: &UpdateConnector,
) -> Result<Option<String>, ApiError> {
    use studio_core::connector::{BlossomHost, NsiteHost};
    let is_nsite = row.kind == NsiteHost::KIND;
    let is_blossom = row.kind == BlossomHost::KIND;
    if body.relays.is_some() && !is_nsite {
        return Err(invalid_config("only an nsite file host has relays"));
    }
    if body.gateway.is_some() && !is_nsite {
        return Err(invalid_config("only an nsite file host has a gateway"));
    }
    if body.servers.is_some() && !(is_nsite || is_blossom) {
        return Err(invalid_config(
            "only an nsite or Blossom file host has servers",
        ));
    }
    if body.relays.is_none() && body.gateway.is_none() && body.servers.is_none() {
        return Ok(None);
    }
    let mut config: Value =
        serde_json::from_str(&row.config).map_err(|e| ApiError::internal(e.to_string()))?;
    if let Some(relays) = &body.relays {
        config["relays"] = json!(clean_relays(relays)?);
    }
    if let Some(servers) = &body.servers {
        config["servers"] = json!(clean_servers(servers)?);
    }
    if let Some(gateway) = &body.gateway {
        config["gateway"] = json!(clean_gateway(gateway)?);
    }
    // The host checks the new configuration. The check needs no secret, so
    // it uses the signing `signer`.
    let mut probe = config.clone();
    probe["signing"] = json!("signer");
    let probe = probe.to_string();
    if is_nsite {
        NsiteHost::from_config(&probe, None, Signing::default())?;
    } else {
        BlossomHost::from_config(&probe, None, Vec::new(), Signing::default())?;
    }
    Ok(Some(config.to_string()))
}

pub(crate) async fn add_connector(
    State(shared): State<Shared>,
    Json(body): Json<NewConnector>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let (name, root, base_url) = match body {
            NewConnector::Local {
                name,
                root,
                base_url,
            } => {
                // A local folder host reads and writes folders of the server
                // computer (ADR 0055 item 24).
                if w.rules.is_some() {
                    return Err(not_on_server(
                        "a local folder file host is not available on the server",
                    ));
                }
                (name, root, base_url)
            }
            NewConnector::S3(body) => return add_s3_connector(w, &body),
            NewConnector::Sftp(body) => return add_sftp_connector(w, &body),
            NewConnector::Nsite(body) => return add_nsite_connector(w, &body),
            NewConnector::Blossom(body) => return add_blossom_connector(w, &body),
        };
        let root = expand_home(root.trim());
        if !std::path::Path::new(&root).is_absolute() {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_config",
                "the folder must be an absolute path",
            ));
        }
        if !std::path::Path::new(&root).is_dir() {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "folder_not_found",
                format!("{root} is not a folder on this computer"),
            ));
        }
        let base_url = base_url.filter(|u| !u.trim().is_empty());
        let folder = LocalFolder::new(root, base_url);
        let id = w
            .collection
            .add_connector(LocalFolder::KIND, &name, &folder.config(), None)?;
        Ok((StatusCode::CREATED, to_json(w.collection.connector(id)?)?))
    })
    .await
}

/// The refusal of a function of the desktop only (ADR 0055 items 24 and 25).
fn not_on_server(message: &str) -> ApiError {
    ApiError::new(StatusCode::FORBIDDEN, "not_on_server", message)
}

/// Replace a leading `~` with the home folder of the user that runs Studio.
fn expand_home(path: &str) -> String {
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    match (path, home) {
        ("~", Some(home)) => home.to_string_lossy().into_owned(),
        (p, Some(home)) if p.starts_with("~/") => home.join(&p[2..]).to_string_lossy().into_owned(),
        (p, _) => p.to_owned(),
    }
}

#[derive(Deserialize)]
pub(crate) struct UpdateConnector {
    name: Option<String>,
    base_url: Option<String>,
    host_key: Option<String>,
    relays: Option<Vec<String>>,
    servers: Option<Vec<String>>,
    gateway: Option<String>,
}

/// Change the name or the public address of a connector (ADR 0021,
/// ADR 0036 item 11). The caller can send one field or both.
///
/// Only a local folder host has a public address. An empty `base_url`
/// removes the address. A `base_url` that is not an `http` or `https`
/// address gives 422 `invalid_address`, with no change of the connector.
pub(crate) async fn update_connector(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateConnector>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = ConnectorId(id);
        let row = w.collection.connector(id)?;
        // The new host key is checked first, so that a refusal changes nothing.
        if let Some(host_key) = &body.host_key {
            accept_host_key(w, &row, host_key)?;
        }
        let row = w.collection.connector(id)?;
        // The new relays, servers and gateway are checked before a change.
        let new_config = changed_nostr_config(&row, &body)?;
        if body.gateway.is_some() && row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        if let Some(name) = &body.name {
            w.collection.set_connector_name(id, name)?;
        }
        if let Some(base_url) = &body.base_url {
            set_connector_base_url(w, &row, base_url)?;
        }
        if let Some(config) = &new_config {
            w.collection.set_connector_config(id, config)?;
            if body.gateway.is_some() {
                // A new gateway changes the address of each linked feed.
                studio_core::address::apply_folder_address(
                    &mut w.collection,
                    id,
                    w.vault.as_ref(),
                )?;
            }
        }
        // A change of the host can change a secret. The grant vaults of the
        // feeds that use the host are made again (ADR 0058 item 7).
        for feed in w.collection.feeds()? {
            if feed.connector_id == Some(id) {
                w.sync_grant_vaults(Some(feed.id));
            }
        }
        to_json(w.collection.connector(id)?)
    })
    .await
}

/// Accept a new host key for an SFTP host (ADR 0047 item 3). The server
/// must give the same fingerprint at this time. If it gives a different
/// fingerprint, the route gives 409 `host_key_mismatch`, and the row does
/// not change.
fn accept_host_key(w: &mut Workspace, row: &ConnectorRow, host_key: &str) -> Result<(), ApiError> {
    let invalid =
        |message: &str| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid_config", message);
    if row.kind != SftpHost::KIND {
        return Err(invalid("only an SFTP file host has a host key"));
    }
    let host_key = host_key.trim();
    if !host_key.starts_with("SHA256:") {
        return Err(invalid("the host key must be a SHA256 fingerprint"));
    }
    let mut config: SftpConfig =
        serde_json::from_str(&row.config).map_err(|e| ApiError::internal(e.to_string()))?;
    let seen = SftpHost::fingerprint_of(&config.host, config.port, &w.net_policy)?;
    if seen != host_key {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "host_key_mismatch",
            format!("the server gives the host key {seen}"),
        ));
    }
    config.host_key = seen;
    let text = serde_json::to_string(&config).map_err(|e| ApiError::internal(e.to_string()))?;
    w.collection.set_connector_config(row.id, &text)?;
    Ok(())
}

/// Give a local folder host, an S3 host or an SFTP host a new public
/// address. An empty `base_url` removes the address (ADR 0021, ADR 0036
/// item 11). The address is typed, not inferred, so Studio does not
/// replace it after this. Each feed address of the folder then changes,
/// as after a folder match.
///
/// A connector of another kind has no public address, so the function
/// does nothing for it.
///
/// # Errors
///
/// Gives 422 `invalid_address` when `base_url` is not empty and is not
/// an `http` or `https` address.
fn set_connector_base_url(
    w: &mut Workspace,
    row: &ConnectorRow,
    base_url: &str,
) -> Result<(), ApiError> {
    if ![LocalFolder::KIND, S3Bucket::KIND, SftpHost::KIND].contains(&row.kind.as_str()) {
        return Ok(());
    }
    let trimmed = base_url.trim();
    let new_url = if trimmed.is_empty() {
        None
    } else {
        let lower = trimmed.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_address",
                "the public address must start with http:// or https://",
            ));
        }
        Some(trimmed.to_owned())
    };
    let mut config: Value =
        serde_json::from_str(&row.config).map_err(|e| ApiError::internal(e.to_string()))?;
    config["base_url"] = new_url.map_or(Value::Null, Value::String);
    // Only a local folder can have an inferred address.
    if row.kind == LocalFolder::KIND {
        config["base_url_inferred"] = Value::Bool(false);
    }
    w.collection
        .set_connector_config(row.id, &config.to_string())?;
    studio_core::address::apply_folder_address(&mut w.collection, row.id, w.vault.as_ref())?;
    Ok(())
}

/// The feeds of the collection. Each row has its draft state and its cover
/// image (ADR 0039 item 2, task 026a).
pub(crate) async fn list_feeds(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        // The cover image comes from the same source as the file list. A
        // feed with no readable source has no image and no link.
        let mut feeds = Vec::new();
        for row in w.collection.feeds()? {
            let feed = files_feed(w, row.id).ok().flatten();
            feeds.push((row, feed));
        }
        // The links of each feed give its destination (ADR 0085).
        let catalogs = publisher::catalogs(&feeds);
        let mut rows = Vec::new();
        for ((row, feed), catalog) in feeds.iter().zip(catalogs) {
            let state = draft_state(w, row)?;
            let image = feed.as_ref().and_then(|feed| feed.image.clone());
            // The UI applies the cover rule to both addresses (ADR 0060).
            let rss_image_url = feed
                .as_ref()
                .and_then(|feed| feed.rss_image.as_ref())
                .and_then(|rss_image| rss_image.url.clone());
            let mut value =
                serde_json::to_value(row).map_err(|e| ApiError::internal(e.to_string()))?;
            value["draft_state"] = json!(state);
            value["image"] = json!(image);
            value["rss_image_url"] = json!(rss_image_url);
            // The release list marks a feed that apps do not list (ADR 0063).
            value["hidden"] = json!(feed.as_ref().is_some_and(Feed::hidden));
            value["release_ids"] =
                json!(catalog.releases.iter().map(|id| id.0).collect::<Vec<_>>());
            value["publisher_ids"] =
                json!(catalog.publishers.iter().map(|id| id.0).collect::<Vec<_>>());
            // The two sides of a link give the first role (ADR 0087).
            value["roles_linked"] = json!(catalog.roles_linked);
            // The levels of the hierarchy (ADR 0088).
            value["child_ids"] = json!(catalog.children.iter().map(|id| id.0).collect::<Vec<_>>());
            value["parent_ids"] = json!(catalog.parents.iter().map(|id| id.0).collect::<Vec<_>>());
            rows.push(value);
        }
        to_json(rows)
    })
    .await
}

/// The draft state of a feed: `none`, `new`, `changed` or `unchanged`.
/// A failed change check gives `changed` (ADR 0039 item 2, task 026a).
fn draft_state(w: &Workspace, row: &FeedRow) -> Result<&'static str, ApiError> {
    if w.collection.draft(row.id)?.is_none() {
        return Ok("none");
    }
    if row.last_version.is_none() {
        return Ok("new");
    }
    Ok(match pipeline::draft_changed(&w.collection, row.id) {
        Ok(false) => "unchanged",
        Ok(true) | Err(_) => "changed",
    })
}

#[derive(Deserialize)]
pub(crate) struct NewFeed {
    connector_id: Option<i64>,
    path: Option<String>,
}

pub(crate) async fn add_feed(
    State(shared): State<Shared>,
    Json(body): Json<NewFeed>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let connector = match (body.connector_id, body.path.as_deref()) {
            (Some(id), Some(path)) => {
                let row = w.collection.connector(ConnectorId(id))?;
                let made = w.connector_of(&row)?;
                Some((row.id, path.to_owned(), made.public_url(path)))
            }
            (None, None) => None,
            _ => {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_feed",
                    "give both a connector and a path, or neither",
                ));
            }
        };
        let id = w
            .collection
            .add_feed(None, connector.as_ref().map(|(c, p, _)| (*c, p.as_str())))?;
        if let Some((_, _, url)) = &connector {
            w.collection
                .set_address(id, AddressSource::Folder, url.as_deref())?;
        }
        Ok((StatusCode::CREATED, to_json(w.collection.feed(id)?)?))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct NewAlbumFeed {
    connector_id: i64,
    path: String,
    title: String,
    author: String,
    /// `music` (the default), `publisher` (ADR 0041 item 2) or `musicL`
    /// (ADR 0053 item 2).
    medium: Option<String>,
    /// `artist` or `label`. A publisher feed needs it (ADR 0041 item 1).
    role: Option<String>,
    /// The ID of the source album. The new feed is a copy of its draft
    /// (ADR 0077 item 4).
    from: Option<i64>,
}

/// Make a new feed in a connected folder, as a draft with no hosted file
/// (ADR 0032 items 6 to 9, ADR 0041 items 1 and 2).
pub(crate) async fn new_album_feed(
    State(shared): State<Shared>,
    Json(body): Json<NewAlbumFeed>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let medium = body.medium.as_deref().unwrap_or("music");
        let (publisher, playlist) = match medium {
            "music" => (false, false),
            "publisher" => (true, false),
            "musicL" => (false, true),
            _ => {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_feed",
                    "the medium is music, publisher or musicL",
                ));
            }
        };
        let title = body.title.trim();
        let author = body.author.trim();
        if title.is_empty() || author.is_empty() {
            let message = if publisher {
                "give the name of the publisher feed"
            } else if playlist {
                "give the name of the playlist"
            } else {
                "give the album title and the artist"
            };
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_feed",
                message,
            ));
        }
        // The role is checked before any change, so a refusal changes
        // nothing.
        let role = if publisher {
            // The form offers the first-class roles (ADR 0083 item 3).
            let text = body.role.as_deref().unwrap_or_default();
            Some(
                PublisherRole::parse(text)
                    .filter(|role| {
                        matches!(
                            role,
                            PublisherRole::Artist | PublisherRole::Label | PublisherRole::Hosting
                        )
                    })
                    .ok_or_else(|| {
                        ApiError::new(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            "invalid_feed",
                            "give the role of the publisher feed: artist, label or hosting",
                        )
                    })?,
            )
        } else {
            None
        };
        let row = w.collection.connector(ConnectorId(body.connector_id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = w.connector_of(&row)?;
        let id = if let Some(from) = body.from {
            // A copy starts only from an album, and makes an album
            // (ADR 0077 item 4). The checks come before any draft starts.
            let source = FeedId(from);
            if medium != "music" || w.collection.feed(source)?.medium.as_deref() != Some("music") {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_feed",
                    "a copy starts from an album and makes an album",
                ));
            }
            ensure_draft(w, source)?;
            pipeline::copy_feed(
                &mut w.collection,
                made.as_ref(),
                row.id,
                &body.path,
                title,
                author,
                source,
            )?
        } else {
            pipeline::new_feed(
                &mut w.collection,
                made.as_ref(),
                row.id,
                &body.path,
                title,
                author,
                medium,
            )?
        };
        if let Some(role) = role {
            w.collection.set_publisher_roles(id, &[role], true)?;
        }
        Ok((StatusCode::CREATED, to_json(w.collection.feed(id)?)?))
    })
    .await
}

pub(crate) async fn refresh_feed(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        // The feed may give a different image at the same address, or a
        // different address, after this read (ADR 0032 item 13).
        w.images.clear();
        let row = w.collection.feed(id)?;
        let Some(connector_id) = row.connector_id else {
            // A read-only feed reads from its public address (ADR 0020).
            let hosted = pipeline::refresh_public(&mut w.collection, &w.web, id)?;
            w.record_remote_files(id, &hosted);
            return to_json(w.collection.feed(id)?);
        };
        let connector_row = w.collection.connector(connector_id)?;
        if connector_row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = w.connector_of(&connector_row)?;
        let hosted = pipeline::refresh(&mut w.collection, made.as_ref(), id)?;
        // A feed that the user added from a host folder gets its release
        // folder at its first read (ADR 0038 item 4).
        if row.folder.is_none()
            && let Some(path) = row.path.as_deref()
        {
            give_linked_feed_folder(w, made.as_ref(), id, path);
        }
        w.record_remote_files(id, &hosted);
        to_json(w.collection.feed(id)?)
    })
    .await
}

pub(crate) async fn delete_feed(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let id = FeedId(id);
        // Remove the rows and the copies of the pending files of the feed
        // before the feed goes out (ADR 0032 item 12).
        pipeline::discard_draft(&w.collection, id)?;
        w.collection.delete_feed(id)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

pub(crate) async fn delete_connector(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        w.collection.delete_connector(row.id)?;
        if let (Some(secret_ref), Some(vault)) = (&row.secret_ref, w.vault.as_mut()) {
            vault.remove(secret_ref);
            w.save_vault()?;
        }
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The feed files on the host of a connector (ADR 0019). Each candidate
/// tells if the collection already has it.
pub(crate) async fn connector_feeds(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = w.connector_of(&row)?;
        let candidates = studio_core::discover::feed_candidates(made.as_ref())?;
        let known: Vec<String> = w
            .collection
            .feeds()?
            .into_iter()
            .filter(|f| f.connector_id == Some(row.id))
            .filter_map(|f| f.path)
            .collect();
        let feeds: Vec<Value> = candidates
            .feeds
            .iter()
            .map(|c| {
                let mut value = serde_json::to_value(c).unwrap_or(Value::Null);
                value["in_collection"] = Value::Bool(known.contains(&c.path));
                value
            })
            .collect();
        Ok(Json(
            json!({ "feeds": feeds, "truncated": candidates.truncated }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Discover {
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    guids: Vec<String>,
}

/// The URLs and the feed GUIDs that the collection has.
fn collection_keys(w: &Workspace) -> Result<(Vec<String>, Vec<String>), ApiError> {
    let feeds = w.collection.feeds()?;
    let urls = feeds.iter().filter_map(|f| f.public_url.clone()).collect();
    let guids = feeds
        .iter()
        .filter_map(|f| f.podcast_guid.as_deref().map(str::to_ascii_lowercase))
        .collect();
    Ok((urls, guids))
}

fn in_collection(found: &studio_core::discover::Found, keys: &(Vec<String>, Vec<String>)) -> bool {
    keys.0.contains(&found.url)
        || found
            .podcast_guid
            .as_deref()
            .is_some_and(|g| keys.1.contains(&g.to_ascii_lowercase()))
}

/// Find feeds from addresses and GUIDs (ADR 0020).
pub(crate) async fn discover(
    State(shared): State<Shared>,
    Json(body): Json<Discover>,
) -> ApiResult {
    blocking(shared, move |w| {
        let found =
            studio_core::discover::from_sources(&w.web, &w.index, &body.addresses, &body.guids);
        // The policy refused an address and discovery found nothing
        // (ADR 0057 item 6).
        if found.feeds.is_empty()
            && let Some(refused) = found.failed.iter().find(|f| f.kind == FailKind::NotPublic)
        {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "address_not_public",
                refused.message.clone(),
            ));
        }
        let keys = collection_keys(w)?;
        let feeds: Vec<Value> = found
            .feeds
            .iter()
            .map(|f| {
                let mut value = serde_json::to_value(f).unwrap_or(Value::Null);
                value["in_collection"] = Value::Bool(in_collection(f, &keys));
                value
            })
            .collect();
        w.discovered = found
            .feeds
            .into_iter()
            .map(|f| (f.url.clone(), f))
            .collect();
        Ok(Json(
            json!({ "feeds": feeds, "failed": found.failed, "truncated": found.truncated }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Search {
    q: String,
}

/// Search musicindex for feeds by name.
pub(crate) async fn search(
    State(shared): State<Shared>,
    axum::extract::Query(query): axum::extract::Query<Search>,
) -> ApiResult {
    blocking(shared, move |w| to_json(w.index.search(&query.q)?)).await
}

/// The model of a feed for the profile search: its draft, or else its
/// newest snapshot. A feed with neither gives `None`.
fn recorded_model(w: &Workspace, id: FeedId) -> Option<Feed> {
    let xml = if let Some(draft) = w.collection.draft(id).ok()? {
        draft.draft_xml
    } else {
        let newest = w.collection.snapshots(id).ok()?.into_iter().next()?;
        w.collection.snapshot_xml(id, newest.id).ok()?
    };
    Feed::read(&studio_core::xml::Document::parse_bytes(&xml).ok()?).ok()
}

#[derive(Deserialize)]
pub(crate) struct ProfilesQuery {
    q: String,
    feed: Option<i64>,
}

/// Find Nostr profiles for a key, a web address, a NIP-05 address or a
/// name (ADR 0074 items 7 to 15). The request to each relay and to each
/// other server goes through the network policy, with no lock of the
/// workspace.
pub(crate) async fn nostr_profiles(
    State(shared): State<Shared>,
    Query(query): Query<ProfilesQuery>,
) -> ApiResult {
    let text = query.q.trim().to_owned();
    if text.is_empty() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_query",
            "enter a key, a NIP-05 address or a name",
        ));
    }
    let feed = query.feed;
    let (web, policy, lookup_relays, relay, index, keys, link, scheme) =
        blocking(shared, move |w| {
            let mut keys: Vec<(String, String)> = Vec::new();
            for row in w.collection.feeds()? {
                if let Some(model) = recorded_model(w, row.id) {
                    for pair in profiles::feed_keys(&model) {
                        if !keys.contains(&pair) {
                            keys.push(pair);
                        }
                    }
                }
            }
            let link = feed
                .and_then(|id| recorded_model(w, FeedId(id)))
                .and_then(|model| model.link);
            Ok((
                w.web.clone(),
                w.net_policy.clone(),
                w.lookup_relays.clone(),
                profiles::search_relay(&w.collection)?,
                w.index.clone(),
                keys,
                link,
                w.nip05_scheme,
            ))
        })
        .await?;
    blocking_unlocked(move || {
        let cx = profiles::Context {
            web: &web,
            policy: &policy,
            lookup_relays: &lookup_relays,
            search_relay: Some(relay.as_str()).filter(|r| !r.is_empty()),
            index: &index,
            collection_keys: &keys,
            nip05_scheme: scheme,
            limit: profiles::SOURCE_LIMIT,
        };
        to_json(profiles::find(
            &cx,
            &profiles::parse_query(&text),
            link.as_deref(),
        ))
    })
    .await
}

/// The Nostr settings: the search relay (ADR 0074 item 16).
pub(crate) async fn get_nostr_settings(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        Ok(Json(
            json!({ "search_relay": profiles::search_relay(&w.collection)? }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct NostrSettingsBody {
    search_relay: String,
}

/// Set the search relay. An empty text stops the relay search.
pub(crate) async fn put_nostr_settings(
    State(shared): State<Shared>,
    Json(body): Json<NostrSettingsBody>,
) -> ApiResult {
    blocking(shared, move |w| {
        if !profiles::valid_search_relay(&body.search_relay) {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_relay",
                "the search relay must be empty or start with wss:// or ws://",
            ));
        }
        profiles::set_search_relay(&w.collection, &body.search_relay)?;
        Ok(Json(
            json!({ "search_relay": profiles::search_relay(&w.collection)? }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Import {
    urls: Vec<String>,
}

/// One failed read of an import, in the form of a discovery failure. The UI
/// gives the words for its kind (musicindex ADR 0023 R-MSG-1).
fn failed_json(url: &str, failure: &ReadFailure) -> Value {
    json!({
        "source": url,
        "kind": failure.kind,
        "title": null,
        "url": url,
        "message": failure.message,
    })
}

/// True when the collection has the feed, by its read address or its GUID.
fn is_known(read: &PublicFeed, keys: &(Vec<String>, Vec<String>)) -> bool {
    let guid = read
        .feed
        .podcast_guid
        .as_deref()
        .map(str::to_ascii_lowercase);
    keys.0.contains(&read.url) || guid.is_some_and(|g| keys.1.contains(&g))
}

/// Add a read-only feed with the address where Studio read it (ADR 0021).
fn add_read_feed(
    w: &mut Workspace,
    read: &PublicFeed,
) -> Result<studio_core::collection::FeedRow, ApiError> {
    let id = w.collection.add_feed(Some(&read.url), None)?;
    w.collection.record_state(id, &read.feed, &read.version)?;
    // A published feed gets its release folder and its feed XML at once
    // (ADR 0038 item 4). Without a library folder, the move into the
    // library gives the folder later. A failure here does not stop the
    // add: the feed stays in the collection with no folder.
    if w.collection.setting("library_root")?.is_some() {
        match w.library() {
            Ok(library) => {
                match library::give_published_folder(&w.collection, &library, id, &read.bytes) {
                    Ok(_) => w.record_remote_files(id, &read.feed),
                    Err(error) => {
                        eprintln!("studio: feed {} has no release folder yet: {error}", id.0);
                    }
                }
            }
            Err(error) => eprintln!("studio: feed {} has no release folder yet: {error:?}", id.0),
        }
    }
    Ok(w.collection.feed(id)?)
}

/// Add read-only feeds by address (ADR 0020). A feed that the collection
/// has is not added again.
pub(crate) async fn import_feeds(
    State(shared): State<Shared>,
    Json(body): Json<Import>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let mut added = Vec::new();
        let mut failed = Vec::new();
        for url in body.urls {
            let keys = collection_keys(w)?;
            let found = match w.discovered.get(&url) {
                Some(found) => PublicFeed {
                    feed: found.feed.clone(),
                    version: found.version.clone(),
                    url: found.url.clone(),
                    bytes: found.bytes.clone(),
                },
                None => match studio_core::discover::read_feed(&w.web, &url) {
                    Ok(read) => read,
                    Err(failure) => {
                        failed.push(failed_json(&url, &failure));
                        continue;
                    }
                },
            };
            if is_known(&found, &keys) {
                continue;
            }
            added.push(add_read_feed(w, &found)?);
        }
        Ok((
            StatusCode::CREATED,
            Json(json!({ "added": added, "failed": failed })),
        ))
    })
    .await
}

/// Link each read-only feed to its file in the folder of a connector, by
/// `podcast:guid` (ADR 0019, ADR 0020). Then find the folder address from
/// the linked feeds (ADR 0021).
pub(crate) async fn match_connector(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = w.connector_of(&row)?;
        let candidates = studio_core::discover::feed_candidates(made.as_ref())?;
        let mut linked = Vec::new();
        for feed in w.collection.feeds()? {
            if feed.connector_id.is_some() {
                continue;
            }
            let Some(guid) = feed.podcast_guid.as_deref() else {
                continue;
            };
            let file = candidates.feeds.iter().find(|c| {
                c.podcast_guid
                    .as_deref()
                    .is_some_and(|g| g.eq_ignore_ascii_case(guid))
            });
            if let Some(file) = file {
                w.collection
                    .set_feed_connector(feed.id, row.id, &file.path)?;
                give_linked_feed_folder(w, made.as_ref(), feed.id, &file.path);
                // The feed keeps its draft, now on the host (ADR 0067
                // item 13). An error goes into the log and the link stays.
                if let Err(error) =
                    pipeline::rebase_linked_draft(&w.collection, made.as_ref(), feed.id)
                {
                    eprintln!(
                        "studio: feed {} keeps its link, but its draft was not rebased: {error}",
                        feed.id.0
                    );
                }
                linked.push(feed.id);
            }
        }
        let folder_address = studio_core::address::apply_folder_address(
            &mut w.collection,
            row.id,
            w.vault.as_ref(),
        )?;
        // A new host gets its inventory immediately, so that the Files
        // page shows its copies (ADR 0036 item 6).
        let hosts = w.hosts()?;
        let inventory = inventory::scan(&w.collection, &hosts, row.id)?;
        let linked = linked
            .into_iter()
            .map(|id| w.collection.feed(id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Json(
            json!({ "linked": linked, "folder_address": folder_address, "inventory": inventory }),
        ))
    })
    .await
}

/// Give a feed that a folder match linked its release folder and its feed
/// XML, from the file on the host (ADR 0038 item 4). A feed that has a
/// folder keeps it. A feed with no library folder yet gets its folder at
/// the move into the library. An error goes into the log.
fn give_linked_feed_folder(w: &Workspace, made: &dyn connector::Connector, id: FeedId, path: &str) {
    let result = (|| -> Result<(), String> {
        if w.collection
            .feed(id)
            .map_err(|e| e.to_string())?
            .folder
            .is_some()
            || w.collection
                .setting("library_root")
                .map_err(|e| e.to_string())?
                .is_none()
        {
            return Ok(());
        }
        let library = w.library().map_err(|e| format!("{e:?}"))?;
        let read = made
            .read(path)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("{path} does not exist on the file host"))?;
        library::give_published_folder(&w.collection, &library, id, &read.bytes)
            .map_err(|e| e.to_string())?;
        let doc =
            studio_core::xml::Document::parse_bytes(&read.bytes).map_err(|e| e.to_string())?;
        let hosted = Feed::read(&doc).map_err(|e| e.to_string())?;
        w.record_remote_files(id, &hosted);
        Ok(())
    })();
    if let Err(message) = result {
        eprintln!("studio: feed {} has no release folder yet: {message}", id.0);
    }
}

#[derive(Deserialize)]
pub(crate) struct MspImport {
    backups: Vec<Value>,
}

/// Import MSP 2.0 backups (ADR 0020). Each feed is read-only, and each edit
/// token goes into the vault. The response contains no edit token.
pub(crate) async fn import_msp(
    State(shared): State<Shared>,
    Json(body): Json<MspImport>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        if w.vault.is_none() {
            return Err(match w.vault_state() {
                crate::workspace::VaultState::Absent => ApiError::vault_required(),
                _ => ApiError::vault_locked(),
            });
        }
        let mut added = Vec::new();
        let mut failed = Vec::new();
        for value in &body.backups {
            let backup = match studio_core::msp::MspBackup::from_json(value) {
                Ok(backup) => backup,
                Err(message) => {
                    failed.push(json!({
                        "source": "backup file",
                        "kind": FailKind::BadBackup,
                        "title": null,
                        "url": null,
                        "message": message,
                    }));
                    continue;
                }
            };
            if let Some(vault) = w.vault.as_mut() {
                vault.put(&backup.secret_ref(), backup.edit_token.as_bytes())?;
            }
            let keys = collection_keys(w)?;
            if keys.0.contains(&backup.feed_url) {
                continue;
            }
            match studio_core::discover::read_feed(&w.web, &backup.feed_url) {
                Ok(read) => {
                    if !is_known(&read, &keys) {
                        added.push(add_read_feed(w, &read)?);
                    }
                }
                Err(failure) => {
                    failed.push(failed_json(&backup.feed_url, &failure));
                }
            }
        }
        w.save_vault()?;
        Ok((
            StatusCode::CREATED,
            Json(json!({ "added": added, "failed": failed })),
        ))
    })
    .await
}

// ---- Drafts (ADR 0007, ADR 0022) ----

/// The connector of a feed that Studio can write. A feed with no connector
/// gives the `read_only` error.
fn writer(w: &Workspace, id: FeedId) -> Result<Box<dyn connector::Connector>, ApiError> {
    let row = w.collection.feed(id)?;
    let Some(connector_id) = row.connector_id else {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "read_only",
            format!("feed {} has no connector, so Studio cannot write it", id.0),
        ));
    };
    let connector_row = w.collection.connector(connector_id)?;
    if connector_row.secret_ref.is_some() && w.vault.is_none() {
        return Err(ApiError::vault_locked());
    }
    Ok(w.connector_of(&connector_row)?)
}

/// Start the draft of a feed when it has none (ADR 0041 item 8). A feed
/// with a draft is not changed. A feed with no connector starts its draft
/// from its release folder (ADR 0067 item 3).
fn ensure_draft(w: &Workspace, id: FeedId) -> Result<(), ApiError> {
    if w.collection.draft(id)?.is_none() {
        if w.collection.feed(id)?.connector_id.is_none() {
            let library = w.library()?;
            pipeline::start_public_draft(&w.collection, &library, id)?;
        } else {
            let made = writer(w, id)?;
            pipeline::start_draft(&w.collection, made.as_ref(), id)?;
        }
    }
    Ok(())
}

/// Refuse a link route for a read-only feed or a feed of the wrong medium
/// before a draft starts, so that a refusal starts no draft. The core
/// makes the same checks again (ADR 0041 items 1 and 5).
fn check_mediums(
    w: &Workspace,
    publisher: FeedId,
    release: Option<FeedId>,
) -> Result<(), ApiError> {
    for id in std::iter::once(publisher).chain(release) {
        if w.collection.feed(id)?.connector_id.is_none() {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "read_only",
                format!("feed {} has no connector, so Studio cannot write it", id.0),
            ));
        }
    }
    if w.collection.feed(publisher)?.medium.as_deref() != Some("publisher") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "not_publisher",
            format!("feed {} is not a publisher feed", publisher.0),
        ));
    }
    // A release is an album or a playlist (ADR 0086 item 1).
    if let Some(release) = release
        && !matches!(
            w.collection.feed(release)?.medium.as_deref(),
            Some("music" | "musicL")
        )
    {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "not_release",
            format!("feed {} is not an album or a playlist", release.0),
        ));
    }
    Ok(())
}

/// The draft bytes of a feed before a publisher change, for each feed
/// that the change can alter.
type DraftBytes = Vec<(FeedId, Option<Vec<u8>>)>;

fn draft_bytes(w: &Workspace, id: FeedId) -> Result<Option<Vec<u8>>, ApiError> {
    Ok(w.collection.draft(id)?.map(|draft| draft.draft_xml))
}

fn before_change(w: &Workspace, ids: &[FeedId]) -> Result<DraftBytes, ApiError> {
    ids.iter()
        .map(|&id| Ok((id, draft_bytes(w, id)?)))
        .collect()
}

/// The feeds whose draft a publisher change altered, the publisher feed
/// too, in the sequence of their IDs (ADR 0041 item 8). Each page leaves
/// out its own feed. The title is the title of the feed row.
fn changed_json(w: &Workspace, before: &DraftBytes) -> ApiResult {
    let mut changed = Vec::new();
    for (id, old) in before {
        if draft_bytes(w, *id)? != *old {
            let row = w.collection.feed(*id)?;
            changed.push(json!({ "id": id.0, "title": row.title }));
        }
    }
    Ok(Json(json!({ "changed": changed })))
}

#[derive(Deserialize)]
pub(crate) struct LinkBody {
    release: i64,
}

#[derive(Deserialize)]
pub(crate) struct RoleBody {
    roles: Vec<String>,
}

/// Link a publisher feed to a release. Both drafts change (ADR 0041
/// items 6 to 8).
pub(crate) async fn add_link(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<LinkBody>,
) -> ApiResult {
    blocking(shared, move |w| {
        let publisher_id = FeedId(id);
        let release_id = FeedId(body.release);
        check_mediums(w, publisher_id, Some(release_id))?;
        ensure_draft(w, publisher_id)?;
        ensure_draft(w, release_id)?;
        let before = before_change(w, &[publisher_id, release_id])?;
        publisher::link(&w.collection, publisher_id, release_id)?;
        changed_json(w, &before)
    })
    .await
}

/// Remove the link between a publisher feed and a release from both
/// drafts (ADR 0041 items 7 and 8).
pub(crate) async fn remove_link(
    State(shared): State<Shared>,
    Path((id, release)): Path<(i64, i64)>,
) -> ApiResult {
    blocking(shared, move |w| {
        let publisher_id = FeedId(id);
        let release_id = FeedId(release);
        check_mediums(w, publisher_id, None)?;
        ensure_draft(w, publisher_id)?;
        ensure_draft(w, release_id)?;
        let before = before_change(w, &[publisher_id, release_id])?;
        publisher::unlink(&w.collection, publisher_id, release_id)?;
        changed_json(w, &before)
    })
    .await
}

/// Change and confirm the roles of a publisher feed. The `rel` of each link
/// changes in its draft and in each linked release (ADR 0041 item 11, ADR
/// 0082 items 7 and 8).
pub(crate) async fn set_publisher_role(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<RoleBody>,
) -> ApiResult {
    blocking(shared, move |w| {
        let roles = body
            .roles
            .iter()
            .map(|text| PublisherRole::parse(text))
            .collect::<Option<Vec<_>>>()
            .filter(|roles| !roles.is_empty())
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_feed",
                    "give one or more roles: artist, label, hosting, producer, host, author, network or sponsor",
                )
            })?;
        let publisher_id = FeedId(id);
        check_mediums(w, publisher_id, None)?;
        ensure_draft(w, publisher_id)?;
        let mut releases = publisher::linked_releases(&w.collection, publisher_id)?;
        releases.sort_by_key(|release| release.0);
        for release in &releases {
            ensure_draft(w, *release)?;
        }
        let mut feeds = vec![publisher_id];
        feeds.extend(&releases);
        feeds.sort_by_key(|feed| feed.0);
        let before = before_change(w, &feeds)?;
        publisher::set_roles(&w.collection, publisher_id, &roles)?;
        changed_json(w, &before)
    })
    .await
}

/// The draft as the UI edits it: the model, its revision, `changed`,
/// `bad_addresses`, `bad_keys` and `hidden` (ADR 0065 item 9, ADR 0074). The value of `changed` is
/// true when the draft is different from the live file. Each bad address
/// stops a write (ADR 0023).
///
/// The value of `has_host` is false for a feed with no connector and no
/// draft feed move (ADR 0067 item 6).
///
/// This route needs no host and no library: it only shows the draft.
fn draft_json(w: &Workspace, id: FeedId) -> ApiResult {
    let has_host = w.collection.feed(id)?.connector_id.is_some()
        || w.collection.draft_feed_move(id)?.is_some();
    let (model, revision) = pipeline::draft_model(&w.collection, id)?;
    let changed = pipeline::draft_changed(&w.collection, id)?;
    let bad_addresses = model.bad_addresses();
    let bad_keys = model.bad_keys();
    let hidden = model.hidden();
    let op3_tracks = json!({
        "prefixed": model
            .items
            .iter()
            .filter(|i| {
                i.value
                    .enclosure
                    .as_ref()
                    .and_then(|e| e.url.as_deref())
                    .is_some_and(media::has_op3_prefix)
            })
            .count(),
        "tracks": model.items.len(),
    });
    Ok(Json(json!({
        "revision": revision,
        "op3_tracks": op3_tracks,
        "model": model,
        "changed": changed,
        "bad_addresses": bad_addresses,
        "bad_keys": bad_keys,
        "hidden": hidden,
        "has_host": has_host,
    })))
}

/// The draft of a feed. Studio starts one from the live file when the feed
/// has none. A feed with no connector starts from its release folder.
pub(crate) async fn get_draft(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        draft_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SentDraft {
    revision: String,
    model: Feed,
}

/// Replace the model of the draft (ADR 0022). The hosted file does not
/// change.
pub(crate) async fn put_draft(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SentDraft>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        pipeline::replace_model(&w.collection, id, &body.revision, body.model)?;
        draft_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct NewLiveItem {
    title: String,
    start: String,
}

/// Add a live item to the draft of a music feed (ADR 0054 items 1 and 2).
/// The route changes the draft only. It never writes to a host.
pub(crate) async fn add_live_item(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<NewLiveItem>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let invalid = |message: &str| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_live_item",
                message,
            )
        };
        let title = body.title.trim();
        let start = body.start.trim();
        if title.is_empty() || start.is_empty() {
            return Err(invalid("give the title and the start time of the show"));
        }
        ensure_draft(w, id)?;
        let (mut model, revision) = pipeline::draft_model(&w.collection, id)?;
        // A feed with no medium is an album for the UI, so it can also
        // have a live show.
        if !matches!(model.medium.as_deref(), None | Some("music")) {
            return Err(invalid("only a music feed can have a live show"));
        }
        model
            .live_items
            .push(Entry::new(live::new_live_item(title, start)));
        pipeline::replace_model(&w.collection, id, &revision, model)?;
        draft_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SentLiveStatus {
    status: String,
}

/// Change the status of a live item in the draft (ADR 0054 item 5). The
/// route changes the draft only. It never writes to a host.
pub(crate) async fn set_live_status(
    State(shared): State<Shared>,
    Path((id, guid)): Path<(i64, String)>,
    Json(body): Json<SentLiveStatus>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let bad = |message: String| ApiError::new(StatusCode::CONFLICT, "bad_status", message);
        let next = LiveStatus::parse(&body.status)
            .filter(|s| *s != LiveStatus::Pending)
            .ok_or_else(|| bad("the status is live or ended".to_owned()))?;
        ensure_draft(w, id)?;
        let (mut model, revision) = pipeline::draft_model(&w.collection, id)?;
        let Some(entry) = model
            .live_items
            .iter_mut()
            .find(|e| e.value.item.guid.as_deref() == Some(guid.as_str()))
        else {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                "the draft has no live show with this guid",
            ));
        };
        live::set_status(&mut entry.value, next).map_err(|e| bad(e.to_string()))?;
        pipeline::replace_model(&w.collection, id, &revision, model)?;
        draft_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SentListing {
    hidden: bool,
}

/// Hide the draft from apps, or show it again (ADR 0063 items 3 and 4, ADR
/// 0065 item 7). The route changes the draft only. It never writes to a host.
pub(crate) async fn set_listing(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SentListing>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        let (mut model, revision) = pipeline::draft_model(&w.collection, id)?;
        model.set_hidden(body.hidden);
        pipeline::replace_model(&w.collection, id, &revision, model)?;
        draft_json(w, id)
    })
    .await
}

/// Hide the draft from one platform, or show it again (ADR 0063 item 5, ADR
/// 0065 item 7). The route changes the draft only. It never writes to a host.
pub(crate) async fn set_platform_listing(
    State(shared): State<Shared>,
    Path((id, platform)): Path<(i64, String)>,
    Json(body): Json<SentListing>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        let (mut model, revision) = pipeline::draft_model(&w.collection, id)?;
        if !model.set_platform_hidden(&platform, body.hidden) {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                "the draft has no block for this platform",
            ));
        }
        pipeline::replace_model(&w.collection, id, &revision, model)?;
        draft_json(w, id)
    })
    .await
}

/// Remove the draft, with the rows and the copies of its pending files. The
/// hosted file does not change (ADR 0032 item 12).
pub(crate) async fn delete_draft(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let id = FeedId(id);
        // A new release has no file on the host until its first write. With
        // no draft, Studio cannot start a draft again, so the release keeps
        // its draft (ADR 0032 item 9).
        if let Some(draft) = w.collection.draft(id)?
            && draft.base_version.is_none()
        {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "no_hosted_file",
                "This release has no file on the file host yet. To remove it, remove it from the collection.",
            ));
        }
        pipeline::discard_draft(&w.collection, id)?;
        w.collection.delete_draft(id)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The diff between the live file and the draft, with the ID that the user
/// approves, the pending files that the write puts on the host, and each
/// host operation of the release (ADR 0036 item 14).
///
/// This needs the library, because the diff also gives each host
/// operation of the release. It gives 409 `no_library` when Studio has no
/// library folder yet.
///
/// The route after `POST .../operations` and `DELETE .../operations/{op}`
/// gives this too, so that the UI sees the new diff with no second
/// request.
fn diff_json(w: &Workspace, id: FeedId) -> ApiResult {
    let hosts = w.hosts()?;
    let library = w.library()?;
    let diff = pipeline::diff(&w.collection, &hosts, &library, id)?;
    Ok(Json(json!({
        "id": diff.id.as_str(),
        "unified": diff.unified,
        "changed": diff.changed,
        "files": diff.files,
        "operations": diff.operations,
        "blocked": diff.blocked,
        "feed_move": diff.feed_move,
        "announcements": diff.announcements,
        "announce_skip": diff.announce_skip,
        "grants": diff.grants,
        "events": diff
            .events
            .iter()
            .map(|event| json!({
                "connector_id": event.connector_id,
                "user_signs": event.user_signs,
                "event": event.to_json(),
            }))
            .collect::<Vec<_>>(),
    })))
}

pub(crate) async fn get_diff(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        diff_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Write {
    diff_id: String,
    /// The events that the user signed for the planned events of the
    /// diff (ADR 0050 item 12).
    #[serde(default)]
    signed_events: Vec<Value>,
}

/// Write the draft with the ID of the diff that the user approved. The
/// write also does each host operation of the release (ADR 0007,
/// ADR 0036 items 14 and 15).
///
/// This route needs the library. It gives 409 `no_library` when Studio has
/// no library folder yet. An event in `signed_events` that is not a
/// Nostr event gives 422 `invalid_event`.
pub(crate) async fn write_feed(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<Write>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let signed = body
            .signed_events
            .iter()
            .map(|event| {
                pipeline::signed_event_from_json(&event.to_string()).map_err(|reason| {
                    ApiError::new(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "invalid_event",
                        format!("a signed event is not a Nostr event: {reason}"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let approved = pipeline::DiffId::new(body.diff_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let plan_time = pipeline::plan_time_for_write(&w.collection, id, &approved, now)?;
        let hosts = w.hosts_with(Signing {
            plan_time,
            signed: signed.clone(),
        })?;
        let library = w.library()?;
        // The write deletes the draft. Keep the feed that the write puts
        // on the host, for the remote file records.
        let written = pipeline::draft_model(&w.collection, id).ok();
        let addresses: &dyn pipeline::AddressCheck = match &w.address_check {
            Some(check) => check.0.as_ref(),
            None => &w.web,
        };
        // A feed with no connector writes through its draft feed move. The
        // write reads the public address through `w.web`, so that the
        // network policy applies (ADR 0057, ADR 0067 item 7).
        let public = w.collection.feed(id)?.connector_id.is_none();
        pipeline::write_draft_signed(
            &mut w.collection,
            &hosts,
            &library,
            addresses,
            id,
            &approved,
            &signed,
            now,
            public.then_some(&w.web),
        )?;
        if let Some((feed, _)) = written {
            w.record_remote_files(id, &feed);
        }
        // The write put its announcements into the queue (ADR 0052).
        w.announcer.wake();
        // The write can record a grant, change the secret of the host or move
        // the feed XML. The grant vaults of the feed are made again (ADR 0058
        // items 6 and 7).
        w.sync_grant_vaults(Some(id));
        to_json(w.collection.feed(id)?)
    })
    .await
}

/// The snapshots of a feed, the newest first.
pub(crate) async fn list_snapshots(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer(w, id)?;
        to_json(w.collection.snapshots(id)?)
    })
    .await
}

/// Make a draft from a snapshot, to roll the feed back. The rows and the
/// copies of the pending files of the old draft go out first (ADR 0032
/// item 12). The user then approves the diff of the new draft.
pub(crate) async fn draft_from_snapshot(
    State(shared): State<Shared>,
    Path((id, snapshot)): Path<(i64, i64)>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let made = writer(w, id)?;
        w.collection.snapshot_xml(id, SnapshotId(snapshot))?;
        pipeline::discard_draft(&w.collection, id)?;
        pipeline::draft_from_snapshot(&w.collection, made.as_ref(), id, SnapshotId(snapshot))?;
        draft_json(w, id)
    })
    .await
}

// ---- The library, the host inventory and host operations (ADR 0036) ----

/// The default of each download limit setting, 2 GiB (ADR 0036 item 17).
/// Milestone 7b uses the limits.
const DEFAULT_DOWNLOAD_LIMIT_BYTES: u64 = download::DEFAULT_LIMIT_BYTES;

/// The two settings of the announcements (ADR 0052 item 5).
pub(crate) async fn get_announce_settings(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        to_json(announce::AnnounceSettings::load(&w.collection)?)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct AnnounceSettingsBody {
    pubnotify: bool,
    podping: bool,
}

/// Save the two settings of the announcements (ADR 0052 item 5).
pub(crate) async fn put_announce_settings(
    State(shared): State<Shared>,
    Json(body): Json<AnnounceSettingsBody>,
) -> ApiResult {
    blocking(shared, move |w| {
        let settings = announce::AnnounceSettings {
            pubnotify: body.pubnotify,
            podping: body.podping,
        };
        settings.save(&w.collection)?;
        to_json(settings)
    })
    .await
}

/// The recent announcements of a feed, the newest first (ADR 0052 item 19).
pub(crate) async fn list_announcements(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        to_json(announce::recent(&w.collection, id)?)
    })
    .await
}

/// The library setting and the download limits (ADR 0036 items 1 and 17).
pub(crate) async fn get_library(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let root = w.collection.setting("library_root")?.unwrap_or(Value::Null);
        let default_root = library::default_root().and_then(|p| p.to_str().map(str::to_owned));
        let max = download_limit(&w.collection, "download_max_file_bytes")?;
        let min = download_limit(&w.collection, "download_min_free_bytes")?;
        Ok(Json(json!({
            "root": root,
            "default_root": default_root,
            "download_max_file_bytes": max,
            "download_min_free_bytes": min,
        })))
    })
    .await
}

/// One download limit setting, or the default when the user has not set
/// it yet (ADR 0036 item 17).
fn download_limit(collection: &Collection, key: &str) -> Result<u64, ApiError> {
    match collection.setting(key)? {
        Some(value) => value
            .as_u64()
            .ok_or_else(|| ApiError::internal(format!("the {key} setting is not a number"))),
        None => Ok(DEFAULT_DOWNLOAD_LIMIT_BYTES),
    }
}

#[derive(Deserialize)]
pub(crate) struct SetLibrary {
    root: String,
}

/// A report of a move into the library, as JSON. [`migrate::MoveReport`]
/// has no `Serialize`, because the core gives no JSON (ADR 0002).
fn move_report_json(report: &migrate::MoveReport) -> Value {
    let errors: Vec<Value> = report
        .errors
        .iter()
        .map(|error| json!({ "feed_id": error.feed_id.0, "message": error.message }))
        .collect();
    json!({
        "release_folders": report.release_folders,
        "feed_files": report.feed_files,
        "moved_files": report.moved_files,
        "errors": errors,
    })
}

/// Give Studio its library folder (ADR 0036 item 1). At the first call,
/// Studio also moves the current collection into it (ADR 0035 item 22,
/// ADR 0036 items 1 to 6).
///
/// A later call with a different path gives 409 `library_set`: a move of
/// the library is not in milestone 7a. A later call with the same path
/// repeats the setting, with no second move.
pub(crate) async fn put_library(
    State(shared): State<Shared>,
    Json(body): Json<SetLibrary>,
) -> ApiResult {
    blocking(shared, move |w| {
        let existing = w.collection.setting("library_root")?;
        let existing_root = existing.as_ref().and_then(Value::as_str).map(str::to_owned);
        // The server has one library folder for each user (ADR 0055 item 25).
        if let Some(rules) = &w.rules
            && rules.library_root.to_str() != Some(body.root.trim())
        {
            return Err(not_on_server(
                "the server has one library folder for each user",
            ));
        }
        let root = expand_home(body.root.trim());
        if let Some(existing_root) = &existing_root
            && existing_root != &root
        {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "library_set",
                "Studio has a library folder already; a move of the library is not available in this version",
            ));
        }
        let library = Library::open(&root).map_err(|error| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "library_not_writable",
                error.to_string(),
            )
        })?;
        let report = if existing_root.is_none() {
            let hosts = w.hosts()?;
            migrate::into_library(&w.collection, &hosts, &library, &w.imports_dir)?
        } else {
            migrate::MoveReport::default()
        };
        w.collection.set_setting("library_root", &json!(root))?;
        // Each feed with a release folder now gets its remote file records
        // (ADR 0038 item 3).
        w.record_all_remote_files();
        Ok(Json(move_report_json(&report)))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct LibraryLimits {
    download_max_file_bytes: u64,
    download_min_free_bytes: u64,
}

/// Give Studio its download limits, in bytes (ADR 0036 item 17).
pub(crate) async fn put_library_limits(
    State(shared): State<Shared>,
    Json(body): Json<LibraryLimits>,
) -> ApiResult {
    blocking(shared, move |w| {
        if body.download_max_file_bytes == 0 || body.download_min_free_bytes == 0 {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_limit",
                "each download limit must be a positive number of bytes",
            ));
        }
        w.collection.set_setting(
            "download_max_file_bytes",
            &json!(body.download_max_file_bytes),
        )?;
        w.collection.set_setting(
            "download_min_free_bytes",
            &json!(body.download_min_free_bytes),
        )?;
        Ok(Json(json!({
            "download_max_file_bytes": body.download_max_file_bytes,
            "download_min_free_bytes": body.download_min_free_bytes,
        })))
    })
    .await
}

/// The library file of a feed, checked to belong to it.
fn feed_library_file(
    collection: &Collection,
    id: FeedId,
    file_id: i64,
) -> Result<LibraryFile, ApiError> {
    let file = collection.library_file(LibraryFileId(file_id))?;
    if file.feed_id != id {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("library file {file_id} is not a file of feed {}", id.0),
        ));
    }
    Ok(file)
}

/// The feed that the Files page compares with: the draft, or the hosted
/// feed from the release folder when the feed has no draft, such as a
/// read-only feed (ADR 0036 item 11, ADR 0038 item 5). Gives `None` when
/// the feed has neither.
fn files_feed(w: &Workspace, id: FeedId) -> Result<Option<Feed>, ApiError> {
    match pipeline::draft_model(&w.collection, id) {
        Ok((feed, _)) => Ok(Some(feed)),
        Err(error) if error.kind() == pipeline::PipelineErrorKind::NoDraft => {
            let Ok(library) = w.library() else {
                return Ok(None);
            };
            let row = w.collection.feed(id)?;
            Ok(crate::workspace::library_feed(&library, &row).ok())
        }
        Err(error) => Err(error.into()),
    }
}

/// The addresses that `feed` gives (ADR 0036 items 10 and 12).
fn feed_address_list(feed: Option<&Feed>) -> Vec<String> {
    feed.map(|feed| {
        pipeline::feed_addresses(feed)
            .into_iter()
            .map(std::borrow::Cow::into_owned)
            .collect()
    })
    .unwrap_or_default()
}

/// The length that `feed` gives for each address of an enclosure or of an
/// alternative source (ADR 0038 item 11).
fn feed_lengths(feed: Option<&Feed>) -> std::collections::HashMap<String, u64> {
    let mut lengths = std::collections::HashMap::new();
    let Some(feed) = feed else {
        return lengths;
    };
    for item in &feed.items {
        if let Some(enclosure) = &item.value.enclosure
            && let (Some(url), Some(length)) = (&enclosure.url, &enclosure.length)
            && let Ok(length) = length.trim().parse::<u64>()
        {
            lengths
                .entry(media::plain_address(url).into_owned())
                .or_insert(length);
        }
        // An alternative source has the length of its alternative
        // enclosure, as the remote records do (ADR 0038 item 2).
        for alternate in &item.value.alternate_enclosures {
            let Some(length) = alternate
                .value
                .length
                .as_deref()
                .and_then(|length| length.trim().parse::<u64>().ok())
            else {
                continue;
            };
            for source in &alternate.value.sources {
                if let Some(uri) = &source.value.uri {
                    lengths
                        .entry(media::plain_address(uri).into_owned())
                        .or_insert(length);
                }
            }
        }
    }
    lengths
}

/// True when `url` is one of the addresses that the draft gives (ADR 0036
/// items 10 and 12).
fn is_in_use(url: Option<&str>, addresses: &[String]) -> bool {
    url.is_some_and(|url| addresses.iter().any(|address| address == url))
}

/// The draft operation that waits for the write of `path` on
/// `connector_id`, as `null` when the draft has none (ADR 0036 items 4,
/// 12 and 14).
fn pending_operation(
    operations: &[DraftOperation],
    connector_id: ConnectorId,
    path: &str,
) -> Value {
    operations
        .iter()
        .find(|op| op.connector_id == connector_id && op.path == path)
        .map_or(
            Value::Null,
            |op| json!({ "kind": op.kind.as_str(), "operation_id": op.id.0 }),
        )
}

/// Each host file of a release folder that is not a copy of one of its
/// library files (ADR 0036 item 11). Studio reads the SHA-256 of a host
/// file only when its size agrees with a library file already (ADR 0036
/// item 6).
fn extra_folder_files(
    collection: &Collection,
    hosts: &Hosts,
    folder: &str,
    library_files: &[LibraryFile],
    operations: &[DraftOperation],
    addresses: &[String],
) -> Result<Vec<Value>, ApiError> {
    let prefix = format!("{folder}/");
    // The feed XML of each feed of the library. Studio does not offer to
    // delete it (ADR 0035 item 17).
    let feed_files: Vec<(ConnectorId, String)> = collection
        .feeds()?
        .into_iter()
        .filter_map(|feed| Some((feed.connector_id?, feed.path?)))
        .collect();
    let mut extra = Vec::new();
    for connector_id in hosts.ids() {
        for host_file in collection.host_files(connector_id)? {
            if !host_file.path.starts_with(&prefix) {
                continue;
            }
            let candidates: Vec<&LibraryFile> = library_files
                .iter()
                .filter(|file| file.size == host_file.size)
                .collect();
            let is_copy = if candidates.is_empty() {
                false
            } else {
                let sha256 = match &host_file.sha256 {
                    Some(sha256) => Some(sha256.clone()),
                    None => inventory::host_sha(collection, hosts, connector_id, &host_file.path)?,
                };
                sha256.is_some_and(|sha256| {
                    candidates
                        .iter()
                        .any(|file| file.sha256.as_deref() == Some(sha256.as_str()))
                })
            };
            if !is_copy {
                let connector = hosts.get(connector_id)?;
                let url = connector.public_url(&host_file.path);
                let in_use = is_in_use(url.as_deref(), addresses);
                let pending = pending_operation(operations, connector_id, &host_file.path);
                extra.push(json!({
                    "connector_id": connector_id.0,
                    "path": host_file.path,
                    "size": host_file.size,
                    "url": url,
                    "in_use": in_use,
                    "pending": pending,
                    "pointer": collection.is_feed_pointer(connector_id, &host_file.path)?,
                    "feed_xml": feed_files
                        .iter()
                        .any(|(id, path)| *id == connector_id && *path == host_file.path),
                }));
            }
        }
    }
    Ok(extra)
}

/// The planned path of a library file, or `None` when its feed and the
/// file give no connected host (ADR 0036 item 9, ADR 0038 item 1).
fn planned_or_none(
    collection: &Collection,
    hosts: &Hosts,
    feed_row: &FeedRow,
    file: &LibraryFile,
) -> Result<Option<inventory::PlannedPath>, ApiError> {
    match inventory::planned_path(collection, hosts, feed_row, file) {
        Ok(planned) => Ok(Some(planned)),
        Err(error)
            if matches!(
                error.kind(),
                inventory::InventoryErrorKind::NoHost | inventory::InventoryErrorKind::UnknownHost
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

/// True when the draft uses the address of a remote file, or the address
/// of a copy of it (ADR 0038 item 5).
fn remote_file_in_use(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
    addresses: &[String],
) -> Result<bool, ApiError> {
    let by_copy = inventory::copies(collection, hosts, file)?
        .iter()
        .any(|copy| is_in_use(copy.url.as_deref(), addresses));
    Ok(is_in_use(file.source_url.as_deref(), addresses) || by_copy)
}

/// True when the draft uses a library file: its source address, an
/// address of a copy, or its planned address when it is in the library
/// (ADR 0036 items 10 and 12, ADR 0038 item 5). `has_host` is true when the
/// file has a planned address.
fn file_in_use(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
    has_host: bool,
    addresses: &[String],
) -> Result<bool, ApiError> {
    Ok(remote_file_in_use(collection, hosts, file, addresses)?
        || (has_host && file.local && pipeline::library_file_in_use(collection, hosts, file)?))
}

/// The query of the picker routes: `kind` is `image` or `audio`.
#[derive(Deserialize)]
pub(crate) struct PickerQuery {
    kind: String,
}

impl PickerQuery {
    fn kind(&self) -> Result<media::FileKind, ApiError> {
        media::FileKind::parse(&self.kind).ok_or_else(|| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_kind",
                "the kind of a file is image, audio or lyrics",
            )
        })
    }
}

/// The files of this release for one kind of file field (ADR 0062 items
/// 3 and 6). The list holds the library files of this feed only. It reads
/// no host. A file with no public address has `address` `null`. The
/// library records no duration, so `duration` is `null`.
pub(crate) async fn picker_release(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Query(query): Query<PickerQuery>,
) -> ApiResult {
    let kind = query.kind()?;
    blocking(shared, move |w| {
        let id = FeedId(id);
        let feed_row = w.collection.feed(id)?;
        let hosts = w.hosts()?;
        let compared = files_feed(w, id)?;
        let addresses = feed_address_list(compared.as_ref());
        let mut files = Vec::new();
        for file in w.collection.library_files(id)? {
            if media::kind_of_path(&file.path) != Some(kind) {
                continue;
            }
            let planned = planned_or_none(&w.collection, &hosts, &feed_row, &file)?;
            let planned_url = planned.as_ref().and_then(|p| p.url.clone());
            // The write puts only a file with bytes in the library at its
            // planned address. A remote file stays at its source address
            // (ADR 0062 item 7).
            let address = if file.local {
                planned_url.or_else(|| file.source_url.clone())
            } else {
                file.source_url.clone().or(planned_url)
            };
            let in_use = file_in_use(&w.collection, &hosts, &file, planned.is_some(), &addresses)?;
            files.push(json!({
                "name": file.path,
                "address": address,
                "local": file.local,
                "size": file.size,
                "media_type": remote::media_type_of_name(&file.path)
                    .unwrap_or_else(|| file.media_type.clone()),
                "duration": Value::Null,
                "in_use": in_use,
            }));
        }
        Ok(Json(json!({ "files": files })))
    })
    .await
}

/// The files of one kind on the host of a connector (ADR 0062 items 3, 8
/// and 9). The route lists the host one time and changes nothing on it
/// (ADR 0019 item 7). A file with no public address has `address` `null`
/// and the reason `no_public_address`.
pub(crate) async fn connector_picker(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Query(query): Query<PickerQuery>,
) -> ApiResult {
    let kind = query.kind()?;
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = w.connector_of(&row)?;
        let listed = made
            .list()
            .map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, "host_failed", e.to_string()))?;
        let files: Vec<Value> = listed
            .files
            .iter()
            .filter(|file| media::kind_of_path(&file.path) == Some(kind))
            .map(|file| {
                let address = made.public_url(&file.path);
                let mut value = json!({
                    "path": file.path,
                    "size": file.size,
                    "media_type": remote::media_type_of_name(&file.path),
                    "address": address,
                });
                if address.is_none() {
                    value["reason"] = json!("no_public_address");
                }
                value
            })
            .collect();
        Ok(Json(
            json!({ "files": files, "truncated": listed.truncated }),
        ))
    })
    .await
}

/// The files of a release (ADR 0036 items 4, 7, 9, 10, 11 and 12).
///
/// The result gives the folder, the default host, and each library
/// file with its planned address and its copies. Each copy marks
/// `in_use` true when the draft uses it, and `pending` with the draft
/// operation of its host and path. Each library file lists its pending
/// copy operations. The result also gives each host file of the folder
/// that is not a copy of a library file, with the same two marks.
pub(crate) async fn list_release_files(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let feed_row = w.collection.feed(id)?;
        let hosts = w.hosts()?;
        let library_files = w.collection.library_files(id)?;
        let operations = w.collection.draft_operations(id)?;
        let compared = files_feed(w, id)?;
        let addresses = feed_address_list(compared.as_ref());
        let lengths = feed_lengths(compared.as_ref());

        let mut files = Vec::new();
        let mut remote_count = 0_u64;
        let mut remote_bytes = 0_u64;
        for file in &library_files {
            // A file of a feed with no host, such as a read-only feed, has
            // no planned address. This is also true after its download
            // (ADR 0038 items 1 and 11).
            let planned = planned_or_none(&w.collection, &hosts, &feed_row, file)?;
            // The draft can use the address that the file came from, also
            // after a download, or the address of a copy (ADR 0038 item 5).
            let in_use = file_in_use(&w.collection, &hosts, file, planned.is_some(), &addresses)?;
            if !file.local {
                remote_count += 1;
                remote_bytes += file.size;
            }
            let copies: Vec<Value> = inventory::copies(&w.collection, &hosts, file)?
                .into_iter()
                .map(|copy| {
                    let copy_in_use = is_in_use(copy.url.as_deref(), &addresses);
                    let pending = pending_operation(&operations, copy.connector_id, &copy.path);
                    json!({
                        "connector_id": copy.connector_id.0,
                        "path": copy.path,
                        "url": copy.url,
                        "in_use": copy_in_use,
                        "pending": pending,
                    })
                })
                .collect();
            let pending_copies: Vec<Value> = operations
                .iter()
                .filter(|op| {
                    op.kind == DraftOperationKind::Copy && op.library_file_id == Some(file.id)
                })
                .map(|op| {
                    json!({
                        "connector_id": op.connector_id.0,
                        "path": op.path,
                        "operation_id": op.id.0,
                    })
                })
                .collect();
            files.push(json!({
                "id": file.id.0,
                "path": file.path,
                "size": file.size,
                "media_type": file.media_type,
                "local": file.local,
                "connector_id": planned.as_ref().map(|p| p.connector_id.0),
                "source_url": file.source_url,
                "length_in_feed": file
                    .source_url
                    .as_deref()
                    .and_then(|url| lengths.get(url)),
                // The host that the user selected for this file, or `null`
                // for the default host of the release (ADR 0036 item 8).
                "own_connector_id": file.connector_id.map(|c| c.0),
                "url": planned.as_ref().and_then(|p| p.url.clone()),
                "in_use": in_use,
                "copies": copies,
                "pending_copies": pending_copies,
            }));
        }

        let other = match feed_row.folder.as_deref() {
            Some(folder) => extra_folder_files(
                &w.collection,
                &hosts,
                folder,
                &library_files,
                &operations,
                &addresses,
            )?,
            None => Vec::new(),
        };

        Ok(Json(json!({
            "folder": feed_row.folder,
            "files_connector_id": feed_row.files_connector_id.map(|c| c.0),
            "files": files,
            "remote_count": remote_count,
            "remote_bytes": remote_bytes,
            "other_host_files": other,
            "feed_move": w.collection.draft_feed_move(id)?.map(|m| json!({
                "connector_id": m.connector_id.0,
                "path": m.path,
            })),
        })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SetFeedHost {
    connector_id: Option<i64>,
}

/// Put the feed XML on a different host through the draft (ADR 0038
/// item 16). The host of the feed removes the move. A host that has a file
/// at the path gives 409 `file_exists`. Gives the new diff summary.
///
/// A feed with no connector chooses its first host here, and the body
/// `{ "connector_id": null }` removes the choice (ADR 0067 item 5). A feed
/// with a connector refuses `null` with 422.
pub(crate) async fn set_feed_host(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SetFeedHost>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let has_connector = w.collection.feed(id)?.connector_id.is_some();
        match body.connector_id {
            Some(target) => {
                if has_connector {
                    writer(w, id)?;
                } else {
                    ensure_draft(w, id)?;
                }
                let hosts = w.hosts()?;
                pipeline::set_feed_host(&w.collection, &hosts, id, ConnectorId(target))?;
            }
            None if has_connector => {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_body",
                    "a feed with a file host needs a connector_id",
                ));
            }
            None => {
                ensure_draft(w, id)?;
                pipeline::clear_feed_host(&w.collection, id)?;
            }
        }
        w.sync_grant_vaults(Some(id));
        diff_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SetFilesHost {
    files_connector_id: Option<i64>,
}

/// Give a release its default host for files (ADR 0036 item 4 and item
/// 8).
pub(crate) async fn set_files_host(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SetFilesHost>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer(w, id)?;
        let connector = body.files_connector_id.map(ConnectorId);
        if let Some(connector) = connector {
            w.collection.connector(connector)?;
        }
        w.collection.set_files_connector(id, connector)?;
        to_json(w.collection.feed(id)?)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SetFileHost {
    connector_id: Option<i64>,
}

/// Give one library file its own host, or clear it back to the default
/// host of the release (ADR 0036 item 4 and item 8).
pub(crate) async fn set_file_host(
    State(shared): State<Shared>,
    Path((id, file)): Path<(i64, i64)>,
    Json(body): Json<SetFileHost>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer(w, id)?;
        let file_row = feed_library_file(&w.collection, id, file)?;
        let connector = body.connector_id.map(ConnectorId);
        if let Some(connector) = connector {
            w.collection.connector(connector)?;
        }
        w.collection
            .set_library_file_connector(file_row.id, connector)?;
        to_json(w.collection.library_file(file_row.id)?)
    })
    .await
}

/// The path and the address where Studio would put `file` on
/// `connector_id` (ADR 0036 item 9). The function forces the host for
/// this one computation. It does not change the host of `file` in the
/// collection.
fn planned_path_on(
    collection: &Collection,
    hosts: &Hosts,
    feed_row: &FeedRow,
    file: &LibraryFile,
    connector_id: ConnectorId,
) -> Result<inventory::PlannedPath, ApiError> {
    let mut forced = file.clone();
    forced.connector_id = Some(connector_id);
    Ok(inventory::planned_path(
        collection, hosts, feed_row, &forced,
    )?)
}

/// The address for the draft, for `file` on `connector_id` (ADR 0036
/// item 12). The function gives the address of a copy that is on the
/// host already. It gives the planned address otherwise.
fn reference_url(
    collection: &Collection,
    hosts: &Hosts,
    feed_row: &FeedRow,
    file: &LibraryFile,
    connector_id: ConnectorId,
) -> Result<String, ApiError> {
    let existing = inventory::copies(collection, hosts, file)?
        .into_iter()
        .find(|copy| copy.connector_id == connector_id)
        .and_then(|copy| copy.url);
    if let Some(url) = existing {
        return Ok(url);
    }
    planned_path_on(collection, hosts, feed_row, file, connector_id)?
        .url
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "no_public_address",
                "the file host gives no public address for this file",
            )
        })
}

/// The `reference` operation: put the address of `file` on `target` in
/// the draft, in place of each other address of the same file. A host
/// with no copy of the file gets one with the write, so the new address
/// has a file (ADR 0036 item 12).
fn reference_operation(
    w: &Workspace,
    hosts: &Hosts,
    feed_row: &FeedRow,
    file: i64,
    target: ConnectorId,
    from_url: Option<String>,
) -> Result<(), ApiError> {
    let id = feed_row.id;
    let file = feed_library_file(&w.collection, id, file)?;
    let to_url = reference_url(&w.collection, hosts, feed_row, &file, target)?;
    // Each address of the same file goes to the new address: its planned
    // address and the address of each copy.
    let mut from_urls = file_addresses(&w.collection, hosts, feed_row, &file)?;
    from_urls.extend(from_url);
    from_urls.retain(|url| url != &to_url);
    let on_target = inventory::copies(&w.collection, hosts, &file)?
        .iter()
        .any(|copy| copy.connector_id == target);
    if !on_target {
        let planned = planned_path_on(&w.collection, hosts, feed_row, &file, target)?;
        w.collection.add_draft_operation(
            id,
            DraftOperationKind::Copy,
            Some(file.id),
            target,
            &planned.path,
        )?;
    }
    pipeline::edit_draft(&w.collection, id, |feed| {
        for from in &from_urls {
            reference_feed(feed, from, &to_url);
        }
    })?;
    Ok(())
}

/// Each address of `file`: its planned address on its own host, and the
/// address of each copy (ADR 0036 items 7 and 9).
fn file_addresses(
    collection: &Collection,
    hosts: &Hosts,
    feed_row: &FeedRow,
    file: &LibraryFile,
) -> Result<Vec<String>, ApiError> {
    let mut urls: Vec<String> = inventory::copies(collection, hosts, file)?
        .into_iter()
        .filter_map(|copy| copy.url)
        .collect();
    urls.extend(inventory::planned_path(collection, hosts, feed_row, file)?.url);
    Ok(urls)
}

/// Replace each address of `feed` that is `from` with `to` (ADR 0036
/// item 12). The function checks the same addresses as the host
/// inventory (ADR 0036 item 7): the channel images, each item's
/// enclosure and image, and each `podcast:source` of an alternate
/// enclosure.
fn reference_feed(feed: &mut Feed, from: &str, to: &str) {
    if feed.image.as_deref() == Some(from) {
        feed.image = Some(to.to_owned());
    }
    if let Some(rss_image) = feed.rss_image.as_mut()
        && rss_image.url.as_deref() == Some(from)
    {
        rss_image.url = Some(to.to_owned());
    }
    for entry in &mut feed.items {
        let item = &mut entry.value;
        if item.image.as_deref() == Some(from) {
            item.image = Some(to.to_owned());
        }
        // An enclosure with an OP3 prefix keeps its prefix (ADR 0076
        // item 4).
        if let Some(enclosure) = item.enclosure.as_mut()
            && let Some(url) = enclosure.url.as_deref()
            && media::plain_address(url) == from
        {
            enclosure.url = Some(media::with_plain_address(url, to));
        }
        for alternate in &mut item.alternate_enclosures {
            for source in &mut alternate.value.sources {
                if let Some(uri) = source.value.uri.as_deref()
                    && media::plain_address(uri) == from
                {
                    source.value.uri = Some(media::with_plain_address(uri, to));
                }
            }
        }
    }
}

/// Remove the address `url` from `feed` (ADR 0036 item 12). An enclosure
/// with a `default="true"` alternate enclosure takes the address of that
/// alternate enclosure, so that the item keeps a playable file.
fn dereference_feed(feed: &mut Feed, url: &str) {
    if feed.image.as_deref() == Some(url) {
        feed.image = None;
    }
    // RSS 2.0 requires `url` in `image`, so the full element goes out
    // (ADR 0060 item 5).
    if feed
        .rss_image
        .as_ref()
        .is_some_and(|rss_image| rss_image.url.as_deref() == Some(url))
    {
        feed.rss_image = None;
    }
    for entry in &mut feed.items {
        let item = &mut entry.value;
        if item.image.as_deref() == Some(url) {
            item.image = None;
        }
        if item
            .enclosure
            .as_ref()
            .and_then(|e| e.url.as_deref())
            .is_some_and(|enclosure_url| media::plain_address(enclosure_url) == url)
        {
            let promoted = item
                .alternate_enclosures
                .iter()
                .find(|alternate| alternate.value.default.as_deref() == Some("true"))
                .and_then(|alternate| alternate.value.sources.first())
                .and_then(|source| source.value.uri.clone());
            match (promoted, item.enclosure.as_mut()) {
                (Some(new_url), Some(enclosure)) => enclosure.url = Some(new_url),
                _ => item.enclosure = None,
            }
        }
        for alternate in &mut item.alternate_enclosures {
            alternate.value.sources.retain(|source| {
                source
                    .value
                    .uri
                    .as_deref()
                    .is_none_or(|uri| media::plain_address(uri) != url)
            });
        }
    }
}

/// The body of `POST /feeds/{id}/operations` (ADR 0036 items 10 and 12).
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum FileOperation {
    /// Put the address of the file on a host in the feed, in place of
    /// each other address of the same file. `from_url` is one more address
    /// to replace, for an address that Studio does not know as a copy.
    Reference {
        file: i64,
        connector_id: i64,
        #[serde(default)]
        from_url: Option<String>,
    },
    /// Remove an address from the feed.
    Dereference { url: String },
    /// Put a library file on a different host, with no change of the
    /// feed.
    Copy { file: i64, connector_id: i64 },
    /// Copy a library file to a host, reference the new copy, and delete
    /// the copy that was on the first host.
    Move { file: i64, from: i64, to: i64 },
    /// Delete a file from a host.
    Delete { connector_id: i64, path: String },
    /// Delete each copy of a library file at one of these addresses that
    /// the draft does not use, after a replacement of the audio of a
    /// track (ADR 0040 item 6).
    DeleteUnused { urls: Vec<String> },
}

/// Change the draft or add a draft operation of a release, from one user
/// operation on a file (ADR 0036 items 10 and 12). Gives the new diff
/// summary.
pub(crate) async fn add_operation(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<FileOperation>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer(w, id)?;
        let hosts = w.hosts()?;
        let feed_row = w.collection.feed(id)?;
        match body {
            FileOperation::Reference {
                file,
                connector_id,
                from_url,
            } => {
                reference_operation(
                    w,
                    &hosts,
                    &feed_row,
                    file,
                    ConnectorId(connector_id),
                    from_url,
                )?;
            }
            FileOperation::Dereference { url } => {
                pipeline::edit_draft(&w.collection, id, |feed| {
                    dereference_feed(feed, &url);
                })?;
            }
            FileOperation::Copy { file, connector_id } => {
                let file = feed_library_file(&w.collection, id, file)?;
                let download = check_download(w, &hosts, &file)?;
                let planned = planned_path_on(
                    &w.collection,
                    &hosts,
                    &feed_row,
                    &file,
                    ConnectorId(connector_id),
                )?;
                w.collection.add_draft_operation(
                    id,
                    DraftOperationKind::Copy,
                    Some(file.id),
                    ConnectorId(connector_id),
                    &planned.path,
                )?;
                start_download(w, &file, download)?;
            }
            FileOperation::Move { file, from, to } => {
                let file = feed_library_file(&w.collection, id, file)?;
                let download = check_download(w, &hosts, &file)?;
                let from_connector = ConnectorId(from);
                let to_connector = ConnectorId(to);
                let from_planned =
                    planned_path_on(&w.collection, &hosts, &feed_row, &file, from_connector)?;
                let from_url =
                    reference_url(&w.collection, &hosts, &feed_row, &file, from_connector)?;
                let to_planned =
                    planned_path_on(&w.collection, &hosts, &feed_row, &file, to_connector)?;
                let to_url = reference_url(&w.collection, &hosts, &feed_row, &file, to_connector)?;
                w.collection.add_draft_operation(
                    id,
                    DraftOperationKind::Copy,
                    Some(file.id),
                    to_connector,
                    &to_planned.path,
                )?;
                pipeline::edit_draft(&w.collection, id, |feed| {
                    reference_feed(feed, &from_url, &to_url);
                })?;
                w.collection.add_draft_operation(
                    id,
                    DraftOperationKind::Delete,
                    None,
                    from_connector,
                    &from_planned.path,
                )?;
                start_download(w, &file, download)?;
            }
            FileOperation::Delete { connector_id, path } => {
                w.collection.add_draft_operation(
                    id,
                    DraftOperationKind::Delete,
                    None,
                    ConnectorId(connector_id),
                    &path,
                )?;
            }
            FileOperation::DeleteUnused { urls } => {
                delete_unused_copies(w, &hosts, id, &urls)?;
            }
        }
        diff_json(w, id)
    })
    .await
}

/// The addresses that the other feeds of the collection use, in their
/// drafts and in their live files (ADR 0062 item 18, ADR 0036 item 7). The
/// live file of a feed with a draft is the file at the start of the draft.
/// A feed that Studio cannot read gives no address.
fn other_feeds_addresses(w: &Workspace, id: FeedId) -> Result<Vec<String>, ApiError> {
    let mut addresses = Vec::new();
    for row in w.collection.feeds()? {
        if row.id == id {
            continue;
        }
        let mut models = Vec::new();
        if let Some(draft) = w.collection.draft(row.id)? {
            for bytes in [&draft.draft_xml, &draft.base_xml] {
                if let Ok(doc) = studio_core::xml::Document::parse_bytes(bytes)
                    && let Ok(feed) = Feed::read(&doc)
                {
                    models.push(feed);
                }
            }
        } else if let Some(feed) = files_feed(w, row.id)? {
            models.push(feed);
        }
        for feed in &models {
            addresses.extend(feed_address_list(Some(feed)));
        }
    }
    Ok(addresses)
}

/// Add a deletion for each copy of a library file of the feed whose
/// address is in `urls`, not in the draft and not in another feed of the
/// collection (ADR 0040 item 6, ADR 0062 item 18). A copy that has a
/// deletion in the draft already gets no second one.
fn delete_unused_copies(
    w: &Workspace,
    hosts: &Hosts,
    id: FeedId,
    urls: &[String],
) -> Result<(), ApiError> {
    let addresses = feed_address_list(files_feed(w, id)?.as_ref());
    let others = other_feeds_addresses(w, id)?;
    let unused: Vec<&str> = urls
        .iter()
        .map(String::as_str)
        .filter(|url| !is_in_use(Some(url), &addresses) && !is_in_use(Some(url), &others))
        .collect();
    if unused.is_empty() {
        return Ok(());
    }
    let mut deleted: Vec<(ConnectorId, String)> = w
        .collection
        .draft_operations(id)?
        .into_iter()
        .filter(|op| op.kind == DraftOperationKind::Delete)
        .map(|op| (op.connector_id, op.path))
        .collect();
    for file in w.collection.library_files(id)? {
        for copy in inventory::copies(&w.collection, hosts, &file)? {
            if !copy.url.as_deref().is_some_and(|url| unused.contains(&url)) {
                continue;
            }
            let key = (copy.connector_id, copy.path);
            if deleted.contains(&key) {
                continue;
            }
            w.collection.add_draft_operation(
                id,
                DraftOperationKind::Delete,
                None,
                key.0,
                &key.1,
            )?;
            deleted.push(key);
        }
    }
    Ok(())
}

/// Check that a remote library file can download now: it has a source and
/// it breaks no limit (ADR 0038 items 8 and 10). Gives `false` for a file
/// that is in the library already, which needs no download.
fn check_download(w: &Workspace, hosts: &Hosts, file: &LibraryFile) -> Result<bool, ApiError> {
    if file.local {
        return Ok(false);
    }
    let library = w.library()?;
    if download::source_of(&w.collection, hosts, file)?.is_none() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "no_source",
            format!(
                "{} has no copy on a file host and no address, so Studio cannot download it",
                file.path
            ),
        ));
    }
    download::check_limits(&w.collection, &library, file.size)?;
    Ok(true)
}

/// Record a download and put it in the queue. `needed` is the result of
/// [`check_download`].
fn start_download(w: &Workspace, file: &LibraryFile, needed: bool) -> Result<(), ApiError> {
    if needed {
        w.collection.queue_download(file.id)?;
        w.downloads.add(file.id, file.feed_id, file.size);
    }
    Ok(())
}

// ---- The download queue (ADR 0038 items 6 to 12) ----

/// The progress of one download, as JSON.
fn download_json(
    file: &LibraryFile,
    state: &str,
    bytes: u64,
    total: Option<u64>,
    message: Option<&str>,
) -> Value {
    json!({
        "file_id": file.id,
        "path": file.path,
        "state": state,
        "bytes": bytes,
        "total": total,
        "message": message,
    })
}

/// Each download of a feed with its progress. Downloads that failed
/// before a restart are in the list too.
pub(crate) async fn list_downloads(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        let queue = Arc::clone(&w.downloads);
        let mut listed = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (file_id, entry) in queue.entries() {
            if entry.feed_id != id {
                continue;
            }
            let Ok(file) = w.collection.library_file(LibraryFileId(file_id)) else {
                continue;
            };
            seen.insert(file_id);
            let state = serde_json::to_value(entry.state)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            listed.push(download_json(
                &file,
                &state,
                entry.bytes,
                entry.total,
                entry.message.as_deref(),
            ));
        }
        for row in w.collection.downloads()? {
            if row.state != "failed" || seen.contains(&row.library_file_id.0) {
                continue;
            }
            let Ok(file) = w.collection.library_file(row.library_file_id) else {
                continue;
            };
            if file.feed_id == id {
                listed.push(download_json(
                    &file,
                    "failed",
                    0,
                    None,
                    row.message.as_deref(),
                ));
            }
        }
        Ok(Json(json!({ "downloads": listed })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct StartDownloads {
    #[serde(default)]
    files: Vec<i64>,
    #[serde(default)]
    all: bool,
}

/// Queue the download of some remote files of a feed, or of all of them.
/// A file that is in the library, or in the queue already, does nothing.
/// A file that breaks a limit gives 422 `download_limit`, and then no file
/// goes in the queue (ADR 0038 items 6 and 10).
pub(crate) async fn start_downloads(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<StartDownloads>,
) -> ApiResult {
    let (queue, queued) = blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        let hosts = w.hosts()?;
        let queue = Arc::clone(&w.downloads);
        let mut chosen = Vec::new();
        if body.all {
            chosen = w
                .collection
                .library_files(id)?
                .into_iter()
                .filter(|file| !file.local)
                .collect();
        }
        for file in body.files {
            chosen.push(feed_library_file(&w.collection, id, file)?);
        }
        chosen.retain(|file| {
            !file.local
                && queue
                    .get(file.id)
                    .is_none_or(|e| e.state == crate::downloads::State::Failed)
        });
        chosen.sort_by_key(|file| file.id.0);
        chosen.dedup_by_key(|file| file.id);
        for file in &chosen {
            check_download(w, &hosts, file)?;
        }
        for file in &chosen {
            w.collection.queue_download(file.id)?;
        }
        Ok((queue, chosen))
    })
    .await?;
    let mut ids = Vec::new();
    for file in &queued {
        if queue.add(file.id, file.feed_id, file.size) {
            ids.push(file.id);
        }
    }
    Ok(Json(json!({ "queued": ids })))
}

/// Stop a download and take it out of the queue. The partial file stays,
/// so a later download continues from it (ADR 0038 item 8).
pub(crate) async fn stop_download(
    State(shared): State<Shared>,
    Path((id, file)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    let queue = blocking(shared, move |w| {
        let file = feed_library_file(&w.collection, FeedId(id), file)?;
        w.collection.remove_download(file.id)?;
        Ok((Arc::clone(&w.downloads), file.id))
    })
    .await?;
    queue.0.stop(queue.1);
    Ok(StatusCode::NO_CONTENT)
}

// ---- The folder scan (ADR 0038 items 13 to 15) ----

/// A difference of the folder scan as JSON: the fields of the
/// difference, the title of the feed, and the path of the file.
fn difference_json(w: &Workspace, difference: &folder_scan::Difference) -> Result<Value, ApiError> {
    let mut value =
        serde_json::to_value(difference).map_err(|e| ApiError::internal(e.to_string()))?;
    let feed = w.collection.feed(difference.feed_id)?;
    let title = feed
        .title
        .unwrap_or_else(|| format!("feed {}", difference.feed_id.0));
    if let Some(object) = value.as_object_mut() {
        object.insert("feed_title".to_owned(), json!(title));
        let file = match &difference.kind {
            folder_scan::DifferenceKind::New { .. } => None,
            folder_scan::DifferenceKind::Missing { file }
            | folder_scan::DifferenceKind::Changed { file, .. } => Some(*file),
        };
        if let Some(file) = file {
            let path = w.collection.library_file(file)?.path;
            object.insert("path".to_owned(), json!(path));
        }
    }
    Ok(value)
}

/// Compare each release folder with its records, and give each
/// difference. The scan changes only the stamp of a file whose bytes did
/// not change (ADR 0038 items 13 to 15).
pub(crate) async fn scan_library(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let library = w.library()?;
        let found = folder_scan::scan(&w.collection, &library)?;
        let differences = found
            .iter()
            .map(|difference| difference_json(w, difference))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Json(json!({ "differences": differences })))
    })
    .await
}

/// The body of `POST /feeds/{id}/library-changes`: the decision of the
/// user for one difference (ADR 0038 item 15).
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum LibraryChange {
    /// Add a new file to the library, as a file that the feed does not
    /// use.
    Add { path: String },
    /// Forget a missing file.
    Forget { file: i64 },
    /// Download a missing file again.
    Download { file: i64 },
    /// Use the new bytes of a changed file.
    UseNew { file: i64 },
    /// Put the recorded bytes of a changed file back.
    Restore { file: i64 },
}

/// The error for a difference that the scan does not find any more.
fn no_change() -> ApiError {
    ApiError::new(
        StatusCode::CONFLICT,
        "no_change",
        "The release folder has no such difference now. Scan the library again.",
    )
}

/// True when the scan of a feed gives a missing file with this ID.
fn is_missing(found: &[folder_scan::Difference], id: LibraryFileId) -> bool {
    found
        .iter()
        .any(|d| matches!(&d.kind, folder_scan::DifferenceKind::Missing { file } if *file == id))
}

/// Set the length of each enclosure and of each alternate enclosure that
/// gives `url` as its address (ADR 0035 item 16).
fn set_length_at(feed: &mut Feed, url: &str, length: u64) {
    for entry in &mut feed.items {
        let item = &mut entry.value;
        if let Some(enclosure) = item.enclosure.as_mut()
            && enclosure
                .url
                .as_deref()
                .is_some_and(|enclosure_url| media::plain_address(enclosure_url) == url)
        {
            enclosure.length = Some(length.to_string());
        }
        for alternate in &mut item.alternate_enclosures {
            if alternate.value.sources.iter().any(|source| {
                source
                    .value
                    .uri
                    .as_deref()
                    .is_some_and(|uri| media::plain_address(uri) == url)
            }) {
                alternate.value.length = Some(length.to_string());
            }
        }
    }
}

/// Decide a difference of the folder scan for one feed. Each action scans
/// the feed again first, and gives 409 `no_change` when the difference is
/// not there any more. No action writes to a host. The write of the draft
/// does the uploads and the deletions (ADR 0038 item 15, ADR 0035 item
/// 16).
pub(crate) async fn library_changes(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<LibraryChange>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        let library = w.library()?;
        let found = folder_scan::scan_feed(&w.collection, &library, id)?;
        match body {
            LibraryChange::Add { path } => {
                let exists = found.iter().any(|d| {
                    matches!(&d.kind, folder_scan::DifferenceKind::New { path: p, .. } if p == &path)
                });
                if !exists {
                    return Err(no_change());
                }
                folder_scan::add_new(&w.collection, &library, id, &path)?;
            }
            LibraryChange::Forget { file } => {
                let file = feed_library_file(&w.collection, id, file)?;
                if !is_missing(&found, file.id) {
                    return Err(no_change());
                }
                let hosts = w.hosts()?;
                let known = file.source_url.is_some()
                    || !inventory::copies(&w.collection, &hosts, &file)?.is_empty();
                if known {
                    w.collection.set_library_file_remote(file.id)?;
                } else {
                    w.collection.remove_library_file(file.id)?;
                }
            }
            LibraryChange::Download { file } => {
                let file = feed_library_file(&w.collection, id, file)?;
                if !is_missing(&found, file.id) {
                    return Err(no_change());
                }
                download_again(w, &file)?;
            }
            LibraryChange::Restore { file } => {
                let file = feed_library_file(&w.collection, id, file)?;
                let changed = found.iter().any(|d| {
                    matches!(&d.kind, folder_scan::DifferenceKind::Changed { file: f, .. } if *f == file.id)
                });
                if !changed {
                    return Err(no_change());
                }
                let hosts = w.hosts()?;
                if inventory::copies(&w.collection, &hosts, &file)?.is_empty() {
                    return Err(ApiError::new(
                        StatusCode::CONFLICT,
                        "no_copy",
                        format!(
                            "No connected file host has a copy of the recorded bytes of {}.",
                            file.path
                        ),
                    ));
                }
                download_again(w, &file)?;
            }
            LibraryChange::UseNew { file } => {
                let file = feed_library_file(&w.collection, id, file)?;
                let changed = found.iter().find_map(|d| match &d.kind {
                    folder_scan::DifferenceKind::Changed { file: f, size, sha256 }
                        if *f == file.id =>
                    {
                        Some((*size, sha256.clone()))
                    }
                    _ => None,
                });
                let Some((size, sha256)) = changed else {
                    return Err(no_change());
                };
                use_new_file(w, &library, &file, size, &sha256)?;
            }
        }
        let remaining = folder_scan::scan_feed(&w.collection, &library, id)?;
        let differences = remaining
            .iter()
            .map(|difference| difference_json(w, difference))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Json(json!({ "differences": differences })))
    })
    .await
}

/// Set `local` false for a file and queue its download (ADR 0038 item
/// 15). The checks come first, so a refusal changes nothing.
fn download_again(w: &Workspace, file: &LibraryFile) -> Result<(), ApiError> {
    let hosts = w.hosts()?;
    let mut remote = file.clone();
    remote.local = false;
    check_download(w, &hosts, &remote)?;
    w.collection.set_library_file_remote(file.id)?;
    start_download(w, &remote, true)
}

/// Record the new bytes of a changed file. Each address of a copy of the
/// bytes from before that the draft uses changes to the new planned
/// address, with the new length. Each such copy gets a deletion in the
/// draft (ADR 0038 item 15, ADR 0035 item 16).
fn use_new_file(
    w: &Workspace,
    library: &Library,
    file: &LibraryFile,
    size: u64,
    sha256: &str,
) -> Result<(), ApiError> {
    let id = file.feed_id;
    let hosts = w.hosts()?;
    let feed_row = w.collection.feed(id)?;
    let addresses = feed_address_list(files_feed(w, id)?.as_ref());
    let referred: Vec<inventory::Copy> = inventory::copies(&w.collection, &hosts, file)?
        .into_iter()
        .filter(|copy| is_in_use(copy.url.as_deref(), &addresses))
        .collect();

    // Find the new planned address before any change, so a refusal
    // changes nothing.
    let mut updated = file.clone();
    updated.size = size;
    updated.sha256 = Some(sha256.to_owned());
    let planned = inventory::planned_path(&w.collection, &hosts, &feed_row, &updated)?;
    let to_url = if referred.is_empty() {
        None
    } else {
        writer(w, id)?;
        Some(planned.url.clone().ok_or_else(|| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "no_public_address",
                "the file host gives no public address for this file",
            )
        })?)
    };

    let folder = feed_row.folder.as_deref().unwrap_or_default();
    let stamp = library::file_stamp(&library.path(folder, &file.path));
    if let Some(to_url) = to_url {
        if w.collection.draft(id)?.is_none() {
            let made = writer(w, id)?;
            pipeline::start_draft(&w.collection, made.as_ref(), id)?;
        }
        pipeline::edit_draft(&w.collection, id, |feed| {
            for copy in &referred {
                if let Some(from) = copy.url.as_deref() {
                    reference_feed(feed, from, &to_url);
                }
            }
            set_length_at(feed, &to_url, size);
        })?;
        for copy in &referred {
            let same = copy.connector_id == planned.connector_id && copy.path == planned.path;
            if !same {
                w.collection.add_draft_operation(
                    id,
                    DraftOperationKind::Delete,
                    None,
                    copy.connector_id,
                    &copy.path,
                )?;
            }
        }
    }
    w.collection
        .set_library_file_local(file.id, size, sha256, stamp.as_deref())?;
    Ok(())
}

/// Remove one draft operation of a release, for example to take back a
/// copy or a deletion before the write (ADR 0036 item 12).
pub(crate) async fn remove_operation(
    State(shared): State<Shared>,
    Path((id, op)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer(w, id)?;
        w.collection
            .remove_draft_operation(id, DraftOperationId(op))?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// List a host, and keep the inventory in step with it (ADR 0036 item 6).
pub(crate) async fn scan_host(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let hosts = w.hosts()?;
        to_json(inventory::scan(&w.collection, &hosts, ConnectorId(id))?)
    })
    .await
}

/// The ID of a library file, in the query of the route that removes it.
#[derive(Deserialize)]
pub(crate) struct LibraryFileQuery {
    file: i64,
}

/// Removes its file when it goes out of scope. An upload keeps no
/// temporary file after its result, also after an error (ADR 0026 item 5,
/// ADR 0032 item 10, ADR 0033 item 1).
pub(crate) struct TempFile(pub(crate) PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
/// The file name of an upload: the last part of the name that the browser
/// gives. A file goes only into the folder of its release, so a name with a
/// folder part loses that part. An empty name, `.` and `..` give 422
/// `invalid_name`.
pub(crate) fn upload_file_name(name: &str) -> Result<String, ApiError> {
    let last = name.rsplit(['/', '\\']).next().unwrap_or_default().trim();
    if last.is_empty() || last == "." || last == ".." || last.chars().any(char::is_control) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_name",
            "Give a file name, for example 01 Song.mp3.",
        ));
    }
    Ok(last.to_owned())
}

/// Import a finished upload file into the release folder of a feed (ADR
/// 0026, ADR 0032 items 1 to 5 and 10, ADR 0033 items 1 to 7, ADR 0036
/// items 5 and 9). The file at `temp_path` is in `incoming/` of the data
/// folder. This function removes it, and each file that the import makes
/// next to it, also after an error (ADR 0026 item 5).
///
/// The import runs on a blocking thread with no lock of the workspace,
/// because an MP3 encode takes some seconds and another request must not
/// wait for it (ADR 0033 item 1). The import into the library then runs in
/// `blocking`, with the workspace locked.
pub(crate) async fn import_file(
    shared: Shared,
    id: i64,
    temp_path: PathBuf,
    file_name: String,
    replace: bool,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    // The guard removes the temporary file at the end of this function,
    // also after an error of the import or the collection.
    let guard = TempFile(temp_path.clone());
    // A feed with no host has no host for its files (ADR 0067 item 7). The
    // check comes before the encode, so that Studio does no work for a
    // refused file.
    blocking(shared.clone(), move |w| {
        if w.collection.feed(FeedId(id))?.connector_id.is_none() {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "read_only",
                "choose a file host for this feed first, in Review",
            ));
        }
        Ok(())
    })
    .await?;
    let import_path = temp_path.clone();
    let imported =
        blocking_unlocked(move || media::import(&import_path, &file_name).map_err(ApiError::from))
            .await?;

    // A WAV import or a FLAC import makes a new MP3 file, and a WAV
    // import also makes a new FLAC file, next to the temporary file.
    // Remove each new file at the end of this function too, also after an
    // error (ADR 0032 item 2, ADR 0033 items 1 and 3).
    let _mp3_guard =
        (imported.path.as_path() != temp_path).then(|| TempFile(imported.path.clone()));
    let _lossless_guard = imported
        .lossless
        .as_ref()
        .filter(|file| file.path.as_path() != temp_path)
        .map(|file| TempFile(file.path.clone()));

    blocking(shared, move |w| {
        let _guard = guard;
        import_upload(w, FeedId(id), imported, replace)
    })
    .await
}

/// The blocking steps of an upload, with the workspace locked: the public
/// address check, and the import of the MP3 file and of the FLAC file
/// into the library (ADR 0026, ADR 0032 items 1 to 5 and 10, ADR 0033
/// items 1 to 7, ADR 0034, ADR 0036 items 5, 9 and 10). With `replace`,
/// a different file at the same name gets the hash name (ADR 0040 item 3).
fn import_upload(
    w: &mut Workspace,
    id: FeedId,
    imported: media::Imported,
    replace: bool,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let import = if replace {
        pipeline::import_replacement_into_library
    } else {
        pipeline::import_into_library
    };
    let made = writer(w, id)?;
    let row = w.collection.feed(id)?;
    let feed_path = row
        .path
        .as_deref()
        .ok_or_else(|| ApiError::internal(format!("feed {} has a connector and no path", id.0)))?;
    if made.public_url(feed_path).is_none() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "no_public_address",
            "the folder of this feed has no public address; give the folder one first",
        ));
    }

    let hosts = w.hosts()?;
    let library = w.library()?;
    give_release_folder(w, id, &row, feed_path)?;

    let main_before = library_file_at(w, id, &imported.file_name)?;
    let main = import(
        &w.collection,
        &library,
        &hosts,
        id,
        &imported.path,
        &imported.file_name,
        &imported.media_type,
    )?;

    let lossless = match imported.lossless.as_ref() {
        Some(file) => {
            match import(
                &w.collection,
                &library,
                &hosts,
                id,
                &file.path,
                &file.file_name,
                &file.media_type,
            ) {
                Ok(imported_lossless) => Some(json!({
                    "path": imported_lossless.planned.path,
                    "url": imported_lossless.planned.url,
                    "size": file.size,
                    "media_type": file.media_type,
                    "file_name": file.file_name,
                    "on_host": imported_lossless.planned.on_host,
                    "library_file_id": imported_lossless.library_file_id.0,
                })),
                Err(error) => {
                    // The FLAC file did not go in. The route adds no new
                    // library file when one of the two fails, so a fresh
                    // MP3 row goes out too (ADR 0033 item 3). A row that
                    // was there already, from an earlier import, stays.
                    if main_before.is_none() {
                        undo_library_file(w, &library, main.library_file_id)?;
                    }
                    return Err(error.into());
                }
            }
        }
        None => None,
    };

    let (kind, title, track_number, duration) = match imported.kind {
        media::ImportedKind::Audio {
            title,
            track_number,
            duration,
        } => ("audio", title, track_number, duration),
        media::ImportedKind::Image => ("image", None, None, None),
        media::ImportedKind::Lyrics => ("lyrics", None, None, None),
    };
    // The core makes the alternative enclosures of a lossless file
    // (ADR 0065 item 10).
    // A file with no address gives no alternative enclosures.
    let alternate_enclosures = match (imported.lossless.as_ref(), &main.planned.url) {
        (Some(file), Some(mp3_url)) => {
            let flac_url = lossless.as_ref().and_then(|facts| facts["url"].as_str());
            flac_url
                .map(|flac_url| {
                    studio_core::feed::lossless_pair(mp3_url, imported.size, flac_url, file.size)
                })
                .unwrap_or_default()
        }
        _ => Vec::new(),
    };
    Ok((
        StatusCode::CREATED,
        to_json(json!({
            "path": main.planned.path,
            "url": main.planned.url,
            "size": imported.size,
            "media_type": imported.media_type,
            "kind": kind,
            "title": title,
            "track_number": track_number,
            "duration": duration,
            "converted_from_wav": imported.converted_from_wav,
            "file_name": imported.file_name,
            "on_host": main.planned.on_host,
            "library_file_id": main.library_file_id.0,
            "lossless": lossless,
            "alternate_enclosures": alternate_enclosures,
        }))?,
    ))
}

/// The library file of this feed at `file_name` already, before an
/// import. [`import_upload`] uses this to know whether the import made a
/// fresh row, or reused one that was there already (ADR 0036 item 5).
fn library_file_at(
    w: &Workspace,
    id: FeedId,
    file_name: &str,
) -> Result<Option<LibraryFileId>, ApiError> {
    Ok(w.collection
        .library_files(id)?
        .into_iter()
        .find(|file| file.path == file_name)
        .map(|file| file.id))
}

/// Give a feed its release folder in the library, when it has none yet
/// (ADR 0036 items 1, 2 and 4). A feed with a folder already keeps it.
fn give_release_folder(
    w: &mut Workspace,
    id: FeedId,
    row: &FeedRow,
    feed_path: &str,
) -> Result<(), ApiError> {
    if row.folder.is_some() {
        return Ok(());
    }
    let taken: std::collections::HashSet<String> = w
        .collection
        .feeds()?
        .into_iter()
        .filter_map(|feed| feed.folder)
        .collect();
    let folder = library::release_folder_name(feed_path, &|name| taken.contains(name));
    w.collection.set_feed_folder(id, &folder)?;
    Ok(())
}

/// Remove a library file row that an import added in this request, with
/// its bytes in the release folder (ADR 0033 item 3).
fn undo_library_file(
    w: &mut Workspace,
    library: &Library,
    file_id: LibraryFileId,
) -> Result<(), ApiError> {
    let file = w.collection.library_file(file_id)?;
    if let Some(folder) = w.collection.feed(file.feed_id)?.folder.as_deref() {
        library
            .remove_file(folder, &file.path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }
    w.collection.remove_library_file(file_id)?;
    Ok(())
}

/// Remove a library file of a feed, with its bytes in the release folder
/// (ADR 0036 items 4, 5 and 11). Gives 409 `file_in_use` when the draft
/// refers to the file, so Studio removes no file that a track or an
/// image still uses.
pub(crate) async fn delete_pending_file(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Query(query): Query<LibraryFileQuery>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let file_id = LibraryFileId(query.file);
        let file = w.collection.library_file(file_id)?;
        if file.feed_id != id {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("library file {} is not a file of feed {}", query.file, id.0),
            ));
        }
        let hosts = w.hosts()?;
        if pipeline::library_file_in_use(&w.collection, &hosts, &file)? {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "file_in_use",
                "the draft refers to this file; remove the reference first",
            ));
        }
        let library = w.library()?;
        if let Some(folder) = w.collection.feed(id)?.folder.as_deref() {
            library
                .remove_file(folder, &file.path)
                .map_err(|e| ApiError::internal(e.to_string()))?;
        }
        w.collection.remove_library_file(file_id)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Media {
    url: String,
}

/// The size and the media type of an audio file, from its host
/// (ADR 0023 item 5). Studio does not read the file.
pub(crate) async fn media_facts(
    State(shared): State<Shared>,
    Json(body): Json<Media>,
) -> ApiResult {
    // The read uses the address with no OP3 prefix, so that it is not a
    // download for OP3 (ADR 0076 item 3).
    blocking(shared, move |w| {
        to_json(w.web.media_facts(&media::plain_address(&body.url))?)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct ImageQuery {
    url: String,
}

/// The image at an address, from the cache or from its host (ADR 0025,
/// ADR 0032 items 13 and 14). The UI cannot put the access token on an
/// `<img>` request, so it gets the image through this route instead.
pub(crate) async fn get_image(
    State(shared): State<Shared>,
    Query(query): Query<ImageQuery>,
) -> Result<Response, ApiError> {
    // The lock is held only to read and to fill the cache. The read from
    // the host of the image occurs with no lock, so that a slow host does
    // not stop the other requests, such as a save of the draft.
    let url = query.url;
    let (cached, pending, web) = {
        let w = lock(&shared)?;
        let cached = w.images.get(&url).cloned();
        let pending = if cached.is_none() {
            pending_image(&w, &url)
        } else {
            None
        };
        (cached, pending, w.web.clone())
    };
    let image = if let Some(image) = cached {
        image
    } else if let Some(image) = pending {
        image
    } else {
        let read_url = url.clone();
        let image = tokio::task::spawn_blocking(move || web.get_image(&read_url))
            .await
            .map_err(|e| ApiError::internal(e.to_string()))??;
        lock(&shared)?.images.put(url, image.clone());
        image
    };
    Ok((
        [
            (header::CONTENT_TYPE, image.media_type),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
        image.bytes,
    )
        .into_response())
}

/// The image at `url` from the release folder, when `url` is the planned
/// address of a library image that is not on its host yet. The preview of
/// an imported image then shows before the write (ADR 0025 item 2, ADR
/// 0036 items 5 and 9). Each failure gives `None`, so the route then
/// reads the address from the web.
fn pending_image(w: &Workspace, url: &str) -> Option<studio_core::web::WebImage> {
    let (path, media_type) = pending_library_file(w, url, "image/")?;
    let bytes = std::fs::read(path).ok()?;
    Some(studio_core::web::WebImage { bytes, media_type })
}

/// The path and the media type of the library file whose planned address is
/// `url`, when its bytes are in the release folder and not on the host yet.
/// The media type starts with `media_prefix`. Each failure gives `None`.
pub(crate) fn pending_library_file(
    w: &Workspace,
    url: &str,
    media_prefix: &str,
) -> Option<(PathBuf, String)> {
    let library = w.library().ok()?;
    let hosts = w.hosts().ok()?;
    for feed in w.collection.feeds().ok()? {
        let Some(folder) = feed.folder.as_deref() else {
            continue;
        };
        let Ok(files) = w.collection.library_files(feed.id) else {
            continue;
        };
        for file in files {
            if !file.local || !file.media_type.starts_with(media_prefix) {
                continue;
            }
            let Ok(planned) = inventory::planned_path(&w.collection, &hosts, &feed, &file) else {
                continue;
            };
            if planned.on_host || planned.url.as_deref() != Some(url) {
                continue;
            }
            return Some((library.path(folder, &file.path), file.media_type));
        }
    }
    None
}

// ---- Recipients (ADR 0027) ----

/// The key of a recipient as a tuple, with empty text for a missing custom
/// key or custom value.
type KeyParts = (String, String, String);

fn key_parts(key: &RecipientKey) -> KeyParts {
    let (address, custom_key, custom_value) = key.parts();
    (
        address.to_owned(),
        custom_key.to_owned(),
        custom_value.to_owned(),
    )
}

/// The stored keysend lookups, by lowercase address.
fn keysend_map(w: &Workspace) -> Result<HashMap<String, Lookup>, ApiError> {
    Ok(w.collection
        .keysend_lookups()?
        .into_iter()
        .map(|lookup| (lookup.address.clone(), lookup))
        .collect())
}

/// Each recipient that Studio knows: from a feed of the collection, or
/// from a save by the user. The list has one entry for each wallet
/// (ADR 0095 items 1 to 3).
pub(crate) async fn list_recipients(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let mut known = w.collection.known_recipients()?;
        // The count of feeds comes from the model of each feed, as in
        // `recipient_uses`. A recipient that only a draft has is added.
        let mut counts: HashMap<KeyParts, UsedRecipient> = HashMap::new();
        for (row, feed) in feed_models(w)? {
            for recipient in recipient_places(&feed).all() {
                let Some(address) = recipient.address.as_deref().filter(|a| !a.is_empty()) else {
                    continue;
                };
                let identity = (
                    address.to_owned(),
                    recipient.custom_key.clone().unwrap_or_default(),
                    recipient.custom_value.clone().unwrap_or_default(),
                );
                let used = counts.entry(identity).or_insert_with(|| UsedRecipient {
                    name: recipient.name.clone(),
                    recipient_type: recipient.recipient_type.clone(),
                    feeds: std::collections::HashSet::new(),
                });
                used.feeds.insert(row.id);
            }
        }
        let mut feed_sets: HashMap<KeyParts, std::collections::HashSet<FeedId>> = counts
            .iter()
            .map(|(identity, used)| (identity.clone(), used.feeds.clone()))
            .collect();
        for entry in &mut known {
            let identity = (
                entry.address.clone(),
                entry.custom_key.clone().unwrap_or_default(),
                entry.custom_value.clone().unwrap_or_default(),
            );
            entry.feeds = counts.remove(&identity).map_or(0, |used| used.feeds.len());
        }
        for ((address, custom_key, custom_value), used) in counts {
            known.push(studio_core::collection::KnownRecipient {
                name: used.name,
                recipient_type: used.recipient_type,
                address,
                custom_key: Some(custom_key).filter(|k| !k.is_empty()),
                custom_value: Some(custom_value).filter(|v| !v.is_empty()),
                feeds: used.feeds.len(),
                saved_id: None,
            });
        }
        if let Some(scheme) = w.keysend_scheme {
            let addresses: Vec<String> = known.iter().map(|k| k.address.clone()).collect();
            keysend::refresh(&w.collection, &w.web, scheme, &addresses, now_seconds())?;
        }
        let lookups = keysend_map(w)?;
        to_json(group_wallets(known, &mut feed_sets, &lookups))
    })
    .await
}

/// Group the recipients by wallet (ADR 0095 items 2 and 3). The list is by
/// name, then by address.
fn group_wallets(
    known: Vec<studio_core::collection::KnownRecipient>,
    feed_sets: &mut HashMap<KeyParts, std::collections::HashSet<FeedId>>,
    lookups: &HashMap<String, Lookup>,
) -> Vec<Value> {
    let mut groups: HashMap<
        KeyParts,
        (RecipientKey, Vec<studio_core::collection::KnownRecipient>),
    > = HashMap::new();
    for entry in known {
        let key = RecipientKey {
            address: entry.address.clone(),
            custom_key: entry.custom_key.clone(),
            custom_value: entry.custom_value.clone(),
        };
        let wallet = keysend::wallet_key(&key, lookups);
        groups
            .entry(key_parts(&wallet))
            .or_insert_with(|| (wallet, Vec::new()))
            .1
            .push(entry);
    }
    let mut wallets: Vec<(String, String, Value)> = Vec::new();
    for (parts, (wallet, mut members)) in groups {
        // The key first, then A to Z by address.
        members.sort_by(|a, b| {
            let is_key = |m: &studio_core::collection::KnownRecipient| {
                (
                    m.address.clone(),
                    m.custom_key.clone().unwrap_or_default(),
                    m.custom_value.clone().unwrap_or_default(),
                ) == parts
            };
            is_key(b)
                .cmp(&is_key(a))
                .then_with(|| a.address.cmp(&b.address))
        });
        let mut feeds = std::collections::HashSet::new();
        for member in &members {
            let identity = (
                member.address.clone(),
                member.custom_key.clone().unwrap_or_default(),
                member.custom_value.clone().unwrap_or_default(),
            );
            if let Some(set) = feed_sets.get(&identity) {
                feeds.extend(set.iter().copied());
            }
        }
        let first = |pick: fn(&studio_core::collection::KnownRecipient) -> Option<String>| {
            members.iter().find_map(pick)
        };
        let name = first(|m| m.name.clone().filter(|n| !n.is_empty()));
        let recipient_type = first(|m| m.recipient_type.clone().filter(|t| !t.is_empty()));
        let saved_id = members.iter().find_map(|m| m.saved_id);
        let addresses: Vec<Value> = members
            .iter()
            .map(|m| {
                json!({
                    "address": m.address,
                    "custom_key": m.custom_key,
                    "custom_value": m.custom_value,
                    "recipient_type": m.recipient_type,
                    "name": m.name,
                    "feeds": m.feeds,
                    "saved_id": m.saved_id,
                    "lookup": lookups
                        .get(&m.address.trim().to_lowercase())
                        .map(|l| l.status.as_str()),
                })
            })
            .collect();
        let sort_name = name.clone().unwrap_or_default().to_lowercase();
        wallets.push((
            sort_name,
            wallet.address.clone(),
            json!({
                "name": name,
                "recipient_type": recipient_type,
                "address": wallet.address,
                "custom_key": wallet.custom_key,
                "custom_value": wallet.custom_value,
                "feeds": feeds.len(),
                "saved_id": saved_id,
                "addresses": addresses,
            }),
        ));
    }
    wallets.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    wallets.into_iter().map(|(_, _, value)| value).collect()
}

/// The time of the computer in seconds, for the keysend lookups.
fn now_seconds() -> i64 {
    i64::try_from(unix_now()).unwrap_or(i64::MAX)
}

/// A recipient that a feed model has, with the feeds that use it.
struct UsedRecipient {
    name: Option<String>,
    recipient_type: Option<String>,
    feeds: std::collections::HashSet<FeedId>,
}

/// The feeds of the collection with their models: the draft, or else the
/// library copy. A feed with no model is not in the list (ADR 0091 item 2).
fn feed_models(w: &Workspace) -> Result<Vec<(studio_core::collection::FeedRow, Feed)>, ApiError> {
    let mut models = Vec::new();
    for row in w.collection.feeds()? {
        if let Some(feed) = files_feed(w, row.id).ok().flatten() {
            models.push((row, feed));
        }
    }
    Ok(models)
}

/// The recipients of the value blocks of one feed model, by place: the
/// channel, each track and each live item. A recipient of a value time
/// split is not in the list (ADR 0091 item 10).
struct RecipientPlaces<'a> {
    channel: Vec<&'a Recipient>,
    items: Vec<Vec<&'a Recipient>>,
    live_items: Vec<Vec<&'a Recipient>>,
}

impl<'a> RecipientPlaces<'a> {
    /// Each recipient of each place.
    fn all(&self) -> impl Iterator<Item = &'a Recipient> + '_ {
        self.channel
            .iter()
            .chain(self.items.iter().flatten())
            .chain(self.live_items.iter().flatten())
            .copied()
    }
}

fn recipient_places(feed: &Feed) -> RecipientPlaces<'_> {
    fn of_blocks(blocks: &[Entry<studio_core::feed::ValueBlock>]) -> Vec<&Recipient> {
        blocks
            .iter()
            .flat_map(|block| block.value.recipients.iter().map(|r| &r.value))
            .collect()
    }
    RecipientPlaces {
        channel: of_blocks(&feed.values),
        items: feed
            .items
            .iter()
            .map(|item| of_blocks(&item.value.values))
            .collect(),
        live_items: feed
            .live_items
            .iter()
            .map(|live| of_blocks(&live.value.item.values))
            .collect(),
    }
}

/// Save a recipient, in the field names of the UI model (ADR 0027 item 5).
/// Studio does not keep the split and the fee of a saved recipient.
pub(crate) async fn save_recipient(
    State(shared): State<Shared>,
    Json(body): Json<Recipient>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let id = w.collection.save_recipient(&body)?;
        Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
    })
    .await
}

/// Remove a saved recipient. A recipient that a feed of the collection has
/// stays in the list.
pub(crate) async fn remove_saved_recipient(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        w.collection.remove_saved_recipient(id)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

// ---- Change sets (ADR 0091) ----

/// The feeds that use a recipient: the channel, the tracks and the live
/// items. A recipient of a value time split does not count (ADR 0091
/// item 10).
pub(crate) async fn recipient_uses(
    State(shared): State<Shared>,
    Json(key): Json<RecipientKey>,
) -> ApiResult {
    blocking(shared, move |w| {
        let (key, members) = keysend::wallet_of(&key, &keysend_map(w)?);
        let wallet = Wallet { key, members };
        let mut rows = Vec::new();
        for (row, feed) in feed_models(w)? {
            let places = recipient_places(&feed);
            let uses = |list: &Vec<&Recipient>| list.iter().any(|r| wallet.matches(r));
            let channel = uses(&places.channel);
            let items = places.items.iter().filter(|list| uses(list)).count();
            let live_items = places.live_items.iter().filter(|list| uses(list)).count();
            if !channel && items == 0 && live_items == 0 {
                continue;
            }
            rows.push(json!({
                "feed_id": row.id.0,
                "title": feed.title.clone().or(row.title.clone()),
                "medium": row.medium,
                "channel": channel,
                "items": items,
                "live_items": live_items,
                "writable": row.connector_id.is_some(),
            }));
        }
        Ok(Json(json!({ "recipient": wallet.key, "feeds": rows })))
    })
    .await
}

/// The feeds of the level of a publisher feed: the feed, each feed below
/// it and the releases of these feeds (ADR 0091 item 9). A feed that is
/// not a publisher feed gives 422 `not_publisher`.
pub(crate) async fn feed_level(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        if w.collection.feed(id)?.medium.as_deref() != Some("publisher") {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "not_publisher",
                format!("feed {} is not a publisher feed", id.0),
            ));
        }
        let mut feeds = Vec::new();
        for row in w.collection.feeds()? {
            let model = files_feed(w, row.id).ok().flatten();
            feeds.push((row, model));
        }
        let catalogs: std::collections::HashMap<FeedId, publisher::Catalog> = feeds
            .iter()
            .map(|(row, _)| row.id)
            .zip(publisher::catalogs(&feeds))
            .collect();
        let ids: Vec<i64> = change_set::level_feeds(&catalogs, id)
            .into_iter()
            .map(|feed| feed.0)
            .collect();
        Ok(Json(json!({ "feed_ids": ids })))
    })
    .await
}

/// Why Studio cannot write a changed feed now (ADR 0091 item 22).
fn change_blocked(w: &Workspace, id: FeedId) -> Option<&'static str> {
    let row = w.collection.feed(id).ok()?;
    let Some(connector_id) = row.connector_id else {
        return Some("read_only_no_host");
    };
    let connector = w.collection.connector(connector_id).ok()?;
    (connector.secret_ref.is_some() && w.vault.is_none()).then_some("vault_locked")
}

/// The JSON of a change set: the operation, the recipient and each feed
/// with its outcome. `blocked` is the cause why Studio cannot write a
/// changed feed now, or null.
fn change_set_json(w: &Workspace, set: &change_set::ChangeSet) -> Value {
    let feeds: Vec<Value> = set
        .feeds
        .iter()
        .map(|feed| {
            let row = w.collection.feed(feed.feed_id).ok();
            let blocked = if feed.outcome.is_changed() && feed.written_at.is_none() {
                change_blocked(w, feed.feed_id)
            } else {
                None
            };
            json!({
                "feed_id": feed.feed_id.0,
                "title": row.as_ref().and_then(|r| r.title.clone()),
                "medium": row.as_ref().and_then(|r| r.medium.clone()),
                "outcome": feed.outcome,
                "had_draft": feed.had_draft,
                "written_at": feed.written_at,
                "blocked": blocked,
            })
        })
        .collect();
    json!({
        "id": set.id,
        "made_at": set.made_at,
        "recipient": set.recipient,
        "operation": set.operation,
        "open": set.is_open(),
        "feeds": feeds,
    })
}

#[derive(Deserialize)]
pub(crate) struct NewChangeSet {
    recipient: RecipientKey,
    operation: RecipientOperation,
    feed_ids: Vec<i64>,
}

/// Make a change set. The core puts the change in the draft of each feed
/// of the scope. This route writes nothing to a file host (ADR 0091
/// items 17 to 19). The core also changes the saved recipient for a change
/// of the address or of the name (item 12).
pub(crate) async fn make_change_set(
    State(shared): State<Shared>,
    Json(body): Json<NewChangeSet>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let invalid = |message: &str| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_change_set",
                message,
            )
        };
        if body.feed_ids.is_empty() {
            return Err(invalid("a change set needs at least one feed"));
        }
        let new_address = match &body.operation {
            RecipientOperation::ChangeAddress { address, .. }
            | RecipientOperation::Add { address, .. } => Some(address),
            _ => None,
        };
        if new_address.is_some_and(|a| a.trim().is_empty()) {
            return Err(invalid("the new recipient needs an address"));
        }
        if body.recipient.address.trim().is_empty() {
            return Err(invalid("the recipient needs an address"));
        }
        // The core refuses an operation with no type too (ADR 0095 item 14).
        // The core error has no code for the route, so the route checks the
        // type first to give 422.
        if let RecipientOperation::ChangeAddress { recipient_type, .. }
        | RecipientOperation::Add { recipient_type, .. } = &body.operation
            && recipient_type
                .as_deref()
                .is_none_or(|t| t.trim().is_empty())
        {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_recipient",
                "the recipient needs a type",
            ));
        }
        let feed_ids: Vec<FeedId> = body.feed_ids.iter().map(|&id| FeedId(id)).collect();
        for &id in &feed_ids {
            w.collection.feed(id)?;
        }
        let mut lookups = keysend_map(w)?;
        if let RecipientOperation::UseLnaddress { address } = &body.operation {
            check_lnaddress(w, &body.recipient, address, &mut lookups)?;
        }
        let (key, members) = keysend::wallet_of(&body.recipient, &lookups);
        let wallet = Wallet { key, members };
        let set = {
            let workspace: &Workspace = w;
            change_set::make_change_set(
                &workspace.collection,
                &mut |id| ensure_draft(workspace, id).map_err(|e| e.coded()),
                &wallet,
                &body.operation,
                &feed_ids,
            )?
        };
        Ok((StatusCode::CREATED, Json(change_set_json(w, &set))))
    })
    .await
}

/// Check the Lightning address of a Use Lightning address operation before
/// the core runs (ADR 0095 items 11 to 13). `lookups` gets the new lookup.
fn check_lnaddress(
    w: &Workspace,
    recipient: &RecipientKey,
    address: &str,
    lookups: &mut HashMap<String, Lookup>,
) -> Result<(), ApiError> {
    let refused = |code: &'static str, message: String| {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    };
    let address = address.trim().to_lowercase();
    if keysend::lookup_url(w.keysend_scheme.unwrap_or("https"), &address).is_none() {
        return Err(refused(
            "invalid_recipient",
            "the Lightning address needs the form name@domain".to_owned(),
        ));
    }
    let wallet = keysend::wallet_key(recipient, lookups);
    if wallet.address.contains('@') {
        return Err(refused(
            "invalid_recipient",
            "the recipient has no node key".to_owned(),
        ));
    }
    if let Some(scheme) = w.keysend_scheme
        && keysend::is_due(lookups.get(&address), now_seconds())
    {
        let fresh = keysend::refresh(
            &w.collection,
            &w.web,
            scheme,
            std::slice::from_ref(&address),
            now_seconds(),
        )?;
        lookups.extend(fresh);
    }
    let found = lookups
        .get(&address)
        .filter(|lookup| lookup.status == LookupStatus::Found)
        .and_then(Lookup::key);
    let Some(found) = found else {
        let domain = address.split_once('@').map_or("", |(_, d)| d);
        return Err(refused(
            "lnaddress_no_keysend",
            format!("Studio found no keysend data at {domain}"),
        ));
    };
    if key_parts(&found) != key_parts(&wallet) {
        return Err(refused(
            "lnaddress_mismatch",
            format!("the Lightning address {address} pays a different wallet"),
        ));
    }
    Ok(())
}

/// The open change sets, the oldest first.
pub(crate) async fn list_change_sets(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let sets: Vec<Value> = change_set::change_sets(&w.collection)?
            .iter()
            .map(|set| change_set_json(w, set))
            .collect();
        Ok(Json(json!(sets)))
    })
    .await
}

/// A change set that exists, or 404.
fn existing_change_set(w: &Workspace, id: i64) -> Result<change_set::ChangeSet, ApiError> {
    if w.collection.change_set_row(id)?.is_none() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("change set {id} does not exist"),
        ));
    }
    Ok(change_set::find_change_set(&w.collection, id)?)
}

/// One change set, open or not.
pub(crate) async fn get_change_set(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let set = existing_change_set(w, id)?;
        Ok(Json(change_set_json(w, &set)))
    })
    .await
}

/// Discard a change set: the draft of each unwritten feed goes back to its
/// state from before the change (ADR 0091 item 20).
pub(crate) async fn discard_change_set(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        existing_change_set(w, id)?;
        change_set::discard_change_set(&w.collection, id)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The error for a GUID that musicindex does not know.
fn index_not_found(entity: &str, guid: &str) -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        format!("musicindex has no {entity} {guid}"),
    )
}

/// The title, the payment routes and the tracks of a feed in musicindex
/// (ADR 0027 item 6).
pub(crate) async fn index_feed_recipients(
    State(shared): State<Shared>,
    Path(guid): Path<String>,
) -> ApiResult {
    blocking(shared, move |w| {
        let routes = w.index.feed_routes(&guid)?;
        to_json(routes.ok_or_else(|| index_not_found("feed", &guid))?)
    })
    .await
}

/// The title and the payment routes of one track in musicindex (ADR 0027
/// item 6). The result has no tracks.
pub(crate) async fn index_track_recipients(
    State(shared): State<Shared>,
    Path((guid, track)): Path<(String, String)>,
) -> ApiResult {
    blocking(shared, move |w| {
        let routes = w.index.track_routes(&guid, &track)?;
        to_json(routes.ok_or_else(|| index_not_found("track", &track))?)
    })
    .await
}

/// The tracks of the music feeds of the collection, for the track picker of
/// a playlist (ADR 0053 item 5).
pub(crate) async fn playlist_tracks(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| to_json(playlist::own_tracks(&w.collection)?)).await
}

/// The address and the tracks of a feed in musicindex, for the track picker
/// of a playlist (ADR 0053 item 5).
pub(crate) async fn index_feed_tracks(
    State(shared): State<Shared>,
    Path(guid): Path<String>,
) -> ApiResult {
    blocking(shared, move |w| {
        let Some(routes) = w.index.feed_routes(&guid)? else {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_in_index",
                format!("musicindex has no feed {guid}"),
            ));
        };
        let feed_url = w.index.feed_url(&guid)?;
        Ok(Json(json!({
            "feed_guid": guid,
            "feed_url": feed_url,
            "title": routes.title,
            "tracks": routes.tracks,
        })))
    })
    .await
}

/// The connected apps. The list has no token (ADR 0054 item 7).
pub(crate) async fn list_apps(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| to_json(apps::apps(&w.collection)?)).await
}

#[derive(Deserialize)]
pub(crate) struct AppBody {
    name: String,
}

/// Add a connected app. This response is the only one that has the token.
pub(crate) async fn add_app(
    State(shared): State<Shared>,
    Json(body): Json<AppBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let (app, token) =
            apps::add_app(&w.collection, &body.name).map_err(|e| match e.kind() {
                CollectionErrorKind::Invalid => ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_name",
                    e.to_string(),
                ),
                _ => ApiError::from(e),
            })?;
        w.app_changed(&crate::workspace::AppChange::Added {
            app_id: app.id,
            token_hash: apps::token_hash(&token),
        });
        // Each grant vault gets the wrapper of the new app now, or at the
        // next unlock (ADR 0058 item 8).
        w.sync_grant_vaults(None);
        Ok((
            StatusCode::CREATED,
            Json(json!({
                "id": app.id,
                "name": app.name,
                "created_at": app.created_at,
                "has_key": app.has_key,
                "token": token,
            })),
        ))
    })
    .await
}

/// Remove a connected app. Its token stops working.
pub(crate) async fn remove_app(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let token_hash = apps::token_hash_of(&w.collection, id)?;
        apps::remove_app(&w.collection, id)?;
        if let Some(token_hash) = token_hash {
            w.app_changed(&crate::workspace::AppChange::Removed {
                app_id: id,
                token_hash,
            });
        }
        // Each grant vault loses the wrapper of the app (ADR 0058 item 9).
        w.sync_grant_vaults(None);
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The layer of the `/app/v1/` routes (ADR 0054 item 8). The layer of
/// `/api/` has already checked the loopback `Host`. This layer needs the
/// token of a connected app, and it puts the app into the request. The
/// access token of the UI is not an app token, so it gives 401.
pub(crate) async fn app_check(
    State(shared): State<Shared>,
    mut request: Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError> {
    let given = Zeroizing::new(
        request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or_default()
            .to_owned(),
    );
    let unauthorized = || {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "app_unauthorized",
            "ADR 0054: this route needs the token of a connected app",
        )
    };
    if given.is_empty() {
        return Err(unauthorized());
    }
    let token = AppToken(given);
    let checked = token.clone();
    let app = blocking(shared, move |w| {
        Ok(apps::app_for_token(&w.collection, checked.as_str())?)
    })
    .await?;
    let Some(app) = app else {
        return Err(unauthorized());
    };
    // No later layer needs the header. The token stays in the extension,
    // which clears its memory (ADR 0058 item 11).
    request.headers_mut().remove(header::AUTHORIZATION);
    request.extensions_mut().insert(app);
    request.extensions_mut().insert(token);
    Ok(next.run(request).await)
}

/// The token of the connected app that made the request. A granted write
/// uses it to open the grant vault. The type clears its memory when it is
/// dropped, and it shows no text in debug output.
#[derive(Clone)]
pub(crate) struct AppToken(Zeroizing<String>);

impl AppToken {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for AppToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppToken(..)")
    }
}

/// The app that the token names. It tests the layer of `/app/v1/`.
pub(crate) async fn whoami(Extension(app): Extension<ConnectedApp>) -> Json<Value> {
    Json(json!({ "id": app.id, "name": app.name }))
}

// ---- Show grants (ADR 0054 items 11 to 17) ----

/// The time of the computer, in seconds since the Unix epoch.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Add a show grant to the draft or remove it, and give the new diff, so
/// that the review sheet needs no second request. The guid must be a
/// `pending` live item of the draft. The route never writes to a host.
fn change_draft_grant(w: &Workspace, id: i64, guid: &str, add: bool) -> ApiResult {
    let id = FeedId(id);
    ensure_draft(w, id)?;
    let (model, _) = pipeline::draft_model(&w.collection, id)?;
    if !grant::is_pending_live_item(&model, guid) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_grant",
            "only a pending live show of the draft can get a grant",
        ));
    }
    if add {
        w.collection.add_draft_grant(id, guid)?;
    } else {
        w.collection.remove_draft_grant(id, guid)?;
    }
    diff_json(w, id)
}

/// Ask for a show grant in the draft (ADR 0054 item 11).
pub(crate) async fn put_draft_grant(
    State(shared): State<Shared>,
    Path((id, guid)): Path<(i64, String)>,
) -> ApiResult {
    blocking(shared, move |w| change_draft_grant(w, id, &guid, true)).await
}

/// Remove a show grant from the draft (ADR 0054 item 11).
pub(crate) async fn delete_draft_grant(
    State(shared): State<Shared>,
    Path((id, guid)): Path<(i64, String)>,
) -> ApiResult {
    blocking(shared, move |w| change_draft_grant(w, id, &guid, false)).await
}

/// End the open grant of a live item: "Stop the grant" on the live item
/// page (ADR 0054 item 17). After it, a connected app gets 409
/// `grant_ended` for this show. A grant that has ended stays ended.
pub(crate) async fn end_feed_grant(
    State(shared): State<Shared>,
    Path((id, guid)): Path<(i64, String)>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        let Some(show) = grant::grant_for(&w.collection, id, &guid)? else {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "no_grant",
                "this show has no grant",
            ));
        };
        grant::end(
            &w.collection,
            show.id,
            i64::try_from(unix_now()).unwrap_or(i64::MAX),
        )?;
        let ended = grant::grant(&w.collection, show.id)?.unwrap_or(show);
        // The grant vault of the grant goes (ADR 0058 item 10).
        w.sync_grant_vaults(None);
        Ok(Json(json!({
            "id": ended.id,
            "item_guid": ended.item_guid,
            "ended_at": ended.ended_at,
        })))
    })
    .await
}

/// The show grants of a feed with their granted writes, for the live item
/// page (ADR 0054 item 17).
pub(crate) async fn list_feed_grants(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.collection.feed(id)?;
        let mut list = Vec::new();
        for show in grant::grants(&w.collection, id)? {
            let writes = grant::writes(&w.collection, show.id)?;
            list.push(json!({
                "id": show.id,
                "item_guid": show.item_guid,
                "created_at": show.created_at,
                "ended_at": show.ended_at,
                "writes": writes
                    .iter()
                    .map(|write| json!({
                        "app_name": write.app_name,
                        "change": write.change,
                        "written_at": write.written_at,
                    }))
                    .collect::<Vec<_>>(),
            }));
        }
        Ok(Json(Value::Array(list)))
    })
    .await
}

/// The feed that the host of a feed gives now, or `None` when the feed has
/// no host or no file. A host that needs the locked vault gives 423
/// `vault_locked`.
fn hosted_feed(w: &Workspace, hosts: &Hosts, id: FeedId) -> Result<Option<Feed>, ApiError> {
    let row = w.collection.feed(id)?;
    let (Some(connector_id), Some(path)) = (row.connector_id, row.path.as_deref()) else {
        return Ok(None);
    };
    let Some(file) = hosts.get(connector_id)?.read(path)? else {
        return Ok(None);
    };
    let doc = studio_core::xml::Document::parse_bytes(&file.bytes)
        .map_err(|e| ApiError::from(pipeline::PipelineError::from(e)))?;
    let feed = Feed::read(&doc).map_err(|e| ApiError::from(pipeline::PipelineError::from(e)))?;
    Ok(Some(feed))
}

/// The hosts for a granted read or write of one grant (ADR 0058 items 11
/// and 13). The secrets come from the grant vault, which the token of the
/// app opens. The user vault is never read, also when it is open. A grant on
/// a host with no secret needs no grant vault.
///
/// # Errors
///
/// Gives 423 `vault_locked` when the grant vault does not open with the
/// token, and an [`ApiError`] when a connector fails.
fn grant_hosts(
    w: &Workspace,
    show: &grant::ShowGrant,
    app: &ConnectedApp,
    token: &AppToken,
    signing: &Signing,
) -> Result<Hosts, ApiError> {
    // An ended grant needs no grant vault. The write then gives
    // `grant_ended` first.
    let vault = if show.ended_at.is_some() {
        None
    } else {
        w.open_grant_vault(show, app.id, token.as_str())?
    };
    w.hosts_with_grant_vault(vault.as_ref(), signing)
}

/// Each active show grant, for a connected app (ADR 0054 item 8). The route
/// reads the hosted file of each feed with a grant, with the hosts of the
/// grant vault. A grant whose vault the token does not open is not in the
/// list, and it gives no error for the full list.
pub(crate) async fn list_shows(
    State(shared): State<Shared>,
    Extension(app): Extension<ConnectedApp>,
    Extension(token): Extension<AppToken>,
) -> ApiResult {
    blocking(shared, move |w| {
        let now = unix_now();
        let signing = Signing::default();
        let mut read: std::collections::HashMap<FeedId, Option<Feed>> =
            std::collections::HashMap::new();
        let mut shows = Vec::new();
        for show in grant::open_grants(&w.collection)? {
            if let std::collections::hash_map::Entry::Vacant(entry) = read.entry(show.feed_id) {
                let hosts = match grant_hosts(w, &show, &app, &token, &signing) {
                    Ok(hosts) => hosts,
                    Err(error) if error.code() == "vault_locked" => continue,
                    Err(error) => return Err(error),
                };
                entry.insert(hosted_feed(w, &hosts, show.feed_id)?);
            }
            let Some(Some(hosted)) = read.get(&show.feed_id) else {
                continue;
            };
            if grant::active(&w.collection, show.feed_id, &show.item_guid, hosted, now)?.is_none() {
                continue;
            }
            let Some(live) = grant::live_item(hosted, &show.item_guid) else {
                continue;
            };
            let row = w.collection.feed(show.feed_id)?;
            shows.push(json!({
                "grant_id": show.id,
                "feed_id": show.feed_id,
                "feed_title": row.title,
                "item_guid": show.item_guid,
                "title": live.item.title,
                "status": live.status,
                "start": live.start,
                "end": live.end,
                "live_value": live.live_value,
            }));
        }
        Ok(Json(Value::Array(shows)))
    })
    .await
}

/// Write one granted change for a connected app (ADR 0054 item 15, ADR
/// 0058 items 11 to 13). Give the version, the status and the live value
/// after the write. The app never gets a secret. The connectors get their
/// secrets from the grant vault, which the token of the app opens. The user
/// vault can be locked, and the write never reads it.
fn granted_change(
    w: &mut Workspace,
    grant_id: i64,
    app: &ConnectedApp,
    token: &AppToken,
    change: &grant::GrantChange,
) -> ApiResult {
    let result = granted_write(w, grant_id, app, token, change);
    // A write can end the grant, and then the grant vault goes.
    w.sync_grant_vaults(None);
    result
}

fn granted_write(
    w: &mut Workspace,
    grant_id: i64,
    app: &ConnectedApp,
    token: &AppToken,
    change: &grant::GrantChange,
) -> ApiResult {
    let show = grant::grant(&w.collection, grant_id)?.ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "no_grant",
            format!("show grant {grant_id} does not exist"),
        )
    })?;
    let now = unix_now();
    // The diff of the granted write makes its signing plan at `now`.
    let signing = Signing {
        plan_time: Some(now),
        signed: Vec::new(),
    };
    let hosts = grant_hosts(w, &show, app, token, &signing)?;
    let library = w.library()?;
    let addresses: &dyn pipeline::AddressCheck = match &w.address_check {
        Some(check) => check.0.as_ref(),
        None => &w.web,
    };
    let version = pipeline::write_granted(
        &mut w.collection,
        &hosts,
        &library,
        addresses,
        show.feed_id,
        &show.item_guid,
        change,
        &app.name,
        now,
    )?;
    // The write put its announcements into the queue (ADR 0052).
    w.announcer.wake();
    let hosted = hosted_feed(w, &hosts, show.feed_id)?;
    if let Some(feed) = &hosted {
        w.record_remote_files_with(show.feed_id, feed, || Ok(hosts));
    }
    let live = hosted
        .as_ref()
        .and_then(|feed| grant::live_item(feed, &show.item_guid));
    Ok(Json(json!({
        "version": version.as_str(),
        "status": live.and_then(|l| l.status.clone()),
        "live_value": live.and_then(|l| l.live_value.clone()),
    })))
}

#[derive(Deserialize)]
pub(crate) struct ShowStatus {
    status: String,
}

/// Start or end a show through its grant: `live` or `ended`.
pub(crate) async fn set_show_status(
    State(shared): State<Shared>,
    Extension(app): Extension<ConnectedApp>,
    Extension(token): Extension<AppToken>,
    Path(grant_id): Path<i64>,
    Json(body): Json<ShowStatus>,
) -> ApiResult {
    blocking(shared, move |w| {
        let change = match body.status.as_str() {
            "live" => grant::GrantChange::GoLive,
            "ended" => grant::GrantChange::End,
            _ => {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "bad_status",
                    "the status is live or ended",
                ));
            }
        };
        granted_change(w, grant_id, &app, &token, &change)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct ShowLiveValue {
    uri: String,
    protocol: String,
}

/// Set or change the `podcast:liveValue` of a show through its grant.
pub(crate) async fn put_show_live_value(
    State(shared): State<Shared>,
    Extension(app): Extension<ConnectedApp>,
    Extension(token): Extension<AppToken>,
    Path(grant_id): Path<i64>,
    Json(body): Json<ShowLiveValue>,
) -> ApiResult {
    blocking(shared, move |w| {
        let uri = body.uri.trim();
        let protocol = body.protocol.trim();
        if uri.is_empty() || protocol.is_empty() {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_live_value",
                "give the uri and the protocol of the live value",
            ));
        }
        let change = grant::GrantChange::SetLiveValue {
            uri: uri.to_owned(),
            protocol: protocol.to_owned(),
        };
        granted_change(w, grant_id, &app, &token, &change)
    })
    .await
}

/// Remove the `podcast:liveValue` of a show through its grant.
pub(crate) async fn delete_show_live_value(
    State(shared): State<Shared>,
    Extension(app): Extension<ConnectedApp>,
    Extension(token): Extension<AppToken>,
    Path(grant_id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        granted_change(
            w,
            grant_id,
            &app,
            &token,
            &grant::GrantChange::RemoveLiveValue,
        )
    })
    .await
}

// ---- The NIP-46 signer of the user (ADR 0056 items 12 to 19) ----

use studio_core::nip46::RemoteSigner;

use crate::workspace::SignerSlot;

/// How long a `nostrconnect://` address waits for the signer.
const SIGNER_WAIT: std::time::Duration = std::time::Duration::from_secs(300);

/// The state of the signer slot, as JSON.
fn signer_state(slot: &SignerSlot) -> Value {
    match slot {
        SignerSlot::None => json!({ "state": "none", "pubkey": null }),
        SignerSlot::Pending { .. } => json!({ "state": "pending", "pubkey": null }),
        SignerSlot::Connected(signer) => json!({
            "state": "connected",
            "pubkey": signer.get_public_key().ok().map(|key| key.to_hex()),
        }),
        SignerSlot::Failed(code) => json!({ "state": "none", "pubkey": null, "error": code }),
    }
}

/// The state of the NIP-46 signer: `none`, `pending` or `connected`. The
/// last failed connection gives its error code in `error`.
pub(crate) async fn get_signer(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| Ok(Json(signer_state(&w.signer)))).await
}

#[derive(Deserialize)]
pub(crate) struct SignerConnect {
    #[serde(default)]
    bunker: Option<String>,
}

/// Connect a NIP-46 signer. A body with no `bunker` starts a
/// `nostrconnect://` address and gives its `uri` and QR code. A `bunker`
/// address connects at once.
pub(crate) async fn connect_signer(
    State(shared): State<Shared>,
    Json(body): Json<SignerConnect>,
) -> ApiResult {
    // The bunker address holds a secret. It clears its memory after use.
    let bunker = body.bunker.map(zeroize::Zeroizing::new);
    let bunker = bunker
        .as_ref()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(|text| zeroize::Zeroizing::new(text.to_owned()));
    let (policy, relays) = blocking(Arc::clone(&shared), |w| {
        Ok((w.net_policy.clone(), w.nip46_relays.clone()))
    })
    .await?;
    if let Some(uri) = bunker {
        connect_bunker(shared, policy, uri).await
    } else {
        start_nostrconnect(shared, policy, relays).await
    }
}

/// Connect with a `bunker://` address at once.
async fn connect_bunker(
    shared: Shared,
    policy: studio_core::net::NetPolicy,
    uri: zeroize::Zeroizing<String>,
) -> ApiResult {
    let connected = blocking_unlocked(move || {
        let made = RemoteSigner::bunker(
            &uri,
            &policy,
            std::time::Instant::now() + studio_core::nip46::REQUEST_TIMEOUT,
        )?;
        made.get_public_key()?;
        Ok(made)
    })
    .await?;
    let connected = Arc::new(connected);
    blocking(shared, move |w| {
        w.signer.clear();
        w.signer = SignerSlot::Connected(connected);
        Ok(Json(signer_state(&w.signer)))
    })
    .await
}

/// Start a `nostrconnect://` address, and wait for the signer on a
/// blocking thread.
async fn start_nostrconnect(
    shared: Shared,
    policy: studio_core::net::NetPolicy,
    relays: Vec<String>,
) -> ApiResult {
    let (pending, uri) =
        blocking_unlocked(move || Ok(RemoteSigner::nostrconnect(&relays, &policy)?)).await?;
    let cancel = pending.canceller();
    let generation = blocking(Arc::clone(&shared), move |w| {
        w.signer.clear();
        w.signer_generation += 1;
        w.signer = SignerSlot::Pending {
            cancel,
            generation: w.signer_generation,
        };
        Ok(w.signer_generation)
    })
    .await?;
    // The wait holds no lock of the workspace.
    tokio::task::spawn_blocking(move || {
        let result = pending
            .wait(std::time::Instant::now() + SIGNER_WAIT)
            .and_then(|made| {
                made.get_public_key()?;
                Ok(Arc::new(made))
            });
        let Ok(mut w) = lock(&shared) else { return };
        let own = matches!(
            &w.signer,
            SignerSlot::Pending { generation: g, .. } if *g == generation
        );
        if own {
            w.signer = match result {
                Ok(made) => SignerSlot::Connected(made),
                Err(error) => SignerSlot::Failed(ApiError::from(error).code()),
            };
        }
    });
    Ok(Json(json!({
        "state": "pending",
        "uri": uri,
        "qr_svg": crate::qr_svg(&uri),
    })))
}

/// Remove the NIP-46 signer, or stop the wait for one.
pub(crate) async fn delete_signer(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        w.signer.clear();
        Ok(Json(signer_state(&w.signer)))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SignerSign {
    event: Value,
}

/// Ask the NIP-46 signer to sign one planned event of a write. The route
/// takes the signer under the lock and then releases the lock. The wait for
/// the signer holds no lock of the workspace.
pub(crate) async fn sign_with_signer(
    State(shared): State<Shared>,
    Json(body): Json<SignerSign>,
) -> ApiResult {
    // The kind check comes first, before any other state.
    let unsigned = studio_core::nip46::parse_planned(&body.event.to_string())?;
    let signer = blocking(shared, |w| match &w.signer {
        SignerSlot::Connected(signer) => Ok(Arc::clone(signer)),
        _ => Err(ApiError::new(
            StatusCode::CONFLICT,
            "signer_not_connected",
            "connect a signer first",
        )),
    })
    .await?;
    let event = blocking_unlocked(move || Ok(signer.sign_planned(unsigned)?)).await?;
    serde_json::from_str::<Value>(&event.as_json())
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use studio_core::feed::RssImage;

    fn demu_feed(url: &str) -> Feed {
        Feed {
            rss_image: Some(RssImage {
                url: Some(url.to_owned()),
                title: Some("Art".to_owned()),
                ..RssImage::default()
            }),
            ..Feed::default()
        }
    }

    #[test]
    fn a_reference_changes_the_rss_image_url_and_keeps_its_other_children() {
        let mut feed = demu_feed("https://a.example/c.png");
        reference_feed(
            &mut feed,
            "https://a.example/c.png",
            "https://b.example/c.png",
        );
        let image = feed.rss_image.unwrap();
        assert_eq!(image.url.as_deref(), Some("https://b.example/c.png"));
        assert_eq!(image.title.as_deref(), Some("Art"));
        assert_eq!(feed.image, None);
    }

    #[test]
    fn a_reference_of_another_address_keeps_the_rss_image() {
        let mut feed = demu_feed("https://a.example/c.png");
        let before = feed.clone();
        reference_feed(
            &mut feed,
            "https://a.example/other.png",
            "https://b.example/x.png",
        );
        assert_eq!(feed, before);
    }

    #[test]
    fn a_dereference_removes_the_full_rss_image() {
        let mut feed = demu_feed("https://a.example/c.png");
        dereference_feed(&mut feed, "https://a.example/c.png");
        assert_eq!(feed.rss_image, None);
    }

    #[test]
    fn a_dereference_of_another_address_keeps_the_rss_image() {
        let mut feed = demu_feed("https://a.example/c.png");
        let before = feed.clone();
        dereference_feed(&mut feed, "https://a.example/other.png");
        assert_eq!(feed, before);
    }
}
