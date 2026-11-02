//! The limits of the server: the Argon2id derivations, the failed sign-ins,
//! the challenges and the address of the client (ADR 0055 item 22, ADR 0106
//! items 4 and 5).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request};
use axum::http::StatusCode;
use axum::response::Response;
use tokio::sync::OwnedSemaphorePermit;

use crate::session::{ServerState, failure};

/// The failed sign-ins that one account can have from one client address in
/// one window (ADR 0106 item 4).
pub const PAIR_LIMIT: usize = 10;
/// The failed sign-ins that one account can have from all addresses in one
/// window (ADR 0106 item 4).
pub const ACCOUNT_LIMIT: usize = 100;
/// The failed sign-ins that one client address can have in one window.
pub const ADDRESS_LIMIT: usize = 30;
/// The length of the window.
const WINDOW: Duration = Duration::from_hours(1);
/// The longest client address that the counter keeps.
const MAX_ADDRESS_CHARS: usize = 64;

/// The failed sign-ins of each pair of account and client address, of each
/// account and of each client address. The counter lives in memory only, so
/// a restart clears it.
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

    /// True when the pair of account and address, the account or the address
    /// has too many failures in the window. A sign-in then gets 429 with no
    /// derivation. A third party at one address blocks only its own pair
    /// (ADR 0106 item 4).
    pub fn blocked(&self, account: &str, address: &str) -> bool {
        let mut map = self.lock();
        Self::count(&mut map, &pair_key(account, address)) >= PAIR_LIMIT
            || Self::count(&mut map, &account_key(account)) >= ACCOUNT_LIMIT
            || Self::count(&mut map, &address_key(address)) >= ADDRESS_LIMIT
    }

    /// Count one failed sign-in for the pair, for the account and for the
    /// address.
    pub fn record_failure(&self, account: &str, address: &str) {
        let mut map = self.lock();
        let now = Instant::now();
        for key in [
            pair_key(account, address),
            account_key(account),
            address_key(address),
        ] {
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

fn pair_key(account: &str, address: &str) -> String {
    format!("pair:{address}|{account}")
}

/// The challenges that one client address can ask for in one challenge
/// window (ADR 0106 item 5).
pub const CHALLENGE_LIMIT: usize = 30;
/// The length of the challenge window.
const CHALLENGE_WINDOW: Duration = Duration::from_mins(5);

/// The challenges that each client address asked for. The counter lives in
/// memory only, so a restart clears it.
#[derive(Debug, Default)]
pub struct ChallengeLimits {
    asked: Mutex<HashMap<String, Vec<Instant>>>,
}

impl ChallengeLimits {
    /// Count one challenge for the address. Gives false when the address is
    /// over its limit in the window.
    pub fn allow(&self, address: &str) -> bool {
        let mut map = lock_map(&self.asked);
        let times = map.entry(address.to_owned()).or_default();
        times.retain(|time| time.elapsed() < CHALLENGE_WINDOW);
        if times.len() >= CHALLENGE_LIMIT {
            return false;
        }
        times.push(Instant::now());
        true
    }

    /// Forget the entries with no challenge in the window.
    pub fn prune(&self) {
        lock_map(&self.asked).retain(|_, times| {
            times.retain(|time| time.elapsed() < CHALLENGE_WINDOW);
            !times.is_empty()
        });
    }
}

/// The imports that one client address can start in one window (ADR 0107
/// item 9). The routes of an import need no session.
pub const IMPORT_START_LIMIT: usize = 5;
/// The length of the window of the import starts.
const IMPORT_START_WINDOW: Duration = Duration::from_hours(1);

/// The imports that each client address started. The counter lives in
/// memory only, so a restart clears it.
#[derive(Debug, Default)]
pub struct ImportStarts {
    started: Mutex<HashMap<String, Vec<Instant>>>,
}

impl ImportStarts {
    /// Count one start for the address. Gives false when the address is over
    /// its limit in the window.
    pub fn allow(&self, address: &str) -> bool {
        let mut map = lock_map(&self.started);
        let times = map.entry(address.to_owned()).or_default();
        times.retain(|time| time.elapsed() < IMPORT_START_WINDOW);
        if times.len() >= IMPORT_START_LIMIT {
            return false;
        }
        times.push(Instant::now());
        true
    }

    /// Forget the entries with no start in the window.
    pub fn prune(&self) {
        lock_map(&self.started).retain(|_, times| {
            times.retain(|time| time.elapsed() < IMPORT_START_WINDOW);
            !times.is_empty()
        });
    }
}

/// The requests that one app key can make in one minute (ADR 0110
/// item 40).
pub const APP_LIMIT: usize = 60;
/// The requests with an unknown key, a signature that is not correct or a
/// wrong code that one client address can send in one hour (ADR 0110
/// item 40).
pub const UNKNOWN_TOKEN_LIMIT: usize = 20;
/// The length of the window of an app.
const APP_WINDOW: Duration = Duration::from_mins(1);

/// The request counters of the `/app/v1/` layer. The counters live in
/// memory only, so a restart clears them.
#[derive(Debug, Default)]
pub struct AppLimits {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    unknown: Mutex<HashMap<String, Vec<Instant>>>,
}

fn lock_map(
    map: &Mutex<HashMap<String, Vec<Instant>>>,
) -> std::sync::MutexGuard<'_, HashMap<String, Vec<Instant>>> {
    // A poisoned lock only means that another thread failed. The map is
    // still usable.
    map.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl AppLimits {
    /// Count one request of an app. The key is the public key of the app.
    /// Gives false when the app is over its limit in the window.
    pub fn allow_app(&self, key: &str) -> bool {
        let mut map = lock_map(&self.requests);
        let times = map.entry(key.to_owned()).or_default();
        times.retain(|time| time.elapsed() < APP_WINDOW);
        if times.len() >= APP_LIMIT {
            return false;
        }
        times.push(Instant::now());
        true
    }

    /// True when the address has sent too many requests with an unknown key.
    pub fn unknown_blocked(&self, address: &str) -> bool {
        let mut map = lock_map(&self.unknown);
        Attempts::count(&mut map, address) >= UNKNOWN_TOKEN_LIMIT
    }

    /// Count one request with an unknown key for the address only.
    pub fn record_unknown(&self, address: &str) {
        lock_map(&self.unknown)
            .entry(address.to_owned())
            .or_default()
            .push(Instant::now());
    }

    /// Forget the entries with no request in the window.
    pub fn prune(&self) {
        lock_map(&self.requests).retain(|_, times| {
            times.retain(|time| time.elapsed() < APP_WINDOW);
            !times.is_empty()
        });
        lock_map(&self.unknown).retain(|_, times| {
            times.retain(|time| time.elapsed() < WINDOW);
            !times.is_empty()
        });
    }
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
    fn a_pair_is_blocked_after_ten_failures_an_account_after_a_hundred_and_an_address_after_thirty()
    {
        let attempts = Attempts::default();
        for _ in 0..PAIR_LIMIT - 1 {
            attempts.record_failure("a@example.org", "1.1.1.1");
        }
        assert!(!attempts.blocked("a@example.org", "1.1.1.1"));
        attempts.record_failure("a@example.org", "1.1.1.1");
        assert!(
            attempts.blocked("a@example.org", "1.1.1.1"),
            "ADR 0106 item 4: ten failures block the pair of account and address."
        );
        assert!(
            !attempts.blocked("a@example.org", "2.2.2.2"),
            "ADR 0106 item 4: a different address must not be blocked by the pair."
        );
        assert!(!attempts.blocked("b@example.org", "1.1.1.1"));
        for i in 0..ADDRESS_LIMIT {
            attempts.record_failure(&format!("{i}@example.org"), "3.3.3.3");
        }
        assert!(attempts.blocked("new@example.org", "3.3.3.3"));
        attempts.prune();
        assert!(attempts.blocked("new@example.org", "3.3.3.3"));
    }

    #[test]
    fn a_hundred_failures_from_many_addresses_block_the_account() {
        let attempts = Attempts::default();
        for i in 0..ACCOUNT_LIMIT {
            attempts.record_failure("a@example.org", &format!("10.0.{}.{}", i / 250, i % 250));
        }
        assert!(
            attempts.blocked("a@example.org", "9.9.9.9"),
            "ADR 0106 item 4: 100 failures from all addresses block the account."
        );
        assert!(!attempts.blocked("b@example.org", "9.9.9.9"));
    }
}
