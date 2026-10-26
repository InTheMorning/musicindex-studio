//! The Blossom connector (ADR 0048).
//!
//! The connector keeps media blobs on one or more Blossom servers. The
//! collection keeps one record for each path. A blob goes from a server
//! only when no other record has its SHA-256.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::io::Read;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use nostr::prelude::{Event, EventId, Keys, PublicKey, UnsignedEvent};
use serde::Deserialize;
use zeroize::Zeroizing;

use super::blossom_http::{self, Spooled};
use super::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, HostChanges,
    KeptBlob, LIST_MAX_DEPTH, LIST_MAX_FILES, ListedFile, PlannedEvent, Stamp, Version,
    relative_path,
};
use crate::nostr_events::{self, AuthAction, Signing};

/// Who signs the authorization events of a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum SigningMode {
    /// The key is in the vault.
    Vault,
    /// The user signs each event with "My signer".
    Signer,
}

/// The configuration of a Blossom host. It holds no secret.
#[derive(Debug, Deserialize)]
struct BlossomConfig {
    public_key: String,
    servers: Vec<String>,
    signing: SigningMode,
}

/// The secret of a Blossom host with the signing `vault`.
#[derive(Deserialize)]
struct BlossomSecret {
    secret_key: String,
}

/// The state that changes during a write.
#[derive(Default)]
struct State {
    /// The record of each path.
    records: BTreeMap<String, KeptBlob>,
    /// The planned events: the server index, the action and the event.
    plan: Vec<(usize, AuthAction, UnsignedEvent)>,
    /// The events that the connector signed, by event ID.
    signed: HashMap<EventId, Event>,
}

/// A host of Blossom servers.
pub struct BlossomHost {
    public_key: PublicKey,
    servers: Vec<String>,
    mode: SigningMode,
    keys: Option<Keys>,
    signing: Signing,
    state: Mutex<State>,
}

impl fmt::Debug for BlossomHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The text shows no secret key.
        f.debug_struct("BlossomHost")
            .field("public_key", &self.public_key.to_hex())
            .field("servers", &self.servers)
            .field("signing", &self.mode)
            .finish_non_exhaustive()
    }
}

pub(super) fn invalid(message: &str) -> ConnectorError {
    ConnectorError::new(ConnectorErrorKind::InvalidConfig, message)
}

pub(super) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub(super) fn version_of(sha256: &str) -> String {
    format!("sha256:{sha256}")
}

/// The media type of a file name, from its extension.
pub(super) fn media_type_of(path: &str) -> String {
    if let Some(known) = crate::remote::media_type_of_name(path) {
        return known;
    }
    let extension = path
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "xml" => "application/xml",
        "json" => "application/json",
        "html" => "text/html",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
    .to_owned()
}

/// The extension of a path for a public address, or an empty text.
fn extension_of(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((stem, extension))
            if !stem.is_empty()
                && !extension.is_empty()
                && extension.bytes().all(|b| b.is_ascii_alphanumeric()) =>
        {
            format!(".{}", extension.to_ascii_lowercase())
        }
        _ => String::new(),
    }
}

/// True when the unsigned event has an `x` tag with this SHA-256.
pub(super) fn has_x_tag(event: &UnsignedEvent, sha256: &str) -> bool {
    event
        .tags
        .iter()
        .any(|tag| tag.kind() == "x" && tag.as_slice().get(1).is_some_and(|value| value == sha256))
}

impl BlossomHost {
    /// The kind of a Blossom connector row.
    pub const KIND: &'static str = "blossom";

    /// Make a host from the configuration JSON of a row.
    ///
    /// `secret` is the JSON `{"secret_key": "<hex>"}` for the signing
    /// `vault`. `blobs` are the records of the collection. `signing` has
    /// the events that the user signed.
    ///
    /// # Errors
    ///
    /// Gives `InvalidConfig` when the configuration is not valid, when the
    /// signing is `vault` and the secret is missing, not valid or not the
    /// key of `public_key`. The messages hold no part of the secret.
    pub fn from_config(
        config: &str,
        secret: Option<&[u8]>,
        blobs: Vec<KeptBlob>,
        signing: Signing,
    ) -> Result<Self, ConnectorError> {
        let config: BlossomConfig = serde_json::from_str(config)
            .map_err(|_| invalid("the Blossom configuration is not valid"))?;
        let hex_key = config.public_key.len() == 64
            && config.public_key.bytes().all(|b| b.is_ascii_hexdigit());
        if !hex_key {
            return Err(invalid(
                "the public key must have 64 hexadecimal characters",
            ));
        }
        let public_key = nostr_events::parse_public_key(&config.public_key)?;
        if config.servers.is_empty() {
            return Err(invalid("the Blossom connector needs one server"));
        }
        let servers = config
            .servers
            .iter()
            .map(|server| {
                let server = server.trim().trim_end_matches('/');
                let host_ok = server.split_once("://").is_some_and(|(scheme, rest)| {
                    matches!(scheme, "http" | "https") && !rest.is_empty()
                });
                if host_ok && nostr_events::server_domain(server).is_ok() {
                    Ok(server.to_owned())
                } else {
                    Err(invalid("a server address must be an http or https address"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let keys = match config.signing {
            SigningMode::Signer => None,
            SigningMode::Vault => {
                let secret =
                    secret.ok_or_else(|| invalid("the Blossom connector needs a secret"))?;
                let parsed: BlossomSecret = serde_json::from_slice(secret)
                    .map_err(|_| invalid("the secret must be JSON with secret_key"))?;
                let text = Zeroizing::new(parsed.secret_key);
                let keys = nostr_events::parse_secret_key(&text)?;
                if keys.public_key() != public_key {
                    return Err(invalid("the secret key is not the key of the host"));
                }
                Some(keys)
            }
        };
        let records = blobs.into_iter().map(|b| (b.path.clone(), b)).collect();
        Ok(Self {
            public_key,
            servers,
            mode: config.signing,
            keys,
            signing,
            state: Mutex::new(State {
                records,
                ..State::default()
            }),
        })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn record_of(&self, path: &str) -> Result<Option<KeptBlob>, ConnectorError> {
        relative_path(path)?;
        Ok(self.state().records.get(path).cloned())
    }

    /// The `Authorization` header for one action on one server.
    ///
    /// The function uses the planned event of the server when it has the
    /// SHA-256. If not, it makes an event for this call.
    fn authorization(
        &self,
        index: usize,
        action: AuthAction,
        sha256: &str,
    ) -> Result<String, ConnectorError> {
        let planned = {
            let state = self.state();
            state
                .plan
                .iter()
                .find(|(i, a, e)| *i == index && *a == action && has_x_tag(e, sha256))
                .map(|(_, _, e)| e.clone())
        };
        let ad_hoc = planned.is_none();
        let unsigned = if let Some(event) = planned {
            event
        } else {
            if self.mode == SigningMode::Signer {
                return Err(ConnectorError::new(
                    ConnectorErrorKind::SignatureMissing,
                    "the write needs an event that the user did not sign",
                ));
            }
            let domain = nostr_events::server_domain(&self.servers[index])?;
            nostr_events::auth_event(
                self.public_key,
                action,
                &[sha256.to_owned()],
                &domain,
                now(),
            )
        };
        let id = unsigned.compute_id();
        if let Some(event) = self.state().signed.get(&id) {
            return Ok(nostr_events::auth_header(event));
        }
        let event = nostr_events::sign(
            unsigned,
            self.keys.as_ref(),
            &self.signing,
            &self.public_key,
        )?;
        if !ad_hoc {
            self.state().signed.insert(id, event.clone());
        }
        Ok(nostr_events::auth_header(&event))
    }

    /// Check `expected` against the record of the path.
    fn check_expected(&self, path: &str, expected: &Expected) -> Result<(), ConnectorError> {
        let record = self.record_of(path)?;
        let conflict = || {
            ConnectorError::new(
                ConnectorErrorKind::Conflict,
                format!("{path} changed on the host"),
            )
        };
        match (expected, record) {
            (Expected::Absent, None) => Ok(()),
            (Expected::Version(version), Some(record))
                if version.as_str() == version_of(&record.sha256) =>
            {
                Ok(())
            }
            _ => Err(conflict()),
        }
    }

    /// Put the bytes on each server and keep the record of the path.
    fn put(
        &self,
        path: &str,
        spooled: &Spooled<'_>,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.check_expected(path, expected)?;
        let sha256 = spooled.sha256();
        let media_type = media_type_of(path);
        let first = &self.servers[0];
        let header = self.authorization(0, AuthAction::Upload, sha256)?;
        blossom_http::upload(first, spooled, &media_type, &header)?;
        let source = format!("{first}/{sha256}");
        for (index, server) in self.servers.iter().enumerate().skip(1) {
            let header = self.authorization(index, AuthAction::Upload, sha256)?;
            blossom_http::mirror(server, &source, sha256, &header)?;
        }
        self.state().records.insert(
            path.to_owned(),
            KeptBlob {
                path: path.to_owned(),
                sha256: sha256.to_owned(),
                size: spooled.size(),
            },
        );
        Ok(Version::new(version_of(sha256)))
    }

    /// The SHA-256 values that a write removes from the servers: those of
    /// the deleted paths that no kept record and no new write has.
    fn removed_hashes(&self, changes: &HostChanges) -> Vec<String> {
        let state = self.state();
        let deleted: BTreeSet<&str> = changes.deletes.iter().map(|d| d.path.as_str()).collect();
        let mut kept: BTreeSet<&str> = state
            .records
            .values()
            .filter(|r| !deleted.contains(r.path.as_str()))
            .map(|r| r.sha256.as_str())
            .collect();
        kept.extend(changes.writes.iter().map(|w| w.sha256.as_str()));
        let mut removed = Vec::new();
        for delete in &changes.deletes {
            let sha256 = delete
                .sha256
                .as_deref()
                .or_else(|| state.records.get(&delete.path).map(|r| r.sha256.as_str()));
            if let Some(sha256) = sha256
                && !kept.contains(sha256)
                && !removed.iter().any(|r: &String| r == sha256)
            {
                removed.push(sha256.to_owned());
            }
        }
        removed
    }
}

impl Connector for BlossomHost {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        let Some(record) = self.record_of(path)? else {
            return Ok(None);
        };
        let mut last_error = None;
        for server in &self.servers {
            match blossom_http::get(server, &record.sha256) {
                Ok(Some(bytes)) => {
                    return Ok(Some(FileRead {
                        bytes,
                        version: Version::new(version_of(&record.sha256)),
                    }));
                }
                Ok(None) => {}
                Err(error) => last_error = Some(error),
            }
        }
        last_error.map_or(Ok(None), Err)
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.check_expected(path, expected)?;
        self.put(path, &Spooled::from_bytes(bytes), expected)
    }

    fn write_from(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.check_expected(path, expected)?;
        let spooled = Spooled::from_reader(reader, length)?;
        self.put(path, &spooled, expected)
    }

    fn sha256(&self, path: &str) -> Result<Option<String>, ConnectorError> {
        Ok(self.record_of(path)?.map(|r| r.sha256))
    }

    fn public_url(&self, path: &str) -> Option<String> {
        let record = self.state().records.get(path).cloned()?;
        Some(format!(
            "{}/{}{}",
            self.servers[0],
            record.sha256,
            extension_of(path)
        ))
    }

    fn planned_url(&self, path: &str, sha256: Option<&str>) -> Option<String> {
        match sha256 {
            Some(sha256) => Some(format!(
                "{}/{sha256}{}",
                self.servers[0],
                extension_of(path)
            )),
            None => self.public_url(path),
        }
    }

    fn plan_events(
        &self,
        changes: &HostChanges,
        plan_time: u64,
    ) -> Result<Vec<PlannedEvent>, ConnectorError> {
        let mut uploads: Vec<String> = Vec::new();
        for write in &changes.writes {
            if !uploads.contains(&write.sha256) {
                uploads.push(write.sha256.clone());
            }
        }
        let removed = self.removed_hashes(changes);
        let user_signs = self.mode == SigningMode::Signer;
        let mut plan = Vec::new();
        let mut events = Vec::new();
        for (index, server) in self.servers.iter().enumerate() {
            let domain = nostr_events::server_domain(server)?;
            for (action, hashes) in [
                (AuthAction::Upload, &uploads),
                (AuthAction::Delete, &removed),
            ] {
                if hashes.is_empty() {
                    continue;
                }
                let event =
                    nostr_events::auth_event(self.public_key, action, hashes, &domain, plan_time);
                plan.push((index, action, event.clone()));
                events.push(PlannedEvent { event, user_signs });
            }
        }
        let mut state = self.state();
        state.plan = plan;
        state.signed.clear();
        Ok(events)
    }

    fn kept_blobs(&self) -> Option<Vec<KeptBlob>> {
        Some(self.state().records.values().cloned().collect())
    }

    fn list(&self) -> Result<FileList, ConnectorError> {
        let state = self.state();
        let mut list = FileList::default();
        for record in state.records.values() {
            let segments: Vec<&str> = record.path.split('/').collect();
            if segments.iter().any(|s| s.starts_with('.')) {
                continue;
            }
            if segments.len() > LIST_MAX_DEPTH {
                list.truncated = true;
                continue;
            }
            if list.files.len() == LIST_MAX_FILES {
                list.truncated = true;
                break;
            }
            list.files.push(ListedFile {
                path: record.path.clone(),
                size: record.size,
                stamp: Stamp::new(version_of(&record.sha256)),
            });
        }
        Ok(list)
    }

    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        Ok(self.record_of(path)?.map(|record| ListedFile {
            path: record.path,
            size: record.size,
            stamp: Stamp::new(version_of(&record.sha256)),
        }))
    }

    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        let Some(record) = self.record_of(path)? else {
            return Err(ConnectorError::new(
                ConnectorErrorKind::NotFound,
                format!("{path} does not exist"),
            ));
        };
        if expected.as_str() != version_of(&record.sha256) {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Conflict,
                format!("{path} changed on the host"),
            ));
        }
        let shared = {
            let mut state = self.state();
            state.records.remove(path);
            state
                .records
                .values()
                .any(|other| other.sha256 == record.sha256)
        };
        if shared {
            return Ok(());
        }
        for (index, server) in self.servers.iter().enumerate() {
            let header = self.authorization(index, AuthAction::Delete, &record.sha256)?;
            // A missing blob is not an error: the goal is that it is gone.
            blossom_http::delete(server, &record.sha256, &header)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::suite;
    use super::*;
    use crate::collection::{ConnectorId, ConnectorRow};
    use crate::connector::{HostEnv, PlannedDelete, PlannedWrite, from_row_in};
    use crate::digest::sha256_hex;
    use crate::nostr_events::{parse_secret_key, sign_with_keys};
    use crate::testing::blossom::TestBlossom;
    use crate::vault::OpenVault;
    use std::cell::RefCell;

    const SECRET_HEX: &str = "0101010101010101010101010101010101010101010101010101010101010101";

    fn keys() -> Keys {
        parse_secret_key(SECRET_HEX).expect("keys")
    }

    fn config(servers: &[String], signing: &str) -> String {
        serde_json::json!({
            "public_key": keys().public_key().to_hex(),
            "servers": servers,
            "signing": signing,
        })
        .to_string()
    }

    fn secret() -> Vec<u8> {
        serde_json::json!({ "secret_key": SECRET_HEX })
            .to_string()
            .into_bytes()
    }

    fn vault_host(servers: &[&TestBlossom]) -> BlossomHost {
        let urls: Vec<String> = servers.iter().map(|s| s.url()).collect();
        BlossomHost::from_config(
            &config(&urls, "vault"),
            Some(&secret()),
            Vec::new(),
            Signing::default(),
        )
        .expect("a host")
    }

    #[test]
    fn the_configuration_is_checked() {
        let urls = vec!["http://127.0.0.1:1".to_owned()];
        let make = |config: &str, secret: Option<&[u8]>| {
            BlossomHost::from_config(config, secret, Vec::new(), Signing::default())
        };
        let kind = |r: Result<BlossomHost, ConnectorError>| r.err().map(|e| e.kind());
        assert!(make(&config(&urls, "vault"), Some(&secret())).is_ok());
        assert!(make(&config(&urls, "signer"), None).is_ok());
        let invalid = Some(ConnectorErrorKind::InvalidConfig);
        assert_eq!(kind(make("{}", None)), invalid);
        assert_eq!(kind(make(&config(&[], "signer"), None)), invalid);
        assert_eq!(
            kind(make(
                &config(&["ftp://example.com".to_owned()], "signer"),
                None
            )),
            invalid
        );
        assert_eq!(kind(make(&config(&urls, "vault"), None)), invalid);
        assert_eq!(kind(make(&config(&urls, "other"), None)), invalid);
        let short = serde_json::json!({"public_key": "abcd", "servers": urls, "signing": "signer"});
        assert_eq!(kind(make(&short.to_string(), None)), invalid);
    }

    #[test]
    fn a_secret_for_another_key_gives_invalid_config() {
        let urls = vec!["http://127.0.0.1:1".to_owned()];
        let other = serde_json::json!({ "secret_key": "02".repeat(32) })
            .to_string()
            .into_bytes();
        let error = BlossomHost::from_config(
            &config(&urls, "vault"),
            Some(&other),
            Vec::new(),
            Signing::default(),
        )
        .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        assert!(!error.to_string().contains(&"02".repeat(32)));
    }

    #[test]
    fn the_debug_text_shows_no_secret_key() {
        let server = TestBlossom::start();
        let host = vault_host(&[&server]);
        let text = format!("{host:?}");
        assert!(!text.contains(SECRET_HEX));
        assert!(text.contains("BlossomHost"));
    }

    #[test]
    fn from_row_in_makes_a_blossom_host_from_the_vault() {
        let server = TestBlossom::start();
        let mut vault = OpenVault::create("correct horse battery staple 2026").unwrap();
        vault.put("host-1", &secret()).unwrap();
        let row = ConnectorRow {
            id: ConnectorId(1),
            kind: BlossomHost::KIND.to_owned(),
            name: "Blobs".to_owned(),
            config: config(&[server.url()], "vault"),
            secret_ref: Some("host-1".to_owned()),
        };
        let blob = KeptBlob {
            path: "a.mp3".to_owned(),
            sha256: sha256_hex(&[b"abc"]),
            size: 3,
        };
        let connector = from_row_in(
            &row,
            &HostEnv {
                vault: Some(&vault),
                blobs: vec![blob.clone()],
                signing: Signing::default(),
            },
        )
        .expect("a connector");
        assert_eq!(connector.kept_blobs(), Some(vec![blob]));
    }

    #[test]
    fn a_write_puts_the_blob_on_both_servers() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let host = vault_host(&[&first, &second]);
        host.write("a/one.mp3", b"abc", &Expected::Absent).unwrap();
        let sha = sha256_hex(&[b"abc"]);
        assert_eq!(first.blobs(), vec![sha.clone()]);
        assert_eq!(second.blobs(), vec![sha]);
    }

    #[test]
    fn a_shared_blob_goes_only_with_its_last_path() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let host = vault_host(&[&first, &second]);
        host.write("one.mp3", b"abc", &Expected::Absent).unwrap();
        host.write("two.mp3", b"abc", &Expected::Absent).unwrap();
        let stamp = host.stat("one.mp3").unwrap().unwrap().stamp;
        host.delete("one.mp3", &stamp).unwrap();
        assert_eq!(first.deletes().len(), 0);
        assert_eq!(second.deletes().len(), 0);
        let stamp = host.stat("two.mp3").unwrap().unwrap().stamp;
        host.delete("two.mp3", &stamp).unwrap();
        let sha = sha256_hex(&[b"abc"]);
        assert_eq!(first.deletes(), vec![sha.clone()]);
        assert_eq!(second.deletes(), vec![sha]);
        assert!(first.blobs().is_empty() && second.blobs().is_empty());
    }

    #[test]
    fn the_public_and_planned_addresses_have_the_extension() {
        let server = TestBlossom::start();
        let host = vault_host(&[&server]);
        let sha = sha256_hex(&[b"abc"]);
        let expected = format!("{}/{sha}.mp3", server.url());
        assert_eq!(host.public_url("a/one.mp3"), None);
        assert_eq!(
            host.planned_url("a/one.mp3", Some(&sha)).as_deref(),
            Some(expected.as_str())
        );
        host.write("a/one.mp3", b"abc", &Expected::Absent).unwrap();
        assert_eq!(
            host.public_url("a/one.mp3").as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(
            host.planned_url("a/one.mp3", None).as_deref(),
            Some(expected.as_str())
        );
    }

    #[test]
    fn the_signer_without_a_plan_gives_signature_missing() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let urls = [first.url(), second.url()];
        let host = BlossomHost::from_config(
            &config(&urls, "signer"),
            None,
            Vec::new(),
            Signing::default(),
        )
        .unwrap();
        let error = host
            .write("one.mp3", b"abc", &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::SignatureMissing);
        assert!(first.blobs().is_empty() && second.blobs().is_empty());
        assert_eq!(host.read("one.mp3").unwrap(), None);
    }

    #[test]
    fn the_signer_uses_the_events_that_the_user_signed() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let urls = [first.url(), second.url()];
        let sha = sha256_hex(&[b"abc"]);
        let changes = HostChanges {
            writes: vec![PlannedWrite {
                path: "one.mp3".to_owned(),
                sha256: sha.clone(),
                size: 3,
            }],
            deletes: Vec::new(),
        };
        let plan_time = now();
        let first_host = BlossomHost::from_config(
            &config(&urls, "signer"),
            None,
            Vec::new(),
            Signing::default(),
        )
        .unwrap();
        let planned = first_host.plan_events(&changes, plan_time).unwrap();
        assert_eq!(planned.len(), 2);
        assert!(planned.iter().all(|p| p.user_signs));
        let signed = planned
            .iter()
            .map(|p| sign_with_keys(p.event.clone(), &keys()).unwrap())
            .collect();
        let host = BlossomHost::from_config(
            &config(&urls, "signer"),
            None,
            Vec::new(),
            Signing {
                plan_time: Some(plan_time),
                signed,
            },
        )
        .unwrap();
        host.plan_events(&changes, plan_time).unwrap();
        host.write("one.mp3", b"abc", &Expected::Absent).unwrap();
        assert_eq!(first.blobs(), vec![sha.clone()]);
        assert_eq!(second.blobs(), vec![sha]);
    }

    #[test]
    fn a_plan_of_two_writes_uses_one_upload_event_for_each_server() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let host = vault_host(&[&first, &second]);
        let (sha_a, sha_b) = (sha256_hex(&[b"aaa"]), sha256_hex(&[b"bbb"]));
        let changes = HostChanges {
            writes: vec![
                PlannedWrite {
                    path: "a.mp3".to_owned(),
                    sha256: sha_a.clone(),
                    size: 3,
                },
                PlannedWrite {
                    path: "b.mp3".to_owned(),
                    sha256: sha_b.clone(),
                    size: 3,
                },
            ],
            deletes: Vec::new(),
        };
        let planned = host.plan_events(&changes, now()).unwrap();
        assert_eq!(planned.len(), 2);
        assert!(planned.iter().all(|p| !p.user_signs));
        host.write("a.mp3", b"aaa", &Expected::Absent).unwrap();
        host.write("b.mp3", b"bbb", &Expected::Absent).unwrap();
        // The two test servers have one host name, so their events are equal.
        assert_eq!(host.state().signed.len(), 1);
        let mut expected = vec![sha_a, sha_b];
        expected.sort();
        assert_eq!(first.blobs(), expected);
        assert_eq!(second.blobs(), expected);
    }

    #[test]
    fn a_planned_delete_has_the_blobs_that_no_record_keeps() {
        let server = TestBlossom::start();
        let host = vault_host(&[&server]);
        host.write("one.mp3", b"abc", &Expected::Absent).unwrap();
        host.write("two.mp3", b"abc", &Expected::Absent).unwrap();
        host.write("three.mp3", b"xyz", &Expected::Absent).unwrap();
        let delete = |path: &str| PlannedDelete {
            path: path.to_owned(),
            sha256: None,
        };
        let shared = HostChanges {
            writes: Vec::new(),
            deletes: vec![delete("one.mp3")],
        };
        assert!(host.plan_events(&shared, now()).unwrap().is_empty());
        let both = HostChanges {
            writes: Vec::new(),
            deletes: vec![delete("one.mp3"), delete("two.mp3"), delete("three.mp3")],
        };
        assert_eq!(host.plan_events(&both, now()).unwrap().len(), 1);
        assert_eq!(host.removed_hashes(&both).len(), 2);
        let stamp = host.stat("three.mp3").unwrap().unwrap().stamp;
        host.delete("three.mp3", &stamp).unwrap();
        assert_eq!(server.deletes(), vec![sha256_hex(&[b"xyz"])]);
    }

    #[test]
    fn write_from_with_a_reader_of_a_different_length_gives_io() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let host = vault_host(&[&first, &second]);
        let error = host
            .write_from("one.mp3", &mut &b"abc"[..], 10, &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
        assert!(first.blobs().is_empty() && second.blobs().is_empty());
        assert_eq!(host.kept_blobs(), Some(Vec::new()));
    }

    #[test]
    fn the_signer_deletes_with_the_planned_event_after_finish() {
        let (first, second) = (TestBlossom::start(), TestBlossom::start());
        let urls = [first.url(), second.url()];
        let writer = vault_host(&[&first, &second]);
        writer.write("old.mp3", b"old", &Expected::Absent).unwrap();
        writer
            .write("keep.mp3", b"keep", &Expected::Absent)
            .unwrap();
        let blobs = writer.kept_blobs().unwrap();
        let changes = HostChanges {
            writes: vec![PlannedWrite {
                path: "new.mp3".to_owned(),
                sha256: sha256_hex(&[b"new"]),
                size: 3,
            }],
            deletes: vec![PlannedDelete {
                path: "old.mp3".to_owned(),
                sha256: None,
            }],
        };
        let plan_time = now();
        let make = |signing: Signing| {
            BlossomHost::from_config(&config(&urls, "signer"), None, blobs.clone(), signing)
                .unwrap()
        };
        let planned = make(Signing::default())
            .plan_events(&changes, plan_time)
            .unwrap();
        // One upload event and one delete event for each of two servers.
        assert_eq!(planned.len(), 4);
        let signed = planned
            .iter()
            .map(|p| sign_with_keys(p.event.clone(), &keys()).unwrap())
            .collect();
        let host = make(Signing {
            plan_time: Some(plan_time),
            signed,
        });
        host.plan_events(&changes, plan_time).unwrap();
        host.write("new.mp3", b"new", &Expected::Absent).unwrap();
        host.finish().unwrap();
        let stamp = host.stat("old.mp3").unwrap().unwrap().stamp;
        host.delete("old.mp3", &stamp).unwrap();
        host.finish().unwrap();
        let mut expected = vec![sha256_hex(&[b"keep"]), sha256_hex(&[b"new"])];
        expected.sort();
        assert_eq!(first.blobs(), expected);
        assert_eq!(second.blobs(), expected);
    }

    /// Do a check of the shared suite. Each connector has two new servers.
    fn suite_check(check: fn(&dyn Fn() -> Box<dyn Connector>)) {
        let servers: RefCell<Vec<TestBlossom>> = RefCell::new(Vec::new());
        check(&|| {
            let (first, second) = (TestBlossom::start(), TestBlossom::start());
            let host = vault_host(&[&first, &second]);
            servers.borrow_mut().extend([first, second]);
            Box::new(host)
        });
    }

    macro_rules! suite_tests {
        ($($name:ident),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    suite_check(suite::$name);
                }
            )*
        };
    }

    suite_tests!(
        read_of_an_absent_file_gives_none,
        write_then_read_gives_the_bytes_and_the_version,
        write_over_a_file_with_absent_is_a_conflict,
        write_with_an_old_version_is_a_conflict,
        write_with_the_current_version_replaces_the_file,
        write_from_gives_the_same_file_as_write,
        write_from_with_a_short_reader_writes_nothing,
        stat_gives_the_size_and_a_new_stamp_after_a_change,
        list_gives_each_file_in_path_sequence,
        delete_with_the_current_stamp_removes_the_file,
        delete_with_an_old_stamp_is_a_conflict,
        delete_of_an_absent_file_is_not_found,
        a_path_out_of_the_host_is_invalid,
        sha256_when_given_is_the_sha256_of_the_bytes,
        set_redirect_without_the_ability_is_unsupported,
        finish_keeps_each_change,
    );

    #[test]
    fn the_suite_runs_all_checks() {
        suite_check(suite::run_all);
    }
}
