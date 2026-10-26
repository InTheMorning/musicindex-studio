//! The limits of the server: the Argon2id derivations, the failed sign-ins
//! and the address of the client (ADR 0055 items 22 and 23).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request};
use axum::http::StatusCode;
use axum::response::Response;
use tokio::sync::OwnedSemaphorePermit;

use crate::session::{ServerState, failure};

/// The failed sign-ins that one account can have in one window.
pub const ACCOUNT_LIMIT: usize = 10;
/// The failed sign-ins that one client address can have in one window.
pub const ADDRESS_LIMIT: usize = 30;
/// The length of the window.
const WINDOW: Duration = Duration::from_hours(1);
/// The longest client address that the counter keeps.
const MAX_ADDRESS_CHARS: usize = 64;

/// The failed sign-ins of each account and of each client address. The
/// counter lives in memory only, so a restart clears it.
#[derive(Debug, Default)]
pub struct Attempts {
    failures: Mutex<HashMap<String, Vec<Instant>>>,
}

impl Attempts {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Vec<Instant>>> {
        // A poisoned lock only means that another thread failed. The map is
        // still usable.
        self.failures
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn count(map: &mut HashMap<String, Vec<Instant>>, key: &str) -> usize {
        let Some(times) = map.get_mut(key) else {
            return 0;
        };
        times.retain(|time| time.elapsed() < WINDOW);
        times.len()
    }

    /// True when the account or the address has too many failures in the
    /// window. A sign-in then gets 429 with no derivation.
    pub fn blocked(&self, account: &str, address: &str) -> bool {
        let mut map = self.lock();
        Self::count(&mut map, &account_key(account)) >= ACCOUNT_LIMIT
            || Self::count(&mut map, &address_key(address)) >= ADDRESS_LIMIT
    }

    /// Count one failed sign-in for the account and for the address.
    pub fn record_failure(&self, account: &str, address: &str) {
        let mut map = self.lock();
        let now = Instant::now();
        for key in [account_key(account), address_key(address)] {
            map.entry(key).or_default().push(now);
        }
    }

    /// Forget the entries with no failure in the window.
    pub fn prune(&self) {
        self.lock().retain(|_, times| {
            times.retain(|time| time.elapsed() < WINDOW);
            !times.is_empty()
        });
    }
}

fn account_key(account: &str) -> String {
    format!("account:{account}")
}

fn address_key(address: &str) -> String {
    format!("address:{address}")
}

/// The address of the client of a request. With a header option, the value
/// of that header counts, and its last item is the one that the reverse
/// proxy added. Without the option, or when the header is missing, the
/// address of the connection counts.
pub fn client_address(header: Option<&str>, request: &Request) -> String {
    let from_header = header
        .and_then(|name| request.headers().get(name))
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.rsplit(',').next())
        .map(str::trim)
        .filter(|text| !text.is_empty());
    if let Some(text) = from_header {
        return text.chars().take(MAX_ADDRESS_CHARS).collect();
    }
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map_or_else(|| "unknown".to_owned(), |info| info.0.ip().to_string())
}

/// Wait for one of the permits of the Argon2id derivations. A request that
/// waits longer than the wait of the server gets 503 `busy`
/// (ADR 0055 item 22).
///
/// # Errors
///
/// Gives the 503 response when no permit comes in time.
pub async fn derivation_permit(state: &ServerState) -> Result<OwnedSemaphorePermit, Response> {
    let busy = || {
        failure(
            StatusCode::SERVICE_UNAVAILABLE,
            "busy",
            "the server is busy: try again in a short time",
        )
    };
    match tokio::time::timeout(
        state.argon2_wait,
        std::sync::Arc::clone(&state.argon2).acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Ok(permit),
        _ => Err(busy()),
    }
}

/// The response for too many failed sign-ins.
pub fn too_many_attempts() -> Response {
    failure(
        StatusCode::TOO_MANY_REQUESTS,
        "too_many_attempts",
        "too many failed sign-ins: try again later",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_account_is_blocked_after_ten_failures_and_an_address_after_thirty() {
        let attempts = Attempts::default();
        for _ in 0..ACCOUNT_LIMIT - 1 {
            attempts.record_failure("a@example.org", "1.1.1.1");
        }
        assert!(!attempts.blocked("a@example.org", "1.1.1.1"));
        attempts.record_failure("a@example.org", "1.1.1.1");
        assert!(attempts.blocked("a@example.org", "2.2.2.2"));
        assert!(!attempts.blocked("b@example.org", "2.2.2.2"));
        for i in 0..ADDRESS_LIMIT {
            attempts.record_failure(&format!("{i}@example.org"), "3.3.3.3");
        }
        assert!(attempts.blocked("new@example.org", "3.3.3.3"));
        attempts.prune();
        assert!(attempts.blocked("new@example.org", "3.3.3.3"));
    }
}
