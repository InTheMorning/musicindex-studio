//! The keysend lookup of a Lightning address (ADR 0095 items 1 to 5).
//!
//! A Lightning address of the type `lnaddress` can name the same wallet as
//! a `node` recipient. The host of the address gives the node key and the
//! custom data at `/.well-known/keysend/<name>`. [`refresh`] requests and
//! keeps these lookups. [`wallet_of`] groups the addresses of one wallet.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::change_set::RecipientKey;
use crate::collection::{Collection, CollectionError};
use crate::profiles::within;
use crate::web::{Web, WebErrorKind};

/// The time limit of one lookup request.
pub const LOOKUP_LIMIT: Duration = Duration::from_secs(5);

/// The age in seconds after which a `Found` or `None` lookup is due.
pub const REFRESH_AFTER: i64 = 86_400;

/// The age in seconds after which a `Failed` lookup is due.
pub const RETRY_AFTER: i64 = 3_600;

/// The largest body of a lookup response, in bytes.
const MAX_BODY: u64 = 16 * 1024;

/// What the collection knows about the lookup of an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupStatus {
    /// The host gave keysend data.
    Found,
    /// The host gave no keysend data.
    None,
    /// The request failed, and the collection has no data.
    Failed,
}

impl LookupStatus {
    /// The text in the collection.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::None => "none",
            Self::Failed => "failed",
        }
    }

    /// Read the text in the collection.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "found" => Some(Self::Found),
            "none" => Some(Self::None),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// The stored lookup of one Lightning address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lookup {
    /// The Lightning address, in lowercase.
    pub address: String,
    /// The node key.
    pub pubkey: Option<String>,
    /// The `customKey` of the first custom data entry.
    pub custom_key: Option<String>,
    /// The `customValue` of the first custom data entry.
    pub custom_value: Option<String>,
    /// The status of the lookup.
    pub status: LookupStatus,
    /// The Unix time of the last request.
    pub checked_at: i64,
}

impl Lookup {
    /// The key of the wallet, for a `Found` lookup. Else `None`.
    #[must_use]
    pub fn key(&self) -> Option<RecipientKey> {
        if self.status != LookupStatus::Found {
            return None;
        }
        Some(RecipientKey {
            address: self.pubkey.clone()?,
            custom_key: self.custom_key.clone(),
            custom_value: self.custom_value.clone(),
        })
    }
}

/// The result of one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupResult {
    /// The host gave keysend data.
    Found {
        /// The node key.
        pubkey: String,
        /// The `customKey`.
        custom_key: Option<String>,
        /// The `customValue`.
        custom_value: Option<String>,
    },
    /// The host gave no keysend data.
    None,
    /// The request failed.
    Failed,
}

/// The lookup address of a Lightning address, or `None` when the text is
/// not a Lightning address.
#[must_use]
pub fn lookup_url(scheme: &str, address: &str) -> Option<String> {
    let address = address.trim().to_lowercase();
    let (name, domain) = address.split_once('@')?;
    let bad =
        |text: &str| text.is_empty() || text.contains('@') || text.contains(char::is_whitespace);
    if bad(name) || bad(domain) || name.contains('/') || domain.contains('/') {
        return None;
    }
    Some(format!("{scheme}://{domain}/.well-known/keysend/{name}"))
}

fn text_of(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

/// Read a keysend response. Gives the node key, the `customKey` and the
/// `customValue` of the first custom data entry. The tag is not examined.
#[must_use]
pub fn parse_lookup(bytes: &[u8]) -> Option<(String, Option<String>, Option<String>)> {
    let json: Value = serde_json::from_slice(bytes).ok()?;
    if json
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status.eq_ignore_ascii_case("error"))
    {
        return None;
    }
    let pubkey = json.get("pubkey")?.as_str()?.trim();
    if pubkey.is_empty() {
        return None;
    }
    let entry = json
        .get("customData")
        .and_then(Value::as_array)
        .and_then(|list| list.first());
    let custom_key = text_of(entry.and_then(|e| e.get("customKey")));
    let custom_value = text_of(entry.and_then(|e| e.get("customValue")));
    Some((pubkey.to_owned(), custom_key, custom_value))
}

/// Request the keysend data of a Lightning address.
#[must_use]
pub fn request(web: &Web, scheme: &str, address: &str) -> LookupResult {
    let Some(url) = lookup_url(scheme, address) else {
        return LookupResult::Failed;
    };
    match web.get(&url, MAX_BODY) {
        Ok(file) => parse_lookup(&file.bytes).map_or(
            LookupResult::None,
            |(pubkey, custom_key, custom_value)| LookupResult::Found {
                pubkey,
                custom_key,
                custom_value,
            },
        ),
        Err(error)
            if error.kind() == WebErrorKind::Status
                && error.status().is_some_and(|s| (400..500).contains(&s)) =>
        {
            LookupResult::None
        }
        Err(_) => LookupResult::Failed,
    }
}

/// True when the lookup must be requested again at `now`. An address with
/// no lookup is due.
#[must_use]
pub fn is_due(lookup: Option<&Lookup>, now: i64) -> bool {
    let Some(lookup) = lookup else {
        return true;
    };
    let after = match lookup.status {
        LookupStatus::Found | LookupStatus::None => REFRESH_AFTER,
        LookupStatus::Failed => RETRY_AFTER,
    };
    now.saturating_sub(lookup.checked_at) >= after
}

/// Request and store the lookups of `addresses` that are due. Gives each
/// stored lookup of the addresses, by lowercase address.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn refresh(
    collection: &Collection,
    web: &Web,
    scheme: &str,
    addresses: &[String],
    now: i64,
) -> Result<HashMap<String, Lookup>, CollectionError> {
    let mut wanted: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for address in addresses {
        let address = address.trim().to_lowercase();
        if lookup_url(scheme, &address).is_some() && seen.insert(address.clone()) {
            wanted.push(address);
        }
    }
    let stored: HashMap<String, Lookup> = collection
        .keysend_lookups()?
        .into_iter()
        .map(|lookup| (lookup.address.clone(), lookup))
        .collect();
    let due: Vec<&String> = wanted
        .iter()
        .filter(|address| is_due(stored.get(*address), now))
        .collect();
    let results: Vec<(&String, LookupResult)> = thread::scope(|scope| {
        let handles: Vec<_> = due
            .iter()
            .map(|address| {
                scope.spawn(move || {
                    let web = web.clone();
                    let scheme = scheme.to_owned();
                    let text = (*address).clone();
                    let result = within(LOOKUP_LIMIT, move || request(&web, &scheme, &text))
                        .unwrap_or(LookupResult::Failed);
                    (*address, result)
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    });
    for (address, result) in results {
        let row = |pubkey, custom_key, custom_value, status| Lookup {
            address: address.clone(),
            pubkey,
            custom_key,
            custom_value,
            status,
            checked_at: now,
        };
        match result {
            LookupResult::Found {
                pubkey,
                custom_key,
                custom_value,
            } => collection.set_keysend_lookup(&row(
                Some(pubkey),
                custom_key,
                custom_value,
                LookupStatus::Found,
            ))?,
            LookupResult::None => {
                collection.set_keysend_lookup(&row(None, None, None, LookupStatus::None))?;
            }
            LookupResult::Failed if stored.contains_key(address) => {
                collection.touch_keysend_lookup(address, now)?;
            }
            LookupResult::Failed => {
                collection.set_keysend_lookup(&row(None, None, None, LookupStatus::Failed))?;
            }
        }
    }
    Ok(collection
        .keysend_lookups()?
        .into_iter()
        .filter(|lookup| seen.contains(&lookup.address))
        .map(|lookup| (lookup.address.clone(), lookup))
        .collect())
}

fn same_key(a: &RecipientKey, b: &RecipientKey) -> bool {
    let parts = |k: &RecipientKey| {
        (
            k.address.clone(),
            k.custom_key.clone().unwrap_or_default(),
            k.custom_value.clone().unwrap_or_default(),
        )
    };
    parts(a) == parts(b)
}

/// The key of the wallet of `key`: the key of the `Found` lookup of its
/// address, or else `key` itself.
#[must_use]
pub fn wallet_key<S: BuildHasher>(
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> RecipientKey {
    lookups
        .get(&key.address.trim().to_lowercase())
        .and_then(Lookup::key)
        .unwrap_or_else(|| key.clone())
}

/// The members of the wallet of `key`: the key itself, then each Lightning
/// address whose `Found` lookup gives the key, from A to Z.
#[must_use]
pub fn members<S: BuildHasher>(
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> Vec<RecipientKey> {
    let mut others: Vec<&Lookup> = lookups
        .values()
        .filter(|lookup| {
            lookup.address != key.address.trim().to_lowercase()
                && lookup.key().is_some_and(|found| same_key(&found, key))
        })
        .collect();
    others.sort_by(|a, b| a.address.cmp(&b.address));
    std::iter::once(key.clone())
        .chain(others.into_iter().map(|lookup| RecipientKey {
            address: lookup.address.clone(),
            custom_key: None,
            custom_value: None,
        }))
        .collect()
}

/// The key of the wallet of `key` and its members.
#[must_use]
pub fn wallet_of<S: BuildHasher>(
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> (RecipientKey, Vec<RecipientKey>) {
    let wallet = wallet_key(key, lookups);
    let list = members(&wallet, lookups);
    (wallet, list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestServer;

    const SPEC: &str = r#"{"status":"OK","tag":"keysend","pubkey":"02abc","customData":[{"customKey":"696969","customValue":"x1"}]}"#;

    fn key(address: &str) -> RecipientKey {
        RecipientKey {
            address: address.to_owned(),
            custom_key: None,
            custom_value: None,
        }
    }

    fn collection() -> (tempfile::TempDir, Collection) {
        let dir = tempfile::tempdir().unwrap();
        let collection = Collection::open(&dir.path().join("c.sqlite")).unwrap();
        (dir, collection)
    }

    #[test]
    fn lookup_url_reads_a_lightning_address() {
        assert_eq!(
            lookup_url("https", "Chadf@GetAlby.com").as_deref(),
            Some("https://getalby.com/.well-known/keysend/chadf")
        );
        for bad in ["02abc", "@x", "a@", "", "a@b/c", "a b@c.d"] {
            assert_eq!(lookup_url("https", bad), None, "text: {bad:?}");
        }
    }

    #[test]
    fn parse_lookup_reads_the_specification_example() {
        assert_eq!(
            parse_lookup(SPEC.as_bytes()),
            Some(("02abc".into(), Some("696969".into()), Some("x1".into())))
        );
        assert_eq!(
            parse_lookup(br#"{"status":"OK","tag":"keysend","pubkey":"02abc"}"#),
            Some(("02abc".into(), None, None))
        );
        for bad in [
            r#"{"status":"ERROR","reason":"x"}"#,
            r#"{"status":"OK"}"#,
            r#"{"pubkey":""}"#,
            "not json",
        ] {
            assert_eq!(parse_lookup(bad.as_bytes()), None, "text: {bad:?}");
        }
    }

    #[test]
    fn refresh_stores_each_status_and_keeps_data_on_failure() {
        let (_dir, collection) = collection();
        let server = TestServer::start(vec![(
            "/.well-known/keysend/ann",
            200,
            SPEC.as_bytes().to_vec(),
        )]);
        let port = server.port();
        let found = format!("ann@127.0.0.1:{port}");
        let none = format!("bob@127.0.0.1:{port}");
        let web = Web::new();
        let both = [found.clone(), none.clone()];
        let result = refresh(&collection, &web, "http", &both, 1000).unwrap();
        assert_eq!(result[&found].status, LookupStatus::Found);
        assert_eq!(result[&found].pubkey.as_deref(), Some("02abc"));
        assert_eq!(result[&found].custom_value.as_deref(), Some("x1"));
        assert_eq!(result[&none].status, LookupStatus::None);

        // Within one day, no request goes out: the changed route is unseen.
        server.set("/.well-known/keysend/ann", 500, Vec::new());
        let again = refresh(&collection, &web, "http", &both, 2000).unwrap();
        assert_eq!(again[&found].checked_at, 1000);

        // After one day, a failed request keeps the data and the status.
        let later = refresh(&collection, &web, "http", &both, 1000 + REFRESH_AFTER).unwrap();
        assert_eq!(later[&found].status, LookupStatus::Found);
        assert_eq!(later[&found].pubkey.as_deref(), Some("02abc"));
        assert_eq!(later[&found].checked_at, 1000 + REFRESH_AFTER);
    }

    #[test]
    fn refresh_stores_failed_for_a_server_that_does_not_answer() {
        let (_dir, collection) = collection();
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let address = format!("ann@127.0.0.1:{port}");
        let result = refresh(
            &collection,
            &Web::new(),
            "http",
            std::slice::from_ref(&address),
            50,
        )
        .unwrap();
        assert_eq!(result[&address].status, LookupStatus::Failed);
        assert_eq!(result[&address].pubkey, None);
        assert!(!is_due(result.get(&address), 50 + RETRY_AFTER - 1));
        assert!(is_due(result.get(&address), 50 + RETRY_AFTER));
        assert!(is_due(None, 0));
    }

    #[test]
    fn wallet_groups_a_node_key_and_its_lightning_addresses() {
        let found = |address: &str| Lookup {
            address: address.to_owned(),
            pubkey: Some("02abc".into()),
            custom_key: Some("696969".into()),
            custom_value: Some("x1".into()),
            status: LookupStatus::Found,
            checked_at: 0,
        };
        let mut lookups = HashMap::new();
        lookups.insert("b@h.example".to_owned(), found("b@h.example"));
        lookups.insert("a@h.example".to_owned(), found("a@h.example"));
        lookups.insert(
            "c@h.example".to_owned(),
            Lookup {
                status: LookupStatus::None,
                pubkey: None,
                custom_key: None,
                custom_value: None,
                ..found("c@h.example")
            },
        );
        let node = RecipientKey {
            address: "02abc".into(),
            custom_key: Some("696969".into()),
            custom_value: Some("x1".into()),
        };
        let (wallet, list) = wallet_of(&key("A@h.example"), &lookups);
        assert!(same_key(&wallet, &node));
        let addresses: Vec<&str> = list.iter().map(|k| k.address.as_str()).collect();
        assert_eq!(addresses, ["02abc", "a@h.example", "b@h.example"]);
        assert_eq!(wallet_of(&node, &lookups).1, list);
        let (lonely, list) = wallet_of(&key("c@h.example"), &lookups);
        assert_eq!(lonely, key("c@h.example"));
        assert_eq!(list, vec![key("c@h.example")]);
    }
}
