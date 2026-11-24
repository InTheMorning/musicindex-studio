//! XML writing tests.

use studio_core::feed::{Entry, Feed, RemoteItem};
use studio_core::xml::Document;

#[test]
fn new_remote_item_uses_later_source_attribute_order() {
    let source = r#"<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>Page</title><podcast:remoteItem feedGuid="first" medium="music" rel="artist"/><podcast:remoteItem feedGuid="second" medium="music" title="Second" rel="artist"/></channel></rss>"#;
    let mut document = Document::parse(source).unwrap();
    let mut feed = Feed::read(&document).unwrap();
    feed.remote_items.push(Entry::new(RemoteItem {
        feed_guid: Some("third".into()),
        medium: Some("music".into()),
        title: Some("Third".into()),
        rel: Some("artist".into()),
        ..RemoteItem::default()
    }));

    feed.write(&mut document).unwrap();

    let output = document.write();
    let third = output
        .split("<podcast:remoteItem")
        .find(|element| element.contains("feedGuid=\"third\""))
        .unwrap();
    assert!(
        third.find("title=").unwrap() < third.find("rel=").unwrap(),
        "observation 5 of the 2026-11-23 walk: xml::attribute_orders must merge later source order"
    );
}
