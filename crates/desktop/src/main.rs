//! Desktop shell of musicindex-studio.
//!
//! The artist runs this binary on their own machine (ADR 0001). It runs the
//! local API on a loopback address and prints the UI address with the access
//! token (ADR 0017). The Tauri window comes in the Desktop Shell
//! phase.
//!
//! Options:
//!
//! - `--data-dir PATH`: the folder of the vault and the collection. The
//!   default is the local application data folder of the operating system.
//! - `--port N`: the port. The default 0 lets the system select a free port.
//! - `--musicindex URL`: the musicindex API. The default is
//!   `https://api.musicindex.org`.
//! - `--podcastindex-hub URL`: the hub that gets the announcements after a
//!   write. The default is `https://hub.podcastindex.org/pubnotify`.
//! - `--version`: print the version and stop.
//! - `--help`: print the options and stop.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::ExitCode;

use studio_api::{Access, Workspace, local_router};

struct Options {
    data_dir: PathBuf,
    port: u16,
    musicindex: Option<String>,
    hub: Option<String>,
}

fn default_data_dir() -> Option<PathBuf> {
    let path = directories::ProjectDirs::from("", "", "musicindex-studio")?
        .data_local_dir()
        .to_path_buf();
    #[cfg(target_os = "macos")]
    {
        let old_path = directories::BaseDirs::new()?
            .home_dir()
            .join(".local/share/musicindex-studio");
        let has_data = |folder: &std::path::Path| {
            folder.join("collection.sqlite").exists() || folder.join("vault.json").exists()
        };
        if has_data(&old_path) && !has_data(&path) {
            return Some(old_path);
        }
    }
    Some(path)
}

fn options() -> Result<Options, String> {
    let mut data_dir = None;
    let mut port = 0;
    let mut musicindex = None;
    let mut hub = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(args.next().ok_or("--data-dir needs a path")?));
            }
            "--port" => {
                port = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .ok_or("--port needs a number from 0 to 65535")?;
            }
            "--musicindex" => {
                musicindex = Some(args.next().ok_or("--musicindex needs an address")?);
            }
            "--podcastindex-hub" => {
                hub = Some(args.next().ok_or("--podcastindex-hub needs an address")?);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    let data_dir = data_dir
        .or_else(default_data_dir)
        .ok_or("give --data-dir, because the system has no application data folder")?;
    Ok(Options {
        data_dir,
        port,
        musicindex,
        hub,
    })
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("studio-desktop: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!("Music Index Studio {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["--help"] {
        println!(
            "Usage: studio-desktop [--data-dir PATH] [--port N] [--musicindex URL] [--podcastindex-hub URL]"
        );
        println!("       studio-desktop --version | --help");
        return Ok(());
    }
    let options = options()?;
    let mut workspace = Workspace::open(&options.data_dir).map_err(|e| format!("{e:?}"))?;
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
    // ADR 0017: listen on a loopback address only.
    let listener =
        tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, options.port)))
            .await
            .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let access = Access::new(port).map_err(|e| format!("{e:?}"))?;
    println!("Music Index Studio {}", env!("CARGO_PKG_VERSION"));
    println!("Data: {}", options.data_dir.display());
    println!("Open this address in your browser:");
    println!("{}", access.ui_url());
    axum_serve(listener, local_router(workspace, access)).await
}

async fn axum_serve(
    listener: tokio::net::TcpListener,
    router: studio_api::Router,
) -> Result<(), String> {
    studio_api::serve(listener, router)
        .await
        .map_err(|e| e.to_string())
}
