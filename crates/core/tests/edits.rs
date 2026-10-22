//! Edits through the typed model change only the edited elements (ADR 0012).

use studio_core::feed::{Entry, Feed, Recipient};
use studio_core::xml::Document;

/// The remote item example from the namespace document for
/// `podcast:valueTimeSplit`, with a second item.
const SPLIT_EXAMPLE: &str = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Metal Showcase</title>
      <description>A great playlist of my favorite metal tracks.</description>
      <podcast:medium>music</podcast:medium>
      <item>
         <title>Special interview with Torcon VII</title>
         <guid>item-1</guid>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Alice (Podcaster)" type="node" address="02d5c1bf8b940dc9cadca86d1b0a3c37fbe39cee4c7e839e33bef9174531d27f52" split="95" />
            <podcast:valueRecipient name="Hosting Provider" type="node" address="03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a" split="5" fee="true" />
            <podcast:valueTimeSplit startTime="60" duration="237" remotePercentage="95">
               <podcast:remoteItem itemGuid="https://podcastindex.org/podcast/4148683#1" feedGuid="a94f5cc9-8c58-55fc-91fe-a324087a655b" medium="music" />
            </podcast:valueTimeSplit>
            <podcast:valueTimeSplit startTime="330" duration="53" remoteStartTime="174" remotePercentage="95">
               <podcast:remoteItem itemGuid="https://podcastindex.org/podcast/4148683#3" feedGuid="a94f5cc9-8c58-55fc-91fe-a324087a655b" medium="music" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
      <item>
         <title>Second</title>
         <guid>item-2</guid>
      </item>
   </channel>
</rss>
"#;

fn edit(source: &str, change: impl FnOnce(&mut Feed)) -> String {
    let mut doc = Document::parse(source).unwrap();
    let mut feed = Feed::read(&doc).unwrap();
    change(&mut feed);
    feed.write(&mut doc).unwrap();
    doc.write()
}

/// The lines of `after` that are not in the same place in `before`.
fn changed_lines<'a>(before: &str, after: &'a str) -> Vec<&'a str> {
    let before: Vec<&str> = before.lines().collect();
    after
        .lines()
        .enumerate()
        .filter(|(i, line)| before.get(*i) != Some(line))
        .map(|(_, line)| line)
        .collect()
}

#[test]
fn the_value_time_split_example_projects_to_the_documented_values() {
    let feed = Feed::read(&Document::parse(SPLIT_EXAMPLE).unwrap()).unwrap();
    let value = &feed.items[0].values[0];
    assert_eq!(value.recipients.len(), 2);
    assert_eq!(value.recipients[1].fee.as_deref(), Some("true"));
    let split = &value.time_splits[1];
    assert_eq!(split.start_time.as_deref(), Some("330"));
    assert_eq!(split.duration.as_deref(), Some("53"));
    assert_eq!(split.remote_start_time.as_deref(), Some("174"));
    assert_eq!(split.remote_percentage.as_deref(), Some("95"));
    assert_eq!(
        split.remote_items[0].item_guid.as_deref(),
        Some("https://podcastindex.org/podcast/4148683#3")
    );
}

#[test]
fn a_changed_split_changes_only_that_recipient() {
    let after = edit(SPLIT_EXAMPLE, |f| {
        f.items[0].values[0].recipients[0].split = Some("90".into());
    });
    let changed = changed_lines(SPLIT_EXAMPLE, &after);
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].contains(r#"split="90""#));
    assert!(changed[0].contains(r#"name="Alice (Podcaster)""#));
}

#[test]
fn a_new_recipient_goes_after_the_last_recipient() {
    let after = edit(SPLIT_EXAMPLE, |f| {
        f.items[0].values[0].recipients.push(Entry::new(Recipient {
            name: Some("Carol".into()),
            recipient_type: Some("lnaddress".into()),
            address: Some("carol@example.com".into()),
            split: Some("1".into()),
            ..Recipient::default()
        }));
    });
    let lines: Vec<&str> = after.lines().collect();
    let carol = lines.iter().position(|l| l.contains("Carol")).unwrap();
    assert!(lines[carol - 1].contains("Hosting Provider"));
    assert!(lines[carol + 1].contains("valueTimeSplit"));
    assert!(lines[carol].starts_with("            <podcast:valueRecipient "));
    let reread = Feed::read(&Document::parse(&after).unwrap()).unwrap();
    assert_eq!(reread.items[0].values[0].recipients.len(), 3);
}

#[test]
fn a_removed_item_goes_out_and_the_other_items_keep_their_bytes() {
    let after = edit(SPLIT_EXAMPLE, |f| {
        f.items.remove(1);
    });
    assert!(!after.contains("item-2"));
    let first_item = &SPLIT_EXAMPLE
        [SPLIT_EXAMPLE.find("<item>").unwrap()..SPLIT_EXAMPLE.find("</item>").unwrap()];
    assert!(after.contains(first_item));
    assert!(after.contains("</item>\n   </channel>"));
}

#[test]
fn reordered_items_keep_their_bytes() {
    let after = edit(SPLIT_EXAMPLE, |f| f.items.reverse());
    let one = after.find("item-1").unwrap();
    let two = after.find("item-2").unwrap();
    assert!(two < one);
    assert_eq!(after.len(), SPLIT_EXAMPLE.len());
}

#[test]
fn a_new_channel_element_goes_before_the_first_item() {
    let after = edit(SPLIT_EXAMPLE, |f| f.language = Some("en".into()));
    let changed = changed_lines(SPLIT_EXAMPLE, &after);
    assert_eq!(changed[0], "      <language>en</language>");
    assert!(after.contains("<language>en</language>\n      <item>"));
}

#[test]
fn a_podcast_element_in_a_feed_without_the_namespace_adds_the_declaration() {
    let source = "<rss version=\"2.0\">\n  <channel>\n    <title>T</title>\n  </channel>\n</rss>\n";
    let after = edit(source, |f| f.medium = Some("music".into()));
    let doc = Document::parse(&after).unwrap();
    assert_eq!(
        doc.root().attribute("xmlns:podcast"),
        Some("https://podcastindex.org/namespace/1.0")
    );
    let feed = Feed::read(&doc).unwrap();
    assert_eq!(feed.medium.as_deref(), Some("music"));
    assert!(after.contains(
        "    <title>T</title>\n    <podcast:medium>music</podcast:medium>\n  </channel>"
    ));
}

#[test]
fn a_cdata_title_stays_cdata_after_a_change() {
    let source = "<rss><channel><title><![CDATA[Old]]></title></channel></rss>";
    let after = edit(source, |f| f.title = Some("New & <b>".into()));
    assert_eq!(
        after,
        "<rss><channel><title><![CDATA[New & <b>]]></title></channel></rss>"
    );
}

#[test]
fn a_removed_channel_value_block_goes_out_with_its_line() {
    let source = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">\n<channel>\n  <title>T</title>\n  <podcast:value type=\"lightning\" method=\"keysend\"/>\n</channel>\n</rss>";
    let after = edit(source, |f| f.values.clear());
    assert_eq!(
        after,
        "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">\n<channel>\n  <title>T</title>\n</channel>\n</rss>"
    );
}
