//! The findings about what the index holds (ADR 0175 items 3, 6, 10 and
//! 11). The check takes the facts as data and sends no request.

use super::{Facts, Finding, FindingKind, FindingLevel, same_but_spelling};
use crate::datetime;
use crate::musicindex::IndexCopy;

/// What the index holds for a feed (ADR 0175 item 2). The caller sets it
/// from the reads of the index. A feed with no `podcast:guid` has no index
/// facts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexFacts {
    /// False when the index has no record of the feed.
    pub held: bool,
    /// The address of the feed in the index.
    pub feed_url: Option<String>,
    /// The `lastBuildDate` that the index read, in Unix seconds.
    pub last_build_date: Option<i64>,
    /// The copies of the feed.
    pub copies: Vec<IndexCopy>,
}

/// The line under the rows (ADR 0175 item 10). It is not a finding and it
/// does not change the count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexLine {
    /// The index has no record of the feed.
    NotHeld,
    /// The index read a build time that is earlier than the last write.
    LastWriteNotRead,
}

impl IndexLine {
    /// The text for the user.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::NotHeld => "Music Index does not hold this feed yet.",
            Self::LastWriteNotRead => "Music Index has not read your last write yet.",
        }
    }
}

/// The state of the line under the rows. `base_build_date` is the
/// `lastBuildDate` text of the feed that Studio last wrote. A date that
/// Studio cannot read gives no line.
#[must_use]
pub fn index_line(index: Option<&IndexFacts>, base_build_date: Option<&str>) -> Option<IndexLine> {
    let index = index?;
    if !index.held {
        return Some(IndexLine::NotHeld);
    }
    let read = index.last_build_date?;
    let written = datetime::parse_rfc2822(base_build_date?)?.seconds;
    (read < written).then_some(IndexLine::LastWriteNotRead)
}

/// The report of ADR 0175 item 8. The user gives it to the operator of
/// Music Index. Studio sends no report. `public_address` is none when the
/// feed has no public address.
#[must_use]
pub fn copy_report(guid: Option<&str>, public_address: Option<&str>, copy_url: &str) -> String {
    format!(
        "Feed GUID: {}\nPublic address of the feed: {}\nCopy URL: {copy_url}\n\
         The copy gives the same feed GUID and has other payment recipients.",
        guid.unwrap_or("not known"),
        public_address.unwrap_or("not known"),
    )
}

/// The findings `index_address_differs` and `copy_other_recipients`.
pub(super) fn check_index(facts: &Facts, found: &mut Vec<Finding>) {
    let Some(index) = facts.index.as_ref().filter(|i| i.held) else {
        return;
    };
    if let (Some(public), Some(held)) = (facts.public_address.as_deref(), index.feed_url.as_deref())
        && held != public
        && !same_but_spelling(held, public)
    {
        found.push(Finding {
            kind: FindingKind::IndexAddressDiffers,
            level: FindingLevel::Notice,
            title: "Music Index has another address".to_owned(),
            reason: format!("Music Index holds {held}. The public address is {public}."),
            entry: None,
            repair: None,
            url: Some(held.to_owned()),
        });
    }
    for copy in index
        .copies
        .iter()
        .filter(|c| c.open && c.differs_recipients)
    {
        found.push(Finding {
            kind: FindingKind::CopyOtherRecipients,
            level: FindingLevel::Warning,
            title: "A copy pays other people".to_owned(),
            reason: format!(
                "The copy at {} gives the same feed GUID. It has other payment recipients.",
                copy.url
            ),
            entry: None,
            repair: None,
            url: Some(copy.url.clone()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::Feed;

    const PUBLIC: &str = "https://music.example/feed.xml";

    fn held(url: Option<&str>) -> IndexFacts {
        IndexFacts {
            held: true,
            feed_url: url.map(str::to_owned),
            ..IndexFacts::default()
        }
    }

    fn found(index: Option<IndexFacts>, public: Option<&str>) -> Vec<Finding> {
        let facts = Facts {
            public_address: public.map(str::to_owned),
            index,
            ..Facts::default()
        };
        let mut found = Vec::new();
        check_index(&facts, &mut found);
        found
    }

    fn kinds(found: &[Finding]) -> Vec<FindingKind> {
        found.iter().map(|f| f.kind).collect()
    }

    fn copy(open: bool, differs_recipients: bool) -> IndexCopy {
        IndexCopy {
            url: "https://mirror.example/feed.xml".to_owned(),
            open,
            differs_recipients,
        }
    }

    #[test]
    fn an_index_address_that_differs_gives_a_notice_with_no_repair() {
        let out = found(
            Some(held(Some("https://old.example/feed.xml"))),
            Some(PUBLIC),
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, FindingKind::IndexAddressDiffers);
        assert_eq!(out[0].level, FindingLevel::Notice);
        assert_eq!(out[0].url.as_deref(), Some("https://old.example/feed.xml"));
        assert_eq!(out[0].repair, None);
    }

    #[test]
    fn an_equal_address_a_spelling_difference_and_missing_facts_give_no_finding() {
        assert_eq!(kinds(&found(Some(held(Some(PUBLIC))), Some(PUBLIC))), []);
        assert_eq!(
            kinds(&found(
                Some(held(Some("HTTPS://Music.Example:443/feed.xml"))),
                Some(PUBLIC)
            )),
            []
        );
        let not_held = IndexFacts {
            held: false,
            feed_url: Some("https://old.example/feed.xml".to_owned()),
            ..IndexFacts::default()
        };
        assert_eq!(kinds(&found(Some(not_held), Some(PUBLIC))), []);
        assert_eq!(kinds(&found(Some(held(None)), Some(PUBLIC))), []);
        assert_eq!(
            kinds(&found(
                Some(held(Some("https://old.example/feed.xml"))),
                None
            )),
            []
        );
        assert_eq!(kinds(&found(None, Some(PUBLIC))), []);
    }

    #[test]
    fn an_open_copy_with_other_recipients_gives_a_warning() {
        let mut index = held(Some(PUBLIC));
        index.copies = vec![copy(true, true)];
        let out = found(Some(index), Some(PUBLIC));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, FindingKind::CopyOtherRecipients);
        assert_eq!(out[0].level, FindingLevel::Warning);
        assert_eq!(out[0].title, "A copy pays other people");
        assert!(out[0].reason.contains("https://mirror.example/feed.xml"));
        assert_eq!(
            out[0].url.as_deref(),
            Some("https://mirror.example/feed.xml")
        );
        assert_eq!(out[0].entry, None);
        assert_eq!(out[0].repair, None);
    }

    #[test]
    fn a_closed_copy_and_a_copy_with_the_same_recipients_give_no_finding() {
        let mut index = held(Some(PUBLIC));
        index.copies = vec![copy(false, true), copy(true, false), copy(false, false)];
        assert_eq!(kinds(&found(Some(index), Some(PUBLIC))), []);
    }

    #[test]
    fn the_check_gives_the_index_findings_and_default_facts_give_none() {
        let mut feed = Feed {
            language: Some("en".to_owned()),
            ..Feed::default()
        };
        feed.atom_links
            .push(crate::feed::Entry::new(crate::feed::AtomLink {
                href: Some(PUBLIC.to_owned()),
                rel: Some("self".to_owned()),
                ..crate::feed::AtomLink::default()
            }));
        let mut index = held(Some("https://old.example/feed.xml"));
        index.copies = vec![copy(true, true)];
        let facts = Facts {
            public_address: Some(PUBLIC.to_owned()),
            index: Some(index),
            ..Facts::default()
        };
        let got: Vec<_> = super::super::check(&feed, &facts)
            .iter()
            .map(|f| f.kind)
            .collect();
        assert_eq!(
            got,
            [
                FindingKind::IndexAddressDiffers,
                FindingKind::CopyOtherRecipients
            ]
        );
        assert_eq!(kinds(&super::super::check(&feed, &Facts::default())), []);
    }

    #[test]
    fn the_line_follows_the_index_record_and_the_build_times() {
        let base = Some("Fri, 6 Feb 2026 09:54:10 +0000");
        let at = datetime::parse_rfc2822("Fri, 6 Feb 2026 09:54:10 +0000")
            .unwrap()
            .seconds;
        let mut index = held(Some(PUBLIC));
        let not_held = IndexFacts::default();
        assert_eq!(index_line(Some(&not_held), base), Some(IndexLine::NotHeld));
        assert_eq!(index_line(None, base), None);

        index.last_build_date = Some(at - 1);
        assert_eq!(
            index_line(Some(&index), base),
            Some(IndexLine::LastWriteNotRead)
        );
        index.last_build_date = Some(at);
        assert_eq!(index_line(Some(&index), base), None);
        index.last_build_date = Some(at + 5);
        assert_eq!(index_line(Some(&index), base), None);
        index.last_build_date = None;
        assert_eq!(index_line(Some(&index), base), None);
        index.last_build_date = Some(at - 1);
        assert_eq!(index_line(Some(&index), None), None);
        assert_eq!(index_line(Some(&index), Some("not a date")), None);
        assert_eq!(
            IndexLine::NotHeld.text(),
            "Music Index does not hold this feed yet."
        );
        assert_eq!(
            IndexLine::LastWriteNotRead.text(),
            "Music Index has not read your last write yet."
        );
    }
}
