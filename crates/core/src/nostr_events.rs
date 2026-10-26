//! Nostr events for the nsite and Blossom connectors.
//!
//! `nostr` has its default features only (ADR 0044, ADR 0051). The core
//! signs an event in the one function of this module.

use nostr::prelude::{Event, Keys, UnsignedEvent};
use secp256k1::Secp256k1;

use crate::connector::{ConnectorError, ConnectorErrorKind};

/// Sign an event with a key.
///
/// The function uses 32 random bytes from the operating system as the
/// auxiliary data of BIP-340. It signs the event ID.
///
/// # Errors
///
/// Gives `Io` when the random source is not available, or when the event
/// does not pass the check of its ID and signature. The message holds no
/// part of the key.
pub fn sign_with_keys(unsigned: UnsignedEvent, keys: &Keys) -> Result<Event, ConnectorError> {
    let secp = Secp256k1::signing_only();
    let mut aux = [0_u8; 32];
    getrandom::fill(&mut aux).map_err(|_| {
        ConnectorError::new(ConnectorErrorKind::Io, "the random source is not available")
    })?;
    let id = unsigned.compute_id();
    let signature = keys.sign_schnorr_with_aux_rand(&secp, id.as_bytes(), &aux);
    unsigned.add_signature(signature).map_err(|_| {
        ConnectorError::new(ConnectorErrorKind::Io, "the event signature is not valid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::prelude::{Kind, SecretKey, Tag, Timestamp};

    fn keys() -> Keys {
        Keys::new(SecretKey::from_slice(&[1; 32]).expect("a secret key"))
    }

    #[test]
    fn a_signed_event_passes_the_check() {
        let keys = keys();
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(1_700_000_000_u64),
            Kind::Custom(35128),
            [Tag::identifier("site")],
            "",
        );
        let id = unsigned.compute_id();
        let event = sign_with_keys(unsigned, &keys).expect("a signed event");
        assert!(event.verify().is_ok());
        assert_eq!(event.id, id);
        assert_eq!(event.pubkey, keys.public_key());
    }
}
