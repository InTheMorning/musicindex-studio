//! The check of a feed (ADR 0174 items 1 to 12).
//!
//! The check examines a feed and gives a list of findings. It sends no
//! request and changes nothing. The caller gathers the facts of the reads
//! and gives them as data in [`Facts`].

use std::collections::{HashMap, HashSet};

use crate::feed::Feed;

/// The kind of a finding. Each kind is one row of the table in ADR 0174
/// item 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// The feed has a public address and no self link.
    NoSelfLink,
    /// The self link differs from the public address.
    SelfLinkDiffers,
    /// The self link or the tag differs from the public address only in its
    /// spelling (item 12).
    SelfLinkSpelling,
    /// An enclosure or an image has an `http` address, and the `https`
    /// address gives the same file.
    HttpFile,
    /// An enclosure has no `length`, or `length` is 0.
    NoFileSize,
    /// The host gives a `Content-Length` that differs from `length`.
    WrongFileSize,
    /// A track has no `itunes:duration`.
    NoDuration,
    /// The tag names a URL, and the body at that URL names a third URL.
    MoveNotConfirmed,
    /// The feed has no `language`.
    NoLanguage,
}

/// How a finding asks for action (ADR 0174 item 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    /// The finding has a repair or a button that opens the field.
    Notice,
    /// The finding needs a value from the user.
    Question,
}

/// The repair that a finding offers (ADR 0174 item 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairKind {
    /// **Add the Link**.
    AddTheLink,
    /// **Use the Public Address**.
    UseThePublicAddress,
    /// **Use the https Address**.
    UseTheHttpsAddress,
    /// **Read the Size**.
    ReadTheSize,
    /// **Open the Track**.
    OpenTheTrack,
    /// **Select a Language**.
    SelectALanguage,
}

/// One finding of the check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Finding {
    /// The kind of the finding.
    pub kind: FindingKind,
    /// The level of the finding.
    pub level: FindingLevel,
    /// The title for the user.
    pub title: String,
    /// The reason for the user.
    pub reason: String,
    /// The `guid` of the item that the finding is about. It is none for a
    /// finding about the feed.
    pub entry: Option<String>,
    /// The repair of the finding, when one exists.
    pub repair: Option<RepairKind>,
    /// The address that the finding names, when it names one. A repair
    /// changes only this address.
    pub url: Option<String>,
}

impl Finding {
    fn naming(mut self, url: &str) -> Self {
        self.url = Some(url.to_owned());
        self
    }
}

/// The facts that the caller gathers before the check (ADR 0174 item 4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// The public address of the feed.
    pub public_address: Option<String>,
    /// The links that [`crate::move_link::check`] refuses. The check gives
    /// no finding for them (ADR 0174 item 5).
    pub refused_links: Vec<String>,
    /// True when the draft changes the self link of the base. ADR 0126
    /// item 6 then gives its notice in Review, and the check gives no
    /// `self_link_differs` finding (ADR 0174 item 7).
    pub self_changed: bool,
    /// The `Content-Length` of each enclosure URL, from a read.
    pub content_lengths: HashMap<String, u64>,
    /// The `http` URLs whose `https` address gives the same file
    /// (ADR 0174 item 14).
    pub same_https: HashSet<String>,
    /// The chain that the read of the tag followed. The first URL is the
    /// tag. The last URL is the end of the chain. It is none when no read
    /// ran.
    pub move_chain: Option<Vec<String>>,
}

/// Examine `feed` and give its findings. The function sends no request and
/// changes nothing.
#[must_use]
pub fn check(feed: &Feed, facts: &Facts) -> Vec<Finding> {
    let mut found = Vec::new();
    check_links(feed, facts, &mut found);
    check_move(feed, facts, &mut found);
    for entry in &feed.items {
        check_item(&entry.value, facts, &mut found);
    }
    check_images(feed, facts, &mut found);
    if feed.language.as_deref().is_none_or(|l| l.trim().is_empty()) {
        found.push(Finding {
            kind: FindingKind::NoLanguage,
            level: FindingLevel::Question,
            title: "No language".to_owned(),
            reason: "Select the language of this feed.".to_owned(),
            entry: None,
            repair: Some(RepairKind::SelectALanguage),
            url: None,
        });
    }
    found
}

fn notice(
    kind: FindingKind,
    title: String,
    reason: String,
    entry: Option<String>,
    repair: Option<RepairKind>,
) -> Finding {
    Finding {
        kind,
        level: FindingLevel::Notice,
        title,
        reason,
        entry,
        repair,
        url: None,
    }
}

fn is_refused(facts: &Facts, link: &str) -> bool {
    facts.refused_links.iter().any(|r| r == link)
}

/// The findings about the self link and the tag (items 7 and 12).
fn check_links(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let Some(public) = facts.public_address.as_deref() else {
        return;
    };
    let Some(link) = feed.self_url() else {
        found.push(notice(
            FindingKind::NoSelfLink,
            "No self link".to_owned(),
            "The feed does not state its address.".to_owned(),
            None,
            Some(RepairKind::AddTheLink),
        ));
        return;
    };
    let link_spelling = link != public && same_but_spelling(link, public);
    let tag = feed.new_feed_url.as_deref();
    let tag_spelling = tag.is_some_and(|t| t != public && same_but_spelling(t, public));
    if link != public && !link_spelling && !facts.self_changed && !is_refused(facts, link) {
        found.push(notice(
            FindingKind::SelfLinkDiffers,
            "The self link differs".to_owned(),
            format!("The self link names {link}. The public address is {public}."),
            None,
            Some(RepairKind::UseThePublicAddress),
        ));
    }
    let spelling_link = link_spelling && !is_refused(facts, link);
    let spelling_tag = tag_spelling && tag.is_some_and(|t| !is_refused(facts, t));
    if spelling_link || spelling_tag {
        let reason = match (spelling_link, spelling_tag) {
            (true, true) => {
                "The self link and the tag differ from the public address only in spelling."
            }
            (true, false) => "The self link differs from the public address only in spelling.",
            _ => "The tag differs from the public address only in spelling.",
        };
        found.push(notice(
            FindingKind::SelfLinkSpelling,
            "The address has a different spelling".to_owned(),
            reason.to_owned(),
            None,
            Some(RepairKind::UseThePublicAddress),
        ));
    }
}

/// The finding about the chain of the tag (item 13).
fn check_move(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let (Some(tag), Some(chain)) = (feed.new_feed_url.as_deref(), facts.move_chain.as_ref()) else {
        return;
    };
    let Some(end) = chain.last() else {
        return;
    };
    if end == tag || is_refused(facts, tag) {
        return;
    }
    found.push(notice(
        FindingKind::MoveNotConfirmed,
        "The move is not confirmed".to_owned(),
        format!("The tag names {tag}. The chain of addresses ends at {end}."),
        None,
        None,
    ));
}

/// The findings about one item (items 7 and 14).
fn check_item(item: &crate::feed::Item, facts: &Facts, found: &mut Vec<Finding>) {
    let Some(enclosure) = &item.enclosure else {
        return;
    };
    let track = item
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or("A track");
    let entry = item.guid.clone();
    let length = enclosure.length.as_deref().map(str::trim);
    let size = length.and_then(|l| l.parse::<u64>().ok()).unwrap_or(0);
    if length.is_none_or(|l| l.is_empty() || l == "0") {
        let reason = if length == Some("0") {
            "The enclosure gives the size 0."
        } else {
            "The enclosure gives no size."
        };
        found.push(notice(
            FindingKind::NoFileSize,
            format!("{track} has no file size"),
            reason.to_owned(),
            entry.clone(),
            Some(RepairKind::ReadTheSize),
        ));
    } else if let Some(url) = enclosure.url.as_deref()
        && let Some(host_size) = facts.content_lengths.get(url)
        && *host_size != size
    {
        found.push(notice(
            FindingKind::WrongFileSize,
            format!("{track} has a wrong file size"),
            format!("The enclosure gives the size {size}. The host gives the size {host_size}."),
            entry.clone(),
            Some(RepairKind::ReadTheSize),
        ));
    }
    if let Some(url) = enclosure.url.as_deref()
        && facts.same_https.contains(url)
        && url.starts_with("http://")
    {
        found.push(
            notice(
                FindingKind::HttpFile,
                format!("{track} has an http file"),
                "The https address gives the same file.".to_owned(),
                entry.clone(),
                Some(RepairKind::UseTheHttpsAddress),
            )
            .naming(url),
        );
    }
    if item.duration.as_deref().is_none_or(|d| d.trim().is_empty()) {
        found.push(notice(
            FindingKind::NoDuration,
            format!("{track} has no duration"),
            "Apps cannot show the length of this track.".to_owned(),
            entry,
            Some(RepairKind::OpenTheTrack),
        ));
    }
}

/// The finding about the image of the feed (item 14). The `itunes:image`
/// and the RSS image can name the same URL. That URL gives one finding.
fn check_images(feed: &Feed, facts: &Facts, found: &mut Vec<Finding>) {
    let rss = feed.rss_image.as_ref().and_then(|i| i.url.as_deref());
    let mut urls = vec![feed.image.as_deref(), rss];
    urls.dedup();
    for url in urls.into_iter().flatten() {
        if url.starts_with("http://") && facts.same_https.contains(url) {
            found.push(
                notice(
                    FindingKind::HttpFile,
                    "The image has an http address".to_owned(),
                    "The https address gives the same file.".to_owned(),
                    None,
                    Some(RepairKind::UseTheHttpsAddress),
                )
                .naming(url),
            );
        }
    }
}

/// True when `a` and `b` differ only in spelling (ADR 0174 item 12). The
/// two URLs are equal after these changes: lowercase the scheme and the
/// host, remove the default port, and give an empty path the path `/`.
/// Equal text is not a spelling difference.
#[must_use]
pub fn same_but_spelling(a: &str, b: &str) -> bool {
    if a == b {
        return false;
    }
    matches!((normalize(a), normalize(b)), (Some(x), Some(y)) if x == y)
}

/// The spelling form of an absolute `http` or `https` URL, or none.
fn normalize(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(end);
    let (userinfo, hostport) = authority
        .rsplit_once('@')
        .map_or(("", authority), |(u, h)| (u, h));
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) if !p.contains(']') && p.bytes().all(|c| c.is_ascii_digit()) => (h, p),
        _ => (hostport, ""),
    };
    if host.is_empty() {
        return None;
    }
    let default = if scheme == "http" { "80" } else { "443" };
    let mut out = format!("{scheme}://");
    if !userinfo.is_empty() {
        out.push_str(userinfo);
        out.push('@');
    }
    out.push_str(&host.to_ascii_lowercase());
    if !port.is_empty() && port != default {
        out.push(':');
        out.push_str(port);
    }
    if !tail.starts_with('/') {
        out.push('/');
    }
    out.push_str(tail);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{AtomLink, Enclosure, Entry, Item, Recipient, RssImage, ValueBlock};

    const PUBLIC: &str = "https://music.example/feed.xml";

    fn facts() -> Facts {
        Facts {
            public_address: Some(PUBLIC.to_owned()),
            ..Facts::default()
        }
    }

    /// A feed with no condition of the table.
    fn clean() -> Feed {
        let mut feed = Feed {
            language: Some("en".to_owned()),
            ..Feed::default()
        };
        feed.atom_links.push(Entry::new(AtomLink {
            href: Some(PUBLIC.to_owned()),
            rel: Some("self".to_owned()),
            ..AtomLink::default()
        }));
        feed
    }

    fn track(title: &str, length: Option<&str>, duration: Option<&str>) -> Entry<Item> {
        Entry::new(Item {
            title: Some(title.to_owned()),
            guid: Some(format!("guid-{title}")),
            enclosure: Some(Enclosure {
                url: Some(format!("http://files.example/{title}.mp3")),
                length: length.map(str::to_owned),
                media_type: Some("audio/mpeg".to_owned()),
            }),
            duration: duration.map(str::to_owned),
            ..Item::default()
        })
    }

    fn kinds(found: &[Finding]) -> Vec<FindingKind> {
        found.iter().map(|f| f.kind).collect()
    }

    #[test]
    fn a_feed_with_no_condition_gives_no_finding() {
        let mut feed = clean();
        feed.items.push(track("One", Some("100"), Some("3:00")));
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn no_self_link_needs_a_public_address() {
        let mut feed = clean();
        feed.atom_links.clear();
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::NoSelfLink]);
        assert_eq!(found[0].level, FindingLevel::Notice);
        assert_eq!(found[0].repair, Some(RepairKind::AddTheLink));
        assert_eq!(found[0].entry, None);
        assert_eq!(kinds(&check(&feed, &Facts::default())), []);
    }

    #[test]
    fn a_different_self_link_gives_a_finding() {
        let mut feed = clean();
        feed.set_self_url("https://other.example/feed.xml");
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::SelfLinkDiffers]);
        assert_eq!(found[0].repair, Some(RepairKind::UseThePublicAddress));
        // A different path is not a spelling difference.
        feed.set_self_url("https://music.example/other.xml");
        assert_eq!(
            kinds(&check(&feed, &facts())),
            [FindingKind::SelfLinkDiffers]
        );
        // A draft that changes the self link gives no finding. Review gives
        // the notice of ADR 0126 item 6.
        let changed = Facts {
            self_changed: true,
            ..facts()
        };
        assert_eq!(kinds(&check(&feed, &changed)), []);
    }

    #[test]
    fn a_spelling_difference_of_the_self_link_gives_its_own_finding() {
        let mut feed = clean();
        feed.set_self_url("HTTPS://Music.Example:443/feed.xml");
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::SelfLinkSpelling]);
        assert_eq!(found[0].repair, Some(RepairKind::UseThePublicAddress));
    }

    #[test]
    fn a_spelling_difference_of_the_tag_gives_a_finding() {
        let mut feed = clean();
        feed.new_feed_url = Some("https://MUSIC.example/feed.xml".to_owned());
        assert_eq!(
            kinds(&check(&feed, &facts())),
            [FindingKind::SelfLinkSpelling]
        );
        // A tag that names the public address exactly is correct.
        feed.new_feed_url = Some(PUBLIC.to_owned());
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn the_spelling_rule_has_three_changes() {
        let p = "https://music.example/feed.xml";
        assert!(same_but_spelling("HTTPS://music.example/feed.xml", p));
        assert!(same_but_spelling("https://MUSIC.Example/feed.xml", p));
        assert!(same_but_spelling("https://music.example:443/feed.xml", p));
        assert!(same_but_spelling(
            "http://a.example:80/x",
            "http://a.example/x"
        ));
        assert!(same_but_spelling(
            "https://music.example",
            "https://music.example/"
        ));
        // Equal text, a different path, a different port and a different scheme.
        assert!(!same_but_spelling(p, p));
        assert!(!same_but_spelling("https://music.example/other.xml", p));
        assert!(!same_but_spelling("https://music.example/Feed.xml", p));
        assert!(!same_but_spelling("https://music.example:8443/feed.xml", p));
        assert!(!same_but_spelling("http://music.example/feed.xml", p));
        assert!(!same_but_spelling("not a url", p));
    }

    #[test]
    fn a_refused_link_gives_no_finding() {
        let other = "https://other.example/feed.xml";
        let mut feed = clean();
        feed.set_self_url(other);
        let mut facts = facts();
        facts.refused_links.push(other.to_owned());
        assert_eq!(kinds(&check(&feed, &facts)), []);

        let spelled = "https://MUSIC.example/feed.xml";
        feed.set_self_url(PUBLIC);
        feed.new_feed_url = Some(spelled.to_owned());
        facts.refused_links.push(spelled.to_owned());
        assert_eq!(kinds(&check(&feed, &facts)), []);
    }

    #[test]
    fn a_missing_or_zero_size_gives_a_finding() {
        for (length, reason) in [
            (None, "The enclosure gives no size."),
            (Some("0"), "The enclosure gives the size 0."),
        ] {
            let mut feed = clean();
            feed.items.push(track("One", length, Some("3:00")));
            let found = check(&feed, &facts());
            assert_eq!(kinds(&found), [FindingKind::NoFileSize]);
            assert_eq!(found[0].title, "One has no file size");
            assert_eq!(found[0].reason, reason);
            assert_eq!(found[0].entry.as_deref(), Some("guid-One"));
            assert_eq!(found[0].repair, Some(RepairKind::ReadTheSize));
        }
    }

    #[test]
    fn a_track_with_no_duration_gives_a_finding() {
        let mut feed = clean();
        feed.items.push(track("One", Some("5"), None));
        feed.items.push(track("Two", Some("5"), Some("  ")));
        let found = check(&feed, &facts());
        assert_eq!(kinds(&found), [FindingKind::NoDuration; 2]);
        assert_eq!(
            found[0].reason,
            "Apps cannot show the length of this track."
        );
        assert_eq!(found[0].repair, Some(RepairKind::OpenTheTrack));
        // An item with no enclosure is not a track.
        let mut feed = clean();
        feed.items.push(Entry::new(Item::default()));
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn a_feed_with_no_language_gives_a_question() {
        for language in [None, Some(" ".to_owned())] {
            let mut feed = clean();
            feed.language = language;
            let found = check(&feed, &facts());
            assert_eq!(kinds(&found), [FindingKind::NoLanguage]);
            assert_eq!(found[0].level, FindingLevel::Question);
            assert_eq!(found[0].repair, Some(RepairKind::SelectALanguage));
        }
    }

    #[test]
    fn a_media_type_no_image_and_no_lock_give_no_finding() {
        let mut feed = clean();
        let mut item = track("One", Some("100"), Some("3:00"));
        item.value.enclosure.as_mut().unwrap().media_type = Some("audio/flac".to_owned());
        item.value.image = None;
        feed.items.push(item);
        feed.locked = None;
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn a_split_sum_other_than_100_gives_no_finding() {
        let mut feed = clean();
        let recipient = |split: &str| {
            Entry::new(Recipient {
                name: Some("One".to_owned()),
                split: Some(split.to_owned()),
                ..Recipient::default()
            })
        };
        feed.values.push(Entry::new(ValueBlock {
            recipients: vec![recipient("7"), recipient("3"), recipient("1")],
            ..ValueBlock::default()
        }));
        assert_eq!(kinds(&check(&feed, &facts())), []);
    }

    #[test]
    fn an_http_image_gives_one_finding_for_one_address() {
        let url = "http://files.example/cover.jpg";
        let mut feed = clean();
        feed.image = Some(url.to_owned());
        feed.rss_image = Some(RssImage {
            url: Some(url.to_owned()),
            ..RssImage::default()
        });
        let mut facts = facts();
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.same_https.insert(url.to_owned());
        let found = check(&feed, &facts);
        assert_eq!(kinds(&found), [FindingKind::HttpFile]);
        assert_eq!(found[0].repair, Some(RepairKind::UseTheHttpsAddress));
    }

    #[test]
    fn the_network_facts_give_their_findings() {
        let mut feed = clean();
        feed.new_feed_url = Some("https://new.example/feed.xml".to_owned());
        feed.items.push(track("One", Some("100"), Some("3:00")));
        let url = "http://files.example/One.mp3".to_owned();
        let mut facts = facts();
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.content_lengths.insert(url.clone(), 100);
        assert_eq!(kinds(&check(&feed, &facts)), []);
        facts.content_lengths.insert(url.clone(), 120);
        facts.same_https.insert(url);
        facts.move_chain = Some(vec![
            "https://new.example/feed.xml".to_owned(),
            "https://third.example/feed.xml".to_owned(),
        ]);
        let found = check(&feed, &facts);
        assert_eq!(
            kinds(&found),
            [
                FindingKind::MoveNotConfirmed,
                FindingKind::WrongFileSize,
                FindingKind::HttpFile
            ]
        );
        facts.move_chain = Some(vec!["https://new.example/feed.xml".to_owned()]);
        assert!(!kinds(&check(&feed, &facts)).contains(&FindingKind::MoveNotConfirmed));
    }
}
