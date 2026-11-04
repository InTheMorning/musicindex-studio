//! The parts of the page that `WebKitGTK` does not give on Linux (ADR 0124
//! items 22 to 25): the file of a drop, and Ctrl+Z.

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

/// Give a drop to the page. The page gets the names, and an address for one
/// regular file only. A drop of many files or of a folder needs no bytes,
/// because the field refuses it.
pub fn give_drop(
    window: &WebviewWindow,
    dropped: &studio_desktop::DroppedFiles,
    token: &str,
    paths: &[PathBuf],
    position: PhysicalPosition<f64>,
) {
    let names: Vec<String> = paths.iter().map(|p| file_name(p)).collect();
    let id = match paths {
        [one] if one.is_file() => dropped.replace(one.clone()).ok(),
        _ => None,
    };
    let call = json!({
        "x": position.x,
        "y": position.y,
        "names": names,
        "id": id,
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
