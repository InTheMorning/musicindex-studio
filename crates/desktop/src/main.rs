//! Desktop shell of musicindex-studio.
//!
//! The artist runs this binary on their own machine (ADR 0001). It runs the
//! local API on a loopback address and prints the UI address with the access
//! token (ADR 0017). The Tauri window comes in Phase 10.
//!
//! Options:
//!
//! - `--data-dir PATH`: the folder of the vault and the collection. The
//!   default is `$XDG_DATA_HOME/musicindex-studio`, or
//!   `~/.local/share/musicindex-studio`.
//! - `--port N`: the port. The default 0 lets the system select a free port.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::ExitCode;

use studio_api::{Access, Workspace, local_router};

struct Options {
    data_dir: PathBuf,
    port: u16,
}

fn default_data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("musicindex-studio"))
}

fn options() -> Result<Options, String> {
    let mut data_dir = None;
    let mut port = 0;
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
            other => return Err(format!("unknown option {other}")),
        }
    }
    let data_dir = data_dir
        .or_else(default_data_dir)
        .ok_or("give --data-dir, because HOME is not set")?;
    Ok(Options { data_dir, port })
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
    let options = options()?;
    let workspace = Workspace::open(&options.data_dir).map_err(|e| format!("{e:?}"))?;
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
