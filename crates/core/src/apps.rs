//! Connected apps (ADR 0054 items 7 to 10).
//!
//! A connected app is a program on the same computer that calls the
//! `/app/v1/` routes. Each app has a token. Studio shows the token one time,
//! when it makes the app. The collection keeps only the SHA-256 of the token
//! and the public app key that comes from the token (ADR 0058 item 4).

use std::fmt::Write as _;

use nostr::prelude::{Keys, SecretKey};
use rusqlite::OptionalExtension;
use serde::Serialize;
use zeroize::Zeroizing;

use crate::collection::{Collection, CollectionError, CollectionErrorKind, now};
use crate::digest::sha256_hex;
use crate::vault::crypto::Key;

/// The HKDF info text of the app key (ADR 0058 item 4).
const APP_KEY_INFO: &str = "musicindex-studio app key v1";

/// One connected app. It has no token and no token hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectedApp {
    /// The ID of the app.
    pub id: i64,
    /// The name that the user gave.
    pub name: String,
    /// The time of creation, in seconds since the Unix epoch.
    pub created_at: i64,
    /// True when the app has an app key. An app from before ADR 0058 has
    /// none, and it cannot get a grant vault.
    pub has_key: bool,
    /// The public app key as hexadecimal text. It is not secret, and the
    /// JSON form of the app does not have it.
    #[serde(skip)]
    pub pubkey: Option<String>,
}

fn app_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConnectedApp> {
    let pubkey: Option<String> = row.get(3)?;
    Ok(ConnectedApp {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        has_key: pubkey.is_some(),
        pubkey,
    })
}

const APP_COLUMNS: &str = "id, name, created_at, app_pubkey";

/// The bytes of a token. A token of 64 hexadecimal characters gives its 32
/// bytes. Any other text gives its UTF-8 bytes.
fn token_bytes(token: &str) -> Zeroizing<Vec<u8>> {
    let hex = token.as_bytes();
    let bytes = (hex.len() == 64)
        .then(|| {
            hex.chunks(2)
                .map(|pair| {
                    let text = std::str::from_utf8(pair).ok()?;
                    u8::from_str_radix(text, 16).ok()
                })
                .collect::<Option<Vec<u8>>>()
        })
        .flatten();
    Zeroizing::new(bytes.unwrap_or_else(|| hex.to_vec()))
}

/// The first app key of a token, or `None` when the HKDF output is not a
/// valid secp256k1 secret key. The chance of `None` is about 2^-128.
fn first_app_keys(token: &str) -> Option<Keys> {
    let key = Key::expand(&token_bytes(token), &[], APP_KEY_INFO);
    SecretKey::from_slice(key.as_bytes()).ok().map(Keys::new)
}

/// The app key of a token (ADR 0058 item 4): a secp256k1 key from
/// HKDF-SHA-256 over the 32 bytes of the token. [`add_app`] makes a token
/// whose first key is valid, so the key of such a token is always the
/// first key.
#[must_use]
pub fn app_keys(token: &str) -> Keys {
    if let Some(keys) = first_app_keys(token) {
        return keys;
    }
    // A token that [`add_app`] did not make. Another HKDF step gives a key
    // that is valid in all practical cases.
    let mut input = token_bytes(token);
    loop {
        let key = Key::expand(&input, &[], APP_KEY_INFO);
        if let Ok(secret) = SecretKey::from_slice(key.as_bytes()) {
            return Keys::new(secret);
        }
        input = Zeroizing::new(key.as_bytes().to_vec());
    }
}

/// Add an app. Gives the new row and its token. The token is 64 hexadecimal
/// characters. This is the only time that the token exists in readable form.
///
/// # Errors
///
/// Gives a [`CollectionError`] when the name is empty, when the random source
/// is not available, or when SQLite fails.
pub fn add_app(
    collection: &Collection,
    name: &str,
) -> Result<(ConnectedApp, String), CollectionError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CollectionError::invalid("a connected app needs a name"));
    }
    // A token whose HKDF output is not a valid key is very rare. A new token
    // is then correct.
    let (token, pubkey) = loop {
        let mut bytes = Zeroizing::new([0_u8; 32]);
        getrandom::fill(bytes.as_mut_slice()).map_err(|_| {
            CollectionError::with_kind(
                CollectionErrorKind::Database,
                "the random source is not available",
            )
        })?;
        let token = bytes
            .iter()
            .fold(Zeroizing::new(String::with_capacity(64)), |mut text, b| {
                let _ = write!(text, "{b:02x}");
                text
            });
        if let Some(keys) = first_app_keys(&token) {
            break (token, keys.public_key().to_hex());
        }
    };
    let created_at = now();
    collection.db().execute(
        "INSERT INTO connected_apps (name, token_hash, created_at, app_pubkey) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, token_hash(&token), created_at, pubkey],
    )?;
    let app = ConnectedApp {
        id: collection.db().last_insert_rowid(),
        name: name.to_owned(),
        created_at,
        has_key: true,
        pubkey: Some(pubkey),
    };
    Ok((app, token.as_str().to_owned()))
}

/// Remove an app. Its token stops working.
///
/// # Errors
///
/// Gives a [`CollectionError`] of kind `NotFound` when the app does not exist.
pub fn remove_app(collection: &Collection, id: i64) -> Result<(), CollectionError> {
    let changed = collection
        .db()
        .execute("DELETE FROM connected_apps WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(CollectionError::with_kind(
            CollectionErrorKind::NotFound,
            format!("connected app {id} does not exist"),
        ));
    }
    Ok(())
}

/// The apps, the oldest first.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn apps(collection: &Collection) -> Result<Vec<ConnectedApp>, CollectionError> {
    let mut statement = collection.db().prepare(&format!(
        "SELECT {APP_COLUMNS} FROM connected_apps ORDER BY id"
    ))?;
    let rows = statement.query_map([], app_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The app that has this token, or `None`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn app_for_token(
    collection: &Collection,
    token: &str,
) -> Result<Option<ConnectedApp>, CollectionError> {
    Ok(collection
        .db()
        .query_row(
            &format!("SELECT {APP_COLUMNS} FROM connected_apps WHERE token_hash = ?1"),
            [token_hash(token)],
            app_row,
        )
        .optional()?)
}

fn token_hash(token: &str) -> String {
    sha256_hex(&[token.as_bytes()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_app_is_found_with_its_token_and_with_no_other_token() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Show tool").unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(app_for_token(&collection, &token).unwrap(), Some(app));
        let other = "0".repeat(64);
        assert_eq!(app_for_token(&collection, &other).unwrap(), None);
        assert_eq!(app_for_token(&collection, "").unwrap(), None);
    }

    #[test]
    fn the_database_file_does_not_contain_the_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let collection = Collection::open(&path).unwrap();
        let (_, token) = add_app(&collection, "Show tool").unwrap();
        drop(collection);
        let bytes = std::fs::read(&path).unwrap();
        let needle = token.as_bytes();
        assert!(!bytes.windows(needle.len()).any(|w| w == needle));
        let collection = Collection::open(&path).unwrap();
        assert!(app_for_token(&collection, &token).unwrap().is_some());
    }

    #[test]
    fn the_same_name_is_permitted_and_an_empty_name_is_not() {
        let collection = Collection::open_in_memory().unwrap();
        let (one, token_one) = add_app(&collection, "Tool").unwrap();
        let (two, token_two) = add_app(&collection, "Tool").unwrap();
        assert_ne!(one.id, two.id);
        assert_ne!(token_one, token_two);
        assert_eq!(apps(&collection).unwrap(), [one, two]);
        let error = add_app(&collection, "  ").unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
    }

    #[test]
    fn the_app_key_comes_from_the_token_and_its_public_key_is_kept() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        assert!(app.has_key);
        let keys = app_keys(&token);
        assert_eq!(app.pubkey, Some(keys.public_key().to_hex()));
        assert_eq!(app_keys(&token).public_key(), keys.public_key());
        let found = app_for_token(&collection, &token).unwrap().unwrap();
        assert_eq!(found.pubkey, app.pubkey);
        // The JSON form has no key, and no token.
        let json = serde_json::to_string(&app).unwrap();
        assert!(json.contains("\"has_key\":true"));
        assert!(!json.contains(&keys.public_key().to_hex()));
        assert!(!json.contains(&token));
    }

    #[test]
    fn the_app_key_of_a_fixed_token_is_fixed_and_a_different_token_gives_another_key() {
        let token = "00".repeat(32);
        let again = app_keys(&token).public_key();
        assert_eq!(app_keys(&token).public_key(), again);
        assert_ne!(app_keys(&"01".repeat(32)).public_key(), again);
        // The key is the HKDF-SHA-256 output, with the info text of the ADR.
        let expected = Key::expand(&[0_u8; 32], &[], APP_KEY_INFO);
        assert_eq!(
            app_keys(&token).secret_key().as_secret_bytes(),
            expected.as_bytes()
        );
    }

    #[test]
    fn an_app_row_from_before_the_key_has_no_key() {
        let collection = Collection::open_in_memory().unwrap();
        collection
            .db()
            .execute(
                "INSERT INTO connected_apps (name, token_hash, created_at) VALUES ('Old', 'h', 1)",
                [],
            )
            .unwrap();
        let list = apps(&collection).unwrap();
        assert_eq!(list.len(), 1);
        assert!(!list[0].has_key);
        assert_eq!(list[0].pubkey, None);
    }

    #[test]
    fn a_removed_app_does_not_match_its_token() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        remove_app(&collection, app.id).unwrap();
        assert_eq!(app_for_token(&collection, &token).unwrap(), None);
        assert_eq!(apps(&collection).unwrap().len(), 0);
        let error = remove_app(&collection, app.id).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::NotFound);
    }

    #[test]
    fn the_debug_and_json_text_of_an_app_has_no_hash() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        let hash = token_hash(&token);
        assert!(!format!("{app:?}").contains(&hash));
        assert!(!serde_json::to_string(&app).unwrap().contains(&hash));
    }
}
