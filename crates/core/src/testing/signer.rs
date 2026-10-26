//! A NIP-46 remote signer for tests. It runs in the test process.
//!
//! The signer is a thread with one key. It answers the NIP-46 requests on a
//! [`TestRelay`](super::relay::TestRelay). It answers a `nostrconnect://`
//! address that a test gives, and it gives a `bunker://` address.

use std::collections::HashSet;
use std::io;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nostr::prelude::{Event, Keys, Kind, PublicKey, SecretKey, Tag, Timestamp, UnsignedEvent};
use serde_json::{Value, json};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error as WsError, Message, WebSocket};

use crate::nip46::{decrypt_text, encrypt_text};
use crate::nostr_events::sign_with_keys;

/// How long the thread waits for a message before it looks at the commands.
const POLL: Duration = Duration::from_millis(50);

/// How the signer answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behavior {
    /// The signer answers each request in the correct way.
    #[default]
    Normal,
    /// The signer gives no answer.
    Silent,
    /// The signer signs an event with a different key.
    SignWithOtherKey,
    /// The signer gives an event with a signature that is not correct.
    BadSignature,
    /// The signer answers with a different key for the messages.
    AnswerWithOtherKey,
    /// The signer gives a wrong text for `nip44_decrypt`.
    WrongDecrypt,
}

#[derive(Debug, Default)]
struct State {
    behavior: Behavior,
    methods: Vec<String>,
    connects: Vec<String>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    stop: AtomicBool,
}

/// A remote signer on a test relay. It stops when it is dropped.
#[derive(Debug)]
pub struct TestSigner {
    keys: Keys,
    relay: String,
    shared: Arc<Shared>,
}

fn keys_from(byte: u8) -> Keys {
    Keys::new(SecretKey::from_slice(&[byte; 32]).expect("a secret key"))
}

fn now() -> Timestamp {
    Timestamp::from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    )
}

impl TestSigner {
    /// Start a signer on the relay at `relay_url`.
    ///
    /// # Panics
    ///
    /// Panics when the signer cannot connect to the relay.
    #[must_use]
    pub fn start(relay_url: &str) -> Self {
        Self::start_with_key(relay_url, 0x31)
    }

    /// Start a signer whose secret key is `byte` repeated 32 times. A test
    /// uses it for a signer with a different key.
    ///
    /// # Panics
    ///
    /// Panics when the signer cannot connect to the relay.
    #[must_use]
    pub fn start_with_key(relay_url: &str, byte: u8) -> Self {
        let keys = keys_from(byte);
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            stop: AtomicBool::new(false),
        });
        let (mut socket, _) = tungstenite::connect(relay_url).expect("a relay connection");
        if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
            stream.set_read_timeout(Some(POLL)).expect("a read timeout");
        }
        let filter = json!({
            "kinds": [24133],
            "#p": [keys.public_key().to_hex()],
            "since": now().as_secs().saturating_sub(5),
        });
        socket
            .send(Message::text(json!(["REQ", "signer", filter]).to_string()))
            .expect("a subscription");
        let thread_shared = Arc::clone(&shared);
        let thread_keys = keys.clone();
        thread::spawn(move || run(&thread_shared, &thread_keys, socket));
        Self {
            keys,
            relay: relay_url.to_owned(),
            shared,
        }
    }

    /// The public key of the signer. It is the key of the user, too.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        self.keys.public_key()
    }

    /// A `bunker://` address for this signer and its relay.
    #[must_use]
    pub fn bunker_uri(&self) -> String {
        format!(
            "bunker://{}?relay={}&secret=test-secret",
            self.keys.public_key().to_hex(),
            self.relay
        )
    }

    /// Answer a `nostrconnect://` address: the signer sends the `connect`
    /// response with the secret of the address.
    pub fn connect(&self, uri: &str) {
        self.with(|state| state.connects.push(uri.to_owned()));
    }

    /// Change how the signer answers.
    pub fn set_behavior(&self, behavior: Behavior) {
        self.with(|state| state.behavior = behavior);
    }

    /// The methods of the requests that the signer got, in sequence.
    #[must_use]
    pub fn methods(&self) -> Vec<String> {
        self.with(|state| state.methods.clone())
    }

    fn with<T>(&self, action: impl FnOnce(&mut State) -> T) -> T {
        action(&mut self.shared.state.lock().expect("the state"))
    }
}

impl Drop for TestSigner {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
    }
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

fn run(shared: &Shared, keys: &Keys, mut socket: Socket) {
    let other = keys_from(0x32);
    let mut seen: HashSet<String> = HashSet::new();
    while !shared.stop.load(Ordering::SeqCst) {
        let connects = std::mem::take(&mut shared.state.lock().expect("the state").connects);
        for uri in connects {
            answer_connect(shared, keys, &other, &mut socket, &uri);
        }
        match socket.read() {
            Ok(Message::Text(text)) => {
                let Ok(Value::Array(message)) = serde_json::from_str::<Value>(text.as_str()) else {
                    continue;
                };
                if message.first().and_then(Value::as_str) != Some("EVENT") {
                    continue;
                }
                let event = message
                    .get(2)
                    .and_then(|v| serde_json::from_value::<Event>(v.clone()).ok());
                if let Some(event) = event
                    && event.verify().is_ok()
                    && seen.insert(event.id.to_hex())
                {
                    answer_request(shared, keys, &other, &mut socket, &event);
                }
            }
            Ok(Message::Close(_)) | Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => {
                break;
            }
            Err(WsError::Io(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

/// The keys that sign the messages for the current behavior.
fn responder<'a>(shared: &Shared, keys: &'a Keys, other: &'a Keys) -> &'a Keys {
    if shared.state.lock().expect("the state").behavior == Behavior::AnswerWithOtherKey {
        other
    } else {
        keys
    }
}

fn send_answer(
    socket: &mut Socket,
    responder: &Keys,
    client: &PublicKey,
    id: &str,
    result: Option<&str>,
    error: Option<&str>,
) {
    let body = json!({ "id": id, "result": result, "error": error }).to_string();
    let Ok(content) = encrypt_text(responder.secret_key(), client, &body) else {
        return;
    };
    let unsigned = UnsignedEvent::new(
        responder.public_key(),
        now(),
        Kind::NostrConnect,
        vec![Tag::public_key(*client)],
        content,
    );
    if let Ok(event) = sign_with_keys(unsigned, responder) {
        let _ = socket.send(Message::text(json!(["EVENT", event]).to_string()));
    }
}

fn answer_connect(shared: &Shared, keys: &Keys, other: &Keys, socket: &mut Socket, uri: &str) {
    // The address has the key of the client, then the query.
    let Some((key, query)) = uri
        .strip_prefix("nostrconnect://")
        .and_then(|rest| rest.split_once('?'))
    else {
        return;
    };
    let Ok(public_key) = PublicKey::from_hex(key) else {
        return;
    };
    let Some(secret) = query.split('&').find_map(|p| p.strip_prefix("secret=")) else {
        return;
    };
    if shared.state.lock().expect("the state").behavior == Behavior::Silent {
        return;
    }
    let responder = responder(shared, keys, other);
    send_answer(
        socket,
        responder,
        &public_key,
        "connect",
        Some(secret),
        None,
    );
}

fn answer_request(shared: &Shared, keys: &Keys, other: &Keys, socket: &mut Socket, event: &Event) {
    let Ok(text) = decrypt_text(keys.secret_key(), &event.pubkey, &event.content) else {
        return;
    };
    let Ok(request) = serde_json::from_str::<Value>(&text) else {
        return;
    };
    let id = request["id"].as_str().unwrap_or_default().to_owned();
    let method = request["method"].as_str().unwrap_or_default().to_owned();
    let params: Vec<String> = request["params"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let behavior = {
        let mut state = shared.state.lock().expect("the state");
        state.methods.push(method.clone());
        state.behavior
    };
    if behavior == Behavior::Silent {
        return;
    }
    let responder = responder(shared, keys, other);
    let outcome = handle(keys, other, behavior, &method, &params);
    match outcome {
        Ok(result) => send_answer(socket, responder, &event.pubkey, &id, Some(&result), None),
        Err(error) => send_answer(socket, responder, &event.pubkey, &id, None, Some(&error)),
    }
}

fn handle(
    keys: &Keys,
    other: &Keys,
    behavior: Behavior,
    method: &str,
    params: &[String],
) -> Result<String, String> {
    run_method(keys, other, behavior, method, params).map_err(str::to_owned)
}

fn run_method(
    keys: &Keys,
    other: &Keys,
    behavior: Behavior,
    method: &str,
    params: &[String],
) -> Result<String, &'static str> {
    let param = |index: usize| params.get(index).cloned().ok_or("missing parameter");
    let peer = |index: usize| {
        PublicKey::from_hex(&param(index)?).map_err(|_| "the public key is not correct")
    };
    match method {
        "connect" => Ok("ack".to_owned()),
        "get_public_key" => Ok(keys.public_key().to_hex()),
        "ping" => Ok("pong".to_owned()),
        "sign_event" => {
            let unsigned =
                UnsignedEvent::from_json(param(0)?).map_err(|_| "the event is not correct")?;
            let signer = if behavior == Behavior::SignWithOtherKey {
                other
            } else {
                keys
            };
            let rebuilt = UnsignedEvent::new(
                signer.public_key(),
                unsigned.created_at,
                unsigned.kind,
                unsigned.tags,
                unsigned.content,
            );
            let event = sign_with_keys(rebuilt, signer).map_err(|_| "the signing failed")?;
            let mut value = serde_json::to_value(&event).map_err(|_| "the event failed")?;
            if behavior == Behavior::BadSignature {
                value["content"] = json!("changed after the signature");
            }
            Ok(value.to_string())
        }
        "nip44_encrypt" => encrypt_text(keys.secret_key(), &peer(0)?, &param(1)?)
            .map_err(|_| "the encryption failed"),
        "nip44_decrypt" if behavior == Behavior::WrongDecrypt => Ok("wrong text".to_owned()),
        "nip44_decrypt" => decrypt_text(keys.secret_key(), &peer(0)?, &param(1)?)
            .map(|text| text.to_string())
            .map_err(|_| "the decryption failed"),
        _ => Err("the method is not known"),
    }
}
