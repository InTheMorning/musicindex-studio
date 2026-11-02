//! Connected apps (ADR 0054 items 7 to 10, ADR 0110).
//!
//! A connected app is a program that calls the `/app/v1/` routes. It has its
//! own secp256k1 key pair, and it signs each request. Studio never sees the
//! private key. The user makes an app with a name. Studio then gives a
//! pairing code one time, and the app pairs with its public key and its live
//! value origin. The collection keeps only the SHA-256 of the code.

use nostr::prelude::{Event, PublicKey, ToBech32};
use rusqlite::OptionalExtension;
use serde::Serialize;
use zeroize::Zeroizing;

use crate::collection::{Collection, CollectionError, CollectionErrorKind, now};
use crate::digest::sha256_hex;
use crate::live::normalize_origin;
use crate::signed_request::{self, ExpectedRequest, SignedRequestError};

/// The time that a pairing code works, in seconds (ADR 0110 item 8).
pub const CODE_LIFETIME: i64 = 15 * 60;
/// The time that Studio keeps an accepted event ID, in seconds (ADR 0110
/// item 15).
pub const REPLAY_WINDOW: i64 = 120;
/// The most seconds that an event can be earlier than the latest event
/// that Studio accepted from the same app (ADR 0110 item 13).
pub const STALE_LIMIT: i64 = 60;

/// The 32 characters of Crockford base32 (ADR 0110 item 6).
const CODE_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// The number of characters of a pairing code.
const CODE_LENGTH: usize = 8;

/// The state of a connected app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppState {
    /// The user made the app, and it did not pair yet.
    Waiting,
    /// The app paired. Studio accepts the requests of its key.
    Active,
    /// The app is from before ADR 0110. It has a token that Studio refuses,
    /// and the user must connect it again (ADR 0110 item 37).
    Token,
}

impl AppState {
    /// The text that the collection keeps.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Active => "active",
            Self::Token => "token",
        }
    }

    fn parse(text: &str) -> Self {
        match text {
            "waiting" => Self::Waiting,
            "active" => Self::Active,
            _ => Self::Token,
        }
    }
}

/// One connected app. It has no code and no code hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectedApp {
    /// The ID of the app.
    pub id: i64,
    /// The name that the user gave.
    pub name: String,
    /// The time of creation, in seconds since the Unix epoch.
    pub created_at: i64,
    /// The state of the app.
    pub state: AppState,
    /// The public key of the app as 64 lowercase hexadecimal characters.
    /// It is not secret, and the JSON form of the app does not have it.
    #[serde(skip)]
    pub pubkey: Option<String>,
    /// The time when the pairing code of a waiting app ends. After a
    /// pairing, the code stays known for the same time, so that a second
    /// pairing with it gives `app_code_used` (ADR 0110 item 11).
    pub code_expires_at: Option<i64>,
    /// The live value origin that the app gave at the pairing.
    pub live_value_origin: Option<String>,
    /// The time of the last accepted request.
    pub last_used_at: Option<i64>,
    /// The client address of the last accepted request.
    pub last_address: Option<String>,
}

const APP_COLUMNS: &str = "id, name, created_at, state, pubkey, code_expires_at, \
     live_value_origin, last_used_at, last_address";

fn app_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConnectedApp> {
    let state: String = row.get(3)?;
    Ok(ConnectedApp {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        state: AppState::parse(&state),
        pubkey: row.get(4)?,
        code_expires_at: row.get(5)?,
        live_value_origin: row.get(6)?,
        last_used_at: row.get(7)?,
        last_address: row.get(8)?,
    })
}

/// The kind of a [`PairError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PairErrorKind {
    /// No waiting app has the code, or the code ended (`app_code_invalid`).
    CodeInvalid,
    /// An app paired with the code in the last 15 minutes
    /// (`app_code_used`).
    CodeUsed,
    /// The public key is not a valid secp256k1 key (`app_key_invalid`).
    KeyInvalid,
    /// An app in this collection has the key already (`app_key_in_use`).
    KeyInUse,
    /// The live value origin is not `https` or `wss` with a host and no
    /// path (`live_value_origin_invalid`).
    OriginInvalid,
    /// SQLite or the random source failed (`internal`).
    Database,
}

/// An error of [`pair`].
#[derive(Debug)]
pub struct PairError {
    kind: PairErrorKind,
    message: String,
}

impl PairError {
    fn new(kind: PairErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> PairErrorKind {
        self.kind
    }

    /// The API error code of the error.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self.kind {
            PairErrorKind::CodeInvalid => "app_code_invalid",
            PairErrorKind::CodeUsed => "app_code_used",
            PairErrorKind::KeyInvalid => "app_key_invalid",
            PairErrorKind::KeyInUse => "app_key_in_use",
            PairErrorKind::OriginInvalid => "live_value_origin_invalid",
            PairErrorKind::Database => "internal",
        }
    }
}

impl std::fmt::Display for PairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PairError {}

impl From<CollectionError> for PairError {
    fn from(error: CollectionError) -> Self {
        Self::new(PairErrorKind::Database, error.to_string())
    }
}

impl From<rusqlite::Error> for PairError {
    fn from(error: rusqlite::Error) -> Self {
        Self::new(PairErrorKind::Database, error.to_string())
    }
}

/// Apply the rules of ADR 0110 item 9 to the text of a pairing code. The
/// function ignores spaces and hyphens, makes the letters uppercase, reads
/// `O` as `0`, and reads `I` and `L` as `1`. It gives the 8 characters of
/// the code, or `None` when the text is not a code.
#[must_use]
pub fn normalize_code(text: &str) -> Option<Zeroizing<String>> {
    let mut code = Zeroizing::new(String::with_capacity(CODE_LENGTH));
    for c in text.chars() {
        if c == '-' || c.is_whitespace() {
            continue;
        }
        let c = match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            other => other,
        };
        let known = u8::try_from(c).is_ok_and(|b| CODE_ALPHABET.contains(&b));
        if !known || code.len() == CODE_LENGTH {
            return None;
        }
        code.push(c);
    }
    (code.len() == CODE_LENGTH).then_some(code)
}

/// The SHA-256 of a normalized code as hexadecimal.
fn code_hash(code: &str) -> String {
    sha256_hex(&[code.as_bytes()])
}

/// Make a waiting app and its pairing code (ADR 0110 item 6). The code has
/// 8 characters of Crockford base32 from the random source, in the form
/// `K7M2-Q9XD`. The collection keeps only its SHA-256, so this call is the
/// only time that the code exists in readable form.
///
/// # Errors
///
/// Gives a [`CollectionError`] when the name is empty, when the random
/// source is not available, or when SQLite fails.
pub fn start_pairing(
    collection: &Collection,
    name: &str,
) -> Result<(ConnectedApp, Zeroizing<String>), CollectionError> {
    start_pairing_at(collection, name, now())
}

/// Like [`start_pairing`], with the time `now` in seconds since the Unix
/// epoch.
///
/// # Errors
///
/// Gives the errors of [`start_pairing`].
pub fn start_pairing_at(
    collection: &Collection,
    name: &str,
    now: i64,
) -> Result<(ConnectedApp, Zeroizing<String>), CollectionError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CollectionError::invalid("a connected app needs a name"));
    }
    purge_expired(collection, now)?;
    let mut bytes = Zeroizing::new([0_u8; CODE_LENGTH]);
    getrandom::fill(bytes.as_mut_slice()).map_err(|_| {
        CollectionError::with_kind(
            CollectionErrorKind::Database,
            "the random source is not available",
        )
    })?;
    // 256 is a multiple of 32, so the remainder has no bias.
    let code: Zeroizing<String> = Zeroizing::new(
        bytes
            .iter()
            .map(|b| char::from(CODE_ALPHABET[usize::from(*b) % CODE_ALPHABET.len()]))
            .collect(),
    );
    let expires_at = now.saturating_add(CODE_LIFETIME);
    collection.db().execute(
        "INSERT INTO connected_apps (name, created_at, state, code_hash, code_expires_at)
         VALUES (?1, ?2, 'waiting', ?3, ?4)",
        rusqlite::params![name, now, code_hash(&code), expires_at],
    )?;
    let app = ConnectedApp {
        id: collection.db().last_insert_rowid(),
        name: name.to_owned(),
        created_at: now,
        state: AppState::Waiting,
        pubkey: None,
        code_expires_at: Some(expires_at),
        live_value_origin: None,
        last_used_at: None,
        last_address: None,
    };
    let shown = Zeroizing::new(format!("{}-{}", &code[..4], &code[4..]));
    Ok((app, shown))
}

/// Remove each waiting app with an expired code, and forget the code of
/// each active app after its 15 minutes (ADR 0110 items 8 and 11).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn purge_expired(collection: &Collection, now: i64) -> Result<(), CollectionError> {
    collection.db().execute(
        "DELETE FROM connected_apps WHERE state = 'waiting' AND code_expires_at <= ?1",
        [now],
    )?;
    collection.db().execute(
        "UPDATE connected_apps SET code_hash = NULL, code_expires_at = NULL
         WHERE state <> 'waiting' AND code_expires_at <= ?1",
        [now],
    )?;
    Ok(())
}

/// A public key as 64 lowercase hexadecimal characters, when it is a valid
/// secp256k1 key.
fn valid_pubkey(text: &str) -> Option<String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    // `PublicKey::from_hex` does not check that the x coordinate is on the
    // curve. The x-only key of secp256k1 does.
    let bytes: Vec<u8> = text
        .as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect::<Option<_>>()?;
    secp256k1::XOnlyPublicKey::from_slice(&bytes).ok()?;
    PublicKey::from_hex(text).ok().map(|key| key.to_hex())
}

/// Pair a waiting app (ADR 0110 items 10 and 11). The function finds the
/// waiting app from the SHA-256 of `code`, keeps the public key and the live
/// value origin, and makes the app active. The code stays known until its 15
/// minutes end, so that a second pairing gives `app_code_used`.
///
/// `live_value_origin` is `https` or `wss`, a host, an optional port and no
/// path. The function keeps it in a normal form.
///
/// # Errors
///
/// Gives a [`PairError`]: `CodeInvalid` for a code that no waiting app has,
/// `CodeUsed` for a code that an app used in the last 15 minutes,
/// `OriginInvalid`, `KeyInvalid`, and `KeyInUse`.
pub fn pair(
    collection: &Collection,
    code: &str,
    pubkey: &str,
    live_value_origin: &str,
) -> Result<ConnectedApp, PairError> {
    pair_at(collection, code, pubkey, live_value_origin, now())
}

/// Like [`pair`], with the time `now` in seconds since the Unix epoch.
///
/// # Errors
///
/// Gives the errors of [`pair`].
pub fn pair_at(
    collection: &Collection,
    code: &str,
    pubkey: &str,
    live_value_origin: &str,
    now: i64,
) -> Result<ConnectedApp, PairError> {
    purge_expired(collection, now)?;
    let invalid = || PairError::new(PairErrorKind::CodeInvalid, "the pairing code is not valid");
    let code = normalize_code(code).ok_or_else(invalid)?;
    let found: Option<ConnectedApp> = collection
        .db()
        .query_row(
            &format!("SELECT {APP_COLUMNS} FROM connected_apps WHERE code_hash = ?1"),
            [code_hash(&code)],
            app_row,
        )
        .optional()?;
    let app = found.ok_or_else(invalid)?;
    if app.state == AppState::Active {
        return Err(PairError::new(
            PairErrorKind::CodeUsed,
            "an app used this pairing code in the last 15 minutes",
        ));
    }
    if app.state != AppState::Waiting {
        return Err(invalid());
    }
    let origin = normalize_origin(live_value_origin).ok_or_else(|| {
        PairError::new(
            PairErrorKind::OriginInvalid,
            "ADR 0110 item 24: the live value origin must be https or wss, with a host and no path",
        )
    })?;
    let key = valid_pubkey(pubkey).ok_or_else(|| {
        PairError::new(
            PairErrorKind::KeyInvalid,
            "the app key is not a valid secp256k1 public key",
        )
    })?;
    let in_use: bool = collection.db().query_row(
        "SELECT EXISTS (SELECT 1 FROM connected_apps WHERE pubkey = ?1)",
        [&key],
        |row| row.get(0),
    )?;
    if in_use {
        return Err(PairError::new(
            PairErrorKind::KeyInUse,
            "an app has this key already",
        ));
    }
    // The code hash and its end stay, for `app_code_used`.
    collection.db().execute(
        "UPDATE connected_apps SET state = 'active', pubkey = ?2, live_value_origin = ?3
         WHERE id = ?1 AND state = 'waiting'",
        rusqlite::params![app.id, key, origin],
    )?;
    Ok(ConnectedApp {
        state: AppState::Active,
        pubkey: Some(key),
        live_value_origin: Some(origin),
        ..app
    })
}

/// The short form of an app key for the user: the first 9 and the last 4
/// characters of the `npub`, with an ellipsis between them (ADR 0110
/// item 18). Gives an empty text when `pubkey` is not a public key.
#[must_use]
pub fn short_key(pubkey: &str) -> String {
    let Some(npub) = PublicKey::from_hex(pubkey)
        .ok()
        .and_then(|key| key.to_bech32().ok())
    else {
        return String::new();
    };
    format!("{}\u{2026}{}", &npub[..9], &npub[npub.len() - 4..])
}

/// Remove an app, and end its grants at once (ADR 0110 item 21). The draft
/// grants of the app go out too. The caller removes the wrappers with the
/// next sync of the grant vaults, which also removes the vault of each grant
/// that ended.
///
/// # Errors
///
/// Gives a [`CollectionError`] of kind `NotFound` when the app does not exist.
pub fn remove_app(collection: &Collection, id: i64) -> Result<(), CollectionError> {
    remove_app_at(collection, id, now())
}

/// Like [`remove_app`], with the time `now` in seconds since the Unix epoch.
///
/// # Errors
///
/// Gives the errors of [`remove_app`].
pub fn remove_app_at(collection: &Collection, id: i64, now: i64) -> Result<(), CollectionError> {
    let changed = collection
        .db()
        .execute("DELETE FROM connected_apps WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(CollectionError::with_kind(
            CollectionErrorKind::NotFound,
            format!("connected app {id} does not exist"),
        ));
    }
    collection.db().execute(
        "UPDATE show_grants SET ended_at = ?2 WHERE app_id = ?1 AND ended_at IS NULL",
        rusqlite::params![id, now],
    )?;
    collection
        .db()
        .execute("DELETE FROM draft_grants WHERE app_id = ?1", [id])?;
    Ok(())
}

/// The apps, the oldest first. A waiting app with an expired code is not in
/// the list.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn apps(collection: &Collection) -> Result<Vec<ConnectedApp>, CollectionError> {
    purge_expired(collection, now())?;
    let mut statement = collection.db().prepare(&format!(
        "SELECT {APP_COLUMNS} FROM connected_apps ORDER BY id"
    ))?;
    let rows = statement.query_map([], app_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The app with this ID, or `None`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn app(collection: &Collection, id: i64) -> Result<Option<ConnectedApp>, CollectionError> {
    purge_expired(collection, now())?;
    Ok(collection
        .db()
        .query_row(
            &format!("SELECT {APP_COLUMNS} FROM connected_apps WHERE id = ?1"),
            [id],
            app_row,
        )
        .optional()?)
}

/// The active app that has this public key, or `None`. The key is
/// 64 hexadecimal characters in lowercase.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn app_for_pubkey(
    collection: &Collection,
    pubkey: &str,
) -> Result<Option<ConnectedApp>, CollectionError> {
    Ok(collection
        .db()
        .query_row(
            &format!(
                "SELECT {APP_COLUMNS} FROM connected_apps WHERE pubkey = ?1 AND state = 'active'"
            ),
            [pubkey],
            app_row,
        )
        .optional()?)
}

/// The SHA-256 of the pairing code of each waiting app with its app ID. The
/// server makes its index from this list (ADR 0110 item 39).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn waiting_code_hashes(collection: &Collection) -> Result<Vec<(i64, String)>, CollectionError> {
    purge_expired(collection, now())?;
    let mut statement = collection.db().prepare(
        "SELECT id, code_hash FROM connected_apps
         WHERE state = 'waiting' AND code_hash IS NOT NULL ORDER BY id",
    )?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The SHA-256 of a pairing code as the collection keeps it, or `None` when
/// the text is not a code. The server uses it to find the account of a
/// waiting app (ADR 0110 item 39).
#[must_use]
pub fn hash_of_code(code: &str) -> Option<String> {
    normalize_code(code).map(|code| code_hash(&code))
}

/// Record the time and the client address of an accepted request (ADR 0110
/// item 18).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record_use(
    collection: &Collection,
    id: i64,
    address: &str,
    now: i64,
) -> Result<(), CollectionError> {
    collection.db().execute(
        "UPDATE connected_apps SET last_used_at = ?2, last_address = ?3 WHERE id = ?1",
        rusqlite::params![id, now, address],
    )?;
    Ok(())
}

/// The kind of a [`RequestError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequestErrorKind {
    /// The signed request is not correct (see [`SignedRequestError`]).
    Signature(SignedRequestError),
    /// The public key is not the key of an active app.
    UnknownKey,
    /// Studio accepted the same event ID in the last 120 seconds.
    Replay,
    /// The event is more than 60 seconds earlier than the latest event that
    /// Studio accepted from the app.
    Stale,
    /// SQLite failed.
    Database,
}

/// An error of [`authenticate`] or [`accept_event`]. Each kind except
/// `Database` gives 401 `app_signature_invalid` in the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestError {
    kind: RequestErrorKind,
}

impl RequestError {
    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> RequestErrorKind {
        self.kind
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The text names no event, no key and no address.
        f.write_str(match self.kind {
            RequestErrorKind::Signature(error) => error.reason(),
            RequestErrorKind::UnknownKey => "the key is not the key of an active app",
            RequestErrorKind::Replay => "Studio accepted this event already",
            RequestErrorKind::Stale => "the event is older than an event that Studio accepted",
            RequestErrorKind::Database => "the collection failed",
        })
    }
}

impl std::error::Error for RequestError {}

impl From<CollectionError> for RequestError {
    fn from(_: CollectionError) -> Self {
        Self {
            kind: RequestErrorKind::Database,
        }
    }
}

impl From<rusqlite::Error> for RequestError {
    fn from(_: rusqlite::Error) -> Self {
        Self {
            kind: RequestErrorKind::Database,
        }
    }
}

/// Record an accepted event ID, and the `created_at` of the event for the
/// app, before the route runs (ADR 0110 items 13 and 15). The store lives in
/// the collection, so a restart does not clear it. A request that fails
/// after this call has used up its event.
///
/// `app_id` is `None` for the pairing request, which has no app yet. Then
/// only the event ID counts. `now` is the time of the server in seconds
/// since the Unix epoch.
///
/// # Errors
///
/// Gives [`RequestErrorKind::Replay`] for an event ID that the store has,
/// and [`RequestErrorKind::Stale`] for an event that is more than
/// [`STALE_LIMIT`] seconds earlier than the latest `created_at` of the app.
pub fn accept_event(
    collection: &Collection,
    app_id: Option<i64>,
    event_id: &str,
    created_at: i64,
    now: i64,
) -> Result<(), RequestError> {
    let db = collection.db();
    db.execute("DELETE FROM app_events WHERE expires_at <= ?1", [now])?;
    let known: bool = db.query_row(
        "SELECT EXISTS (SELECT 1 FROM app_events WHERE event_id = ?1)",
        [event_id],
        |row| row.get(0),
    )?;
    if known {
        return Err(RequestError {
            kind: RequestErrorKind::Replay,
        });
    }
    if let Some(id) = app_id {
        let latest: Option<i64> = db
            .query_row(
                "SELECT latest_event_at FROM connected_apps WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        if latest.is_some_and(|latest| latest.saturating_sub(created_at) > STALE_LIMIT) {
            return Err(RequestError {
                kind: RequestErrorKind::Stale,
            });
        }
        db.execute(
            "UPDATE connected_apps SET latest_event_at = max(COALESCE(latest_event_at, 0), ?2)
             WHERE id = ?1",
            rusqlite::params![id, created_at],
        )?;
    }
    db.execute(
        "INSERT INTO app_events (event_id, expires_at) VALUES (?1, ?2)",
        rusqlite::params![event_id, now.saturating_add(REPLAY_WINDOW)],
    )?;
    Ok(())
}

/// Check the signed request of a connected app and record its event (ADR
/// 0110 items 12, 13 and 15). The steps are:
///
/// 1. [`signed_request::check`] examines the event against `expected`.
/// 2. The public key must be the key of an active app.
/// 3. [`accept_event`] records the event ID and the time.
///
/// Gives the app. The route runs after this call. `expected.now` is the
/// time of the server.
///
/// # Errors
///
/// Gives a [`RequestError`]. Every kind except `Database` gives
/// 401 `app_signature_invalid`.
pub fn authenticate(
    collection: &Collection,
    event: &Event,
    expected: &ExpectedRequest<'_>,
) -> Result<ConnectedApp, RequestError> {
    let key = signed_request::check(event, expected).map_err(|error| RequestError {
        kind: RequestErrorKind::Signature(error),
    })?;
    let app = app_for_pubkey(collection, &key.to_hex())?.ok_or(RequestError {
        kind: RequestErrorKind::UnknownKey,
    })?;
    let now = i64::try_from(expected.now).unwrap_or(i64::MAX);
    let created_at = i64::try_from(event.created_at.as_secs()).unwrap_or(i64::MAX);
    accept_event(
        collection,
        Some(app.id),
        &event.id.to_hex(),
        created_at,
        now,
    )?;
    Ok(app)
}

#[cfg(test)]
mod tests {
    use nostr::prelude::Keys;

    use super::*;

    const ORIGIN: &str = "https://live.example.com";

    fn keys() -> Keys {
        crate::signed_request::generate_keys()
    }

    fn started(collection: &Collection, now: i64) -> (ConnectedApp, String) {
        let (app, code) = start_pairing_at(collection, "Tool", now).unwrap();
        (app, code.as_str().to_owned())
    }

    #[test]
    fn a_code_has_8_crockford_characters_and_the_collection_keeps_only_its_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let collection = Collection::open(&path).unwrap();
        // The list of the waiting codes reads the clock, so the test uses it.
        let start = crate::collection::now();
        let (app, code) = started(&collection, start);
        assert_eq!(code.len(), 9);
        assert_eq!(&code[4..5], "-");
        let plain = normalize_code(&code).unwrap();
        assert!(plain.bytes().all(|b| CODE_ALPHABET.contains(&b)));
        assert_eq!(app.state, AppState::Waiting);
        assert_eq!(app.code_expires_at, Some(start + CODE_LIFETIME));
        // The JSON form has no code, no hash and no key.
        let json = serde_json::to_string(&app).unwrap();
        assert!(!json.contains(plain.as_str()));
        assert!(!json.contains(&code_hash(&plain)));
        drop(collection);
        let bytes = std::fs::read(&path).unwrap();
        for needle in [plain.as_bytes(), code.as_bytes()] {
            assert!(
                !bytes.windows(needle.len()).any(|w| w == needle),
                "ADR 0110 item 7: the database file must not hold the pairing code. Keep only its SHA-256."
            );
        }
        let collection = Collection::open(&path).unwrap();
        assert_eq!(
            waiting_code_hashes(&collection).unwrap(),
            [(app.id, code_hash(&plain))]
        );
    }

    #[test]
    fn the_rules_of_a_code_follow_crockford_base32() {
        let same = ["K7M2Q9XD", "k7m2 q9xd", "K7M2-Q9XD", " k7m2-q9xd "];
        for text in same {
            assert_eq!(
                normalize_code(text).as_deref().map(String::as_str),
                Some("K7M2Q9XD")
            );
        }
        // O reads as 0, and I and L read as 1.
        assert_eq!(
            normalize_code("OIL0-1111").as_deref().map(String::as_str),
            Some("01101111")
        );
        for text in [
            "",
            "K7M2Q9X",
            "K7M2Q9XDA",
            "K7M2Q9XU",
            "K7M2-Q9X!",
            "K7M2Q9X\u{e9}",
        ] {
            assert!(normalize_code(text).is_none(), "{text}");
        }
        assert_eq!(hash_of_code("k7m2 q9xd"), hash_of_code("K7M2-Q9XD"));
        assert_eq!(hash_of_code("nope"), None);
    }

    #[test]
    fn a_name_is_needed_and_the_same_name_is_permitted() {
        let collection = Collection::open_in_memory().unwrap();
        let (one, code_one) = started(&collection, 1);
        let (two, code_two) = started(&collection, 1);
        assert_ne!(one.id, two.id);
        assert_ne!(code_one, code_two);
        let error = start_pairing(&collection, "  ").unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
    }

    #[test]
    fn a_pairing_makes_the_app_active_with_a_lowercase_key() {
        let collection = Collection::open_in_memory().unwrap();
        let (waiting, code) = started(&collection, 1_000);
        let key = keys().public_key();
        let upper = key.to_hex().to_ascii_uppercase();
        let app = pair_at(&collection, &code.to_lowercase(), &upper, ORIGIN, 1_100).unwrap();
        assert_eq!(app.id, waiting.id);
        assert_eq!(app.state, AppState::Active);
        assert_eq!(app.pubkey.as_deref(), Some(key.to_hex().as_str()));
        assert_eq!(app.live_value_origin.as_deref(), Some(ORIGIN));
        let found = app_for_pubkey(&collection, &key.to_hex()).unwrap().unwrap();
        assert_eq!(found.id, waiting.id);
        assert_eq!(found.state, AppState::Active);
        // The JSON form has no key.
        assert!(
            !serde_json::to_string(&found)
                .unwrap()
                .contains(&key.to_hex())
        );
    }

    #[test]
    fn a_pairing_refuses_an_expired_code_a_wrong_origin_and_an_invalid_key() {
        let collection = Collection::open_in_memory().unwrap();
        let key = keys().public_key().to_hex();
        let (app, code) = started(&collection, 1_000);

        let error = pair_at(&collection, &code, &key, ORIGIN, 1_000 + CODE_LIFETIME).unwrap_err();
        assert_eq!(
            error.kind(),
            PairErrorKind::CodeInvalid,
            "ADR 0110 item 8: an expired code must give app_code_invalid. Fix pair_at."
        );
        assert_eq!(error.code(), "app_code_invalid");
        assert_eq!(app_for_pubkey(&collection, &key).unwrap(), None);
        assert!(
            !app_row_exists(&collection, app.id),
            "an expired waiting app goes out"
        );

        for origin in [
            "http://live.example.com",
            "ws://live.example.com",
            "https://live.example.com/",
            "https://live.example.com/path",
            "live.example.com",
            "",
        ] {
            let (_, code) = started(&collection, 2_000);
            let error = pair_at(&collection, &code, &key, origin, 2_001).unwrap_err();
            assert_eq!(
                error.kind(),
                PairErrorKind::OriginInvalid,
                "ADR 0110 item 24: {origin:?} must be refused. Fix pair_at."
            );
            assert_eq!(error.code(), "live_value_origin_invalid");
        }
        for key in [
            "",
            "abc",
            &"g".repeat(64),
            &"0".repeat(64),
            &format!("{}00", keys().public_key().to_hex()),
            &keys().public_key().to_bech32().unwrap(),
        ] {
            let (_, code) = started(&collection, 3_000);
            let error = pair_at(&collection, &code, key, ORIGIN, 3_001).unwrap_err();
            assert_eq!(
                error.kind(),
                PairErrorKind::KeyInvalid,
                "ADR 0110 item 10: {key:?} must be refused. Fix pair_at."
            );
            assert_eq!(error.code(), "app_key_invalid");
        }
        // A refused pairing keeps the waiting app and its code.
        let (_, code) = started(&collection, 4_000);
        pair_at(&collection, &code, "", ORIGIN, 4_001).unwrap_err();
        assert!(pair_at(&collection, &code, &key, ORIGIN, 4_002).is_ok());
    }

    fn app_row_exists(collection: &Collection, id: i64) -> bool {
        collection
            .db()
            .query_row(
                "SELECT COUNT(*) FROM connected_apps WHERE id = ?1",
                [id],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
            > 0
    }

    #[test]
    fn a_used_code_gives_app_code_used_for_15_minutes_and_then_app_code_invalid() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, code) = started(&collection, 1_000);
        pair_at(
            &collection,
            &code,
            &keys().public_key().to_hex(),
            ORIGIN,
            1_010,
        )
        .unwrap();
        let other = keys().public_key().to_hex();
        let error = pair_at(&collection, &code, &other, ORIGIN, 1_020).unwrap_err();
        assert_eq!(
            error.kind(),
            PairErrorKind::CodeUsed,
            "ADR 0110 item 11: a code that an app used must give app_code_used. Keep the code hash after the pairing."
        );
        assert_eq!(error.code(), "app_code_used");
        // The 15 minutes count from the start of the pairing code.
        let error = pair_at(
            &collection,
            &code,
            &other,
            ORIGIN,
            1_000 + CODE_LIFETIME - 1,
        )
        .unwrap_err();
        assert_eq!(error.kind(), PairErrorKind::CodeUsed);
        let error = pair_at(&collection, &code, &other, ORIGIN, 1_000 + CODE_LIFETIME).unwrap_err();
        assert_eq!(error.kind(), PairErrorKind::CodeInvalid);
        // The active app stays, and its code is forgotten.
        assert_eq!(app_for_pubkey(&collection, &other).unwrap(), None);
        assert_eq!(
            apps(&collection).unwrap()[0].id,
            app.id,
            "an active app does not go out when its code ends"
        );
        let hashes: Option<String> = collection
            .db()
            .query_row(
                "SELECT code_hash FROM connected_apps WHERE id = ?1",
                [app.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hashes, None);
    }

    #[test]
    fn a_code_after_the_removal_of_the_waiting_app_gives_app_code_invalid() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, code) = started(&collection, 1_000);
        remove_app_at(&collection, app.id, 1_001).unwrap();
        let error = pair_at(
            &collection,
            &code,
            &keys().public_key().to_hex(),
            ORIGIN,
            1_002,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            PairErrorKind::CodeInvalid,
            "ADR 0110 item 8: a code after Done must give app_code_invalid. Remove the code with the waiting app."
        );
        let error = remove_app_at(&collection, app.id, 1_003).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::NotFound);
    }

    #[test]
    fn a_key_is_active_in_one_app_only() {
        let collection = Collection::open_in_memory().unwrap();
        let key = keys().public_key().to_hex();
        let (_, one) = started(&collection, 1);
        pair_at(&collection, &one, &key, ORIGIN, 2).unwrap();
        let (_, two) = started(&collection, 3);
        let error = pair_at(&collection, &two, &key, ORIGIN, 4).unwrap_err();
        assert_eq!(error.kind(), PairErrorKind::KeyInUse);
        assert_eq!(error.code(), "app_key_in_use");
    }

    #[test]
    fn a_waiting_app_with_an_expired_code_goes_out_at_each_call() {
        let collection = Collection::open_in_memory().unwrap();
        let (old, _) = started(&collection, 1_000);
        // A later start removes the old app.
        let (fresh, _) = started(&collection, 1_000 + CODE_LIFETIME);
        assert!(!app_row_exists(&collection, old.id));
        assert!(app_row_exists(&collection, fresh.id));
        purge_expired(&collection, 1_000 + 2 * CODE_LIFETIME).unwrap();
        assert!(!app_row_exists(&collection, fresh.id));
    }

    #[test]
    fn the_short_key_has_9_and_4_characters_of_the_npub() {
        let key = keys().public_key();
        let npub = key.to_bech32().unwrap();
        let short = short_key(&key.to_hex());
        assert_eq!(
            short,
            format!("{}\u{2026}{}", &npub[..9], &npub[npub.len() - 4..])
        );
        assert!(short.starts_with("npub1"));
        assert_eq!(short_key("nope"), "");
    }

    #[test]
    fn removing_an_app_ends_its_grants_and_drops_its_draft_grants() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let (app, code) = started(&collection, 1);
        pair_at(&collection, &code, &keys().public_key().to_hex(), ORIGIN, 2).unwrap();
        crate::grant::record(&collection, feed, "a", app.id, ORIGIN, 3).unwrap();
        collection.add_draft_grant(feed, "b", app.id).unwrap();
        remove_app_at(&collection, app.id, 50).unwrap();
        let grant = crate::grant::grant_for(&collection, feed, "a")
            .unwrap()
            .unwrap();
        assert_eq!(grant.ended_at, Some(50));
        assert_eq!(collection.draft_grants(feed).unwrap(), Vec::<String>::new());
    }

    /// A signed request event for the tests.
    fn event(keys: &Keys, url: &str, created_at: u64) -> Event {
        crate::signed_request::signed_event(
            keys,
            27_235,
            &[["u", url], ["method", "GET"]],
            created_at,
        )
    }

    fn expected(url: &str, now: u64) -> ExpectedRequest<'_> {
        ExpectedRequest {
            url,
            method: "GET",
            body: Some(b""),
            now,
        }
    }

    fn active_app(collection: &Collection) -> (ConnectedApp, Keys) {
        let keys = keys();
        let (_, code) = started(collection, 1);
        let app = pair_at(collection, &code, &keys.public_key().to_hex(), ORIGIN, 2).unwrap();
        (app, keys)
    }

    const URL: &str = "https://studio.example.com/app/v1/whoami";

    #[test]
    fn an_accepted_event_is_refused_a_second_time_for_120_seconds() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, keys) = active_app(&collection);
        let first = event(&keys, URL, 10_000);
        assert_eq!(
            authenticate(&collection, &first, &expected(URL, 10_000))
                .unwrap()
                .id,
            app.id
        );
        for now in [10_000, 10_000 + 59] {
            let error = authenticate(&collection, &first, &expected(URL, now)).unwrap_err();
            assert_eq!(
                error.kind(),
                RequestErrorKind::Replay,
                "ADR 0110 item 15: a repeated event ID must be refused. Record the ID before the route runs."
            );
        }
        // A new event of the same app is accepted.
        let second = event(&keys, URL, 10_001);
        assert!(authenticate(&collection, &second, &expected(URL, 10_001)).is_ok());
        // After 120 seconds, the store forgets the ID. The time check now
        // refuses the old event.
        let error = authenticate(
            &collection,
            &first,
            &expected(URL, 10_000 + REPLAY_WINDOW as u64),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            RequestErrorKind::Signature(SignedRequestError::Time)
        );
    }

    #[test]
    fn the_replay_store_is_in_the_collection_and_survives_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let collection = Collection::open(&path).unwrap();
        let (_, keys) = active_app(&collection);
        let first = event(&keys, URL, 10_000);
        authenticate(&collection, &first, &expected(URL, 10_000)).unwrap();
        drop(collection);
        let collection = Collection::open(&path).unwrap();
        let error = authenticate(&collection, &first, &expected(URL, 10_001)).unwrap_err();
        assert_eq!(
            error.kind(),
            RequestErrorKind::Replay,
            "ADR 0110 item 15: the replay store must be in the collection, not in memory."
        );
    }

    #[test]
    fn after_the_clock_goes_back_one_hour_an_accepted_event_is_refused() {
        let collection = Collection::open_in_memory().unwrap();
        let (_, keys) = active_app(&collection);
        let accepted = event(&keys, URL, 100_000);
        authenticate(&collection, &accepted, &expected(URL, 100_000)).unwrap();
        // The clock of the server goes back by one hour.
        let back = 100_000 - 3_600;
        let error = authenticate(&collection, &accepted, &expected(URL, back)).unwrap_err();
        assert!(
            matches!(
                error.kind(),
                RequestErrorKind::Replay | RequestErrorKind::Signature(SignedRequestError::Time)
            ),
            "ADR 0110 item 13: an event that Studio accepted before must be refused after the clock goes back"
        );
        // An older event of the same app, in the new time, is also stale:
        // it is more than 60 seconds earlier than the latest event.
        let older = event(&keys, URL, back);
        let error = authenticate(&collection, &older, &expected(URL, back)).unwrap_err();
        assert_eq!(
            error.kind(),
            RequestErrorKind::Stale,
            "ADR 0110 item 13: an event more than 60 seconds earlier than the latest accepted event must be refused. Keep the latest created_at of each app."
        );
        // Within 60 seconds earlier is still accepted.
        let near = event(&keys, URL, 100_000 - 60);
        assert!(authenticate(&collection, &near, &expected(URL, 100_000 - 60)).is_ok());
        let far = event(&keys, URL, 100_000 - 61);
        let error = authenticate(&collection, &far, &expected(URL, 100_000 - 61)).unwrap_err();
        assert_eq!(error.kind(), RequestErrorKind::Stale);
    }

    #[test]
    fn a_key_that_is_not_an_active_app_is_refused() {
        let collection = Collection::open_in_memory().unwrap();
        let (waiting, _code) = started(&collection, 1);
        let stranger = keys();
        let error = authenticate(
            &collection,
            &event(&stranger, URL, 500),
            &expected(URL, 500),
        )
        .unwrap_err();
        assert_eq!(error.kind(), RequestErrorKind::UnknownKey);
        // A waiting app has no key. A token app has none either.
        assert_eq!(waiting.pubkey, None);
        // The refused event does not use up an ID.
        let (_, keys) = active_app(&collection);
        let signed = event(&keys, URL, 500);
        assert!(authenticate(&collection, &signed, &expected(URL, 500)).is_ok());
    }

    #[test]
    fn the_record_of_a_use_keeps_the_time_and_the_address() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, _) = active_app(&collection);
        record_use(&collection, app.id, "203.0.113.7", 77).unwrap();
        let found = self::app(&collection, app.id).unwrap().unwrap();
        assert_eq!(found.last_used_at, Some(77));
        assert_eq!(found.last_address.as_deref(), Some("203.0.113.7"));
    }
}
