//! Nostr events for the nsite and Blossom connectors.
//!
//! `nostr` has its default features only (ADR 0044, ADR 0051). The core
//! signs an event in the one function of this module.

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use nostr::nips::nip19::FromBech32;
use nostr::prelude::{Event, Keys, Kind, PublicKey, SecretKey, Tag, Timestamp, UnsignedEvent};
use secp256k1::Secp256k1;

use crate::connector::{ConnectorError, ConnectorErrorKind};
use crate::digest::sha256_hex;

/// The kind of the nsite named site manifest (NIP-5A).
const KIND_MANIFEST: u16 = 35128;
/// The kind of the Blossom authorization event (BUD-11).
const KIND_AUTH: u16 = 24242;
/// How long a Blossom authorization event is valid, in seconds.
const AUTH_LIFETIME: u64 = 3600;

/// Sign an event with a key.
///
/// The function uses 32 random bytes from the operating system as the
/// auxiliary data of BIP-340. It signs the event ID.
///
/// # Errors
///
/// Gives `Io` when the random source is not available, or when the event
/// does not pass the check of its ID and signature. The message holds no
/// part of the key.
pub fn sign_with_keys(unsigned: UnsignedEvent, keys: &Keys) -> Result<Event, ConnectorError> {
    let secp = Secp256k1::signing_only();
    let mut aux = [0_u8; 32];
    getrandom::fill(&mut aux).map_err(|_| {
        ConnectorError::new(ConnectorErrorKind::Io, "the random source is not available")
    })?;
    let id = unsigned.compute_id();
    let signature = keys.sign_schnorr_with_aux_rand(&secp, id.as_bytes(), &aux);
    unsigned.add_signature(signature).map_err(|_| {
        ConnectorError::new(ConnectorErrorKind::Io, "the event signature is not valid")
    })
}

/// Parse a secret key from an `nsec` or from 64 hexadecimal characters.
///
/// # Errors
///
/// Gives `InvalidConfig` for other text. The message holds no part of the
/// text.
pub fn parse_secret_key(text: &str) -> Result<Keys, ConnectorError> {
    let text = text.trim();
    let secret = if text.starts_with("nsec1") {
        SecretKey::from_bech32(text)
    } else if is_hex64(text) {
        SecretKey::from_hex(text)
    } else {
        return Err(bad_secret());
    };
    secret.map(Keys::new).map_err(|_| bad_secret())
}

fn bad_secret() -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::InvalidConfig,
        "the secret key is not an nsec or 64 hexadecimal characters",
    )
}

/// Parse a public key from an `npub` or from 64 hexadecimal characters.
///
/// # Errors
///
/// Gives `InvalidConfig` for other text.
pub fn parse_public_key(text: &str) -> Result<PublicKey, ConnectorError> {
    let text = text.trim();
    let key = if text.starts_with("npub1") {
        PublicKey::from_bech32(text)
    } else if is_hex64(text) {
        PublicKey::from_hex(text)
    } else {
        return Err(bad_public());
    };
    key.map_err(|_| bad_public())
}

fn bad_public() -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::InvalidConfig,
        "the public key is not an npub or 64 hexadecimal characters",
    )
}

fn is_hex64(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The public key as 50 characters of base36 (NIP-5A named site host name).
///
/// The text has `0` at the start when necessary.
#[must_use]
pub fn base36_public_key(key: &PublicKey) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut number = key.to_bytes();
    let mut out = [b'0'; 50];
    for slot in out.iter_mut().rev() {
        // Divide the big-endian number by 36. The rest is the next digit.
        let mut rest = 0_u32;
        for byte in &mut number {
            let value = (rest << 8) | u32::from(*byte);
            *byte = u8::try_from(value / 36).unwrap_or(0);
            rest = value % 36;
        }
        *slot = DIGITS[rest as usize];
    }
    out.iter().map(|&b| char::from(b)).collect()
}

/// True when `d` is a valid named site identifier (NIP-5A).
///
/// It has 1 to 13 characters of `[a-z0-9-]`, and no `-` at the end.
#[must_use]
pub fn valid_site_name(d: &str) -> bool {
    (1..=13).contains(&d.len())
        && d.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !d.ends_with('-')
}

/// The aggregate hash of the paths of a site (NIP-5A).
///
/// `paths` maps each absolute path to the SHA-256 text of its file.
#[must_use]
pub fn aggregate_hash(paths: &BTreeMap<String, String>) -> String {
    let mut lines: Vec<String> = paths
        .iter()
        .map(|(path, hash)| format!("{hash} {path}\n"))
        .collect();
    lines.sort();
    let joined = lines.concat();
    sha256_hex(&[joined.as_bytes()])
}

/// The manifest of an nsite named site (kind 35128).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The site identifier.
    pub d: String,
    /// The absolute path of each file, with the SHA-256 text of the file.
    pub paths: BTreeMap<String, String>,
    /// The Blossom servers that hold the files.
    pub servers: Vec<String>,
    /// The title of the site.
    pub title: Option<String>,
}

impl Manifest {
    /// Make the unsigned manifest event.
    ///
    /// The tags are `d`, `title`, one `server` for each server, one `path`
    /// for each path, and the `x` aggregate tag.
    #[must_use]
    pub fn unsigned(&self, public_key: PublicKey, created_at: u64) -> UnsignedEvent {
        let mut tags = vec![Tag::identifier(&self.d)];
        if let Some(title) = &self.title {
            tags.extend(Tag::parse(["title", title.as_str()]));
        }
        for server in &self.servers {
            tags.extend(Tag::parse(["server", server.as_str()]));
        }
        for (path, hash) in &self.paths {
            tags.extend(Tag::parse(["path", path.as_str(), hash.as_str()]));
        }
        let aggregate = aggregate_hash(&self.paths);
        tags.extend(Tag::parse(["x", aggregate.as_str(), "aggregate"]));
        UnsignedEvent::new(
            public_key,
            Timestamp::from(created_at),
            Kind::Custom(KIND_MANIFEST),
            tags,
            "",
        )
    }

    /// Read the manifest from an event.
    ///
    /// # Errors
    ///
    /// Gives `Io` when a `path` tag has a path that does not start with
    /// `/`, or a value that is not 64 lowercase hexadecimal characters.
    pub fn parse(event: &Event) -> Result<Self, ConnectorError> {
        let mut manifest = Self {
            d: event.tags.identifier().unwrap_or_default(),
            paths: BTreeMap::new(),
            servers: Vec::new(),
            title: None,
        };
        for tag in event.tags.iter() {
            let parts = tag.as_slice();
            match (tag.kind(), parts.get(1), parts.get(2)) {
                ("path", Some(path), Some(hash)) => {
                    let hash_ok = hash.len() == 64
                        && hash
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
                    if !path.starts_with('/') || !hash_ok {
                        return Err(ConnectorError::new(
                            ConnectorErrorKind::Io,
                            "a path tag of the manifest is not valid",
                        ));
                    }
                    manifest.paths.insert(path.clone(), hash.clone());
                }
                ("path", _, _) => {
                    return Err(ConnectorError::new(
                        ConnectorErrorKind::Io,
                        "a path tag of the manifest is not valid",
                    ));
                }
                ("server", Some(server), _) => manifest.servers.push(server.clone()),
                ("title", Some(title), _) if manifest.title.is_none() => {
                    manifest.title = Some(title.clone());
                }
                _ => {}
            }
        }
        Ok(manifest)
    }
}

/// The action that a Blossom authorization event allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthAction {
    /// Upload a blob.
    Upload,
    /// Delete a blob.
    Delete,
}

/// Make the unsigned Blossom authorization event (BUD-11, kind 24242).
///
/// The event is valid for one hour.
#[must_use]
pub fn auth_event(
    public_key: PublicKey,
    action: AuthAction,
    sha256s: &[String],
    server_domain: &str,
    created_at: u64,
) -> UnsignedEvent {
    let (verb, content) = match action {
        AuthAction::Upload => ("upload", "Upload with Music Index Studio"),
        AuthAction::Delete => ("delete", "Delete with Music Index Studio"),
    };
    let mut tags: Vec<Tag> = Tag::parse(["t", verb]).into_iter().collect();
    for hash in sha256s {
        tags.extend(Tag::parse(["x", hash.as_str()]));
    }
    tags.extend(Tag::parse(["server", server_domain]));
    let expiration = Timestamp::from(created_at.saturating_add(AUTH_LIFETIME));
    tags.push(Tag::expiration(expiration));
    UnsignedEvent::new(
        public_key,
        Timestamp::from(created_at),
        Kind::Custom(KIND_AUTH),
        tags,
        content,
    )
}

/// The host name of a URL in lowercase, with no port (BUD-11).
///
/// # Errors
///
/// Gives `InvalidConfig` when the URL has no host name.
pub fn server_domain(url: &str) -> Result<String, ConnectorError> {
    let bad = || {
        ConnectorError::new(
            ConnectorErrorKind::InvalidConfig,
            "the URL has no host name",
        )
    };
    let (_, rest) = url.split_once("://").ok_or_else(bad)?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(inner) = host_port.strip_prefix('[') {
        inner.split(']').next().unwrap_or_default()
    } else {
        host_port.split(':').next().unwrap_or_default()
    };
    if host.is_empty() {
        return Err(bad());
    }
    Ok(host.to_ascii_lowercase())
}

/// The `Authorization` header value of a signed event (BUD-11).
#[must_use]
pub fn auth_header(event: &Event) -> String {
    format!("Nostr {}", URL_SAFE_NO_PAD.encode(event.as_json()))
}

/// The events that the user signed for one write (ADR 0050 item 4).
#[derive(Debug, Clone, Default)]
pub struct Signing {
    /// The time that the plan used for its events.
    pub plan_time: Option<u64>,
    /// The events that the user signed.
    pub signed: Vec<Event>,
}

/// Sign an event with the keys, or find the event that the user signed.
///
/// With `keys`, the function signs the event. With no `keys`, it finds the
/// event with the same ID in `signing.signed`.
///
/// # Errors
///
/// Gives `Unauthorized` when the keys or the signed event are for a
/// different public key than the host key, or the signature is not
/// correct. Gives `SignatureMissing` when no keys and no signed event are
/// available.
pub fn sign(
    unsigned: UnsignedEvent,
    keys: Option<&Keys>,
    signing: &Signing,
    host_key: &PublicKey,
) -> Result<Event, ConnectorError> {
    let wrong_key = || {
        ConnectorError::new(
            ConnectorErrorKind::Unauthorized,
            "the key is not the key of the host",
        )
    };
    if let Some(keys) = keys {
        if keys.public_key() != unsigned.pubkey || unsigned.pubkey != *host_key {
            return Err(wrong_key());
        }
        return sign_with_keys(unsigned, keys);
    }
    let id = unsigned.compute_id();
    let Some(event) = signing.signed.iter().find(|e| e.id == id) else {
        return Err(ConnectorError::new(
            ConnectorErrorKind::SignatureMissing,
            "the write needs an event that the user did not sign",
        ));
    };
    if event.pubkey != *host_key {
        return Err(wrong_key());
    }
    if event.verify().is_err() {
        return Err(ConnectorError::new(
            ConnectorErrorKind::Unauthorized,
            "the signature of the event is not correct",
        ));
    }
    Ok(event.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::nips::nip19::ToBech32;
    use nostr::prelude::{Kind, SecretKey, Tag, Timestamp};

    fn keys() -> Keys {
        Keys::new(SecretKey::from_slice(&[1; 32]).expect("a secret key"))
    }

    #[test]
    fn a_signed_event_passes_the_check() {
        let keys = keys();
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(1_700_000_000_u64),
            Kind::Custom(35128),
            [Tag::identifier("site")],
            "",
        );
        let id = unsigned.compute_id();
        let event = sign_with_keys(unsigned, &keys).expect("a signed event");
        assert!(event.verify().is_ok());
        assert_eq!(event.id, id);
        assert_eq!(event.pubkey, keys.public_key());
    }

    const HEX_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEX_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn pk(hex: &str) -> PublicKey {
        PublicKey::from_hex(hex).expect("a public key")
    }

    #[test]
    fn base36_matches_the_test_vectors() {
        let cases = [
            (
                "fa984bd7dbb282f07e16e7ae87b26a2a7b9b90b7246a44771f0cf5ae58018f52",
                "68ukf094zwffw6sq71wn17fst37v9rcdz4uimbb2s5gor83u4y",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000001",
                "00000000000000000000000000000000000000000000000001",
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                "6dp5qcb22im238nr3wvp0ic7q99w035jmy2iw7i6n43d37jtof",
            ),
        ];
        for (hex, base36) in cases {
            assert_eq!(base36_public_key(&pk(hex)), base36);
        }
    }

    #[test]
    fn the_aggregate_hash_matches_the_test_vector() {
        let paths = BTreeMap::from([
            ("/feed.xml".to_owned(), HEX_A.to_owned()),
            ("/album/01.mp3".to_owned(), HEX_B.to_owned()),
        ]);
        assert_eq!(
            aggregate_hash(&paths),
            "ab4e9f81e87c04126edc5f356e57f628b9fce3d9a615924cad70cb8000902921"
        );
    }

    #[test]
    fn site_names_follow_nip_5a() {
        assert!(valid_site_name("music"));
        assert!(valid_site_name("a-b-1"));
        assert!(valid_site_name("abcdefghijklm"));
        assert!(!valid_site_name(""));
        assert!(!valid_site_name("abcdefghijklmn"));
        assert!(!valid_site_name("abc-"));
        assert!(!valid_site_name("Abc"));
        assert!(!valid_site_name("a_b"));
    }

    #[test]
    fn keys_parse_from_bech32_and_hex() {
        let hex = "0101010101010101010101010101010101010101010101010101010101010101";
        let from_hex = parse_secret_key(hex).expect("hex keys");
        assert_eq!(from_hex.public_key(), keys().public_key());
        let nsec = from_hex.secret_key().to_bech32().expect("an nsec");
        let from_nsec = parse_secret_key(&nsec).expect("nsec keys");
        assert_eq!(from_nsec.public_key(), keys().public_key());
        let npub = keys().public_key().to_bech32().expect("an npub");
        assert_eq!(parse_public_key(&npub).expect("npub"), keys().public_key());
        assert_eq!(
            parse_public_key(&keys().public_key().to_hex()).expect("hex"),
            keys().public_key()
        );
    }

    #[test]
    fn a_bad_secret_key_gives_a_message_without_the_text() {
        let text = "not-a-key-secret-text";
        let error = parse_secret_key(text).expect_err("refused");
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        assert!(!error.to_string().contains(text));
        assert_eq!(
            parse_public_key("xyz").expect_err("refused").kind(),
            ConnectorErrorKind::InvalidConfig
        );
    }

    fn manifest() -> Manifest {
        Manifest {
            d: "music".to_owned(),
            paths: BTreeMap::from([
                ("/feed.xml".to_owned(), HEX_A.to_owned()),
                ("/album/01.mp3".to_owned(), HEX_B.to_owned()),
            ]),
            servers: vec![
                "https://one.example".to_owned(),
                "https://two.example".to_owned(),
            ],
            title: Some("My music".to_owned()),
        }
    }

    #[test]
    fn a_manifest_goes_through_signing_and_parse() {
        let keys = keys();
        let manifest = manifest();
        let unsigned = manifest.unsigned(keys.public_key(), 1_700_000_000);
        assert_eq!(unsigned.kind, Kind::Custom(35128));
        let kinds: Vec<&str> = unsigned.tags.iter().map(Tag::kind).collect();
        assert_eq!(
            kinds,
            ["d", "title", "server", "server", "path", "path", "x"]
        );
        let event = sign(
            unsigned,
            Some(&keys),
            &Signing::default(),
            &keys.public_key(),
        )
        .expect("a signed event");
        assert_eq!(Manifest::parse(&event).expect("a manifest"), manifest);
    }

    #[test]
    fn a_manifest_with_a_bad_path_tag_gives_io() {
        let keys = keys();
        for tag in [
            ["path", "relative.txt", HEX_A],
            ["path", "/a.txt", "ABCDEF"],
        ] {
            let unsigned = UnsignedEvent::new(
                keys.public_key(),
                Timestamp::from(1_u64),
                Kind::Custom(35128),
                [Tag::identifier("music"), Tag::parse(tag).expect("a tag")],
                "",
            );
            let event = sign_with_keys(unsigned, &keys).expect("signed");
            let error = Manifest::parse(&event).expect_err("refused");
            assert_eq!(error.kind(), ConnectorErrorKind::Io);
        }
    }

    #[test]
    fn an_authorization_event_has_the_bud_11_tags() {
        let keys = keys();
        let hashes = vec![HEX_A.to_owned(), HEX_B.to_owned()];
        let unsigned = auth_event(
            keys.public_key(),
            AuthAction::Upload,
            &hashes,
            "blossom.example",
            1000,
        );
        assert_eq!(unsigned.kind, Kind::Custom(24242));
        assert_eq!(unsigned.content, "Upload with Music Index Studio");
        let tags: Vec<Vec<String>> = unsigned
            .tags
            .iter()
            .map(|t| t.as_slice().to_vec())
            .collect();
        assert_eq!(
            tags,
            vec![
                vec!["t", "upload"],
                vec!["x", HEX_A],
                vec!["x", HEX_B],
                vec!["server", "blossom.example"],
                vec!["expiration", "4600"],
            ]
        );
        let delete = auth_event(keys.public_key(), AuthAction::Delete, &[], "b.example", 0);
        assert_eq!(delete.content, "Delete with Music Index Studio");
        assert_eq!(
            delete.tags.iter().next().expect("a tag").as_slice()[1],
            "delete"
        );
    }

    #[test]
    fn server_domain_gives_a_lowercase_host_name_with_no_port() {
        assert_eq!(
            server_domain("https://Blossom.Example:8443/upload?x=1").expect("a host"),
            "blossom.example"
        );
        assert_eq!(
            server_domain("http://user@host.test").expect("a host"),
            "host.test"
        );
        assert_eq!(server_domain("http://[::1]:80/").expect("a host"), "::1");
        assert!(server_domain("no-scheme").is_err());
        assert!(server_domain("https:///x").is_err());
    }

    #[test]
    fn auth_header_is_base64url_json_without_padding() {
        let keys = keys();
        let unsigned = auth_event(keys.public_key(), AuthAction::Upload, &[], "a.test", 5);
        let event = sign_with_keys(unsigned, &keys).expect("signed");
        let header = auth_header(&event);
        let encoded = header.strip_prefix("Nostr ").expect("the prefix");
        assert!(!encoded.contains(['=', '+', '/']));
        let json = URL_SAFE_NO_PAD.decode(encoded).expect("Base64url");
        let back: Event = serde_json::from_slice(&json).expect("an event");
        assert_eq!(back, event);
    }

    fn unsigned_for(keys: &Keys) -> UnsignedEvent {
        auth_event(keys.public_key(), AuthAction::Upload, &[], "a.test", 5)
    }

    #[test]
    fn sign_uses_keys_of_the_host_only() {
        let keys = keys();
        let other = Keys::new(SecretKey::from_slice(&[2; 32]).expect("a key"));
        let signing = Signing::default();
        let event = sign(
            unsigned_for(&keys),
            Some(&keys),
            &signing,
            &keys.public_key(),
        )
        .expect("signed");
        assert!(event.verify().is_ok());
        let error = sign(
            unsigned_for(&other),
            Some(&other),
            &signing,
            &keys.public_key(),
        )
        .expect_err("refused");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
    }

    #[test]
    fn sign_without_keys_needs_a_signed_event() {
        let keys = keys();
        let other = Keys::new(SecretKey::from_slice(&[2; 32]).expect("a key"));
        let host = keys.public_key();
        let missing =
            sign(unsigned_for(&keys), None, &Signing::default(), &host).expect_err("missing");
        assert_eq!(missing.kind(), ConnectorErrorKind::SignatureMissing);

        let good = sign_with_keys(unsigned_for(&keys), &keys).expect("signed");
        let signing = Signing {
            plan_time: Some(5),
            signed: vec![good.clone()],
        };
        assert_eq!(
            sign(unsigned_for(&keys), None, &signing, &host).expect("found"),
            good
        );

        let foreign = sign_with_keys(unsigned_for(&other), &other).expect("signed");
        let signing = Signing {
            plan_time: None,
            signed: vec![foreign],
        };
        let error = sign(unsigned_for(&other), None, &signing, &host).expect_err("refused");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);

        let mut bad = good;
        bad.content = "changed".to_owned();
        let signing = Signing {
            plan_time: None,
            signed: vec![bad],
        };
        let error = sign(unsigned_for(&keys), None, &signing, &host).expect_err("refused");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
    }
}
