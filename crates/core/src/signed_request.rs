//! The check of a signed request (ADR 0110 items 12 to 14, ADR 0056
//! item 6).
//!
//! A request carries an event of kind 27235, in the form of NIP-98. The
//! server sign-in and the `/app/v1/` routes use the same check. The check is
//! pure: it reads no database and no clock. The caller gives the expected
//! address, the method, the body and the time.

use base64::Engine as _;
use nostr::prelude::{Event, PublicKey};
use sha2::{Digest, Sha256};

/// The kind of a signed request event (NIP-98).
pub const KIND_HTTP_AUTH: u16 = 27_235;
/// The most seconds that the `created_at` of an event can differ from the
/// time of the server.
pub const TIME_WINDOW: u64 = 60;
/// The header scheme of an app request (ADR 0110 item 12).
const SCHEME: &str = "Nostr ";
/// The most bytes of the decoded event in a header. A normal event has less
/// than 1 KiB.
const MAX_EVENT_BYTES: usize = 8 * 1024;

/// A reason why a signed request is not correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SignedRequestError {
    /// The kind is not 27235.
    Kind,
    /// The ID or the signature is not correct.
    Signature,
    /// The `created_at` differs from the time of the server by more than
    /// 60 seconds.
    Time,
    /// The `u` tag is not the expected address, character for character.
    Url,
    /// The `method` tag is not the method of the request, character for
    /// character.
    Method,
    /// The request has a body, and the event has no `payload` tag or a
    /// `payload` tag that is not the SHA-256 of the body.
    Payload,
}

impl SignedRequestError {
    /// A short reason. It names no event, key or address.
    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Self::Kind => "the event kind is not 27235",
            Self::Signature => "the event signature is not correct",
            Self::Time => "the event time is not within 60 seconds of the server time",
            Self::Url => "the u tag is not the address of the request",
            Self::Method => "the method tag is not the method of the request",
            Self::Payload => "the payload tag is not the SHA-256 of the body",
        }
    }
}

impl std::fmt::Display for SignedRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason())
    }
}

impl std::error::Error for SignedRequestError {}

/// What the caller expects of a signed request.
#[derive(Debug, Clone, Copy)]
pub struct ExpectedRequest<'a> {
    /// The full address of the request. The caller makes it from the public
    /// origin of its configuration, not from the `Host` header.
    pub url: &'a str,
    /// The HTTP method, in uppercase.
    pub method: &'a str,
    /// The raw bytes of the body. A request with a body needs a `payload`
    /// tag with the SHA-256 of these bytes. `None` skips the payload check.
    /// Only a request that carries its own event in the body, such as the
    /// sign-in, can use `None`, because the body cannot hold its own hash.
    pub body: Option<&'a [u8]>,
    /// The time of the server in seconds since the Unix epoch.
    pub now: u64,
}

/// The first value of the tag `name`.
fn tag_value<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event.tags.iter().find_map(|tag| match tag.as_slice() {
        [first, value, ..] if first == name => Some(value.as_str()),
        _ => None,
    })
}

/// Check a signed request against what the caller expects. Gives the public
/// key of the signer. The check refuses each of these conditions:
///
/// - The kind is not 27235, or the ID or the signature is not correct.
/// - The `created_at` differs from `expected.now` by more than 60 seconds.
/// - The `u` or `method` tag is not the same as `expected`, character for
///   character.
/// - The request has a body, and the event has no `payload` tag or one that
///   is not the SHA-256 of the body.
///
/// The check does not know the replay store or the app list. The caller
/// examines them.
///
/// # Errors
///
/// Gives a [`SignedRequestError`] for the first condition that fails.
pub fn check(
    event: &Event,
    expected: &ExpectedRequest<'_>,
) -> Result<PublicKey, SignedRequestError> {
    if event.kind.as_u16() != KIND_HTTP_AUTH {
        return Err(SignedRequestError::Kind);
    }
    if event.verify().is_err() {
        return Err(SignedRequestError::Signature);
    }
    if event.created_at.as_secs().abs_diff(expected.now) > TIME_WINDOW {
        return Err(SignedRequestError::Time);
    }
    if tag_value(event, "u") != Some(expected.url) {
        return Err(SignedRequestError::Url);
    }
    if tag_value(event, "method") != Some(expected.method) {
        return Err(SignedRequestError::Method);
    }
    if let Some(body) = expected.body
        && !body.is_empty()
    {
        let hash = hex_sha256(body);
        if tag_value(event, "payload") != Some(hash.as_str()) {
            return Err(SignedRequestError::Payload);
        }
    }
    Ok(event.pubkey)
}

/// Read the event of an `Authorization: Nostr <base64 event>` header value.
/// Gives `None` when the value has another scheme, is not base64, is too
/// large, or is not an event. The function does not verify the event.
#[must_use]
pub fn parse_header(value: &str) -> Option<Event> {
    let encoded = value.strip_prefix(SCHEME)?.trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    if bytes.len() > MAX_EVENT_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// The SHA-256 of `bytes` as lowercase hexadecimal.
fn hex_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, b| {
            let _ = write!(text, "{b:02x}");
            text
        })
}

/// A new random key pair for a test.
#[cfg(test)]
pub(crate) fn generate_keys() -> nostr::prelude::Keys {
    loop {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).unwrap();
        if let Ok(secret) = nostr::prelude::SecretKey::from_slice(&bytes) {
            return nostr::prelude::Keys::new(secret);
        }
    }
}

/// A signed event for a test.
#[cfg(test)]
pub(crate) fn signed_event(
    keys: &nostr::prelude::Keys,
    kind: u16,
    tags: &[[&str; 2]],
    created_at: u64,
) -> Event {
    use nostr::prelude::{Kind, Tag, Timestamp, UnsignedEvent};

    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::from_secs(created_at),
        Kind::Custom(kind),
        tags.iter().map(|t| Tag::parse(*t).unwrap()),
        "",
    );
    crate::nostr_events::sign_with_keys(unsigned, keys).unwrap()
}

#[cfg(test)]
mod tests {
    use nostr::prelude::Keys;

    use super::generate_keys as keys_generate;
    use super::*;

    const URL: &str = "https://studio.example.com/app/v1/shows";
    const NOW: u64 = 1_000_000;

    fn build(kind: u16, tags: &[[&str; 2]], created_at: u64, keys: &Keys) -> Event {
        signed_event(keys, kind, tags, created_at)
    }

    fn good(keys: &Keys) -> Event {
        build(KIND_HTTP_AUTH, &[["u", URL], ["method", "GET"]], NOW, keys)
    }

    fn get(now: u64) -> ExpectedRequest<'static> {
        ExpectedRequest {
            url: URL,
            method: "GET",
            body: Some(b""),
            now,
        }
    }

    #[test]
    fn a_correct_request_gives_the_public_key() {
        let keys = keys_generate();
        assert_eq!(check(&good(&keys), &get(NOW)), Ok(keys.public_key()));
        assert_eq!(check(&good(&keys), &get(NOW + 60)), Ok(keys.public_key()));
        assert_eq!(check(&good(&keys), &get(NOW - 60)), Ok(keys.public_key()));
    }

    #[test]
    fn each_condition_of_adr_0110_item_13_is_refused() {
        let keys = keys_generate();
        let hint =
            "ADR 0110 item 13: the shared check must refuse this. Fix signed_request::check.";

        // The kind.
        let event = build(1, &[["u", URL], ["method", "GET"]], NOW, &keys);
        assert_eq!(
            check(&event, &get(NOW)),
            Err(SignedRequestError::Kind),
            "{hint}"
        );

        // The signature: a changed tag after signing.
        let mut json = serde_json::to_value(good(&keys)).unwrap();
        json["sig"] = serde_json::json!("0".repeat(128));
        let event: Event = serde_json::from_value(json).unwrap();
        assert_eq!(
            check(&event, &get(NOW)),
            Err(SignedRequestError::Signature),
            "{hint}"
        );
        let mut json = serde_json::to_value(good(&keys)).unwrap();
        json["tags"] = serde_json::json!([["u", URL], ["method", "POST"]]);
        let event: Event = serde_json::from_value(json).unwrap();
        assert_eq!(
            check(&event, &get(NOW)),
            Err(SignedRequestError::Signature),
            "{hint}"
        );

        // The time, in both directions.
        assert_eq!(
            check(&good(&keys), &get(NOW + 61)),
            Err(SignedRequestError::Time),
            "{hint}"
        );
        assert_eq!(
            check(&good(&keys), &get(NOW - 61)),
            Err(SignedRequestError::Time),
            "{hint}"
        );

        // The address and the method, character for character.
        for url in [
            "https://studio.example.com/app/v1/shows/",
            "https://studio.example.com/app/v1/Shows",
            "http://studio.example.com/app/v1/shows",
            "https://studio.example.com/app/v1/shows?x=1",
            "https://studio.example.com:443/app/v1/shows",
        ] {
            let expected = ExpectedRequest { url, ..get(NOW) };
            assert_eq!(
                check(&good(&keys), &expected),
                Err(SignedRequestError::Url),
                "{url}: {hint}"
            );
        }
        let expected = ExpectedRequest {
            method: "POST",
            ..get(NOW)
        };
        assert_eq!(
            check(&good(&keys), &expected),
            Err(SignedRequestError::Method),
            "{hint}"
        );
        let lower = build(KIND_HTTP_AUTH, &[["u", URL], ["method", "get"]], NOW, &keys);
        assert_eq!(
            check(&lower, &get(NOW)),
            Err(SignedRequestError::Method),
            "{hint}"
        );
        let no_url = build(KIND_HTTP_AUTH, &[["method", "GET"]], NOW, &keys);
        assert_eq!(
            check(&no_url, &get(NOW)),
            Err(SignedRequestError::Url),
            "{hint}"
        );
    }

    #[test]
    fn a_request_with_a_body_needs_the_payload_hash() {
        let keys = keys_generate();
        let body = br#"{"code":"K7M2-Q9XD"}"#;
        let hash = hex_sha256(body);
        let post = |tags: &[[&str; 2]]| build(KIND_HTTP_AUTH, tags, NOW, &keys);
        let expected = ExpectedRequest {
            method: "POST",
            body: Some(body),
            ..get(NOW)
        };
        let ok = post(&[["u", URL], ["method", "POST"], ["payload", &hash]]);
        assert_eq!(check(&ok, &expected), Ok(keys.public_key()));
        let hint = "ADR 0110 item 13: a body needs a payload tag with its SHA-256. Fix signed_request::check.";
        let none = post(&[["u", URL], ["method", "POST"]]);
        assert_eq!(
            check(&none, &expected),
            Err(SignedRequestError::Payload),
            "{hint}"
        );
        let other = post(&[
            ["u", URL],
            ["method", "POST"],
            ["payload", &hex_sha256(b"x")],
        ]);
        assert_eq!(
            check(&other, &expected),
            Err(SignedRequestError::Payload),
            "{hint}"
        );
        let upper = post(&[
            ["u", URL],
            ["method", "POST"],
            ["payload", &hash.to_uppercase()],
        ]);
        assert_eq!(
            check(&upper, &expected),
            Err(SignedRequestError::Payload),
            "{hint}"
        );
        // A body with one changed byte does not match.
        let changed = ExpectedRequest {
            body: Some(br#"{"code":"K7M2-Q9XE"}"#),
            ..expected
        };
        assert_eq!(
            check(&ok, &changed),
            Err(SignedRequestError::Payload),
            "{hint}"
        );
        // `None` skips the check, for a request that carries its own event.
        let skip = ExpectedRequest {
            body: None,
            ..expected
        };
        assert_eq!(check(&none, &skip), Ok(keys.public_key()));
    }

    #[test]
    fn a_header_has_the_scheme_nostr_and_a_base64_event() {
        let keys = keys_generate();
        let event = good(&keys);
        let encoded = base64::engine::general_purpose::STANDARD.encode(event.as_json());
        let parsed = parse_header(&format!("Nostr {encoded}")).unwrap();
        assert_eq!(parsed, event);
        for value in [
            format!("Bearer {encoded}"),
            format!("nostr {encoded}"),
            encoded.clone(),
            "Nostr ".to_owned(),
            "Nostr !!!".to_owned(),
            format!(
                "Nostr {}",
                base64::engine::general_purpose::STANDARD.encode("{}")
            ),
            format!(
                "Nostr {}",
                base64::engine::general_purpose::STANDARD.encode(vec![b' '; MAX_EVENT_BYTES + 1])
            ),
        ] {
            assert!(parse_header(&value).is_none());
        }
    }
}
