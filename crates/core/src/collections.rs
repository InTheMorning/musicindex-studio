//! The collections of the desktop app (ADR 0149).
//!
//! The default data folder holds each collection in `collections/{folder}/`.
//! The file `collections.json` gives the names and the collection that is
//! open. Each collection is one complete data folder, with its own vault. A
//! server has no such list. This module has only file work and SQLite.

use std::fmt;
use std::path::{Path, PathBuf};

use rusqlite::OptionalExtension as _;
use serde::{Deserialize, Serialize};

use crate::account_archive::{LibraryPolicy, clear_queues, reset_library_root};
use crate::collection::Collection;

/// The folder of the first collection.
pub const MAIN_FOLDER: &str = "main";
/// The name of the first collection (ADR 0149 item 2).
pub const MAIN_NAME: &str = "Main collection";
/// The file with the list. The move at the first start writes it last, so
/// the file also tells that the move is complete.
const LIST_FILE: &str = "collections.json";
const COLLECTIONS: &str = "collections";
const LIBRARY: &str = "library";
const DATABASE: &str = "collection.sqlite";
/// The lock of a data folder. `studio-desktop` owns its use.
const LOCK_FILE: &str = "studio.lock";
/// The entries of the default data folder that stay in it at the move. The
/// choice for a close of the window belongs to the app, not to a collection.
const STAY_IN_ROOT: [&str; 5] = [
    LOCK_FILE,
    COLLECTIONS,
    LIST_FILE,
    "collections.json.new",
    "window-close",
];
/// The entries of a collection that a copy leaves out: temporary work, the
/// grant vaults and the database, which the copy makes with SQLite.
const NOT_IN_COPY: [&str; 8] = [
    LOCK_FILE,
    DATABASE,
    "collection.sqlite-wal",
    "collection.sqlite-shm",
    "grants",
    "import-staging",
    "imports",
    "incoming",
];
/// The longest name of a collection, in characters.
const MAX_NAME: usize = 60;

/// The kind of a [`CollectionsError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectionsErrorKind {
    /// The name is empty or too long.
    InvalidName,
    /// A different collection has this name.
    NameTaken,
    /// No collection has this folder.
    NotFound,
    /// The operation does not apply to this collection (ADR 0149 item 12).
    Refused,
    /// The move at the first start found a file at its destination.
    MoveConflict,
    /// A file or SQLite failed.
    Io,
}

/// An error of the collections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionsError {
    kind: CollectionsErrorKind,
    message: String,
}

impl CollectionsError {
    fn new(kind: CollectionsErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn io(path: &Path, error: &dyn fmt::Display) -> Self {
        Self::new(
            CollectionsErrorKind::Io,
            format!("{}: {error}", path.display()),
        )
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> CollectionsErrorKind {
        self.kind
    }
}

impl fmt::Display for CollectionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CollectionsError {}

/// One collection of the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionEntry {
    /// The name of its folder in `collections/`.
    pub folder: String,
    /// The name that the user gave.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListFile {
    open: String,
    collections: Vec<CollectionEntry>,
}

/// The collections in one default data folder.
#[derive(Debug, Clone)]
pub struct Collections {
    root: PathBuf,
}

impl Collections {
    /// The collections in the default data folder `root`. The call does no
    /// file work.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    fn read(&self) -> Result<ListFile, CollectionsError> {
        let path = self.root.join(LIST_FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| CollectionsError::io(&path, &e))?;
        serde_json::from_str(&text).map_err(|e| CollectionsError::io(&path, &e))
    }

    fn write(&self, list: &ListFile) -> Result<(), CollectionsError> {
        write_list(&self.root, list)
    }

    /// The folder of a collection.
    #[must_use]
    pub fn folder(&self, folder: &str) -> PathBuf {
        self.root.join(COLLECTIONS).join(folder)
    }

    /// The list of the collections, in the sequence of their start.
    ///
    /// # Errors
    ///
    /// Gives an error when the list file cannot be read.
    pub fn list(&self) -> Result<Vec<CollectionEntry>, CollectionsError> {
        Ok(self.read()?.collections)
    }

    /// The folder name of the collection that is open.
    ///
    /// # Errors
    ///
    /// Gives an error when the list file cannot be read.
    pub fn open(&self) -> Result<String, CollectionsError> {
        Ok(self.read()?.open)
    }

    /// Select the collection that opens at the next start (ADR 0149 item 14).
    ///
    /// # Errors
    ///
    /// Gives `NotFound` for an unknown folder, and `Io` when the write fails.
    pub fn set_open(&self, folder: &str) -> Result<(), CollectionsError> {
        let mut list = self.read()?;
        if !list.collections.iter().any(|c| c.folder == folder) {
            return Err(not_found(folder));
        }
        folder.clone_into(&mut list.open);
        self.write(&list)
    }

    /// The count of the feeds of a collection, or 0 when it has no database.
    ///
    /// # Errors
    ///
    /// Gives `Io` when SQLite fails.
    pub fn feed_count(&self, folder: &str) -> Result<u64, CollectionsError> {
        let path = self.folder(folder).join(DATABASE);
        if !path.exists() {
            return Ok(0);
        }
        let db = rusqlite::Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| CollectionsError::io(&path, &e))?;
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM feeds", [], |r| r.get(0))
            .map_err(|e| CollectionsError::io(&path, &e))?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// Make a new collection, with no vault and no feed (ADR 0149 item 7).
    ///
    /// # Errors
    ///
    /// Gives `InvalidName` or `NameTaken` for the name, and `Io` when a file
    /// fails.
    pub fn create(&self, name: &str) -> Result<CollectionEntry, CollectionsError> {
        let mut list = self.read()?;
        let name = checked_name(&list, name)?;
        let folder = free_folder(&self.root, &list, &name);
        let path = self.folder(&folder);
        private_folder(&path)?;
        // A new collection keeps its library in its own folder, so that it
        // shares no release folder with a different collection.
        let database = path.join(DATABASE);
        Collection::open(&database)
            .and_then(|db| db.set_setting("library_root", &serde_json::json!(LIBRARY)))
            .map_err(|e| CollectionsError::io(&database, &e))?;
        let entry = CollectionEntry { folder, name };
        list.collections.push(entry.clone());
        self.write(&list)?;
        Ok(entry)
    }

    /// The size in bytes of a copy of the collection in `source` (ADR 0149
    /// item 9). The size includes a library outside that folder.
    ///
    /// # Errors
    ///
    /// Gives `Io` when a file or SQLite fails.
    pub fn copy_size(&self, source: &Path) -> Result<u64, CollectionsError> {
        let mut total = folder_size(source, &NOT_IN_COPY)?;
        let database = source.join(DATABASE);
        if let Ok(meta) = std::fs::metadata(&database) {
            total += meta.len();
        }
        if let Some(library) = outer_library(source)? {
            total += folder_size(&library, &[])?;
        }
        Ok(total)
    }

    /// Copy the collection in `source` to a new collection (ADR 0149 items 8
    /// to 10). The copy keeps the vault, so it has the passphrase of its
    /// source. It holds no connected app, no show grant and no queued
    /// announcement. A library outside `source` goes into the copy. The list
    /// changes last, and a failure removes the new folder.
    ///
    /// # Errors
    ///
    /// Gives `InvalidName` or `NameTaken` for the name, and `Io` when a file
    /// or SQLite fails.
    pub fn copy(&self, source: &Path, name: &str) -> Result<CollectionEntry, CollectionsError> {
        let mut list = self.read()?;
        let name = checked_name(&list, name)?;
        let folder = free_folder(&self.root, &list, &name);
        let path = self.folder(&folder);
        private_folder(&path)?;
        if let Err(error) = fill_copy(source, &path) {
            let _ = std::fs::remove_dir_all(&path);
            return Err(error);
        }
        let entry = CollectionEntry { folder, name };
        list.collections.push(entry.clone());
        self.write(&list)?;
        Ok(entry)
    }

    /// Remove a collection and its local files (ADR 0149 items 12 and 13).
    /// Nothing changes on a file host. The list changes first, so a stop
    /// leaves an unused folder and no entry with no folder.
    ///
    /// # Errors
    ///
    /// Gives `Refused` for "Main collection", for the open collection and
    /// for a folder that a shell holds. Gives `NotFound` for an unknown
    /// folder, and `Io` when a file fails.
    pub fn delete(&self, folder: &str) -> Result<(), CollectionsError> {
        let mut list = self.read()?;
        if folder == MAIN_FOLDER {
            return Err(refused("Studio cannot delete the main collection."));
        }
        if folder == list.open {
            return Err(refused("Studio cannot delete the open collection."));
        }
        let before = list.collections.len();
        list.collections.retain(|c| c.folder != folder);
        if list.collections.len() == before {
            return Err(not_found(folder));
        }
        let path = self.folder(folder);
        if is_held(&path) {
            return Err(refused("A different Studio uses this collection."));
        }
        self.write(&list)?;
        std::fs::remove_dir_all(&path).map_err(|e| CollectionsError::io(&path, &e))
    }
}

/// Make the list and do the move of a new version, then give the folder of
/// the collection that is open (ADR 0149 items 1 to 3). The caller holds the
/// lock of `root`. A complete move changes nothing.
///
/// # Errors
///
/// Gives `MoveConflict` when a file is at its destination, and `Io` when a
/// file or SQLite fails. Studio deletes nothing, and the next start continues.
pub fn prepare(root: &Path) -> Result<PathBuf, CollectionsError> {
    prepare_with_steps(root, &mut |_| Ok(()))
}

/// [`prepare`] with a call before each step, so that a test can stop the
/// move at each step.
fn prepare_with_steps(
    root: &Path,
    step: &mut dyn FnMut(usize) -> std::io::Result<()>,
) -> Result<PathBuf, CollectionsError> {
    let collections = Collections::new(root);
    if root.join(LIST_FILE).exists() {
        let open = collections.open()?;
        return Ok(collections.folder(&open));
    }
    let mut count = 0;
    let mut next = |path: &Path| {
        count += 1;
        step(count).map_err(|e| CollectionsError::io(path, &e))
    };
    let main = collections.folder(MAIN_FOLDER);
    next(root)?;
    private_folder(root)?;
    private_folder(&root.join(COLLECTIONS))?;
    private_folder(&main)?;
    let entries = std::fs::read_dir(root).map_err(|e| CollectionsError::io(root, &e))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| CollectionsError::io(root, &e))?;
        let name = entry.file_name();
        if !STAY_IN_ROOT.iter().any(|keep| name == *keep) {
            names.push(name);
        }
    }
    names.sort();
    // Each rename is complete or absent, so the next start moves the rest.
    for name in names {
        let from = root.join(&name);
        let to = main.join(&name);
        if to.exists() {
            return Err(CollectionsError::new(
                CollectionsErrorKind::MoveConflict,
                format!(
                    "Studio could not move {} because {} exists. Studio deleted nothing.",
                    from.display(),
                    to.display()
                ),
            ));
        }
        next(&from)?;
        std::fs::rename(&from, &to).map_err(|e| CollectionsError::io(&from, &e))?;
    }
    next(root)?;
    change_stored_paths(root, &main)?;
    next(root)?;
    write_list(
        root,
        &ListFile {
            open: MAIN_FOLDER.to_owned(),
            collections: vec![CollectionEntry {
                folder: MAIN_FOLDER.to_owned(),
                name: MAIN_NAME.to_owned(),
            }],
        },
    )?;
    Ok(main)
}

/// Change each stored absolute path into `root`, after the move to `main`
/// (ADR 0149 item 3). The call can run again with no change. A path outside
/// `root` stays.
fn change_stored_paths(root: &Path, main: &Path) -> Result<(), CollectionsError> {
    let path = main.join(DATABASE);
    if !path.exists() {
        return Ok(());
    }
    let fail = |e: rusqlite::Error| CollectionsError::io(&path, &e);
    let db = rusqlite::Connection::open(&path).map_err(fail)?;
    let moved = |text: &str| -> Option<PathBuf> {
        let old = Path::new(text);
        // The folder of a collection is below `root`, so a changed path
        // does not change again.
        if !old.is_absolute() || old.starts_with(main) {
            return None;
        }
        old.strip_prefix(root).ok().map(Path::to_path_buf)
    };
    if table_exists(&db, "settings").map_err(fail)? {
        let value: Option<String> = db
            .query_row(
                "SELECT value FROM settings WHERE key = 'library_root'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(fail)?;
        let old = value.and_then(|text| serde_json::from_str::<String>(&text).ok());
        if let Some(inside) = old.as_deref().and_then(moved) {
            // A library in the data folder is a relative path (ADR 0107 item 1).
            let text = inside.to_string_lossy().into_owned();
            db.execute(
                "UPDATE settings SET value = ?1 WHERE key = 'library_root'",
                [serde_json::Value::from(text).to_string()],
            )
            .map_err(fail)?;
        }
    }
    if table_exists(&db, "connectors").map_err(fail)? {
        let rows: Vec<(i64, String)> = {
            let mut statement = db
                .prepare("SELECT id, config FROM connectors")
                .map_err(fail)?;
            let found = statement
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(fail)?;
            found.collect::<Result<_, _>>().map_err(fail)?
        };
        for (id, config) in rows {
            let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&config) else {
                continue;
            };
            let inside = value
                .get("root")
                .and_then(serde_json::Value::as_str)
                .and_then(moved);
            if let Some(inside) = inside {
                value["root"] = main.join(inside).to_string_lossy().into_owned().into();
                db.execute(
                    "UPDATE connectors SET config = ?1 WHERE id = ?2",
                    rusqlite::params![value.to_string(), id],
                )
                .map_err(fail)?;
            }
        }
    }
    Ok(())
}

fn table_exists(db: &rusqlite::Connection, name: &str) -> rusqlite::Result<bool> {
    db.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [name],
        |r| r.get(0),
    )
}

/// Write the list file in one step.
fn write_list(root: &Path, list: &ListFile) -> Result<(), CollectionsError> {
    let path = root.join(LIST_FILE);
    let new = root.join("collections.json.new");
    let text = serde_json::to_string_pretty(list).map_err(|e| CollectionsError::io(&path, &e))?;
    std::fs::write(&new, text).map_err(|e| CollectionsError::io(&new, &e))?;
    std::fs::rename(&new, &path).map_err(|e| CollectionsError::io(&path, &e))
}

/// Make a folder with the mode 0700 (ADR 0104 item 2).
fn private_folder(path: &Path) -> Result<(), CollectionsError> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder
        .create(path)
        .map_err(|e| CollectionsError::io(path, &e))
}

fn not_found(folder: &str) -> CollectionsError {
    CollectionsError::new(
        CollectionsErrorKind::NotFound,
        format!("no collection has the folder {folder}"),
    )
}

fn refused(message: &str) -> CollectionsError {
    CollectionsError::new(CollectionsErrorKind::Refused, message)
}

/// The trimmed name, when it is valid and free.
fn checked_name(list: &ListFile, name: &str) -> Result<String, CollectionsError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(CollectionsError::new(
            CollectionsErrorKind::InvalidName,
            format!("a name has 1 to {MAX_NAME} characters"),
        ));
    }
    if list
        .collections
        .iter()
        .any(|c| c.name.eq_ignore_ascii_case(name))
    {
        return Err(CollectionsError::new(
            CollectionsErrorKind::NameTaken,
            "a collection has this name",
        ));
    }
    Ok(name.to_owned())
}

/// A folder name from `name` that no collection and no file uses.
fn free_folder(root: &Path, list: &ListFile, name: &str) -> String {
    let plain: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let plain = plain
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let base = if plain.is_empty() {
        "collection".to_owned()
    } else {
        plain
    };
    let free = |folder: &str| {
        !list.collections.iter().any(|c| c.folder == folder)
            && !root.join(COLLECTIONS).join(folder).exists()
    };
    if free(&base) {
        return base;
    }
    (2..=u32::MAX)
        .map(|n| format!("{base}-{n}"))
        .find(|folder| free(folder))
        .unwrap_or(base)
}

/// True when a shell holds the lock of this folder.
fn is_held(path: &Path) -> bool {
    let Ok(file) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join(LOCK_FILE))
    else {
        return false;
    };
    matches!(file.try_lock(), Err(std::fs::TryLockError::WouldBlock))
}

/// The library of a collection when it is outside the folder of that
/// collection.
fn outer_library(source: &Path) -> Result<Option<PathBuf>, CollectionsError> {
    let path = source.join(DATABASE);
    if !path.exists() {
        return Ok(None);
    }
    let fail = |e: rusqlite::Error| CollectionsError::io(&path, &e);
    let db =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(fail)?;
    if !table_exists(&db, "settings").map_err(fail)? {
        return Ok(None);
    }
    let value: Option<String> = db
        .query_row(
            "SELECT value FROM settings WHERE key = 'library_root'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(fail)?;
    let root = value
        .and_then(|text| serde_json::from_str::<String>(&text).ok())
        .map(PathBuf::from)
        .filter(|root| root.is_absolute() && !root.starts_with(source));
    Ok(root)
}

fn fill_copy(source: &Path, target: &Path) -> Result<(), CollectionsError> {
    let outer = outer_library(source)?;
    copy_folder(source, target, &NOT_IN_COPY)?;
    let database = source.join(DATABASE);
    if database.exists() {
        // The source can be open, so SQLite makes the copy.
        let fail = |e: rusqlite::Error| CollectionsError::io(&database, &e);
        let db = rusqlite::Connection::open_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(fail)?;
        let new = target.join(DATABASE);
        db.execute("VACUUM INTO ?1", [new.to_string_lossy().as_ref()])
            .map_err(fail)?;
        if let Some(library) = &outer {
            let into = target.join(LIBRARY);
            private_folder(&into)?;
            copy_folder(library, &into, &[])?;
        }
        clear_queues(target).map_err(|e| CollectionsError::io(target, &e))?;
        reset_library_root(target, LibraryPolicy::Relative)
            .map_err(|e| CollectionsError::io(target, &e))?;
        clear_apps(&new)?;
    }
    Ok(())
}

/// Remove the connected apps and the show grants of a copy (ADR 0149 item 10).
fn clear_apps(path: &Path) -> Result<(), CollectionsError> {
    let fail = |e: rusqlite::Error| CollectionsError::io(path, &e);
    let db = rusqlite::Connection::open(path).map_err(fail)?;
    for table in [
        "app_events",
        "grant_writes",
        "draft_grants",
        "show_grants",
        "connected_apps",
    ] {
        if table_exists(&db, table).map_err(fail)? {
            db.execute(&format!("DELETE FROM {table}"), [])
                .map_err(fail)?;
        }
    }
    Ok(())
}

fn copy_folder(from: &Path, to: &Path, leave_out: &[&str]) -> Result<(), CollectionsError> {
    let entries = std::fs::read_dir(from).map_err(|e| CollectionsError::io(from, &e))?;
    for entry in entries {
        let entry = entry.map_err(|e| CollectionsError::io(from, &e))?;
        let name = entry.file_name();
        if leave_out.iter().any(|skip| name == *skip) {
            continue;
        }
        let source = entry.path();
        let target = to.join(&name);
        let kind = entry
            .file_type()
            .map_err(|e| CollectionsError::io(&source, &e))?;
        if kind.is_dir() {
            private_folder(&target)?;
            copy_folder(&source, &target, &[])?;
        } else if kind.is_file() {
            std::fs::copy(&source, &target).map_err(|e| CollectionsError::io(&source, &e))?;
        }
        // A link goes outside the folder, so a copy leaves it out.
    }
    Ok(())
}

fn folder_size(folder: &Path, leave_out: &[&str]) -> Result<u64, CollectionsError> {
    let mut total = 0;
    let entries = std::fs::read_dir(folder).map_err(|e| CollectionsError::io(folder, &e))?;
    for entry in entries {
        let entry = entry.map_err(|e| CollectionsError::io(folder, &e))?;
        if leave_out.iter().any(|skip| entry.file_name() == *skip) {
            continue;
        }
        let meta = entry
            .metadata()
            .map_err(|e| CollectionsError::io(&entry.path(), &e))?;
        if meta.is_dir() {
            total += folder_size(&entry.path(), &[])?;
        } else {
            total += meta.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A default data folder of an earlier version: a real database, a vault
    /// file, a library file and a local file host in the data folder.
    fn old_folder(root: &Path) {
        std::fs::create_dir_all(root.join("library/release")).unwrap();
        std::fs::create_dir_all(root.join("hosted")).unwrap();
        std::fs::write(root.join("vault.json"), "vault").unwrap();
        std::fs::write(root.join("library/release/a.mp3"), "audio").unwrap();
        std::fs::write(root.join("window-close"), "tray").unwrap();
        std::fs::write(root.join(LOCK_FILE), "").unwrap();
        let db = Collection::open(&root.join(DATABASE)).unwrap();
        db.set_setting(
            "library_root",
            &serde_json::json!(root.join("library").to_string_lossy()),
        )
        .unwrap();
        drop(db);
        let raw = rusqlite::Connection::open(root.join(DATABASE)).unwrap();
        raw.execute(
            "INSERT INTO connectors (kind, name, config, created_at)
             VALUES ('local', 'Home', ?1, 1), ('local', 'Far', ?2, 1)",
            [
                serde_json::json!({ "root": root.join("hosted").to_string_lossy() }).to_string(),
                serde_json::json!({ "root": "/elsewhere/music" }).to_string(),
            ],
        )
        .unwrap();
    }

    fn check_moved(root: &Path) {
        let main = root.join("collections/main");
        assert_eq!(
            std::fs::read_to_string(main.join("vault.json")).unwrap(),
            "vault"
        );
        assert_eq!(
            std::fs::read_to_string(main.join("library/release/a.mp3")).unwrap(),
            "audio"
        );
        assert!(!root.join("vault.json").exists());
        assert!(!root.join(DATABASE).exists());
        assert_eq!(
            std::fs::read_to_string(root.join("window-close")).unwrap(),
            "tray",
            "ADR 0149: the choice for a close belongs to the app."
        );
        let db = Collection::open(&main.join(DATABASE)).unwrap();
        assert_eq!(
            db.library_root().unwrap().unwrap(),
            main.join("library"),
            "ADR 0149 item 3: the library path must follow the move."
        );
        drop(db);
        let raw = rusqlite::Connection::open(main.join(DATABASE)).unwrap();
        let configs: Vec<String> = raw
            .prepare("SELECT config FROM connectors ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert!(
            configs[0].contains(main.join("hosted").to_str().unwrap()),
            "ADR 0149 item 3: a file host in the data folder must follow the move: {}",
            configs[0]
        );
        assert!(configs[1].contains("/elsewhere/music"));
    }

    #[test]
    fn the_move_gives_collections_main_and_a_second_start_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        old_folder(dir.path());
        let open = prepare(dir.path()).unwrap();
        assert_eq!(open, dir.path().join("collections/main"));
        check_moved(dir.path());
        let list = Collections::new(dir.path()).list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, MAIN_NAME);
        let before = std::fs::read_to_string(dir.path().join(LIST_FILE)).unwrap();
        assert_eq!(prepare(dir.path()).unwrap(), open);
        check_moved(dir.path());
        assert_eq!(
            std::fs::read_to_string(dir.path().join(LIST_FILE)).unwrap(),
            before
        );
    }

    #[test]
    fn a_stop_at_each_step_of_the_move_completes_at_the_next_start() {
        for stops in 0.. {
            let dir = tempfile::tempdir().unwrap();
            old_folder(dir.path());
            let result = prepare_with_steps(dir.path(), &mut |step| {
                if step == stops + 1 {
                    Err(std::io::Error::other("stopped"))
                } else {
                    Ok(())
                }
            });
            if result.is_ok() {
                assert!(stops > 4, "the move must have more steps: {stops}");
                return;
            }
            assert!(
                !dir.path().join(LIST_FILE).exists(),
                "ADR 0149 item 3: the list comes after each other step."
            );
            prepare(dir.path()).unwrap();
            check_moved(dir.path());
        }
    }

    #[test]
    fn a_file_at_the_destination_stops_the_move_and_deletes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        old_folder(dir.path());
        std::fs::create_dir_all(dir.path().join("collections/main")).unwrap();
        std::fs::write(dir.path().join("collections/main/vault.json"), "other").unwrap();
        let error = prepare(dir.path()).unwrap_err();
        assert_eq!(error.kind(), CollectionsErrorKind::MoveConflict);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("vault.json")).unwrap(),
            "vault"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("collections/main/vault.json")).unwrap(),
            "other"
        );
        assert!(!dir.path().join(LIST_FILE).exists());
    }

    #[test]
    fn a_new_install_gets_an_empty_main_collection() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("new");
        assert_eq!(prepare(&root).unwrap(), root.join("collections/main"));
        assert!(root.join("collections/main").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn the_folders_have_the_mode_0700() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("new");
        prepare(&root).unwrap();
        let made = Collections::new(&root).create("Test").unwrap();
        for path in [
            root.join("collections"),
            root.join("collections/main"),
            root.join("collections").join(made.folder),
        ] {
            let bits = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(bits, 0o700, "ADR 0104 item 2: {}", path.display());
        }
    }

    fn ready() -> (tempfile::TempDir, Collections) {
        let dir = tempfile::tempdir().unwrap();
        prepare(dir.path()).unwrap();
        let collections = Collections::new(dir.path());
        (dir, collections)
    }

    #[test]
    fn a_new_collection_has_its_own_empty_folder_and_a_free_name() {
        let (_dir, collections) = ready();
        let made = collections.create("  Test: new start ").unwrap();
        assert_eq!(made.name, "Test: new start");
        assert_eq!(made.folder, "test-new-start");
        assert!(collections.folder(&made.folder).is_dir());
        assert!(!collections.folder(&made.folder).join("vault.json").exists());
        let db = Collection::open(&collections.folder(&made.folder).join(DATABASE)).unwrap();
        assert_eq!(
            db.library_root().unwrap().unwrap(),
            collections.folder(&made.folder).join(LIBRARY),
            "A new collection must share no library with a different collection."
        );
        drop(db);
        assert_eq!(collections.feed_count(&made.folder).unwrap(), 0);
        assert_eq!(
            collections.create("test: new START").unwrap_err().kind(),
            CollectionsErrorKind::NameTaken
        );
        assert_eq!(
            collections.create("  ").unwrap_err().kind(),
            CollectionsErrorKind::InvalidName
        );
        let same_folder = collections.create("Test new start").unwrap();
        assert_eq!(same_folder.folder, "test-new-start-2");
        let symbols = collections.create("★").unwrap();
        assert_eq!(symbols.folder, "collection");
    }

    #[test]
    fn a_copy_keeps_the_vault_and_holds_no_app_grant_or_queue() {
        let (dir, collections) = ready();
        let main = collections.folder(MAIN_FOLDER);
        std::fs::write(main.join("vault.json"), "vault-a").unwrap();
        std::fs::create_dir_all(main.join("grants")).unwrap();
        std::fs::write(main.join("grants/1.json"), "grant").unwrap();
        std::fs::create_dir_all(main.join("library/r")).unwrap();
        std::fs::write(main.join("library/r/a.mp3"), "audio").unwrap();
        std::fs::write(main.join(LOCK_FILE), "").unwrap();
        let db = Collection::open(&main.join(DATABASE)).unwrap();
        db.set_setting("library_root", &serde_json::json!("library"))
            .unwrap();
        drop(db);
        let raw = rusqlite::Connection::open(main.join(DATABASE)).unwrap();
        for table in ["connected_apps", "announcements"] {
            let columns: Vec<(String, String)> = raw
                .prepare(&format!("PRAGMA table_info({table})"))
                .unwrap()
                .query_map([], |r| Ok((r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let names: Vec<&str> = columns.iter().map(|c| c.0.as_str()).collect();
            let values: Vec<String> = columns
                .iter()
                .map(|(name, kind)| match (name.as_str(), kind.as_str()) {
                    ("state", _) => "'waiting'".to_owned(),
                    (_, "INTEGER") => "1".to_owned(),
                    _ => "'x'".to_owned(),
                })
                .collect();
            raw.execute("PRAGMA foreign_keys = OFF", []).ok();
            raw.execute(
                &format!(
                    "INSERT INTO {table} ({}) VALUES ({})",
                    names.join(", "),
                    values.join(", ")
                ),
                [],
            )
            .unwrap();
        }
        drop(raw);
        let size = collections.copy_size(&main).unwrap();
        assert!(size > 10, "the size must count the files: {size}");
        let made = collections.copy(&main, "Test: 4 releases").unwrap();
        let copy = collections.folder(&made.folder);
        assert_eq!(
            std::fs::read_to_string(copy.join("vault.json")).unwrap(),
            "vault-a",
            "ADR 0149 item 8: a copy has the passphrase of its source."
        );
        assert_eq!(
            std::fs::read_to_string(copy.join("library/r/a.mp3")).unwrap(),
            "audio"
        );
        assert!(!copy.join("grants").exists(), "ADR 0149 item 10");
        assert!(!copy.join(LOCK_FILE).exists());
        let raw = rusqlite::Connection::open(copy.join(DATABASE)).unwrap();
        for table in ["connected_apps", "show_grants", "announcements"] {
            let count: i64 = raw
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 0, "ADR 0149 item 10: {table}");
        }
        drop(raw);
        let source: i64 = rusqlite::Connection::open(main.join(DATABASE))
            .unwrap()
            .query_row("SELECT COUNT(*) FROM connected_apps", [], |r| r.get(0))
            .unwrap();
        assert_eq!(source, 1, "the source must not change");
        drop(dir);
    }

    #[test]
    fn a_library_outside_the_source_goes_into_the_copy() {
        let (_dir, collections) = ready();
        let main = collections.folder(MAIN_FOLDER);
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(outside.path().join("r")).unwrap();
        std::fs::write(outside.path().join("r/a.mp3"), "audio").unwrap();
        let db = Collection::open(&main.join(DATABASE)).unwrap();
        db.set_setting(
            "library_root",
            &serde_json::json!(outside.path().to_string_lossy()),
        )
        .unwrap();
        drop(db);
        assert!(collections.copy_size(&main).unwrap() >= 5);
        let made = collections.copy(&main, "Copy").unwrap();
        let copy = collections.folder(&made.folder);
        assert_eq!(
            std::fs::read_to_string(copy.join("library/r/a.mp3")).unwrap(),
            "audio"
        );
        let db = Collection::open(&copy.join(DATABASE)).unwrap();
        assert_eq!(db.library_root().unwrap().unwrap(), copy.join("library"));
        assert!(outside.path().join("r/a.mp3").exists());
    }

    #[test]
    fn a_delete_removes_only_that_folder_and_refuses_main_and_the_open_one() {
        let (_dir, collections) = ready();
        let other = collections.create("Other").unwrap();
        let open = collections.create("Open").unwrap();
        collections.set_open(&open.folder).unwrap();
        let main = collections.folder(MAIN_FOLDER);
        std::fs::write(main.join("vault.json"), "keep").unwrap();
        assert_eq!(
            collections.delete(MAIN_FOLDER).unwrap_err().kind(),
            CollectionsErrorKind::Refused
        );
        assert_eq!(
            collections.delete(&open.folder).unwrap_err().kind(),
            CollectionsErrorKind::Refused
        );
        assert_eq!(
            collections.delete("nothing").unwrap_err().kind(),
            CollectionsErrorKind::NotFound
        );
        collections.delete(&other.folder).unwrap();
        assert!(!collections.folder(&other.folder).exists());
        assert!(main.join("vault.json").exists());
        assert!(collections.folder(&open.folder).exists());
        let names: Vec<String> = collections
            .list()
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(names, [MAIN_NAME, "Open"]);
    }

    #[test]
    fn a_delete_refuses_a_folder_that_a_shell_holds() {
        let (_dir, collections) = ready();
        let other = collections.create("Other").unwrap();
        let lock =
            std::fs::File::create(collections.folder(&other.folder).join(LOCK_FILE)).unwrap();
        lock.try_lock().unwrap();
        assert_eq!(
            collections.delete(&other.folder).unwrap_err().kind(),
            CollectionsErrorKind::Refused
        );
        assert!(collections.folder(&other.folder).exists());
    }

    #[test]
    fn set_open_selects_a_known_collection_only() {
        let (dir, collections) = ready();
        let other = collections.create("Other").unwrap();
        collections.set_open(&other.folder).unwrap();
        assert_eq!(
            prepare(dir.path()).unwrap(),
            collections.folder(&other.folder)
        );
        assert_eq!(
            collections.set_open("nothing").unwrap_err().kind(),
            CollectionsErrorKind::NotFound
        );
    }
}
