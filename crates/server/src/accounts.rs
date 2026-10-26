//! The database `server.sqlite`: accounts, sessions and invite codes
//! (ADR 0055 item 6). It holds no feed data, no secret, no session token
//! and no invite code. It keeps the SHA-256 of each token and code.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

/// The schema migrations. Migration `n` takes the schema from version `n`
/// to version `n + 1`.
const MIGRATIONS: &[&str] = &["
    CREATE TABLE accounts (
        id INTEGER PRIMARY KEY,
        folder TEXT NOT NULL UNIQUE,
        created_at INTEGER NOT NULL
    );
    CREATE TABLE account_emails (
        account_id INTEGER NOT NULL UNIQUE REFERENCES accounts(id) ON DELETE CASCADE,
        email TEXT NOT NULL UNIQUE
    );
    CREATE TABLE sessions (
        token_hash TEXT PRIMARY KEY,
        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        created_at INTEGER NOT NULL,
        last_seen INTEGER NOT NULL
    );
    CREATE TABLE invites (
        code_hash TEXT PRIMARY KEY,
        created_at INTEGER NOT NULL,
        used_at INTEGER
    );
"];

/// Give `bytes` as lowercase hexadecimal.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, b| {
        let _ = write!(text, "{b:02x}");
        text
    })
}

/// The SHA-256 of `text` as hexadecimal.
pub fn sha256_hex(text: &str) -> String {
    hex(&Sha256::digest(text.as_bytes()))
}

/// `count` random bytes as hexadecimal.
///
/// # Errors
///
/// Gives a message when the random source is not available.
pub fn random_hex(count: usize) -> Result<String, String> {
    let mut bytes = vec![0_u8; count];
    getrandom::fill(&mut bytes).map_err(|_| "the random source is not available".to_owned())?;
    Ok(hex(&bytes))
}

/// The seconds since the Unix epoch.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Why a sign-up failed.
#[derive(Debug, PartialEq, Eq)]
pub enum SignUpError {
    /// The mode needs an invite code, and the code is missing or used.
    InviteRequired,
    /// Another account has the email address.
    EmailInUse,
    /// The database failed.
    Database(String),
}

impl From<rusqlite::Error> for SignUpError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

/// A session row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRow {
    /// The account of the session.
    pub account_id: i64,
    /// The time of the sign-in.
    pub created_at: i64,
    /// The time of the last request that the server counted.
    pub last_seen: i64,
}

/// The database of the server.
#[derive(Debug)]
pub struct Accounts {
    db: Mutex<Connection>,
}

impl Accounts {
    /// Open `<data>/server.sqlite`, and make or migrate its schema.
    ///
    /// # Errors
    ///
    /// Gives a message when the file cannot be opened or has a newer
    /// schema.
    pub fn open(data_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|e| format!("{}: {e}", data_dir.display()))?;
        let db = Connection::open(data_dir.join("server.sqlite")).map_err(|e| e.to_string())?;
        Self::prepare(db)
    }

    /// Open a database in memory, for tests.
    ///
    /// # Errors
    ///
    /// Gives a message when SQLite fails.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        Self::prepare(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn prepare(mut db: Connection) -> Result<Self, String> {
        db.pragma_update(None, "foreign_keys", true)
            .map_err(|e| e.to_string())?;
        let current: i64 = db
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let known = i64::try_from(MIGRATIONS.len()).unwrap_or(i64::MAX);
        if current > known {
            return Err(format!(
                "server.sqlite has schema version {current}, and this server knows only {known}"
            ));
        }
        let start = usize::try_from(current).map_err(|_| "the schema version is negative")?;
        for (index, migration) in MIGRATIONS.iter().enumerate().skip(start) {
            let tx = db.transaction().map_err(|e| e.to_string())?;
            tx.execute_batch(migration).map_err(|e| e.to_string())?;
            tx.pragma_update(
                None,
                "user_version",
                i64::try_from(index + 1).unwrap_or(i64::MAX),
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        }
        Ok(Self { db: Mutex::new(db) })
    }

    fn db(&self) -> MutexGuard<'_, Connection> {
        // A poisoned lock only means that another thread failed. The
        // connection is still usable.
        self.db
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Make a single-use invite code. The database keeps its SHA-256 only.
    ///
    /// # Errors
    ///
    /// Gives a message when the random source or the database fails.
    pub fn create_invite(&self) -> Result<String, String> {
        let code = random_hex(16)?;
        self.db()
            .execute(
                "INSERT INTO invites (code_hash, created_at) VALUES (?1, ?2)",
                params![sha256_hex(&code), now()],
            )
            .map_err(|e| e.to_string())?;
        Ok(code)
    }

    /// Make an account in one database step. With `invite`, the step
    /// checks that the code is unused and marks it used. Gives the ID of
    /// the account.
    ///
    /// # Errors
    ///
    /// Gives [`SignUpError`] when the invite is missing or used, when the
    /// email address is in use, or when the database fails.
    pub fn sign_up(
        &self,
        invite: Option<&str>,
        email: &str,
        folder: &str,
    ) -> Result<i64, SignUpError> {
        let mut db = self.db();
        let tx = db.transaction()?;
        let time = now();
        if let Some(code) = invite {
            let used = tx.execute(
                "UPDATE invites SET used_at = ?2 WHERE code_hash = ?1 AND used_at IS NULL",
                params![sha256_hex(code), time],
            )?;
            if used == 0 {
                return Err(SignUpError::InviteRequired);
            }
        }
        let taken: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM account_emails WHERE email = ?1)",
            [email],
            |r| r.get(0),
        )?;
        if taken {
            // The step ends here, so the invite stays unused.
            return Err(SignUpError::EmailInUse);
        }
        tx.execute(
            "INSERT INTO accounts (folder, created_at) VALUES (?1, ?2)",
            params![folder, time],
        )?;
        let id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO account_emails (account_id, email) VALUES (?1, ?2)",
            params![id, email],
        )?;
        tx.commit()?;
        Ok(id)
    }

    /// Remove an account that the server could not finish, and give its
    /// invite code back.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn undo_sign_up(&self, account_id: i64, invite: Option<&str>) -> Result<(), String> {
        let db = self.db();
        db.execute("DELETE FROM accounts WHERE id = ?1", [account_id])
            .map_err(|e| e.to_string())?;
        if let Some(code) = invite {
            db.execute(
                "UPDATE invites SET used_at = NULL WHERE code_hash = ?1",
                [sha256_hex(code)],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// The account ID and the folder of an email address.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn account_by_email(&self, email: &str) -> Result<Option<(i64, String)>, String> {
        self.db()
            .query_row(
                "SELECT a.id, a.folder FROM accounts a
                 JOIN account_emails e ON e.account_id = a.id WHERE e.email = ?1",
                [email],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// The email address of an account.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn email_of(&self, account_id: i64) -> Result<Option<String>, String> {
        self.db()
            .query_row(
                "SELECT email FROM account_emails WHERE account_id = ?1",
                [account_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// The ID and the folder of each account.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn folders(&self) -> Result<Vec<(i64, String)>, String> {
        let db = self.db();
        let mut statement = db
            .prepare("SELECT id, folder FROM accounts ORDER BY id")
            .map_err(|e| e.to_string())?;
        statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .and_then(Iterator::collect)
            .map_err(|e| e.to_string())
    }

    /// The folder of an account.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn folder_of(&self, account_id: i64) -> Result<Option<String>, String> {
        self.db()
            .query_row(
                "SELECT folder FROM accounts WHERE id = ?1",
                [account_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// Keep a session. The caller gives the SHA-256 of the token.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn create_session(&self, token_hash: &str, account_id: i64) -> Result<(), String> {
        let time = now();
        self.db()
            .execute(
                "INSERT INTO sessions (token_hash, account_id, created_at, last_seen)
                 VALUES (?1, ?2, ?3, ?3)",
                params![token_hash, account_id, time],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// The session row of a token hash.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn session(&self, token_hash: &str) -> Result<Option<SessionRow>, String> {
        self.db()
            .query_row(
                "SELECT account_id, created_at, last_seen FROM sessions WHERE token_hash = ?1",
                [token_hash],
                |r| {
                    Ok(SessionRow {
                        account_id: r.get(0)?,
                        created_at: r.get(1)?,
                        last_seen: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// Set `last_seen` to `time` when the old value is at least `gap`
    /// seconds older.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn touch_session(&self, token_hash: &str, time: i64, gap: i64) -> Result<(), String> {
        self.db()
            .execute(
                "UPDATE sessions SET last_seen = ?2 WHERE token_hash = ?1 AND last_seen <= ?3",
                params![token_hash, time, time - gap],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Remove a session.
    ///
    /// # Errors
    ///
    /// Gives a message when the database fails.
    pub fn delete_session(&self, token_hash: &str) -> Result<(), String> {
        self.db()
            .execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invite_works_one_time_and_the_database_keeps_its_hash() {
        let accounts = Accounts::open_in_memory().unwrap();
        let code = accounts.create_invite().unwrap();
        assert_eq!(code.len(), 32);
        let id = accounts
            .sign_up(Some(&code), "a@example.org", "f1")
            .unwrap();
        assert_eq!(
            accounts.sign_up(Some(&code), "b@example.org", "f2"),
            Err(SignUpError::InviteRequired)
        );
        assert_eq!(accounts.folder_of(id).unwrap().as_deref(), Some("f1"));
        let stored: String = accounts
            .db()
            .query_row("SELECT code_hash FROM invites", [], |r| r.get(0))
            .unwrap();
        assert_ne!(stored, code);
        assert_eq!(stored, sha256_hex(&code));
    }

    #[test]
    fn a_used_email_keeps_the_invite_unused() {
        let accounts = Accounts::open_in_memory().unwrap();
        accounts.sign_up(None, "a@example.org", "f1").unwrap();
        let code = accounts.create_invite().unwrap();
        assert_eq!(
            accounts.sign_up(Some(&code), "a@example.org", "f2"),
            Err(SignUpError::EmailInUse)
        );
        assert!(accounts.sign_up(Some(&code), "b@example.org", "f2").is_ok());
    }

    #[test]
    fn undo_gives_the_invite_back() {
        let accounts = Accounts::open_in_memory().unwrap();
        let code = accounts.create_invite().unwrap();
        let id = accounts
            .sign_up(Some(&code), "a@example.org", "f1")
            .unwrap();
        accounts.undo_sign_up(id, Some(&code)).unwrap();
        assert!(
            accounts
                .account_by_email("a@example.org")
                .unwrap()
                .is_none()
        );
        assert!(accounts.sign_up(Some(&code), "a@example.org", "f1").is_ok());
    }

    #[test]
    fn a_session_is_found_by_its_hash_and_touched_at_most_once_in_the_gap() {
        let accounts = Accounts::open_in_memory().unwrap();
        let id = accounts.sign_up(None, "a@example.org", "f1").unwrap();
        accounts.create_session("h", id).unwrap();
        let first = accounts.session("h").unwrap().unwrap();
        assert_eq!(first.account_id, id);
        accounts
            .touch_session("h", first.last_seen + 30, 60)
            .unwrap();
        assert_eq!(
            accounts.session("h").unwrap().unwrap().last_seen,
            first.last_seen
        );
        accounts
            .touch_session("h", first.last_seen + 61, 60)
            .unwrap();
        assert_eq!(
            accounts.session("h").unwrap().unwrap().last_seen,
            first.last_seen + 61
        );
        accounts.delete_session("h").unwrap();
        assert!(accounts.session("h").unwrap().is_none());
    }

    #[test]
    fn the_schema_version_is_kept() {
        let accounts = Accounts::open_in_memory().unwrap();
        let version: i64 = accounts
            .db()
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(usize::try_from(version).unwrap(), MIGRATIONS.len());
    }
}
