//! The archive of an account (ADR 0107 items 6 and 11).
//!
//! An archive is one `tar` stream. It holds `manifest.json` first, then
//! `vault.json`, `grants/`, `collection.sqlite` and `library/`. The vault
//! files go in as they are. The manifest holds no secret and no wrapper.
//!
//! The module never calls an `unpack` function of the `tar` crate. It checks
//! each entry, and then it writes the file itself. A refused archive leaves
//! no file in the target folder.

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::collection::SCHEMA_VERSION;

/// The name of the manifest entry.
const MANIFEST: &str = "manifest.json";
/// The name of the vault file.
const VAULT: &str = "vault.json";
/// The name of the collection file.
const COLLECTION: &str = "collection.sqlite";
/// The folder of the grant vaults.
const GRANTS: &str = "grants";
/// The folder of the library.
const LIBRARY: &str = "library";

/// The limits of one archive. A hostile archive cannot fill the disk
/// without bound (ADR 0107 item 15).
///
/// The server also applies its storage limit before it calls the core. These
/// limits are the backstop of the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The largest number of entries.
    pub max_entries: u64,
    /// The largest sum of the entry sizes, in bytes.
    pub max_total_bytes: u64,
}

impl Limits {
    /// 200 000 entries and 32 GiB.
    pub const DEFAULT: Self = Self {
        max_entries: 200_000,
        max_total_bytes: 32 * 1024 * 1024 * 1024,
    };
}

/// The largest size of the manifest entry, in bytes.
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// `manifest.json`. It holds no secret and no wrapper (ADR 0107 item 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The format version of the archive. This Studio writes
    /// [`Manifest::FORMAT_VERSION`].
    pub format_version: u32,
    /// The Studio version that made the archive.
    pub studio_version: String,
    /// The schema version of the collection in the archive.
    pub schema_version: usize,
    /// The Unix time of the export.
    pub exported_at: u64,
    /// The email of the account, when it has one.
    pub email: Option<String>,
    /// The public key of each Nostr method, as hexadecimal text.
    pub nostr_keys: Vec<String>,
}

impl Manifest {
    /// The archive format that this Studio writes and reads.
    pub const FORMAT_VERSION: u32 = 1;

    /// A manifest for an export now, with the versions of this Studio.
    #[must_use]
    pub fn new(email: Option<String>, nostr_keys: Vec<String>) -> Self {
        let exported_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self {
            format_version: Self::FORMAT_VERSION,
            studio_version: env!("CARGO_PKG_VERSION").to_owned(),
            schema_version: SCHEMA_VERSION,
            exported_at,
            email,
            nostr_keys,
        }
    }
}

/// The cause of a refusal (ADR 0107 item 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArchiveCause {
    /// The manifest has a format version that this Studio does not know.
    UnknownFormat,
    /// The collection schema is newer than this Studio.
    NewerSchema,
    /// An entry has an absolute path, a `..` segment or a bad name.
    UnsafePath,
    /// An entry is a symbolic link or a hard link.
    LinkEntry,
    /// An entry is a device file, a pipe or another special file.
    SpecialEntry,
    /// An entry is not one of the files of ADR 0107 item 6.
    UnexpectedEntry,
    /// Two entries have the same path.
    DuplicateEntry,
    /// The first entry is not `manifest.json`.
    MissingManifest,
    /// `manifest.json` is not valid.
    InvalidManifest,
    /// `vault.json` or `collection.sqlite` is not in the archive.
    MissingFile,
    /// The archive has more entries than the limit.
    TooManyEntries,
    /// The entries are larger than the limit.
    TooLarge,
    /// The `tar` stream is damaged.
    Unreadable,
    /// The target folder is not a new, empty folder.
    BadTarget,
    /// A local file operation failed. The archive is not at fault.
    Io,
}

impl ArchiveCause {
    /// A stable code for the API, in snake case.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::UnknownFormat => "unknown_format",
            Self::NewerSchema => "newer_schema",
            Self::UnsafePath => "unsafe_path",
            Self::LinkEntry => "link_entry",
            Self::SpecialEntry => "special_entry",
            Self::UnexpectedEntry => "unexpected_entry",
            Self::DuplicateEntry => "duplicate_entry",
            Self::MissingManifest => "missing_manifest",
            Self::InvalidManifest => "invalid_manifest",
            Self::MissingFile => "missing_file",
            Self::TooManyEntries => "too_many_entries",
            Self::TooLarge => "too_large",
            Self::Unreadable => "unreadable",
            Self::BadTarget => "bad_target",
            Self::Io => "io",
        }
    }
}

/// An error of the archive module.
#[derive(Debug, Clone)]
pub struct ArchiveError {
    cause: ArchiveCause,
    message: String,
}

impl ArchiveError {
    fn new(cause: ArchiveCause, message: impl Into<String>) -> Self {
        Self {
            cause,
            message: message.into(),
        }
    }

    fn io(error: &io::Error) -> Self {
        Self::new(ArchiveCause::Io, error.to_string())
    }

    fn unreadable(error: &io::Error) -> Self {
        Self::new(ArchiveCause::Unreadable, error.to_string())
    }

    /// The cause.
    #[must_use]
    pub fn cause(&self) -> ArchiveCause {
        self.cause
    }

    /// True when the archive is at fault. The API gives 422
    /// `archive_invalid`. False means a local failure: 500, or a bad target.
    #[must_use]
    pub fn is_refusal(&self) -> bool {
        !matches!(self.cause, ArchiveCause::Io | ArchiveCause::BadTarget)
    }
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.cause.code(), self.message)
    }
}

impl std::error::Error for ArchiveError {}

// ---------------------------------------------------------------- writing

/// A temporary file that is removed when it goes out of scope.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let mut journal = self.0.as_os_str().to_owned();
        journal.push("-journal");
        let _ = fs::remove_file(journal);
    }
}

/// Write the archive of the account in `folder` to `out`.
///
/// The folder is the data folder of the account. The function reads
/// `vault.json`, `grants/`, `collection.sqlite` and `library/` in it. A
/// library outside the folder is not in the archive. `collection.sqlite`
/// goes through the SQLite backup command into a temporary file in `folder`,
/// and Studio removes the file at the end, also after a failure. With
/// `include_assets` false, `library/` holds only the `.xml` files.
///
/// # Errors
///
/// Gives an [`ArchiveError`] with the cause `Io` when a file cannot be read
/// or `out` cannot be written, and `MissingFile` when the folder has no
/// `vault.json` or no `collection.sqlite`.
pub fn write_archive<W: Write>(
    folder: &Path,
    manifest: &Manifest,
    include_assets: bool,
    out: W,
) -> Result<(), ArchiveError> {
    let vault = folder.join(VAULT);
    let collection = folder.join(COLLECTION);
    for (path, name) in [(&vault, VAULT), (&collection, COLLECTION)] {
        if !path.is_file() {
            return Err(ArchiveError::new(
                ArchiveCause::MissingFile,
                format!("the folder has no {name}"),
            ));
        }
    }
    let mut builder = tar::Builder::new(out);
    let mtime = manifest.exported_at;
    let manifest_json = serde_json::to_vec_pretty(manifest)
        .map_err(|e| ArchiveError::new(ArchiveCause::InvalidManifest, e.to_string()))?;
    append_bytes(&mut builder, MANIFEST, &manifest_json, mtime)?;
    append_file(&mut builder, VAULT, &vault, mtime)?;
    let grants = folder.join(GRANTS);
    if grants.is_dir() {
        append_dir(&mut builder, GRANTS, mtime)?;
        for (name, path, is_dir) in sorted_children(&grants)? {
            if !is_dir {
                append_file(&mut builder, &format!("{GRANTS}/{name}"), &path, mtime)?;
            }
        }
    }
    {
        let temporary = TempFile(folder.join(format!(
            ".collection-export-{}-{}.tmp",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos())
        )));
        let source = rusqlite::Connection::open_with_flags(
            &collection,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| ArchiveError::new(ArchiveCause::Io, e.to_string()))?;
        source
            .backup(rusqlite::MAIN_DB, &temporary.0, None)
            .map_err(|e| ArchiveError::new(ArchiveCause::Io, e.to_string()))?;
        drop(source);
        append_file(&mut builder, COLLECTION, &temporary.0, mtime)?;
    }
    let library = folder.join(LIBRARY);
    if library.is_dir() {
        append_dir(&mut builder, LIBRARY, mtime)?;
        append_tree(&mut builder, LIBRARY, &library, include_assets, mtime)?;
    }
    builder.finish().map_err(|e| ArchiveError::io(&e))?;
    Ok(())
}

/// The children of `dir` as (name, path, is a folder), sorted by name.
/// A link is left out. A name that is not UTF-8 is an error.
fn sorted_children(dir: &Path) -> Result<Vec<(String, PathBuf, bool)>, ArchiveError> {
    let mut children = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| ArchiveError::io(&e))? {
        let entry = entry.map_err(|e| ArchiveError::io(&e))?;
        let kind = entry.file_type().map_err(|e| ArchiveError::io(&e))?;
        if kind.is_symlink() || !(kind.is_file() || kind.is_dir()) {
            continue;
        }
        let name = entry.file_name().into_string().map_err(|name| {
            ArchiveError::new(
                ArchiveCause::Io,
                format!("{} has a name that is not UTF-8", name.to_string_lossy()),
            )
        })?;
        children.push((name, entry.path(), kind.is_dir()));
    }
    children.sort();
    Ok(children)
}

fn append_tree<W: Write>(
    builder: &mut tar::Builder<W>,
    prefix: &str,
    dir: &Path,
    include_assets: bool,
    mtime: u64,
) -> Result<(), ArchiveError> {
    for (name, path, is_dir) in sorted_children(dir)? {
        let entry_name = format!("{prefix}/{name}");
        if is_dir {
            append_dir(builder, &entry_name, mtime)?;
            append_tree(builder, &entry_name, &path, include_assets, mtime)?;
        } else if include_assets || is_feed_xml(&name) {
            append_file(builder, &entry_name, &path, mtime)?;
        }
    }
    Ok(())
}

fn is_feed_xml(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
}

fn header(size: u64, mode: u32, mtime: u64) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(mode);
    header.set_mtime(mtime);
    header.set_uid(0);
    header.set_gid(0);
    header
}

fn append_bytes<W: Write>(
    builder: &mut tar::Builder<W>,
    name: &str,
    bytes: &[u8],
    mtime: u64,
) -> Result<(), ArchiveError> {
    let mut header = header(bytes.len() as u64, 0o600, mtime);
    header.set_entry_type(tar::EntryType::Regular);
    builder
        .append_data(&mut header, name, bytes)
        .map_err(|e| ArchiveError::io(&e))
}

fn append_dir<W: Write>(
    builder: &mut tar::Builder<W>,
    name: &str,
    mtime: u64,
) -> Result<(), ArchiveError> {
    let mut header = header(0, 0o700, mtime);
    header.set_entry_type(tar::EntryType::Directory);
    builder
        .append_data(&mut header, name, io::empty())
        .map_err(|e| ArchiveError::io(&e))
}

fn append_file<W: Write>(
    builder: &mut tar::Builder<W>,
    name: &str,
    path: &Path,
    mtime: u64,
) -> Result<(), ArchiveError> {
    let file = fs::File::open(path).map_err(|e| ArchiveError::io(&e))?;
    let size = file.metadata().map_err(|e| ArchiveError::io(&e))?.len();
    let mut header = header(size, 0o600, mtime);
    header.set_entry_type(tar::EntryType::Regular);
    // `take` keeps the stream at the size of the header, also if the file grows.
    builder
        .append_data(&mut header, name, file.take(size))
        .map_err(|e| ArchiveError::io(&e))
}

// ---------------------------------------------------------------- reading

/// Read each entry of the archive, and give the manifest.
///
/// This is the check of ADR 0107 item 11. It writes nothing. It uses
/// [`Limits::DEFAULT`].
///
/// # Errors
///
/// Gives an [`ArchiveError`] with the cause of the refusal. See
/// [`ArchiveError::is_refusal`].
pub fn check_archive<R: Read>(reader: R) -> Result<Manifest, ArchiveError> {
    check_archive_with(reader, Limits::DEFAULT)
}

/// Like [`check_archive`], with other [`Limits`].
///
/// # Errors
///
/// Gives an [`ArchiveError`] with the cause of the refusal.
pub fn check_archive_with<R: Read>(reader: R, limits: Limits) -> Result<Manifest, ArchiveError> {
    walk(reader, limits, None)
}

/// Write each entry of the archive into `folder`.
///
/// `folder` must not exist, or must be an empty folder. Each entry gets the
/// checks of ADR 0107 item 11 before Studio writes it. `manifest.json` is
/// checked and not written. On Unix, folders get the mode 0700, and
/// `vault.json` and the grant vaults get 0600 (ADR 0104). After a refusal or
/// a failure, `folder` has no file (it stays only when it existed before).
/// It uses [`Limits::DEFAULT`].
///
/// # Errors
///
/// Gives an [`ArchiveError`] with the cause of the refusal, or `Io`, or
/// `BadTarget` when `folder` is not new or not empty.
pub fn unpack_archive<R: Read>(reader: R, folder: &Path) -> Result<Manifest, ArchiveError> {
    unpack_archive_with(reader, folder, Limits::DEFAULT)
}

/// Like [`unpack_archive`], with other [`Limits`].
///
/// # Errors
///
/// Gives an [`ArchiveError`] as [`unpack_archive`] does.
pub fn unpack_archive_with<R: Read>(
    reader: R,
    folder: &Path,
    limits: Limits,
) -> Result<Manifest, ArchiveError> {
    let created = prepare_target(folder)?;
    let result = walk(reader, limits, Some(folder));
    if result.is_err() {
        clean_target(folder, created);
    }
    result
}

/// Check that `folder` is new or empty, and make it. Gives true when this
/// call made it.
fn prepare_target(folder: &Path) -> Result<bool, ArchiveError> {
    match fs::symlink_metadata(folder) {
        Ok(meta) if meta.is_dir() => {
            let mut children = fs::read_dir(folder).map_err(|e| ArchiveError::io(&e))?;
            if children.next().is_some() {
                return Err(ArchiveError::new(
                    ArchiveCause::BadTarget,
                    "ADR 0107 item 9: an import writes only into a new, empty folder",
                ));
            }
            Ok(false)
        }
        Ok(_) => Err(ArchiveError::new(
            ArchiveCause::BadTarget,
            "ADR 0107 item 9: the target is not a folder",
        )),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            make_dir(folder).map_err(|e| ArchiveError::io(&e))?;
            Ok(true)
        }
        Err(e) => Err(ArchiveError::io(&e)),
    }
}

/// Remove what a failed unpack wrote. Studio made the folder empty at the start.
fn clean_target(folder: &Path, created: bool) {
    if created {
        let _ = fs::remove_dir_all(folder);
        return;
    }
    if let Ok(children) = fs::read_dir(folder) {
        for child in children.flatten() {
            let path = child.path();
            if child.file_type().is_ok_and(|t| t.is_dir()) {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

fn make_dir(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path)
}

/// What an entry is, by its place in the archive.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Manifest,
    /// The vault and the grant vaults. Mode 0600.
    Vault,
    /// Any other allowed entry.
    Other,
}

/// Check the path of an entry against the files of ADR 0107 item 6.
fn place(path: &Path, is_dir: bool) -> Result<(Vec<String>, Place), ArchiveError> {
    let unsafe_path = |why: &str| {
        ArchiveError::new(
            ArchiveCause::UnsafePath,
            format!("ADR 0107 item 11: the entry {} {why}", path.display()),
        )
    };
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => {
                let name = name
                    .to_str()
                    .ok_or_else(|| unsafe_path("has a name that is not UTF-8"))?;
                if name.contains('\\') || name.contains('\0') {
                    return Err(unsafe_path("has a bad character in a name"));
                }
                parts.push(name.to_owned());
            }
            Component::ParentDir => return Err(unsafe_path("has a .. segment")),
            Component::RootDir | Component::Prefix(_) => {
                return Err(unsafe_path("has an absolute path"));
            }
            Component::CurDir => return Err(unsafe_path("has a . segment")),
        }
    }
    let names: Vec<&str> = parts.iter().map(String::as_str).collect();
    let found = match (names.as_slice(), is_dir) {
        ([MANIFEST], false) => Place::Manifest,
        ([VAULT] | [GRANTS, _], false) => Place::Vault,
        ([COLLECTION] | [LIBRARY, _, ..], false) | ([GRANTS] | [LIBRARY, ..], true) => Place::Other,
        _ => {
            return Err(ArchiveError::new(
                ArchiveCause::UnexpectedEntry,
                format!(
                    "ADR 0107 item 11: the entry {} is not a file of an account archive",
                    path.display()
                ),
            ));
        }
    };
    Ok((parts, found))
}

/// Go through the archive. With `target`, write each file into it.
fn walk<R: Read>(
    reader: R,
    limits: Limits,
    target: Option<&Path>,
) -> Result<Manifest, ArchiveError> {
    let mut archive = tar::Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| ArchiveError::unreadable(&e))?;
    let mut manifest: Option<Manifest> = None;
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    let mut count: u64 = 0;
    let mut total: u64 = 0;
    let (mut has_vault, mut has_collection) = (false, false);
    for entry in entries {
        let mut entry = entry.map_err(|e| ArchiveError::unreadable(&e))?;
        count += 1;
        if count > limits.max_entries {
            return Err(ArchiveError::new(
                ArchiveCause::TooManyEntries,
                "ADR 0107 item 15: the archive has too many entries",
            ));
        }
        let path = entry
            .path()
            .map_err(|e| ArchiveError::new(ArchiveCause::UnsafePath, e.to_string()))?
            .into_owned();
        let is_dir = match entry.header().entry_type() {
            tar::EntryType::Regular | tar::EntryType::Continuous => false,
            tar::EntryType::Directory => true,
            tar::EntryType::Symlink | tar::EntryType::Link => {
                return Err(ArchiveError::new(
                    ArchiveCause::LinkEntry,
                    format!("ADR 0107 item 11: the entry {} is a link", path.display()),
                ));
            }
            _ => {
                return Err(ArchiveError::new(
                    ArchiveCause::SpecialEntry,
                    format!(
                        "ADR 0107 item 11: the entry {} is not a plain file or folder",
                        path.display()
                    ),
                ));
            }
        };
        let (parts, found) = place(&path, is_dir)?;
        total = total.saturating_add(entry.size());
        if total > limits.max_total_bytes {
            return Err(ArchiveError::new(
                ArchiveCause::TooLarge,
                "ADR 0107 item 15: the archive is larger than the limit",
            ));
        }
        if manifest.is_none() {
            if found != Place::Manifest {
                return Err(ArchiveError::new(
                    ArchiveCause::MissingManifest,
                    "ADR 0107 item 6: manifest.json must be the first entry",
                ));
            }
            let size = entry.size();
            manifest = Some(read_manifest(&mut entry, size)?);
            seen.insert(parts);
            continue;
        }
        if found == Place::Manifest {
            return Err(ArchiveError::new(
                ArchiveCause::DuplicateEntry,
                "ADR 0107 item 11: manifest.json is in the archive two times",
            ));
        }
        if !is_dir && !seen.insert(parts.clone()) {
            return Err(ArchiveError::new(
                ArchiveCause::DuplicateEntry,
                format!(
                    "ADR 0107 item 11: the entry {} is in the archive two times",
                    path.display()
                ),
            ));
        }
        has_vault |= parts == [VAULT];
        has_collection |= parts == [COLLECTION];
        match target {
            Some(root) => write_entry(root, &parts, found, is_dir, &mut entry)?,
            None => {
                io::copy(&mut entry, &mut io::sink()).map_err(|e| ArchiveError::unreadable(&e))?;
            }
        }
    }
    let manifest = manifest.ok_or_else(|| {
        ArchiveError::new(
            ArchiveCause::MissingManifest,
            "ADR 0107 item 6: the archive has no manifest.json",
        )
    })?;
    if !(has_vault && has_collection) {
        return Err(ArchiveError::new(
            ArchiveCause::MissingFile,
            "ADR 0107 item 6: the archive needs vault.json and collection.sqlite",
        ));
    }
    Ok(manifest)
}

fn read_manifest<R: Read>(entry: &mut R, size: u64) -> Result<Manifest, ArchiveError> {
    if size > MAX_MANIFEST_BYTES {
        return Err(ArchiveError::new(
            ArchiveCause::InvalidManifest,
            "manifest.json is too large",
        ));
    }
    let mut bytes = Vec::new();
    entry
        .take(MAX_MANIFEST_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| ArchiveError::unreadable(&e))?;
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|e| ArchiveError::new(ArchiveCause::InvalidManifest, e.to_string()))?;
    if manifest.format_version != Manifest::FORMAT_VERSION {
        return Err(ArchiveError::new(
            ArchiveCause::UnknownFormat,
            format!(
                "ADR 0107 item 11: the archive has format version {}, and this Studio reads version {}",
                manifest.format_version,
                Manifest::FORMAT_VERSION
            ),
        ));
    }
    if manifest.schema_version > SCHEMA_VERSION {
        return Err(ArchiveError::new(
            ArchiveCause::NewerSchema,
            format!(
                "ADR 0107 item 11: the collection has schema version {}, and this Studio knows only {SCHEMA_VERSION}",
                manifest.schema_version
            ),
        ));
    }
    Ok(manifest)
}

/// Write one checked entry below `root`. `parts` are plain names.
fn write_entry<R: Read>(
    root: &Path,
    parts: &[String],
    found: Place,
    is_dir: bool,
    entry: &mut R,
) -> Result<(), ArchiveError> {
    let mut path = root.to_path_buf();
    path.extend(parts);
    if is_dir {
        return make_dir(&path).map_err(|e| ArchiveError::io(&e));
    }
    if let Some(parent) = path.parent() {
        make_dir(parent).map_err(|e| ArchiveError::io(&e))?;
    }
    let mut options = fs::OpenOptions::new();
    // `create_new` refuses an existing file, and a link at the path.
    options.write(true).create_new(true);
    #[cfg(unix)]
    if found == Place::Vault {
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    }
    #[cfg(not(unix))]
    let _ = found;
    let mut file = options.open(&path).map_err(|e| {
        if e.kind() == io::ErrorKind::AlreadyExists {
            ArchiveError::new(
                ArchiveCause::DuplicateEntry,
                "ADR 0107 item 11: an entry is in the archive two times",
            )
        } else {
            ArchiveError::io(&e)
        }
    })?;
    io::copy(entry, &mut file).map_err(|e| ArchiveError::unreadable(&e))?;
    file.sync_all().map_err(|e| ArchiveError::io(&e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection;
    use serde_json::json;

    /// Make an account folder with every kind of entry.
    fn account(dir: &Path) {
        fs::create_dir_all(dir.join("grants")).unwrap();
        fs::write(dir.join("vault.json"), b"{\"vault\":true}").unwrap();
        fs::write(dir.join("grants/4.json"), b"{\"grant\":4}").unwrap();
        fs::create_dir_all(dir.join("library/album")).unwrap();
        fs::write(dir.join("library/album/album.xml"), b"<rss/>").unwrap();
        fs::write(dir.join("library/album/track.mp3"), b"audio").unwrap();
        fs::write(dir.join("library/loose.txt"), b"x").unwrap();
        let collection = Collection::open(&dir.join("collection.sqlite")).unwrap();
        collection
            .set_setting("library_root", &json!("library"))
            .unwrap();
    }

    fn pack(dir: &Path, assets: bool) -> Vec<u8> {
        let manifest = Manifest::new(Some("a@example.org".to_owned()), vec!["ab".repeat(32)]);
        let mut out = Vec::new();
        write_archive(dir, &manifest, assets, &mut out).unwrap();
        out
    }

    /// Every file and folder below `dir`, relative, sorted.
    fn listing(dir: &Path) -> Vec<String> {
        fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                out.push(path.strip_prefix(base).unwrap().display().to_string());
                if path.is_dir() {
                    walk(&path, base, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out.sort();
        out
    }

    #[test]
    fn a_round_trip_gives_the_same_files_and_the_manifest_holds_no_secret() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        account(&source);
        let bytes = pack(&source, true);
        let manifest = check_archive(&bytes[..]).unwrap();
        let target = dir.path().join("target");
        let unpacked = unpack_archive(&bytes[..], &target).unwrap();
        assert_eq!(
            manifest, unpacked,
            "ADR 0107 item 6: check and unpack must read one manifest"
        );
        for name in [
            "vault.json",
            "grants/4.json",
            "library/album/album.xml",
            "library/album/track.mp3",
            "library/loose.txt",
        ] {
            assert_eq!(
                fs::read(source.join(name)).unwrap(),
                fs::read(target.join(name)).unwrap(),
                "ADR 0107 item 6: {name} must arrive as it is"
            );
        }
        assert_eq!(
            listing(&target),
            listing(&source),
            "ADR 0107 item 6: the export must leave no temporary file, and the import must write the same files"
        );
        let json = serde_json::to_value(&manifest).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "email",
                "exported_at",
                "format_version",
                "nostr_keys",
                "schema_version",
                "studio_version"
            ],
            "ADR 0107 item 6: the manifest must hold no secret and no wrapper. Do not add a field."
        );
    }

    #[test]
    fn an_export_without_assets_keeps_only_the_feed_xml() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        account(&source);
        let bytes = pack(&source, false);
        let target = dir.path().join("target");
        unpack_archive(&bytes[..], &target).unwrap();
        assert!(target.join("library/album/album.xml").is_file());
        assert!(
            !target.join("library/album/track.mp3").exists(),
            "ADR 0107 item 6: include_assets false must leave out audio and images"
        );
    }

    #[test]
    fn an_export_removes_its_temporary_file_after_a_failure() {
        struct Broken(usize);
        impl Write for Broken {
            fn write(&mut self, data: &[u8]) -> io::Result<usize> {
                // The manifest and the vault pass. The failure comes late,
                // when the collection copy exists.
                if self.0 > 20_000 {
                    return Err(io::Error::other("closed"));
                }
                self.0 += data.len();
                Ok(data.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let dir = tempfile::tempdir().unwrap();
        account(dir.path());
        let manifest = Manifest::new(None, Vec::new());
        let result = write_archive(dir.path(), &manifest, true, Broken(0));
        assert!(result.is_err());
        assert!(
            listing(dir.path())
                .iter()
                .all(|n| !n.starts_with(".collection-export")),
            "ADR 0107 item 6: write_archive must remove its temporary copy of collection.sqlite after a failure"
        );
    }

    /// A tar stream from raw headers, so a test can write names that the
    /// `tar` crate refuses to set.
    struct Raw(Vec<u8>);

    impl Raw {
        fn good_start() -> Self {
            let mut raw = Self(Vec::new());
            let manifest = serde_json::to_vec(&Manifest::new(None, Vec::new())).unwrap();
            raw.entry(b"manifest.json", b'0', b"", &manifest);
            raw.entry(b"vault.json", b'0', b"", b"{}");
            raw.entry(b"library/", b'5', b"", b"");
            raw.entry(b"library/a.xml", b'0', b"", b"<rss/>");
            raw
        }

        fn entry(&mut self, name: &[u8], kind: u8, link: &[u8], data: &[u8]) {
            let mut block = [0u8; 512];
            block[..name.len()].copy_from_slice(name);
            block[100..107].copy_from_slice(b"0000644");
            block[108..115].copy_from_slice(b"0000000");
            block[116..123].copy_from_slice(b"0000000");
            let size = format!("{:011o}", data.len());
            block[124..135].copy_from_slice(size.as_bytes());
            block[136..147].copy_from_slice(b"00000000000");
            block[148..156].copy_from_slice(b"        ");
            block[156] = kind;
            block[157..157 + link.len()].copy_from_slice(link);
            block[257..263].copy_from_slice(b"ustar\0");
            block[263..265].copy_from_slice(b"00");
            let sum: u32 = block.iter().map(|b| u32::from(*b)).sum();
            block[148..155].copy_from_slice(format!("{sum:06o}\0").as_bytes());
            block[155] = b' ';
            self.0.extend_from_slice(&block);
            self.0.extend_from_slice(data);
            let pad = (512 - data.len() % 512) % 512;
            self.0.extend(std::iter::repeat_n(0u8, pad));
        }

        fn finish(mut self) -> Vec<u8> {
            self.0.extend(std::iter::repeat_n(0u8, 1024));
            self.0
        }
    }

    #[test]
    fn unpack_refuses_a_bad_entry_and_leaves_no_file() {
        type Case = (&'static str, &'static [u8], u8, &'static [u8], ArchiveCause);
        let cases: [Case; 9] = [
            (
                "dot dot",
                b"library/../../escape.txt",
                b'0',
                b"",
                ArchiveCause::UnsafePath,
            ),
            (
                "leading dot dot",
                b"../escape.txt",
                b'0',
                b"",
                ArchiveCause::UnsafePath,
            ),
            (
                "absolute",
                b"/etc/escape.txt",
                b'0',
                b"",
                ArchiveCause::UnsafePath,
            ),
            (
                "symlink",
                b"library/link",
                b'2',
                b"/etc",
                ArchiveCause::LinkEntry,
            ),
            (
                "hard link",
                b"library/hard",
                b'1',
                b"vault.json",
                ArchiveCause::LinkEntry,
            ),
            (
                "char device",
                b"library/null",
                b'3',
                b"",
                ArchiveCause::SpecialEntry,
            ),
            (
                "fifo",
                b"library/pipe",
                b'6',
                b"",
                ArchiveCause::SpecialEntry,
            ),
            (
                "other file",
                b"server.sqlite",
                b'0',
                b"",
                ArchiveCause::UnexpectedEntry,
            ),
            (
                "backslash",
                b"library\\..\\x",
                b'0',
                b"",
                ArchiveCause::UnsafePath,
            ),
        ];
        for (label, name, kind, link, cause) in cases {
            let mut raw = Raw::good_start();
            raw.entry(name, kind, link, b"");
            let bytes = raw.finish();
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("new");
            let error = unpack_archive(&bytes[..], &target).unwrap_err();
            assert_eq!(
                error.cause(),
                cause,
                "ADR 0107 item 11: {label} must be refused with its cause. Check the entry before the write in walk()."
            );
            assert!(error.is_refusal());
            assert!(
                !target.exists() && !dir.path().join("escape.txt").exists(),
                "ADR 0107 item 11: {label}: a refused archive must leave no file. Clean the target in unpack_archive_with()."
            );
            // An existing empty folder stays, and stays empty.
            let again = dir.path().join("empty");
            fs::create_dir(&again).unwrap();
            assert!(unpack_archive(&bytes[..], &again).is_err());
            assert!(
                listing(&again).is_empty(),
                "ADR 0107 item 11: {label}: the empty folder must have no file after a refusal"
            );
            assert!(
                check_archive(&bytes[..]).is_err(),
                "ADR 0107 item 11: {label}: check_archive must refuse it too"
            );
        }
    }

    #[test]
    fn unpack_refuses_the_other_conditions_of_item_11() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = |format: u32, schema: usize| {
            json!({"format_version": format, "studio_version": "x", "schema_version": schema,
                   "exported_at": 0, "email": null, "nostr_keys": []})
            .to_string()
        };
        let build = |manifest: &str| {
            let mut raw = Raw(Vec::new());
            raw.entry(b"manifest.json", b'0', b"", manifest.as_bytes());
            raw.entry(b"vault.json", b'0', b"", b"{}");
            raw.entry(b"collection.sqlite", b'0', b"", b"x");
            raw
        };
        let cause = |bytes: Vec<u8>, name: &str| {
            let target = dir.path().join(name);
            let error = unpack_archive(&bytes[..], &target).unwrap_err();
            assert!(
                !target.exists(),
                "ADR 0107 item 11: {name}: a refused archive must leave no folder"
            );
            error.cause()
        };
        assert_eq!(
            cause(build(&manifest(2, 1)).finish(), "a"),
            ArchiveCause::UnknownFormat
        );
        assert_eq!(
            cause(build(&manifest(1, SCHEMA_VERSION + 1)).finish(), "b"),
            ArchiveCause::NewerSchema
        );
        assert_eq!(
            cause(
                build("{\"format_version\":1,\"wrapper\":\"x\"}").finish(),
                "c"
            ),
            ArchiveCause::InvalidManifest,
            "ADR 0107 item 6: the manifest must refuse a field it does not know"
        );
        let mut raw = Raw(Vec::new());
        raw.entry(b"vault.json", b'0', b"", b"{}");
        assert_eq!(cause(raw.finish(), "d"), ArchiveCause::MissingManifest);
        let mut raw = build(&manifest(1, 1));
        raw.entry(b"vault.json", b'0', b"", b"{}");
        assert_eq!(cause(raw.finish(), "e"), ArchiveCause::DuplicateEntry);
        let mut raw = Raw(Vec::new());
        raw.entry(b"manifest.json", b'0', b"", manifest(1, 1).as_bytes());
        raw.entry(b"vault.json", b'0', b"", b"{}");
        assert_eq!(cause(raw.finish(), "f"), ArchiveCause::MissingFile);
    }

    #[test]
    fn unpack_writes_only_into_a_new_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = Raw::good_start().finish();
        let full = dir.path().join("full");
        fs::create_dir(&full).unwrap();
        fs::write(full.join("keep.txt"), b"mine").unwrap();
        let error = unpack_archive(&bytes[..], &full).unwrap_err();
        assert_eq!(
            error.cause(),
            ArchiveCause::BadTarget,
            "ADR 0107 item 9: refuse a folder with a file"
        );
        assert_eq!(fs::read(full.join("keep.txt")).unwrap(), b"mine");
        assert_eq!(listing(&full), ["keep.txt"]);
    }

    #[test]
    fn the_limits_stop_a_large_archive() {
        let dir = tempfile::tempdir().unwrap();
        let mut raw = Raw::good_start();
        raw.entry(b"collection.sqlite", b'0', b"", &[0u8; 2000]);
        let bytes = raw.finish();
        let few = Limits {
            max_entries: 2,
            max_total_bytes: u64::MAX,
        };
        assert_eq!(
            unpack_archive_with(&bytes[..], &dir.path().join("a"), few)
                .unwrap_err()
                .cause(),
            ArchiveCause::TooManyEntries
        );
        let small = Limits {
            max_entries: 100,
            max_total_bytes: 1000,
        };
        let target = dir.path().join("b");
        assert_eq!(
            unpack_archive_with(&bytes[..], &target, small)
                .unwrap_err()
                .cause(),
            ArchiveCause::TooLarge,
            "ADR 0107 item 15: a hostile archive must not fill the disk. Keep the total size limit."
        );
        assert!(!target.exists());
    }

    #[cfg(unix)]
    #[test]
    fn unpack_gives_the_modes_of_adr_0104() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        account(&source);
        let bytes = pack(&source, true);
        let target = dir.path().join("target");
        unpack_archive(&bytes[..], &target).unwrap();
        let mode = |p: &str| fs::metadata(target.join(p)).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode(""),
            0o700,
            "ADR 0104 item 2: the folder must have the mode 0700"
        );
        assert_eq!(mode("grants"), 0o700);
        assert_eq!(mode("library/album"), 0o700);
        assert_eq!(
            mode("vault.json"),
            0o600,
            "ADR 0104 item 2: a vault file must have the mode 0600"
        );
        assert_eq!(mode("grants/4.json"), 0o600);
    }

    #[test]
    fn an_archive_with_a_relative_library_opens_in_a_different_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        account(&source);
        let bytes = pack(&source, true);
        let target = dir.path().join("elsewhere/user");
        unpack_archive(&bytes[..], &target).unwrap();
        let collection = Collection::open(&target.join("collection.sqlite")).unwrap();
        assert_eq!(
            collection.setting("library_root").unwrap(),
            Some(json!("library")),
            "ADR 0107 item 1: the archive must keep the library path relative"
        );
        assert_eq!(
            collection.library_root().unwrap(),
            Some(target.join("library")),
            "ADR 0107 item 1: a relative library_root must read against the new data folder"
        );
    }

    #[test]
    fn a_relative_library_root_that_leaves_the_data_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
        for bad in ["../x", "a/../../x", ""] {
            collection.set_setting("library_root", &json!(bad)).unwrap();
            assert!(
                collection.library_root().is_err(),
                "ADR 0107 item 1: an imported library_root must not leave the data folder. Refuse {bad:?}."
            );
        }
    }
}
