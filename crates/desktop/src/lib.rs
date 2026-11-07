//! Start of the desktop shell of musicindex-studio.
//!
//! `studio-desktop` and the Tauri app (ADR 0124) call this library. It binds
//! a loopback address (ADR 0017), opens the workspace and makes the access
//! token. It gives the UI address and a future that serves the API.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::pin::Pin;

use studio_api::{Access, Workspace, local_router_with_drops};
pub use studio_api::{DroppedFiles, OnClose, WindowClose, Work};
use tokio::net::TcpListener;

/// The port that the desktop shells try first (ADR 0124 item 7). A fixed
/// port lets the browser or webview keep the settings of the UI.
pub const DEFAULT_PORT: u16 = 42069;

/// The default data folder: the local data folder of the operating system.
/// On macOS, an earlier data folder continues to be used when only it has data.
#[must_use]
pub fn default_data_dir() -> Option<PathBuf> {
    let path = directories::ProjectDirs::from("", "", "musicindex-studio")?
        .data_local_dir()
        .to_path_buf();
    #[cfg(target_os = "macos")]
    {
        let old_path = directories::BaseDirs::new()?
            .home_dir()
            .join(".local/share/musicindex-studio");
        let has_data = |folder: &Path| {
            folder.join("collection.sqlite").exists() || folder.join("vault.json").exists()
        };
        if has_data(&old_path) && !has_data(&path) {
            return Some(old_path);
        }
    }
    Some(path)
}

/// Bind a loopback port (ADR 0017).
///
/// With `None`, bind `127.0.0.1:42069`. When a different program uses that
/// port, bind a free port. With `Some(n)`, bind `127.0.0.1:n` only, and
/// `Some(0)` gives a free port.
///
/// # Errors
///
/// Returns the error of the operating system when the bind fails.
pub async fn bind(port: Option<u16>) -> std::io::Result<TcpListener> {
    match port {
        Some(port) => TcpListener::bind(loopback(port)).await,
        None => bind_first_free(DEFAULT_PORT).await,
    }
}

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

/// Bind `port`, or a free port when a different program uses it.
async fn bind_first_free(port: u16) -> std::io::Result<TcpListener> {
    match TcpListener::bind(loopback(port)).await {
        Ok(listener) => Ok(listener),
        Err(_) => TcpListener::bind(loopback(0)).await,
    }
}

/// Mark the process as not dumpable, so that no core dump holds a vault
/// key (ADR 0104 item 1). A failure gives a warning only. Each desktop
/// process calls this function first.
pub fn harden_process() {
    #[cfg(target_os = "linux")]
    if let Err(error) =
        rustix::process::set_dumpable_behavior(rustix::process::DumpableBehavior::NotDumpable)
    {
        eprintln!("studio-desktop: warning: the process cannot become not dumpable: {error}");
    }
}

/// The settings that the start of the shell needs.
#[derive(Debug, Clone)]
pub struct StartOptions {
    /// The folder of the vault and the collection.
    pub data_dir: PathBuf,
    /// A different musicindex API address.
    pub musicindex: Option<String>,
    /// A different hub for the announcements.
    pub hub: Option<String>,
    /// The shell has a window, so the user can choose what a close does.
    pub has_window: bool,
}

impl StartOptions {
    /// Options with the given data folder and the default addresses.
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
            musicindex: None,
            hub: None,
            has_window: false,
        }
    }
}

/// The future that serves the API until the shutdown signal.
pub type Serve = Pin<Box<dyn Future<Output = Result<(), StartError>> + Send>>;

/// A started shell.
pub struct Started {
    /// The UI address with `#token=`. It holds a secret.
    pub ui_url: String,
    /// The bound port.
    pub port: u16,
    /// Serves the API. After the shutdown signal, it accepts no connection,
    /// waits for each request that started, and then ends.
    pub serve: Serve,
    /// The count of the writes and the uploads that run (ADR 0124 items 16
    /// and 17).
    pub work: studio_api::Work,
    /// The file of the last drop on the window of the app (ADR 0124 item 22).
    pub dropped: studio_api::DroppedFiles,
    /// The choice for a close of the window (ADR 0124 item 29).
    pub window_close: studio_api::WindowClose,
}

impl std::fmt::Debug for Started {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The UI address holds the token, so it stays out of the output.
        f.debug_struct("Started")
            .field("port", &self.port)
            .finish_non_exhaustive()
    }
}

/// An error of the start or of the serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartError {
    message: String,
    already_running: bool,
}

impl StartError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            already_running: false,
        }
    }

    /// True when a different shell uses the data folder (ADR 0124 item 10).
    #[must_use]
    pub fn already_running(&self) -> bool {
        self.already_running
    }
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StartError {}

/// The name of the lock file in the data folder.
const LOCK_FILE: &str = "studio.lock";

/// Lock the data folder, so that only one shell uses it (ADR 0124 item 10).
///
/// The single instance plugin of the app needs D-Bus on Linux. The lock
/// also works with no D-Bus, and it also stops a second `studio-desktop`.
/// The system removes the lock when the file closes or the process ends.
/// A missing data folder gets the mode 0700 (ADR 0104 item 2).
///
/// # Errors
///
/// Returns an error with [`StartError::already_running`] when a different
/// shell holds the lock, and a different error when the file fails.
pub fn lock_data_dir(data_dir: &Path) -> Result<std::fs::File, StartError> {
    if !data_dir.exists() {
        if let Some(parent) = data_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StartError::new(format!("{}: {e}", parent.display())))?;
        }
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        match builder.create(data_dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(StartError::new(format!("{}: {e}", data_dir.display()))),
        }
    }
    let path = data_dir.join(LOCK_FILE);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| StartError::new(format!("{}: {e}", path.display())))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(StartError {
            message: "Studio is already running with this data folder.".to_string(),
            already_running: true,
        }),
        Err(std::fs::TryLockError::Error(e)) => {
            Err(StartError::new(format!("{}: {e}", path.display())))
        }
    }
}

/// Open the workspace, make the token and the router, and prepare the serve.
///
/// The caller binds `listener` with [`bind`]. The returned future serves
/// until `shutdown` completes, and then waits for each request that
/// started. It holds the lock of [`lock_data_dir`] until
/// it ends.
///
/// # Errors
///
/// Returns an error when the lock, the workspace, the address or the token
/// fails. [`StartError::already_running`] tells when a different shell
/// uses the data folder.
pub fn start<S>(
    options: &StartOptions,
    listener: TcpListener,
    shutdown: S,
) -> Result<Started, StartError>
where
    S: Future<Output = ()> + Send + 'static,
{
    let lock = lock_data_dir(&options.data_dir)?;
    let mut workspace =
        Workspace::open(&options.data_dir).map_err(|e| StartError::new(format!("{e:?}")))?;
    if let Some(base) = &options.musicindex {
        workspace = workspace.with_musicindex(base);
    }
    if let Some(url) = &options.hub {
        workspace = workspace.with_hub(url);
    }
    // Keep the remote file records of each feed current (ADR 0038 item 3).
    // The function does nothing when Studio has no library folder yet.
    workspace.record_all_remote_files();
    // Continue each download that was not complete (ADR 0038 item 9).
    workspace.resume_downloads();
    // Prepare the announcement queue (ADR 0052 item 18).
    workspace.resume_announcements();
    let port = listener
        .local_addr()
        .map_err(|e| StartError::new(e.to_string()))?
        .port();
    let access = Access::new(port).map_err(|e| StartError::new(format!("{e:?}")))?;
    let ui_url = access.ui_url();
    let work = workspace.work();
    let dropped = studio_api::DroppedFiles::default();
    let window_close = if options.has_window {
        studio_api::WindowClose::in_folder(&options.data_dir)
    } else {
        studio_api::WindowClose::default()
    };
    let router = local_router_with_drops(workspace, access, dropped.clone(), window_close.clone());
    let serve = async move {
        // The lock stays until the serve ends.
        let _lock = lock;
        studio_api::serve(listener, router, shutdown)
            .await
            .map_err(|e| StartError::new(e.to_string()))
    };
    Ok(Started {
        ui_url,
        port,
        serve: Box::pin(serve),
        work,
        dropped,
        window_close,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_port_is_42069() {
        assert_eq!(
            DEFAULT_PORT, 42069,
            "ADR 0124 item 7: the desktop shells try port 42069 first."
        );
    }

    // These two tests use a port of their own, because a Studio of the user
    // or a parallel test run can hold port 42069 at any time.
    #[tokio::test]
    async fn a_held_first_port_gives_a_free_port() {
        let holder = bind(Some(0)).await.expect("bind(Some(0)) must bind");
        let held = holder.local_addr().unwrap().port();
        let other = bind_first_free(held)
            .await
            .expect("the bind must fall back");
        assert_ne!(
            other.local_addr().unwrap().port(),
            held,
            "ADR 0124 item 7: bind(None) must use a free port when 42069 is in use."
        );
        assert!(other.local_addr().unwrap().ip().is_loopback());
    }

    #[tokio::test]
    async fn a_free_first_port_is_used() {
        // A different program can take a released port. One of the attempts
        // gets a port that stays free.
        for _ in 0..5 {
            let probe = bind(Some(0)).await.expect("bind(Some(0)) must bind");
            let port = probe.local_addr().unwrap().port();
            drop(probe);
            let listener = bind_first_free(port).await.expect("the bind must work");
            if listener.local_addr().unwrap().port() == port {
                return;
            }
        }
        panic!("ADR 0124 item 7: bind(None) must use port 42069 when it is free.");
    }

    #[tokio::test]
    async fn a_given_port_in_use_gives_an_error() {
        let first = bind(Some(0)).await.expect("bind(Some(0)) must bind");
        let port = first.local_addr().unwrap().port();
        assert!(
            bind(Some(port)).await.is_err(),
            "ADR 0124 item 7: bind(Some(n)) must not fall back when n is in use."
        );
    }

    #[test]
    fn a_second_lock_of_the_data_folder_fails() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        let first = lock_data_dir(&data_dir).expect("the first lock must succeed");
        let second =
            lock_data_dir(&data_dir).expect_err("ADR 0124 item 10: a second lock must fail");
        assert!(
            second.already_running(),
            "ADR 0124 item 10: a second shell on one data folder must get already_running()."
        );
        drop(first);
        assert!(
            lock_data_dir(&data_dir).is_ok(),
            "ADR 0124 item 10: the lock must be free after the first shell ends."
        );
        #[cfg(unix)]
        assert_eq!(
            std::os::unix::fs::PermissionsExt::mode(
                &std::fs::metadata(&data_dir).unwrap().permissions()
            ) & 0o777,
            0o700,
            "ADR 0104 item 2: a new data folder must have the mode 0700."
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_start_step_marks_the_process_not_dumpable() {
        harden_process();
        assert_eq!(
            rustix::process::dumpable_behavior().unwrap(),
            rustix::process::DumpableBehavior::NotDumpable,
            "ADR 0104 item 1: the desktop must be not dumpable. Call harden_process() at the start of run()."
        );
    }

    #[tokio::test]
    async fn the_serve_ends_after_the_shutdown_with_an_idle_connection() {
        let dir = tempfile::tempdir().unwrap();
        let listener = bind(Some(0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let started = start(
            &StartOptions::new(&dir.path().join("data")),
            listener,
            async {
                let _ = stopped.await;
            },
        )
        .expect("the start must work");
        assert_eq!(started.work.writes(), 0);
        // A connection that sent no request must not hold the shutdown.
        let _idle = tokio::net::TcpStream::connect(address).await.unwrap();
        let serve = tokio::spawn(started.serve);
        let _ = stop.send(());
        let done = tokio::time::timeout(std::time::Duration::from_secs(5), serve).await;
        assert!(
            done.is_ok(),
            "ADR 0124 item 20: the serve must end after the shutdown signal."
        );
    }
}
