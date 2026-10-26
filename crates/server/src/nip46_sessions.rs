//! The NIP-46 connections of the server (ADR 0056 items 12 to 16).
//!
//! The server keeps a pending connection or a connected signer in memory
//! only, under a random connection ID. A connection that nobody uses ends
//! after five minutes. After a sign-in, the signer belongs to the session
//! and goes away with it. The lock of this map is never held while the
//! server waits for a relay or a signer.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use studio_core::net::NetPolicy;
use studio_core::nip46::RemoteSigner;
use tokio::sync::Notify;

/// A connection that nobody uses ends after this time.
pub const EXPIRY: Duration = Duration::from_secs(300);

/// The most connections in memory. Each pending connection holds a
/// blocking thread.
const MAX_CONNECTIONS: usize = 128;

/// The most pending connections of one client address.
const MAX_PENDING_PER_ADDRESS: usize = 4;

/// What a connection is now.
enum Stage {
    /// The signer has not answered the `nostrconnect://` address.
    Pending {
        /// Ends the wait for the signer.
        cancel: Arc<AtomicBool>,
        /// The client address that started the connection.
        address: String,
    },
    /// The signer answered. `busy` is true while a sign-in uses it.
    Connected {
        signer: Arc<RemoteSigner>,
        pubkey: String,
        busy: bool,
    },
    /// The connection failed. The text is the error code of the route.
    Failed(&'static str),
}

struct Entry {
    stage: Stage,
    since: Instant,
}

/// A signer that belongs to a session.
struct Bound {
    account_id: i64,
    signer: Arc<RemoteSigner>,
}

#[derive(Default)]
struct Inner {
    connections: HashMap<String, Entry>,
    /// The signers of the sessions, by the hash of the session token.
    bound: HashMap<String, Bound>,
}

/// The public view of a connection.
#[derive(Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// `pending`, `connected` or `failed`.
    pub state: &'static str,
    /// The public key of the user, when the signer is connected.
    pub pubkey: Option<String>,
    /// The error code, when the connection failed.
    pub error: Option<&'static str>,
}

/// Why a sign-in cannot use a connection.
#[derive(Debug, PartialEq, Eq)]
pub enum Checkout {
    /// No such connection.
    Unknown,
    /// The signer has not connected.
    NotReady,
    /// Another sign-in uses the signer.
    Busy,
}

/// The connections and the settings for them.
pub struct Nip46Sessions {
    relays: Vec<String>,
    policy: NetPolicy,
    timeout: Duration,
    inner: Mutex<Inner>,
    changed: Notify,
}

impl std::fmt::Debug for Nip46Sessions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nip46Sessions")
            .field("relays", &self.relays)
            .finish_non_exhaustive()
    }
}

impl Nip46Sessions {
    /// Make an empty map. `relays` are the relays for a `nostrconnect://`
    /// address, `policy` is the network policy of the server and `timeout`
    /// is the time for each request to a signer.
    #[must_use]
    pub fn new(relays: Vec<String>, policy: NetPolicy, timeout: Duration) -> Self {
        Self {
            relays,
            policy,
            timeout,
            inner: Mutex::new(Inner::default()),
            changed: Notify::new(),
        }
    }

    /// The relays for a `nostrconnect://` address.
    #[must_use]
    pub fn relays(&self) -> &[String] {
        &self.relays
    }

    /// The network policy for the relays.
    #[must_use]
    pub fn policy(&self) -> &NetPolicy {
        &self.policy
    }

    /// The time for each request to a signer.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A poisoned lock only means that another thread failed. The map is
        // still usable.
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Remove the connections that nobody used for the expiry time. A
    /// pending connection also stops its wait.
    fn expire(inner: &mut Inner) {
        inner.connections.retain(|_, entry| {
            let alive = entry.since.elapsed() < EXPIRY;
            if !alive && let Stage::Pending { cancel, .. } = &entry.stage {
                cancel.store(true, Ordering::SeqCst);
            }
            alive
        });
    }

    fn insert(&self, stage: Stage, limit_address: Option<&str>) -> Option<String> {
        let id = crate::accounts::random_hex(16).ok()?;
        let mut inner = self.lock();
        Self::expire(&mut inner);
        if inner.connections.len() >= MAX_CONNECTIONS {
            return None;
        }
        if let Some(address) = limit_address {
            let pending = inner
                .connections
                .values()
                .filter(|e| matches!(&e.stage, Stage::Pending { address: a, .. } if a == address))
                .count();
            if pending >= MAX_PENDING_PER_ADDRESS {
                return None;
            }
        }
        inner.connections.insert(
            id.clone(),
            Entry {
                stage,
                since: Instant::now(),
            },
        );
        drop(inner);
        self.changed.notify_waiters();
        Some(id)
    }

    /// Keep a pending connection. Gives its ID, or `None` when the map is
    /// full.
    pub fn insert_pending(&self, cancel: Arc<AtomicBool>, address: &str) -> Option<String> {
        self.insert(
            Stage::Pending {
                cancel,
                address: address.to_owned(),
            },
            Some(address),
        )
    }

    /// Keep a signer that is connected already. Gives its ID, or `None`
    /// when the map is full.
    pub fn insert_connected(&self, signer: Arc<RemoteSigner>, pubkey: String) -> Option<String> {
        self.insert(
            Stage::Connected {
                signer,
                pubkey,
                busy: false,
            },
            None,
        )
    }

    /// Set the result of a pending connection. A connection that was
    /// stopped or that expired is gone, and the signer goes out of memory.
    pub fn finish_pending(
        &self,
        id: &str,
        result: Result<(Arc<RemoteSigner>, String), &'static str>,
    ) {
        {
            let mut inner = self.lock();
            if let Some(entry) = inner.connections.get_mut(id) {
                entry.stage = match result {
                    Ok((signer, pubkey)) => Stage::Connected {
                        signer,
                        pubkey,
                        busy: false,
                    },
                    Err(code) => Stage::Failed(code),
                };
                entry.since = Instant::now();
            }
        }
        self.changed.notify_waiters();
    }

    /// The state of a connection. Gives `None` for an unknown connection.
    pub fn snapshot(&self, id: &str) -> Option<Snapshot> {
        let mut inner = self.lock();
        Self::expire(&mut inner);
        let entry = inner.connections.get(id)?;
        Some(match &entry.stage {
            Stage::Pending { .. } => Snapshot {
                state: "pending",
                pubkey: None,
                error: None,
            },
            Stage::Connected { pubkey, .. } => Snapshot {
                state: "connected",
                pubkey: Some(pubkey.clone()),
                error: None,
            },
            Stage::Failed(code) => Snapshot {
                state: "failed",
                pubkey: None,
                error: Some(code),
            },
        })
    }

    /// A future that ends at the next change of any connection. Make it
    /// before the check of the state, so that no change is missed.
    pub fn changed(&self) -> tokio::sync::futures::Notified<'_> {
        self.changed.notified()
    }

    /// End a connection, and stop its wait. Gives true when it existed.
    pub fn stop(&self, id: &str) -> bool {
        let removed = self.lock().connections.remove(id);
        if let Some(Entry {
            stage: Stage::Pending { cancel, .. },
            ..
        }) = &removed
        {
            cancel.store(true, Ordering::SeqCst);
        }
        removed.is_some()
    }

    /// Take the connected signer for a sign-in. The signer stays busy
    /// until [`Nip46Sessions::release`] or [`Nip46Sessions::bind`].
    ///
    /// # Errors
    ///
    /// Gives the reason when the connection cannot be used.
    pub fn checkout(&self, id: &str) -> Result<(Arc<RemoteSigner>, String), Checkout> {
        let mut inner = self.lock();
        Self::expire(&mut inner);
        match inner.connections.get_mut(id).map(|entry| &mut entry.stage) {
            None => Err(Checkout::Unknown),
            Some(Stage::Connected { busy: true, .. }) => Err(Checkout::Busy),
            Some(Stage::Connected {
                signer,
                pubkey,
                busy,
            }) => {
                *busy = true;
                Ok((Arc::clone(signer), pubkey.clone()))
            }
            Some(_) => Err(Checkout::NotReady),
        }
    }

    /// Give back a signer after a sign-in that failed.
    pub fn release(&self, id: &str) {
        if let Some(Entry {
            stage: Stage::Connected { busy, .. },
            ..
        }) = self.lock().connections.get_mut(id)
        {
            *busy = false;
        }
    }

    /// Move the signer of a connection to a session. The connection ID
    /// stops working.
    pub fn bind(&self, id: &str, token_hash: &str, account_id: i64, signer: Arc<RemoteSigner>) {
        let mut inner = self.lock();
        inner.connections.remove(id);
        inner
            .bound
            .insert(token_hash.to_owned(), Bound { account_id, signer });
    }

    /// The signer of a session.
    pub fn signer_of(&self, token_hash: &str) -> Option<Arc<RemoteSigner>> {
        self.lock()
            .bound
            .get(token_hash)
            .map(|bound| Arc::clone(&bound.signer))
    }

    /// True when the session has a signer.
    pub fn has_signer(&self, token_hash: &str) -> bool {
        self.lock().bound.contains_key(token_hash)
    }

    /// Remove the signer of a session.
    pub fn remove_session(&self, token_hash: &str) {
        self.lock().bound.remove(token_hash);
    }

    /// Remove each signer of an account.
    pub fn remove_account(&self, account_id: i64) {
        self.lock()
            .bound
            .retain(|_, bound| bound.account_id != account_id);
    }

    /// Remove the expired connections, and the signers of the sessions
    /// that `exists` does not know.
    pub fn sweep(&self, exists: impl Fn(&str) -> bool) {
        let hashes: Vec<String> = self.lock().bound.keys().cloned().collect();
        // The check of a session reads the database, so the lock is free.
        let gone: Vec<String> = hashes.into_iter().filter(|hash| !exists(hash)).collect();
        let mut inner = self.lock();
        Self::expire(&mut inner);
        for hash in gone {
            inner.bound.remove(&hash);
        }
    }

    /// The number of connections and of bound signers, for tests.
    #[cfg(test)]
    pub fn counts(&self) -> (usize, usize) {
        let inner = self.lock();
        (inner.connections.len(), inner.bound.len())
    }

    /// Make a connection older, for tests.
    #[cfg(test)]
    pub fn age_all(&self, by: Duration) {
        for entry in self.lock().connections.values_mut() {
            entry.since = Instant::now().checked_sub(by).unwrap_or_else(Instant::now);
        }
    }
}
