//! The address fields of a feed and the address rule (ADR 0023).

use serde::Serialize;

use super::{Entry, Feed, Funding, Image, Item, License, Person, Transcript};

/// An address field that is not a correct web address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadAddress {
    /// The path of the field in the JSON model, such as
    /// `items/2/enclosure/url`.
    pub field: String,
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

/// Collects each address field that is not correct.
struct Checker {
    found: Vec<BadAddress>,
}

impl Checker {
    fn field(&mut self, path: &str, value: Option<&String>) {
        // An empty field is no address (ADR 0023 item 2). A placeholder
        // address is replaced at the public address (ADR 0122 item 15).
        if let Some(value) = value.filter(|v| !v.is_empty())
            && !is_web_address(value)
            && !crate::inventory::is_placeholder_address(value)
        {
            self.found.push(BadAddress {
                field: path.to_owned(),
                value: value.clone(),
            });
        }
    }

    fn images(&mut self, path: &str, images: &[Entry<Image>]) {
        for (i, image) in images.iter().enumerate() {
            self.field(&format!("{path}images/{i}/href"), image.href.as_ref());
        }
    }

    fn persons(&mut self, path: &str, persons: &[Entry<Person>]) {
        for (i, person) in persons.iter().enumerate() {
            self.field(&format!("{path}persons/{i}/img"), person.img.as_ref());
            self.field(&format!("{path}persons/{i}/href"), person.href.as_ref());
        }
    }

    fn fundings(&mut self, path: &str, fundings: &[Entry<Funding>]) {
        for (i, funding) in fundings.iter().enumerate() {
            self.field(&format!("{path}fundings/{i}/url"), funding.url.as_ref());
        }
    }

    fn license(&mut self, path: &str, license: Option<&License>) {
        self.field(
            &format!("{path}license/url"),
            license.and_then(|l| l.url.as_ref()),
        );
    }

    fn transcripts(&mut self, path: &str, transcripts: &[Entry<Transcript>]) {
        for (i, transcript) in transcripts.iter().enumerate() {
            self.field(
                &format!("{path}transcripts/{i}/url"),
                transcript.url.as_ref(),
            );
        }
    }

    fn item(&mut self, path: &str, item: &Item) {
        self.field(
            &format!("{path}enclosure/url"),
            item.enclosure.as_ref().and_then(|e| e.url.as_ref()),
        );
        self.field(&format!("{path}image"), item.image.as_ref());
        self.images(path, &item.images);
        self.persons(path, &item.persons);
        self.fundings(path, &item.fundings);
        self.transcripts(path, &item.transcripts);
        self.license(path, item.license.as_ref());
    }
}

impl Feed {
    /// Each address field of the feed that is not a correct web address
    /// (ADR 0023 items 1 and 2). The core refuses a write when the list is
    /// not empty.
    #[must_use]
    pub fn bad_addresses(&self) -> Vec<BadAddress> {
        let mut checker = Checker { found: Vec::new() };
        checker.field("link", self.link.as_ref());
        checker.field("image", self.image.as_ref());
        checker.field(
            "rss_image/url",
            self.rss_image.as_ref().and_then(|i| i.url.as_ref()),
        );
        checker.images("", &self.images);
        checker.persons("", &self.persons);
        checker.fundings("", &self.fundings);
        checker.license("", self.license.as_ref());
        for (i, item) in self.items.iter().enumerate() {
            checker.item(&format!("items/{i}/"), item);
        }
        // A live item has the same address fields as an item, and its
        // content links (ADR 0023 item 1, ADR 0054 item 3).
        for (i, live) in self.live_items.iter().enumerate() {
            let path = format!("live_items/{i}/");
            checker.item(&format!("{path}item/"), &live.value.item);
            for (j, link) in live.value.content_links.iter().enumerate() {
                checker.field(
                    &format!("{path}content_links/{j}/href"),
                    link.value.href.as_ref(),
                );
            }
        }
        checker.found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
