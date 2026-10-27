//! The records of remote files: the library files whose bytes are not in
//! the library (ADR 0038 items 1 to 3).
//!
//! A published feed has only its feed XML in its release folder. Each
//! audio file and image of the feed is on a host. This module gives each
//! address of the feed a library file with `local` false and no SHA-256.
//! It changes no file on disk.

use std::collections::HashSet;

use crate::collection::{Collection, CollectionError, FeedId};
use crate::feed::Feed;
use crate::inventory::{self, Hosts};
use crate::library::is_plain_name;
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
/// (ADR 0038 items 1 to 3).
///
/// The function skips an address that is the `source_url` of a row of the
/// feed. It also skips an address that is the planned address of a local
/// library file, or the address of a copy of that file. It adds a row
/// with `local` false for each other address. It removes each row with
/// `local` false whose address is no longer in `feed`, with its download
/// record.
///
/// A host that Studio cannot read gives no planned address and no copy.
/// The function then treats the addresses of that host as other addresses.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
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
                .is_some_and(|url| !addresses.contains(&url));
        if gone {
            collection.remove_library_file(file.id)?;
            report.removed += 1;
        } else {
            kept.push(file);
        }
    }

    // The addresses that have a library file already.
    let mut known: HashSet<String> = HashSet::new();
    for file in &kept {
        known.extend(file.source_url.clone());
        if !file.local {
            continue;
        }
        if let Ok(planned) = inventory::planned_path(collection, hosts, &row, file) {
            known.extend(planned.url);
        }
        if let Ok(copies) = inventory::copies(collection, hosts, file) {
            known.extend(copies.into_iter().filter_map(|copy| copy.url));
        }
    }

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

    for address in addresses {
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

/// The size and the media type that the feed gives for an address.
fn facts_of(feed: &Feed, address: &str) -> Facts {
    let mut facts = Facts {
        size: None,
        media_type: None,
    };
    for item in &feed.items {
        if let Some(enclosure) = &item.value.enclosure
            && enclosure.url.as_deref() == Some(address)
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
                if source.value.uri.as_deref() == Some(address) {
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
/// is not a plain file name (ADR 0038 item 2).
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
        _ => return None,
    };
    Some(media_type.to_owned())
}

/// The name `file`, with the extension of a media type when Studio knows
/// one (ADR 0038 item 2).
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
/// that the result is not in `taken` (ADR 0038 item 2).
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
}
