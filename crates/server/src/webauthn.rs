//! The check of `WebAuthn` responses (ADR 0056 items 20, 24 and 26). Only the
//! algorithm ES256 is accepted. The browser gives the public key as SPKI
//! through `getPublicKey()`, so no CBOR parser is needed.
//!
//! Each function checks in this order and stops at the first failure: the
//! `type`, the challenge, the `origin`, the SHA-256 of the relying party ID,
//! the flags UP and UV, the signature, and last the counter.

use p256::ecdsa::signature::Verifier as _;
use p256::ecdsa::{Signature, VerifyingKey};
use p256::pkcs8::DecodePublicKey as _;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// The COSE number of ES256.
pub const ES256: i64 = -7;

const FLAG_UP: u8 = 0x01;
const FLAG_UV: u8 = 0x04;
const FLAG_AT: u8 = 0x40;
/// The length of the fixed part of the authenticator data.
const FIXED_LEN: usize = 37;
/// The length of the AAGUID and of the credential ID length field.
const ATTESTED_HEAD: usize = 16 + 2;
/// The longest credential ID that `WebAuthn` allows.
const MAX_CREDENTIAL_ID: usize = 1023;

/// Why a check failed. The text names no secret and no user data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAuthnError {
    /// The client data is not JSON of the expected form.
    ClientData,
    /// The `type` is not the type that the ceremony needs.
    Type,
    /// The challenge is not the challenge that the server gave.
    Challenge,
    /// The `origin` is not the public origin.
    Origin,
    /// The authenticator data is too short or has an incorrect form.
    AuthenticatorData,
    /// The hash of the relying party ID is not correct.
    RelyingParty,
    /// The flag UP or the flag UV is not set.
    Flags,
    /// The algorithm is not ES256, or the key is not a P-256 key.
    Algorithm,
    /// The signature is not correct.
    Signature,
    /// The signature counter did not increase.
    Counter,
}

impl std::fmt::Display for WebAuthnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ClientData => "the client data is not correct",
            Self::Type => "the type of the response is not correct",
            Self::Challenge => "the challenge is not correct",
            Self::Origin => "the origin is not the public origin",
            Self::AuthenticatorData => "the authenticator data is not correct",
            Self::RelyingParty => "the relying party is not correct",
            Self::Flags => "the passkey did not confirm the user presence and the user",
            Self::Algorithm => "the passkey does not use ES256",
            Self::Signature => "the signature is not correct",
            Self::Counter => "the signature counter did not increase",
        })
    }
}

impl std::error::Error for WebAuthnError {}

/// What the server expects in a response.
#[derive(Debug, Clone, Copy)]
pub struct Expected<'a> {
    /// The public origin, as the browser writes it.
    pub origin: &'a str,
    /// The relying party ID: the host of the public origin.
    pub rp_id: &'a str,
    /// The challenge bytes that the server stored. The server never takes
    /// them from the response.
    pub challenge: &'a [u8],
}

/// A passkey that a registration made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registered {
    /// The credential ID.
    pub credential_id: Vec<u8>,
    /// The public key as SPKI.
    pub public_key: Vec<u8>,
    /// The signature counter of the registration.
    pub counter: u32,
}

#[derive(Deserialize)]
struct ClientData {
    #[serde(rename = "type")]
    kind: String,
    challenge: String,
    origin: String,
    #[serde(default, rename = "crossOrigin")]
    cross_origin: Option<bool>,
}

/// Check the client data: the `type`, the challenge and the `origin`.
fn check_client_data(
    client_data: &[u8],
    kind: &str,
    expected: &Expected<'_>,
) -> Result<(), WebAuthnError> {
    let data: ClientData =
        serde_json::from_slice(client_data).map_err(|_| WebAuthnError::ClientData)?;
    if data.kind != kind {
        return Err(WebAuthnError::Type);
    }
    // The server compares the bytes that it stored with the decoded text.
    let given = URL_SAFE_NO_PAD
        .decode(data.challenge.as_bytes())
        .map_err(|_| WebAuthnError::Challenge)?;
    if !same(&given, expected.challenge) {
        return Err(WebAuthnError::Challenge);
    }
    if data.origin != expected.origin || data.cross_origin == Some(true) {
        return Err(WebAuthnError::Origin);
    }
    Ok(())
}

/// Check the hash of the relying party ID and the flags UP and UV. Gives
/// the flags and the counter.
fn check_authenticator_data(
    authenticator_data: &[u8],
    expected: &Expected<'_>,
) -> Result<(u8, u32), WebAuthnError> {
    if authenticator_data.len() < FIXED_LEN {
        return Err(WebAuthnError::AuthenticatorData);
    }
    let hash = Sha256::digest(expected.rp_id.as_bytes());
    if !same(&authenticator_data[..32], hash.as_slice()) {
        return Err(WebAuthnError::RelyingParty);
    }
    let flags = authenticator_data[32];
    if flags & FLAG_UP == 0 || flags & FLAG_UV == 0 {
        return Err(WebAuthnError::Flags);
    }
    let counter = u32::from_be_bytes([
        authenticator_data[33],
        authenticator_data[34],
        authenticator_data[35],
        authenticator_data[36],
    ]);
    Ok((flags, counter))
}

/// Check the signature over the authenticator data and the hash of the
/// client data.
fn check_signature(
    key: &VerifyingKey,
    authenticator_data: &[u8],
    client_data: &[u8],
    signature_der: &[u8],
) -> Result<(), WebAuthnError> {
    let signature = Signature::from_der(signature_der).map_err(|_| WebAuthnError::Signature)?;
    let mut signed = authenticator_data.to_vec();
    signed.extend_from_slice(&Sha256::digest(client_data));
    key.verify(&signed, &signature)
        .map_err(|_| WebAuthnError::Signature)
}

/// Check a registration (`webauthn.create`). `algorithm` is the number that
/// `getPublicKeyAlgorithm()` gives, and `public_key_spki` is the value of
/// `getPublicKey()`. The caller must then check one assertion of the same
/// credential, because that assertion proves that the authenticator has the
/// private key.
///
/// # Errors
///
/// Gives the first [`WebAuthnError`] in the order of the module.
pub fn verify_registration(
    expected: &Expected<'_>,
    client_data: &[u8],
    authenticator_data: &[u8],
    public_key_spki: &[u8],
    algorithm: i64,
) -> Result<Registered, WebAuthnError> {
    check_client_data(client_data, "webauthn.create", expected)?;
    let (flags, counter) = check_authenticator_data(authenticator_data, expected)?;
    if flags & FLAG_AT == 0 {
        return Err(WebAuthnError::AuthenticatorData);
    }
    if algorithm != ES256 {
        return Err(WebAuthnError::Algorithm);
    }
    VerifyingKey::from_public_key_der(public_key_spki).map_err(|_| WebAuthnError::Algorithm)?;
    let attested = &authenticator_data[FIXED_LEN..];
    if attested.len() < ATTESTED_HEAD {
        return Err(WebAuthnError::AuthenticatorData);
    }
    let length = usize::from(u16::from_be_bytes([attested[16], attested[17]]));
    let id = attested
        .get(ATTESTED_HEAD..ATTESTED_HEAD + length)
        .filter(|id| !id.is_empty() && id.len() <= MAX_CREDENTIAL_ID)
        .ok_or(WebAuthnError::AuthenticatorData)?;
    Ok(Registered {
        credential_id: id.to_vec(),
        public_key: public_key_spki.to_vec(),
        counter,
    })
}

/// Check an assertion (`webauthn.get`). `public_key` is the SPKI that the
/// server stored. Gives the new counter. A counter must be larger than the
/// stored counter, unless both are zero.
///
/// # Errors
///
/// Gives the first [`WebAuthnError`] in the order of the module.
pub fn verify_assertion(
    expected: &Expected<'_>,
    client_data: &[u8],
    authenticator_data: &[u8],
    signature: &[u8],
    public_key: &[u8],
    stored_counter: u32,
) -> Result<u32, WebAuthnError> {
    check_client_data(client_data, "webauthn.get", expected)?;
    let (_, counter) = check_authenticator_data(authenticator_data, expected)?;
    let key =
        VerifyingKey::from_public_key_der(public_key).map_err(|_| WebAuthnError::Algorithm)?;
    check_signature(&key, authenticator_data, client_data, signature)?;
    if (counter != 0 || stored_counter != 0) && counter <= stored_counter {
        return Err(WebAuthnError::Counter);
    }
    Ok(counter)
}

/// The challenge text of a response, to find the stored challenge. The
/// server must still compare the stored bytes with the response, as
/// [`verify_registration`] and [`verify_assertion`] do.
#[must_use]
pub fn challenge_of(client_data: &[u8]) -> Option<Vec<u8>> {
    let data: ClientData = serde_json::from_slice(client_data).ok()?;
    URL_SAFE_NO_PAD.decode(data.challenge.as_bytes()).ok()
}

/// Compare two byte strings. The time does not depend on the first
/// difference.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0_u8, |d, (x, y)| d | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer as _;
    use p256::pkcs8::EncodePublicKey as _;

    const ORIGIN: &str = "https://studio.example.org";
    const RP_ID: &str = "studio.example.org";
    const CHALLENGE: [u8; 32] = [7; 32];
    const CREDENTIAL: [u8; 20] = [9; 20];

    fn signing_key() -> SigningKey {
        SigningKey::from_slice(&[3_u8; 32]).unwrap()
    }

    fn spki(key: &SigningKey) -> Vec<u8> {
        key.verifying_key()
            .to_public_key_der()
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    fn expected() -> Expected<'static> {
        Expected {
            origin: ORIGIN,
            rp_id: RP_ID,
            challenge: &CHALLENGE,
        }
    }

    fn client_data(kind: &str, challenge: &[u8], origin: &str) -> Vec<u8> {
        serde_json::json!({
            "type": kind,
            "challenge": URL_SAFE_NO_PAD.encode(challenge),
            "origin": origin,
            "crossOrigin": false,
        })
        .to_string()
        .into_bytes()
    }

    fn fixed(rp_id: &str, flags: u8, counter: u32) -> Vec<u8> {
        let mut data = Sha256::digest(rp_id.as_bytes()).to_vec();
        data.push(flags);
        data.extend_from_slice(&counter.to_be_bytes());
        data
    }

    fn attested(rp_id: &str, flags: u8, counter: u32) -> Vec<u8> {
        let mut data = fixed(rp_id, flags, counter);
        data.extend_from_slice(&[0; 16]);
        data.extend_from_slice(&u16::try_from(CREDENTIAL.len()).unwrap().to_be_bytes());
        data.extend_from_slice(&CREDENTIAL);
        // Stands for the COSE key. The check does not read it.
        data.extend_from_slice(&[0xa5, 0x01, 0x02]);
        data
    }

    fn sign(key: &SigningKey, authenticator_data: &[u8], client_data: &[u8]) -> Vec<u8> {
        let mut signed = authenticator_data.to_vec();
        signed.extend_from_slice(&Sha256::digest(client_data));
        let signature: Signature = key.sign(&signed);
        signature.to_der().as_bytes().to_vec()
    }

    #[test]
    fn a_correct_registration_gives_the_credential_and_the_key() {
        let key = signing_key();
        let registered = verify_registration(
            &expected(),
            &client_data("webauthn.create", &CHALLENGE, ORIGIN),
            &attested(RP_ID, 0x45, 0),
            &spki(&key),
            ES256,
        )
        .unwrap();
        assert_eq!(registered.credential_id, CREDENTIAL);
        assert_eq!(registered.public_key, spki(&key));
        assert_eq!(registered.counter, 0);
    }

    fn registration_error(data: &[u8], auth: &[u8], key: &[u8], algorithm: i64) -> WebAuthnError {
        verify_registration(&expected(), data, auth, key, algorithm).unwrap_err()
    }

    #[test]
    fn a_registration_with_a_wrong_part_is_refused() {
        let key = signing_key();
        let good = client_data("webauthn.create", &CHALLENGE, ORIGIN);
        let auth = attested(RP_ID, 0x45, 0);
        let spki = spki(&key);
        assert_eq!(
            registration_error(
                &client_data("webauthn.get", &CHALLENGE, ORIGIN),
                &auth,
                &spki,
                ES256
            ),
            WebAuthnError::Type
        );
        assert_eq!(
            registration_error(
                &client_data("webauthn.create", &[8; 32], ORIGIN),
                &auth,
                &spki,
                ES256
            ),
            WebAuthnError::Challenge
        );
        assert_eq!(
            registration_error(
                &client_data("webauthn.create", &CHALLENGE, "https://evil.example.org"),
                &auth,
                &spki,
                ES256
            ),
            WebAuthnError::Origin
        );
        assert_eq!(
            registration_error(&good, &attested("other.example.org", 0x45, 0), &spki, ES256),
            WebAuthnError::RelyingParty
        );
        // The flag UV is not set.
        assert_eq!(
            registration_error(&good, &attested(RP_ID, 0x41, 0), &spki, ES256),
            WebAuthnError::Flags
        );
        // The flag AT is not set.
        assert_eq!(
            registration_error(&good, &attested(RP_ID, 0x05, 0), &spki, ES256),
            WebAuthnError::AuthenticatorData
        );
        assert_eq!(
            registration_error(&good, &auth, &spki, -257),
            WebAuthnError::Algorithm
        );
        assert_eq!(
            registration_error(&good, &auth, b"not a key", ES256),
            WebAuthnError::Algorithm
        );
        assert_eq!(
            registration_error(&good, &auth[..60], &spki, ES256),
            WebAuthnError::AuthenticatorData
        );
        assert_eq!(
            registration_error(b"{", &auth, &spki, ES256),
            WebAuthnError::ClientData
        );
    }

    fn assertion(counter: u32, flags: u8) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let key = signing_key();
        let data = client_data("webauthn.get", &CHALLENGE, ORIGIN);
        let auth = fixed(RP_ID, flags, counter);
        let signature = sign(&key, &auth, &data);
        (data, auth, signature)
    }

    #[test]
    fn a_correct_assertion_gives_the_counter() {
        let (data, auth, signature) = assertion(5, 0x05);
        let key = spki(&signing_key());
        assert_eq!(
            verify_assertion(&expected(), &data, &auth, &signature, &key, 4),
            Ok(5)
        );
        let (data, auth, signature) = assertion(0, 0x05);
        assert_eq!(
            verify_assertion(&expected(), &data, &auth, &signature, &key, 0),
            Ok(0)
        );
    }

    #[test]
    fn an_assertion_with_a_wrong_part_is_refused() {
        let key = spki(&signing_key());
        let (data, auth, signature) = assertion(5, 0x05);
        let check = |data: &[u8], auth: &[u8], signature: &[u8], stored: u32| {
            verify_assertion(&expected(), data, auth, signature, &key, stored)
        };
        assert_eq!(
            check(
                &client_data("webauthn.create", &CHALLENGE, ORIGIN),
                &auth,
                &signature,
                0
            ),
            Err(WebAuthnError::Type)
        );
        assert_eq!(
            check(
                &client_data("webauthn.get", &[1; 32], ORIGIN),
                &auth,
                &signature,
                0
            ),
            Err(WebAuthnError::Challenge)
        );
        assert_eq!(
            check(
                &client_data("webauthn.get", &CHALLENGE, "http://studio.example.org"),
                &auth,
                &signature,
                0
            ),
            Err(WebAuthnError::Origin)
        );
        assert_eq!(
            check(&data, &fixed("other.example.org", 0x05, 5), &signature, 0),
            Err(WebAuthnError::RelyingParty)
        );
        // The flag UV is not set. The signature covers these data, so only
        // the flag check can refuse them.
        let key_pair = signing_key();
        let weak = fixed(RP_ID, 0x01, 5);
        let weak_signature = sign(&key_pair, &weak, &data);
        assert_eq!(
            check(&data, &weak, &weak_signature, 0),
            Err(WebAuthnError::Flags)
        );
        // A signature of other data.
        let (_, _, other) = assertion(6, 0x05);
        assert_eq!(
            check(&data, &auth, &other, 0),
            Err(WebAuthnError::Signature)
        );
        assert_eq!(
            check(&data, &auth, b"junk", 0),
            Err(WebAuthnError::Signature)
        );
        // The key of another signer.
        let other_key = spki(&SigningKey::from_slice(&[4_u8; 32]).unwrap());
        assert_eq!(
            verify_assertion(&expected(), &data, &auth, &signature, &other_key, 0),
            Err(WebAuthnError::Signature)
        );
        // The counter does not increase.
        assert_eq!(
            check(&data, &auth, &signature, 5),
            Err(WebAuthnError::Counter)
        );
        assert_eq!(
            check(&data, &auth, &signature, 9),
            Err(WebAuthnError::Counter)
        );
    }

    #[test]
    fn a_counter_of_zero_after_a_counter_above_zero_is_refused() {
        let key = spki(&signing_key());
        let (data, auth, signature) = assertion(0, 0x05);
        assert_eq!(
            verify_assertion(&expected(), &data, &auth, &signature, &key, 3),
            Err(WebAuthnError::Counter)
        );
    }

    #[test]
    fn the_challenge_comes_from_the_stored_bytes() {
        // A response with the right form but a different challenge text.
        let data = client_data("webauthn.get", &CHALLENGE, ORIGIN);
        assert_eq!(challenge_of(&data).as_deref(), Some(CHALLENGE.as_slice()));
        assert!(challenge_of(b"{}").is_none());
    }
}
