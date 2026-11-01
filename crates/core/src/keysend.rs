//! The keysend lookup of a Lightning address (ADR 0095 items 1 to 5).
//!
//! A Lightning address of the type `lnaddress` can name the same wallet as
//! a `node` recipient. The host of the address gives the node key and the
//! custom data at `/.well-known/keysend/<name>`. [`refresh`] requests and
//! keeps these lookups. [`wallet_of`] groups the addresses of one wallet.
//! [`suggest`] checks guesses for the Lightning address of a wallet
//! (ADR 0097).

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::change_set::{RecipientKey, Wallet};
use crate::collection::{Collection, CollectionError};
use crate::feed::{Feed, Recipient};
use crate::profiles::{feed_keys, within};
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

/// Node keys that many Lightning addresses share, with the domain of the
/// provider. The evidence is in
/// `docs/research/2026-10-31-keysend-and-lightning-address-recipients.md`.
/// An entry needs a lookup or a graph alias as evidence.
pub const SHARED_NODES: &[(&str, &str)] = &[
    (
        "03b6f613e88bd874177c28c6ad83b3baba43c4c656f56be1f8df84669556054b79",
        "fountain.fm",
    ),
    (
        "030a58b8653d32b99200a2334cfe913e51dc7d155aa0116c176657a4f1722677a3",
        "getalby.com",
    ),
];

/// The provider of the node `pubkey`. The key is compared in lowercase.
///
/// The table [`SHARED_NODES`] gives the domain first. Else the function
/// reads each `Found` lookup of the key. A domain with two or more
/// different custom pairs names the node. With two or more such domains,
/// the result is the domains from A to Z, with `, ` between them. Else the
/// result is `None`.
#[must_use]
pub fn provider_of<S: BuildHasher>(
    pubkey: &str,
    lookups: &HashMap<String, Lookup, S>,
) -> Option<String> {
    let pubkey = pubkey.trim().to_lowercase();
    if let Some((_, domain)) = SHARED_NODES.iter().find(|(key, _)| *key == pubkey) {
        return Some((*domain).to_owned());
    }
    let mut pairs: HashMap<&str, HashSet<(&str, &str)>> = HashMap::new();
    for lookup in lookups.values() {
        let named = lookup.status == LookupStatus::Found
            && lookup
                .pubkey
                .as_deref()
                .is_some_and(|key| key.trim().to_lowercase() == pubkey);
        let Some((_, domain)) = lookup.address.rsplit_once('@') else {
            continue;
        };
        if named {
            pairs.entry(domain).or_default().insert((
                lookup.custom_key.as_deref().unwrap_or_default(),
                lookup.custom_value.as_deref().unwrap_or_default(),
            ));
        }
    }
    let mut domains: Vec<&str> = pairs
        .into_iter()
        .filter(|(_, set)| set.len() >= 2)
        .map(|(domain, _)| domain)
        .collect();
    domains.sort_unstable();
    if domains.is_empty() {
        None
    } else {
        Some(domains.join(", "))
    }
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

// ---- The suggestions of a Lightning address (ADR 0097) ----

/// The largest number of guesses that [`suggest`] checks (ADR 0097 item 3).
pub const MAX_GUESSES: usize = 16;

/// The largest number of Nostr keys for the guesses (ADR 0097 item 2).
pub const MAX_GUESS_KEYS: usize = 8;

/// Where a guess came from, in the sequence of the suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuessSource {
    /// A Lightning address of a recipient in the collection with a name of
    /// the wallet.
    Collection,
    /// The `lud16` of a Nostr key of a feed that uses the wallet.
    Nostr,
    /// A name of the wallet at the domain of the provider of its node.
    Provider,
}

/// A Lightning address that pays the wallet, with the sources that gave it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    /// The Lightning address, in lowercase.
    pub address: String,
    /// Each source of the address, in the sequence of [`GuessSource`].
    pub sources: Vec<GuessSource>,
}

/// The facts of the collection for the guesses of a wallet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gathered {
    /// The names of the recipients of the wallet, each one time.
    pub names: Vec<String>,
    /// The collection guesses, in lowercase.
    pub collection: Vec<String>,
    /// The Nostr keys of the feeds that use the wallet, as `npub`. A key
    /// with a name of the wallet comes first.
    pub npubs: Vec<String>,
}

/// Each recipient of the channel, the items and the live items of `feed`.
fn recipients_of(feed: &Feed) -> impl Iterator<Item = &Recipient> {
    feed.values
        .iter()
        .chain(feed.items.iter().flat_map(|item| item.value.values.iter()))
        .chain(
            feed.live_items
                .iter()
                .flat_map(|live| live.value.item.values.iter()),
        )
        .flat_map(|block| block.value.recipients.iter().map(|r| &r.value))
}

fn is_member(wallet: &Wallet, address: &str) -> bool {
    wallet
        .members
        .iter()
        .any(|m| m.address.trim().eq_ignore_ascii_case(address.trim()))
}

/// Read the names, the collection guesses and the Nostr keys of `wallet`
/// from the feeds of the collection (ADR 0097 item 2).
#[must_use]
pub fn gather(feeds: &[Feed], wallet: &Wallet) -> Gathered {
    let mut names: Vec<String> = Vec::new();
    let mut paying: Vec<&Feed> = Vec::new();
    for feed in feeds {
        let mut uses = false;
        for recipient in recipients_of(feed).filter(|r| wallet.matches(r)) {
            uses = true;
            let name = recipient.name.as_deref().map(str::trim).unwrap_or_default();
            if !name.is_empty()
                && !names
                    .iter()
                    .any(|n| n.to_lowercase() == name.to_lowercase())
            {
                names.push(name.to_owned());
            }
        }
        if uses {
            paying.push(feed);
        }
    }
    let has_name = |text: &str| {
        let text = text.trim().to_lowercase();
        names.iter().any(|n| n.to_lowercase() == text)
    };
    let mut collection: Vec<String> = Vec::new();
    for recipient in feeds.iter().flat_map(recipients_of) {
        let address = recipient
            .address
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        if address.contains('@')
            && recipient.name.as_deref().is_some_and(has_name)
            && !is_member(wallet, &address)
            && !collection.contains(&address)
        {
            collection.push(address);
        }
    }
    let mut keys: Vec<(String, String)> = Vec::new();
    for pair in paying.into_iter().flat_map(feed_keys) {
        if !keys.iter().any(|(_, npub)| *npub == pair.1) {
            keys.push(pair);
        }
    }
    keys.sort_by_key(|(name, _)| !has_name(name));
    let npubs = keys
        .into_iter()
        .take(MAX_GUESS_KEYS)
        .map(|(_, npub)| npub)
        .collect();
    Gathered {
        names,
        collection,
        npubs,
    }
}

/// The local part of a Lightning address for `name`: the name in lowercase,
/// with only the letters `a` to `z`, the digits and `.`, `_` and `-`. A
/// name with `@`, such as `ann@fountain.fm`, gives the text before the `@`.
/// A name with no such character gives `None`.
#[must_use]
pub fn local_part(name: &str) -> Option<String> {
    let name = name.split_once('@').map_or(name, |(local, _)| local);
    let text: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    let text = text.trim_matches('.');
    (!text.is_empty()).then(|| text.to_owned())
}

/// Each name at each domain of `provider`, the result of [`provider_of`]
/// (ADR 0097 item 2).
#[must_use]
pub fn provider_guesses(names: &[String], provider: Option<&str>) -> Vec<String> {
    let Some(provider) = provider else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for name in names.iter().filter_map(|n| local_part(n)) {
        for domain in provider
            .split(", ")
            .map(str::trim)
            .filter(|d| !d.is_empty())
        {
            let address = format!("{name}@{domain}");
            if !out.contains(&address) {
                out.push(address);
            }
        }
    }
    out
}

/// Request the lookup of each guess, and give each guess that pays
/// `wallet` (ADR 0097 items 1, 3 and 4). The collection keeps no lookup of
/// a guess. A guess that is a member of the wallet is not requested. At
/// most [`MAX_GUESSES`] guesses are requested, the best source first.
#[must_use]
pub fn suggest(
    web: &Web,
    scheme: &str,
    wallet: &Wallet,
    guesses: &[(String, GuessSource)],
) -> Vec<Suggestion> {
    let mut merged: Vec<Suggestion> = Vec::new();
    for (address, source) in guesses {
        let address = address.trim().to_lowercase();
        if lookup_url(scheme, &address).is_none() || is_member(wallet, &address) {
            continue;
        }
        if let Some(known) = merged.iter_mut().find(|s| s.address == address) {
            if !known.sources.contains(source) {
                known.sources.push(*source);
                known.sources.sort_unstable();
            }
        } else {
            merged.push(Suggestion {
                address,
                sources: vec![*source],
            });
        }
    }
    merged.sort_by_key(|s| s.sources.first().copied());
    merged.truncate(MAX_GUESSES);
    let found: Vec<bool> = thread::scope(|scope| {
        let handles: Vec<_> = merged
            .iter()
            .map(|suggestion| {
                scope.spawn(move || {
                    let web = web.clone();
                    let scheme = scheme.to_owned();
                    let text = suggestion.address.clone();
                    let result = within(LOOKUP_LIMIT, move || request(&web, &scheme, &text));
                    match result {
                        Some(LookupResult::Found {
                            pubkey,
                            custom_key,
                            custom_value,
                        }) => {
                            let found = RecipientKey {
                                address: pubkey,
                                custom_key,
                                custom_value,
                            };
                            same_key(&found, &wallet.key)
                        }
                        _ => false,
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or(false))
            .collect()
    });
    merged
        .into_iter()
        .zip(found)
        .filter_map(|(suggestion, pays)| pays.then_some(suggestion))
        .collect()
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

    #[test]
    fn provider_of_names_a_node_by_table_or_by_lookups() {
        let found = |address: &str, value: &str| Lookup {
            address: address.to_owned(),
            pubkey: Some("02abc".into()),
            custom_key: Some("696969".into()),
            custom_value: Some(value.into()),
            status: LookupStatus::Found,
            checked_at: 0,
        };
        let empty: HashMap<String, Lookup> = HashMap::new();
        let fountain = SHARED_NODES[0].0;
        assert_eq!(
            provider_of(fountain, &empty).as_deref(),
            Some("fountain.fm")
        );
        assert_eq!(
            provider_of(&fountain.to_uppercase(), &empty).as_deref(),
            Some("fountain.fm")
        );

        let mut lookups = HashMap::new();
        lookups.insert(
            "a@fountain.example".to_owned(),
            found("a@fountain.example", "1"),
        );
        assert_eq!(provider_of("02abc", &lookups), None);
        lookups.insert(
            "b@fountain.example".to_owned(),
            found("b@fountain.example", "2"),
        );
        assert_eq!(
            provider_of("02ABC", &lookups).as_deref(),
            Some("fountain.example")
        );
        assert_eq!(provider_of("02def", &lookups), None);

        let mut same = HashMap::new();
        same.insert("a@h.example".to_owned(), found("a@h.example", "1"));
        same.insert("b@h.example".to_owned(), found("b@h.example", "1"));
        assert_eq!(provider_of("02abc", &same), None);
    }

    // ---- The suggestions (ADR 0097) ----

    const NODE: &str = "03aaa";

    fn spec(custom_value: &str) -> Vec<u8> {
        format!(r#"{{"status":"OK","tag":"keysend","pubkey":"{NODE}","customData":[{{"customKey":"696969","customValue":"{custom_value}"}}]}}"#).into_bytes()
    }

    fn node_wallet() -> Wallet {
        Wallet::single(RecipientKey {
            address: NODE.to_owned(),
            custom_key: Some("696969".to_owned()),
            custom_value: Some("acct1".to_owned()),
        })
    }

    fn feed_of(recipients: &[(&str, &str, Option<&str>)]) -> Feed {
        use crate::feed::{Entry, ValueBlock};
        let recipients = recipients
            .iter()
            .map(|(name, address, value)| {
                Entry::new(Recipient {
                    name: Some((*name).to_owned()),
                    address: Some((*address).to_owned()),
                    custom_key: value.map(|_| "696969".to_owned()),
                    custom_value: value.map(str::to_owned),
                    ..Recipient::default()
                })
            })
            .collect();
        Feed {
            values: vec![Entry::new(ValueBlock {
                recipients,
                ..ValueBlock::default()
            })],
            ..Feed::default()
        }
    }

    #[test]
    fn local_part_keeps_the_letters_of_a_name() {
        assert_eq!(local_part("Steven B").as_deref(), Some("stevenb"));
        assert_eq!(
            local_part("boost_bot-2.0").as_deref(),
            Some("boost_bot-2.0")
        );
        assert_eq!(local_part(".Ann."), Some("ann".to_owned()));
        assert_eq!(
            local_part("Malachi@fountain.fm").as_deref(),
            Some("malachi")
        );
        assert_eq!(local_part("☆ ☆"), None);
        assert_eq!(local_part(""), None);
    }

    #[test]
    fn provider_guesses_put_each_name_at_each_domain() {
        let names = vec!["Steven B".to_owned(), "☆".to_owned()];
        assert_eq!(
            provider_guesses(&names, Some("a.example, b.example")),
            vec!["stevenb@a.example", "stevenb@b.example"]
        );
        assert_eq!(provider_guesses(&names, None), Vec::<String>::new());
    }

    #[test]
    fn gather_reads_names_collection_guesses_and_keys() {
        use nostr::nips::nip19::ToBech32;
        let keys = nostr::prelude::Keys::new(
            nostr::prelude::SecretKey::from_slice(&[5; 32]).expect("a secret key"),
        );
        let npub = keys.public_key().to_bech32().expect("an npub");
        let mut used = feed_of(&[("Ann", NODE, Some("acct1")), ("Bob", "02bbb", None)]);
        used.txts = vec![crate::feed::Entry::new(crate::feed::Txt {
            purpose: Some("npub".to_owned()),
            value: npub.clone(),
        })];
        let other = feed_of(&[
            ("ann", "Ann@Pay.example", None),
            ("Carl", "carl@pay.example", None),
        ]);
        let gathered = gather(&[used, other], &node_wallet());
        assert_eq!(gathered.names, vec!["Ann"]);
        assert_eq!(gathered.collection, vec!["ann@pay.example"]);
        assert_eq!(gathered.npubs, vec![npub]);
    }

    #[test]
    fn suggest_gives_only_guesses_that_pay_the_wallet() {
        let (_dir, collection) = collection();
        let server = TestServer::start(vec![
            ("/.well-known/keysend/ann", 200, spec("acct1")),
            ("/.well-known/keysend/annie", 200, spec("acct1")),
            ("/.well-known/keysend/bob", 200, spec("acct2")),
            ("/.well-known/keysend/member", 200, spec("acct1")),
        ]);
        let port = server.port();
        let at = |name: &str| format!("{name}@127.0.0.1:{port}");
        let mut wallet = node_wallet();
        wallet.members.push(key(&at("member")));
        let guesses = vec![
            (at("ann"), GuessSource::Provider),
            (at("bob"), GuessSource::Provider),
            (at("member"), GuessSource::Collection),
            (at("nobody"), GuessSource::Provider),
            (at("annie"), GuessSource::Nostr),
            (at("ANN"), GuessSource::Collection),
        ];
        let found = suggest(&Web::new(), "http", &wallet, &guesses);
        assert_eq!(
            found,
            vec![
                Suggestion {
                    address: at("ann"),
                    sources: vec![GuessSource::Collection, GuessSource::Provider],
                },
                Suggestion {
                    address: at("annie"),
                    sources: vec![GuessSource::Nostr],
                },
            ]
        );
        assert_eq!(collection.keysend_lookups().unwrap(), Vec::<Lookup>::new());
    }
}
