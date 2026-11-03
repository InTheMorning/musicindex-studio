//! The Tauri app of musicindex-studio (ADR 0124).
//!
//! The app runs the local API of `studio-desktop` in its own process and
//! opens one window on it. The window has no Tauri command. A close of the
//! window hides it, and the app stays in the tray.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloads;
mod links;
mod quit;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::tray::TrayIconId;
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{
    AppHandle, Manager, RunEvent, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_dialog::{
    DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult,
};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::oneshot;

use links::{LinkAction, link_action};
use quit::{QuitAction, QuitEvent, QuitState, WorkCount};

const WINDOW_LABEL: &str = "main";
const TITLE: &str = "Music Index Studio";
/// The ID of the tray icon, so that the app can change its menu.
const TRAY_ID: &str = "main";

/// The shutdown signal of the API. "Quit" sends it.
struct Shutdown(Mutex<Option<oneshot::Sender<()>>>);

/// The end of the serve of the API, for "Quit After Write".
struct ApiEnded(Mutex<Option<oneshot::Receiver<()>>>);

/// The quit: its state, the work count and the generation of the wait.
struct QuitControl {
    state: Mutex<QuitState>,
    work: studio_desktop::Work,
    /// Each wait gets a number, so that an old watcher ends.
    generation: AtomicU64,
}

impl QuitControl {
    fn count(&self) -> WorkCount {
        WorkCount {
            writes: self.work.writes(),
            uploads: self.work.uploads(),
        }
    }
}

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

/// Send the shutdown signal of the API.
fn signal_shutdown(app: &AppHandle) {
    if let Some(shutdown) = app.try_state::<Shutdown>()
        && let Ok(mut sender) = shutdown.0.lock()
        && let Some(sender) = sender.take()
    {
        let _ = sender.send(());
    }
}

/// Send the shutdown signal, then end the app with no wait (ADR 0124
/// items 9 and 19).
fn end_now(app: &AppHandle) {
    signal_shutdown(app);
    app.exit(0);
}

/// Send the shutdown signal, wait for each request that started, then end
/// the app (ADR 0124 item 20).
fn stop_api_and_end(app: &AppHandle) {
    signal_shutdown(app);
    let ended = app
        .try_state::<ApiEnded>()
        .and_then(|ended| ended.0.lock().ok().and_then(|mut r| r.take()));
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(ended) = ended {
            let _ = ended.await;
        }
        handle.exit(0);
    });
}

/// The menu of the tray while the app runs (ADR 0124 item 9).
fn normal_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let open = MenuItem::with_id(app, "open", "Open Studio", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    Menu::with_items(app, &[&open, &quit_item])
}

/// The menu of the tray while the app waits for the work (ADR 0124 item 20).
fn waiting_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let note = MenuItem::with_id(
        app,
        "waiting",
        "Quits after the write…",
        false,
        None::<&str>,
    )?;
    let open = MenuItem::with_id(app, "open", "Open Studio", true, None::<&str>)?;
    let now = MenuItem::with_id(app, "quit_now", "Quit Now", true, None::<&str>)?;
    Menu::with_items(app, &[&note, &open, &now])
}

/// Change the menu of the tray. The change runs on the main thread.
fn set_tray_menu(app: &AppHandle, waiting: bool) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let menu = if waiting {
            waiting_menu(&handle)
        } else {
            normal_menu(&handle)
        };
        if let (Ok(menu), Some(tray)) = (menu, handle.tray_by_id(&TrayIconId::new(TRAY_ID))) {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

/// Show the dialog of the system. Its answer is a quit event.
fn show_quit_dialog(app: &AppHandle, work: WorkCount) {
    let handle = app.clone();
    app.dialog()
        .message(quit::DIALOG_MESSAGE)
        .title(quit::dialog_title(work))
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            quit::QUIT_AFTER_WRITE.to_string(),
            quit::QUIT_NOW.to_string(),
            quit::CANCEL.to_string(),
        ))
        .show_with_result(move |result| {
            let event = match result {
                MessageDialogResult::Yes => QuitEvent::QuitAfterWrite,
                MessageDialogResult::No => QuitEvent::QuitNow,
                MessageDialogResult::Custom(label) => quit::event_for_label(&label),
                MessageDialogResult::Ok | MessageDialogResult::Cancel => QuitEvent::Cancel,
            };
            handle_quit(&handle, event);
        });
}

/// Read the work each second. When no work runs, the quit goes on. The
/// watcher ends when a newer wait starts or the wait ends.
fn watch_work(app: &AppHandle, generation: u64) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let Some(control) = handle.try_state::<QuitControl>() else {
                return;
            };
            if control.generation.load(Ordering::SeqCst) != generation {
                return;
            }
            let waiting = control.state.lock().is_ok_and(|s| *s == QuitState::Waiting);
            if !waiting {
                return;
            }
            if control.count().is_idle() {
                handle_quit(&handle, QuitEvent::NoWork);
                return;
            }
        }
    });
}

/// Apply one quit event: take the next state, then do the action.
fn handle_quit(app: &AppHandle, event: QuitEvent) {
    let Some(control) = app.try_state::<QuitControl>() else {
        end_now(app);
        return;
    };
    let work = control.count();
    let (before, action) = {
        let Ok(mut state) = control.state.lock() else {
            return;
        };
        let before = *state;
        let (next, action) = quit::step(work, before, event);
        *state = next;
        (before, action)
    };
    match action {
        QuitAction::EndNow => end_now(app),
        QuitAction::ShowDialog => show_quit_dialog(app, work),
        QuitAction::Wait => {
            if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                let _ = window.hide();
            }
            set_tray_menu(app, true);
            let generation = control.generation.fetch_add(1, Ordering::SeqCst) + 1;
            watch_work(app, generation);
        }
        QuitAction::StopApiAndEnd => stop_api_and_end(app),
        QuitAction::Continue => {
            if before == QuitState::Waiting && event == QuitEvent::OpenStudio {
                set_tray_menu(app, false);
            }
        }
    }
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
    let menu = normal_menu(app)?;
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                show_window(app);
                handle_quit(app, QuitEvent::OpenStudio);
            }
            "quit" => handle_quit(app, QuitEvent::Quit),
            "quit_now" => handle_quit(app, QuitEvent::QuitNow),
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
    let (ended_sender, ended_receiver) = oneshot::channel::<()>();
    app.manage(Shutdown(Mutex::new(Some(sender))));
    app.manage(ApiEnded(Mutex::new(Some(ended_receiver))));
    app.manage(QuitControl {
        state: Mutex::new(QuitState::Running),
        work: started.work,
        generation: AtomicU64::new(0),
    });
    let serve = started.serve;
    tauri::async_runtime::spawn(async move {
        if let Err(error) = serve.await {
            eprintln!("studio-desktop-app: the API stopped: {error}");
        }
        let _ = ended_sender.send(());
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
