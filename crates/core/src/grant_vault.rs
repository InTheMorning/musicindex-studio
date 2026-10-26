//! The grant vault (ADR 0058 items 1 to 5).
//!
//! A grant vault is a small vault with the format of ADR 0013. It holds a
//! copy of the secrets of the host of the feed XML of one show grant. It has
//! its own random vault key and no passphrase. It has one wrapper for each
//! connected app that has an app key: the vault key, encrypted with NIP-44
//! version 2 from a new random key to the public key of the app. Only the
//! token of that app opens the vault.

use nostr::prelude::{Keys, PublicKey, SecretKey};
use zeroize::Zeroizing;

use crate::apps::{ConnectedApp, app_keys};
use crate::nip46::{decrypt_text, encrypt_text};
use crate::vault::{OpenVault, VaultError, VaultFile};

/// Make a grant vault with a copy of the secrets `secret_ids` of the user
/// vault, and one wrapper for each app that has an app key. Gives the file
/// and the grant vault key as base64. The caller keeps the key in the user
/// vault and does not keep it anywhere else.
///
/// A secret ID that the user vault does not have is left out.
///
/// # Errors
///
/// Gives a [`VaultError`] when a record does not decrypt or when the random
/// source is not available.
pub fn make(
    user_vault: &OpenVault,
    secret_ids: &[&str],
    apps: &[ConnectedApp],
) -> Result<(VaultFile, Zeroizing<String>), VaultError> {
    let mut grant = OpenVault::create_with_no_wrapper()?;
    for id in secret_ids {
        if let Some(secret) = user_vault.get(id)? {
            grant.put(id, &secret)?;
        }
    }
    let key = grant.key_for_signer();
    let mut file = grant.file().clone();
    for app in apps {
        if app.pubkey.is_some() {
            add_wrapper(&mut file, &key, app)?;
        }
    }
    Ok((file, key))
}

/// Open a grant vault with the token of a connected app.
///
/// # Errors
///
/// Gives [`VaultErrorKind::WrongKey`](crate::vault::VaultErrorKind::WrongKey)
/// when the vault has no wrapper for the app or when the token is not the
/// token of the app.
pub fn open(file: VaultFile, app_id: i64, token: &str) -> Result<OpenVault, VaultError> {
    let (sender, ciphertext) = file.app_wrapper(app_id).ok_or_else(VaultError::wrong_key)?;
    let sender = PublicKey::from_hex(sender).map_err(|_| VaultError::wrong_key())?;
    let keys = app_keys(token);
    let key = decrypt_text(keys.secret_key(), &sender, ciphertext)
        .map_err(|_| VaultError::wrong_key())?;
    file.open_with_key(&key)
}

/// Add the wrapper of an app to a grant vault file. `grant_key` is the
/// grant vault key as base64. Studio needs no token for it, because it
/// uses only the public key of the app. An older wrapper of the same app
/// is replaced.
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
    use crate::apps::add_app;
    use crate::collection::Collection;
    use crate::vault::VaultErrorKind;

    const PASSPHRASE: &str = "correct horse battery";

    fn user_vault() -> OpenVault {
        let mut vault = OpenVault::create(PASSPHRASE).unwrap();
        vault.put("host-1", b"secret one").unwrap();
        vault.put("host-2", b"secret two").unwrap();
        vault
    }

    #[test]
    fn the_token_of_the_app_opens_the_grant_vault_and_no_other_token_does() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        let (other, other_token) = add_app(&collection, "Other").unwrap();
        let (file, key) = make(&user_vault(), &["host-1"], std::slice::from_ref(&app)).unwrap();

        let opened = open(file.clone(), app.id, &token).unwrap();
        assert_eq!(
            opened.get("host-1").unwrap().unwrap().as_slice(),
            b"secret one"
        );
        // The grant vault key opens the file too. It is the key in the wrapper.
        assert!(file.clone().open_with_key(&key).is_ok());

        // A different token, the token of an app with no wrapper, and an
        // unknown app ID.
        let error = open(file.clone(), app.id, &other_token).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
        let error = open(file.clone(), other.id, &other_token).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
        let error = open(file, app.id + 100, &token).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn the_grant_vault_has_only_the_records_of_the_secret_ids() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        let (file, _) = make(
            &user_vault(),
            &["host-2", "missing"],
            std::slice::from_ref(&app),
        )
        .unwrap();
        let opened = open(file, app.id, &token).unwrap();
        assert_eq!(opened.ids().collect::<Vec<_>>(), ["host-2"]);
        assert_eq!(opened.get("host-1").unwrap(), None);
        // A host with no secret gives an empty vault.
        let (file, _) = make(&user_vault(), &[], std::slice::from_ref(&app)).unwrap();
        assert_eq!(open(file, app.id, &token).unwrap().ids().count(), 0);
    }

    #[test]
    fn a_removed_app_does_not_open_the_vault_and_an_added_app_does() {
        let collection = Collection::open_in_memory().unwrap();
        let (one, token_one) = add_app(&collection, "One").unwrap();
        let (two, token_two) = add_app(&collection, "Two").unwrap();
        let (mut file, key) = make(&user_vault(), &["host-1"], std::slice::from_ref(&one)).unwrap();
        assert_eq!(file.app_ids(), [one.id]);

        assert!(open(file.clone(), two.id, &token_two).is_err());
        add_wrapper(&mut file, &key, &two).unwrap();
        assert_eq!(file.app_ids(), [one.id, two.id]);
        assert!(open(file.clone(), two.id, &token_two).is_ok());

        assert!(remove_wrapper(&mut file, one.id));
        assert!(!remove_wrapper(&mut file, one.id));
        let error = open(file.clone(), one.id, &token_one).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
        assert!(open(file, two.id, &token_two).is_ok());
    }

    #[test]
    fn an_app_with_no_key_gets_no_wrapper() {
        let collection = Collection::open_in_memory().unwrap();
        collection
            .db()
            .execute(
                "INSERT INTO connected_apps (name, token_hash, created_at) VALUES ('Old', 'h', 1)",
                [],
            )
            .unwrap();
        let old = crate::apps::apps(&collection).unwrap().remove(0);
        assert!(!old.has_key);
        let (mut file, key) = make(&user_vault(), &["host-1"], std::slice::from_ref(&old)).unwrap();
        assert_eq!(file.app_ids(), Vec::<i64>::new());
        let error = add_wrapper(&mut file, &key, &old).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn the_stored_form_has_no_token_no_key_and_no_secret() {
        let collection = Collection::open_in_memory().unwrap();
        let (app, token) = add_app(&collection, "Tool").unwrap();
        let (file, key) = make(&user_vault(), &["host-1"], &[app]).unwrap();
        let text = file.to_json();
        assert!(!text.contains(&token));
        assert!(!text.contains(key.as_str()));
        assert!(!text.contains("secret one"));
    }
}
