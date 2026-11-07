//! Desktop shell of musicindex-studio.
//!
//! The artist runs this binary on their own machine (ADR 0001). It runs the
//! local API on a loopback address and prints the UI address with the access
//! token (ADR 0017). The start is in the library of this crate, which the
//! Tauri app also calls (ADR 0124).
//!
//! Options:
//!
//! - `--data-dir PATH`: the folder of the vault and the collection. The
//!   default is the selected collection in the local application data folder
//!   of the operating system (ADR 0149).
//! - `--port N`: the port. The default is 42069, else a free port. `0` gives
//!   a free port.
//! - `--musicindex URL`: the musicindex API. The default is
//!   `https://api.musicindex.org`.
//! - `--podcastindex-hub URL`: the hub that gets the announcements after a
//!   write. The default is `https://hub.podcastindex.org/pubnotify`.
//! - `--version`: print the version and stop.
//! - `--help`: print the options and stop.

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use studio_desktop::{
    DEFAULT_PORT, StartOptions, bind, default_data_dir, harden_process, shell_folder, start,
};

struct Options {
    data_dir: PathBuf,
    port: Option<u16>,
    musicindex: Option<String>,
    hub: Option<String>,
}

fn options() -> Result<Options, String> {
    let mut data_dir = None;
    let mut port = None;
    let mut musicindex = None;
    let mut hub = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(args.next().ok_or("--data-dir needs a path")?));
            }
            "--port" => {
                port = Some(
                    args.next()
                        .and_then(|p| p.parse().ok())
                        .ok_or("--port needs a number from 0 to 65535")?,
                );
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
    let root = if data_dir.is_some() {
        None
    } else {
        default_data_dir()
    };
    let data_dir = shell_folder(data_dir, root)
        .map_err(|e| e.to_string())?
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
    harden_process();
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
        println!("The default port is {DEFAULT_PORT}, else a free port.");
        return Ok(());
    }
    let options = options()?;
    // ADR 0017: listen on a loopback address only.
    let listener = bind(options.port).await.map_err(|e| e.to_string())?;
    let mut start_options = StartOptions::new(&options.data_dir);
    start_options.musicindex = options.musicindex;
    start_options.hub = options.hub;
    // The shell ends at Ctrl+C with no wait (ADR 0124 item 21), so the
    // shutdown future of the serve does not end.
    let started =
        start(&start_options, listener, std::future::pending()).map_err(|e| e.to_string())?;
    println!("Music Index Studio {}", env!("CARGO_PKG_VERSION"));
    println!("Data: {}", options.data_dir.display());
    if options.port.is_none() && started.port != DEFAULT_PORT {
        println!(
            "Port {DEFAULT_PORT} is in use. Studio uses port {} for this start. The UI has its default settings for this start.",
            started.port
        );
    }
    println!("Open this address in your browser:");
    println!("{}", started.ui_url);
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    tokio::select! {
        result = started.serve => result.map_err(|e| e.to_string()),
        // A failed signal handler also stops the shell.
        _ = tokio::signal::ctrl_c() => Ok(()),
    }
}
