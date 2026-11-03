//! The place and the name of a download (ADR 0124 item 6).

use std::path::{Path, PathBuf};

/// The path for `name` in `dir`. When the name is in use, add ` 2`, ` 3` and
/// so on before the extension. `exists` tells whether a path is in use.
#[must_use]
pub fn free_path(dir: &Path, name: &str, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let first = dir.join(name);
    if !exists(&first) {
        return first;
    }
    let as_path = Path::new(name);
    let stem = as_path
        .file_stem()
        .map_or_else(|| name.to_string(), |s| s.to_string_lossy().into_owned());
    let extension = as_path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (2_u32..=u32::MAX)
        .map(|n| dir.join(format!("{stem} {n}{extension}")))
        .find(|path| !exists(path))
        .unwrap_or(first)
}

/// The downloads folder of the user, else the home folder.
#[must_use]
pub fn downloads_dir() -> Option<PathBuf> {
    let dirs = directories::UserDirs::new()?;
    Some(
        dirs.download_dir()
            .unwrap_or_else(|| dirs.home_dir())
            .to_path_buf(),
    )
}

/// The message after a download.
#[must_use]
pub fn finished_message(name: &str, success: bool) -> String {
    if success {
        format!("Studio saved {name} in your Downloads folder.")
    } else {
        format!("The download of {name} did not complete.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_free_name_stays() {
        let path = free_path(Path::new("/d"), "a.json", |_| false);
        assert_eq!(path, Path::new("/d/a.json"));
    }

    #[test]
    fn a_name_in_use_gets_a_number_before_the_extension() {
        let used = [PathBuf::from("/d/a.json"), PathBuf::from("/d/a 2.json")];
        let path = free_path(Path::new("/d"), "a.json", |p| used.iter().any(|u| u == p));
        assert_eq!(
            path,
            Path::new("/d/a 3.json"),
            "ADR 0124 item 6: a name in use must get ` 2`, ` 3` and so on before the extension."
        );
    }

    #[test]
    fn a_name_without_extension_gets_a_number() {
        let path = free_path(Path::new("/d"), "notes", |p| p == Path::new("/d/notes"));
        assert_eq!(path, Path::new("/d/notes 2"));
    }

    #[test]
    fn a_real_folder_gives_a_free_name() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("x.txt"), b"1").unwrap();
        let path = free_path(dir.path(), "x.txt", std::path::Path::exists);
        assert_eq!(path, dir.path().join("x 2.txt"));
    }
}
