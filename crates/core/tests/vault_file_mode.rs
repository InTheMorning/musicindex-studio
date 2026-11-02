//! The mode of a saved vault file (ADR 0104 item 2).
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use studio_core::vault::OpenVault;

#[test]
fn a_saved_vault_file_has_the_mode_0600() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.json");
    let vault = OpenVault::create("correct horse battery staple 42").unwrap();
    let old = rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o022));
    let saved = vault.file().save(&path);
    rustix::process::umask(old);
    saved.unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "ADR 0104 item 2: a vault file must have the mode 0600. Open the temporary file in VaultFile::save with OpenOptionsExt::mode(0o600)."
    );
}
