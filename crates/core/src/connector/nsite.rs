//! The nsite connector (ADR 0048, ADR 0050).
//!
//! The connector holds the feed XML and the media of one named site. The
//! blobs go to Blossom servers. The site is one manifest, kind 35128, on
//! the relays. A write changes the view in memory, and `finish` publishes
//! the view as one new manifest.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::Read;
use std::sync::{Mutex, MutexGuard, PoisonError};

use nostr::prelude::{Event, EventId, Filter, Keys, Kind, PublicKey, UnsignedEvent};
use serde::Deserialize;
use zeroize::Zeroizing;

use super::blossom::{SigningMode, has_x_tag, invalid, media_type_of, now, version_of};
use super::blossom_http::{self, Spooled};
use super::relay::{self, RELAY_TIMEOUT, Reads};
use super::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, HostChanges,
    LIST_MAX_DEPTH, LIST_MAX_FILES, ListedFile, PlannedEvent, Stamp, Version, relative_path,
};
use crate::nostr_events::{self, AuthAction, Manifest, Signing};

/// The kind of the nsite named site manifest (NIP-5A).
const KIND_MANIFEST: u16 = 35128;
/// The gateway that a host uses when its configuration has none.
const DEFAULT_GATEWAY: &str = "nsite.lol";

/// The configuration of an nsite host. It holds no secret.
#[derive(Debug, Deserialize)]
struct NsiteConfig {
    public_key: String,
    d: String,
    relays: Vec<String>,
    servers: Vec<String>,
    #[serde(default)]
    gateway: Option<String>,
    signing: SigningMode,
    #[serde(default)]
    title: Option<String>,
}

/// The secret of an nsite host with the signing `vault`.
#[derive(Deserialize)]
struct NsiteSecret {
    secret_key: String,
}

/// The events of a write that the connector plans.
struct Plan {
    /// The planned manifests, each with the paths that it has.
    manifests: Vec<(BTreeMap<String, String>, UnsignedEvent)>,
    /// The planned upload events, by the index of the server.
    uploads: Vec<(usize, UnsignedEvent)>,
    /// The number of planned manifests that `finish` used.
    next: usize,
}

/// The state that changes during a write.
#[derive(Default)]
struct State {
    /// True when the view has the manifest of the relays.
    loaded: bool,
    /// The manifest that the view started from.
    base: Option<Event>,
    /// The view: the SHA-256 of each manifest path.
    paths: BTreeMap<String, String>,
    /// The size of each blob that the connector found, by SHA-256.
    sizes: HashMap<String, u64>,
    /// True when the view has a change that `finish` did not publish.
    dirty: bool,
    plan: Option<Plan>,
    /// The events that the connector signed, by event ID.
    signed: HashMap<EventId, Event>,
    /// The ID of the newest manifest of each relay after the last `finish`.
    relay_ids: Vec<(String, Option<EventId>)>,
}

/// A host that holds one nsite named site.
pub struct NsiteHost {
    public_key: PublicKey,
    d: String,
    relays: Vec<String>,
    servers: Vec<String>,
    gateway: String,
    title: Option<String>,
    mode: SigningMode,
    keys: Option<Keys>,
    signing: Signing,
    state: Mutex<State>,
}

impl fmt::Debug for NsiteHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The text shows no secret key.
        f.debug_struct("NsiteHost")
            .field("public_key", &self.public_key.to_hex())
            .field("d", &self.d)
            .field("relays", &self.relays)
            .field("servers", &self.servers)
            .field("gateway", &self.gateway)
            .field("signing", &self.mode)
            .finish_non_exhaustive()
    }
}

/// The ID and the time of a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestRef {
    /// The event ID, as lowercase hexadecimal.
    pub id: String,
    /// The `created_at` time of the manifest.
    pub created_at: u64,
}

/// The newest manifest of one relay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayState {
    /// The address of the relay.
    pub address: String,
    /// The newest manifest of the site, `None` when the relay has no
    /// manifest, or the error of the read.
    pub newest: Result<Option<ManifestRef>, ConnectorError>,
}

/// The newest manifest of the site on each relay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteState {
    /// The state of each relay, in the sequence of the configuration.
    pub relays: Vec<RelayState>,
    /// True when each relay answered with the same manifest ID.
    pub consistent: bool,
}

/// True when `a` is newer than `b` (NIP-01): the larger `created_at`, and
/// with the same time the lowest ID.
fn is_newer(a: &Event, b: &Event) -> bool {
    (a.created_at, b.id) > (b.created_at, a.id)
}

/// The newest event of a list.
fn newest_of(events: &[Event]) -> Option<&Event> {
    let mut newest: Option<&Event> = None;
    for event in events {
        if newest.is_none_or(|n| is_newer(event, n)) {
            newest = Some(event);
        }
    }
    newest
}

fn conflict(message: &str) -> ConnectorError {
    ConnectorError::new(ConnectorErrorKind::Conflict, message)
}

/// The manifest path of a Studio path.
fn manifest_path(path: &str) -> String {
    format!("/{path}")
}

/// A server address without a `/` at the end. It must be an http or https
/// address with a host name.
fn parse_server(server: &str) -> Result<String, ConnectorError> {
    let server = server.trim().trim_end_matches('/');
    let host_ok = server
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty());
    if host_ok && nostr_events::server_domain(server).is_ok() {
        Ok(server.to_owned())
    } else {
        Err(invalid("a server address must be an http or https address"))
    }
}

/// A relay address. It must be a ws or wss address with a host name.
fn parse_relay(relay: &str) -> Result<String, ConnectorError> {
    let relay = relay.trim();
    let host_ok = relay
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "ws" | "wss") && !rest.is_empty());
    if host_ok && nostr_events::server_domain(relay).is_ok() {
        Ok(relay.to_owned())
    } else {
        Err(invalid("a relay address must be a ws or wss address"))
    }
}

/// A gateway host name in lowercase.
fn parse_gateway(gateway: &str) -> Result<String, ConnectorError> {
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
        Err(invalid("the gateway must be a host name"))
    }
}

/// True when the refusal message of a relay means that the key is not
/// allowed to publish.
fn is_authorization_refusal(message: &str) -> bool {
    ["auth-required", "blocked", "restricted"]
        .iter()
        .any(|prefix| message.starts_with(prefix))
}

impl NsiteHost {
    /// The kind of an nsite connector row.
    pub const KIND: &'static str = "nsite";

    /// Make a host from the configuration JSON of a row.
    ///
    /// `secret` is the JSON `{"secret_key": "<hex>"}` for the signing
    /// `vault`. `signing` has the events that the user signed.
    ///
    /// # Errors
    ///
    /// Gives `InvalidConfig` when the configuration is not valid, when `d`
    /// is not a valid site name, when the signing is `vault` and the secret
    /// is missing, not valid or not the key of `public_key`. The messages
    /// hold no part of the secret.
    pub fn from_config(
        config: &str,
        secret: Option<&[u8]>,
        signing: Signing,
    ) -> Result<Self, ConnectorError> {
        let config: NsiteConfig = serde_json::from_str(config)
            .map_err(|_| invalid("the nsite configuration is not valid"))?;
        let hex_key = config.public_key.len() == 64
            && config.public_key.bytes().all(|b| b.is_ascii_hexdigit());
        if !hex_key {
            return Err(invalid(
                "the public key must have 64 hexadecimal characters",
            ));
        }
        let public_key = nostr_events::parse_public_key(&config.public_key)?;
        if !nostr_events::valid_site_name(&config.d) {
            return Err(invalid("the site name is not valid"));
        }
        if config.relays.is_empty() {
            return Err(invalid("the nsite connector needs one relay"));
        }
        if config.servers.is_empty() {
            return Err(invalid("the nsite connector needs one server"));
        }
        let relays = config
            .relays
            .iter()
            .map(|relay| parse_relay(relay))
            .collect::<Result<Vec<_>, _>>()?;
        let servers = config
            .servers
            .iter()
            .map(|server| parse_server(server))
            .collect::<Result<Vec<_>, _>>()?;
        let gateway = parse_gateway(config.gateway.as_deref().unwrap_or(DEFAULT_GATEWAY))?;
        let keys = match config.signing {
            SigningMode::Signer => None,
            SigningMode::Vault => {
                let secret = secret.ok_or_else(|| invalid("the nsite connector needs a secret"))?;
                let parsed: NsiteSecret = serde_json::from_slice(secret)
                    .map_err(|_| invalid("the secret must be JSON with secret_key"))?;
                let text = Zeroizing::new(parsed.secret_key);
                let keys = nostr_events::parse_secret_key(&text)?;
                if keys.public_key() != public_key {
                    return Err(invalid("the secret key is not the key of the host"));
                }
                Some(keys)
            }
        };
        Ok(Self {
            public_key,
            d: config.d,
            relays,
            servers,
            gateway,
            title: config.title,
            mode: config.signing,
            keys,
            signing,
            state: Mutex::new(State::default()),
        })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The filter for the manifest of this site.
    fn filter(&self) -> Filter {
        Filter::new()
            .kind(Kind::Custom(KIND_MANIFEST))
            .author(self.public_key)
            .identifier(&self.d)
    }

    /// Read the manifest of the site from each relay.
    fn read_relays(&self) -> Reads {
        relay::newest(&self.relays, &self.filter(), RELAY_TIMEOUT)
    }

    /// Read the newest manifest into the view, at the first need.
    fn load(&self) -> Result<(), ConnectorError> {
        if self.state().loaded {
            return Ok(());
        }
        let newest = self.read_relays().into_newest()?;
        let paths = match &newest {
            Some(event) => Manifest::parse(event)?.paths,
            None => BTreeMap::new(),
        };
        let mut state = self.state();
        if !state.loaded {
            state.loaded = true;
            state.base = newest;
            state.paths = paths;
        }
        Ok(())
    }

    /// The SHA-256 of a path in the view.
    fn sha_of(&self, path: &str) -> Result<Option<String>, ConnectorError> {
        relative_path(path)?;
        self.load()?;
        Ok(self.state().paths.get(&manifest_path(path)).cloned())
    }

    /// The manifest of the view, with no signature.
    fn manifest_of(&self, paths: &BTreeMap<String, String>) -> Manifest {
        Manifest {
            d: self.d.clone(),
            paths: paths.clone(),
            servers: self.servers.clone(),
            title: self.title.clone(),
        }
    }

    /// The `Authorization` header for an upload of one blob to one server.
    ///
    /// The function uses the planned event of the server when it has the
    /// SHA-256. If not, it makes an event for this call.
    fn authorization(&self, index: usize, sha256: &str) -> Result<String, ConnectorError> {
        let planned = {
            let state = self.state();
            state.plan.as_ref().and_then(|plan| {
                plan.uploads
                    .iter()
                    .find(|(i, e)| *i == index && has_x_tag(e, sha256))
                    .map(|(_, e)| e.clone())
            })
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
                AuthAction::Upload,
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

    /// Check `expected` against the path in the view.
    fn check_expected(&self, path: &str, expected: &Expected) -> Result<(), ConnectorError> {
        let sha256 = self.sha_of(path)?;
        match (expected, sha256) {
            (Expected::Absent, None) => Ok(()),
            (Expected::Version(version), Some(sha256))
                if version.as_str() == version_of(&sha256) =>
            {
                Ok(())
            }
            _ => Err(conflict(&format!("{path} changed on the host"))),
        }
    }

    /// Put the bytes on each server and add the path to the view.
    fn put(
        &self,
        path: &str,
        spooled: &Spooled<'_>,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.check_expected(path, expected)?;
        let sha256 = spooled.sha256();
        // Get each header first, so a missing signed event stops the write
        // before the first upload.
        let headers = (0..self.servers.len())
            .map(|index| self.authorization(index, sha256))
            .collect::<Result<Vec<_>, _>>()?;
        let media_type = media_type_of(path);
        let first = &self.servers[0];
        blossom_http::upload(first, spooled, &media_type, &headers[0])?;
        let source = format!("{first}/{sha256}");
        for (index, server) in self.servers.iter().enumerate().skip(1) {
            blossom_http::mirror(server, &source, sha256, &headers[index])?;
        }
        let mut state = self.state();
        state.paths.insert(manifest_path(path), sha256.to_owned());
        state.sizes.insert(sha256.to_owned(), spooled.size());
        state.dirty = true;
        Ok(Version::new(version_of(sha256)))
    }

    /// The size of a blob. It comes from a write, or from `head` on the
    /// servers in sequence. The connector keeps each size that it found.
    fn size_of(&self, path: &str, sha256: &str) -> Result<u64, ConnectorError> {
        if let Some(size) = self.state().sizes.get(sha256) {
            return Ok(*size);
        }
        let mut last_error = None;
        for server in &self.servers {
            match blossom_http::head(server, sha256) {
                Ok(Some(size)) => {
                    self.state().sizes.insert(sha256.to_owned(), size);
                    return Ok(size);
                }
                Ok(None) => {}
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            ConnectorError::new(
                ConnectorErrorKind::Io,
                format!("no server has the blob of {path}"),
            )
        }))
    }

    fn listed(&self, path: &str, sha256: &str) -> Result<ListedFile, ConnectorError> {
        Ok(ListedFile {
            path: path.to_owned(),
            size: self.size_of(path, sha256)?,
            stamp: Stamp::new(version_of(sha256)),
        })
    }

    /// The unsigned manifest that `finish` publishes, and the index of the
    /// planned manifest that it uses.
    ///
    /// With a plan, the function uses the planned manifest. With no plan,
    /// it makes the manifest of the view.
    fn manifest_to_publish(
        &self,
        newest: Option<&Event>,
    ) -> Result<(UnsignedEvent, Option<usize>), ConnectorError> {
        let state = self.state();
        if let Some(plan) = &state.plan {
            let found = plan
                .manifests
                .iter()
                .enumerate()
                .skip(plan.next)
                .find(|(_, (paths, _))| *paths == state.paths);
            return match found {
                Some((index, (_, event))) => Ok((event.clone(), Some(index))),
                None => Err(ConnectorError::new(
                    ConnectorErrorKind::Io,
                    "the write is different from its plan",
                )),
            };
        }
        let time = newest.map_or_else(now, |e| now().max(e.created_at.as_secs() + 1));
        let event = self
            .manifest_of(&state.paths)
            .unsigned(self.public_key, time);
        Ok((event, None))
    }

    /// Publish the manifest to each relay.
    fn publish_to_relays(&self, event: &Event) -> Result<(), ConnectorError> {
        let mut accepted = false;
        let mut only_authorization = true;
        for relay in &self.relays {
            match relay::publish(relay, event, RELAY_TIMEOUT) {
                Ok(answer) if answer.accepted => accepted = true,
                Ok(answer) => {
                    if !is_authorization_refusal(&answer.message) {
                        only_authorization = false;
                    }
                }
                Err(_) => only_authorization = false,
            }
        }
        if accepted {
            Ok(())
        } else if only_authorization {
            Err(ConnectorError::new(
                ConnectorErrorKind::Unauthorized,
                "each relay refused the manifest",
            ))
        } else {
            Err(ConnectorError::new(
                ConnectorErrorKind::Unavailable,
                "no relay accepted the manifest",
            ))
        }
    }

    /// The newest manifest of the site on each relay now.
    ///
    /// # Errors
    ///
    /// Gives no error. A relay that does not answer has its error in its
    /// [`RelayState`].
    pub fn site_state(&self) -> Result<SiteState, ConnectorError> {
        let reads = self.read_relays();
        let relays: Vec<RelayState> = reads
            .results
            .iter()
            .map(|read| RelayState {
                address: read.relay.clone(),
                newest: read.result.as_ref().map_or_else(
                    |error| Err(error.clone()),
                    |events| {
                        Ok(newest_of(events).map(|e| ManifestRef {
                            id: e.id.to_hex(),
                            created_at: e.created_at.as_secs(),
                        }))
                    },
                ),
            })
            .collect();
        let id_of = |r: &RelayState| {
            r.newest
                .as_ref()
                .ok()
                .and_then(|n| n.as_ref())
                .map(|m| m.id.clone())
        };
        let consistent = relays.iter().all(|r| r.newest.is_ok())
            && relays.windows(2).all(|w| id_of(&w[0]) == id_of(&w[1]));
        Ok(SiteState { relays, consistent })
    }

    /// The ID of the newest manifest of each relay, as `finish` found it
    /// after its last publication. `None` means that the relay had no
    /// manifest. The list is empty before the first `finish` that
    /// published.
    #[must_use]
    pub fn relay_manifest_ids(&self) -> Vec<(String, Option<String>)> {
        self.state()
            .relay_ids
            .iter()
            .map(|(relay, id)| (relay.clone(), id.map(|id| id.to_hex())))
            .collect()
    }
}

impl Connector for NsiteHost {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        let Some(sha256) = self.sha_of(path)? else {
            return Ok(None);
        };
        let mut last_error = None;
        for server in &self.servers {
            match blossom_http::get(server, &sha256) {
                Ok(Some(bytes)) => {
                    return Ok(Some(FileRead {
                        bytes,
                        version: Version::new(version_of(&sha256)),
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
        self.sha_of(path)
    }

    fn public_url(&self, path: &str) -> Option<String> {
        Some(format!(
            "https://{}{}.{}/{path}",
            nostr_events::base36_public_key(&self.public_key),
            self.d,
            self.gateway
        ))
    }

    fn finish(&self) -> Result<(), ConnectorError> {
        {
            let state = self.state();
            if !state.loaded || !state.dirty {
                return Ok(());
            }
        }
        // Step 1: the newest manifest must be the manifest of the start.
        let newest = self.read_relays().into_newest()?;
        let started = self.state().base.as_ref().map(|e| e.id);
        if newest.as_ref().map(|e| e.id) != started {
            return Err(conflict("the site changed on the relays"));
        }
        // Steps 2 and 3: make the manifest and sign it.
        let (unsigned, planned_index) = self.manifest_to_publish(newest.as_ref())?;
        let event = nostr_events::sign(
            unsigned,
            self.keys.as_ref(),
            &self.signing,
            &self.public_key,
        )?;
        // Step 4: publish it.
        self.publish_to_relays(&event)?;
        // Step 5: no relay can have a newer manifest of a different client.
        let after = self.read_relays();
        let mut relay_ids = Vec::new();
        let mut newer_found = false;
        for read in &after.results {
            let here = read
                .result
                .as_ref()
                .ok()
                .and_then(|events| newest_of(events));
            if here.is_some_and(|e| e.id != event.id && is_newer(e, &event)) {
                newer_found = true;
            }
            relay_ids.push((read.relay.clone(), here.map(|e| e.id)));
        }
        if newer_found {
            return Err(conflict("a different client published a newer manifest"));
        }
        // Step 6: the manifest is the start of the view.
        let mut state = self.state();
        state.base = Some(event);
        state.dirty = false;
        state.relay_ids = relay_ids;
        if let (Some(plan), Some(index)) = (state.plan.as_mut(), planned_index) {
            plan.next = index + 1;
        }
        Ok(())
    }

    fn plan_events(
        &self,
        changes: &HostChanges,
        plan_time: u64,
    ) -> Result<Vec<PlannedEvent>, ConnectorError> {
        self.load()?;
        let (mut paths, newest_time) = {
            let state = self.state();
            (
                state.paths.clone(),
                state.base.as_ref().map_or(0, |e| e.created_at.as_secs()),
            )
        };
        let user_signs = self.mode == SigningMode::Signer;
        let mut manifests = Vec::new();
        let first_time = plan_time.max(newest_time + 1);
        for write in &changes.writes {
            paths.insert(manifest_path(&write.path), write.sha256.clone());
        }
        // A write with no new file needs no manifest before its deletions.
        if !changes.writes.is_empty() || changes.deletes.is_empty() {
            let unsigned = self
                .manifest_of(&paths)
                .unsigned(self.public_key, first_time);
            manifests.push((paths.clone(), unsigned));
        }
        if !changes.deletes.is_empty() {
            for delete in &changes.deletes {
                paths.remove(&manifest_path(&delete.path));
            }
            let unsigned = self
                .manifest_of(&paths)
                .unsigned(self.public_key, first_time + 1);
            manifests.push((paths, unsigned));
        }
        let mut hashes: Vec<String> = Vec::new();
        for write in &changes.writes {
            if !hashes.contains(&write.sha256) {
                hashes.push(write.sha256.clone());
            }
        }
        let mut uploads = Vec::new();
        if !hashes.is_empty() {
            for (index, server) in self.servers.iter().enumerate() {
                let domain = nostr_events::server_domain(server)?;
                let event = nostr_events::auth_event(
                    self.public_key,
                    AuthAction::Upload,
                    &hashes,
                    &domain,
                    plan_time,
                );
                uploads.push((index, event));
            }
        }
        let events = manifests
            .iter()
            .map(|(_, event)| event.clone())
            .chain(uploads.iter().map(|(_, event)| event.clone()))
            .map(|event| PlannedEvent { event, user_signs })
            .collect();
        let mut state = self.state();
        state.plan = Some(Plan {
            manifests,
            uploads,
            next: 0,
        });
        state.signed.clear();
        Ok(events)
    }

    fn list(&self) -> Result<FileList, ConnectorError> {
        self.load()?;
        let entries: Vec<(String, String)> = self
            .state()
            .paths
            .iter()
            .filter_map(|(path, sha256)| {
                let path = path.strip_prefix('/')?;
                relative_path(path).ok()?;
                Some((path.to_owned(), sha256.clone()))
            })
            .collect();
        let mut list = FileList::default();
        for (path, sha256) in entries {
            let segments: Vec<&str> = path.split('/').collect();
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
            list.files.push(self.listed(&path, &sha256)?);
        }
        Ok(list)
    }

    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        match self.sha_of(path)? {
            Some(sha256) => Ok(Some(self.listed(path, &sha256)?)),
            None => Ok(None),
        }
    }

    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        let Some(sha256) = self.sha_of(path)? else {
            return Err(ConnectorError::new(
                ConnectorErrorKind::NotFound,
                format!("{path} does not exist"),
            ));
        };
        if expected.as_str() != version_of(&sha256) {
            return Err(conflict(&format!("{path} changed on the host")));
        }
        // The connector sends no DELETE to a server (ADR 0048 item 11).
        let mut state = self.state();
        state.paths.remove(&manifest_path(path));
        state.dirty = true;
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
    use crate::testing::relay::TestRelay;
    use crate::vault::OpenVault;
    use std::cell::{Cell, RefCell};

    const SECRET_HEX: &str = "0101010101010101010101010101010101010101010101010101010101010101";

    fn keys() -> Keys {
        parse_secret_key(SECRET_HEX).expect("keys")
    }

    /// Two relays and two Blossom servers, with a new site name for each
    /// host.
    struct Servers {
        relays: Vec<TestRelay>,
        blossoms: Vec<TestBlossom>,
    }

    impl Servers {
        fn start() -> Self {
            Self {
                relays: vec![TestRelay::start(), TestRelay::start()],
                blossoms: vec![TestBlossom::start(), TestBlossom::start()],
            }
        }

        fn config(&self, d: &str, signing: &str) -> String {
            serde_json::json!({
                "public_key": keys().public_key().to_hex(),
                "d": d,
                "relays": self.relays.iter().map(TestRelay::url).collect::<Vec<_>>(),
                "servers": self.blossoms.iter().map(TestBlossom::url).collect::<Vec<_>>(),
                "signing": signing,
            })
            .to_string()
        }

        fn host(&self, d: &str) -> NsiteHost {
            NsiteHost::from_config(
                &self.config(d, "vault"),
                Some(&secret()),
                Signing::default(),
            )
            .expect("a host")
        }

        fn signer_host(&self, d: &str, signing: Signing) -> NsiteHost {
            NsiteHost::from_config(&self.config(d, "signer"), None, signing).expect("a host")
        }
    }

    fn secret() -> Vec<u8> {
        serde_json::json!({ "secret_key": SECRET_HEX })
            .to_string()
            .into_bytes()
    }

    fn kind(result: Result<NsiteHost, ConnectorError>) -> Option<ConnectorErrorKind> {
        result.err().map(|e| e.kind())
    }

    #[test]
    fn the_configuration_is_checked() {
        let servers = Servers::start();
        let make = |config: &str, secret: Option<&[u8]>| {
            NsiteHost::from_config(config, secret, Signing::default())
        };
        assert!(make(&servers.config("abc", "vault"), Some(&secret())).is_ok());
        assert!(make(&servers.config("abc", "signer"), None).is_ok());
        let invalid = Some(ConnectorErrorKind::InvalidConfig);
        assert_eq!(kind(make("{}", None)), invalid);
        assert_eq!(kind(make(&servers.config("abc", "vault"), None)), invalid);
        assert_eq!(kind(make(&servers.config("abc", "other"), None)), invalid);
        let with = |key: &str, value: serde_json::Value| {
            let mut config: serde_json::Value =
                serde_json::from_str(&servers.config("abc", "signer")).unwrap();
            config[key] = value;
            config.to_string()
        };
        for (key, value) in [
            ("relays", serde_json::json!([])),
            ("relays", serde_json::json!(["http://127.0.0.1:1"])),
            ("servers", serde_json::json!([])),
            ("servers", serde_json::json!(["ftp://example.com"])),
            ("public_key", serde_json::json!("abcd")),
            ("gateway", serde_json::json!("bad host")),
        ] {
            assert_eq!(kind(make(&with(key, value), None)), invalid, "{key}");
        }
    }

    #[test]
    fn the_site_name_must_be_valid() {
        let servers = Servers::start();
        for d in ["", "Abc", "abc-", "abcdefghijklmn", "a_b"] {
            let error = NsiteHost::from_config(
                &servers.config(d, "vault"),
                Some(&secret()),
                Signing::default(),
            )
            .expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig, "{d:?}");
        }
    }

    #[test]
    fn a_secret_for_another_key_gives_invalid_config() {
        let servers = Servers::start();
        let other = serde_json::json!({ "secret_key": "02".repeat(32) })
            .to_string()
            .into_bytes();
        let error = NsiteHost::from_config(
            &servers.config("abc", "vault"),
            Some(&other),
            Signing::default(),
        )
        .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        assert!(!error.to_string().contains(&"02".repeat(32)));
    }

    #[test]
    fn the_debug_text_shows_no_secret_key() {
        let servers = Servers::start();
        let text = format!("{:?}", servers.host("abc"));
        assert!(!text.contains(SECRET_HEX));
        assert!(text.contains("NsiteHost"));
    }

    #[test]
    fn from_row_in_makes_an_nsite_host_from_the_vault() {
        let servers = Servers::start();
        let mut vault = OpenVault::create("correct horse battery staple 2026").unwrap();
        vault.put("host-1", &secret()).unwrap();
        let row = ConnectorRow {
            id: ConnectorId(1),
            kind: NsiteHost::KIND.to_owned(),
            name: "Site".to_owned(),
            config: servers.config("abc", "vault"),
            secret_ref: Some("host-1".to_owned()),
        };
        let connector = from_row_in(
            &row,
            &HostEnv {
                vault: Some(&vault),
                blobs: Vec::new(),
                signing: Signing::default(),
            },
        )
        .expect("a connector");
        assert_eq!(connector.read("absent.xml").unwrap(), None);
    }

    #[test]
    fn a_new_site_is_empty() {
        let servers = Servers::start();
        let host = servers.host("abc");
        assert_eq!(host.read("feed.xml").unwrap(), None);
        assert_eq!(host.list().unwrap().files.len(), 0);
    }

    #[test]
    fn no_answer_from_each_relay_gives_unavailable() {
        let servers = Servers::start();
        let config = servers.config("abc", "vault");
        drop(servers);
        let host = NsiteHost::from_config(&config, Some(&secret()), Signing::default()).unwrap();
        let error = host.read("feed.xml").unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    #[test]
    fn two_connectors_on_one_site_see_the_same_file() {
        let servers = Servers::start();
        let a = servers.host("abc");
        a.write("feed.xml", b"<rss/>", &Expected::Absent).unwrap();
        a.finish().unwrap();
        let b = servers.host("abc");
        let read = b.read("feed.xml").unwrap().expect("the file");
        assert_eq!(read.bytes, b"<rss/>");
        assert_eq!(b.list().unwrap().files.len(), 1);
        assert_eq!(b.list().unwrap().files[0].size, 6);
    }

    #[test]
    fn a_connector_that_did_not_finish_changes_no_relay() {
        let servers = Servers::start();
        let a = servers.host("abc");
        a.write("feed.xml", b"<rss/>", &Expected::Absent).unwrap();
        assert!(servers.relays.iter().all(|r| r.received() == 0));
        assert_eq!(servers.host("abc").read("feed.xml").unwrap(), None);
    }

    #[test]
    fn finish_with_no_change_does_nothing() {
        let servers = Servers::start();
        let host = servers.host("abc");
        host.finish().unwrap();
        host.read("feed.xml").unwrap();
        host.finish().unwrap();
        assert!(servers.relays.iter().all(|r| r.received() == 0));
    }

    /// A manifest of a different client, with the time `created_at`.
    fn other_manifest(servers: &Servers, created_at: u64) -> Event {
        let manifest = Manifest {
            d: "abc".to_owned(),
            paths: BTreeMap::new(),
            servers: servers.blossoms.iter().map(TestBlossom::url).collect(),
            title: None,
        };
        sign_with_keys(manifest.unsigned(keys().public_key(), created_at), &keys()).unwrap()
    }

    #[test]
    fn a_newer_manifest_of_a_different_client_gives_conflict_and_no_event() {
        let servers = Servers::start();
        let a = servers.host("abc");
        a.write("feed.xml", b"<rss/>", &Expected::Absent).unwrap();
        let newer = other_manifest(&servers, now() + 1000);
        servers.relays[0].insert(newer.clone());
        let error = a.finish().unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        assert!(servers.relays.iter().all(|r| r.received() == 0));
        assert_eq!(servers.relays[0].events(), vec![newer]);
        assert_eq!(servers.relays[1].events().len(), 0);
    }

    #[test]
    fn a_relay_that_drops_events_gives_ok_and_an_inconsistent_site() {
        let servers = Servers::start();
        let host = servers.host("abc");
        servers.relays[1].drop_events(true);
        host.write("feed.xml", b"<rss/>", &Expected::Absent)
            .unwrap();
        host.finish().unwrap();
        let state = host.site_state().unwrap();
        assert!(!state.consistent);
        assert_eq!(state.relays.len(), 2);
        assert!(matches!(&state.relays[0].newest, Ok(Some(_))));
        assert_eq!(state.relays[1].newest, Ok(None));
        let ids = host.relay_manifest_ids();
        assert!(ids[0].1.is_some() && ids[1].1.is_none());
        servers.relays[1].drop_events(false);
        let other = servers.host("abc");
        assert_eq!(other.read("feed.xml").unwrap().unwrap().bytes, b"<rss/>");
    }

    #[test]
    fn a_site_with_the_same_manifest_on_each_relay_is_consistent() {
        let servers = Servers::start();
        let host = servers.host("abc");
        host.write("feed.xml", b"<rss/>", &Expected::Absent)
            .unwrap();
        host.finish().unwrap();
        let state = host.site_state().unwrap();
        assert!(state.consistent);
        let ids: Vec<_> = state
            .relays
            .iter()
            .map(|r| r.newest.clone().unwrap().unwrap().id)
            .collect();
        assert_eq!(ids[0], ids[1]);
    }

    #[test]
    fn a_relay_that_refuses_with_blocked_gives_unauthorized() {
        let servers = Servers::start();
        let host = servers.host("abc");
        host.write("feed.xml", b"<rss/>", &Expected::Absent)
            .unwrap();
        for relay in &servers.relays {
            relay.refuse_events(Some("blocked: no"));
        }
        assert_eq!(
            host.finish().unwrap_err().kind(),
            ConnectorErrorKind::Unauthorized
        );
        servers.relays[1].refuse_events(Some("error: busy"));
        assert_eq!(
            host.finish().unwrap_err().kind(),
            ConnectorErrorKind::Unavailable
        );
        servers.relays[1].refuse_events(None);
        host.finish().unwrap();
    }

    #[test]
    fn a_delete_sends_no_delete_to_a_server() {
        let servers = Servers::start();
        let host = servers.host("abc");
        host.write("a.mp3", b"abc", &Expected::Absent).unwrap();
        host.finish().unwrap();
        let stamp = host.stat("a.mp3").unwrap().unwrap().stamp;
        host.delete("a.mp3", &stamp).unwrap();
        host.finish().unwrap();
        assert!(servers.blossoms.iter().all(|b| b.deletes().is_empty()));
        let sha = sha256_hex(&[b"abc"]);
        assert!(
            servers
                .blossoms
                .iter()
                .all(|b| b.blobs() == vec![sha.clone()])
        );
        assert_eq!(servers.host("abc").read("a.mp3").unwrap(), None);
    }

    #[test]
    fn the_public_url_has_the_base36_key_the_site_name_and_the_gateway() {
        let config = serde_json::json!({
            "public_key": "fa984bd7dbb282f07e16e7ae87b26a2a7b9b90b7246a44771f0cf5ae58018f52",
            "d": "abc",
            "relays": ["ws://127.0.0.1:1"],
            "servers": ["http://127.0.0.1:1"],
            "signing": "signer",
        });
        let host = NsiteHost::from_config(&config.to_string(), None, Signing::default()).unwrap();
        assert_eq!(
            host.public_url("a/b.mp3").as_deref(),
            Some("https://68ukf094zwffw6sq71wn17fst37v9rcdz4uimbb2s5gor83u4yabc.nsite.lol/a/b.mp3")
        );
        let mut config = config;
        config["gateway"] = serde_json::json!("Gateway.Example");
        let host = NsiteHost::from_config(&config.to_string(), None, Signing::default()).unwrap();
        assert!(
            host.public_url("a.mp3")
                .unwrap()
                .ends_with("abc.gateway.example/a.mp3")
        );
    }

    #[test]
    fn the_signer_without_a_plan_gives_signature_missing() {
        let servers = Servers::start();
        let host = servers.signer_host("abc", Signing::default());
        let error = host
            .write("one.mp3", b"abc", &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::SignatureMissing);
        assert!(servers.blossoms.iter().all(|b| b.blobs().is_empty()));
        assert_eq!(host.read("one.mp3").unwrap(), None);
    }

    fn changes_for(path: &str, bytes: &[u8]) -> HostChanges {
        HostChanges {
            writes: vec![PlannedWrite {
                path: path.to_owned(),
                sha256: sha256_hex(&[bytes]),
                size: bytes.len() as u64,
            }],
            deletes: Vec::new(),
        }
    }

    #[test]
    fn the_signer_uses_the_events_that_the_user_signed() {
        let servers = Servers::start();
        let changes = changes_for("one.mp3", b"abc");
        let plan_time = now();
        let first_host = servers.signer_host("abc", Signing::default());
        let planned = first_host.plan_events(&changes, plan_time).unwrap();
        // One manifest and one upload event for each of two servers.
        assert_eq!(planned.len(), 3);
        assert!(planned.iter().all(|p| p.user_signs));
        let signed = planned
            .iter()
            .map(|p| sign_with_keys(p.event.clone(), &keys()).unwrap())
            .collect();
        let host = servers.signer_host(
            "abc",
            Signing {
                plan_time: Some(plan_time),
                signed,
            },
        );
        host.plan_events(&changes, plan_time).unwrap();
        host.write("one.mp3", b"abc", &Expected::Absent).unwrap();
        host.finish().unwrap();
        let read = servers.host("abc").read("one.mp3").unwrap().unwrap();
        assert_eq!(read.bytes, b"abc");
    }

    #[test]
    fn a_write_that_is_not_the_plan_gives_io_and_publishes_nothing() {
        let servers = Servers::start();
        let host = servers.host("abc");
        host.plan_events(&changes_for("one.mp3", b"abc"), now())
            .unwrap();
        host.write("two.mp3", b"abc", &Expected::Absent).unwrap();
        let error = host.finish().unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
        assert_eq!(error.to_string(), "the write is different from its plan");
        assert!(servers.relays.iter().all(|r| r.received() == 0));
    }

    #[test]
    fn a_plan_with_an_old_time_gives_a_manifest_newer_than_the_newest() {
        let servers = Servers::start();
        let future = now() + 5000;
        let newest = other_manifest(&servers, future);
        servers.relays[0].insert(newest);
        let host = servers.host("abc");
        let planned = host
            .plan_events(&changes_for("one.mp3", b"abc"), 100)
            .unwrap();
        let manifest = &planned[0].event;
        assert_eq!(manifest.kind, Kind::Custom(KIND_MANIFEST));
        assert_eq!(manifest.created_at.as_secs(), future + 1);
    }

    #[test]
    fn a_plan_with_a_deletion_has_two_manifests_and_finish_publishes_both() {
        let servers = Servers::start();
        let first = servers.host("abc");
        first.write("old.mp3", b"old", &Expected::Absent).unwrap();
        first.finish().unwrap();
        let changes = HostChanges {
            writes: changes_for("new.mp3", b"new").writes,
            deletes: vec![PlannedDelete {
                path: "old.mp3".to_owned(),
                sha256: None,
            }],
        };
        let host = servers.host("abc");
        let planned = host.plan_events(&changes, now()).unwrap();
        // Two manifests and one upload event for each of two servers.
        assert_eq!(planned.len(), 4);
        assert_eq!(
            planned[1].event.created_at.as_secs(),
            planned[0].event.created_at.as_secs() + 1
        );
        let before = servers.relays[0].received();
        host.write("new.mp3", b"new", &Expected::Absent).unwrap();
        host.finish().unwrap();
        let stamp = host.stat("old.mp3").unwrap().unwrap().stamp;
        host.delete("old.mp3", &stamp).unwrap();
        host.finish().unwrap();
        assert_eq!(servers.relays[0].received() - before, 2);
        let reader = servers.host("abc");
        let files: Vec<String> = reader
            .list()
            .unwrap()
            .files
            .into_iter()
            .map(|f| f.path)
            .collect();
        assert_eq!(files, ["new.mp3"]);
        assert!(servers.blossoms.iter().all(|b| b.deletes().is_empty()));
    }

    #[test]
    fn the_manifest_has_the_servers_the_title_and_the_site_name() {
        let servers = Servers::start();
        let mut config: serde_json::Value =
            serde_json::from_str(&servers.config("abc", "vault")).unwrap();
        config["title"] = serde_json::json!("My music");
        let host = NsiteHost::from_config(&config.to_string(), Some(&secret()), Signing::default())
            .unwrap();
        host.write("a/b.mp3", b"abc", &Expected::Absent).unwrap();
        host.finish().unwrap();
        let event = servers.relays[0].events().remove(0);
        let manifest = Manifest::parse(&event).unwrap();
        assert_eq!(manifest.d, "abc");
        assert_eq!(manifest.title.as_deref(), Some("My music"));
        assert_eq!(
            manifest.servers,
            servers
                .blossoms
                .iter()
                .map(TestBlossom::url)
                .collect::<Vec<_>>()
        );
        assert_eq!(manifest.paths.get("/a/b.mp3"), Some(&sha256_hex(&[b"abc"])));
    }

    #[test]
    fn the_size_comes_from_the_servers_for_a_path_of_the_manifest() {
        let servers = Servers::start();
        let writer = servers.host("abc");
        writer.write("a.mp3", b"abcd", &Expected::Absent).unwrap();
        writer.finish().unwrap();
        let reader = servers.host("abc");
        assert_eq!(reader.stat("a.mp3").unwrap().unwrap().size, 4);
        assert_eq!(reader.stat("absent.mp3").unwrap(), None);
    }

    /// Do a check of the shared suite. Each connector has a new site on
    /// new servers.
    fn suite_check(check: fn(&dyn Fn() -> Box<dyn Connector>)) {
        let started: RefCell<Vec<Servers>> = RefCell::new(Vec::new());
        let counter = Cell::new(0_u32);
        check(&|| {
            let servers = Servers::start();
            counter.set(counter.get() + 1);
            let host = servers.host(&format!("site-{}", counter.get()));
            started.borrow_mut().push(servers);
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
