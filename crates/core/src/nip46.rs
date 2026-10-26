//! A NIP-46 client: a remote signer such as Amber (ADR 0056 items 12 to 16).
//!
//! The client has its own keys for each connection. It talks to the remote
//! signer through relays with NIP-44 messages of the kind 24133. The relay
//! client is the blocking client of the core. Each function blocks until
//! the answer comes or the deadline passes, so a server runs it on a
//! blocking thread.

use std::fmt::{self, Write as _};
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nostr::nips::nip44::{self, Nonce};
use nostr::nips::nip46::{
    NostrConnectMessage, NostrConnectMethod, NostrConnectRequest, NostrConnectUri,
};
use nostr::prelude::{
    Event, Filter, Keys, Kind, PublicKey, SecretKey, Tag, Timestamp, UnsignedEvent,
};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::connector::relay;
use crate::connector::{ConnectorError, ConnectorErrorKind};
use crate::net::NetPolicy;
use crate::nostr_events::sign_with_keys;

/// The name that the signer app shows for the connection.
pub const APP_NAME: &str = "Music Index Studio";

/// The permissions that Studio asks for: the sign-in event, the nsite
/// manifest and the Blossom authorization (ADR 0056 item 13), and NIP-44.
pub const PERMISSIONS: &str =
    "sign_event:27235,sign_event:35128,sign_event:24242,nip44_encrypt,nip44_decrypt";

/// The time that a request to the signer can take.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// A connection uses at most this many relays.
pub const MAX_RELAYS: usize = 4;

/// A response can be this many seconds older than the request.
const CLOCK_MARGIN: u64 = 60;

/// An error of the NIP-46 client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nip46Error {
    /// The address is not a `bunker://` address, or it has no usable relay.
    InvalidAddress,
    /// The network policy refused each relay. The text names the host.
    NotPublic(String),
    /// The signer gave no answer before the deadline.
    Timeout,
    /// No relay is reachable.
    Unavailable,
    /// The signer refused the request. The text is its message.
    Rejected(String),
    /// The answer of the signer is not correct.
    Invalid(&'static str),
}

impl fmt::Display for Nip46Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress => f.write_str("the signer address is not correct"),
            Self::NotPublic(message) => f.write_str(message),
            Self::Timeout => f.write_str("the signer gave no answer in time"),
            Self::Unavailable => f.write_str("Studio cannot reach the relay of the signer"),
            Self::Rejected(message) => write!(f, "the signer refused the request: {message}"),
            Self::Invalid(message) => write!(f, "the signer gave a wrong answer: {message}"),
        }
    }
}

impl std::error::Error for Nip46Error {}

impl From<ConnectorError> for Nip46Error {
    fn from(error: ConnectorError) -> Self {
        match error.kind() {
            ConnectorErrorKind::NotPublic => Self::NotPublic(error.to_string()),
            ConnectorErrorKind::InvalidConfig => Self::InvalidAddress,
            _ => Self::Unavailable,
        }
    }
}

/// A NIP-44 encryption with a new random nonce.
pub(crate) fn encrypt_text(
    secret: &SecretKey,
    peer: &PublicKey,
    text: &str,
) -> Result<String, Nip46Error> {
    let mut nonce = [0_u8; 32];
    getrandom::fill(&mut nonce).map_err(|_| Nip46Error::Invalid("no random source"))?;
    nip44::encrypt_with_nonce(secret, peer, text, Nonce::V2(nonce))
        .map_err(|_| Nip46Error::Invalid("the text cannot be encrypted"))
}

/// A NIP-44 decryption.
pub(crate) fn decrypt_text(
    secret: &SecretKey,
    peer: &PublicKey,
    payload: &str,
) -> Result<Zeroizing<String>, Nip46Error> {
    nip44::decrypt(secret, peer, payload)
        .map(Zeroizing::new)
        .map_err(|_| Nip46Error::Invalid("the message cannot be decrypted"))
}

fn random_hex(bytes: usize) -> Result<String, Nip46Error> {
    let mut buffer = vec![0_u8; bytes];
    getrandom::fill(&mut buffer).map_err(|_| Nip46Error::Invalid("no random source"))?;
    let mut text = String::new();
    for byte in &buffer {
        let _ = write!(text, "{byte:02x}");
    }
    Ok(text)
}

fn new_keys() -> Result<Keys, Nip46Error> {
    loop {
        let mut bytes = Zeroizing::new([0_u8; 32]);
        getrandom::fill(bytes.as_mut()).map_err(|_| Nip46Error::Invalid("no random source"))?;
        // A value outside the curve order is very rare. Another try is correct.
        if let Ok(secret) = SecretKey::from_slice(bytes.as_ref()) {
            return Ok(Keys::new(secret));
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Percent-encode text. The characters in `keep` stay as they are.
fn percent_encode(text: &str, keep: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric()
            || b"-._~".contains(&byte)
            || keep.as_bytes().contains(&byte)
        {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// A message of the signer: the ID of the request, the result and the error.
#[derive(Deserialize)]
struct Answer {
    id: String,
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// Decrypt the content of an event from the signer and read the answer.
fn open_answer(client: &Keys, event: &Event) -> Option<Answer> {
    let text = decrypt_text(client.secret_key(), &event.pubkey, &event.content).ok()?;
    serde_json::from_str::<Answer>(&text).ok()
}

/// The filter for the answers to a client key.
fn answer_filter(client: &PublicKey, signer: Option<&PublicKey>) -> Filter {
    let mut filter = Filter::new()
        .kind(Kind::NostrConnect)
        .pubkey(*client)
        .since(Timestamp::from(now_secs().saturating_sub(CLOCK_MARGIN)));
    if let Some(signer) = signer {
        filter = filter.author(*signer);
    }
    filter
}

/// A `nostrconnect://` connection that waits for the signer.
pub struct Pending {
    client: Keys,
    relays: Vec<String>,
    secret: String,
    policy: NetPolicy,
    cancel: Arc<AtomicBool>,
}

impl fmt::Debug for Pending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pending")
            .field("relays", &self.relays)
            .finish_non_exhaustive()
    }
}

impl Pending {
    /// A flag that ends [`Pending::wait`] early when a caller sets it.
    #[must_use]
    pub fn canceller(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// Wait for the `connect` response with the same secret.
    ///
    /// # Errors
    ///
    /// Gives [`Nip46Error::Timeout`] at the deadline or after a cancel.
    /// Gives an error for a relay that the policy refuses or that is not
    /// reachable.
    pub fn wait(self, deadline: Instant) -> Result<RemoteSigner, Nip46Error> {
        let filter = answer_filter(&self.client.public_key(), None);
        let accept = |event: &Event| {
            open_answer(&self.client, event).is_some_and(|answer| {
                answer.result.as_deref() == Some(self.secret.as_str())
                    && answer.error.as_deref().is_none_or(str::is_empty)
            })
        };
        let found = relay::exchange(
            &self.relays,
            &filter,
            None,
            &self.policy,
            deadline,
            &accept,
            &self.cancel,
        )?;
        let event = found.event.ok_or(Nip46Error::Timeout)?;
        Ok(RemoteSigner::new(
            self.client,
            self.relays,
            event.pubkey,
            self.policy,
        ))
    }
}

/// A remote signer that the client reached through relays.
///
/// The client keys clear their memory when the value goes out of scope.
pub struct RemoteSigner {
    client: Keys,
    relays: Vec<String>,
    signer: PublicKey,
    user: OnceLock<PublicKey>,
    policy: NetPolicy,
    timeout: Duration,
}

impl fmt::Debug for RemoteSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteSigner")
            .field("relays", &self.relays)
            .field("signer", &self.signer)
            .finish_non_exhaustive()
    }
}

impl RemoteSigner {
    fn new(client: Keys, relays: Vec<String>, signer: PublicKey, policy: NetPolicy) -> Self {
        Self {
            client,
            relays,
            signer,
            user: OnceLock::new(),
            policy,
            timeout: REQUEST_TIMEOUT,
        }
    }

    /// Make new client keys and a secret, and give the `nostrconnect://`
    /// address for the signer. The function opens no connection.
    ///
    /// # Errors
    ///
    /// Gives [`Nip46Error::NotPublic`] when the policy refuses each relay.
    /// Gives [`Nip46Error::InvalidAddress`] for a relay address that is not
    /// `ws://` or `wss://`, or for no relay.
    pub fn nostrconnect(
        relays: &[String],
        policy: &NetPolicy,
    ) -> Result<(Pending, String), Nip46Error> {
        if relays.is_empty() || relays.len() > MAX_RELAYS {
            return Err(Nip46Error::InvalidAddress);
        }
        relay::check_relays(relays, policy)?;
        let client = new_keys()?;
        let secret = random_hex(16)?;
        let mut uri = format!("nostrconnect://{}", client.public_key().to_hex());
        let mut separator = '?';
        for relay in relays {
            uri.push(separator);
            separator = '&';
            uri.push_str("relay=");
            uri.push_str(&percent_encode(relay, ":/"));
        }
        uri.push_str("&secret=");
        uri.push_str(&secret);
        uri.push_str("&name=");
        uri.push_str(&percent_encode(APP_NAME, ""));
        uri.push_str("&perms=");
        uri.push_str(&percent_encode(PERMISSIONS, ":,"));
        let pending = Pending {
            client,
            relays: relays.to_vec(),
            secret,
            policy: policy.clone(),
            cancel: Arc::new(AtomicBool::new(false)),
        };
        Ok((pending, uri))
    }

    /// Connect with a `bunker://` address.
    ///
    /// # Errors
    ///
    /// Gives [`Nip46Error::InvalidAddress`] for text that is not a
    /// `bunker://` address with a relay. Gives [`Nip46Error::NotPublic`]
    /// when the policy refuses each relay of the address, and
    /// [`Nip46Error::Timeout`] when the signer does not answer.
    pub fn bunker(uri: &str, policy: &NetPolicy, deadline: Instant) -> Result<Self, Nip46Error> {
        let Ok(NostrConnectUri::Bunker {
            remote_signer_public_key,
            relays,
            secret,
        }) = NostrConnectUri::parse(uri.trim())
        else {
            return Err(Nip46Error::InvalidAddress);
        };
        let relays: Vec<String> = relays.iter().map(ToString::to_string).collect();
        if relays.is_empty() || relays.len() > MAX_RELAYS {
            return Err(Nip46Error::InvalidAddress);
        }
        relay::check_relays(&relays, policy)?;
        let signer = Self::new(
            new_keys()?,
            relays,
            remote_signer_public_key,
            policy.clone(),
        );
        // The third parameter asks for the permissions (NIP-46).
        let params = vec![
            remote_signer_public_key.to_hex(),
            secret.unwrap_or_default(),
            PERMISSIONS.to_owned(),
        ];
        let result = signer.call(NostrConnectMethod::Connect, params, deadline)?;
        if result.is_empty() {
            return Err(Nip46Error::Invalid("the answer to connect is empty"));
        }
        Ok(signer)
    }

    /// Set the time for each request. The default is 60 seconds.
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a request and wait for its answer. Gives the result text.
    fn call(
        &self,
        method: NostrConnectMethod,
        params: Vec<String>,
        deadline: Instant,
    ) -> Result<Zeroizing<String>, Nip46Error> {
        let id = random_hex(8)?;
        let message = NostrConnectMessage::Request {
            id: id.clone(),
            method,
            params,
        };
        let text = Zeroizing::new(
            serde_json::to_string(&message).map_err(|_| Nip46Error::Invalid("no request"))?,
        );
        let content = encrypt_text(self.client.secret_key(), &self.signer, &text)?;
        let unsigned = UnsignedEvent::new(
            self.client.public_key(),
            Timestamp::from(now_secs()),
            Kind::NostrConnect,
            vec![Tag::public_key(self.signer)],
            content,
        );
        let request = sign_with_keys(unsigned, &self.client)?;
        let filter = answer_filter(&self.client.public_key(), Some(&self.signer));
        // An answer with a wait for the user (`auth_url`) is not the end.
        let accept = |event: &Event| {
            event.pubkey == self.signer
                && open_answer(&self.client, event)
                    .is_some_and(|a| a.id == id && a.result.as_deref() != Some("auth_url"))
        };
        let stop = AtomicBool::new(false);
        let found = relay::exchange(
            &self.relays,
            &filter,
            Some(&request),
            &self.policy,
            deadline,
            &accept,
            &stop,
        )?;
        let event = found.event.ok_or(Nip46Error::Timeout)?;
        let answer = open_answer(&self.client, &event)
            .ok_or(Nip46Error::Invalid("the answer cannot be read"))?;
        if let Some(error) = answer.error.filter(|e| !e.is_empty()) {
            return Err(Nip46Error::Rejected(error));
        }
        answer
            .result
            .map(Zeroizing::new)
            .ok_or(Nip46Error::Invalid("the answer has no result"))
    }

    fn request(&self, request: &NostrConnectRequest) -> Result<Zeroizing<String>, Nip46Error> {
        self.call(
            request.method(),
            request.params(),
            Instant::now() + self.timeout,
        )
    }

    /// The public key of the signer. This is the key of the user, and it
    /// can differ from the key that the signer uses for the messages.
    ///
    /// # Errors
    ///
    /// Gives an error when the signer does not answer or answers with a
    /// text that is not a public key.
    pub fn get_public_key(&self) -> Result<PublicKey, Nip46Error> {
        if let Some(key) = self.user.get() {
            return Ok(*key);
        }
        let result = self.request(&NostrConnectRequest::GetPublicKey)?;
        let key = PublicKey::from_hex(result.trim())
            .map_err(|_| Nip46Error::Invalid("the public key is not correct"))?;
        Ok(*self.user.get_or_init(|| key))
    }

    /// Ask the signer to sign an event. Studio refuses an event with an
    /// incorrect signature, and an event that is not the event it sent.
    ///
    /// # Errors
    ///
    /// Gives an error when the signer refuses, does not answer, or gives an
    /// event that is not correct.
    pub fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Nip46Error> {
        let user = self.get_public_key()?;
        if unsigned.pubkey != user {
            return Err(Nip46Error::Invalid("the event has a different public key"));
        }
        let expected = unsigned.clone();
        let result = self.request(&NostrConnectRequest::SignEvent(unsigned))?;
        let event: Event = serde_json::from_str(&result)
            .map_err(|_| Nip46Error::Invalid("the signed event cannot be read"))?;
        if event.verify().is_err() {
            return Err(Nip46Error::Invalid("the event signature is not correct"));
        }
        if event.pubkey != user {
            return Err(Nip46Error::Invalid("the event has a different public key"));
        }
        if event.kind != expected.kind
            || event.content != expected.content
            || event.tags != expected.tags
            || event.created_at != expected.created_at
        {
            return Err(Nip46Error::Invalid("the signer changed the event"));
        }
        Ok(event)
    }

    /// Ask the signer to encrypt a text to a public key with NIP-44.
    ///
    /// # Errors
    ///
    /// Gives an error when the signer refuses or does not answer.
    pub fn nip44_encrypt(&self, public_key: &PublicKey, text: &str) -> Result<String, Nip46Error> {
        let result = self.request(&NostrConnectRequest::Nip44Encrypt {
            public_key: *public_key,
            text: text.to_owned(),
        })?;
        Ok(result.to_string())
    }

    /// Ask the signer to decrypt a NIP-44 text from a public key. The
    /// plaintext clears its memory when it goes out of scope.
    ///
    /// # Errors
    ///
    /// Gives an error when the signer refuses or does not answer.
    pub fn nip44_decrypt(
        &self,
        public_key: &PublicKey,
        ciphertext: &str,
    ) -> Result<Zeroizing<String>, Nip46Error> {
        self.request(&NostrConnectRequest::Nip44Decrypt {
            public_key: *public_key,
            ciphertext: ciphertext.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::relay::TestRelay;
    use crate::testing::signer::{Behavior, TestSigner};
    use std::sync::atomic::Ordering;

    const SHORT: Duration = Duration::from_secs(10);

    fn connected(relay: &TestRelay, signer: &TestSigner) -> RemoteSigner {
        let (pending, uri) =
            RemoteSigner::nostrconnect(&[relay.url()], &NetPolicy::Open).expect("an address");
        assert!(uri.starts_with("nostrconnect://"));
        assert!(uri.contains("perms=sign_event:27235,"));
        signer.connect(&uri);
        pending.wait(Instant::now() + SHORT).expect("a signer")
    }

    fn unsigned(pubkey: PublicKey) -> UnsignedEvent {
        UnsignedEvent::new(
            pubkey,
            Timestamp::from(now_secs()),
            Kind::Custom(27_235),
            vec![Tag::parse(["challenge", "abc"]).expect("a tag")],
            "",
        )
    }

    #[test]
    fn a_signer_answers_a_nostrconnect_address() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = connected(&relay, &signer);
        let user = remote.get_public_key().expect("a public key");
        assert_eq!(user, signer.public_key());
        let event = remote.sign_event(unsigned(user)).expect("a signed event");
        assert!(event.verify().is_ok());
        assert_eq!(event.pubkey, user);
        let ciphertext = remote
            .nip44_encrypt(&user, "the vault key")
            .expect("a ciphertext");
        assert_ne!(ciphertext, "the vault key");
        let plain = remote
            .nip44_decrypt(&user, &ciphertext)
            .expect("a plaintext");
        assert_eq!(plain.as_str(), "the vault key");
    }

    #[test]
    fn a_signer_answers_a_bunker_address() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = RemoteSigner::bunker(
            &signer.bunker_uri(),
            &NetPolicy::Open,
            Instant::now() + SHORT,
        )
        .expect("a connection");
        assert_eq!(remote.get_public_key().expect("a key"), signer.public_key());
        assert!(signer.methods().contains(&"connect".to_owned()));
    }

    #[test]
    fn a_bad_bunker_address_is_refused() {
        let result = RemoteSigner::bunker("https://example.org", &NetPolicy::Open, Instant::now());
        assert_eq!(result.unwrap_err(), Nip46Error::InvalidAddress);
        let key = super::new_keys().expect("keys").public_key().to_hex();
        let result =
            RemoteSigner::bunker(&format!("bunker://{key}"), &NetPolicy::Open, Instant::now());
        assert_eq!(result.unwrap_err(), Nip46Error::InvalidAddress);
    }

    #[test]
    fn an_event_from_a_different_key_is_refused() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = connected(&relay, &signer);
        let user = remote.get_public_key().expect("a public key");
        signer.set_behavior(Behavior::SignWithOtherKey);
        let error = remote.sign_event(unsigned(user)).unwrap_err();
        assert!(matches!(error, Nip46Error::Invalid(_)), "{error:?}");
    }

    #[test]
    fn an_event_with_a_wrong_signature_is_refused() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = connected(&relay, &signer);
        let user = remote.get_public_key().expect("a public key");
        signer.set_behavior(Behavior::BadSignature);
        let error = remote.sign_event(unsigned(user)).unwrap_err();
        assert_eq!(
            error,
            Nip46Error::Invalid("the event signature is not correct")
        );
    }

    #[test]
    fn an_answer_from_a_different_public_key_is_refused() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = connected(&relay, &signer).with_request_timeout(Duration::from_millis(600));
        signer.set_behavior(Behavior::AnswerWithOtherKey);
        assert_eq!(remote.get_public_key().unwrap_err(), Nip46Error::Timeout);
    }

    #[test]
    fn a_silent_signer_gives_a_timeout() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let remote = connected(&relay, &signer).with_request_timeout(Duration::from_millis(600));
        signer.set_behavior(Behavior::Silent);
        assert_eq!(remote.get_public_key().unwrap_err(), Nip46Error::Timeout);
    }

    #[test]
    fn the_policy_refuses_a_loopback_relay() {
        let relay = TestRelay::start();
        let error = RemoteSigner::nostrconnect(&[relay.url()], &NetPolicy::public()).unwrap_err();
        assert!(matches!(error, Nip46Error::NotPublic(_)), "{error:?}");
        let key = super::new_keys().expect("keys").public_key().to_hex();
        let uri = format!("bunker://{key}?relay={}", relay.url());
        let error =
            RemoteSigner::bunker(&uri, &NetPolicy::public(), Instant::now() + SHORT).unwrap_err();
        assert!(matches!(error, Nip46Error::NotPublic(_)), "{error:?}");
    }

    #[test]
    fn a_connect_answer_with_a_wrong_secret_is_ignored() {
        let relay = TestRelay::start();
        let signer = TestSigner::start(&relay.url());
        let (pending, uri) =
            RemoteSigner::nostrconnect(&[relay.url()], &NetPolicy::Open).expect("an address");
        signer.connect(&uri.replace("secret=", "secret=00"));
        let result = pending.wait(Instant::now() + Duration::from_millis(600));
        assert_eq!(result.unwrap_err(), Nip46Error::Timeout);
    }

    #[test]
    fn a_cancel_ends_the_wait() {
        let relay = TestRelay::start();
        let (pending, _) =
            RemoteSigner::nostrconnect(&[relay.url()], &NetPolicy::Open).expect("an address");
        let cancel = pending.canceller();
        let started = Instant::now();
        let waiter = std::thread::spawn(move || pending.wait(Instant::now() + SHORT));
        std::thread::sleep(Duration::from_millis(300));
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(
            waiter.join().expect("a thread").unwrap_err(),
            Nip46Error::Timeout
        );
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
