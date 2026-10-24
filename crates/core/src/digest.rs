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

/// The SHA-256 of a file, as lowercase hexadecimal. The function reads the
/// file in blocks, so a large file does not fill the memory.
pub(crate) fn sha256_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut block = vec![0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut block)?;
        if read == 0 {
            break;
        }
        hash.update(&block[..read]);
    }
    Ok(hash
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        }))
}

/// The version marker of file content: its SHA-256 (ADR 0015).
pub(crate) fn content_version(bytes: &[u8]) -> crate::connector::Version {
    crate::connector::Version::new(format!("sha256:{}", sha256_hex(&[bytes])))
}
