//! The grant vault (ADR 0058 items 1 to 5, ADR 0110 items 20 to 23).
//!
//! A grant vault is a small vault with the format of ADR 0013. It holds a
//! copy of the secrets of the host of the feed XML of one show grant. It has
//! its own random vault key and no passphrase. It has one wrapper: the vault
//! key, encrypted with NIP-44 version 2 from a new random key to the public
//! key of the one app that the grant names. Only that app can decrypt the
//! wrapper. The app sends the vault key in the body of a signed request, and
//! [`open_with_key`] opens the vault. Studio keeps no copy of the key outside
//! the user vault.

use base64::Engine as _;
use nostr::prelude::{Keys, PublicKey, SecretKey};
use serde::Serialize;
use zeroize::Zeroizing;

use crate::apps::ConnectedApp;
use crate::nip46::encrypt_text;
use crate::vault::{OpenVault, VaultError, VaultFile};

/// The wrapper of one app, as the app needs it to decrypt the grant vault
/// key (ADR 0110 item 23).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Wrapper {
    /// The public key of the random sender key, as hexadecimal text.
    pub sender: String,
    /// The NIP-44 version 2 ciphertext of the grant vault key.
    pub ciphertext: String,
}

/// Make a grant vault with a copy of the secrets `secret_ids` of the user
/// vault, and the one wrapper of `app`, the app that the grant names (ADR
/// 0110 item 20). Gives the file and the grant vault key as base64. The
/// caller keeps the key in the user vault and does not keep it anywhere
/// else.
///
/// A secret ID that the user vault does not have is left out.
///
/// # Errors
///
/// Gives a [`VaultError`] when a record does not decrypt, when the app has
/// no public key, or when the random source is not available.
pub fn make(
    user_vault: &OpenVault,
    secret_ids: &[&str],
    app: &ConnectedApp,
) -> Result<(VaultFile, Zeroizing<String>), VaultError> {
    let mut grant = OpenVault::create_with_no_wrapper()?;
    for id in secret_ids {
        if let Some(secret) = user_vault.get(id)? {
            grant.put(id, &secret)?;
        }
    }
    let key = grant.key_for_signer();
    let mut file = grant.file().clone();
    add_wrapper(&mut file, &key, app)?;
    Ok((file, key))
}

/// Open a grant vault with the grant vault key, as base64. The app decrypts
/// the key from its wrapper and sends it in the body of a signed request.
///
/// # Errors
///
/// Gives [`VaultErrorKind::WrongKey`](crate::vault::VaultErrorKind::WrongKey)
/// when the key does not open the vault.
pub fn open_with_key(file: VaultFile, key: &str) -> Result<OpenVault, VaultError> {
    file.open_with_key(key)
}

/// The wrapper of one app in a grant vault file, or `None` when the file
/// has no wrapper for that app (ADR 0110 item 23).
#[must_use]
pub fn wrapper(file: &VaultFile, app_id: i64) -> Option<Wrapper> {
    file.app_wrapper(app_id)
        .map(|(sender, ciphertext)| Wrapper {
            sender: sender.to_owned(),
            ciphertext: ciphertext.to_owned(),
        })
}

/// Add the wrapper of an app to a grant vault file. `grant_key` is the
/// grant vault key as base64. Studio uses only the public key of the app.
/// An older wrapper of the same app is replaced.
///
/// # Errors
///
/// Gives [`VaultErrorKind::WrongKey`](crate::vault::VaultErrorKind::WrongKey)
/// when the app has no app key, and a [`VaultError`] when the random source
/// is not available.
pub fn add_wrapper(
    file: &mut VaultFile,
    grant_key: &str,
    app: &ConnectedApp,
) -> Result<(), VaultError> {
    let pubkey = app
        .pubkey
        .as_deref()
        .and_then(|hex| PublicKey::from_hex(hex).ok())
        .ok_or_else(VaultError::wrong_key)?;
    let sender = random_keys()?;
    let ciphertext = encrypt_text(sender.secret_key(), &pubkey, grant_key)
        .map_err(|_| VaultError::wrong_key())?;
    file.set_app_wrapper(app.id, &sender.public_key().to_hex(), &ciphertext);
    Ok(())
}

/// A wrapper of a random key for an app, with no grant vault file. A grant
/// on a host with no secret has no grant vault (ADR 0058 item 2), but the
/// app still takes the wrapper step before a write (ADR 0110 item 23). The
/// key is random, Studio keeps no copy of it, and the write ignores it.
///
/// # Errors
///
/// Gives [`VaultErrorKind::WrongKey`](crate::vault::VaultErrorKind::WrongKey)
/// when the app has no app key, and a [`VaultError`] when the random source
/// is not available.
pub fn placeholder_wrapper(app: &ConnectedApp) -> Result<Wrapper, VaultError> {
    let mut bytes = Zeroizing::new([0_u8; 32]);
    getrandom::fill(bytes.as_mut_slice())
        .map_err(|_| VaultError::corrupt("the random source is not available"))?;
    let key = Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(bytes.as_slice()));
    let pubkey = app
        .pubkey
        .as_deref()
        .and_then(|hex| PublicKey::from_hex(hex).ok())
        .ok_or_else(VaultError::wrong_key)?;
    let sender = random_keys()?;
    let ciphertext =
        encrypt_text(sender.secret_key(), &pubkey, &key).map_err(|_| VaultError::wrong_key())?;
    Ok(Wrapper {
        sender: sender.public_key().to_hex(),
        ciphertext,
    })
}

/// Remove the wrapper of an app from a grant vault file. Gives `true` when
/// the file had it.
pub fn remove_wrapper(file: &mut VaultFile, app_id: i64) -> bool {
    file.remove_app_wrapper(app_id)
}

/// A new random key, for one wrapper.
fn random_keys() -> Result<Keys, VaultError> {
    loop {
        let mut bytes = Zeroizing::new([0_u8; 32]);
        getrandom::fill(bytes.as_mut_slice())
            .map_err(|_| VaultError::corrupt("the random source is not available"))?;
        // A value outside the curve order is very rare. Another try is correct.
        if let Ok(secret) = SecretKey::from_slice(bytes.as_ref()) {
            return Ok(Keys::new(secret));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::{pair_at, start_pairing_at};
    use crate::collection::Collection;
    use crate::nip46::decrypt_text;
    use crate::vault::VaultErrorKind;

    const PASSPHRASE: &str = "correct horse battery";

    fn user_vault() -> OpenVault {
        let mut vault = OpenVault::create(PASSPHRASE).unwrap();
        vault.put("host-1", b"secret one").unwrap();
        vault.put("host-2", b"secret two").unwrap();
        vault
    }

    /// An active app and the keys that it holds.
    fn active_app(collection: &Collection, name: &str) -> (ConnectedApp, Keys) {
        let keys = random_keys().unwrap();
        let (_, code) = start_pairing_at(collection, name, 1).unwrap();
        let app = pair_at(
            collection,
            &code,
            &keys.public_key().to_hex(),
            "https://live.example.com",
            2,
        )
        .unwrap();
        (app, keys)
    }

    /// What an app does: decrypt its wrapper with its private key.
    fn decrypt_key(file: &VaultFile, app: &ConnectedApp, keys: &Keys) -> Option<String> {
        let found = wrapper(file, app.id)?;
        let sender = PublicKey::from_hex(&found.sender).ok()?;
        decrypt_text(keys.secret_key(), &sender, &found.ciphertext)
            .ok()
            .map(|key| key.as_str().to_owned())
    }

    #[test]
    fn the_key_in_the_wrapper_opens_the_grant_vault_and_no_other_key_does() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, keys) = active_app(&collection, "Tool");
        let (file, key) = make(&user_vault(), &["host-1"], &app).unwrap();

        let from_wrapper = decrypt_key(&file, &app, &keys).unwrap();
        assert_eq!(from_wrapper.as_str(), key.as_str());
        let opened = open_with_key(file.clone(), &from_wrapper).unwrap();
        assert_eq!(
            opened.get("host-1").unwrap().unwrap().as_slice(),
            b"secret one"
        );

        // The key of a different app does not decrypt the wrapper.
        let (other, other_keys) = active_app(&collection, "Other");
        assert_eq!(decrypt_key(&file, &app, &other_keys), None);
        assert_eq!(decrypt_key(&file, &other, &other_keys), None);
        // A wrong grant vault key does not open the file.
        let wrong = OpenVault::create_with_no_wrapper()
            .unwrap()
            .key_for_signer();
        let error = open_with_key(file, &wrong).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn a_grant_vault_has_only_the_wrapper_of_the_app_that_the_grant_names() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, _) = active_app(&collection, "Tool");
        let (other, _) = active_app(&collection, "Other");
        let (file, _) = make(&user_vault(), &["host-1"], &app).unwrap();
        assert_eq!(
            file.app_ids(),
            [app.id],
            "ADR 0110 item 20: a grant vault must have only the wrapper of the app that the grant names. Do not add a wrapper for another app."
        );
        assert!(wrapper(&file, app.id).is_some());
        assert_eq!(wrapper(&file, other.id), None);
        assert_eq!(wrapper(&file, app.id + 100), None);
    }

    #[test]
    fn the_grant_vault_has_only_the_records_of_the_secret_ids() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, keys) = active_app(&collection, "Tool");
        let (file, _) = make(&user_vault(), &["host-2", "missing"], &app).unwrap();
        let key = decrypt_key(&file, &app, &keys).unwrap();
        let opened = open_with_key(file, &key).unwrap();
        assert_eq!(opened.ids().collect::<Vec<_>>(), ["host-2"]);
        assert_eq!(opened.get("host-1").unwrap(), None);
        // A host with no secret gives an empty vault.
        let (file, _) = make(&user_vault(), &[], &app).unwrap();
        let key = decrypt_key(&file, &app, &keys).unwrap();
        assert_eq!(open_with_key(file, &key).unwrap().ids().count(), 0);
    }

    #[test]
    fn a_wrapper_can_be_replaced_and_removed() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, keys) = active_app(&collection, "One");
        let (mut file, key) = make(&user_vault(), &["host-1"], &app).unwrap();
        add_wrapper(&mut file, &key, &app).unwrap();
        assert_eq!(file.app_ids(), [app.id]);
        assert_eq!(decrypt_key(&file, &app, &keys).unwrap(), key.as_str());
        assert!(remove_wrapper(&mut file, app.id));
        assert!(!remove_wrapper(&mut file, app.id));
        assert_eq!(wrapper(&file, app.id), None);
        // The key still opens the vault. Only the wrapper is gone.
        assert!(open_with_key(file, &key).is_ok());
    }

    #[test]
    fn an_app_with_no_key_gets_no_wrapper() {
        let collection = Collection::open_in_memory().unwrap();
        let (waiting, _) = start_pairing_at(&collection, "Waiting", 1).unwrap();
        let error = make(&user_vault(), &["host-1"], &waiting).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn the_stored_form_has_no_key_and_no_secret() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, _) = active_app(&collection, "Tool");
        let (file, key) = make(&user_vault(), &["host-1"], &app).unwrap();
        let text = file.to_json();
        assert!(!text.contains(key.as_str()));
        assert!(!text.contains("secret one"));
    }

    #[cfg(unix)]
    #[test]
    fn a_grant_vault_file_has_mode_0600() {
        use std::os::unix::fs::PermissionsExt as _;

        let collection = Collection::open_in_memory().unwrap();
        let (app, _) = active_app(&collection, "Tool");
        let (file, _) = make(&user_vault(), &["host-1"], &app).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("1.json");
        file.save(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "ADR 0110 item 22: a grant vault file must have mode 0600. Save it with VaultFile::save."
        );
    }
}
