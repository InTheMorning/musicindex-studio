//! Tests of the client against a local test server (ADR 0110).

#![allow(missing_docs)]

mod common;

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use common::{Resp, Server, now, verify};
use nostr::nips::nip44::{self, Nonce};
use nostr::prelude::{Keys, PublicKey, SecretKey};
use studio_app_client::{
    AppSigner, Client, Error, FileSigner, MemorySigner, PairingText, ShowStatus,
};

const GRANT_KEY: &str = "R3JhbnRWYXVsdEtleUZvclRlc3RzMDAwMDAwMDAwMDA=";

fn pairing(server: &Server) -> PairingText {
    PairingText::parse(&format!("{}#pair=K7M2-Q9XD", server.origin)).unwrap()
}

fn encrypt_for(app: &PublicKey) -> String {
    let sender = Keys::new(SecretKey::from_slice(&[7_u8; 32]).unwrap());
    let ciphertext =
        nip44::encrypt_with_nonce(sender.secret_key(), app, GRANT_KEY, Nonce::V2([3_u8; 32]))
            .unwrap();
    format!(
        r#"{{"sender":"{}","ciphertext":"{ciphertext}"}}"#,
        sender.public_key().to_hex()
    )
}

/// A server that checks each request like Studio does, with a clock that is
/// `skew` seconds ahead of the clock of the test.
fn studio(skew: Arc<AtomicI64>) -> Server {
    let origin = Arc::new(Mutex::new(String::new()));
    let origin_set = Arc::clone(&origin);
    let server = Server::start(move |req, _| {
        let server_time = now() + skew.load(Ordering::SeqCst);
        let origin = origin.lock().unwrap().clone();
        let Ok(event) = verify(req, &origin, server_time) else {
            return Resp::error(401, "app_signature_invalid").with_date(server_time);
        };
        match (req.method.as_str(), req.path.as_str()) {
            ("POST", "/app/v1/pair") | ("GET", "/app/v1/whoami") => {
                Resp::json(200, r#"{"id":1,"name":"Tool","state":"active"}"#)
            }
            ("GET", "/app/v1/shows") => Resp::json(
                200,
                r#"[{"grant_id":5,"feed_id":2,"item_guid":"g","status":"pending","live_value_origin":"wss://live.example.com"}]"#,
            ),
            ("GET", "/app/v1/shows/5/wrapper") => Resp::json(200, &encrypt_for(&event.pubkey)),
            (_, p) if p.starts_with("/app/v1/shows/5/") => {
                if req.json()["grant_key"] == GRANT_KEY {
                    Resp::json(200, r#"{"version":"v1","status":"live"}"#)
                } else {
                    Resp::error(423, "vault_locked")
                }
            }
            _ => Resp::error(404, "not_found"),
        }
    });
    origin_set.lock().unwrap().clone_from(&server.origin);
    server
}

fn app(server: &Server) -> Client {
    Client::connect(&server.origin, MemorySigner::generate().unwrap()).unwrap()
}

#[test]
fn each_request_carries_a_fresh_event_with_correct_tags() {
    let server = studio(Arc::new(AtomicI64::new(0)));
    let client = app(&server);
    client.whoami().unwrap();
    client.whoami().unwrap();
    client.shows().unwrap();
    client.set_status(5, ShowStatus::Live).unwrap();
    client
        .set_live_value(5, "wss://live.example.com/a", "lnurlp")
        .unwrap();
    client.remove_live_value(5).unwrap();

    let requests = server.requests();
    let mut ids = std::collections::HashSet::new();
    for req in &requests {
        let event = verify(req, &server.origin, now()).unwrap_or_else(|why| {
            panic!("ADR 0110 item 12: u, method and payload must be exact ({why}) for {} {}. Fix Client::authorization", req.method, req.path)
        });
        assert!(
            ids.insert(event.id),
            "ADR 0110 item 12: each request needs a new event. Sign again for each request"
        );
        let has_payload = event.tags.iter().any(|t| t.as_slice()[0] == "payload");
        assert_eq!(
            has_payload,
            !req.body.is_empty(),
            "ADR 0110 item 12: a payload tag exactly when the body is not empty"
        );
    }
    let wrappers = requests
        .iter()
        .filter(|r| r.path.ends_with("/wrapper"))
        .count();
    assert_eq!(
        wrappers, 1,
        "ADR 0110 item 29: get the wrapper once for a grant"
    );
    let delete = requests.iter().find(|r| r.method == "DELETE").unwrap();
    assert_eq!(delete.json()["grant_key"], GRANT_KEY);
}

#[test]
fn pair_sends_the_code_and_the_origin_in_the_signed_body() {
    let server = studio(Arc::new(AtomicI64::new(0)));
    let text = PairingText::parse_with_origin("k7m2 q9xd", &server.origin).unwrap();
    let client = Client::pair(
        &text,
        MemorySigner::generate().unwrap(),
        "wss://live.example.com",
    )
    .unwrap();
    assert_eq!(client.origin(), server.origin);
    let req = &server.requests()[0];
    assert_eq!(
        (req.method.as_str(), req.path.as_str()),
        ("POST", "/app/v1/pair")
    );
    assert_eq!(req.json()["code"], "K7M2Q9XD");
    assert_eq!(req.json()["live_value_origin"], "wss://live.example.com");
    assert!(verify(req, &server.origin, now()).is_ok());
}

#[test]
fn the_grant_key_is_cleared_at_the_end_404_and_423() {
    let server = studio(Arc::new(AtomicI64::new(0)));
    let client = app(&server);
    client.set_status(5, ShowStatus::Live).unwrap();
    assert!(client.holds_grant_key(5));
    client.set_status(5, ShowStatus::Ended).unwrap();
    assert!(
        !client.holds_grant_key(5),
        "ADR 0110 item 29: clear the grant vault key when the grant ends"
    );

    // 404 for a grant that the app does not have: no key, NotGranted.
    assert_eq!(
        client.set_status(9, ShowStatus::Live),
        Err(Error::NotGranted)
    );
    assert!(!client.holds_grant_key(9));
}

#[test]
fn a_423_and_a_404_from_a_write_clear_the_key() {
    let count = Arc::new(AtomicI64::new(0));
    let seen = Arc::clone(&count);
    let origin = Arc::new(Mutex::new(String::new()));
    let origin_set = Arc::clone(&origin);
    let server = Server::start(move |req, _| {
        let origin = origin.lock().unwrap().clone();
        let event = verify(req, &origin, now()).expect("correct request");
        if req.path.ends_with("/wrapper") {
            return Resp::json(200, &encrypt_for(&event.pubkey));
        }
        match seen.fetch_add(1, Ordering::SeqCst) {
            0 => Resp::error(423, "vault_locked"),
            1 => Resp::error(404, "not_found"),
            _ => Resp::json(200, "{}"),
        }
    });
    origin_set.lock().unwrap().clone_from(&server.origin);
    let client = app(&server);
    assert_eq!(
        client.set_status(5, ShowStatus::Live),
        Err(Error::VaultLocked)
    );
    assert!(
        !client.holds_grant_key(5),
        "ADR 0110 item 29: clear the grant vault key after 423"
    );
    assert_eq!(client.remove_live_value(5), Err(Error::NotGranted));
    assert!(
        !client.holds_grant_key(5),
        "ADR 0110 item 29: clear the key after 404"
    );
    client.remove_live_value(5).unwrap();
    let wrappers = server
        .requests()
        .iter()
        .filter(|r| r.path.ends_with("/wrapper"))
        .count();
    assert_eq!(wrappers, 3, "a cleared key needs a new wrapper");
}

#[test]
fn a_grant_that_is_not_in_the_list_any_more_loses_its_key() {
    let server = studio(Arc::new(AtomicI64::new(0)));
    let client = app(&server);
    client.set_status(5, ShowStatus::Live).unwrap();
    client.forget_grant_key(5);
    assert!(!client.holds_grant_key(5));
    client.set_status(5, ShowStatus::Live).unwrap();
    assert!(client.holds_grant_key(5));
    client.shows().unwrap();
    assert!(client.holds_grant_key(5), "grant 5 is still in the list");
}

#[test]
fn a_clock_3_minutes_wrong_gets_one_correction_and_a_warning() {
    let skew = Arc::new(AtomicI64::new(180));
    let server = studio(Arc::clone(&skew));
    let warnings = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&warnings);
    let client = app(&server).with_clock_warning(move |s| sink.lock().unwrap().push(s));
    client.whoami().unwrap();
    assert_eq!(
        server.count(),
        2,
        "ADR 0110 item 36: one correction with one second request"
    );
    let warned = warnings.lock().unwrap().clone();
    assert_eq!(warned.len(), 1, "ADR 0110 item 36: warn above 30 seconds");
    assert!((warned[0] - 180).abs() <= 2);
    assert!((client.clock_offset() - 180).abs() <= 2);
    // The correction stays for the next requests.
    client.whoami().unwrap();
    assert_eq!(server.count(), 3, "ADR 0110 item 36: keep the correction");
}

#[test]
fn a_clock_off_by_more_than_a_minute_gets_a_correction_and_a_warning() {
    let skew = Arc::new(AtomicI64::new(-90));
    let server = studio(skew);
    let warnings = Arc::new(Mutex::new(0));
    let sink = Arc::clone(&warnings);
    let client = app(&server).with_clock_warning(move |_| *sink.lock().unwrap() += 1);
    client.whoami().unwrap();
    assert_eq!(server.count(), 2);
    assert_eq!(*warnings.lock().unwrap(), 1);

    let skew = Arc::new(AtomicI64::new(70));
    let server = studio(skew);
    let warnings = Arc::new(Mutex::new(0));
    let sink = Arc::clone(&warnings);
    let client = app(&server).with_clock_warning(move |_| *sink.lock().unwrap() += 1);
    client.whoami().unwrap();
    assert_eq!(*warnings.lock().unwrap(), 1, "70 seconds is above 30");
}

#[test]
fn a_clock_10_minutes_wrong_gives_clock_wrong_and_no_second_request() {
    let server = studio(Arc::new(AtomicI64::new(600)));
    let client = app(&server);
    let error = client.whoami().unwrap_err();
    assert!(
        matches!(error, Error::ClockWrong(s) if (s - 600).abs() <= 2),
        "ADR 0110 item 36: above 5 minutes give ClockWrong ({error:?})"
    );
    assert_eq!(
        server.count(),
        1,
        "ADR 0110 item 36: send no second request above 5 minutes"
    );
}

#[test]
fn a_forged_date_on_the_second_401_causes_no_third_request() {
    for second in [now() + 240, now() + 900] {
        let server = Server::start(move |_, index| {
            let date = if index == 0 { now() + 180 } else { second };
            Resp::error(401, "app_signature_invalid").with_date(date)
        });
        let client = app(&server);
        let error = client.whoami().unwrap_err();
        assert!(
            matches!(error, Error::SignatureInvalid | Error::ClockWrong(_)),
            "{error:?}"
        );
        assert_eq!(
            server.count(),
            2,
            "ADR 0110 item 36: a second 401 must not cause a third request. Repeat once only"
        );
    }
}

#[test]
fn a_401_with_the_right_clock_is_not_repeated() {
    let server = Server::start(|_, _| Resp::error(401, "app_signature_invalid").with_date(now()));
    assert_eq!(app(&server).whoami(), Err(Error::SignatureInvalid));
    assert_eq!(server.count(), 1);
    // No Date header: no correction.
    let server = Server::start(|_, _| Resp::error(401, "app_signature_invalid"));
    assert_eq!(app(&server).whoami(), Err(Error::SignatureInvalid));
    assert_eq!(server.count(), 1);
}

#[test]
fn a_code_error_is_not_a_clock_error() {
    let server = Server::start(|_, _| Resp::error(401, "app_code_invalid").with_date(now() + 120));
    let text = pairing(&server);
    let error = Client::pair(
        &text,
        MemorySigner::generate().unwrap(),
        "wss://l.example.com",
    )
    .unwrap_err();
    assert_eq!(error, Error::CodeInvalid);
    assert_eq!(
        server.count(),
        1,
        "app_code_invalid starts no clock correction"
    );
}

#[test]
fn each_error_answer_has_its_error_type() {
    let cases: [(u16, &str, Error); 9] = [
        (409, "app_code_used", Error::CodeUsed),
        (401, "app_key_required", Error::KeyRequired),
        (409, "app_key_in_use", Error::KeyInUse),
        (400, "live_value_invalid", Error::LiveValueInvalid),
        (404, "not_found", Error::NotGranted),
        (423, "vault_locked", Error::VaultLocked),
        (429, "too_many_attempts", Error::TooManyAttempts),
        (401, "app_code_invalid", Error::CodeInvalid),
        (401, "app_signature_invalid", Error::SignatureInvalid),
    ];
    for (status, code, expected) in cases {
        let server = Server::start(move |_, _| Resp::error(status, code));
        assert_eq!(
            app(&server).whoami(),
            Err(expected),
            "ADR 0110 item 9: {status} {code} needs its own error type. Fix map_error"
        );
    }
    let server = Server::start(|_, _| Resp::error(503, "x"));
    assert!(matches!(app(&server).whoami(), Err(Error::Network(_))));
}

#[test]
fn an_unreachable_studio_gives_network() {
    let server = Server::start(|_, _| Resp::json(200, "{}"));
    let origin = server.origin.clone();
    drop(server);
    // The listener thread still holds the port, so use a port with no listener.
    let closed = origin.replace(origin.rsplit(':').next().unwrap(), "9");
    let client = Client::connect(&closed, MemorySigner::generate().unwrap()).unwrap();
    assert!(matches!(client.whoami(), Err(Error::Network(_))));
}

#[test]
fn an_http_origin_that_is_not_loopback_is_refused() {
    for origin in [
        "http://studio.example.com",
        "http://10.0.0.5:8080",
        "ftp://x.example.com",
    ] {
        let result = Client::connect(origin, MemorySigner::generate().unwrap());
        assert!(
            matches!(result, Err(Error::OriginInvalid(_))),
            "ADR 0110 item 35: refuse a Studio origin that is not https ({origin}). Use https"
        );
    }
    assert!(Client::connect("http://127.0.0.1:1234", MemorySigner::generate().unwrap()).is_ok());
    assert!(
        Client::connect(
            "https://studio.example.com",
            MemorySigner::generate().unwrap()
        )
        .is_ok()
    );
}

#[test]
fn no_error_text_holds_the_code_an_event_or_a_key() {
    let server = Server::start(|req, _| {
        // Echo secrets in the error body, to see that the client keeps them out.
        Resp::json(
            409,
            &format!(
                r#"{{"error":{{"code":"app_code_used","message":"K7M2Q9XD {}"}}}}"#,
                req.headers
                    .get("authorization")
                    .cloned()
                    .unwrap_or_default()
            ),
        )
    });
    let text = pairing(&server);
    let signer = MemorySigner::generate().unwrap();
    let key_hex = format!("{signer:?}");
    let error = Client::pair(&text, signer, "wss://l.example.com").unwrap_err();
    let shown = format!("{error} {error:?} {text:?} {key_hex}");
    for secret in ["K7M2", "Q9XD", "Nostr "] {
        assert!(
            !shown.contains(secret),
            "ADR 0110 item 9: no secret in {shown}"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_key_file_has_mode_0600_in_a_folder_with_mode_0700() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("new").join("studio.key");
    let signer = FileSigner::load_or_create(&path).unwrap();
    let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode(&path),
        0o600,
        "ADR 0110 item 2: the key file has mode 0600. Create it with that mode"
    );
    assert_eq!(
        mode(path.parent().unwrap()),
        0o700,
        "ADR 0110 item 2: the key folder has mode 0700"
    );
    // Reading it again gives the same key.
    assert_eq!(
        FileSigner::load_or_create(&path).unwrap().public_key(),
        signer.public_key()
    );
    // A file that others can read is refused.
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        FileSigner::load_or_create(&path),
        Err(Error::Signer(_))
    ));
}
