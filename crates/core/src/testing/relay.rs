//! A small Nostr relay for tests. It runs in the test process.
//!
//! It has the NIP-01 messages `EVENT`, `REQ` and `CLOSE`, with the storage
//! rules of NIP-01. A `REQ` gets the stored events and then `EOSE`. The
//! subscription stays open until a `CLOSE`, and the relay sends each new
//! stored event that matches it.

use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use nostr::prelude::{Event, Filter, MatchEventOptions};
use serde_json::{Value, json};
use tungstenite::{Error as WsError, Message, WebSocket};

/// How long a connection waits for a message before it looks at the stop
/// flag.
const POLL: Duration = Duration::from_millis(50);

#[derive(Debug, Default)]
struct State {
    events: Vec<Event>,
    refuse: Option<String>,
    drop_events: bool,
    received: usize,
    /// Each event that the relay stored, in the sequence of the storage.
    /// A live subscription reads it.
    log: Vec<Event>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    stop: AtomicBool,
}

/// A relay on a loopback port. It stops when it is dropped.
#[derive(Debug)]
pub struct TestRelay {
    port: u16,
    shared: Arc<Shared>,
}

impl TestRelay {
    /// Start a relay.
    ///
    /// # Panics
    ///
    /// Panics when the relay cannot listen on a loopback port.
    #[must_use]
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            stop: AtomicBool::new(false),
        });
        let accept = Arc::clone(&shared);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if accept.stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let connection = Arc::clone(&accept);
                thread::spawn(move || serve(&connection, stream));
            }
        });
        Self { port, shared }
    }

    /// The address of the relay, for example `ws://127.0.0.1:41234`.
    #[must_use]
    pub fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// With a message, each `EVENT` gets `OK false` with that message.
    /// With `None`, the relay accepts events again.
    pub fn refuse_events(&self, message: Option<&str>) {
        self.with(|state| state.refuse = message.map(str::to_owned));
    }

    /// When true, each `EVENT` gets `OK true`, but the relay does not store
    /// it. This is a relay that did not get the newest event.
    pub fn drop_events(&self, drop: bool) {
        self.with(|state| state.drop_events = drop);
    }

    /// Store an event with no message. This is a write by a different
    /// client. The event must have a correct ID and signature.
    pub fn insert(&self, event: Event) {
        self.with(|state| {
            let kept = event.clone();
            if store(&mut state.events, event) == Stored::Kept {
                state.log.push(kept);
            }
        });
    }

    /// The stored events, the newest first.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.with(|state| newest_first(state.events.clone()))
    }

    /// The number of `EVENT` messages that the relay got.
    #[must_use]
    pub fn received(&self) -> usize {
        self.with(|state| state.received)
    }

    fn with<T>(&self, action: impl FnOnce(&mut State) -> T) -> T {
        action(&mut self.shared.state.lock().expect("the state"))
    }
}

impl Drop for TestRelay {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A connection wakes the accept loop so that it can end.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

/// The identity under which a relay keeps one event, or `None` for an event
/// that the relay always keeps.
fn slot(event: &Event) -> Option<(nostr::prelude::PublicKey, u16, String)> {
    let kind = event.kind.as_u16();
    match kind {
        0 | 3 | 10_000..=19_999 => Some((event.pubkey, kind, String::new())),
        30_000..=39_999 => {
            let d = event.tags.identifier().unwrap_or_default();
            Some((event.pubkey, kind, d))
        }
        _ => None,
    }
}

/// True when `new` replaces `old` (NIP-01): the newer `created_at` wins, and
/// with the same time the lowest ID wins.
fn replaces(new: &Event, old: &Event) -> bool {
    (new.created_at, old.id) > (old.created_at, new.id)
}

/// The result of a store.
#[derive(Debug, PartialEq, Eq)]
enum Stored {
    Kept,
    Duplicate,
    Older,
}

fn store(events: &mut Vec<Event>, event: Event) -> Stored {
    if events.iter().any(|e| e.id == event.id) {
        return Stored::Duplicate;
    }
    if let Some(wanted) = slot(&event)
        && let Some(index) = events
            .iter()
            .position(|e| slot(e).as_ref() == Some(&wanted))
    {
        if !replaces(&event, &events[index]) {
            return Stored::Older;
        }
        events.remove(index);
    }
    events.push(event);
    Stored::Kept
}

fn newest_first(mut events: Vec<Event>) -> Vec<Event> {
    events.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.id.cmp(&b.id)));
    events
}

type Socket = WebSocket<TcpStream>;

/// The open subscriptions of one connection.
#[derive(Default)]
struct Live {
    /// The subscription ID and the filters of each subscription.
    subscriptions: Vec<(String, Vec<Filter>)>,
    /// The number of events of the log that the connection has seen.
    cursor: usize,
}

/// Send each event stored after the last look to the subscriptions that
/// match it.
fn push_live(shared: &Shared, socket: &mut Socket, live: &mut Live) {
    if live.subscriptions.is_empty() {
        return;
    }
    let fresh: Vec<Event> = {
        let state = shared.state.lock().expect("the state");
        let fresh = state.log[live.cursor.min(state.log.len())..].to_vec();
        live.cursor = state.log.len();
        fresh
    };
    for event in fresh {
        for (id, filters) in &live.subscriptions {
            if filters
                .iter()
                .any(|f| f.match_event(&event, MatchEventOptions::new()))
            {
                reply(socket, &json!(["EVENT", id, event]));
            }
        }
    }
}

/// Serve one connection until the client or the relay ends it.
fn serve(shared: &Shared, stream: TcpStream) {
    let Ok(mut socket) = tungstenite::accept(stream) else {
        return;
    };
    let _ = socket.get_ref().set_read_timeout(Some(POLL));
    let mut live = Live::default();
    while !shared.stop.load(Ordering::SeqCst) {
        push_live(shared, &mut socket, &mut live);
        match socket.read() {
            Ok(Message::Text(text)) => handle(shared, &mut socket, &mut live, text.as_str()),
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

fn reply(socket: &mut Socket, value: &Value) {
    let _ = socket.send(Message::text(value.to_string()));
}

fn handle(shared: &Shared, socket: &mut Socket, live: &mut Live, text: &str) {
    let Ok(Value::Array(message)) = serde_json::from_str::<Value>(text) else {
        reply(socket, &json!(["NOTICE", "invalid: not a JSON array"]));
        return;
    };
    match message.first().and_then(Value::as_str) {
        Some("EVENT") => on_event(shared, socket, message.get(1)),
        Some("REQ") => {
            // The open subscriptions get their events before the new one.
            push_live(shared, socket, live);
            on_req(shared, socket, live, &message);
        }
        Some("CLOSE") => {
            let id = message.get(1).and_then(Value::as_str);
            live.subscriptions
                .retain(|(sub, _)| Some(sub.as_str()) != id);
        }
        _ => reply(socket, &json!(["NOTICE", "invalid: unknown message"])),
    }
}

fn on_event(shared: &Shared, socket: &mut Socket, value: Option<&Value>) {
    let Some(event) = value.and_then(|v| serde_json::from_value::<Event>(v.clone()).ok()) else {
        reply(socket, &json!(["NOTICE", "invalid: not an event"]));
        return;
    };
    let id = event.id.to_hex();
    let answer = {
        let mut state = shared.state.lock().expect("the state");
        state.received += 1;
        if let Some(message) = state.refuse.clone() {
            (false, message)
        } else if let Err(e) = event.verify() {
            (false, format!("invalid: {e}"))
        } else if state.drop_events {
            (true, String::new())
        } else {
            let kept = event.clone();
            match store(&mut state.events, event) {
                Stored::Kept => {
                    state.log.push(kept);
                    (true, String::new())
                }
                Stored::Duplicate => (true, "duplicate: the event is stored".to_owned()),
                Stored::Older => (false, "duplicate: a newer event exists".to_owned()),
            }
        }
    };
    reply(socket, &json!(["OK", id, answer.0, answer.1]));
}

fn on_req(shared: &Shared, socket: &mut Socket, live: &mut Live, message: &[Value]) {
    let Some(subscription) = message.get(1).and_then(Value::as_str) else {
        reply(socket, &json!(["NOTICE", "invalid: no subscription"]));
        return;
    };
    let filters: Option<Vec<Filter>> = message[2..]
        .iter()
        .map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    let Some(filters) = filters else {
        reply(
            socket,
            &json!(["CLOSED", subscription, "invalid: bad filter"]),
        );
        return;
    };
    let stored = {
        let state = shared.state.lock().expect("the state");
        // The subscription gets the later events from the log.
        live.cursor = live.cursor.max(state.log.len());
        state.events.clone()
    };
    live.subscriptions.retain(|(sub, _)| sub != subscription);
    live.subscriptions
        .push((subscription.to_owned(), filters.clone()));
    let mut found: Vec<Event> = Vec::new();
    for filter in &filters {
        let mut matches = newest_first(
            stored
                .iter()
                .filter(|e| filter.match_event(e, MatchEventOptions::new()))
                .cloned()
                .collect(),
        );
        if let Some(limit) = filter.limit {
            matches.truncate(limit);
        }
        for event in matches {
            if !found.iter().any(|e| e.id == event.id) {
                found.push(event);
            }
        }
    }
    for event in newest_first(found) {
        reply(socket, &json!(["EVENT", subscription, event]));
    }
    reply(socket, &json!(["EOSE", subscription]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr_events::sign_with_keys;
    use nostr::prelude::{Keys, Kind, SecretKey, Tag, Timestamp, UnsignedEvent};

    fn keys(byte: u8) -> Keys {
        Keys::new(SecretKey::from_slice(&[byte; 32]).expect("a secret key"))
    }

    fn event(keys: &Keys, time: u64, kind: u16, d: Option<&str>, content: &str) -> Event {
        let tags: Vec<Tag> = d.map(Tag::identifier).into_iter().collect();
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(time),
            Kind::Custom(kind),
            tags,
            content,
        );
        sign_with_keys(unsigned, keys).expect("a signed event")
    }

    fn connect(relay: &TestRelay) -> WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>> {
        tungstenite::connect(relay.url()).expect("a connection").0
    }

    fn send(socket: &mut WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>, value: &Value) {
        socket
            .send(Message::text(value.to_string()))
            .expect("a sent message");
    }

    fn next(socket: &mut WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>) -> Value {
        match socket.read().expect("a message") {
            Message::Text(text) => serde_json::from_str(text.as_str()).expect("JSON"),
            other => panic!("unexpected message {other:?}"),
        }
    }

    /// Send an event and return the `OK` message.
    fn publish(
        socket: &mut WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>,
        event: &Event,
    ) -> Value {
        send(socket, &json!(["EVENT", event]));
        next(socket)
    }

    /// The IDs of the events that a `REQ` gives, in sequence.
    fn query(
        socket: &mut WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>,
        filter: &Value,
    ) -> Vec<String> {
        send(socket, &json!(["REQ", "s", filter]));
        let mut ids = Vec::new();
        loop {
            let message = next(socket);
            match message[0].as_str() {
                Some("EVENT") => ids.push(message[2]["id"].as_str().expect("an ID").to_owned()),
                Some("EOSE") => return ids,
                _ => panic!("unexpected message {message}"),
            }
        }
    }

    #[test]
    fn a_replaceable_event_keeps_the_newest() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        let key = keys(1);
        let first = event(&key, 100, 10_002, None, "one");
        let second = event(&key, 200, 10_002, None, "two");
        assert_eq!(publish(&mut socket, &first)[2], true);
        assert_eq!(publish(&mut socket, &second)[2], true);
        let older = publish(&mut socket, &first);
        assert_eq!(older[2], false);
        assert_eq!(older[3], "duplicate: a newer event exists");
        assert_eq!(relay.events(), vec![second.clone()]);
        let other = event(&keys(2), 50, 10_002, None, "other");
        assert_eq!(publish(&mut socket, &other)[2], true);
        assert_eq!(relay.events().len(), 2);
        assert_eq!(relay.received(), 4);
    }

    #[test]
    fn the_lowest_id_stays_at_the_same_time() {
        let relay = TestRelay::start();
        let key = keys(1);
        let a = event(&key, 100, 10_002, None, "a");
        let b = event(&key, 100, 10_002, None, "b");
        let (low, high) = if a.id < b.id { (a, b) } else { (b, a) };
        relay.insert(high);
        relay.insert(low.clone());
        assert_eq!(relay.events(), vec![low]);
    }

    #[test]
    fn an_addressable_event_is_kept_for_each_d_value() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        let key = keys(1);
        let site_a = event(&key, 100, 35_128, Some("a"), "");
        let site_b = event(&key, 100, 35_128, Some("b"), "");
        let site_a_new = event(&key, 300, 35_128, Some("a"), "new");
        for e in [&site_a, &site_b, &site_a_new] {
            assert_eq!(publish(&mut socket, e)[2], true);
        }
        let stored = relay.events();
        assert_eq!(stored.len(), 2);
        assert!(stored.contains(&site_a_new));
        assert!(stored.contains(&site_b));
        assert_eq!(publish(&mut socket, &site_a)[2], false);
    }

    #[test]
    fn the_filters_select_events() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        let (one, two) = (keys(1), keys(2));
        let a = event(&one, 100, 1, None, "a");
        let b = event(&one, 200, 1, None, "b");
        let c = event(&two, 300, 35_128, Some("x"), "c");
        let d = event(&two, 400, 35_128, Some("y"), "d");
        for e in [&a, &b, &c, &d] {
            relay.insert(e.clone());
        }
        let id = |e: &Event| e.id.to_hex();
        let all = query(&mut socket, &json!({}));
        assert_eq!(all, [id(&d), id(&c), id(&b), id(&a)]);
        assert_eq!(query(&mut socket, &json!({"ids": [id(&a)]})), [id(&a)]);
        let author = one.public_key().to_hex();
        assert_eq!(
            query(&mut socket, &json!({"authors": [author]})),
            [id(&b), id(&a)]
        );
        assert_eq!(
            query(&mut socket, &json!({"kinds": [35_128]})),
            [id(&d), id(&c)]
        );
        assert_eq!(
            query(&mut socket, &json!({"kinds": [35_128], "#d": ["x"]})),
            [id(&c)]
        );
        assert_eq!(
            query(&mut socket, &json!({"since": 200, "until": 300})),
            [id(&c), id(&b)]
        );
        assert_eq!(query(&mut socket, &json!({"limit": 1})), [id(&d)]);
        // Two filters give the union, and each event shows one time.
        assert_eq!(
            query_two(&mut socket, &json!({"ids": [id(&a)]}), &json!({"limit": 2})),
            [id(&d), id(&c), id(&a)]
        );
    }

    fn query_two(
        socket: &mut WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>,
        first: &Value,
        second: &Value,
    ) -> Vec<String> {
        send(socket, &json!(["REQ", "s", first, second]));
        let mut ids = Vec::new();
        loop {
            let message = next(socket);
            if message[0] == "EOSE" {
                return ids;
            }
            ids.push(message[2]["id"].as_str().expect("an ID").to_owned());
        }
    }

    #[test]
    fn refuse_events_gives_ok_false() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        let e = event(&keys(1), 100, 1, None, "a");
        relay.refuse_events(Some("blocked: no writes"));
        let answer = publish(&mut socket, &e);
        assert_eq!(answer[2], false);
        assert_eq!(answer[3], "blocked: no writes");
        assert_eq!(relay.events(), Vec::<Event>::new());
        relay.refuse_events(None);
        assert_eq!(publish(&mut socket, &e)[2], true);
        assert_eq!(relay.events().len(), 1);
    }

    #[test]
    fn drop_events_says_ok_and_stores_nothing() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        relay.drop_events(true);
        let e = event(&keys(1), 100, 1, None, "a");
        let answer = publish(&mut socket, &e);
        assert_eq!(answer[2], true);
        assert_eq!(relay.events(), Vec::<Event>::new());
        assert_eq!(relay.received(), 1);
    }

    #[test]
    fn an_event_with_a_bad_signature_is_refused() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        let mut forged = serde_json::to_value(event(&keys(1), 100, 1, None, "a")).expect("JSON");
        forged["content"] = json!("changed");
        send(&mut socket, &json!(["EVENT", forged]));
        let answer = next(&mut socket);
        assert_eq!(answer[0], "OK");
        assert_eq!(answer[2], false);
        assert!(answer[3].as_str().expect("text").starts_with("invalid:"));
        assert_eq!(relay.events(), Vec::<Event>::new());
    }

    #[test]
    fn close_needs_no_answer() {
        let relay = TestRelay::start();
        let mut socket = connect(&relay);
        send(&mut socket, &json!(["CLOSE", "s"]));
        assert_eq!(query(&mut socket, &json!({})), Vec::<String>::new());
    }

    #[test]
    fn the_relay_stops_when_it_drops() {
        let relay = TestRelay::start();
        let url = relay.url();
        let port = relay.port;
        drop(relay);
        thread::sleep(Duration::from_millis(300));
        assert!(TcpStream::connect(("127.0.0.1", port)).is_err(), "{url}");
    }
}
