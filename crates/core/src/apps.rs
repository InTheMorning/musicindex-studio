//! Connected apps (ADR 0054 items 7 to 10).
//!
//! A connected app is a program on the same computer that calls the
//! `/app/v1/` routes. Each app has a token. Studio shows the token one time,
//! when it makes the app. The collection keeps only the SHA-256 of the token.

use std::fmt::Write as _;

use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::collection::{Collection, CollectionError, CollectionErrorKind, now};
use crate::digest::sha256_hex;

/// One connected app. It has no token and no token hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectedApp {
    /// The ID of the app.
    pub id: i64,
    /// The name that the user gave.
    pub name: String,
    /// The time of creation, in seconds since the Unix epoch.
    pub created_at: i64,
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
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| {
        CollectionError::with_kind(
            CollectionErrorKind::Database,
            "the random source is not available",
        )
    })?;
    let token = bytes.iter().fold(String::with_capacity(64), |mut text, b| {
        let _ = write!(text, "{b:02x}");
        text
    });
    let created_at = now();
    collection.db().execute(
        "INSERT INTO connected_apps (name, token_hash, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, token_hash(&token), created_at],
    )?;
    let app = ConnectedApp {
        id: collection.db().last_insert_rowid(),
        name: name.to_owned(),
        created_at,
    };
    Ok((app, token))
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
    let mut statement = collection
        .db()
        .prepare("SELECT id, name, created_at FROM connected_apps ORDER BY id")?;
    let rows = statement.query_map([], |row| {
        Ok(ConnectedApp {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
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
            "SELECT id, name, created_at FROM connected_apps WHERE token_hash = ?1",
            [token_hash(token)],
            |row| {
                Ok(ConnectedApp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
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
