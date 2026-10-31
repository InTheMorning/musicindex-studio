//! The search for Nostr profiles (ADR 0074 items 7 to 17).
//!
//! [`parse_query`] reads the text of a user as a key, a NIP-05 address or a
//! name. [`find`] gives the profiles for it. A name search asks the
//! collection, musicindex and a search relay at the same time. Each source
//! has a time limit, and a source that fails does not stop the others.

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use nostr::nips::nip19::ToBech32;
use nostr::prelude::{Event, Filter, Kind, PublicKey};
use serde::Serialize;
use serde_json::Value;

use crate::collection::{Collection, CollectionError};
use crate::connector::relay::fetch_until;
use crate::feed::{Entry, Feed, Person};
use crate::musicindex::MusicIndex;
use crate::net::NetPolicy;
use crate::nostr_events::{npub_text, parse_public_key};
use crate::web::{Web, host_of};

/// The time limit of each source and of each lookup (ADR 0074 item 10).
pub const SOURCE_LIMIT: Duration = Duration::from_secs(5);

/// The start value of the search relay setting (ADR 0074 item 16).
pub const DEFAULT_SEARCH_RELAY: &str = "wss://search.nos.today";

/// The collection setting that holds the search relay.
const SEARCH_RELAY_SETTING: &str = "nostr_search_relay";

/// The largest NIP-05 answer that Studio reads, in bytes.
const MAX_NIP05_BYTES: u64 = 64 * 1024;

/// The largest number of profiles that one search gives.
const MAX_PROFILES: usize = 50;

/// The number of musicindex feeds that a name search reads.
const MUSICINDEX_FEEDS: usize = 5;

/// The number of results that the search relay gives.
const RELAY_RESULTS: usize = 20;

/// What the text of a user means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// A Nostr key.
    Key(PublicKey),
    /// A NIP-05 address. A domain alone has the name `_`.
    Nip05 {
        /// The local part of the address.
        name: String,
        /// The domain, in lowercase. It can have a port.
        domain: String,
    },
    /// A name for a search.
    Name(String),
}

/// Where a result came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A person or a feed key in a feed of the user.
    Collection,
    /// A key in feeds of musicindex.
    MusicIndex {
        /// The number of musicindex feeds that gave the key.
        feeds: usize,
    },
    /// The search relay.
    Relay,
}

impl Source {
    /// The place of the source in the sequence of the results.
    fn rank(&self) -> u8 {
        match self {
            Self::Collection => 0,
            Self::MusicIndex { .. } => 1,
            Self::Relay => 2,
        }
    }
}

/// A profile for the user to select.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Profile {
    /// The key, as `npub`.
    pub npub: String,
    /// The `display_name` of the profile, or its `name`, or the name that a
    /// source gave.
    pub name: Option<String>,
    /// The address of the picture. It is an `http` or `https` address.
    pub picture: Option<String>,
    /// The NIP-05 address of the profile.
    pub nip05: Option<String>,
    /// True when the domain of the NIP-05 address gives this key.
    pub nip05_ok: bool,
    /// True when `nip05_ok` is true and the NIP-05 domain is the domain of
    /// the feed `link` (ADR 0074 item 13).
    pub domain_match: bool,
    /// Each source of the key.
    pub sources: Vec<Source>,
}

/// The answer of [`find`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Found {
    /// The profiles, in the sequence for the user.
    pub profiles: Vec<Profile>,
    /// Each source that did not answer: `collection`, `musicindex` or
    /// `relay`.
    pub failed: Vec<String>,
}

/// What [`find`] needs from its caller.
#[derive(Debug)]
pub struct Context<'a> {
    /// The client for the NIP-05 requests. It follows the network policy.
    pub web: &'a Web,
    /// The network policy for each relay connection (ADR 0057).
    pub policy: &'a NetPolicy,
    /// The relays that give the kind 0 of a key.
    pub lookup_relays: &'a [String],
    /// The search relay, or `None` for no relay search (ADR 0074 item 17).
    pub search_relay: Option<&'a str>,
    /// The musicindex client.
    pub index: &'a MusicIndex,
    /// The keys of the collection, as a name and an `npub`.
    pub collection_keys: &'a [(String, String)],
    /// The scheme of a NIP-05 request. It is `https`. A test uses `http`.
    pub nip05_scheme: &'a str,
    /// The time limit of each source and of each lookup.
    pub limit: Duration,
}

// ---- The setting of the search relay ----

/// True when `text` can be the search relay setting: it is empty, or it
/// starts with `wss://` or `ws://`.
#[must_use]
pub fn valid_search_relay(text: &str) -> bool {
    let text = text.trim();
    text.is_empty() || text.starts_with("wss://") || text.starts_with("ws://")
}

/// The search relay setting. A missing setting gives
/// [`DEFAULT_SEARCH_RELAY`]. An empty text means no relay search.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn search_relay(collection: &Collection) -> Result<String, CollectionError> {
    Ok(match collection.setting(SEARCH_RELAY_SETTING)? {
        Some(Value::String(text)) => text.trim().to_owned(),
        _ => DEFAULT_SEARCH_RELAY.to_owned(),
    })
}

/// Set the search relay. The caller checks the text with
/// [`valid_search_relay`].
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn set_search_relay(collection: &Collection, text: &str) -> Result<(), CollectionError> {
    collection.set_setting(SEARCH_RELAY_SETTING, &Value::String(text.trim().to_owned()))
}

// ---- The text of the user ----

fn key_of(text: &str) -> Option<PublicKey> {
    npub_text(text).and_then(|npub| parse_public_key(&npub).ok())
}

fn is_web_address(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

/// True when `text` can be the domain of a NIP-05 address: it has a dot,
/// and only letters, digits, hyphens, dots and a port.
fn is_domain(text: &str) -> bool {
    let host = text.rsplit_once(':').map_or(text, |(host, port)| {
        if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) {
            host
        } else {
            text
        }
    });
    host.contains('.')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
}

/// Read the text of a user (ADR 0074 item 8), in this sequence: a key, a
/// web address with a key, a NIP-05 address, and a name.
#[must_use]
pub fn parse_query(text: &str) -> Query {
    let text = text.trim();
    if let Some(key) = key_of(text) {
        return Query::Key(key);
    }
    if is_web_address(text) {
        let key = text
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|word| word.starts_with("npub1") || word.starts_with("nprofile1"))
            .find_map(key_of);
        return key.map_or_else(|| Query::Name(text.to_owned()), Query::Key);
    }
    if !text.contains(char::is_whitespace) {
        if let Some((name, domain)) = text.split_once('@') {
            if !domain.contains('@') && is_domain(domain) {
                let name = if name.is_empty() { "_" } else { name };
                return Query::Nip05 {
                    name: name.to_owned(),
                    domain: domain.to_ascii_lowercase(),
                };
            }
        } else if is_domain(text) {
            return Query::Nip05 {
                name: "_".to_owned(),
                domain: text.to_ascii_lowercase(),
            };
        }
    }
    Query::Name(text.to_owned())
}

// ---- The keys of the collection ----

fn person_keys(out: &mut Vec<(String, String)>, persons: &[Entry<Person>]) {
    for person in persons {
        if let Some(npub) = person.npub.as_deref().and_then(npub_text) {
            out.push((person.name.clone(), npub));
        }
    }
}

/// The person keys and the feed key of a feed, as a name and an `npub`.
/// The name of the feed key is the author of the feed, or its title.
#[must_use]
pub fn feed_keys(feed: &Feed) -> Vec<(String, String)> {
    let mut out = Vec::new();
    person_keys(&mut out, &feed.persons);
    for txt in &feed.txts {
        if txt.purpose.as_deref() == Some("npub")
            && let Some(npub) = npub_text(&txt.value.value)
        {
            let name = feed.author.as_ref().or(feed.title.as_ref());
            out.push((name.cloned().unwrap_or_default(), npub));
        }
    }
    for item in &feed.items {
        person_keys(&mut out, &item.persons);
    }
    for live in &feed.live_items {
        person_keys(&mut out, &live.item.persons);
    }
    out
}

// ---- The sources ----

/// A key with the facts that the sources gave.
struct Candidate {
    key: PublicKey,
    hint: Option<String>,
    sources: Vec<Source>,
}

fn add_source(sources: &mut Vec<Source>, source: Source) {
    match source {
        Source::MusicIndex { feeds } => {
            if let Some(Source::MusicIndex { feeds: total }) = sources
                .iter_mut()
                .find(|s| matches!(s, Source::MusicIndex { .. }))
            {
                *total += feeds;
            } else {
                sources.push(Source::MusicIndex { feeds });
            }
        }
        other => {
            if !sources.contains(&other) {
                sources.push(other);
            }
        }
    }
}

/// Add a key to the list, or add the source to the key that the list has.
fn add_candidate(
    list: &mut Vec<Candidate>,
    key: PublicKey,
    hint: Option<String>,
    source: Option<Source>,
) {
    let index = list.iter().position(|c| c.key == key).unwrap_or_else(|| {
        list.push(Candidate {
            key,
            hint: None,
            sources: Vec::new(),
        });
        list.len() - 1
    });
    let candidate = &mut list[index];
    if candidate.hint.is_none() {
        candidate.hint = hint.filter(|h| !h.trim().is_empty());
    }
    if let Some(source) = source {
        add_source(&mut candidate.sources, source);
    }
}

fn contains_ignore_case(name: &str, text: &str) -> bool {
    name.to_lowercase().contains(&text.to_lowercase())
}

/// Run `work` on a new thread and wait at most `limit` for the result. A
/// late result is dropped. The thread ends with its own network time limit.
pub(crate) fn within<T: Send + 'static>(
    limit: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver.recv_timeout(limit).ok()
}

/// A key that musicindex gives, with the name that it has in the feed.
type IndexKey = (PublicKey, String);

/// The keys of the first feeds of a musicindex search, with a name that
/// contains `text` (ADR 0074 item 9). Each feed gives a key one time.
fn musicindex_keys(index: &MusicIndex, text: &str) -> Option<Vec<IndexKey>> {
    let hits = index.search(text).ok()?;
    let guids: Vec<String> = hits
        .into_iter()
        .take(MUSICINDEX_FEEDS)
        .map(|h| h.feed_guid)
        .collect();
    let feeds: Vec<Vec<IndexKey>> = thread::scope(|scope| {
        let handles: Vec<_> = guids
            .iter()
            .map(|guid| scope.spawn(move || index.feed_keys(guid)))
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                let Ok(Ok(Some(keys))) = handle.join() else {
                    return Vec::new();
                };
                let artist = keys.artist.unwrap_or_default();
                let mut list: Vec<IndexKey> = Vec::new();
                let named = keys
                    .feed_keys
                    .into_iter()
                    .map(|npub| (artist.clone(), npub))
                    .chain(keys.contributors);
                for (name, npub) in named {
                    if !contains_ignore_case(&name, text) {
                        continue;
                    }
                    if let Some(key) = key_of(&npub)
                        && !list.iter().any(|(k, _)| *k == key)
                    {
                        list.push((key, name));
                    }
                }
                list
            })
            .collect()
    });
    Some(feeds.into_iter().flatten().collect())
}

/// The events of the search relay for a name (ADR 0074 item 9).
fn relay_events(cx: &Context<'_>, relay: &str, text: &str) -> Result<Vec<Event>, ()> {
    let filter = Filter::new()
        .kind(Kind::Metadata)
        .search(text)
        .limit(RELAY_RESULTS);
    fetch_until(relay, &filter, cx.limit, cx.policy).map_err(|_| ())
}

/// The three sources of a name search, at the same time. Gives the
/// candidates, the events of the search relay and the sources that failed.
fn name_sources(cx: &Context<'_>, text: &str) -> (Vec<Candidate>, Vec<Event>, Vec<String>) {
    let (index_result, relay_result) = thread::scope(|scope| {
        let index = scope.spawn(|| {
            let (index, text) = (cx.index.clone(), text.to_owned());
            within(cx.limit, move || musicindex_keys(&index, &text)).flatten()
        });
        let relay = scope.spawn(|| {
            cx.search_relay
                .filter(|relay| !relay.trim().is_empty())
                .map(|relay| relay_events(cx, relay, text))
        });
        (index.join().ok().flatten(), relay.join().ok().flatten())
    });
    let mut failed = Vec::new();
    let mut candidates = Vec::new();
    for (name, npub) in cx.collection_keys {
        if contains_ignore_case(name, text)
            && let Some(key) = key_of(npub)
        {
            add_candidate(
                &mut candidates,
                key,
                Some(name.clone()),
                Some(Source::Collection),
            );
        }
    }
    match index_result {
        Some(keys) => {
            for (key, name) in keys {
                add_candidate(
                    &mut candidates,
                    key,
                    Some(name),
                    Some(Source::MusicIndex { feeds: 1 }),
                );
            }
        }
        None => failed.push("musicindex".to_owned()),
    }
    let mut events = Vec::new();
    match relay_result {
        Some(Ok(found)) => {
            for event in found.into_iter().filter(|e| e.kind == Kind::Metadata) {
                add_candidate(&mut candidates, event.pubkey, None, Some(Source::Relay));
                events.push(event);
            }
        }
        Some(Err(())) => failed.push("relay".to_owned()),
        None => {}
    }
    (candidates, events, failed)
}

// ---- NIP-05 ----

/// The local part and the domain of a NIP-05 address. A domain alone has
/// the name `_`.
fn split_address(address: &str) -> Option<(String, String)> {
    let address = address.trim().to_ascii_lowercase();
    match address.split_once('@') {
        Some((name, domain)) if is_domain(domain) => {
            let name = if name.is_empty() { "_" } else { name };
            Some((name.to_owned(), domain.to_owned()))
        }
        None if is_domain(&address) => Some(("_".to_owned(), address)),
        Some(_) | None => None,
    }
}

/// The text of a NIP-05 address.
fn address_text(name: &str, domain: &str) -> String {
    if name == "_" {
        domain.to_owned()
    } else {
        format!("{name}@{domain}")
    }
}

/// The key, as hexadecimal text, that the domain gives for the name.
fn nip05_key(web: &Web, scheme: &str, name: &str, domain: &str) -> Option<String> {
    let url = format!("{scheme}://{domain}/.well-known/nostr.json");
    let file = web
        .get_with_query(&url, &[("name", name)], MAX_NIP05_BYTES)
        .ok()?;
    // NIP-05: a client ignores each HTTP redirect of `nostr.json`. A file
    // that came from a different address gives no key.
    let read_at = file.final_url.split('?').next().unwrap_or_default();
    if read_at != url {
        return None;
    }
    let json: Value = serde_json::from_slice(&file.bytes).ok()?;
    json.get("names")?
        .as_object()?
        .iter()
        .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
        .and_then(|(_, key)| key.as_str())
        .map(str::to_ascii_lowercase)
}

/// The keys of some NIP-05 addresses, asked at the same time. The result
/// has the lowercase address and its key, or `None` for no answer.
fn check_addresses(cx: &Context<'_>, addresses: &[String]) -> HashMap<String, Option<String>> {
    thread::scope(|scope| {
        let handles: Vec<_> = addresses
            .iter()
            .map(|address| {
                scope.spawn(move || {
                    let key = split_address(address).and_then(|(name, domain)| {
                        let web = cx.web.clone();
                        let scheme = cx.nip05_scheme.to_owned();
                        within(cx.limit, move || nip05_key(&web, &scheme, &name, &domain)).flatten()
                    });
                    (address.clone(), key)
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    })
}

// ---- The kind 0 of the keys ----

/// The newest kind 0 event of each key, from the lookup relays and from
/// `known` events. A relay that fails gives nothing.
fn newest_profiles(
    cx: &Context<'_>,
    keys: &[PublicKey],
    known: Vec<Event>,
) -> HashMap<PublicKey, Event> {
    let mut all = known;
    if !keys.is_empty() {
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .authors(keys.iter().copied())
            .limit(keys.len() * 2);
        let found: Vec<Vec<Event>> = thread::scope(|scope| {
            let handles: Vec<_> = cx
                .lookup_relays
                .iter()
                .map(|relay| {
                    let filter = &filter;
                    scope.spawn(move || {
                        fetch_until(relay, filter, cx.limit, cx.policy).unwrap_or_default()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap_or_default())
                .collect()
        });
        all.extend(found.into_iter().flatten());
    }
    let mut newest: HashMap<PublicKey, Event> = HashMap::new();
    for event in all.into_iter().filter(|e| e.kind == Kind::Metadata) {
        let replace = newest
            .get(&event.pubkey)
            .is_none_or(|old| (event.created_at, old.id) > (old.created_at, event.id));
        if replace {
            newest.insert(event.pubkey, event);
        }
    }
    newest
}

/// The facts of the content of a kind 0 event.
#[derive(Debug, Default)]
struct Meta {
    name: Option<String>,
    picture: Option<String>,
    nip05: Option<String>,
}

fn meta_of(event: &Event) -> Meta {
    let Ok(Value::Object(content)) = serde_json::from_str::<Value>(&event.content) else {
        return Meta::default();
    };
    let text = |field: &str| {
        content
            .get(field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
    };
    Meta {
        name: text("display_name").or_else(|| text("name")),
        picture: text("picture").filter(|p| is_web_address(p)),
        nip05: text("nip05"),
    }
}

// ---- The result ----

/// The host of an address without `www.`, in lowercase.
fn bare_host(url: &str) -> Option<String> {
    let host = host_of(url)?;
    Some(host.strip_prefix("www.").unwrap_or(&host).to_owned())
}

/// Put the profiles in the sequence for the user (ADR 0074 item 11): the
/// profiles with `domain_match`, then the profiles with more than one
/// source, then the others. In each group, the first source decides.
fn order(profiles: &mut [Profile]) {
    let first = |p: &Profile| p.sources.iter().map(Source::rank).min().unwrap_or(u8::MAX);
    profiles.sort_by_key(|p| {
        let group = if p.domain_match {
            0_u8
        } else if p.sources.len() > 1 {
            1
        } else {
            2
        };
        (group, first(p))
    });
}

/// Find the profiles for a query (ADR 0074 items 7 to 17). `feed_link` is
/// the `link` of the feed that the user edits. A profile whose NIP-05 domain
/// is the domain of that link has `domain_match`.
#[must_use]
pub fn find(cx: &Context<'_>, query: &Query, feed_link: Option<&str>) -> Found {
    let mut failed = Vec::new();
    let mut events = Vec::new();
    // The NIP-05 addresses that the query already proved.
    let mut proved: HashMap<String, Option<String>> = HashMap::new();
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut query_address = None;
    match query {
        Query::Key(key) => add_candidate(&mut candidates, *key, None, None),
        Query::Nip05 { name, domain } => {
            let address = address_text(name, domain);
            let found = check_addresses(cx, std::slice::from_ref(&address));
            let hex = found.get(&address).cloned().flatten();
            proved.insert(address.clone(), hex.clone());
            if let Some(key) = hex.and_then(|h| PublicKey::from_hex(&h).ok()) {
                add_candidate(&mut candidates, key, None, None);
                query_address = Some((key, address));
            }
        }
        Query::Name(text) => {
            let (found, relay_events, lost) = name_sources(cx, text);
            candidates = found;
            events = relay_events;
            failed = lost;
        }
    }
    // The keys with more sources come first, so that the limit keeps them.
    candidates.sort_by_key(|c| std::cmp::Reverse(c.sources.len()));
    candidates.truncate(MAX_PROFILES);
    let keys: Vec<PublicKey> = candidates.iter().map(|c| c.key).collect();
    let newest = newest_profiles(cx, &keys, events);
    let metas: Vec<Meta> = candidates
        .iter()
        .map(|c| {
            let mut meta = newest.get(&c.key).map(meta_of).unwrap_or_default();
            if meta.nip05.is_none()
                && let Some((key, address)) = &query_address
                && *key == c.key
            {
                meta.nip05 = Some(address.clone());
            }
            meta
        })
        .collect();
    let mut to_check: Vec<String> = metas
        .iter()
        .filter_map(|m| m.nip05.as_deref())
        .filter_map(split_address)
        .map(|(name, domain)| address_text(&name, &domain))
        .filter(|address| !proved.contains_key(address))
        .collect();
    to_check.sort();
    to_check.dedup();
    proved.extend(check_addresses(cx, &to_check));
    let link_host = feed_link.and_then(bare_host);
    let mut profiles: Vec<Profile> = candidates
        .into_iter()
        .zip(metas)
        .filter_map(|(candidate, meta)| {
            let npub = candidate.key.to_bech32().ok()?;
            let parts = meta.nip05.as_deref().and_then(split_address);
            let nip05_ok = parts.as_ref().is_some_and(|(name, domain)| {
                proved
                    .get(&address_text(name, domain))
                    .and_then(Option::as_deref)
                    .is_some_and(|hex| hex == candidate.key.to_hex())
            });
            let domain_match = nip05_ok
                && parts.as_ref().is_some_and(|(_, domain)| {
                    link_host.is_some()
                        && bare_host(&format!("{}://{domain}", cx.nip05_scheme)) == link_host
                });
            let mut sources = candidate.sources;
            sources.sort_by_key(Source::rank);
            Some(Profile {
                npub,
                name: meta.name.or(candidate.hint),
                picture: meta.picture,
                nip05: meta.nip05,
                nip05_ok,
                domain_match,
                sources,
            })
        })
        .collect();
    order(&mut profiles);
    Found { profiles, failed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr_events::test_npub;

    fn profile(tag: &str, sources: Vec<Source>, domain_match: bool) -> Profile {
        Profile {
            npub: tag.to_owned(),
            name: None,
            picture: None,
            nip05: None,
            nip05_ok: domain_match,
            domain_match,
            sources,
        }
    }

    #[test]
    fn parse_query_reads_each_form_of_a_key() {
        let npub = test_npub(1);
        let key = parse_public_key(&npub).expect("a key");
        let hex = key.to_hex();
        let nprofile =
            nostr::nips::nip19::Nip19Profile::new(key, Vec::<nostr::prelude::RelayUrl>::new())
                .to_bech32()
                .expect("an nprofile");
        for text in [
            npub.clone(),
            format!("  {npub}  "),
            nprofile.clone(),
            hex,
            format!("nostr:{npub}"),
            format!("https://njump.me/{npub}"),
            format!("https://example.com/p/{nprofile}?x=1"),
        ] {
            assert_eq!(parse_query(&text), Query::Key(key), "{text}");
        }
    }

    #[test]
    fn parse_query_reads_a_nip05_address_a_domain_and_a_name() {
        assert_eq!(
            parse_query("Ann@Example.com"),
            Query::Nip05 {
                name: "Ann".to_owned(),
                domain: "example.com".to_owned()
            }
        );
        assert_eq!(
            parse_query("example.com"),
            Query::Nip05 {
                name: "_".to_owned(),
                domain: "example.com".to_owned()
            }
        );
        assert_eq!(
            parse_query("@example.com"),
            Query::Nip05 {
                name: "_".to_owned(),
                domain: "example.com".to_owned()
            }
        );
        assert_eq!(
            parse_query("ann@127.0.0.1:4100"),
            Query::Nip05 {
                name: "ann".to_owned(),
                domain: "127.0.0.1:4100".to_owned()
            }
        );
        for name in [
            "Matt Finlay",
            "phafe",
            "Matt F. Finlay",
            "ann@localhost",
            "https://example.com/about",
            "nsec1abc",
        ] {
            assert_eq!(parse_query(name), Query::Name(name.to_owned()), "{name}");
        }
    }

    #[test]
    fn the_sequence_has_domain_matches_then_more_sources_then_the_rest() {
        let mut list = vec![
            profile("relay", vec![Source::Relay], false),
            profile("own", vec![Source::Collection], false),
            profile(
                "two",
                vec![Source::MusicIndex { feeds: 2 }, Source::Relay],
                false,
            ),
            profile("domain", vec![Source::Relay], true),
            profile("index", vec![Source::MusicIndex { feeds: 1 }], false),
        ];
        order(&mut list);
        let tags: Vec<&str> = list.iter().map(|p| p.npub.as_str()).collect();
        assert_eq!(tags, ["domain", "two", "own", "index", "relay"]);
    }

    #[test]
    fn a_key_from_two_sources_is_one_candidate_with_both_sources() {
        let key = parse_public_key(&test_npub(1)).expect("a key");
        let mut list = Vec::new();
        add_candidate(
            &mut list,
            key,
            Some("Ann".to_owned()),
            Some(Source::Collection),
        );
        add_candidate(
            &mut list,
            key,
            Some("Other".to_owned()),
            Some(Source::MusicIndex { feeds: 1 }),
        );
        add_candidate(&mut list, key, None, Some(Source::MusicIndex { feeds: 1 }));
        add_candidate(&mut list, key, None, Some(Source::Collection));
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].hint.as_deref(), Some("Ann"));
        assert_eq!(
            list[0].sources,
            [Source::Collection, Source::MusicIndex { feeds: 2 }]
        );
    }

    #[test]
    fn the_search_relay_setting_has_a_start_value_and_a_rule() {
        assert!(valid_search_relay(""));
        assert!(valid_search_relay("wss://search.example"));
        assert!(valid_search_relay("ws://127.0.0.1:4000"));
        assert!(!valid_search_relay("https://search.example"));
        assert!(!valid_search_relay("search.example"));
    }

    #[test]
    fn a_kind_0_event_gives_the_display_name_before_the_name() {
        use nostr::prelude::{Keys, SecretKey, Timestamp, UnsignedEvent};
        let keys = Keys::new(SecretKey::from_slice(&[3; 32]).expect("a secret key"));
        let make = |content: &str| {
            let unsigned = UnsignedEvent::new(
                keys.public_key(),
                Timestamp::from(100),
                Kind::Metadata,
                Vec::new(),
                content,
            );
            crate::nostr_events::sign_with_keys(unsigned, &keys).expect("an event")
        };
        let both = meta_of(&make(
            r#"{"name":"mattf","display_name":"Matt Finlay","picture":"https://x.example/a.png","nip05":"matt@x.example"}"#,
        ));
        assert_eq!(both.name.as_deref(), Some("Matt Finlay"));
        assert_eq!(both.picture.as_deref(), Some("https://x.example/a.png"));
        assert_eq!(both.nip05.as_deref(), Some("matt@x.example"));
        let plain = meta_of(&make(
            r#"{"name":"mattf","display_name":"","picture":"ftp://x"}"#,
        ));
        assert_eq!(plain.name.as_deref(), Some("mattf"));
        assert_eq!(plain.picture, None);
        assert!(meta_of(&make("not JSON")).name.is_none());
    }
}
