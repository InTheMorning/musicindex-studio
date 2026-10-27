//! Edits through the typed model change only the edited elements (ADR 0012).

use studio_core::feed::{Block, Entry, Feed, Item, License, Person, Recipient, Transcript};
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

#[test]
fn a_host_move_changes_only_the_self_link_and_adds_the_new_feed_url() {
    let source = "<rss xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\n<channel>\n  <title>T</title>\n  <atom:link href=\"https://old.example/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n  <item><guid>g1</guid></item>\n</channel>\n</rss>";
    let after = edit(source, |f| {
        assert_eq!(f.self_url(), Some("https://old.example/feed.xml"));
        let link = f
            .atom_links
            .iter_mut()
            .find(|l| l.rel.as_deref() == Some("self"))
            .unwrap();
        link.href = Some("https://new.example/feed.xml".into());
        f.new_feed_url = Some("https://new.example/feed.xml".into());
    });
    let feed = Feed::read(&Document::parse(&after).unwrap()).unwrap();
    assert_eq!(feed.self_url(), Some("https://new.example/feed.xml"));
    assert_eq!(
        feed.new_feed_url.as_deref(),
        Some("https://new.example/feed.xml")
    );
    assert_eq!(feed.items[0].guid.as_deref(), Some("g1"));
    assert!(after.contains(
        "<itunes:new-feed-url>https://new.example/feed.xml</itunes:new-feed-url>\n  <item>"
    ));
    let expected = source
        .replace(
            "https://old.example/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>",
            "https://new.example/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>",
        )
        .replace(
            "  <item>",
            "  <itunes:new-feed-url>https://new.example/feed.xml</itunes:new-feed-url>\n  <item>",
        );
    assert_eq!(after, expected);
}

#[test]
fn an_edit_of_one_item_keeps_the_feed_guid_and_each_item_guid() {
    let source = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:guid>f</podcast:guid><item><title>A</title><guid>a</guid></item><item><title>B</title><guid>b</guid></item></channel></rss>";
    let after = edit(source, |f| {
        f.items[1].title = Some("B2".into());
        f.items[0].transcripts.push(Entry::new(Transcript {
            url: Some("https://example.com/a.vtt".into()),
            media_type: Some("text/vtt".into()),
            ..Transcript::default()
        }));
        f.persons.push(Entry::new(Person {
            name: "Ann".into(),
            role: Some("vocals".into()),
            ..Person::default()
        }));
    });
    let feed = Feed::read(&Document::parse(&after).unwrap()).unwrap();
    assert_eq!(feed.podcast_guid.as_deref(), Some("f"));
    let guids: Vec<_> = feed.items.iter().map(|i| i.guid.clone().unwrap()).collect();
    assert_eq!(guids, ["a", "b"]);
    assert_eq!(
        feed.items[0].transcripts[0].media_type.as_deref(),
        Some("text/vtt")
    );
    assert_eq!(feed.persons[0].name, "Ann");
    assert!(after.contains("<podcast:person role=\"vocals\">Ann</podcast:person><item>"));
}

/// A feed with `title`, `itunes:author`, `description`, `link` and
/// `language`, then one item, in that sequence (ADR 0031).
const CHANNEL_SEQUENCE_EXAMPLE: &str = "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\n<channel>\n  <title>T</title>\n  <itunes:author>A</itunes:author>\n  <description>Old</description>\n  <link>https://example.com</link>\n  <language>en</language>\n  <item><guid>g1</guid></item>\n</channel>\n</rss>";

#[test]
fn a_description_removed_and_added_again_goes_back_to_its_hosted_position() {
    let hosted = Document::parse(CHANNEL_SEQUENCE_EXAMPLE).unwrap();
    let mut doc = Document::parse(CHANNEL_SEQUENCE_EXAMPLE).unwrap();
    // The user clears the field. The automatic save removes the element
    // (ADR 0022 item 6).
    let mut feed = Feed::read(&doc).unwrap();
    feed.description = None;
    feed.write(&mut doc).unwrap();
    assert!(!doc.write().contains("<description>"));
    // The user types a new text. The pipeline gives the hosted file to the
    // writer at each change of the draft (ADR 0031 item 4).
    let mut feed = Feed::read(&doc).unwrap();
    feed.description = Some("New".into());
    feed.write_hosted(&mut doc, Some(&hosted)).unwrap();
    let after = doc.write();
    let author = after.find("<itunes:author>").unwrap();
    let description = after.find("<description>").unwrap();
    let link = after.find("<link>").unwrap();
    assert!(author < description, "{after}");
    assert!(description < link, "{after}");
}

/// A feed with one item that has `title`, `itunes:image` and `guid`, in that
/// sequence (ADR 0031).
const ITEM_SEQUENCE_EXAMPLE: &str = "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\n<channel>\n  <item>\n    <title>T</title>\n    <itunes:image href=\"https://example.com/a.jpg\"/>\n    <guid>g1</guid>\n  </item>\n</channel>\n</rss>";

#[test]
fn an_item_image_removed_and_added_again_goes_back_to_its_hosted_position() {
    let hosted = Document::parse(ITEM_SEQUENCE_EXAMPLE).unwrap();
    let mut doc = Document::parse(ITEM_SEQUENCE_EXAMPLE).unwrap();
    let mut feed = Feed::read(&doc).unwrap();
    feed.items[0].image = None;
    feed.write(&mut doc).unwrap();
    assert!(!doc.write().contains("itunes:image"));
    let mut feed = Feed::read(&doc).unwrap();
    feed.items[0].image = Some("https://example.com/b.jpg".into());
    feed.write_hosted(&mut doc, Some(&hosted)).unwrap();
    let after = doc.write();
    let title = after.find("<title>").unwrap();
    let image = after.find("itunes:image").unwrap();
    let guid = after.find("<guid>").unwrap();
    assert!(title < image, "{after}");
    assert!(image < guid, "{after}");
}

#[test]
fn a_new_item_takes_the_sequence_of_the_last_hosted_item() {
    let hosted = Document::parse(ITEM_SEQUENCE_EXAMPLE).unwrap();
    let mut doc = Document::parse(ITEM_SEQUENCE_EXAMPLE).unwrap();
    // The first save gives the new item only its GUID, and a later save
    // gives it a title, as the editor does.
    let mut feed = Feed::read(&doc).unwrap();
    feed.items.push(Entry::new(Item {
        guid: Some("g2".into()),
        ..Item::default()
    }));
    feed.write_hosted(&mut doc, Some(&hosted)).unwrap();
    let mut feed = Feed::read(&doc).unwrap();
    feed.items[1].title = Some("New".into());
    feed.write_hosted(&mut doc, Some(&hosted)).unwrap();
    let after = doc.write();
    let new_item = &after[after.find("<guid>g1</guid>").unwrap()..];
    let title = new_item.find("<title>New</title>").unwrap();
    let guid = new_item.find(">g2</guid>").unwrap();
    assert!(title < guid, "{after}");
}

/// An item with `enclosure` and one `podcast:alternateEnclosure`, with
/// `height` and `timeOffset`, attributes that the model does not have
/// (ADR 0033).
const ALTERNATE_ENCLOSURE_EXAMPLE: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">\n  <channel>\n    <item>\n      <guid>g1</guid>\n      <enclosure url=\"https://example.com/a.mp3\" type=\"audio/mpeg\"/>\n      <podcast:alternateEnclosure type=\"video/mp4\" length=\"100\" height=\"720\" timeOffset=\"1.5\">\n        <podcast:source uri=\"https://example.com/a.mp4\"/>\n      </podcast:alternateEnclosure>\n    </item>\n  </channel>\n</rss>";

#[test]
fn a_changed_alternate_enclosure_length_keeps_height_and_time_offset() {
    let after = edit(ALTERNATE_ENCLOSURE_EXAMPLE, |f| {
        f.items[0].alternate_enclosures[0].length = Some("200".into());
    });
    let changed = changed_lines(ALTERNATE_ENCLOSURE_EXAMPLE, &after);
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].contains(r#"length="200""#));
    assert!(changed[0].contains(r#"height="720""#));
    assert!(changed[0].contains(r#"timeOffset="1.5""#));
}

#[test]
fn a_channel_element_absent_from_the_hosted_file_keeps_its_old_position() {
    let hosted = Document::parse(SPLIT_EXAMPLE).unwrap();
    let mut doc = Document::parse(SPLIT_EXAMPLE).unwrap();
    let mut feed = Feed::read(&doc).unwrap();
    feed.license = Some(License {
        identifier: "cc-by-4.0".into(),
        url: None,
    });
    feed.write_hosted(&mut doc, Some(&hosted)).unwrap();
    let after = doc.write();
    assert!(after.contains("<podcast:license>cc-by-4.0</podcast:license>\n      <item>"));
}

// Task 038: the space around text (ADR 0061) and the channel image (ADR 0060).

const SPACED: &str = "<rss version=\"2.0\">\n <channel>\n  <title>T</title>\n  <description>\n    testing stophammer\n  </description>\n  <link>  </link>\n </channel>\n</rss>\n";

const SPACED_CDATA: &str = "<rss version=\"2.0\">\n <channel>\n  <title>T</title>\n  <description><![CDATA[\n    testing <b>x</b>\n  ]]></description>\n </channel>\n</rss>\n";

#[test]
fn a_description_with_space_at_its_ends_reads_without_that_space() {
    let feed = Feed::read(&Document::parse(SPACED).unwrap()).unwrap();
    assert_eq!(feed.description.as_deref(), Some("testing stophammer"));
}

#[test]
fn a_write_with_no_edit_keeps_the_bytes_of_a_value_with_space() {
    assert_eq!(edit(SPACED, |_| {}), SPACED);
    assert_eq!(edit(SPACED_CDATA, |_| {}), SPACED_CDATA);
}

#[test]
fn a_new_value_is_written_with_no_added_space() {
    let after = edit(SPACED, |f| f.description = Some("new text".into()));
    assert!(
        after.contains("<description>new text</description>"),
        "{after}"
    );
}

#[test]
fn a_new_value_with_space_at_its_ends_is_written_without_it() {
    let after = edit(SPACED, |f| {
        f.description = Some(" \t\r\nnew text \n".into());
    });
    assert!(
        after.contains("<description>new text</description>"),
        "{after}"
    );
}

#[test]
fn a_value_with_space_that_equals_the_source_after_removal_keeps_the_source_bytes() {
    let after = edit(SPACED, |f| {
        f.description = Some("  testing stophammer\n".into());
    });
    assert_eq!(after, SPACED);
    let after = edit(SPACED_CDATA, |f| {
        f.description = Some("\ntesting <b>x</b> ".into());
    });
    assert_eq!(after, SPACED_CDATA);
}

#[test]
fn an_element_with_only_space_reads_as_an_empty_value() {
    let feed = Feed::read(&Document::parse(SPACED).unwrap()).unwrap();
    assert_eq!(feed.link.as_deref(), Some(""));
    assert_eq!(edit(SPACED, |f| f.link = Some(String::new())), SPACED);
}

#[test]
fn a_cdata_section_stays_a_cdata_section_after_an_edit() {
    let feed = Feed::read(&Document::parse(SPACED_CDATA).unwrap()).unwrap();
    assert_eq!(feed.description.as_deref(), Some("testing <b>x</b>"));
    let after = edit(SPACED_CDATA, |f| {
        f.description = Some(" other <i>y</i> ".into());
    });
    assert!(
        after.contains("<description><![CDATA[other <i>y</i>]]></description>"),
        "{after}"
    );
}

const DEMU_IMAGE: &str = "<rss version=\"2.0\">\n  <channel>\n    <title>T</title>\n    <language>en</language>\n    <image>\n      <url>https://m.example/a.jpg</url>\n      <title>T</title>\n      <link>https://m.example</link>\n      <description>Art</description>\n      <width>144</width>\n    </image>\n  </channel>\n</rss>\n";

#[test]
fn the_demu_feed_template_gives_rss_image_url_and_no_itunes_image() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/demu-feed-template.xml"
    ))
    .unwrap();
    let feed = Feed::read(&Document::parse_bytes(&bytes).unwrap()).unwrap();
    let image = feed.rss_image.expect("the template has an image");
    assert_eq!(
        image.url.as_deref(),
        Some("https://pbs.twimg.com/profile_images/1566110862087278593/puLJTmxF_400x400.jpg")
    );
    assert_eq!(image.title.as_deref(), Some("My Demu Album"));
    assert_eq!(feed.image, None);
}

#[test]
fn a_changed_rss_image_url_changes_one_line_and_adds_no_itunes_image() {
    let after = edit(DEMU_IMAGE, |f| {
        f.rss_image.as_mut().unwrap().url = Some("https://m.example/b.png".into());
    });
    let changed = changed_lines(DEMU_IMAGE, &after);
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].contains("https://m.example/b.png"));
    assert!(!after.contains("itunes:image"));
    assert!(after.contains("<width>144</width>"));
}

#[test]
fn a_feed_with_no_edit_keeps_the_bytes_of_its_rss_image() {
    assert_eq!(edit(DEMU_IMAGE, |_| {}), DEMU_IMAGE);
}

#[test]
fn removing_rss_image_and_image_leaves_no_cover_tag() {
    let both = "<rss xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\">\n  <channel>\n    <title>T</title>\n    <image>\n      <url>https://m.example/a.jpg</url>\n    </image>\n    <itunes:image href=\"https://m.example/b.jpg\"/>\n  </channel>\n</rss>\n";
    let feed = Feed::read(&Document::parse(both).unwrap()).unwrap();
    assert_eq!(feed.image.as_deref(), Some("https://m.example/b.jpg"));
    let after = edit(both, |f| {
        f.rss_image = None;
        f.image = None;
    });
    assert!(!after.contains("<image"), "{after}");
    assert!(!after.contains("itunes:image"), "{after}");
}

#[test]
fn feed_addresses_gives_the_url_of_rss_image() {
    let feed = Feed::read(&Document::parse(DEMU_IMAGE).unwrap()).unwrap();
    assert_eq!(
        studio_core::pipeline::feed_addresses(&feed),
        ["https://m.example/a.jpg"]
    );
}

#[test]
fn a_bad_rss_image_url_is_a_bad_address_with_its_model_path() {
    let feed = edit_feed(DEMU_IMAGE, |f| {
        f.rss_image.as_mut().unwrap().url = Some("m.example/a.jpg".into());
    });
    let fields: Vec<_> = feed.bad_addresses().into_iter().map(|b| b.field).collect();
    assert_eq!(fields, ["rss_image/url"]);
}

fn edit_feed(source: &str, change: impl FnOnce(&mut Feed)) -> Feed {
    let mut feed = Feed::read(&Document::parse(source).unwrap()).unwrap();
    change(&mut feed);
    feed
}

const BLOCKED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\">\n  <channel>\n    <podcast:block>yes</podcast:block>\n    <itunes:block>yes</itunes:block>\n    <title>T</title>\n    <podcast:block id=\"spotify\">no</podcast:block>\n  </channel>\n</rss>\n";

#[test]
fn the_model_reads_each_block_tag_and_a_read_with_no_edit_keeps_the_bytes() {
    let feed = Feed::read(&Document::parse(BLOCKED).unwrap()).unwrap();
    assert_eq!(feed.itunes_block.as_deref(), Some("yes"));
    let blocks: Vec<_> = feed
        .blocks
        .iter()
        .map(|b| (b.id.as_deref(), b.value.value.as_str()))
        .collect();
    assert_eq!(blocks, [(None, "yes"), (Some("spotify"), "no")]);
    assert!(feed.hidden());
    assert_eq!(edit(BLOCKED, |_| {}), BLOCKED);
}

#[test]
fn an_unblock_writes_no_in_place_and_keeps_the_platform_block() {
    let after = edit(BLOCKED, |f| {
        f.itunes_block = Some("No".into());
        f.blocks[0].value.value = "no".into();
    });
    assert_eq!(
        after,
        BLOCKED
            .replace(
                "<podcast:block>yes</podcast:block>",
                "<podcast:block>no</podcast:block>"
            )
            .replace(
                "<itunes:block>yes</itunes:block>",
                "<itunes:block>No</itunes:block>"
            )
    );
    let feed = Feed::read(&Document::parse(&after).unwrap()).unwrap();
    assert!(!feed.hidden());
}

#[test]
fn a_block_of_a_feed_with_no_block_tag_adds_both_tags() {
    let source = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\">\n  <channel>\n    <title>T</title>\n  </channel>\n</rss>\n";
    assert!(
        !Feed::read(&Document::parse(source).unwrap())
            .unwrap()
            .hidden()
    );
    let after = edit(source, |f| {
        f.itunes_block = Some("Yes".into());
        f.blocks.push(Entry::new(Block {
            value: "yes".into(),
            id: None,
        }));
    });
    assert!(
        after.contains("<itunes:block>Yes</itunes:block>"),
        "{after}"
    );
    assert!(
        after.contains("<podcast:block>yes</podcast:block>"),
        "{after}"
    );
    assert!(
        Feed::read(&Document::parse(&after).unwrap())
            .unwrap()
            .hidden()
    );
}

#[test]
fn only_a_block_for_each_app_hides_the_feed() {
    let feed = |channel: &str| {
        Feed::read(&Document::parse(&format!("<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\"><channel>{channel}</channel></rss>")).unwrap()).unwrap()
    };
    assert!(feed("<itunes:block>YES</itunes:block>").hidden());
    assert!(feed("<podcast:block> yes </podcast:block>").hidden());
    assert!(!feed("<podcast:block id=\"spotify\">yes</podcast:block>").hidden());
    assert!(!feed("<itunes:block>No</itunes:block><podcast:block>no</podcast:block>").hidden());
    assert!(!feed("").hidden());
}

const LISTED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" version=\"2.0\">\n  <channel>\n    <title>T</title>\n  </channel>\n</rss>\n";

fn read_feed(source: &str) -> Feed {
    Feed::read(&Document::parse(source).unwrap()).unwrap()
}

#[test]
fn set_hidden_true_adds_both_tags() {
    let mut feed = read_feed(LISTED);
    feed.set_hidden(true);
    assert_eq!(feed.itunes_block.as_deref(), Some("Yes"));
    assert_eq!(feed.blocks.len(), 1);
    assert_eq!(feed.blocks[0].value.value, "yes");
    assert_eq!(feed.blocks[0].value.id, None);
    assert!(feed.hidden());
}

#[test]
fn set_hidden_true_keeps_a_platform_block_in_its_place() {
    let line = "<podcast:block id=\"spotify\">no</podcast:block>";
    let source = LISTED.replace("<title>T</title>", &format!("<title>T</title>\n    {line}"));
    let mut feed = read_feed(&source);
    feed.set_hidden(true);
    assert_eq!(feed.blocks.len(), 2);
    assert_eq!(feed.blocks[0].value.id.as_deref(), Some("spotify"));
    assert_eq!(feed.blocks[1].value.id, None);
    assert_eq!(feed.blocks[1].value.value, "yes");
    // The document up to the platform block keeps its bytes (ADR 0012).
    let after = edit(&source, |f| f.set_hidden(true));
    let kept = &source[..source.find(line).unwrap() + line.len()];
    assert!(after.starts_with(kept), "{after}");
    assert!(
        after.contains("<podcast:block>yes</podcast:block>"),
        "{after}"
    );
}

#[test]
fn set_hidden_false_writes_no_and_keeps_the_platform_block() {
    let mut feed = read_feed(&BLOCKED.replace(
        "<podcast:block id=\"spotify\">no</podcast:block>",
        "<podcast:block id=\"spotify\">yes</podcast:block>",
    ));
    feed.set_hidden(false);
    assert_eq!(feed.itunes_block.as_deref(), Some("No"));
    assert_eq!(feed.blocks.len(), 2);
    assert_eq!(feed.blocks[0].value.value, "no");
    assert_eq!(feed.blocks[1].value.id.as_deref(), Some("spotify"));
    assert_eq!(feed.blocks[1].value.value, "yes");
    assert!(!feed.hidden());
}

#[test]
fn set_hidden_false_adds_no_tag() {
    let mut feed = read_feed(LISTED);
    feed.set_hidden(false);
    assert_eq!(feed.itunes_block, None);
    assert_eq!(feed.blocks.len(), 0);
}

#[test]
fn set_platform_hidden_changes_only_the_blocks_with_that_id() {
    let mut feed = read_feed(BLOCKED);
    assert!(feed.set_platform_hidden("spotify", true));
    assert_eq!(feed.blocks[0].value.value, "yes");
    assert_eq!(feed.blocks[1].value.value, "yes");
    assert_eq!(feed.itunes_block.as_deref(), Some("yes"));
    assert!(feed.set_platform_hidden("spotify", false));
    assert_eq!(feed.blocks[1].value.value, "no");
    assert_eq!(feed.blocks[0].value.value, "yes");
}

#[test]
fn set_platform_hidden_gives_false_for_an_unknown_id() {
    let mut feed = read_feed(BLOCKED);
    assert!(!feed.set_platform_hidden("apple", true));
    assert_eq!(feed.blocks[0].value.value, "yes");
    assert_eq!(feed.blocks[1].value.value, "no");
}
