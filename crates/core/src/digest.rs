//! SHA-256 in hexadecimal text.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

/// The SHA-256 of the parts, one after the other, as lowercase hexadecimal.
pub(crate) fn sha256_hex(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part);
    }
    hash.finalize()
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// The version marker of file content: its SHA-256 (ADR 0015).
pub(crate) fn content_version(bytes: &[u8]) -> crate::connector::Version {
    crate::connector::Version::new(format!("sha256:{}", sha256_hex(&[bytes])))
}
