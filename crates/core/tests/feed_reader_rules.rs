//! The rules of musicindex ADR 0088 R-FEED-6 item 3 for the Studio feed
//! reader: the two namespace URIs of musicindex ADR 0086 R-FEED-3 with any
//! prefix, and the `fee` values of musicindex ADR 0086 R-FEED-4 item 5.
//! ADR 0244 records this work in Studio.

use studio_core::feed::{FeeReading, Feed};
use studio_core::feed_check::{Facts, FindingKind, FindingLevel, check};
use studio_core::xml::Document;

const OWNER: &str = "musicindex ADR 0088 R-FEED-6 item 3 (ADR 0244): fix FeeReading::of in crates/core/src/feed/value.rs";

fn reads(text: Option<&str>) -> FeeReading {
    FeeReading::of(text)
}

#[test]
fn r_feed_6_fee_true_is_a_fee() {
    assert_eq!(reads(Some("true")), FeeReading::Fee, "{OWNER}");
    assert_eq!(reads(Some(" TRUE ")), FeeReading::Fee, "{OWNER}");
}

#[test]
fn r_feed_6_fee_1_is_a_fee() {
    assert_eq!(reads(Some("1")), FeeReading::Fee, "{OWNER}");
    assert_eq!(reads(Some(" 1\n")), FeeReading::Fee, "{OWNER}");
}

#[test]
fn r_feed_6_fee_yes_is_a_fee() {
    assert_eq!(reads(Some("yes")), FeeReading::Fee, "{OWNER}");
    assert_eq!(reads(Some("Yes")), FeeReading::Fee, "{OWNER}");
}

#[test]
fn r_feed_6_fee_false_is_not_a_fee() {
    assert_eq!(reads(Some("false")), FeeReading::NotFee, "{OWNER}");
    assert_eq!(reads(Some(" False ")), FeeReading::NotFee, "{OWNER}");
}

#[test]
fn r_feed_6_fee_0_is_not_a_fee() {
    assert_eq!(reads(Some("0")), FeeReading::NotFee, "{OWNER}");
}

#[test]
fn r_feed_6_fee_no_is_not_a_fee() {
    assert_eq!(reads(Some("NO")), FeeReading::NotFee, "{OWNER}");
}

#[test]
fn r_feed_6_an_empty_fee_is_not_a_fee() {
    assert_eq!(reads(Some("")), FeeReading::NotFee, "{OWNER}");
    assert_eq!(reads(Some("   ")), FeeReading::NotFee, "{OWNER}");
}

#[test]
fn r_feed_6_a_missing_fee_is_not_a_fee() {
    assert_eq!(reads(None), FeeReading::NotFee, "{OWNER}");
}

#[test]
fn r_feed_6_an_unknown_fee_is_not_a_fee_and_is_unknown() {
    assert_eq!(reads(Some("maybe")), FeeReading::Unknown, "{OWNER}");
    assert_eq!(reads(Some("2")), FeeReading::Unknown, "{OWNER}");
}

fn feed(fees: &[Option<&str>]) -> (String, Feed) {
    let recipients: String = fees
        .iter()
        .enumerate()
        .map(|(n, fee)| {
            let fee = fee.map(|f| format!(" fee=\"{f}\"")).unwrap_or_default();
            format!("<podcast:valueRecipient name=\"R{n}\" type=\"node\" address=\"02{n:064}\" split=\"10\"{fee}/>")
        })
        .collect::<Vec<_>>()
        .concat();
    let source = format!(
        "<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>T</title><podcast:value type=\"lightning\" method=\"keysend\">{recipients}</podcast:value></channel></rss>"
    );
    let parsed = Feed::read(&Document::parse(&source).unwrap()).unwrap();
    (source, parsed)
}

fn unknown_fee_findings(feed: &Feed) -> Vec<studio_core::feed_check::Finding> {
    check(feed, &Facts::default())
        .into_iter()
        .filter(|f| f.kind == FindingKind::UnknownFee)
        .collect()
}

#[test]
fn r_feed_6_an_unknown_fee_gives_a_warning_and_keeps_the_recipient() {
    let owner = "musicindex ADR 0088 R-FEED-6 item 3 (ADR 0244 item 2): fix check_fee_values in crates/core/src/feed_check.rs";
    let (_, parsed) = feed(&[Some("maybe"), Some("true")]);
    assert_eq!(parsed.values[0].value.recipients.len(), 2, "{owner}");
    let found = unknown_fee_findings(&parsed);
    assert_eq!(found.len(), 1, "{owner}");
    assert_eq!(found[0].level, FindingLevel::Warning, "{owner}");
    assert_eq!(found[0].title, "R0 has an unknown fee value", "{owner}");
    assert!(found[0].reason.contains("\u{201c}maybe\u{201d}"), "{owner}");
}

#[test]
fn r_feed_6_each_known_fee_value_gives_no_warning() {
    let owner = "musicindex ADR 0088 R-FEED-6 item 3 (ADR 0244 item 2): fix check_fee_values in crates/core/src/feed_check.rs";
    let (_, parsed) = feed(&[
        Some("true"),
        Some("1"),
        Some("yes"),
        Some("false"),
        Some("0"),
        Some("no"),
        Some(""),
        None,
    ]);
    assert!(unknown_fee_findings(&parsed).is_empty(), "{owner}");
}

#[test]
fn r_feed_6_a_save_keeps_the_fee_text() {
    let owner = "musicindex ADR 0088 R-FEED-6 item 3 (ADR 0244 item 3): a save must not change the fee text. Fix the Recipient write in crates/core/src/feed/elements.rs";
    let (source, _) = feed(&[Some(" Yes "), Some("1"), Some("maybe"), Some("")]);
    let mut doc = Document::parse(&source).unwrap();
    let mut parsed = Feed::read(&doc).unwrap();
    parsed.title = Some("T2".into());
    parsed.write(&mut doc).unwrap();
    let saved = doc.write();
    for text in ["fee=\" Yes \"", "fee=\"1\"", "fee=\"maybe\"", "fee=\"\""] {
        assert!(saved.contains(text), "{owner}: {text} is not in {saved}");
    }
}

fn read_with(namespace: &str, prefix: &str) -> Feed {
    let source = format!(
        "<rss version=\"2.0\" xmlns:{prefix}=\"{namespace}\"><channel><title>T</title><{prefix}:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</{prefix}:guid><{prefix}:value type=\"lightning\" method=\"keysend\"><{prefix}:valueRecipient name=\"A\" type=\"node\" address=\"02{zero:064}\" split=\"100\" fee=\"yes\"/></{prefix}:value></channel></rss>",
        zero = 0
    );
    Feed::read(&Document::parse(&source).unwrap()).unwrap()
}

#[test]
fn r_feed_6_the_reader_accepts_the_namespace_uri_with_another_prefix() {
    let owner = "musicindex ADR 0088 R-FEED-6 item 3 and ADR 0086 R-FEED-3: fix PODCAST_NS_ALIASES in crates/core/src/xml.rs";
    let read = read_with("https://podcastindex.org/namespace/1.0", "pi");
    assert_eq!(
        read.podcast_guid.as_deref(),
        Some("917393e3-1b1e-5cef-ace4-edaa54e1f810"),
        "{owner}"
    );
    let recipient = &read.values[0].value.recipients[0].value;
    assert_eq!(
        FeeReading::of(recipient.fee.as_deref()),
        FeeReading::Fee,
        "{owner}"
    );
}

#[test]
fn r_feed_6_the_reader_accepts_the_specification_uri_with_another_prefix() {
    let owner = "musicindex ADR 0088 R-FEED-6 item 3 and ADR 0086 R-FEED-3: fix PODCAST_NS_ALIASES in crates/core/src/xml.rs";
    let read = read_with(
        "https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md",
        "ns",
    );
    assert_eq!(
        read.podcast_guid.as_deref(),
        Some("917393e3-1b1e-5cef-ace4-edaa54e1f810"),
        "{owner}"
    );
    assert_eq!(read.values[0].value.recipients.len(), 1, "{owner}");
}
