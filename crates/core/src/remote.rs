//! The records of remote files: the library files whose bytes are not in
//! the library (ADR 0204 items 1 to 3).
//!
//! A published feed has only its feed XML in its release folder. Each
//! audio file and image of the feed is on a host. This module gives each
//! address of the feed a library file with `local` false and no SHA-256.
//! It changes no file on disk.

use std::collections::{HashMap, HashSet};

use crate::collection::{
    Collection, CollectionError, CollectionErrorKind, FeedId, LibraryFile, LibraryFileId,
};
use crate::feed::Feed;
use crate::inventory::{self, Hosts, InventoryError, InventoryErrorKind};
use crate::library::is_plain_name;
use crate::media::plain_address;
use crate::pipeline::feed_addresses;

/// The result of [`record_remote_files`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemoteReport {
    /// The number of rows that the call added.
    pub added: usize,
    /// The number of rows that the call removed.
    pub removed: usize,
}

/// What the feed says about one address.
struct Facts {
    /// The `length` of the enclosure or of the alternative enclosure,
    /// when the feed gives one.
    size: Option<u64>,
    /// The media type from the feed, when the feed gives one.
    media_type: Option<String>,
}

/// Keep one library file for each address of a feed with a release folder
/// (ADR 0204 items 1 to 3).
///
/// The function skips an address that is the `source_url` of a row of the
/// feed. It also skips an address that is the planned address of a local
/// library file, or the address of a copy of that file. It adds a row
/// with `local` false for each other address. It removes each row with
/// `local` false whose address is no longer in `feed`, with its download
/// record.
///
/// A row with `local` false, no SHA-256 and no removal time is a second
/// row of a local file when its address is the planned address of that
/// file on a connected host, or the address of a copy of that file. The
/// function gives the address to the local row as its `source_url` and
/// removes the remote row. It keeps a remote row that a draft operation
/// names (ADR 0231 items 12 and 13).
///
/// When a read of the planned address or of the copies of a local library
/// file fails, the function gives up and adds no row (ADR 0179 item 1).
/// A host that Studio cannot read would else make its addresses look like
/// other addresses. A local file with no host has no planned address, and
/// that is no failure.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails, or when a read of a host
/// fails. The next call tries again.
pub fn record_remote_files(
    collection: &Collection,
    hosts: &Hosts,
    feed_id: FeedId,
    feed: &Feed,
) -> Result<RemoteReport, CollectionError> {
    let row = collection.feed(feed_id)?;
    let addresses = feed_addresses(feed);
    let mut report = RemoteReport::default();

    // Remove each remote row whose address is not in the feed.
    let files = collection.library_files(feed_id)?;
    let mut kept = Vec::new();
    for file in files {
        let gone = !file.local
            && file
                .source_url
                .as_deref()
                .is_some_and(|url| !addresses.iter().any(|address| address == url));
        if gone {
            collection.remove_library_file(file.id)?;
            report.removed += 1;
        } else {
            kept.push(file);
        }
    }

    // The addresses that have a library file already. `claims` holds the
    // addresses that a local file has on a connected host, as its planned
    // address or as the address of a copy (ADR 0231 item 12).
    let mut known: HashSet<String> = HashSet::new();
    let mut claims: HashMap<String, LibraryFileId> = HashMap::new();
    for file in &kept {
        known.extend(file.source_url.clone());
        if !file.local {
            continue;
        }
        match inventory::planned_path(collection, hosts, &row, file) {
            Ok(planned) => known.extend(planned.url),
            Err(error) if error.kind() == InventoryErrorKind::NoHost => {}
            Err(error) => return Err(read_failed(file, &error)),
        }
        for connector_id in hosts.ids() {
            let planned = inventory::planned_path_on(collection, hosts, &row, file, connector_id)
                .map_err(|error| read_failed(file, &error))?;
            if let Some(url) = planned.url {
                claims.entry(url).or_insert(file.id);
            }
        }
        let copies = inventory::copies(collection, hosts, file)
            .map_err(|error| read_failed(file, &error))?;
        for url in copies.into_iter().filter_map(|copy| copy.url) {
            claims.entry(url).or_insert(file.id);
        }
    }
    known.extend(claims.keys().cloned());

    report.removed += join_remote_rows(collection, feed_id, &mut kept, &claims)?;

    // The paths in the release folder that are in use. The feed XML is in
    // the same folder.
    let mut taken: HashSet<String> = kept.iter().map(|file| file.path.clone()).collect();
    if let Some(path) = row.path.as_deref() {
        taken.insert(
            path.rsplit_once('/')
                .map_or(path, |(_, name)| name)
                .to_owned(),
        );
    }

    for address in addresses.iter().map(AsRef::as_ref) {
        if known.contains(address) {
            continue;
        }
        let facts = facts_of(feed, address);
        let name = last_segment(address);
        let media_type = facts
            .media_type
            .or_else(|| name.as_deref().and_then(media_type_of_name))
            .unwrap_or_else(|| "application/octet-stream".to_owned());
        let base = name.unwrap_or_else(|| default_name(&media_type));
        let path = free_path(&base, &taken);
        collection.add_library_file(
            feed_id,
            &path,
            facts.size.unwrap_or(0),
            None,
            &media_type,
            false,
            Some(address),
            None,
        )?;
        taken.insert(path);
        known.insert(address.to_owned());
        report.added += 1;
    }
    Ok(report)
}

/// Join each remote row of `kept` to the local row that claims its address,
/// and give the count of removed rows. The function takes the joined rows out
/// of `kept`. A remote row that a draft operation names stays (ADR 0231
/// items 12 and 13).
fn join_remote_rows(
    collection: &Collection,
    feed_id: FeedId,
    kept: &mut Vec<LibraryFile>,
    claims: &HashMap<String, LibraryFileId>,
) -> Result<usize, CollectionError> {
    let in_draft: HashSet<LibraryFileId> = collection
        .draft_operations(feed_id)?
        .into_iter()
        .filter_map(|operation| operation.library_file_id)
        .collect();
    let mut joined = Vec::new();
    for file in kept.iter() {
        let is_remote_row = !file.local && file.sha256.is_none() && file.removed_at.is_none();
        let Some(url) = file.source_url.as_deref().filter(|_| is_remote_row) else {
            continue;
        };
        let Some(&local_id) = claims.get(url) else {
            continue;
        };
        if in_draft.contains(&file.id) {
            continue;
        }
        if kept
            .iter()
            .find(|other| other.id == local_id)
            .is_some_and(|local| local.source_url.as_deref() != Some(url))
        {
            collection.set_library_file_source_url(local_id, url)?;
        }
        collection.remove_library_file(file.id)?;
        joined.push(file.id);
    }
    kept.retain(|file| !joined.contains(&file.id));
    Ok(joined.len())
}

/// The error for a failed read of the host of a local library file
/// (ADR 0179 item 1).
fn read_failed(file: &LibraryFile, error: &InventoryError) -> CollectionError {
    CollectionError::with_kind(
        CollectionErrorKind::Database,
        format!(
            "the read of the host of {} failed, so no remote row is added: {error}",
            file.path
        ),
    )
}

/// The size and the media type that the feed gives for an address.
fn facts_of(feed: &Feed, address: &str) -> Facts {
    let mut facts = Facts {
        size: None,
        media_type: None,
    };
    for item in &feed.items {
        if let Some(enclosure) = &item.value.enclosure
            && enclosure.url.as_deref().map(plain_address).as_deref() == Some(address)
        {
            facts.size = facts.size.or_else(|| {
                enclosure
                    .length
                    .as_deref()
                    .and_then(|length| length.trim().parse().ok())
            });
            facts.media_type = facts.media_type.or_else(|| enclosure.media_type.clone());
        }
        for alternate in &item.value.alternate_enclosures {
            for source in &alternate.value.sources {
                if source.value.uri.as_deref().map(plain_address).as_deref() == Some(address) {
                    facts.size = facts.size.or_else(|| {
                        alternate
                            .value
                            .length
                            .as_deref()
                            .and_then(|length| length.trim().parse().ok())
                    });
                    facts.media_type = facts.media_type.or_else(|| {
                        alternate
                            .value
                            .media_type
                            .clone()
                            .or_else(|| source.value.content_type.clone())
                    });
                }
            }
        }
    }
    facts
}

/// The last segment of the path of an address, with the percent encoding
/// decoded. Gives `None` when the segment is empty, is not valid text, or
/// is not a plain file name (ADR 0204 item 2).
fn last_segment(address: &str) -> Option<String> {
    let rest = address.split_once("://").map_or(address, |(_, rest)| rest);
    let path = rest.split(['?', '#']).next().unwrap_or("");
    let path = path.split_once('/').map_or("", |(_, path)| path);
    let segment = path.rsplit('/').next().unwrap_or("");
    let decoded = percent_decode(segment)?;
    is_plain_name(&decoded).then_some(decoded)
}

/// `text` with each `%XX` decoded. Gives `None` when the result is not
/// valid UTF-8.
fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let hex = bytes
            .get(index + 1..index + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (bytes[index], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                index += 3;
            }
            (byte, _) => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// The media type of a file name, from its extension.
#[must_use]
pub fn media_type_of_name(name: &str) -> Option<String> {
    let (_, extension) = name.rsplit_once('.')?;
    let media_type = match extension.to_ascii_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "m4a" | "mp4" => "audio/mp4",
        "ogg" | "opus" => "audio/ogg",
        "wav" => "audio/wav",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        // The types of the `podcast:transcript` specification.
        "srt" => "application/srt",
        "vtt" => "text/vtt",
        "txt" => "text/plain",
        "json" => "application/json",
        "html" => "text/html",
        _ => return None,
    };
    Some(media_type.to_owned())
}

/// The name `file`, with the extension of a media type when Studio knows
/// one (ADR 0204 item 2).
fn default_name(media_type: &str) -> String {
    let extension = match media_type {
        "audio/mpeg" => "mp3",
        "audio/flac" => "flac",
        "audio/mp4" => "m4a",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => return "file".to_owned(),
    };
    format!("file.{extension}")
}

/// `name`, or `name` with ` 2`, ` 3` and so on before the extension, so
/// that the result is not in `taken` (ADR 0204 item 2).
fn free_path(name: &str, taken: &HashSet<String>) -> String {
    if !taken.contains(name) {
        return name.to_owned();
    }
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => (stem, Some(extension)),
        _ => (name, None),
    };
    let mut n = 2;
    loop {
        let candidate = match extension {
            Some(extension) => format!("{stem} {n}.{extension}"),
            None => format!("{stem} {n}"),
        };
        if !taken.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::LocalFolder;
    use crate::inventory::scan;
    use crate::xml::Document;

    fn parse(xml: &str) -> Feed {
        Feed::read(&Document::parse_bytes(xml.as_bytes()).unwrap()).unwrap()
    }

    #[test]
    fn media_type_of_name_gives_the_transcript_types_of_the_lyrics_extensions() {
        for (name, media_type) in [
            ("a.srt", "application/srt"),
            ("a.vtt", "text/vtt"),
            ("a.txt", "text/plain"),
            ("a.json", "application/json"),
            ("a.html", "text/html"),
            ("A.SRT", "application/srt"),
        ] {
            assert_eq!(
                media_type_of_name(name).as_deref(),
                Some(media_type),
                "{name}"
            );
        }
    }

    const FEED: &str = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <title>Album</title>
    <itunes:image href="https://m.example/art/cover.jpg"/>
    <item>
      <title>One</title>
      <guid>one</guid>
      <enclosure url="https://m.example/a/01%20One.mp3" length="1234" type="audio/mpeg"/>
      <podcast:alternateEnclosure type="audio/flac" length="9999">
        <podcast:source uri="https://m.example/a/01%20One.flac"/>
      </podcast:alternateEnclosure>
    </item>
    <item>
      <title>Two</title>
      <guid>two</guid>
      <enclosure url="https://m.example/a/02.mp3" length="55" type="audio/mpeg"/>
    </item>
  </channel>
</rss>"#;

    fn setup() -> (Collection, FeedId) {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed_id = collection.add_feed(None, None).unwrap();
        (collection, feed_id)
    }

    #[test]
    fn each_address_of_a_feed_gets_a_remote_row_and_a_second_call_adds_nothing() {
        let (collection, feed_id) = setup();
        let hosts = Hosts::new();
        let feed = parse(FEED);
        let report = record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        assert_eq!(
            report,
            RemoteReport {
                added: 4,
                removed: 0
            }
        );

        let files = collection.library_files(feed_id).unwrap();
        let row = |path: &str| files.iter().find(|f| f.path == path).unwrap();
        let one = row("01 One.mp3");
        assert_eq!((one.size, one.media_type.as_str()), (1234, "audio/mpeg"));
        assert_eq!(
            one.source_url.as_deref(),
            Some("https://m.example/a/01%20One.mp3")
        );
        assert!(!one.local && one.sha256.is_none() && one.stamp.is_none());
        let flac = row("01 One.flac");
        assert_eq!((flac.size, flac.media_type.as_str()), (9999, "audio/flac"));
        assert_eq!(row("02.mp3").size, 55);
        let image = row("cover.jpg");
        assert_eq!((image.size, image.media_type.as_str()), (0, "image/jpeg"));

        let again = record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        assert_eq!(again, RemoteReport::default());
        assert_eq!(collection.library_files(feed_id).unwrap().len(), 4);
    }

    #[test]
    fn two_addresses_with_one_last_segment_get_numbered_paths() {
        let (collection, feed_id) = setup();
        let feed = parse(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>1</guid><enclosure url="https://m.example/x/a.mp3" length="1" type="audio/mpeg"/></item>
            <item><guid>2</guid><enclosure url="https://m.example/y/a.mp3" length="1" type="audio/mpeg"/></item>
            <item><guid>3</guid><enclosure url="https://m.example/z/" length="1" type="audio/mpeg"/></item>
            </channel></rss>"#,
        );
        record_remote_files(&collection, &Hosts::new(), feed_id, &feed).unwrap();
        let paths: Vec<String> = collection
            .library_files(feed_id)
            .unwrap()
            .into_iter()
            .map(|f| f.path)
            .collect();
        assert_eq!(paths, ["a 2.mp3", "a.mp3", "file.mp3"]);
    }

    #[test]
    fn a_row_whose_address_left_the_feed_goes_out_with_its_download_record() {
        let (collection, feed_id) = setup();
        let hosts = Hosts::new();
        record_remote_files(&collection, &hosts, feed_id, &parse(FEED)).unwrap();
        let local = collection
            .add_library_file(
                feed_id,
                "mine.mp3",
                1,
                Some("abc"),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let gone = collection.library_files(feed_id).unwrap()[0].id;
        collection
            .db()
            .execute(
                "INSERT INTO downloads (library_file_id, state, added_at) VALUES (?1, 'queued', 0)",
                [gone.0],
            )
            .unwrap();
        let smaller = parse(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>2</guid><enclosure url="https://m.example/a/02.mp3" length="55" type="audio/mpeg"/></item>
            </channel></rss>"#,
        );
        let report = record_remote_files(&collection, &hosts, feed_id, &smaller).unwrap();
        assert_eq!(
            report,
            RemoteReport {
                added: 0,
                removed: 3
            }
        );
        let paths: Vec<String> = collection
            .library_files(feed_id)
            .unwrap()
            .into_iter()
            .map(|f| f.path)
            .collect();
        assert_eq!(paths, ["02.mp3", "mine.mp3"]);
        assert!(collection.library_file(local).is_ok());
        let downloads: i64 = collection
            .db()
            .query_row("SELECT COUNT(*) FROM downloads", [], |r| r.get(0))
            .unwrap();
        assert_eq!(downloads, 0);
    }

    #[test]
    fn a_local_file_and_its_copy_get_no_remote_row() {
        let dir = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let home = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let second = collection
            .add_connector("local", "Second", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(
            home,
            Box::new(LocalFolder::new(
                dir.path(),
                Some("https://m.example".into()),
            )),
        );
        hosts.insert(
            second,
            Box::new(LocalFolder::new(
                other.path(),
                Some("https://n.example".into()),
            )),
        );
        // A copy of the second track is on the second host.
        std::fs::write(other.path().join("two-copy.mp3"), b"music").unwrap();
        scan(&collection, &hosts, home).unwrap();
        scan(&collection, &hosts, second).unwrap();

        let feed_id = collection
            .add_feed(None, Some((home, "album.xml")))
            .unwrap();
        collection.set_feed_folder(feed_id, "album").unwrap();
        let sha = crate::digest::sha256_hex(&[b"music"]);
        // The first local file plans the address
        // https://m.example/album/01%20One.mp3 on the home host.
        collection
            .add_library_file(
                feed_id,
                "01 One.mp3",
                5,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        collection
            .add_library_file(
                feed_id,
                "two.mp3",
                5,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let feed = parse(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>1</guid><enclosure url="https://m.example/album/01%20One.mp3" length="5" type="audio/mpeg"/></item>
            <item><guid>2</guid><enclosure url="https://n.example/two-copy.mp3" length="5" type="audio/mpeg"/></item>
            <item><guid>3</guid><enclosure url="https://m.example/other.mp3" length="5" type="audio/mpeg"/></item>
            </channel></rss>"#,
        );
        let report = record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        assert_eq!(report.added, 1);
        let remote: Vec<String> = collection
            .library_files(feed_id)
            .unwrap()
            .into_iter()
            .filter(|f| !f.local)
            .map(|f| f.path)
            .collect();
        assert_eq!(remote, ["other.mp3"]);
    }

    #[test]
    fn a_remote_file_has_the_host_file_at_its_source_url_as_a_copy() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("02.mp3"), b"music").unwrap();
        std::fs::write(dir.path().join("03.mp3"), b"music").unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let home = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(
            home,
            Box::new(LocalFolder::new(
                dir.path(),
                Some("https://m.example/a".into()),
            )),
        );
        scan(&collection, &hosts, home).unwrap();
        let feed_id = collection
            .add_feed(None, Some((home, "album.xml")))
            .unwrap();
        collection.set_feed_folder(feed_id, "album").unwrap();
        let feed = parse(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>2</guid><enclosure url="https://m.example/a/02.mp3" length="5" type="audio/mpeg"/></item>
            </channel></rss>"#,
        );
        record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        let file = collection.library_files(feed_id).unwrap().remove(0);
        let copies = inventory::copies(&collection, &hosts, &file).unwrap();
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].path, "02.mp3");

        // The file is not an upload, and its planned path is the clean path.
        let row = collection.feed(feed_id).unwrap();
        let planned = inventory::planned_path(&collection, &hosts, &row, &file).unwrap();
        assert!(!planned.on_host);
    }

    /// A connector that fails each read while `fail` is true.
    struct FailingRead {
        folder: LocalFolder,
        fail: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl crate::connector::Connector for FailingRead {
        fn read(
            &self,
            path: &str,
        ) -> Result<Option<crate::connector::FileRead>, crate::connector::ConnectorError> {
            if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(crate::connector::ConnectorError::new(
                    crate::connector::ConnectorErrorKind::Io,
                    "the host does not answer",
                ));
            }
            self.folder.read(path)
        }
        fn write(
            &self,
            path: &str,
            bytes: &[u8],
            expected: &crate::connector::Expected,
        ) -> Result<crate::connector::Version, crate::connector::ConnectorError> {
            self.folder.write(path, bytes, expected)
        }
        fn public_url(&self, path: &str) -> Option<String> {
            self.folder.public_url(path)
        }
        fn list(&self) -> Result<crate::connector::FileList, crate::connector::ConnectorError> {
            self.folder.list()
        }
        fn stat(
            &self,
            path: &str,
        ) -> Result<Option<crate::connector::ListedFile>, crate::connector::ConnectorError>
        {
            self.folder.stat(path)
        }
        fn delete(
            &self,
            path: &str,
            expected: &crate::connector::Stamp,
        ) -> Result<(), crate::connector::ConnectorError> {
            self.folder.delete(path, expected)
        }
    }

    #[test]
    fn a_failed_read_of_a_host_adds_no_row_and_the_next_call_adds_it() {
        use std::sync::{Arc, atomic::AtomicBool};
        let dir = tempfile::tempdir().unwrap();
        // The host file has the size of the local file, and no SHA-256 in
        // the inventory. The copies read of the local file reads it.
        std::fs::write(dir.path().join("x.mp3"), b"music").unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let home = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let fail = Arc::new(AtomicBool::new(true));
        let mut hosts = Hosts::new();
        hosts.insert(
            home,
            Box::new(FailingRead {
                folder: LocalFolder::new(dir.path(), Some("https://m.example".into())),
                fail: Arc::clone(&fail),
            }),
        );
        scan(&collection, &hosts, home).unwrap();
        let feed_id = collection
            .add_feed(None, Some((home, "album.xml")))
            .unwrap();
        collection.set_feed_folder(feed_id, "album").unwrap();
        let sha = crate::digest::sha256_hex(&[b"music"]);
        collection
            .add_library_file(
                feed_id,
                "01 One.mp3",
                5,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let feed = parse(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>1</guid><enclosure url="https://m.example/album/01%20One.mp3" length="5" type="audio/mpeg"/></item>
            <item><guid>2</guid><enclosure url="https://m.example/other.mp3" length="5" type="audio/mpeg"/></item>
            </channel></rss>"#,
        );

        assert!(record_remote_files(&collection, &hosts, feed_id, &feed).is_err());
        assert_eq!(collection.library_files(feed_id).unwrap().len(), 1);

        fail.store(false, std::sync::atomic::Ordering::SeqCst);
        let report = record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        assert_eq!(report.added, 1);
        let remote: Vec<String> = collection
            .library_files(feed_id)
            .unwrap()
            .into_iter()
            .filter(|f| !f.local)
            .map(|f| f.path)
            .collect();
        assert_eq!(remote, ["other.mp3"]);
    }

    /// A feed with a local file on a first host, and a second connected
    /// host. The remote row of the feed address is at the planned address
    /// of the local file on the second host (ADR 0231 item 12).
    struct Joined {
        _dirs: Vec<tempfile::TempDir>,
        collection: Collection,
        hosts: Hosts,
        feed_id: FeedId,
        local: LibraryFileId,
        remote: LibraryFileId,
        address: String,
        feed: Feed,
    }

    fn joined() -> Joined {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let home = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let other = collection
            .add_connector(
                "local",
                "Other",
                &LocalFolder::new(second.path(), Some("https://n.example/b".into())).config(),
                None,
            )
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(
            home,
            Box::new(LocalFolder::new(
                first.path(),
                Some("https://m.example".into()),
            )),
        );
        hosts.insert(
            other,
            Box::new(LocalFolder::new(
                second.path(),
                Some("https://n.example/b".into()),
            )),
        );
        let feed_id = collection
            .add_feed(None, Some((home, "album.xml")))
            .unwrap();
        collection.set_feed_folder(feed_id, "album").unwrap();
        let sha = crate::digest::sha256_hex(&[b"music"]);
        let local = collection
            .add_library_file(
                feed_id,
                "one.mp3",
                5,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let row = collection.feed(feed_id).unwrap();
        let file = collection.library_file(local).unwrap();
        let address = inventory::planned_path_on(&collection, &hosts, &row, &file, other)
            .unwrap()
            .url
            .unwrap();
        assert!(address.starts_with("https://n.example/b/"), "{address}");
        let remote = collection
            .add_library_file(
                feed_id,
                "one 2.mp3",
                5,
                None,
                "audio/mpeg",
                false,
                Some(&address),
                None,
            )
            .unwrap();
        let feed = parse(&format!(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>1</guid><enclosure url="{address}" length="5" type="audio/mpeg"/></item>
            </channel></rss>"#
        ));
        Joined {
            _dirs: vec![first, second],
            collection,
            hosts,
            feed_id,
            local,
            remote,
            address,
            feed,
        }
    }

    #[test]
    fn a_remote_row_at_the_planned_address_on_a_second_host_joins_the_local_row() {
        let j = joined();
        let report = record_remote_files(&j.collection, &j.hosts, j.feed_id, &j.feed).unwrap();
        assert_eq!(
            report,
            RemoteReport {
                added: 0,
                removed: 1
            }
        );
        let files = j.collection.library_files(j.feed_id).unwrap();
        assert_eq!(files.len(), 1, "ADR 0231 item 12: one row for one file.");
        assert_eq!(files[0].id, j.local);
        assert_eq!(files[0].source_url.as_deref(), Some(j.address.as_str()));

        let again = record_remote_files(&j.collection, &j.hosts, j.feed_id, &j.feed).unwrap();
        assert_eq!(again, RemoteReport::default());
        assert_eq!(j.collection.library_files(j.feed_id).unwrap().len(), 1);
    }

    #[test]
    fn a_remote_row_that_a_draft_operation_names_stays() {
        let j = joined();
        let connector = j.hosts.ids().next().unwrap();
        j.collection
            .add_draft_operation(
                j.feed_id,
                crate::collection::DraftOperationKind::Delete,
                Some(j.remote),
                connector,
                "one 2.mp3",
            )
            .unwrap();
        let report = record_remote_files(&j.collection, &j.hosts, j.feed_id, &j.feed).unwrap();
        assert_eq!(report, RemoteReport::default());
        assert!(
            j.collection.library_file(j.remote).is_ok(),
            "ADR 0231 item 13: a draft operation keeps the remote row."
        );
        assert_eq!(
            j.collection.library_file(j.local).unwrap().source_url,
            None,
            "the local row does not change while the remote row stays"
        );
    }

    #[test]
    fn a_remote_row_at_the_address_of_a_copy_joins_the_local_row() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("copy.mp3"), b"music").unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let home = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(
            home,
            Box::new(LocalFolder::new(
                dir.path(),
                Some("https://m.example".into()),
            )),
        );
        scan(&collection, &hosts, home).unwrap();
        let feed_id = collection
            .add_feed(None, Some((home, "album.xml")))
            .unwrap();
        collection.set_feed_folder(feed_id, "album").unwrap();
        let sha = crate::digest::sha256_hex(&[b"music"]);
        let local = collection
            .add_library_file(
                feed_id,
                "one.mp3",
                5,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let address = "https://m.example/copy.mp3";
        collection
            .add_library_file(
                feed_id,
                "copy.mp3",
                5,
                None,
                "audio/mpeg",
                false,
                Some(address),
                None,
            )
            .unwrap();
        let feed = parse(&format!(
            r#"<rss version="2.0"><channel><title>A</title>
            <item><guid>1</guid><enclosure url="{address}" length="5" type="audio/mpeg"/></item>
            </channel></rss>"#
        ));
        let report = record_remote_files(&collection, &hosts, feed_id, &feed).unwrap();
        assert_eq!(report.removed, 1);
        let files = collection.library_files(feed_id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].id, local);
        assert_eq!(files[0].source_url.as_deref(), Some(address));
    }
}
