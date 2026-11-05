//! The parts of the page that `WebKitGTK` does not give on Linux (ADR 0124
//! items 22 to 25 and 27): the file of a drop, Ctrl+Z, and an opaque toolbar.

use std::path::PathBuf;

use serde_json::json;
use tauri::{PhysicalPosition, Url, WebviewWindow};

/// The script of the window. It runs before the UI.
pub const SCRIPT: &str = include_str!("linux_page.js");

/// The access token from the UI address.
pub fn token_of(address: &Url) -> String {
    address
        .fragment()
        .and_then(|f| f.strip_prefix("token="))
        .unwrap_or_default()
        .to_string()
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// Give a drop to the page. The page gets the name of each path, and the ID
/// of each regular file. A file path does not go to the page.
pub fn give_drop(
    window: &WebviewWindow,
    dropped: &studio_desktop::DroppedFiles,
    token: &str,
    paths: &[PathBuf],
    position: PhysicalPosition<f64>,
) {
    let regular: Vec<PathBuf> = paths.iter().filter(|p| p.is_file()).cloned().collect();
    let mut ids = dropped.replace(regular).unwrap_or_default().into_iter();
    let files: Vec<_> = paths
        .iter()
        .map(|path| {
            let id = if path.is_file() { ids.next() } else { None };
            json!({ "name": file_name(path), "id": id })
        })
        .collect();
    let call = json!({
        "x": position.x,
        "y": position.y,
        "files": files,
        "token": token,
    });
    let _ = window.eval(format!("window.__studioDrop({call});"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_comes_from_the_fragment_of_the_ui_address() {
        let address = Url::parse("http://127.0.0.1:42069/#token=abc123").unwrap();
        assert_eq!(token_of(&address), "abc123");
    }
}
