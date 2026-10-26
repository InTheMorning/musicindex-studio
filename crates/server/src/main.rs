//! Server shell of musicindex-studio.
//!
//! A host operator runs this binary to give a public service to web users
//! (ADR 0001, ADR 0055).
//!
//! Commands:
//!
//! - `serve --data-dir PATH --public-origin URL`: run the server. The
//!   options `--listen ADDR` (default `127.0.0.1:8080`), `--signup
//!   open|invite` (default `invite`), `--client-address-header NAME`,
//!   `--musicindex URL` and `--podcastindex-hub URL` are optional.
//!   Limit options: `--session-idle-days` (default 7), `--session-max-days`
//!   (default 30), `--vault-idle-minutes` (default 10), `--storage-limit-gib`
//!   (default 10) and `--argon2-concurrency` (default 2).
//! - `invite --data-dir PATH`: print one new single-use invite code.

// The handlers give an axum response as the error. The response is large by design.
#![expect(
    clippy::result_large_err,
    reason = "an axum handler gives its error as a response"
)]

mod accounts;
mod auth;
mod config;
mod idle;
mod limits;
mod session;
mod users;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use studio_core::net::PublicRules;
use studio_core::vault::OpenVault;

use config::{Command, ServeOptions};
use session::ServerState;
use users::{UserSettings, Users};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("studio-server: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    match config::parse(std::env::args().skip(1))? {
        Command::Invite { data_dir } => {
            let accounts = accounts::Accounts::open(&data_dir)?;
            println!("{}", accounts.create_invite()?);
            Ok(())
        }
        Command::Serve(options) => serve(*options).await,
    }
}

/// Make the state of the server. `rules` are the exceptions of the network
/// policy `Public`. Only a test build gives exceptions.
fn make_state(options: &ServeOptions, rules: PublicRules) -> Result<session::Shared, String> {
    let accounts = accounts::Accounts::open(&options.data_dir)?;
    // The passphrase is random and nobody keeps it.
    let dummy = OpenVault::create(&accounts::random_hex(16)?).map_err(|e| e.to_string())?;
    let users = Users::new(UserSettings {
        data_dir: options.data_dir.clone(),
        musicindex: options.musicindex.clone(),
        hub: options.hub.clone(),
        rules,
        storage_limit: options.storage_limit,
    });
    Ok(Arc::new(ServerState {
        accounts,
        users,
        origin: options.origin.clone(),
        signup: options.signup,
        dummy_wrapper: dummy.file().clone(),
        session_idle: options.session_idle,
        session_max: options.session_max,
        vault_idle: options.vault_idle,
        argon2: Arc::new(tokio::sync::Semaphore::new(options.argon2_concurrency)),
        argon2_wait: options.argon2_wait,
        attempts: limits::Attempts::default(),
        client_header: options.client_address_header.clone(),
    }))
}

async fn serve(options: ServeOptions) -> Result<(), String> {
    let state = make_state(&options, PublicRules::default())?;
    // Open each user with work to do (ADR 0055 item 4).
    state.users.open_busy(&state.accounts.folders()?).await;
    let listener = tokio::net::TcpListener::bind(options.listen)
        .await
        .map_err(|e| e.to_string())?;
    println!("Music Index Studio server {}", env!("CARGO_PKG_VERSION"));
    println!("Data: {}", options.data_dir.display());
    println!("Listening on {}", options.listen);
    println!("Public origin: {}", options.origin.text);
    idle::spawn(Arc::clone(&state));
    // The connection address gives the client address when the operator
    // names no header (ADR 0055 item 23).
    let service = auth::server_router(&state).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service)
        .await
        .map_err(|e| e.to_string())
}
