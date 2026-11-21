use super::*;
use crate::{DeviceKeyStore, DeviceSecret};
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

/// A keyring in memory. Field 1 makes each call fail. Field 2 counts the
/// reads.
#[derive(Debug, Clone, Default)]
struct MemoryStore(
    Arc<Mutex<HashMap<String, Vec<u8>>>>,
    Arc<AtomicBool>,
    Arc<AtomicUsize>,
);

impl DeviceKeyStore for MemoryStore {
    fn get(&self, id: &str) -> Result<Option<DeviceSecret>, String> {
        self.2.fetch_add(1, Ordering::SeqCst);
        if self.1.load(Ordering::SeqCst) {
            return Err("store unavailable".to_owned());
        }
        Ok(self
            .0
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(DeviceSecret::new))
    }

    fn set(&self, id: &str, secret: &[u8]) -> Result<(), String> {
        if self.1.load(Ordering::SeqCst) {
            return Err("store unavailable".to_owned());
        }
        self.0
            .lock()
            .unwrap()
            .insert(id.to_owned(), secret.to_vec());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        if self.1.load(Ordering::SeqCst) {
            return Err("store unavailable".to_owned());
        }
        self.0.lock().unwrap().remove(id);
        Ok(())
    }
}

fn desktop_app(store: MemoryStore) -> App {
    let data = tempfile::tempdir().unwrap();
    let feeds = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_device_store(Arc::new(store));
    App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds,
    }
}

#[test]
fn an_ask_start_does_not_report_a_device_unlock_failure() {
    let dir = tempfile::tempdir().unwrap();
    let mut workspace = Workspace::open(dir.path())
        .unwrap()
        .with_device_store(Arc::new(MemoryStore::default()));
    workspace.unlock_from_device_store();
    assert!(!workspace.start_unlock_failed());
}

#[tokio::test]
async fn device_start_setting_round_trips_and_opens_without_a_recent_unlock() {
    let store = MemoryStore::default();
    let app = desktop_app(store.clone());
    let (status, settings) = call(&app, "GET", "/api/v1/settings/lock", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["start_choices"], json!(["ask", "device"]));
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({"passphrase": PASSPHRASE, "recovery_key": false})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, changed) = call(
        &app,
        "PUT",
        "/api/v1/settings/lock",
        Some(json!({"minutes": 0, "start": "device"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{changed}");
    assert_eq!(changed["start"], "device");
    assert_eq!(store.0.lock().unwrap().len(), 1);
    let reopened = Workspace::open(app.data.path())
        .unwrap()
        .with_device_store(Arc::new(store.clone()));
    let mut reopened = reopened;
    reopened.unlock_from_device_store();
    assert_eq!(reopened.vault_state(), crate::VaultState::Open);
    assert!(!reopened.start_unlock_failed());
    drop(reopened);
    store.1.store(true, Ordering::SeqCst);
    let mut unavailable = Workspace::open(app.data.path())
        .unwrap()
        .with_device_store(Arc::new(store.clone()));
    unavailable.unlock_from_device_store();
    assert_eq!(unavailable.vault_state(), crate::VaultState::Locked);
    assert!(unavailable.start_unlock_failed());
}

#[tokio::test]
async fn server_lock_settings_have_no_start_choices() {
    let app = app_with_rules(1024);
    let (status, settings) = call(&app, "GET", "/api/v1/settings/lock", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["start_choices"], json!([]));
    let minutes = settings["choices"][0].clone();
    let (status, answer) = call(
        &app,
        "PUT",
        "/api/v1/settings/lock",
        Some(json!({"minutes": minutes, "start": "ask"})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the UI sends the saved start value with a lock time: {answer}"
    );
}

#[tokio::test]
async fn a_failed_keyring_write_leaves_the_vault_file_unchanged() {
    let store = MemoryStore::default();
    let app = desktop_app(store.clone());
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault",
        Some(json!({"passphrase": PASSPHRASE, "recovery_key": false})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let path = app.data.path().join("vault.json");
    let before = std::fs::read(&path).unwrap();
    store.1.store(true, Ordering::SeqCst);
    let (status, answer) = call(
        &app,
        "PUT",
        "/api/v1/settings/lock",
        Some(json!({"minutes": 0, "start": "device"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(answer["error"]["code"], "keyring_failed");
    assert_eq!(answer["error"]["details"]["detail"], "store unavailable");
    assert_eq!(std::fs::read(path).unwrap(), before);
}

const DEVICE_ON: &str = "/api/v1/settings/lock";

async fn make_device_vault(app: &App) {
    let (status, _) = call(
        app,
        "POST",
        "/api/v1/vault",
        Some(json!({"passphrase": PASSPHRASE, "recovery_key": false})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn unlock_on_this_computer_needs_a_recent_unlock_and_a_same_value_needs_none() {
    let (clock, offset) = test_clock();
    let store = MemoryStore::default();
    let data = tempfile::tempdir().unwrap();
    let workspace = Workspace::open(data.path())
        .unwrap()
        .with_clock(clock)
        .with_device_store(Arc::new(store.clone()));
    let app = App {
        router: local_router(workspace, Access::with_token(TOKEN, 4100)),
        data,
        feeds: tempfile::tempdir().unwrap(),
    };
    make_device_vault(&app).await;
    offset.store(6 * 60, Ordering::SeqCst);
    let body = Some(json!({"minutes": 0, "start": "device"}));
    let (status, answer) = call(&app, "PUT", DEVICE_ON, body.clone()).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "ADR 0235 item 3: the change to device needs a recent unlock: {answer}"
    );
    assert!(store.0.lock().unwrap().is_empty());
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({"passphrase": PASSPHRASE})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(&app, "PUT", DEVICE_ON, body).await;
    assert_eq!(status, StatusCode::OK);
    let keys: Vec<String> = store.0.lock().unwrap().keys().cloned().collect();
    // A change of the lock time sends the saved start value again. Later
    // than 5 minutes, it must work and keep the same key.
    offset.store(20 * 60, Ordering::SeqCst);
    let (status, answer) = call(
        &app,
        "PUT",
        DEVICE_ON,
        Some(json!({"minutes": 5, "start": "device"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let after: Vec<String> = store.0.lock().unwrap().keys().cloned().collect();
    assert_eq!(keys, after, "the same start value must make no new key");
}

#[tokio::test]
async fn a_wrong_key_gives_the_notice_and_a_lock_does_not_read_the_keyring() {
    let store = MemoryStore::default();
    let app = desktop_app(store.clone());
    make_device_vault(&app).await;
    let (status, _) = call(
        &app,
        "PUT",
        DEVICE_ON,
        Some(json!({"minutes": 0, "start": "device"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    for secret in store.0.lock().unwrap().values_mut() {
        *secret = vec![1; 32];
    }
    let mut wrong = Workspace::open(app.data.path())
        .unwrap()
        .with_device_store(Arc::new(store.clone()));
    wrong.unlock_from_device_store();
    assert_eq!(wrong.vault_state(), crate::VaultState::Locked);
    assert!(
        wrong.start_unlock_failed(),
        "ADR 0235 item 12: a wrong key must give the notice"
    );
    let reads = store.2.load(Ordering::SeqCst);
    call(&app, "POST", "/api/v1/vault/lock", None).await;
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/vault/unlock",
        Some(json!({"passphrase": PASSPHRASE})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        store.2.load(Ordering::SeqCst),
        reads,
        "ADR 0235 item 13: a lock and an unlock must not read the keyring"
    );
}
