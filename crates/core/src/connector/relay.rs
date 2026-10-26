//! A blocking Nostr relay client (NIP-01) for the nsite connector.
//!
//! The client reads events with `REQ` and publishes events with `EVENT`.
//! It uses `ws://` and `wss://` addresses.

use std::io;
use std::net::TcpStream;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nostr::prelude::{Event, EventId, Filter, Kind, MatchEventOptions, PublicKey};
use serde_json::{Value, json};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error as WsError, Message, WebSocket};

use super::{ConnectorError, ConnectorErrorKind};
use crate::net::{self, NetError, NetPolicy};

/// The timeout for a real relay.
pub(crate) const RELAY_TIMEOUT: Duration = Duration::from_secs(10);

/// How often a waiting connection looks at the stop flag.
const STOP_POLL: Duration = Duration::from_millis(200);

/// The answer of a relay to a published event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Published {
    /// True when the relay accepted the event.
    pub accepted: bool,
    /// The message of the relay. It can be empty.
    pub message: String,
}

/// The result of one relay in [`newest`].
#[derive(Debug)]
pub(crate) struct RelayRead {
    /// The address of the relay.
    pub relay: String,
    /// The events of the relay, or the error.
    pub result: Result<Vec<Event>, ConnectorError>,
}

/// The reads of a list of relays.
#[derive(Debug)]
pub(crate) struct Reads {
    /// The result of each relay, in the sequence of the addresses.
    pub results: Vec<RelayRead>,
    /// The newest event of all relays.
    pub newest: Option<Event>,
}

impl Reads {
    /// The newest event, or `None` when no relay has a matching event.
    ///
    /// # Errors
    ///
    /// Gives `Unavailable` when no relay answered. Gives `NotPublic` when
    /// the policy refused each relay.
    pub(crate) fn into_newest(self) -> Result<Option<Event>, ConnectorError> {
        if self.results.iter().any(|r| r.result.is_ok()) {
            Ok(self.newest)
        } else if let Some(refused) = self.results.into_iter().find_map(|r| {
            r.result
                .err()
                .filter(|e| e.kind() == ConnectorErrorKind::NotPublic)
        }) {
            Err(refused)
        } else {
            Err(ConnectorError::new(
                ConnectorErrorKind::Unavailable,
                "no relay answered",
            ))
        }
    }
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/// A connection, with a handle to set the timeout of the socket.
struct Connection {
    socket: Socket,
    control: TcpStream,
    deadline: Instant,
}

fn unavailable(message: &str) -> ConnectorError {
    ConnectorError::new(ConnectorErrorKind::Unavailable, message)
}

/// The parts of a relay address: secure flag, host name and port.
fn parse_relay(url: &str) -> Result<(bool, String, u16), ConnectorError> {
    let bad = || {
        ConnectorError::new(
            ConnectorErrorKind::InvalidConfig,
            "the relay address is not a ws:// or wss:// address",
        )
    };
    let (secure, rest) = if let Some(rest) = url.strip_prefix("wss://") {
        (true, rest)
    } else if let Some(rest) = url.strip_prefix("ws://") {
        (false, rest)
    } else {
        return Err(bad());
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let (host, port_text) = if let Some(inner) = host_port.strip_prefix('[') {
        let (host, after) = inner.split_once(']').ok_or_else(bad)?;
        (host, after.strip_prefix(':'))
    } else {
        match host_port.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (host_port, None),
        }
    };
    if host.is_empty() {
        return Err(bad());
    }
    let port = match port_text {
        Some(text) => text.parse::<u16>().map_err(|_| bad())?,
        None if secure => 443,
        None => 80,
    };
    Ok((secure, host.to_owned(), port))
}

fn connect(url: &str, timeout: Duration, policy: &NetPolicy) -> Result<Connection, ConnectorError> {
    let (_, host, port) = parse_relay(url)?;
    let deadline = Instant::now() + timeout;
    let stream = net::connect(&host, port, policy, timeout).map_err(|e| match e {
        NetError::NotPublic { host } => ConnectorError::new(
            ConnectorErrorKind::NotPublic,
            net::not_public_message(&host),
        ),
        NetError::Io(_) => unavailable("Studio cannot connect to the relay"),
    })?;
    let control = stream
        .try_clone()
        .map_err(|_| unavailable("Studio cannot use the connection to the relay"))?;
    set_timeouts(&control, deadline)?;
    let (socket, _) = tungstenite::client_tls(url, stream)
        .map_err(|_| unavailable("the WebSocket handshake with the relay failed"))?;
    Ok(Connection {
        socket,
        control,
        deadline,
    })
}

/// Set the read and write timeout of the socket to the time that is left.
fn set_timeouts(control: &TcpStream, deadline: Instant) -> Result<(), ConnectorError> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(unavailable("the relay gave no answer in time"));
    }
    control
        .set_read_timeout(Some(left))
        .and_then(|()| control.set_write_timeout(Some(left)))
        .map_err(|_| unavailable("Studio cannot set the timeout of the relay connection"))
}

impl Connection {
    fn send(&mut self, value: &Value) -> Result<(), ConnectorError> {
        set_timeouts(&self.control, self.deadline)?;
        self.socket
            .send(Message::text(value.to_string()))
            .map_err(|_| unavailable("Studio cannot send a message to the relay"))
    }

    /// Read the next JSON array message from the relay.
    fn receive(&mut self) -> Result<Vec<Value>, ConnectorError> {
        loop {
            set_timeouts(&self.control, self.deadline)?;
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(Value::Array(message)) = serde_json::from_str::<Value>(text.as_str())
                    {
                        return Ok(message);
                    }
                }
                Ok(Message::Close(_)) | Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => {
                    return Err(unavailable("the relay closed the connection"));
                }
                Ok(_) => {}
                Err(WsError::Io(e))
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Err(unavailable("the relay gave no answer in time"));
                }
                Err(_) => return Err(unavailable("the connection to the relay failed")),
            }
        }
    }

    /// Read the next JSON array message, and look at the stop flag at
    /// short intervals. Gives `None` when the flag is set or the deadline
    /// has passed.
    fn receive_or_stop(&mut self, stop: &AtomicBool) -> Result<Option<Vec<Value>>, ConnectorError> {
        loop {
            if stop.load(Ordering::SeqCst) {
                return Ok(None);
            }
            let left = self.deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                // The deadline is not an error for a wait.
                return Ok(None);
            }
            self.control
                .set_read_timeout(Some(left.min(STOP_POLL)))
                .map_err(|_| {
                    unavailable("Studio cannot set the timeout of the relay connection")
                })?;
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(Value::Array(message)) = serde_json::from_str::<Value>(text.as_str())
                    {
                        return Ok(Some(message));
                    }
                }
                Ok(Message::Close(_)) | Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => {
                    return Err(unavailable("the relay closed the connection"));
                }
                Ok(_) => {}
                Err(WsError::Io(e))
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return Err(unavailable("the connection to the relay failed")),
            }
        }
    }

    /// Close the connection. A failure here does not matter.
    fn finish(mut self) {
        let _ = self.socket.close(None);
        let _ = self.socket.flush();
    }
}

/// Read the events that match a filter from one relay.
///
/// The function drops each event with a bad ID or a bad signature.
///
/// # Errors
///
/// Gives `InvalidConfig` for an address that is not `ws://` or `wss://`.
/// Gives `Unavailable` when the relay does not connect, or does not send
/// `EOSE` before the timeout.
pub(crate) fn fetch(
    relay_url: &str,
    filter: &Filter,
    timeout: Duration,
    policy: &NetPolicy,
) -> Result<Vec<Event>, ConnectorError> {
    let mut connection = connect(relay_url, timeout, policy)?;
    let id = uuid::Uuid::new_v4().simple().to_string();
    connection.send(&json!(["REQ", id, filter]))?;
    let mut events = Vec::new();
    let result = loop {
        let message = match connection.receive() {
            Ok(message) => message,
            Err(e) => break Err(e),
        };
        if message.get(1).and_then(Value::as_str) != Some(id.as_str()) {
            continue;
        }
        match message.first().and_then(Value::as_str) {
            Some("EVENT") => {
                let event = message
                    .get(2)
                    .and_then(|v| serde_json::from_value::<Event>(v.clone()).ok());
                if let Some(event) = event
                    && event.verify().is_ok()
                {
                    events.push(event);
                }
            }
            Some("EOSE") => break Ok(()),
            Some("CLOSED") => {
                break Err(ConnectorError::new(
                    ConnectorErrorKind::Io,
                    "the relay closed the subscription",
                ));
            }
            _ => {}
        }
    };
    if result.is_ok() {
        let _ = connection.send(&json!(["CLOSE", id]));
    }
    connection.finish();
    result.map(|()| events)
}

/// Publish an event to one relay and wait for the `OK` answer.
///
/// # Errors
///
/// Gives `InvalidConfig` for an address that is not `ws://` or `wss://`.
/// Gives `Unavailable` when the relay does not connect or does not answer
/// before the timeout. A refusal by the relay is not an error: it is in
/// [`Published`].
pub(crate) fn publish(
    relay_url: &str,
    event: &Event,
    timeout: Duration,
    policy: &NetPolicy,
) -> Result<Published, ConnectorError> {
    let mut connection = connect(relay_url, timeout, policy)?;
    let wanted: EventId = event.id;
    let result = (|| {
        connection.send(&json!(["EVENT", event]))?;
        loop {
            let message = connection.receive()?;
            if message.first().and_then(Value::as_str) == Some("OK")
                && message.get(1).and_then(Value::as_str) == Some(wanted.to_hex().as_str())
            {
                return Ok(Published {
                    accepted: message.get(2).and_then(Value::as_bool).unwrap_or(false),
                    message: message
                        .get(3)
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                });
            }
        }
    })();
    connection.finish();
    result
}

/// What [`exchange`] found.
#[derive(Debug)]
pub(crate) struct Exchange {
    /// The first event that the check accepted.
    pub event: Option<Event>,
}

/// Keep one subscription open on each relay until an event that matches
/// the filter and passes `accept` comes, or until the deadline. When
/// `send` is set, the function publishes it on the same connection after
/// the subscription. The function drops each event with a bad ID or a bad
/// signature. Thus a relay that does not store the event of the answer
/// still gives the answer to the waiting client. The caller can set
/// `stop` to end the wait early.
///
/// # Errors
///
/// Gives `NotPublic` when the policy refused each relay. Gives
/// `Unavailable` when no relay connected, for other reasons. A deadline
/// with no event is not an error: `event` is `None`.
pub(crate) fn exchange(
    relays: &[String],
    filter: &Filter,
    send: Option<&Event>,
    policy: &NetPolicy,
    deadline: Instant,
    accept: &(dyn Fn(&Event) -> bool + Sync),
    stop: &AtomicBool,
) -> Result<Exchange, ConnectorError> {
    let found: Mutex<Option<Event>> = Mutex::new(None);
    let outcomes: Vec<Result<(), ConnectorError>> = thread::scope(|scope| {
        let handles: Vec<_> = relays
            .iter()
            .map(|relay| {
                let found = &found;
                scope.spawn(move || {
                    wait_on_relay(relay, filter, send, policy, deadline, accept, stop, found)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(unavailable("the relay read stopped")))
            })
            .collect()
    });
    let event = found.into_inner().unwrap_or_default();
    if event.is_none() && outcomes.iter().all(Result::is_err) {
        let refused = outcomes
            .into_iter()
            .filter_map(Result::err)
            .find(|e| e.kind() == ConnectorErrorKind::NotPublic);
        return Err(refused.unwrap_or_else(|| unavailable("no relay answered")));
    }
    Ok(Exchange { event })
}

#[expect(
    clippy::too_many_arguments,
    reason = "the thread of one relay needs the whole context of the exchange"
)]
fn wait_on_relay(
    relay: &str,
    filter: &Filter,
    send: Option<&Event>,
    policy: &NetPolicy,
    deadline: Instant,
    accept: &(dyn Fn(&Event) -> bool + Sync),
    stop: &AtomicBool,
    found: &Mutex<Option<Event>>,
) -> Result<(), ConnectorError> {
    let timeout = deadline.saturating_duration_since(Instant::now());
    let mut connection = connect(relay, timeout, policy)?;
    let id = uuid::Uuid::new_v4().simple().to_string();
    let result = (|| {
        connection.send(&json!(["REQ", id, filter]))?;
        if let Some(event) = send {
            connection.send(&json!(["EVENT", event]))?;
        }
        loop {
            let Some(message) = connection.receive_or_stop(stop)? else {
                return Ok(());
            };
            if message.first().and_then(Value::as_str) != Some("EVENT")
                || message.get(1).and_then(Value::as_str) != Some(id.as_str())
            {
                continue;
            }
            let event = message
                .get(2)
                .and_then(|v| serde_json::from_value::<Event>(v.clone()).ok());
            if let Some(event) = event
                && event.verify().is_ok()
                && filter.match_event(&event, MatchEventOptions::new())
                && accept(&event)
            {
                if let Ok(mut slot) = found.lock()
                    && slot.is_none()
                {
                    *slot = Some(event);
                }
                stop.store(true, Ordering::SeqCst);
                return Ok(());
            }
        }
    })();
    let _ = connection.send(&json!(["CLOSE", id]));
    connection.finish();
    result
}

/// Check that the policy permits at least one of the relays.
///
/// # Errors
///
/// Gives `InvalidConfig` for an address that is not `ws://` or `wss://`.
/// Gives `NotPublic` when the policy refuses the host of each relay.
pub(crate) fn check_relays(relays: &[String], policy: &NetPolicy) -> Result<(), ConnectorError> {
    let mut refused = None;
    for relay in relays {
        let (_, host, port) = parse_relay(relay)?;
        match net::check_host(&host, port, policy) {
            Ok(()) => return Ok(()),
            Err(NetError::NotPublic { host }) => {
                refused = Some(ConnectorError::new(
                    ConnectorErrorKind::NotPublic,
                    net::not_public_message(&host),
                ));
            }
            Err(NetError::Io(_)) => {}
        }
    }
    Err(refused.unwrap_or_else(|| unavailable("Studio cannot find the relay")))
}

/// True when `a` is newer than `b` (NIP-01): the larger `created_at`, and
/// with the same time the lowest ID.
fn is_newer(a: &Event, b: &Event) -> bool {
    (a.created_at, b.id) > (b.created_at, a.id)
}

/// Read a filter from each relay at the same time, and find the newest
/// event of all relays.
pub(crate) fn newest(
    relays: &[String],
    filter: &Filter,
    timeout: Duration,
    policy: &NetPolicy,
) -> Reads {
    let results: Vec<RelayRead> = thread::scope(|scope| {
        let handles: Vec<_> = relays
            .iter()
            .map(|relay| scope.spawn(move || fetch(relay, filter, timeout, policy)))
            .collect();
        relays
            .iter()
            .zip(handles)
            .map(|(relay, handle)| RelayRead {
                relay: relay.clone(),
                result: handle
                    .join()
                    .unwrap_or_else(|_| Err(unavailable("the relay read stopped"))),
            })
            .collect()
    });
    let mut newest: Option<&Event> = None;
    for event in results
        .iter()
        .filter_map(|r| r.result.as_ref().ok())
        .flatten()
    {
        if newest.is_none_or(|n| is_newer(event, n)) {
            newest = Some(event);
        }
    }
    let newest = newest.cloned();
    Reads { results, newest }
}

/// The relay list and the server list of a key.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyLists {
    /// The relays of the key that accept a write (NIP-65, kind 10002).
    pub relays: Vec<String>,
    /// The Blossom servers of the key (BUD-03, kind 10063).
    pub servers: Vec<String>,
}

/// The values of the tags with a name, in the sequence of the event.
fn tag_values(event: Option<&Event>, name: &str, write_only: bool) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    for tag in event.iter().flat_map(|e| e.tags.iter()) {
        let parts = tag.as_slice();
        if parts.first().map(String::as_str) != Some(name) {
            continue;
        }
        let Some(value) = parts.get(1).filter(|v| !v.is_empty()) else {
            continue;
        };
        // A marker `read` means that the relay is not for a write.
        let marker = parts.get(2).map(String::as_str);
        if write_only && !matches!(marker, None | Some("" | "write")) {
            continue;
        }
        if !values.contains(value) {
            values.push(value.clone());
        }
    }
    values
}

/// Read the newest relay list (kind 10002) and the newest server list
/// (kind 10063) of a key from the lookup relays. The operator gives the
/// lookup relays, so the policy is `Open`.
///
/// `relays` has each `r` tag with no marker or with the marker `write`.
/// A key with no event has empty lists.
///
/// # Errors
///
/// Gives `Unavailable` when no lookup relay answered.
pub fn key_lists(
    public_key: &PublicKey,
    lookup_relays: &[String],
) -> Result<KeyLists, ConnectorError> {
    let filter = |kind: u16| Filter::new().author(*public_key).kind(Kind::Custom(kind));
    let relay_list = newest(
        lookup_relays,
        &filter(10002),
        RELAY_TIMEOUT,
        &NetPolicy::Open,
    )
    .into_newest()?;
    let server_list = newest(
        lookup_relays,
        &filter(10063),
        RELAY_TIMEOUT,
        &NetPolicy::Open,
    )
    .into_newest()?;
    Ok(KeyLists {
        relays: tag_values(relay_list.as_ref(), "r", true),
        servers: tag_values(server_list.as_ref(), "server", false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr_events::sign_with_keys;
    use crate::testing::relay::TestRelay;
    use nostr::prelude::{Keys, Kind, SecretKey, Tag, Timestamp, UnsignedEvent};
    use std::net::TcpListener;

    const SHORT: Duration = Duration::from_secs(5);

    #[test]
    fn exchange_gets_an_event_that_comes_after_the_subscription() {
        let relay = TestRelay::start();
        let key = keys(7);
        let filter = filter(&key);
        let stop = AtomicBool::new(false);
        let url = relay.url();
        let later = event(&key, 100, "x", "late");
        let sender = thread::spawn({
            let relay_url = url.clone();
            move || {
                thread::sleep(Duration::from_millis(300));
                let sent = publish(&relay_url, &later, SHORT, &NetPolicy::Open).expect("a send");
                assert!(sent.accepted);
            }
        });
        let found = exchange(
            &[url],
            &filter,
            None,
            &NetPolicy::Open,
            Instant::now() + SHORT,
            &|_| true,
            &stop,
        )
        .expect("an exchange");
        sender.join().expect("a thread");
        assert_eq!(found.event.expect("an event").content, "late");
    }

    #[test]
    fn exchange_gives_no_event_at_the_deadline_and_an_error_for_no_relay() {
        let relay = TestRelay::start();
        let stop = AtomicBool::new(false);
        let filter = Filter::new().kind(Kind::Custom(1));
        let found = exchange(
            &[relay.url()],
            &filter,
            None,
            &NetPolicy::Open,
            Instant::now() + Duration::from_millis(400),
            &|_| true,
            &stop,
        )
        .expect("an exchange");
        assert!(found.event.is_none());
        let closed = TcpListener::bind("127.0.0.1:0").expect("a port");
        let url = format!("ws://{}", closed.local_addr().expect("an address"));
        drop(closed);
        let error = exchange(
            &[url],
            &filter,
            None,
            &NetPolicy::Open,
            Instant::now() + SHORT,
            &|_| true,
            &stop,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
        let error = exchange(
            &[relay.url()],
            &filter,
            None,
            &NetPolicy::public(),
            Instant::now() + SHORT,
            &|_| true,
            &stop,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::NotPublic);
    }

    fn keys(byte: u8) -> Keys {
        Keys::new(SecretKey::from_slice(&[byte; 32]).expect("a secret key"))
    }

    fn event(keys: &Keys, time: u64, d: &str, content: &str) -> Event {
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(time),
            Kind::Custom(35128),
            [Tag::identifier(d)],
            content,
        );
        sign_with_keys(unsigned, keys).expect("a signed event")
    }

    fn filter(keys: &Keys) -> Filter {
        Filter::new()
            .author(keys.public_key())
            .kind(Kind::Custom(35128))
    }

    fn dead_address() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
        let port = listener.local_addr().expect("an address").port();
        drop(listener);
        format!("ws://127.0.0.1:{port}")
    }

    #[test]
    fn fetch_gives_the_events_of_a_filter() {
        let relay = TestRelay::start();
        let a = keys(1);
        let b = keys(2);
        let mine = event(&a, 100, "site", "mine");
        relay.insert(mine.clone());
        relay.insert(event(&b, 100, "site", "other"));
        let events = fetch(&relay.url(), &filter(&a), SHORT, &NetPolicy::Open).expect("events");
        assert_eq!(events, vec![mine]);
    }

    #[test]
    fn fetch_drops_an_event_with_a_bad_signature() {
        // A relay that sends a changed event is made with a raw server.
        let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
        let port = listener.local_addr().expect("an address").port();
        let a = keys(1);
        let good = event(&a, 100, "one", "good");
        let mut bad_json = serde_json::to_value(event(&a, 101, "two", "bad")).expect("JSON");
        bad_json["content"] = json!("changed");
        let good_json = serde_json::to_value(&good).expect("JSON");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("a client");
            let mut socket = tungstenite::accept(stream).expect("a handshake");
            let Message::Text(text) = socket.read().expect("a REQ") else {
                panic!("not text");
            };
            let request: Value = serde_json::from_str(text.as_str()).expect("JSON");
            let sub = request[1].clone();
            for value in [bad_json, good_json] {
                socket
                    .send(Message::text(json!(["EVENT", sub, value]).to_string()))
                    .expect("sent");
            }
            socket
                .send(Message::text(json!(["EOSE", sub]).to_string()))
                .expect("sent");
            let _ = socket.read();
        });
        let events = fetch(
            &format!("ws://127.0.0.1:{port}"),
            &filter(&a),
            SHORT,
            &NetPolicy::Open,
        )
        .expect("events");
        assert_eq!(events, vec![good]);
        server.join().expect("the server thread");
    }

    #[test]
    fn publish_gives_ok_true_and_the_relay_stores_the_event() {
        let relay = TestRelay::start();
        let a = keys(1);
        let e = event(&a, 100, "site", "x");
        let answer = publish(&relay.url(), &e, RELAY_TIMEOUT, &NetPolicy::Open).expect("an answer");
        assert!(answer.accepted);
        assert_eq!(relay.events(), vec![e]);
    }

    #[test]
    fn publish_gives_the_refusal_of_the_relay() {
        let relay = TestRelay::start();
        relay.refuse_events(Some("blocked: no"));
        let e = event(&keys(1), 100, "site", "x");
        let answer = publish(&relay.url(), &e, SHORT, &NetPolicy::Open).expect("an answer");
        assert!(!answer.accepted);
        assert_eq!(answer.message, "blocked: no");
    }

    #[test]
    fn newest_gives_the_largest_time_of_two_relays() {
        let (one, two) = (TestRelay::start(), TestRelay::start());
        let a = keys(1);
        let old = event(&a, 100, "site", "old");
        let new = event(&a, 200, "site", "new");
        one.insert(old);
        two.insert(new.clone());
        let relays = vec![one.url(), two.url()];
        let reads = newest(&relays, &filter(&a), SHORT, &NetPolicy::Open);
        assert_eq!(reads.results.len(), 2);
        assert_eq!(reads.results[0].relay, relays[0]);
        assert_eq!(reads.into_newest().expect("answered"), Some(new));
    }

    #[test]
    fn newest_gives_the_lowest_id_for_the_same_time() {
        let (one, two) = (TestRelay::start(), TestRelay::start());
        let a = keys(1);
        let first = event(&a, 100, "site", "first");
        let second = event(&a, 100, "site", "second");
        let lowest = if first.id < second.id {
            &first
        } else {
            &second
        };
        one.insert(first.clone());
        two.insert(second.clone());
        let relays = vec![one.url(), two.url()];
        let reads = newest(&relays, &filter(&a), SHORT, &NetPolicy::Open);
        assert_eq!(reads.newest.as_ref(), Some(lowest));
    }

    #[test]
    fn newest_skips_a_relay_with_no_server() {
        let relay = TestRelay::start();
        let a = keys(1);
        let e = event(&a, 100, "site", "x");
        relay.insert(e.clone());
        let relays = vec![dead_address(), relay.url()];
        let reads = newest(&relays, &filter(&a), SHORT, &NetPolicy::Open);
        assert_eq!(
            reads.results[0]
                .result
                .as_ref()
                .expect_err("no server")
                .kind(),
            ConnectorErrorKind::Unavailable
        );
        assert_eq!(reads.into_newest().expect("answered"), Some(e));
    }

    #[test]
    fn newest_gives_unavailable_when_no_relay_answers() {
        let reads = newest(&[dead_address()], &Filter::new(), SHORT, &NetPolicy::Open);
        let error = reads.into_newest().expect_err("no answer");
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    fn list_event(keys: &Keys, time: u64, kind: u16, tags: Vec<Tag>) -> Event {
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(time),
            Kind::Custom(kind),
            tags,
            "",
        );
        sign_with_keys(unsigned, keys).expect("a signed event")
    }

    fn tag(parts: &[&str]) -> Tag {
        Tag::parse(parts.iter().map(|p| (*p).to_owned()).collect::<Vec<_>>()).expect("a tag")
    }

    #[test]
    fn key_lists_gives_the_newest_relays_and_servers_of_a_key() {
        let relay = TestRelay::start();
        let a = keys(1);
        relay.insert(list_event(
            &a,
            100,
            10002,
            vec![tag(&["r", "wss://old.example"])],
        ));
        relay.insert(list_event(
            &a,
            200,
            10002,
            vec![
                tag(&["r", "wss://one.example"]),
                tag(&["r", "wss://two.example", "write"]),
                tag(&["r", "wss://three.example", "read"]),
            ],
        ));
        relay.insert(list_event(
            &a,
            150,
            10063,
            vec![tag(&["server", "https://blob.example"])],
        ));
        let lists = key_lists(&a.public_key(), &[relay.url()]).expect("lists");
        assert_eq!(lists.relays, ["wss://one.example", "wss://two.example"]);
        assert_eq!(lists.servers, ["https://blob.example"]);
    }

    #[test]
    fn key_lists_gives_empty_lists_for_a_key_with_no_events() {
        let relay = TestRelay::start();
        let lists = key_lists(&keys(1).public_key(), &[relay.url()]).expect("lists");
        assert_eq!(lists, KeyLists::default());
    }

    #[test]
    fn key_lists_gives_unavailable_when_no_relay_answers() {
        let error = key_lists(&keys(1).public_key(), &[dead_address()]).expect_err("no answer");
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    #[test]
    fn an_address_that_is_not_a_relay_address_gives_invalid_config() {
        let error = fetch(
            "http://127.0.0.1:1",
            &Filter::new(),
            SHORT,
            &NetPolicy::Open,
        )
        .expect_err("refused");
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
    }

    #[test]
    fn a_relay_address_has_a_default_port() {
        assert_eq!(parse_relay("wss://Relay.example/x").expect("parsed").2, 443);
        assert_eq!(parse_relay("ws://[::1]:7000").expect("parsed").2, 7000);
    }
}
