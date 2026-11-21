//! The keyring of the computer for the device wrapper of the vault
//! (ADR 0235 item 4).

use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use studio_api::{DeviceKeyStore, DeviceSecret};

/// The longest wait for a read at the start. A keyring such as `KeePassXC`
/// can ask the user to unlock it first. After this time the vault stays
/// locked, and the unlock screen asks for the passphrase (ADR 0235 item 12).
const READ_LIMIT: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
pub(crate) struct KeyringStore;

impl KeyringStore {
    fn entry(device_id: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new("musicindex-studio", &format!("vault-{device_id}"))
            .map_err(|error| error.to_string())
    }
}

fn read(device_id: &str) -> Result<Option<DeviceSecret>, String> {
    match KeyringStore::entry(device_id)?.get_secret() {
        Ok(secret) => Ok(Some(DeviceSecret::new(secret))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

/// Run `read` while the process is dumpable, and then set the earlier
/// dumpable state again (ADR 0235 item 15). A keyring such as `KeePassXC`
/// reads `/proc/PID/exe` to identify the app, and the kernel lets it do that
/// only for a dumpable process. The vault is locked during the read, so no
/// vault key is in memory.
pub(crate) fn while_dumpable<T>(read: impl FnOnce() -> T) -> T {
    #[cfg(target_os = "linux")]
    {
        use rustix::process::{DumpableBehavior, dumpable_behavior, set_dumpable_behavior};
        let earlier = dumpable_behavior().ok();
        if earlier == Some(DumpableBehavior::NotDumpable) {
            let _ = set_dumpable_behavior(DumpableBehavior::Dumpable);
        }
        let value = read();
        if earlier == Some(DumpableBehavior::NotDumpable) {
            crate::harden_process();
        }
        value
    }
    #[cfg(not(target_os = "linux"))]
    read()
}

impl DeviceKeyStore for KeyringStore {
    fn get(&self, device_id: &str) -> Result<Option<DeviceSecret>, String> {
        let (send, receive) = mpsc::channel();
        let id = device_id.to_owned();
        while_dumpable(|| {
            std::thread::spawn(move || {
                // The receiver can be gone after the time limit.
                let _ = send.send(read(&id));
            });
            receive
                .recv_timeout(READ_LIMIT)
                .map_err(|_| "the keyring gave no answer in 60 seconds".to_owned())
        })?
    }

    fn set(&self, device_id: &str, secret: &[u8]) -> Result<(), String> {
        Self::entry(device_id)?
            .set_secret(secret)
            .map_err(|error| error.to_string())
    }

    fn delete(&self, device_id: &str) -> Result<(), String> {
        match Self::entry(device_id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub(crate) fn store() -> Arc<dyn DeviceKeyStore> {
    Arc::new(KeyringStore)
}
