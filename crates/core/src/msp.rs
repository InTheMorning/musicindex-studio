//! Import from an MSP 2.0 backup file (ADR 0005, ADR 0020).
//!
//! An MSP 2.0 backup holds one hosted feed and its edit token. Studio keeps
//! the feed in the collection and the edit token in the vault (ADR 0003).

use serde_json::Value;
use zeroize::Zeroizing;

/// The address of MSP 2.0 hosted feeds, as MSP 2.0 `buildHostedUrl` makes it.
const HOSTED_BASE: &str = "https://musicsideproject.com/api/hosted";

/// The facts of one MSP 2.0 backup.
pub struct MspBackup {
    /// The MSP 2.0 feed ID.
    pub feed_id: String,
    /// The public address of the hosted feed.
    pub feed_url: String,
    /// The edit token. Its memory is cleared when it goes out of scope.
    pub edit_token: Zeroizing<String>,
}

impl std::fmt::Debug for MspBackup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MspBackup")
            .field("feed_id", &self.feed_id)
            .field("feed_url", &self.feed_url)
            .finish_non_exhaustive()
    }
}

impl MspBackup {
    /// Read a backup. It accepts the field names that MSP 2.0 accepts:
    /// `feedId` or `feed_id`, `editToken` or `edit_token`, and the
    /// `msp_hosted_feed_backup` object of older files.
    ///
    /// # Errors
    ///
    /// Gives a message when the backup has no feed ID or no edit token.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        let text = |keys: &[&str]| {
            keys.iter().find_map(|key| {
                let mut node = value;
                for part in key.split('.') {
                    node = node.get(part)?;
                }
                node.as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
            })
        };
        let feed_id = text(&["feedId", "feed_id", "msp_hosted_feed_backup.feed_id"])
            .ok_or("the backup has no feed ID")?;
        let edit_token = text(&[
            "editToken",
            "edit_token",
            "msp_hosted_feed_backup.edit_token",
        ])
        .ok_or("the backup has no edit token")?;
        if !feed_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("the feed ID of the backup is not valid".to_owned());
        }
        let feed_url = text(&["feedUrl", "feed_url"])
            .unwrap_or_else(|| format!("{HOSTED_BASE}/{feed_id}.xml"));
        Ok(Self {
            feed_id,
            feed_url,
            edit_token: Zeroizing::new(edit_token),
        })
    }

    /// The ID of the vault record that holds the edit token.
    #[must_use]
    pub fn secret_ref(&self) -> String {
        format!("msp:{}", self.feed_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn each_field_name_that_msp_accepts_reads() {
        let new = MspBackup::from_json(&json!({
            "feedUrl": "https://musicsideproject.com/api/hosted/abc.xml",
            "feedId": "abc", "editToken": "t1"
        }))
        .unwrap();
        assert_eq!(
            new.feed_url,
            "https://musicsideproject.com/api/hosted/abc.xml"
        );
        assert_eq!(new.secret_ref(), "msp:abc");
        let old = MspBackup::from_json(
            &json!({ "msp_hosted_feed_backup": { "feed_id": "x-1", "edit_token": "t2" } }),
        )
        .unwrap();
        assert_eq!(
            old.feed_url,
            "https://musicsideproject.com/api/hosted/x-1.xml"
        );
        assert_eq!(old.edit_token.as_str(), "t2");
        assert!(!format!("{old:?}").contains("t2"));
    }

    #[test]
    fn a_backup_without_a_token_or_with_a_bad_id_gives_an_error() {
        assert!(MspBackup::from_json(&json!({ "feedId": "abc" })).is_err());
        assert!(MspBackup::from_json(&json!({ "feedId": "../x", "editToken": "t" })).is_err());
    }
}
