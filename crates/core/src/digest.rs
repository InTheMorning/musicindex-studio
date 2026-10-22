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
