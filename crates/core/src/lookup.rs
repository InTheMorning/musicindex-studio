//! The lookup of the artist and label feeds of musicindex (ADR 0117).
//!
//! The lookup finds the publisher feeds that a name matches. It sorts them
//! into three levels, and it reads the values of one feed for a person. The
//! lookup adds nothing to the collection. A value goes to the collection
//! only through a save of the person (ADR 0117 items 15 and 17).
//!
//! The lookup has two steps for the values of a feed. [`fetch_values`] makes
//! the requests, and it reads no collection. [`compose_values`] reads the
//! collection, and it makes no request. A caller can thus run the requests
//! with no lock on the collection.

use std::collections::HashMap;

use serde::Serialize;

use crate::change_set::RecipientKey;
use crate::collection::{Collection, CollectionError};
use crate::feed::Recipient;
use crate::keysend::{self, Lookup, LookupResult};
use crate::musicindex::{IndexSearchRequest, MusicIndex, PublisherHit, PublisherValues};
use crate::people::{self, PersonId};
use crate::web::{Web, WebError, host_of};

/// The number of rows whose detail a search reads (ADR 0117 item 7).
pub const DETAIL_LIMIT: usize = 10;

/// The level of a row, in the sequence of the sheet (ADR 0117 item 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// The confirmed links agree on a role.
    Role,
    /// Two-way links, and no role.
    TwoWay,
    /// No two-way link. Only the name matches.
    NameOnly,
}

/// The kind of a row, for its type line (ADR 0117 item 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Kind {
    /// The confirmed links agree on these roles, in A to Z.
    Role {
        /// The roles, in A to Z.
        roles: Vec<String>,
    },
    /// Two-way links, and a release artist that matches the name.
    InferredArtist,
    /// Two-way links, and no role.
    NoRole,
    /// No two-way link.
    Unverified,
}

/// The presence of each value of a feed, from its detail (ADR 0117 item 7).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "ADR 0117 item 7: four values, each present or absent"
)]
pub struct Has {
    /// The feed has a picture address.
    pub picture: bool,
    /// The feed has a website address.
    pub website: bool,
    /// The feed has a channel payment route.
    pub wallet: bool,
    /// The feed has a Nostr key.
    pub npub: bool,
}

/// One row of a search, in its level (ADR 0117 item 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LookupRow {
    /// `podcast:guid` of the feed.
    pub feed_guid: String,
    /// The feed title.
    pub title: Option<String>,
    /// The picture address of the feed, when it is `http` or `https`.
    pub image_url: Option<String>,
    /// The level of the row.
    pub level: Level,
    /// The kind of the row.
    pub kind: Kind,
    /// The number of two-way links. Missing for an unverified row.
    pub albums: Option<i64>,
    /// The host of the feed address. Missing when the detail is not read.
    pub host: Option<String>,
    /// The values of the feed. Each value is false when the detail is not
    /// read.
    pub has: Has,
}

/// The state of a value for one person (ADR 0117 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueState {
    /// The value is new for the person.
    New,
    /// The value is the value of the person already.
    Current,
    /// The person has the wallet already.
    HasWallet,
    /// A different person has the value.
    OtherPerson {
        /// The ID of the other person.
        id: i64,
        /// The name of the other person.
        name: String,
    },
}

/// A picture, a website or a Nostr key of a feed, with its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueRow {
    /// The value.
    pub value: String,
    /// The state of the value for the person.
    pub state: ValueState,
}

/// The result of the keysend lookup of a wallet, when the route is a
/// Lightning address (ADR 0117 item 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletLookup {
    /// The host gave keysend data.
    Found,
    /// The host gave no keysend data.
    NoData,
    /// The request failed.
    Failed,
}

/// A channel payment route of a feed, with its state (ADR 0117 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WalletRow {
    /// The address of the route, such as a Lightning address or a node key.
    pub value: String,
    /// The name of the recipient, when the route gives one.
    pub name: Option<String>,
    /// The recipient type, `node` or `lnaddress`.
    pub recipient_type: String,
    /// The split of the route.
    pub split: Option<String>,
    /// The keysend result, for a Lightning address.
    pub lookup: Option<WalletLookup>,
    /// The key that a save links, in the form of the wallet sheet: the
    /// Lightning address itself, or the node key with its custom values.
    /// Missing for a route with no address.
    pub key: Option<RecipientKey>,
    /// The state of the wallet for the person.
    pub state: ValueState,
}

/// The values of one feed for one person (ADR 0117 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LookupValues {
    /// The feed title.
    pub title: Option<String>,
    /// The host of the feed address.
    pub host: Option<String>,
    /// The picture, when it is an `http` or `https` address.
    pub picture: Option<ValueRow>,
    /// The website, when it is an `http` or `https` address.
    pub website: Option<ValueRow>,
    /// Each channel payment route of type `node` or `lnaddress`.
    pub wallets: Vec<WalletRow>,
    /// The Nostr key of the feed.
    pub npub: Option<ValueRow>,
}

/// The level of a publisher row, and its kind (ADR 0117 item 4). A row with
/// no two-way link count is a name only.
#[must_use]
pub fn classify(hit: &PublisherHit) -> (Level, Kind) {
    if !hit.agreed_roles.is_empty() {
        let mut roles = hit.agreed_roles.clone();
        roles.sort();
        return (Level::Role, Kind::Role { roles });
    }
    match hit.two_way_link_count {
        Some(count) if count > 0 => {
            if names_match(hit) {
                (Level::TwoWay, Kind::InferredArtist)
            } else {
                (Level::TwoWay, Kind::NoRole)
            }
        }
        _ => (Level::NameOnly, Kind::Unverified),
    }
}

/// The lowercase words of a name, with `&` as "and" and each run of other
/// characters as one space (the `normName` rule of musicindex.org).
#[must_use]
pub fn norm_name(name: &str) -> String {
    name.to_lowercase()
        .replace('&', " and ")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// True when `credit` is one of `names`, or contains one of them as whole
/// words (the `creditMatch` rule of musicindex.org). A part of a word does
/// not match. The names are normalized here.
#[must_use]
pub fn name_match(credit: &str, names: &[String]) -> bool {
    let credit = norm_name(credit);
    if credit.is_empty() {
        return false;
    }
    let padded = format!(" {credit} ");
    names
        .iter()
        .map(|name| norm_name(name))
        .filter(|name| !name.is_empty())
        .any(|name| credit == name || padded.contains(&format!(" {name} ")))
}

/// The names of a publisher feed: its title, and its release artist when
/// the release artist is not a placeholder.
fn hit_names(hit: &PublisherHit) -> Vec<String> {
    let mut names: Vec<String> = hit.title.iter().cloned().collect();
    if hit.release_artist_source.as_deref() != Some("placeholder")
        && let Some(artist) = &hit.release_artist
    {
        names.push(artist.clone());
    }
    names
}

fn names_match(hit: &PublisherHit) -> bool {
    let names = hit_names(hit);
    hit.confirmed_release_artists
        .iter()
        .any(|credit| name_match(credit, &names))
}

/// True for an address that starts with `http://` or `https://`, in any
/// letter case (ADR 0109).
fn is_web_address(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// The trimmed text when it is an `http` or `https` address.
fn web_value(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| is_web_address(text))
}

/// True for a payment route that a person can use: a `node` or a
/// `lnaddress` route.
fn is_channel_route(recipient: &Recipient) -> bool {
    matches!(
        recipient.recipient_type.as_deref(),
        Some("node" | "lnaddress")
    )
}

/// The row of a search hit, without its detail.
fn row_of(hit: &PublisherHit) -> LookupRow {
    let (level, kind) = classify(hit);
    let albums = match kind {
        Kind::Unverified => None,
        _ => hit.two_way_link_count,
    };
    LookupRow {
        feed_guid: hit.feed_guid.clone(),
        title: hit.title.clone(),
        image_url: web_value(hit.image_url.clone()),
        level,
        kind,
        albums,
        host: None,
        has: Has::default(),
    }
}

fn apply_detail(row: &mut LookupRow, values: &PublisherValues) {
    row.host = values.feed_url.as_deref().and_then(host_of);
    row.has = Has {
        picture: web_value(values.image_url.clone()).is_some(),
        website: web_value(values.website.clone()).is_some(),
        wallet: values.routes.iter().any(is_channel_route),
        npub: values.npub.is_some(),
    };
}

/// Search the publisher feeds of musicindex for a name, in three levels
/// (ADR 0117 items 3 and 4). The rows keep the sequence of the search in
/// each level. The detail of the first [`DETAIL_LIMIT`] rows is read. A
/// failed detail read leaves its row with no host and no value.
///
/// # Errors
///
/// Gives a [`WebError`] when the search request fails.
pub fn search(
    index: &MusicIndex,
    request: &IndexSearchRequest,
) -> Result<Vec<LookupRow>, WebError> {
    let mut rows: Vec<LookupRow> = index
        .search_publishers(request)?
        .iter()
        .map(row_of)
        .collect();
    rows.sort_by_key(|row| row.level);
    for row in rows.iter_mut().take(DETAIL_LIMIT) {
        if let Ok(Some(values)) = index.publisher_values(&row.feed_guid) {
            apply_detail(row, &values);
        }
    }
    Ok(rows)
}

/// One channel payment route of a feed, with its keysend result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteValue {
    /// The route, as a recipient.
    pub recipient: Recipient,
    /// The keysend result, for a Lightning address with a scheme.
    pub lookup: Option<LookupResult>,
}

/// The values of one feed, read from musicindex and from keysend. It holds
/// no collection data (see [`compose_values`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedValues {
    /// The feed title.
    pub title: Option<String>,
    /// The host of the feed address.
    pub host: Option<String>,
    /// The picture address, when it is `http` or `https`.
    pub picture: Option<String>,
    /// The website address, when it is `http` or `https`.
    pub website: Option<String>,
    /// The Nostr key of the feed.
    pub npub: Option<String>,
    /// Each channel payment route.
    pub routes: Vec<RouteValue>,
}

/// Read the values of one feed from musicindex, and the keysend result of
/// each Lightning address route (ADR 0117 items 5 and 6). The UI takes the
/// kind of the feed from its search row. Gives `None` when musicindex does
/// not know the feed. The keysend request needs a `scheme`, and a shell with
/// no scheme gives no keysend result. The function reads no collection.
///
/// # Errors
///
/// Gives a [`WebError`] when a musicindex request fails.
pub fn fetch_values(
    index: &MusicIndex,
    web: &Web,
    scheme: Option<&str>,
    feed_guid: &str,
) -> Result<Option<FeedValues>, WebError> {
    let Some(values) = index.publisher_values(feed_guid)? else {
        return Ok(None);
    };
    let routes = values
        .routes
        .into_iter()
        .filter(is_channel_route)
        .map(|recipient| {
            let lookup = match (
                recipient.recipient_type.as_deref(),
                scheme,
                recipient.address.as_deref(),
            ) {
                (Some("lnaddress"), Some(scheme), Some(address)) => {
                    Some(keysend::request(web, scheme, address))
                }
                _ => None,
            };
            RouteValue { recipient, lookup }
        })
        .collect();
    Ok(Some(FeedValues {
        title: values.title,
        host: values.feed_url.as_deref().and_then(host_of),
        picture: web_value(values.image_url),
        website: web_value(values.website),
        npub: values.npub,
        routes,
    }))
}

fn wallet_lookup(result: &LookupResult) -> WalletLookup {
    match result {
        LookupResult::Found { .. } => WalletLookup::Found,
        LookupResult::None => WalletLookup::NoData,
        LookupResult::Failed => WalletLookup::Failed,
    }
}

/// The key that a save links for a route. A Lightning address is linked as
/// the address, as the wallet sheet links it. The keysend lookup that the
/// save stores joins it to its node (ADR 0095).
fn save_key_of(recipient: &Recipient) -> Option<RecipientKey> {
    let address = recipient.address.as_deref()?.trim();
    if address.is_empty() {
        return None;
    }
    Some(
        if recipient.recipient_type.as_deref() == Some("lnaddress") {
            RecipientKey {
                address: address.to_lowercase(),
                custom_key: None,
                custom_value: None,
            }
        } else {
            RecipientKey {
                address: address.to_owned(),
                custom_key: recipient.custom_key.clone(),
                custom_value: recipient.custom_value.clone(),
            }
        },
    )
}

/// The wallet key of a route, to find the person of the wallet: the node key
/// of a Lightning address with a found lookup, or the key of a `node` route.
/// Gives `None` otherwise.
fn wallet_key_of(recipient: &Recipient, lookup: Option<&LookupResult>) -> Option<RecipientKey> {
    match lookup {
        Some(LookupResult::Found {
            pubkey,
            custom_key,
            custom_value,
        }) => Some(RecipientKey {
            address: pubkey.clone(),
            custom_key: custom_key.clone(),
            custom_value: custom_value.clone(),
        }),
        None if recipient.recipient_type.as_deref() == Some("node") => Some(RecipientKey {
            address: recipient.address.clone()?,
            custom_key: recipient.custom_key.clone(),
            custom_value: recipient.custom_value.clone(),
        }),
        Some(_) | None => None,
    }
}

/// The state of a Nostr key for a person: the same person, a different
/// person, or new.
fn key_state(
    collection: &Collection,
    id: PersonId,
    npub: &str,
) -> Result<ValueState, CollectionError> {
    for other in people::people(collection)? {
        if other.keys.iter().any(|key| key.npub == npub) {
            return Ok(if other.id == id {
                ValueState::Current
            } else {
                ValueState::OtherPerson {
                    id: other.id.0,
                    name: other.name,
                }
            });
        }
    }
    Ok(ValueState::New)
}

/// Give the values of a feed for a person, with the state of each value
/// (ADR 0117 items 5 and 6). The function reads the collection and makes no
/// request.
///
/// # Errors
///
/// Gives a [`CollectionError`] when the person is unknown or SQLite fails.
pub fn compose_values(
    collection: &Collection,
    id: PersonId,
    feed: FeedValues,
) -> Result<LookupValues, CollectionError> {
    let person = people::person(collection, id)?;
    let lookups: HashMap<String, Lookup> = people::lookup_map(collection)?;
    let picture = match feed.picture {
        Some(value) => {
            let state = if person.img.trim() == value {
                ValueState::Current
            } else {
                ValueState::New
            };
            Some(ValueRow { value, state })
        }
        None => None,
    };
    let website = match feed.website {
        Some(value) => {
            let state = if person.href.trim() == value {
                ValueState::Current
            } else {
                ValueState::New
            };
            Some(ValueRow { value, state })
        }
        None => None,
    };
    let npub = match feed.npub {
        Some(npub) => {
            let npub = npub.trim().to_lowercase();
            let state = key_state(collection, id, &npub)?;
            Some(ValueRow { value: npub, state })
        }
        None => None,
    };
    let mut wallets = Vec::new();
    for route in feed.routes {
        let key = save_key_of(&route.recipient);
        let wallet_key =
            wallet_key_of(&route.recipient, route.lookup.as_ref()).or_else(|| key.clone());
        let state = match &wallet_key {
            None => ValueState::New,
            Some(key) => match people::person_of_wallet(collection, key, &lookups)? {
                Some(owner) if owner == id => ValueState::HasWallet,
                Some(owner) => ValueState::OtherPerson {
                    id: owner.0,
                    name: people::person(collection, owner)?.name,
                },
                None => ValueState::New,
            },
        };
        let lookup = route.lookup.as_ref().map(wallet_lookup);
        let recipient = route.recipient;
        wallets.push(WalletRow {
            value: recipient.address.unwrap_or_default(),
            name: recipient.name,
            recipient_type: recipient.recipient_type.unwrap_or_default(),
            split: recipient.split,
            lookup,
            key,
            state,
        });
    }
    Ok(LookupValues {
        title: feed.title,
        host: feed.host,
        picture,
        website,
        wallets,
        npub,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestServer;

    fn hit(guid: &str) -> PublisherHit {
        PublisherHit {
            feed_guid: guid.to_owned(),
            title: Some("Ann Lee".to_owned()),
            image_url: None,
            release_artist: Some("Ann Lee".to_owned()),
            release_artist_source: Some("itunes_author".to_owned()),
            two_way_link_count: None,
            agreed_roles: Vec::new(),
            confirmed_release_artists: Vec::new(),
        }
    }

    #[test]
    fn classify_gives_each_level_and_kind() {
        let role = PublisherHit {
            two_way_link_count: Some(3),
            agreed_roles: vec!["label".to_owned(), "artist".to_owned()],
            ..hit("g1")
        };
        assert_eq!(
            classify(&role),
            (
                Level::Role,
                Kind::Role {
                    roles: vec!["artist".to_owned(), "label".to_owned()]
                }
            )
        );
        let inferred = PublisherHit {
            two_way_link_count: Some(2),
            confirmed_release_artists: vec!["Ann Lee & Bo".to_owned()],
            ..hit("g2")
        };
        assert_eq!(classify(&inferred), (Level::TwoWay, Kind::InferredArtist));
        let no_role = PublisherHit {
            two_way_link_count: Some(2),
            confirmed_release_artists: vec!["Someone Else".to_owned()],
            ..hit("g3")
        };
        assert_eq!(classify(&no_role), (Level::TwoWay, Kind::NoRole));
        let zero = PublisherHit {
            two_way_link_count: Some(0),
            ..hit("g4")
        };
        assert_eq!(classify(&zero), (Level::NameOnly, Kind::Unverified));
        assert_eq!(classify(&hit("g5")), (Level::NameOnly, Kind::Unverified));
    }

    #[test]
    fn a_placeholder_release_artist_is_no_name() {
        let placeholder = PublisherHit {
            two_way_link_count: Some(1),
            release_artist_source: Some("placeholder".to_owned()),
            confirmed_release_artists: vec!["Ann Lee".to_owned()],
            title: Some("Other".to_owned()),
            release_artist: Some("Ann Lee".to_owned()),
            ..hit("g6")
        };
        assert_eq!(
            classify(&placeholder),
            (Level::TwoWay, Kind::NoRole),
            "the release artist is a placeholder, so only the title counts"
        );
    }

    #[test]
    fn name_match_gives_equal_and_whole_word_names_only() {
        let names = vec!["Ann Lee".to_owned()];
        assert!(name_match("ann lee", &names));
        assert!(name_match("Ann Lee & Bo", &names));
        assert!(name_match("Bo and Ann-Lee", &names));
        assert!(!name_match("Anne Lee", &names));
        assert!(!name_match("Mann Lee", &names));
        assert!(!name_match("", &names));
        assert_eq!(norm_name("Ann  &  Bo!"), "ann and bo");
    }

    /// The search rows of the test server. Each feed has a route.
    fn search_body(count: usize) -> Vec<u8> {
        let rows: Vec<String> = (0..count)
            .map(|i| {
                // The rows alternate between a name only and a role, so
                // that the sort is visible.
                let (links, roles) = if i % 2 == 0 {
                    (0, "[]")
                } else {
                    (2, r#"["artist"]"#)
                };
                format!(
                    r#"{{"entity_type":"publisher","entity_id":"c0ffee-{i:02}","title":"Row {i}","release_artist":"Row {i}","release_artist_source":"itunes_author","two_way_link_count":{links},"agreed_roles":{roles},"confirmed_release_artists":[]}}"#
                )
            })
            .collect();
        format!(r#"{{"data":[{}]}}"#, rows.join(",")).into_bytes()
    }

    fn feed_body(website: &str) -> Vec<u8> {
        format!(
            r#"{{"data":{{"title":"Feed","feed_url":"https://feed.example/x.xml","image_url":"https://img.example/a.png",
                "source_links":[{{"link_type":"website","url":"{website}"}}],
                "source_ids":[{{"scheme":"nostr_npub","value":"npub1feed"}}],
                "payment_routes":[{{"recipient_name":"Ann","route_type":"node","address":"02node","split":100,"fee":false}}]}}}}"#
        )
        .into_bytes()
    }

    #[test]
    fn search_keeps_the_sequence_in_a_level_and_reads_ten_rows_only() {
        let server = TestServer::start(vec![("/v1/search", 200, search_body(12))]);
        for i in 0..12 {
            server.set(
                &format!("/v1/feeds/c0ffee-{i:02}"),
                200,
                feed_body("https://site.example"),
            );
        }
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let rows = search(&index, &IndexSearchRequest::words("row")).unwrap();
        assert_eq!(rows.len(), 12);
        // The role rows come first, in the sequence of the search.
        let guids: Vec<&str> = rows.iter().map(|r| r.feed_guid.as_str()).collect();
        assert_eq!(
            &guids[..6],
            [
                "c0ffee-01",
                "c0ffee-03",
                "c0ffee-05",
                "c0ffee-07",
                "c0ffee-09",
                "c0ffee-11"
            ]
        );
        assert_eq!(rows[0].level, Level::Role);
        assert_eq!(rows[6].level, Level::NameOnly);
        // The detail is read for the first ten rows in that sequence only.
        for (position, row) in rows.iter().enumerate() {
            if position < DETAIL_LIMIT {
                assert_eq!(row.host.as_deref(), Some("feed.example"), "row {position}");
                assert!(row.has.picture && row.has.website && row.has.wallet && row.has.npub);
            } else {
                assert_eq!(row.host, None, "row {position}");
                assert_eq!(row.has, Has::default(), "row {position}");
            }
        }
    }

    #[test]
    fn values_give_the_states_and_refuse_a_picture_that_is_not_web() {
        let server = TestServer::start(vec![
            ("/v1/feeds/c0ffee-50", 200, feed_body("https://ann.example")),
            ("/v1/feeds/c0ffee-51", 200, br#"{"data":{"title":"Feed","image_url":"ftp://img.example/a.png","source_links":[],"source_ids":[],"payment_routes":[]}}"#.to_vec()),
            ("/v1/search", 200, br#"{"data":[]}"#.to_vec()),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let web = Web::new();

        let collection = Collection::open_in_memory().unwrap();
        let ann = people::add_person(
            &collection,
            &people::PersonFields {
                name: "Ann".to_owned(),
                img: "https://img.example/a.png".to_owned(),
                ..people::PersonFields::default()
            },
        )
        .unwrap();
        let bob = people::add_person(
            &collection,
            &people::PersonFields {
                name: "Bob".to_owned(),
                ..people::PersonFields::default()
            },
        )
        .unwrap();
        people::add_wallet(
            &collection,
            ann,
            &RecipientKey {
                address: "02node".to_owned(),
                custom_key: None,
                custom_value: None,
            },
            "node",
            people::LinkEvidence::User,
        )
        .unwrap();
        people::add_key(&collection, bob, "npub1feed").unwrap();

        let feed = fetch_values(&index, &web, None, "c0ffee-50")
            .unwrap()
            .unwrap();
        assert_eq!(feed.picture.as_deref(), Some("https://img.example/a.png"));
        let values = compose_values(&collection, ann, feed).unwrap();
        assert_eq!(
            values.picture,
            Some(ValueRow {
                value: "https://img.example/a.png".to_owned(),
                state: ValueState::Current
            })
        );
        assert_eq!(
            values.website,
            Some(ValueRow {
                value: "https://ann.example".to_owned(),
                state: ValueState::New
            })
        );
        assert_eq!(
            values.npub.as_ref().map(|row| &row.state),
            Some(&ValueState::OtherPerson {
                id: bob.0,
                name: "Bob".to_owned()
            })
        );
        assert_eq!(values.wallets.len(), 1);
        assert_eq!(values.wallets[0].state, ValueState::HasWallet);
        assert_eq!(values.wallets[0].lookup, None);
        assert_eq!(values.host.as_deref(), Some("feed.example"));

        // A different person gives the wallet, and the state names that person.
        let feed = fetch_values(&index, &web, None, "c0ffee-50")
            .unwrap()
            .unwrap();
        let values = compose_values(&collection, bob, feed).unwrap();
        assert_eq!(
            values.wallets[0].state,
            ValueState::OtherPerson {
                id: ann.0,
                name: "Ann".to_owned()
            }
        );
        assert_eq!(
            values.npub.as_ref().map(|row| &row.state),
            Some(&ValueState::Current)
        );

        let refused = fetch_values(&index, &web, None, "c0ffee-51")
            .unwrap()
            .unwrap();
        assert_eq!(refused.picture, None);
        assert_eq!(
            fetch_values(&index, &web, None, "00000000-0000-0000-0000-000000000000").unwrap(),
            None
        );
    }

    #[test]
    fn a_lightning_address_route_gets_the_keysend_lookup() {
        let server = TestServer::start(Vec::new());
        let address = format!("ann@127.0.0.1:{}", server.port());
        server.set(
            "/.well-known/keysend/ann",
            200,
            br#"{"pubkey":"02ln","customData":[]}"#.to_vec(),
        );
        server.set(
            "/v1/feeds/c0ffee-60",
            200,
            format!(
                r#"{{"data":{{"title":"Feed","payment_routes":[{{"recipient_name":"Ann","route_type":"lnaddress","address":"{address}","split":5,"fee":false}}]}}}}"#
            )
            .into_bytes(),
        );
        server.set("/v1/search", 200, br#"{"data":[]}"#.to_vec());
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let web = Web::new();
        let collection = Collection::open_in_memory().unwrap();
        let ann = people::add_person(
            &collection,
            &people::PersonFields {
                name: "Ann".to_owned(),
                ..people::PersonFields::default()
            },
        )
        .unwrap();

        let feed = fetch_values(&index, &web, Some("http"), "c0ffee-60")
            .unwrap()
            .unwrap();
        let values = compose_values(&collection, ann, feed).unwrap();
        assert_eq!(values.wallets[0].lookup, Some(WalletLookup::Found));
        // A save links the address, as the wallet sheet does.
        let address_key = Some(RecipientKey {
            address: address.clone(),
            custom_key: None,
            custom_value: None,
        });
        assert_eq!(values.wallets[0].key, address_key);
        assert_eq!(values.wallets[0].state, ValueState::New);
        // The node of the lookup finds a different person with that node.
        let bob = people::add_person(
            &collection,
            &people::PersonFields {
                name: "Bob".to_owned(),
                ..people::PersonFields::default()
            },
        )
        .unwrap();
        people::add_wallet(
            &collection,
            bob,
            &RecipientKey {
                address: "02ln".to_owned(),
                custom_key: None,
                custom_value: None,
            },
            "node",
            people::LinkEvidence::User,
        )
        .unwrap();
        let feed = fetch_values(&index, &web, Some("http"), "c0ffee-60")
            .unwrap()
            .unwrap();
        let values = compose_values(&collection, ann, feed).unwrap();
        assert_eq!(
            values.wallets[0].state,
            ValueState::OtherPerson {
                id: bob.0,
                name: "Bob".to_owned(),
            }
        );
        // The lookup adds nothing to the collection.
        assert_eq!(collection.keysend_lookups().unwrap().len(), 0);

        // With no scheme, the shell makes no keysend request.
        let feed = fetch_values(&index, &web, None, "c0ffee-60")
            .unwrap()
            .unwrap();
        let values = compose_values(&collection, ann, feed).unwrap();
        assert_eq!(values.wallets[0].lookup, None);
        assert_eq!(values.wallets[0].key, address_key);
    }
}
