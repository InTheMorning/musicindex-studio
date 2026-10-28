//! The Nostr key fields of a feed and the key rule (ADR 0074).

use serde::Serialize;

use super::{Entry, Feed, Person};
use crate::nostr_events::npub_text;

/// A key field that does not decode as a Nostr key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadKey {
    /// The path of the field in the JSON model, such as
    /// `items/2/persons/0/npub`.
    pub field: String,
    /// The value of the field.
    pub value: String,
}

/// True when `value` is not empty and is not in the `npub` form.
fn is_bad(value: &str) -> bool {
    !value.is_empty() && npub_text(value).as_deref() != Some(value)
}

fn persons<'a>(out: &mut Vec<(String, &'a str)>, path: &str, list: &'a [Entry<Person>]) {
    for (i, person) in list.iter().enumerate() {
        if let Some(npub) = person.npub.as_deref() {
            out.push((format!("{path}persons/{i}/npub"), npub));
        }
    }
}

/// Each key field of the feed, with its path in the JSON model.
fn fields(feed: &Feed) -> Vec<(String, &str)> {
    let mut out = Vec::new();
    persons(&mut out, "", &feed.persons);
    for (i, txt) in feed.txts.iter().enumerate() {
        if txt.purpose.as_deref() == Some("npub") {
            out.push((format!("txts/{i}/value"), txt.value.value.as_str()));
        }
    }
    for (j, item) in feed.items.iter().enumerate() {
        persons(&mut out, &format!("items/{j}/"), &item.persons);
    }
    for (j, live) in feed.live_items.iter().enumerate() {
        persons(&mut out, &format!("live_items/{j}/"), &live.item.persons);
    }
    out
}

impl Feed {
    /// Each key field of the feed that is not empty and is not an `npub`
    /// (ADR 0074 item 5). The core refuses a write for a bad key that the
    /// hosted feed does not have.
    #[must_use]
    pub fn bad_keys(&self) -> Vec<BadKey> {
        fields(self)
            .into_iter()
            .filter(|(_, value)| is_bad(value))
            .map(|(field, value)| BadKey {
                field,
                value: value.to_owned(),
            })
            .collect()
    }

    /// The text of each key field of the feed.
    #[must_use]
    pub fn key_texts(&self) -> Vec<String> {
        fields(self)
            .into_iter()
            .map(|(_, value)| value.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Item, LiveItem, Txt};
    use crate::nostr_events::test_npub;

    fn person(npub: Option<&str>) -> Entry<Person> {
        Entry::new(Person {
            name: "A".to_owned(),
            npub: npub.map(str::to_owned),
            ..Person::default()
        })
    }

    #[test]
    fn each_bad_key_field_has_its_model_path() {
        let good = test_npub(1);
        let good = good.as_str();
        let item = |npub| {
            Entry::new(Item {
                persons: vec![person(npub)],
                ..Item::default()
            })
        };
        let feed = Feed {
            persons: vec![person(Some(good)), person(Some("npub1xyz")), person(None)],
            txts: vec![
                Entry::new(Txt {
                    value: "other".to_owned(),
                    purpose: Some("proof".to_owned()),
                }),
                Entry::new(Txt {
                    value: "nsec1abc".to_owned(),
                    purpose: Some("npub".to_owned()),
                }),
            ],
            items: vec![item(None), item(Some("zzz")), item(Some(""))],
            live_items: vec![Entry::new(LiveItem {
                item: Item {
                    persons: vec![person(Some("bad"))],
                    ..Item::default()
                },
                ..LiveItem::default()
            })],
            ..Feed::default()
        };
        let fields: Vec<_> = feed.bad_keys().into_iter().map(|b| b.field).collect();
        assert_eq!(
            fields,
            [
                "persons/1/npub",
                "txts/1/value",
                "items/1/persons/0/npub",
                "live_items/0/persons/0/npub"
            ]
        );
    }
}
