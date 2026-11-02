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
//!   `--musicindex URL`, `--podcastindex-hub URL` and `--nip46-relay URL`
//!   (more than one time, default `wss://relay.nsec.app`) are optional.
//!   Limit options: `--session-idle-days` (default 7), `--session-max-days`
//!   (default 30), `--vault-idle-minutes` (default 10), `--storage-limit-gib`
//!   (default 10) and `--argon2-concurrency` (default 2). The option
//!   `--hold-assets off|on` (default `off`) sets whether the server keeps
//!   audio files and images (ADR 0105).
//! - `invite --data-dir PATH`: print one new single-use invite code.

// The handlers give an axum response as the error. The response is large by design.
#![expect(
    clippy::result_large_err,
    reason = "an axum handler gives its error as a response"
)]

mod accounts;
mod apps;
mod auth;
mod config;
mod idle;
mod import;
mod limits;
mod nip46_sessions;
mod session;
mod users;
mod webauthn;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use studio_core::net::{NetPolicy, PublicRules};
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

/// Mark the process as not dumpable, so that no core dump holds a vault
/// key (ADR 0104 item 1).
///
/// # Errors
///
/// Gives a message when the system refuses the call.
fn harden_process() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    rustix::process::set_dumpable_behavior(rustix::process::DumpableBehavior::NotDumpable)
        .map_err(|e| format!("the process cannot become not dumpable: {e}"))?;
    Ok(())
}

async fn run() -> Result<(), String> {
    harden_process()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The same two options as `studio-desktop`. The image smoke test of
    // ADR 0066 runs them.
    if args == ["--version"] {
        println!("Music Index Studio server {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["--help"] {
        println!("Usage: studio-server serve --data-dir PATH --public-origin URL [OPTIONS]");
        println!("       studio-server invite --data-dir PATH");
        println!("       studio-server --version | --help");
        println!("The runbook docs/runbooks/studio-server.md gives each option.");
        return Ok(());
    }
    match config::parse(args)? {
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
    let accounts = Arc::new(accounts::Accounts::open(&options.data_dir)?);
    // A folder that an older deletion could not remove (task 072c item 3).
    accounts.remove_leftover_folders(&options.data_dir.join("users"))?;
    // The passphrase is random and nobody keeps it.
    let mut dummy = OpenVault::create(&accounts::random_hex(16)?).map_err(|e| e.to_string())?;
    // A recovery sign-in with an unknown account opens this vault with the
    // given key. Nobody keeps the recovery key.
    drop(dummy.add_recovery_key().map_err(|e| e.to_string())?);
    let nip46 = nip46_sessions::Nip46Sessions::new(
        options.nip46_relays.clone(),
        NetPolicy::Public(rules.clone()),
        options.nip46_timeout,
    );
    let users = Users::new(UserSettings {
        data_dir: options.data_dir.clone(),
        musicindex: options.musicindex.clone(),
        hub: options.hub.clone(),
        rules,
        storage_limit: options.storage_limit,
        hold_assets: options.hold_assets,
        nip46_relays: options.nip46_relays.clone(),
        accounts: Arc::clone(&accounts),
    });
    // A lost row must not stop an app (ADR 0058 item 16).
    users::rebuild_app_tokens(&users, &accounts)?;
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
        nip46,
        app_limits: limits::AppLimits::default(),
        challenges: limits::ChallengeLimits::default(),
        imports: studio_api::ImportStore::new(options.data_dir.join("import-staging")),
        import_starts: limits::ImportStarts::default(),
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

#[cfg(test)]
mod hardening_tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn the_start_step_marks_the_process_not_dumpable() {
        super::harden_process().unwrap();
        assert_eq!(
            rustix::process::dumpable_behavior().unwrap(),
            rustix::process::DumpableBehavior::NotDumpable,
            "ADR 0104 item 1: the server must be not dumpable. Call harden_process() at the start of run()."
        );
    }

    #[test]
    fn the_compose_file_sets_the_core_limit_to_0() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../deploy/compose.yml");
        let text = std::fs::read_to_string(path).unwrap();
        let service = text
            .split("\nvolumes:")
            .next()
            .unwrap()
            .split("\n  studio:")
            .nth(1)
            .expect("the service studio");
        let lines: Vec<&str> = service
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with('#'))
            .collect();
        let at = lines.iter().position(|l| l.starts_with("ulimits:"));
        let has_core = at.is_some_and(|i| {
            lines[i].contains("core: 0") || lines.get(i + 1).is_some_and(|l| l.contains("core: 0"))
        });
        assert!(
            has_core,
            "ADR 0104 item 1: the service studio needs `ulimits: {{ core: 0 }}` in deploy/compose.yml."
        );
    }
}
