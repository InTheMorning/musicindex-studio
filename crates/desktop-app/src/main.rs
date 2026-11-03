//! The Tauri app of musicindex-studio (ADR 0124).
//!
//! The app runs the local API of `studio-desktop` in its own process and
//! opens one window on it. The window has no Tauri command. A close of the
//! window hides it, and the app stays in the tray.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloads;
mod links;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{
    AppHandle, Manager, RunEvent, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::oneshot;

use links::{LinkAction, link_action};

const WINDOW_LABEL: &str = "main";
const TITLE: &str = "Music Index Studio";

/// The shutdown signal of the API. "Quit" sends it.
struct Shutdown(Mutex<Option<oneshot::Sender<()>>>);

/// The data folder of the app: the folder of `studio-desktop` (ADR 0124 item 11).
fn data_dir() -> Option<PathBuf> {
    studio_desktop::default_data_dir()
}

/// Show the window of the app and give it the focus.
fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Stop the API, then end the app (ADR 0124 item 9).
fn quit(app: &AppHandle) {
    if let Some(shutdown) = app.try_state::<Shutdown>()
        && let Ok(mut sender) = shutdown.0.lock()
        && let Some(sender) = sender.take()
    {
        let _ = sender.send(());
    }
    app.exit(0);
}

/// Show an error in a dialog, then end the app. The text holds no address.
fn fail(app: &AppHandle, message: &str) {
    let handle = app.clone();
    app.dialog()
        .message(message)
        .title(TITLE)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::Ok)
        .show(move |_| handle.exit(1));
}

fn open_outside(app: &AppHandle, address: &Url) {
    // A failure to open leaves the window as it is.
    let _ = app.opener().open_url(address.as_str(), None::<&str>);
}

fn build_window(app: &AppHandle, ui_url: &str) -> Result<WebviewWindow, String> {
    let address = Url::parse(ui_url).map_err(|e| e.to_string())?;
    let origin = address.origin();
    let navigation_origin = origin.clone();
    let navigation_app = app.clone();
    let window_app = app.clone();
    let download_app = app.clone();
    let planned: Mutex<HashMap<String, PathBuf>> = Mutex::new(HashMap::new());
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(address))
        .title(TITLE)
        .inner_size(1280.0, 860.0)
        .disable_drag_drop_handler()
        .on_navigation(
            move |target| match link_action(&navigation_origin, target, false) {
                LinkAction::Keep => true,
                LinkAction::OpenOutside => {
                    open_outside(&navigation_app, target);
                    false
                }
                LinkAction::Block => false,
            },
        )
        .on_new_window(move |target, _features| {
            if link_action(&origin, &target, true) == LinkAction::OpenOutside {
                open_outside(&window_app, &target);
            }
            NewWindowResponse::Deny
        })
        .on_download(move |_webview, event| {
            match event {
                DownloadEvent::Requested { url, destination } => {
                    let Some(dir) = downloads::downloads_dir() else {
                        return false;
                    };
                    let name = destination.file_name().map_or_else(
                        || "download".to_string(),
                        |n| n.to_string_lossy().into_owned(),
                    );
                    *destination = downloads::free_path(&dir, &name, std::path::Path::exists);
                    if let Ok(mut planned) = planned.lock() {
                        planned.insert(url.to_string(), destination.clone());
                    }
                }
                DownloadEvent::Finished { url, path, success } => {
                    let planned_path = planned
                        .lock()
                        .ok()
                        .and_then(|mut planned| planned.remove(url.as_str()));
                    // macOS gives no path at the end, so the planned name serves.
                    let name = path
                        .or(planned_path)
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                        .unwrap_or_else(|| "the file".to_string());
                    download_app
                        .dialog()
                        .message(downloads::finished_message(&name, success))
                        .title(TITLE)
                        .buttons(MessageDialogButtons::Ok)
                        .show(|_| {});
                }
                _ => {}
            }
            true
        })
        .build()
        .map_err(|e| e.to_string())?;
    let hidden = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = hidden.hide();
        }
    });
    Ok(window)
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Studio", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit_item])?;
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "quit" => quit(app),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Start the API, then open the window and the tray.
fn setup(app: &AppHandle) -> Result<(), String> {
    let data_dir =
        data_dir().ok_or("Studio could not start. The system has no application data folder.")?;
    let (sender, receiver) = oneshot::channel::<()>();
    let started = tauri::async_runtime::block_on(async {
        let listener = studio_desktop::bind(None)
            .await
            .map_err(|e| format!("Studio could not start. {e}"))?;
        let options = studio_desktop::StartOptions::new(&data_dir);
        let shutdown = async {
            let _ = receiver.await;
        };
        studio_desktop::start(&options, listener, shutdown).map_err(|e| {
            // With no D-Bus, the single instance plugin does not find the
            // first app. The lock of the data folder then stops this start.
            if e.already_running() {
                "Studio is already running.".to_string()
            } else {
                format!("Studio could not start. {e}")
            }
        })
    })?;
    app.manage(Shutdown(Mutex::new(Some(sender))));
    let serve = started.serve;
    tauri::async_runtime::spawn(async move {
        if let Err(error) = serve.await {
            eprintln!("studio-desktop-app: the API stopped: {error}");
        }
    });
    build_window(app, &started.ui_url)?;
    build_tray(app).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    // ADR 0104 item 1: this call comes first.
    studio_desktop::harden_process();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Err(error) = setup(app.handle()) {
                fail(app.handle(), &error);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("the Tauri app must build from its configuration");
    app.run(|_app, event| {
        // A close of the last window does not end the app (ADR 0124 item 7).
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_app_uses_the_data_folder_of_studio_desktop() {
        let folder = super::data_dir().expect("the test computer must have a data folder");
        assert_eq!(
            Some(folder.clone()),
            studio_desktop::default_data_dir(),
            "ADR 0124 item 11: the app must use default_data_dir() of studio-desktop."
        );
        assert!(
            folder.to_string_lossy().contains("musicindex-studio"),
            "ADR 0124 item 11: the data folder must be the folder musicindex-studio."
        );
    }
}
