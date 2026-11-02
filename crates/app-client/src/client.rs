//! The client of the `/app/v1/` routes (ADR 0110 items 10, 12, 28 to 36).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use nostr::prelude::{PublicKey, Tag};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::clock::parse_http_date;
use crate::error::Error;
use crate::pairing::{PairingText, live_value_origin, studio_origin};
use crate::signer::AppSigner;
use crate::wire::{
    ErrorBody, GrantKeyRequest, LiveValueRequest, PairRequest, Show, StatusRequest, WhoAmI,
    Wrapper, WriteResult,
};

/// The kind of a signed request event (NIP-98).
const KIND_HTTP_AUTH: u16 = 27_235;
/// The clock difference above which the client gives a warning, in seconds.
pub const CLOCK_WARNING_SECONDS: i64 = 30;
/// The clock difference above which the client gives `ClockWrong`, in seconds.
pub const CLOCK_LIMIT_SECONDS: i64 = 5 * 60;
/// A difference this small between two readings of the clock of Studio shows
/// that the clock is not the cause of a refusal.
const CLOCK_SAME_SECONDS: i64 = 5;
/// The most bytes of an answer.
const MAX_ANSWER: u64 = 1024 * 1024;

/// The status of a live item that an app sets (ADR 0054 item 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowStatus {
    /// Go live.
    Live,
    /// End the show.
    Ended,
}

impl ShowStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Ended => "ended",
        }
    }
}

type ClockWarning = Box<dyn Fn(i64) + Send + Sync>;

#[derive(Default)]
struct State {
    /// The seconds that the client adds to its clock (ADR 0110 item 36).
    offset: i64,
    /// The grant vault key of each grant, in memory only (item 29).
    keys: HashMap<i64, Zeroizing<String>>,
}

/// A connection from an app to one Studio.
///
/// The client signs each request with a new event. It corrects a clock
/// difference of 5 minutes or less. It keeps a grant vault key in memory only,
/// and clears it when the grant ends, or when a write gives 404 or 423.
///
/// ```no_run
/// use studio_app_client::{Client, FileSigner, PairingText};
///
/// let signer = FileSigner::load_or_create("/var/lib/app/keys/studio.key")?;
/// let text = PairingText::parse("https://studio.example.com#pair=K7M2-Q9XD")?;
/// // Step 1: the pairing. The app gives its live value origin.
/// let client = Client::pair(&text, signer, "wss://live.example.com")?;
/// // Step 2: examine the connection before a show.
/// let me = client.whoami()?;
/// println!("connected as {}", me.name);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Client {
    origin: String,
    signer: Arc<dyn AppSigner>,
    agent: ureq::Agent,
    state: Mutex<State>,
    warning: Option<ClockWarning>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// A client for a Studio that the app paired with before. The app saved
    /// the origin at the first pairing.
    ///
    /// # Errors
    ///
    /// Gives [`Error::OriginInvalid`] when the origin is not `https` and not a
    /// loopback address (ADR 0110 item 35).
    pub fn connect(origin: &str, signer: impl AppSigner + 'static) -> Result<Self, Error> {
        Self::connect_shared(origin, Arc::new(signer))
    }

    /// The same as [`Client::connect`], with a signer that the app shares.
    ///
    /// # Errors
    ///
    /// The same as [`Client::connect`].
    pub fn connect_shared(origin: &str, signer: Arc<dyn AppSigner>) -> Result<Self, Error> {
        let origin = studio_origin(origin)?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        Ok(Self {
            origin,
            signer,
            agent,
            state: Mutex::new(State::default()),
            warning: None,
        })
    }

    /// Pair this app with Studio: send `POST /app/v1/pair`, signed with the
    /// key of `signer`. `live_value_origin` is the scheme, host and port where
    /// the listeners of the app get the live value, for example
    /// `wss://live.example.com` (ADR 0110 item 24).
    ///
    /// # Errors
    ///
    /// Gives [`Error::CodeInvalid`], [`Error::CodeUsed`], [`Error::KeyInUse`],
    /// [`Error::ClockWrong`], [`Error::TooManyAttempts`] or [`Error::Network`]
    /// from Studio, and [`Error::OriginInvalid`] for a bad origin.
    pub fn pair(
        pairing: &PairingText,
        signer: impl AppSigner + 'static,
        live_value_origin_text: &str,
    ) -> Result<Self, Error> {
        Self::pair_with(pairing, signer, live_value_origin_text, None)
    }

    /// The same as [`Client::pair`], with a callback for the clock warning.
    ///
    /// # Errors
    ///
    /// The same as [`Client::pair`].
    pub fn pair_with(
        pairing: &PairingText,
        signer: impl AppSigner + 'static,
        live_value_origin_text: &str,
        warning: Option<ClockWarning>,
    ) -> Result<Self, Error> {
        let live = live_value_origin(live_value_origin_text)?;
        let mut client = Self::connect(pairing.origin(), signer)?;
        client.warning = warning;
        let body = Zeroizing::new(
            serde_json::to_vec(&PairRequest {
                code: pairing.code(),
                live_value_origin: &live,
            })
            .map_err(|_| network("the request cannot be made"))?,
        );
        client.send("POST", "/app/v1/pair", Some(&body))?;
        Ok(client)
    }

    /// Set the function that the client calls when the clock of this computer
    /// differs from the clock of Studio by more than 30 seconds (ADR 0110 item
    /// 36). The argument is the difference in seconds. The app tells the user
    /// to correct the clock.
    #[must_use]
    pub fn with_clock_warning(mut self, warning: impl Fn(i64) + Send + Sync + 'static) -> Self {
        self.warning = Some(Box::new(warning));
        self
    }

    /// The origin of Studio. The app can save it for later pairings.
    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// The seconds that the client now adds to its clock.
    #[must_use]
    pub fn clock_offset(&self) -> i64 {
        self.lock().offset
    }

    /// `GET /app/v1/whoami`: the app, its state and its live value origin.
    ///
    /// # Errors
    ///
    /// Gives an [`Error`] as the module documentation lists.
    pub fn whoami(&self) -> Result<WhoAmI, Error> {
        let text = self.send("GET", "/app/v1/whoami", None)?;
        decode(&text)
    }

    /// `GET /app/v1/shows`: the show grants of the app. The client clears the
    /// grant vault key of each grant that this list no longer has.
    ///
    /// # Errors
    ///
    /// Gives an [`Error`] as the module documentation lists.
    pub fn shows(&self) -> Result<Vec<Show>, Error> {
        let text = self.send("GET", "/app/v1/shows", None)?;
        let shows: Vec<Show> = decode(&text)?;
        self.lock()
            .keys
            .retain(|id, _| shows.iter().any(|show| show.grant_id == *id));
        Ok(shows)
    }

    /// Go live or end a show: `POST /app/v1/shows/{grant_id}/status`. This
    /// writes the feed. The first write of a grant gets the wrapper first.
    /// After `Ended`, the client clears the grant vault key.
    ///
    /// # Errors
    ///
    /// Gives [`Error::NotGranted`] or [`Error::VaultLocked`], and then clears
    /// the key. It gives other errors as the module documentation lists.
    pub fn set_status(&self, grant_id: i64, status: ShowStatus) -> Result<WriteResult, Error> {
        let result = self.write(grant_id, "POST", "status", |key| {
            to_body(&StatusRequest {
                status: status.as_str(),
                grant_key: key,
            })
        })?;
        if status == ShowStatus::Ended {
            self.forget_grant_key(grant_id);
        }
        Ok(result)
    }

    /// Set the live value of a show: `PUT /app/v1/shows/{grant_id}/live-value`.
    /// The `uri` must be on the live value origin of the grant. This writes
    /// the feed.
    ///
    /// # Errors
    ///
    /// Gives [`Error::LiveValueInvalid`] when Studio refuses the value. It
    /// gives other errors as [`Client::set_status`] does.
    pub fn set_live_value(
        &self,
        grant_id: i64,
        uri: &str,
        protocol: &str,
    ) -> Result<WriteResult, Error> {
        self.write(grant_id, "PUT", "live-value", |key| {
            to_body(&LiveValueRequest {
                uri,
                protocol,
                grant_key: key,
            })
        })
    }

    /// Remove the live value of a show:
    /// `DELETE /app/v1/shows/{grant_id}/live-value`. This writes the feed.
    ///
    /// # Errors
    ///
    /// The same as [`Client::set_status`].
    pub fn remove_live_value(&self, grant_id: i64) -> Result<WriteResult, Error> {
        self.write(grant_id, "DELETE", "live-value", |key| {
            to_body(&GrantKeyRequest { grant_key: key })
        })
    }

    /// Clear the grant vault key of a grant from memory. Call it when the app
    /// knows that the grant ended.
    pub fn forget_grant_key(&self, grant_id: i64) {
        self.lock().keys.remove(&grant_id);
    }

    /// Whether the client holds the grant vault key of a grant now.
    #[must_use]
    pub fn holds_grant_key(&self, grant_id: i64) -> bool {
        self.lock().keys.contains_key(&grant_id)
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The grant vault key of a grant. The first call of a grant gets the
    /// wrapper and decrypts it with the key of the app (ADR 0110 item 23).
    fn grant_key(&self, grant_id: i64) -> Result<Zeroizing<String>, Error> {
        if let Some(key) = self.lock().keys.get(&grant_id) {
            return Ok(key.clone());
        }
        let path = format!("/app/v1/shows/{grant_id}/wrapper");
        let text = self.send("GET", &path, None)?;
        let wrapper: Wrapper = decode(&text)?;
        let sender = PublicKey::from_hex(&wrapper.sender)
            .map_err(|_| network("the wrapper has no valid sender"))?;
        let key = self
            .signer
            .nip44_decrypt(&sender, &wrapper.ciphertext)
            .map_err(|_| Error::VaultLocked)?;
        self.lock().keys.insert(grant_id, key.clone());
        Ok(key)
    }

    fn write(
        &self,
        grant_id: i64,
        method: &str,
        route: &str,
        body: impl FnOnce(&str) -> Result<Vec<u8>, Error>,
    ) -> Result<WriteResult, Error> {
        let key = self.grant_key(grant_id)?;
        let body = Zeroizing::new(body(&key)?);
        let path = format!("/app/v1/shows/{grant_id}/{route}");
        match self.send(method, &path, Some(&body)) {
            Ok(text) => decode(&text),
            Err(error) => {
                if matches!(error, Error::NotGranted | Error::VaultLocked) {
                    self.forget_grant_key(grant_id);
                }
                Err(error)
            }
        }
    }

    /// Send one request, signed with a new event. Corrects the clock as ADR
    /// 0110 item 36 gives, with one repeat at most.
    fn send(&self, method: &str, path: &str, body: Option<&[u8]>) -> Result<String, Error> {
        let url = format!("{}{path}", self.origin);
        let mut repeated = false;
        loop {
            let created_at =
                u64::try_from(local_now().saturating_add(self.lock().offset)).unwrap_or(0);
            let header = self.authorization(&url, method, body, created_at)?;
            let answer = self.call(method, &url, &header, body)?;
            if (200..300).contains(&answer.status) {
                return Ok(answer.body);
            }
            let code = error_code(&answer.body);
            if answer.status != 401
                || matches!(code.as_str(), "app_code_invalid" | "app_key_required")
            {
                return Err(map_error(answer.status, &code));
            }
            // 401 for a signature: the clock can be the cause.
            let Some(server) = answer.date.as_deref().and_then(parse_http_date) else {
                return Err(Error::SignatureInvalid);
            };
            let difference = server - local_now();
            if difference.abs() > CLOCK_LIMIT_SECONDS {
                return Err(Error::ClockWrong(difference));
            }
            // A forged `Date` on the second 401 causes no third request.
            if repeated || (difference - self.lock().offset).abs() <= CLOCK_SAME_SECONDS {
                return Err(Error::SignatureInvalid);
            }
            if difference.abs() > CLOCK_WARNING_SECONDS
                && let Some(warning) = &self.warning
            {
                warning(difference);
            }
            self.lock().offset = difference;
            repeated = true;
        }
    }

    /// The header value `Nostr <base64 event>` for one request.
    fn authorization(
        &self,
        url: &str,
        method: &str,
        body: Option<&[u8]>,
        created_at: u64,
    ) -> Result<Zeroizing<String>, Error> {
        let tag = |name: &str, value: &str| {
            Tag::parse([name, value]).map_err(|_| Error::Signer("a tag is not valid".to_owned()))
        };
        let mut tags = vec![tag("u", url)?, tag("method", method)?];
        if let Some(body) = body.filter(|b| !b.is_empty()) {
            tags.push(tag("payload", &hex_sha256(body))?);
        }
        // Two requests with the same tags in the same second would have the
        // same event ID, and Studio refuses a repeated ID (ADR 0110 item 13).
        // A random `nonce` tag makes each event new.
        let mut nonce = [0_u8; 16];
        getrandom::fill(&mut nonce)
            .map_err(|_| Error::Signer("the random source is not available".to_owned()))?;
        tags.push(tag("nonce", &hex(&nonce))?);
        let event = self.signer.sign(KIND_HTTP_AUTH, tags, created_at)?;
        let json = Zeroizing::new(
            serde_json::to_vec(&event)
                .map_err(|_| Error::Signer("the event is not valid".to_owned()))?,
        );
        let encoded = base64::engine::general_purpose::STANDARD.encode(&*json);
        Ok(Zeroizing::new(format!("Nostr {encoded}")))
    }

    fn call(
        &self,
        method: &str,
        url: &str,
        authorization: &str,
        body: Option<&[u8]>,
    ) -> Result<Answer, Error> {
        let json = "application/json";
        let result = match (method, body) {
            ("GET", _) => self
                .agent
                .get(url)
                .header("Authorization", authorization)
                .call(),
            ("POST", body) => self
                .agent
                .post(url)
                .header("Authorization", authorization)
                .header("Content-Type", json)
                .send(body.unwrap_or_default()),
            ("PUT", body) => self
                .agent
                .put(url)
                .header("Authorization", authorization)
                .header("Content-Type", json)
                .send(body.unwrap_or_default()),
            ("DELETE", body) => self
                .agent
                .delete(url)
                .force_send_body()
                .header("Authorization", authorization)
                .header("Content-Type", json)
                .send(body.unwrap_or_default()),
            _ => return Err(network("the method is not known")),
        };
        let mut response = result.map_err(|_| network("the connection failed"))?;
        let status = response.status().as_u16();
        let date = response
            .headers()
            .get("date")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_ANSWER)
            .read_to_string()
            .map_err(|_| network("the answer cannot be read"))?;
        Ok(Answer { status, date, body })
    }
}

struct Answer {
    status: u16,
    date: Option<String>,
    body: String,
}

fn network(why: &str) -> Error {
    Error::Network(why.to_owned())
}

fn local_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

fn to_body<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| network("the request cannot be made"))
}

fn decode<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    serde_json::from_str(text).map_err(|_| network("the answer has an unknown form"))
}

fn error_code(body: &str) -> String {
    serde_json::from_str::<ErrorBody>(body)
        .map(|b| b.error.code)
        .unwrap_or_default()
}

/// Map an error answer to an error type (ADR 0110 item 9).
fn map_error(status: u16, code: &str) -> Error {
    match (status, code) {
        (401, "app_code_invalid") => Error::CodeInvalid,
        (409, "app_code_used") => Error::CodeUsed,
        (401, "app_key_required") => Error::KeyRequired,
        (401, _) => Error::SignatureInvalid,
        (409, "app_key_in_use") => Error::KeyInUse,
        (400, "live_value_invalid") => Error::LiveValueInvalid,
        (404, _) => Error::NotGranted,
        (423, _) => Error::VaultLocked,
        (429, _) => Error::TooManyAttempts,
        (500..=599, _) => Error::Network(format!("Studio gave the status {status}")),
        _ => Error::Other {
            status,
            code: code.chars().take(64).collect(),
        },
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, b| {
            let _ = write!(text, "{b:02x}");
            text
        })
}
