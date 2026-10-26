//! The public address of a feed (ADR 0021).
//!
//! Each source can give one address for a feed. The collection keeps the
//! address of each source, and the public address is the address of the
//! first source in [`SEQUENCE`]. Studio does not select silently: each
//! address that is different from the public address is a difference that
//! the UI shows.

use serde::Serialize;

use crate::collection::{Collection, CollectionError, ConnectorId, ConnectorRow, FeedId, FeedRow};
use crate::connector::{
    self, ConnectorError, LocalFolder, S3Bucket, S3Config, SftpConfig, SftpHost,
};
use crate::vault::OpenVault;

/// A source of a feed address (ADR 0021 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressSource {
    /// The address where Studio read the feed, after each redirect.
    Read,
    /// The `href` of the first `atom:link` with `rel="self"` in the feed.
    SelfLink,
    /// The public address of the folder connector, with the file path added.
    Folder,
    /// `itunes:new-feed-url`. Studio does not use it as the public address
    /// without a user action (ADR 0021 item 4).
    NewFeedUrl,
}

/// The sources that can give the public address, first source first.
pub const SEQUENCE: [AddressSource; 3] = [
    AddressSource::Read,
    AddressSource::SelfLink,
    AddressSource::Folder,
];

impl AddressSource {
    /// The text of the source in the collection.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::SelfLink => "self_link",
            Self::Folder => "folder",
            Self::NewFeedUrl => "new_feed_url",
        }
    }

    /// The source for its text in the collection.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        [Self::Read, Self::SelfLink, Self::Folder, Self::NewFeedUrl]
            .into_iter()
            .find(|s| s.as_str() == text)
    }

    /// True when the address of this source proves itself, without the
    /// folder: a read or a `self` link (ADR 0021 item 6).
    #[must_use]
    pub fn is_from_feed(self) -> bool {
        matches!(self, Self::Read | Self::SelfLink)
    }
}

/// An address and the source that gave it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Address {
    /// The source.
    pub source: AddressSource,
    /// The address.
    pub url: String,
}

/// The public address: the address of the first source in [`SEQUENCE`]
/// that gives one.
#[must_use]
pub fn choose(addresses: &[Address]) -> Option<&Address> {
    SEQUENCE
        .iter()
        .find_map(|source| addresses.iter().find(|a| a.source == *source))
}

/// Each address that is different from the public address (ADR 0021 items
/// 3 and 4).
#[must_use]
pub fn differences(addresses: &[Address]) -> Vec<Address> {
    let chosen = choose(addresses).map(|a| a.url.as_str());
    addresses
        .iter()
        .filter(|a| chosen.is_some_and(|c| c != a.url))
        .cloned()
        .collect()
}

/// The possible folder address that a feed address gives: the start of the
/// address, when the address ends with the file path (ADR 0021 item 6).
#[must_use]
pub fn folder_candidate(url: &str, path: &str) -> Option<String> {
    let start = url.strip_suffix(path)?.strip_suffix('/')?;
    let after_scheme = start.split_once("://")?.1;
    (!after_scheme.is_empty()).then(|| start.to_owned())
}

/// The folder address that the linked feeds of a folder give.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FolderAddress {
    /// The user typed the folder address. Studio does not replace it.
    Typed {
        /// The folder address.
        url: String,
    },
    /// Studio found the folder address from the feeds.
    Inferred {
        /// The folder address.
        url: String,
        /// True when Studio found it in this operation.
        new: bool,
    },
    /// The candidates are different, so the folder has no address.
    Different,
    /// No feed gives a candidate, so the folder has no address.
    Unknown,
}

/// The feed path and the public address of each linked feed. Only an
/// address from a read or a `self` link is in the list.
fn feed_addresses(feeds: &[FeedRow]) -> impl Iterator<Item = (&FeedRow, &str, &str)> {
    feeds.iter().filter_map(|f| {
        let source = f.address_source?;
        let url = f.public_url.as_deref()?;
        let path = f.path.as_deref()?;
        source.is_from_feed().then_some((f, path, url))
    })
}

/// The candidates of the linked feeds of a folder.
enum Candidates {
    /// No feed gives a candidate.
    None,
    /// All the candidates are the same.
    Agreed(String),
    /// Two or more candidates are different.
    Different,
}

fn candidates(feeds: &[FeedRow]) -> Candidates {
    let mut agreed = Candidates::None;
    for (_, path, url) in feed_addresses(feeds) {
        let Some(candidate) = folder_candidate(url, path) else {
            continue;
        };
        match &agreed {
            Candidates::Agreed(known) if *known != candidate => return Candidates::Different,
            Candidates::Agreed(_) | Candidates::Different => {}
            Candidates::None => agreed = Candidates::Agreed(candidate),
        }
    }
    agreed
}

/// Find the folder address of a connector from its linked feeds, and give
/// each linked feed its folder address (ADR 0021 items 6 to 10).
///
/// Studio sets an inferred address only when the folder has no address and
/// all the candidates are the same. It does not replace an address that the
/// user typed. An S3 host has only a typed address.
///
/// The vault gives the secret of a connector that needs one. A host of the
/// kind `s3` thus needs the open vault.
///
/// # Errors
///
/// Gives an [`AddressError`] when the collection fails or the connector
/// configuration is not valid.
pub fn apply_folder_address(
    collection: &mut Collection,
    id: ConnectorId,
    vault: Option<&OpenVault>,
) -> Result<FolderAddress, AddressError> {
    let row = collection.connector(id)?;
    let linked: Vec<FeedRow> = collection
        .feeds()?
        .into_iter()
        .filter(|f| f.connector_id == Some(id))
        .collect();
    let outcome = if row.kind == LocalFolder::KIND {
        let folder = LocalFolder::from_config(&row.config)?;
        match (folder.base_url(), candidates(&linked)) {
            (Some(url), _) if folder.base_url_inferred() => FolderAddress::Inferred {
                url: url.to_owned(),
                new: false,
            },
            (Some(url), _) => FolderAddress::Typed {
                url: url.to_owned(),
            },
            (None, Candidates::Different) => FolderAddress::Different,
            (None, Candidates::None) => FolderAddress::Unknown,
            (None, Candidates::Agreed(url)) => {
                let inferred = folder.with_inferred_base_url(url.clone());
                collection.set_connector_config(id, &inferred.config())?;
                FolderAddress::Inferred { url, new: true }
            }
        }
    } else if let Some(url) = folder_url(&row) {
        FolderAddress::Typed { url }
    } else {
        FolderAddress::Unknown
    };
    let made = connector::from_row(&collection.connector(id)?, vault)?;
    for feed in &linked {
        if let Some(path) = &feed.path {
            collection.set_address(
                feed.id,
                AddressSource::Folder,
                made.public_url(path).as_deref(),
            )?;
        }
    }
    Ok(outcome)
}

/// The public address of a folder connector, typed or inferred, or the
/// public address of an S3 or SFTP connector. Other connector kinds give
/// `None`.
#[must_use]
pub fn folder_url(row: &ConnectorRow) -> Option<String> {
    match row.kind.as_str() {
        LocalFolder::KIND => LocalFolder::from_config(&row.config)
            .ok()
            .and_then(|f| f.base_url().map(str::to_owned)),
        S3Bucket::KIND => serde_json::from_str::<S3Config>(&row.config)
            .ok()
            .and_then(|c| c.base_url),
        SftpHost::KIND => serde_json::from_str::<SftpConfig>(&row.config)
            .ok()
            .and_then(|c| c.base_url),
        _ => None,
    }
}

/// The linked feeds of a folder with an address that does not agree with
/// the folder (ADR 0021 item 8).
///
/// When the folder has an address, these are the feeds with a public
/// address that is different from the folder address. When it has none, and the
/// candidates are different, these are the feeds that give a candidate.
#[must_use]
pub fn folder_disagreements(has_folder_address: bool, linked: &[FeedRow]) -> Vec<FeedId> {
    if has_folder_address {
        return linked
            .iter()
            .filter(|f| {
                f.address_differences
                    .iter()
                    .any(|d| d.source == AddressSource::Folder)
            })
            .map(|f| f.id)
            .collect();
    }
    if !matches!(candidates(linked), Candidates::Different) {
        return Vec::new();
    }
    feed_addresses(linked)
        .filter(|(_, path, url)| folder_candidate(url, path).is_some())
        .map(|(f, _, _)| f.id)
        .collect()
}

/// An error from [`apply_folder_address`].
#[derive(Debug)]
#[non_exhaustive]
pub enum AddressError {
    /// The collection failed.
    Collection(CollectionError),
    /// The connector configuration is not valid.
    Connector(ConnectorError),
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collection(e) => e.fmt(f),
            Self::Connector(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for AddressError {}

impl From<CollectionError> for AddressError {
    fn from(error: CollectionError) -> Self {
        Self::Collection(error)
    }
}

impl From<ConnectorError> for AddressError {
    fn from(error: ConnectorError) -> Self {
        Self::Connector(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(source: AddressSource, url: &str) -> Address {
        Address {
            source,
            url: url.to_owned(),
        }
    }

    #[test]
    fn the_first_source_gives_the_address_and_each_other_address_is_a_difference() {
        let read = address(AddressSource::Read, "https://a.example/f.xml");
        let stale = address(AddressSource::SelfLink, "https://old.example/f.xml");
        let folder = address(AddressSource::Folder, "https://a.example/f.xml");
        let all = [folder.clone(), stale.clone(), read.clone()];
        assert_eq!(choose(&all), Some(&read));
        assert_eq!(differences(&all), std::slice::from_ref(&stale));
        assert_eq!(choose(&[folder.clone(), stale.clone()]), Some(&stale));
        assert_eq!(choose(std::slice::from_ref(&folder)), Some(&folder));
        assert_eq!(choose(&[]), None);
    }

    #[test]
    fn a_new_feed_url_is_a_difference_and_never_the_address() {
        let moved = address(AddressSource::NewFeedUrl, "https://new.example/f.xml");
        assert_eq!(choose(std::slice::from_ref(&moved)), None);
        assert_eq!(differences(std::slice::from_ref(&moved)), Vec::new());
        let read = address(AddressSource::Read, "https://a.example/f.xml");
        assert_eq!(differences(&[read, moved.clone()]), [moved]);
    }

    /// A folder with no address and three linked feeds. Two feeds have read
    /// addresses, and the third feed came from the folder.
    fn folder(a: &str, b: &str, base_url: Option<&str>) -> (Collection, ConnectorId, [FeedId; 3]) {
        let mut collection = Collection::open_in_memory().unwrap();
        let config = LocalFolder::new("/srv", base_url.map(str::to_owned)).config();
        let id = collection
            .add_connector(LocalFolder::KIND, "Home", &config, None)
            .unwrap();
        let first = collection
            .add_feed(Some(a), Some((id, "music/a.xml")))
            .unwrap();
        let second = collection.add_feed(Some(b), Some((id, "b.xml"))).unwrap();
        let third = collection.add_feed(None, Some((id, "c.xml"))).unwrap();
        (collection, id, [first, second, third])
    }

    fn disagreements(collection: &Collection, id: ConnectorId) -> Vec<FeedId> {
        let row = collection.connector(id).unwrap();
        let linked: Vec<FeedRow> = collection.feeds().unwrap();
        folder_disagreements(folder_url(&row).is_some(), &linked)
    }

    #[test]
    fn an_s3_host_gives_its_public_address_as_a_typed_address() {
        let collection = Collection::open_in_memory().unwrap();
        let config = |base_url: Option<&str>| {
            serde_json::json!({
                "endpoint": "https://s3.example.com",
                "region": "auto",
                "bucket": "bk",
                "url_style": "path",
                "prefix": "",
                "base_url": base_url,
            })
            .to_string()
        };
        let with = collection
            .add_connector(
                S3Bucket::KIND,
                "R2",
                &config(Some("https://cdn.example")),
                None,
            )
            .unwrap();
        let without = collection
            .add_connector(S3Bucket::KIND, "R2 b", &config(None), None)
            .unwrap();
        assert_eq!(
            folder_url(&collection.connector(with).unwrap()).as_deref(),
            Some("https://cdn.example")
        );
        assert_eq!(folder_url(&collection.connector(without).unwrap()), None);
    }

    #[test]
    fn an_sftp_host_gives_its_public_address_as_a_typed_address() {
        let collection = Collection::open_in_memory().unwrap();
        let config = |base_url: Option<&str>| {
            serde_json::json!({
                "host": "sftp.example.com",
                "port": 22,
                "user": "me",
                "root": "/srv/feeds",
                "base_url": base_url,
                "login": "agent",
                "host_key": "SHA256:abc",
            })
            .to_string()
        };
        let with = collection
            .add_connector(
                SftpHost::KIND,
                "Host",
                &config(Some("https://cdn.example")),
                None,
            )
            .unwrap();
        let without = collection
            .add_connector(SftpHost::KIND, "Host b", &config(None), None)
            .unwrap();
        assert_eq!(
            folder_url(&collection.connector(with).unwrap()).as_deref(),
            Some("https://cdn.example")
        );
        assert_eq!(folder_url(&collection.connector(without).unwrap()), None);
    }

    #[test]
    fn candidates_that_agree_give_the_folder_address_and_the_feed_from_the_folder_gets_it() {
        let (mut collection, id, [_, _, third]) = folder(
            "https://m.example/feeds/music/a.xml",
            "https://m.example/feeds/b.xml",
            None,
        );
        let outcome = apply_folder_address(&mut collection, id, None).unwrap();
        let url = "https://m.example/feeds".to_owned();
        assert_eq!(
            outcome,
            FolderAddress::Inferred {
                url: url.clone(),
                new: true
            }
        );
        let row = collection.connector(id).unwrap();
        assert_eq!(folder_url(&row), Some(url.clone()));
        let feed = collection.feed(third).unwrap();
        assert_eq!(
            feed.public_url.as_deref(),
            Some("https://m.example/feeds/c.xml")
        );
        assert_eq!(feed.address_source, Some(AddressSource::Folder));
        assert_eq!(disagreements(&collection, id), Vec::new());
        // A second match keeps the inferred address.
        assert_eq!(
            apply_folder_address(&mut collection, id, None).unwrap(),
            FolderAddress::Inferred { url, new: false }
        );
    }

    #[test]
    fn candidates_that_disagree_give_no_folder_address_and_the_host_lists_them() {
        let (mut collection, id, [first, second, third]) = folder(
            "https://m.example/music/a.xml",
            "https://cdn.example/b.xml",
            None,
        );
        let outcome = apply_folder_address(&mut collection, id, None).unwrap();
        assert_eq!(outcome, FolderAddress::Different);
        assert_eq!(folder_url(&collection.connector(id).unwrap()), None);
        assert_eq!(collection.feed(third).unwrap().public_url, None);
        assert_eq!(disagreements(&collection, id), [first, second]);
    }

    #[test]
    fn an_address_that_the_user_typed_stays_and_the_host_lists_each_other_feed() {
        let (mut collection, id, [first, second, third]) = folder(
            "https://m.example/music/a.xml",
            "https://cdn.example/b.xml",
            Some("https://m.example"),
        );
        let outcome = apply_folder_address(&mut collection, id, None).unwrap();
        assert_eq!(
            outcome,
            FolderAddress::Typed {
                url: "https://m.example".to_owned()
            }
        );
        assert_eq!(
            folder_url(&collection.connector(id).unwrap()).as_deref(),
            Some("https://m.example")
        );
        assert_eq!(
            collection.feed(second).unwrap().public_url.as_deref(),
            Some("https://cdn.example/b.xml")
        );
        assert_eq!(
            collection.feed(third).unwrap().public_url.as_deref(),
            Some("https://m.example/c.xml")
        );
        assert_eq!(disagreements(&collection, id), [second]);
        assert!(!disagreements(&collection, id).contains(&first));
    }

    #[test]
    fn a_candidate_is_the_start_of_an_address_that_ends_with_the_path() {
        assert_eq!(
            folder_candidate("https://music.example/feeds/a/album.xml", "a/album.xml").as_deref(),
            Some("https://music.example/feeds")
        );
        assert_eq!(
            folder_candidate("https://music.example/album.xml", "album.xml").as_deref(),
            Some("https://music.example")
        );
        assert_eq!(
            folder_candidate("https://music.example/myalbum.xml", "album.xml"),
            None
        );
        assert_eq!(folder_candidate("https:///album.xml", "album.xml"), None);
        assert_eq!(folder_candidate("album.xml", "album.xml"), None);
    }
}
