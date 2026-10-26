//! The task that locks idle vaults and closes idle users (ADR 0055 items 4
//! and 16).

use std::time::Duration;

use studio_api::VaultState;

use crate::session::Shared;

/// How often the task runs.
const PERIOD: Duration = Duration::from_secs(30);

/// Run the sweep each 30 seconds, until the process stops.
pub fn spawn(state: Shared) {
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(PERIOD);
        loop {
            timer.tick().await;
            sweep(&state).await;
        }
    });
}

/// One run of the task. For each open user:
///
/// - With no active session, the vault locks.
/// - With a session, the vault locks after the idle limit with no use of a
///   secret.
/// - A user with no session, a closed vault, no waiting announcement and no
///   queued download closes. The router and the handle go out of memory.
pub async fn sweep(state: &Shared) {
    if let Err(message) = state
        .accounts
        .delete_expired_sessions(state.session_idle, state.session_max)
    {
        eprintln!("studio-server: the session cleanup failed: {message}");
    }
    state.attempts.prune();
    // A signer goes away with its session. A connection that nobody used
    // goes away after five minutes.
    let gone = state
        .nip46
        .sweep(|hash| matches!(state.accounts.session(hash), Ok(Some(_))));
    crate::session::forget_signers(state, gone).await;
    for (id, user) in state.users.snapshot().await {
        let active = state
            .accounts
            .has_session(id, state.session_idle, state.session_max)
            .unwrap_or(true);
        if active {
            user.handle.lock_if_idle(state.vault_idle).await;
            continue;
        }
        let open = matches!(user.handle.vault_state().await, Ok(VaultState::Open));
        if open {
            let _ = user.handle.lock().await;
        }
        let Ok(Some(folder)) = state.accounts.folder_of(id) else {
            // The account is gone. Nothing can reach the user.
            state.users.forget(id).await;
            continue;
        };
        if !state.users.is_busy_folder(&folder).await {
            state.users.forget(id).await;
        }
    }
}
