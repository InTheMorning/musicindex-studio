//! What the desktop app does when the user closes its window (ADR 0124
//! item 29).
//!
//! The app reads this choice at each close, and the Settings page changes it
//! through the local router. The server and the browser shell have no window,
//! so the route gives no choice for them.

use std::path::{Path, PathBuf};

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ApiError;

/// The file in the data folder that keeps the choice.
const FILE_NAME: &str = "window-close";

/// The choice of the user for a close of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnClose {
    /// The window hides, and Studio continues to run in the tray.
    Tray,
    /// Studio quits.
    Quit,
}

impl OnClose {
    fn text(self) -> &'static str {
        match self {
            Self::Tray => "tray",
            Self::Quit => "quit",
        }
    }
}

/// A cloneable holder of the place of the choice. The default has no place,
/// for a shell with no window.
#[derive(Debug, Clone, Default)]
pub struct WindowClose {
    path: Option<PathBuf>,
}

impl WindowClose {
    /// The choice of the app with this data folder.
    #[must_use]
    pub fn in_folder(data_dir: &Path) -> Self {
        Self {
            path: Some(data_dir.join(FILE_NAME)),
        }
    }

    /// The choice, or `None` when the user has made no choice.
    #[must_use]
    pub fn read(&self) -> Option<OnClose> {
        let text = std::fs::read_to_string(self.path.as_ref()?).ok()?;
        match text.trim() {
            "tray" => Some(OnClose::Tray),
            "quit" => Some(OnClose::Quit),
            _ => None,
        }
    }

    /// Keep the choice.
    ///
    /// # Errors
    ///
    /// Gives the error of the write. A shell with no window gives
    /// [`std::io::ErrorKind::Unsupported`].
    pub fn write(&self, choice: OnClose) -> std::io::Result<()> {
        let Some(path) = &self.path else {
            return Err(std::io::ErrorKind::Unsupported.into());
        };
        std::fs::write(path, choice.text())
    }
}

#[derive(Deserialize)]
pub(crate) struct SetWindowClose {
    on_close: OnClose,
}

/// `GET /api/v1/window-close`.
pub(crate) async fn get(State(window): State<WindowClose>) -> Json<Value> {
    Json(json!({ "available": window.path.is_some(), "on_close": window.read() }))
}

/// `PUT /api/v1/window-close`.
pub(crate) async fn put(
    State(window): State<WindowClose>,
    Json(body): Json<SetWindowClose>,
) -> Result<Json<Value>, ApiError> {
    window.write(body.on_close).map_err(|_| {
        ApiError::new(
            StatusCode::CONFLICT,
            "window_close_not_saved",
            "Studio could not save this setting.",
        )
    })?;
    Ok(Json(
        json!({ "available": true, "on_close": body.on_close }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_choice_stays_in_the_data_folder() {
        let folder = tempfile::tempdir().unwrap();
        let window = WindowClose::in_folder(folder.path());
        assert_eq!(window.read(), None);
        window.write(OnClose::Quit).unwrap();
        assert_eq!(window.read(), Some(OnClose::Quit));
        window.write(OnClose::Tray).unwrap();
        assert_eq!(window.read(), Some(OnClose::Tray));
        std::fs::write(folder.path().join(FILE_NAME), "other").unwrap();
        assert_eq!(window.read(), None, "An unknown text is no choice.");
    }

    #[test]
    fn a_shell_with_no_window_keeps_no_choice() {
        let window = WindowClose::default();
        assert_eq!(window.read(), None);
        assert!(window.write(OnClose::Tray).is_err());
    }
}
