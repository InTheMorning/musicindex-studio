//! The address fields of a feed and the address rule (ADR 0023).

use serde::Serialize;

use super::Feed;

/// An address field that is not a correct web address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadAddress {
    /// The path of the field in the JSON model, such as
    /// `items/2/enclosure/url`.
    pub field: String,
    /// The place of the field in words for the user, such as
    /// `Credit · The Doerfels · Information address` (ADR 0245 item 3).
    pub place: String,
    /// The value of the field.
    pub value: String,
}

/// True when `text` is a correct web address: it starts with `http://` or
/// `https://`, has a host name, and contains no space (ADR 0023 item 2).
#[must_use]
pub fn is_web_address(text: &str) -> bool {
    if text.chars().any(char::is_whitespace) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    !host.is_empty()
}

/// Walk each address field of a feed with its model path and its place
/// (ADR 0023 item 1, ADR 0245 item 3). The macro gives one walk for a read
/// and for a change, so [`Feed::bad_addresses`] and
/// [`Feed::set_address`] cannot disagree (ADR 0245 item 2).
///
/// `$visit` is a closure that gets the path, the place and the field. `$iter`
/// is `iter` or `iter_mut`. `$m` is empty or `mut`.
macro_rules! walk_feed {
    ($visit:ident, $feed:expr, $iter:ident, [$($m:tt)*]) => {{
        let feed = $feed;
        $visit("link", "Website", &$($m)* feed.link);
        $visit("image", "Cover image address", &$($m)* feed.image);
        if let Some(image) = &$($m)* feed.rss_image {
            $visit("rss_image/url", "Cover image address", &$($m)* image.url);
        }
        for (i, image) in feed.images.$iter().enumerate() {
            $visit(
                &format!("images/{i}/href"),
                &join(&["Artwork", "Image address"]),
                &$($m)* image.value.href,
            );
        }
        for (i, person) in feed.persons.$iter().enumerate() {
            let name = credit_part(&person.value.name);
            $visit(
                &format!("persons/{i}/img"),
                &join(&[&name, "Picture address"]),
                &$($m)* person.value.img,
            );
            $visit(
                &format!("persons/{i}/href"),
                &join(&[&name, "Information address"]),
                &$($m)* person.value.href,
            );
        }
        for (i, funding) in feed.fundings.$iter().enumerate() {
            $visit(
                &format!("fundings/{i}/url"),
                &join(&["Support link", "Address"]),
                &$($m)* funding.value.url,
            );
        }
        if let Some(license) = &$($m)* feed.license {
            $visit("license/url", "License text address", &$($m)* license.url);
        }
        for (i, entry) in feed.items.$iter().enumerate() {
            let path = format!("items/{i}/");
            let head = format!("Track {} \u{b7} {}", i + 1, title_part(entry.value.title.as_deref()));
            walk_item!($visit, &$($m)* entry.value, $iter, [$($m)*], &path, &head);
        }
        // A live item has the same address fields as an item, and its
        // content links (ADR 0023 item 1, ADR 0255 item 38).
        for (i, live) in feed.live_items.$iter().enumerate() {
            let path = format!("live_items/{i}/");
            let head = format!(
                "Live item \u{b7} {}",
                title_part(live.value.item.title.as_deref())
            );
            walk_item!(
                $visit,
                &$($m)* live.value.item,
                $iter,
                [$($m)*],
                &format!("{path}item/"),
                &head
            );
            for (j, link) in live.value.content_links.$iter().enumerate() {
                $visit(
                    &format!("{path}content_links/{j}/href"),
                    &join(&[&head, "Link address"]),
                    &$($m)* link.value.href,
                );
            }
        }
    }};
}

/// Walk the address fields of one item (ADR 0245 item 3).
macro_rules! walk_item {
    ($visit:ident, $item:expr, $iter:ident, [$($m:tt)*], $path:expr, $head:expr) => {{
        let item = $item;
        let path: &str = $path;
        let head: &str = $head;
        if let Some(enclosure) = &$($m)* item.enclosure {
            let label = if head.starts_with("Live item") {
                "Stream address"
            } else {
                "Audio file address"
            };
            $visit(
                &format!("{path}enclosure/url"),
                &join(&[head, label]),
                &$($m)* enclosure.url,
            );
        }
        let label = if head.starts_with("Live item") {
            "Image address"
        } else {
            "Track image address"
        };
        $visit(&format!("{path}image"), &join(&[head, label]), &$($m)* item.image);
        for (i, image) in item.images.$iter().enumerate() {
            $visit(
                &format!("{path}images/{i}/href"),
                &join(&[head, "Artwork", "Image address"]),
                &$($m)* image.value.href,
            );
        }
        for (i, person) in item.persons.$iter().enumerate() {
            let name = credit_part(&person.value.name);
            $visit(
                &format!("{path}persons/{i}/img"),
                &join(&[head, &name, "Picture address"]),
                &$($m)* person.value.img,
            );
            $visit(
                &format!("{path}persons/{i}/href"),
                &join(&[head, &name, "Information address"]),
                &$($m)* person.value.href,
            );
        }
        for (i, funding) in item.fundings.$iter().enumerate() {
            $visit(
                &format!("{path}fundings/{i}/url"),
                &join(&[head, "Support link", "Address"]),
                &$($m)* funding.value.url,
            );
        }
        for (i, transcript) in item.transcripts.$iter().enumerate() {
            $visit(
                &format!("{path}transcripts/{i}/url"),
                &join(&[head, "Lyrics file address"]),
                &$($m)* transcript.value.url,
            );
        }
        if let Some(license) = &$($m)* item.license {
            $visit(
                &format!("{path}license/url"),
                &join(&[head, "License text address"]),
                &$($m)* license.url,
            );
        }
    }};
}

/// Join the parts of a place with " \u{b7} " (ADR 0245 item 3).
fn join(parts: &[&str]) -> String {
    parts.join(" \u{b7} ")
}

/// The part of a place for a credit (ADR 0245 item 3).
fn credit_part(name: &str) -> String {
    match Some(name.trim()).filter(|n| !n.is_empty()) {
        Some(name) => format!("Credit \u{b7} {name}"),
        None => "Credit".to_owned(),
    }
}

/// The title part of a place. The editor shows "Untitled track" for an item
/// with no title (ADR 0245 item 3).
fn title_part(title: Option<&str>) -> &str {
    title
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("Untitled track")
}

/// The text with `https://` in front, when that gives a web address
/// (ADR 0245 item 1). The text must have no `://`, and its host must have a
/// dot, so a word such as `none` gives none. A text with a space, an empty
/// host or no text gives none.
#[must_use]
pub fn short_address_repair(text: &str) -> Option<String> {
    let text = text.trim();
    if text.contains("://") {
        return None;
    }
    let full = format!("https://{text}");
    let host = text.split(['/', '?', '#']).next().unwrap_or_default();
    (host.contains('.') && is_web_address(&full)).then_some(full)
}

/// The error of [`Feed::set_address`]: the model path names no address field
/// of the feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAddressField(pub String);

impl std::fmt::Display for UnknownAddressField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the feed has no address field at {}", self.0)
    }
}

impl std::error::Error for UnknownAddressField {}

impl Feed {
    /// Each address field of the feed that is not a correct web address
    /// (ADR 0023 items 1 and 2). The core refuses a write when the list is
    /// not empty.
    #[must_use]
    pub fn bad_addresses(&self) -> Vec<BadAddress> {
        let mut found = Vec::new();
        let mut visit = |path: &str, place: &str, slot: &Option<String>| {
            // An empty field is no address (ADR 0023 item 2). A placeholder
            // address is replaced at the public address (ADR 0122 item 15).
            if let Some(value) = slot.as_ref().filter(|v| !v.is_empty())
                && !is_web_address(value)
                && !crate::inventory::is_placeholder_address(value)
            {
                found.push(BadAddress {
                    field: path.to_owned(),
                    place: place.to_owned(),
                    value: value.clone(),
                });
            }
        };
        walk_feed!(visit, self, iter, []);
        found
    }

    /// Set the text of the address field at the model path `field`
    /// (ADR 0245 item 2). The walk is the walk of [`Feed::bad_addresses`].
    ///
    /// # Errors
    ///
    /// Gives [`UnknownAddressField`] when `field` names no address field.
    pub fn set_address(&mut self, field: &str, text: &str) -> Result<(), UnknownAddressField> {
        let mut done = false;
        let mut visit = |path: &str, _place: &str, slot: &mut Option<String>| {
            if path == field && !done {
                *slot = Some(text.to_owned());
                done = true;
            }
        };
        walk_feed!(visit, self, iter_mut, [mut]);
        if done {
            Ok(())
        } else {
            Err(UnknownAddressField(field.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Entry, Item, License};

    #[test]
    fn a_web_address_has_a_scheme_and_a_host_and_no_space() {
        for good in [
            "https://music.example/a.mp3",
            "http://music.example",
            "HTTPS://Music.Example/x?y=1",
        ] {
            assert!(is_web_address(good), "{good}");
        }
        for bad in [
            "none",
            "www.example.com",
            "https://",
            "https:///a.mp3",
            "ftp://music.example/a.mp3",
            "https://music.example/a b.mp3",
            " https://music.example",
        ] {
            assert!(!is_web_address(bad), "{bad}");
        }
    }

    #[test]
    fn each_bad_address_field_has_its_model_path() {
        let feed = Feed {
            link: Some("www.example.com".to_owned()),
            image: Some(String::new()),
            license: Some(License {
                identifier: "cc-by-4.0".to_owned(),
                url: Some("none".to_owned()),
            }),
            items: vec![
                Entry::new(Item::default()),
                Entry::new(Item {
                    enclosure: Some(super::super::Enclosure {
                        url: Some("music.example/a.mp3".to_owned()),
                        ..Default::default()
                    }),
                    image: Some("https://music.example/a.jpg".to_owned()),
                    ..Item::default()
                }),
            ],
            ..Feed::default()
        };
        let fields: Vec<_> = feed.bad_addresses().into_iter().map(|b| b.field).collect();
        assert_eq!(fields, ["link", "license/url", "items/1/enclosure/url"]);
    }

    #[test]
    fn a_live_item_address_with_a_space_is_a_bad_address() {
        let mut live = crate::live::new_live_item("Show", "2026-11-01T10:00:00Z");
        live.item.enclosure = Some(super::super::Enclosure {
            url: Some("   https://music.example/live.m3u8".to_owned()),
            ..Default::default()
        });
        live.content_links
            .push(Entry::new(super::super::ContentLink {
                href: Some("https://music.example/show".to_owned()),
                ..Default::default()
            }));
        let mut feed = Feed::default();
        feed.live_items.push(Entry::new(live));
        let fields: Vec<String> = feed.bad_addresses().into_iter().map(|b| b.field).collect();
        assert_eq!(fields, ["live_items/0/item/enclosure/url"]);
    }

    #[test]
    fn a_short_address_gets_https_only_when_that_gives_a_web_address() {
        let owner = "ADR 0245 item 1: fix short_address_repair in links.rs";
        assert_eq!(
            short_address_repair("www.doerfelverse.com").as_deref(),
            Some("https://www.doerfelverse.com"),
            "{owner}"
        );
        assert_eq!(
            short_address_repair(" doerfelverse.com/artists ").as_deref(),
            Some("https://doerfelverse.com/artists"),
            "{owner}"
        );
        for text in ["none", "https://", "www.a b.com", "", "ftp://a.example"] {
            assert_eq!(short_address_repair(text), None, "{owner}: {text:?}");
        }
    }

    fn credit(href: &str) -> Entry<crate::feed::Person> {
        Entry::new(crate::feed::Person {
            name: "The Doerfels".to_owned(),
            href: Some(href.to_owned()),
            ..Default::default()
        })
    }

    #[test]
    fn the_place_of_an_address_gives_the_words_of_the_editor() {
        let owner = "ADR 0245 item 3: fix the walk in links.rs";
        let mut feed = Feed {
            link: Some("none".to_owned()),
            persons: vec![credit("www.doerfelverse.com")],
            items: vec![Entry::new(Item {
                title: Some("Neon Diner".to_owned()),
                persons: vec![credit("www.doerfelverse.com")],
                enclosure: Some(super::super::Enclosure {
                    url: Some("music.example/a.mp3".to_owned()),
                    ..Default::default()
                }),
                ..Item::default()
            })],
            ..Feed::default()
        };
        feed.items.push(Entry::new(Item::default()));
        let places: Vec<String> = feed.bad_addresses().into_iter().map(|b| b.place).collect();
        assert_eq!(
            places,
            [
                "Website",
                "Credit \u{b7} The Doerfels \u{b7} Information address",
                "Track 1 \u{b7} Neon Diner \u{b7} Audio file address",
                "Track 1 \u{b7} Neon Diner \u{b7} Credit \u{b7} The Doerfels \u{b7} Information address",
            ],
            "{owner}"
        );
    }

    #[test]
    fn the_setter_changes_only_its_one_field() {
        let owner = "ADR 0245 item 2: fix Feed::set_address in links.rs";
        let mut feed = Feed {
            persons: vec![credit("www.a.com"), credit("www.b.com")],
            ..Feed::default()
        };
        feed.set_address("persons/1/href", "https://www.b.com")
            .unwrap();
        let hrefs: Vec<_> = feed.persons.iter().map(|p| p.value.href.clone()).collect();
        assert_eq!(
            hrefs,
            [
                Some("www.a.com".to_owned()),
                Some("https://www.b.com".to_owned())
            ],
            "{owner}"
        );
        assert!(feed.set_address("persons/2/href", "x").is_err(), "{owner}");
        assert!(feed.set_address("title", "x").is_err(), "{owner}");
    }
}
