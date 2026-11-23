//! Detection of a bad or missing Podcasting 2.0 medium (ADR 0250 items 1 to 3).

use serde::Serialize;

use crate::xml::{Document, Element, Node, Ns};

/// The cause of a medium problem (ADR 0250 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediumProblemKind {
    /// The Podcasting 2.0 element name is close to `medium` (ADR 0250 item 1).
    MisspelledTag,
    /// An element named `medium` uses a different namespace (ADR 0250 item 1).
    OtherNamespace,
    /// The Podcasting 2.0 medium value is close to an edited value (ADR 0250 item 1).
    MisspelledValue,
    /// The channel has audio enclosures but no medium element (ADR 0250 item 1).
    NoTag,
}

/// A bad or missing medium and its suggested correction (ADR 0250 item 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MediumProblem {
    /// The cause of the problem (ADR 0250 item 2).
    pub kind: MediumProblemKind,
    /// The complete XML element, when one caused the problem (ADR 0250 item 2).
    pub tag: Option<String>,
    /// The line of the element in the source document (ADR 0250 item 2).
    pub line: Option<u32>,
    /// The suggested edited medium (ADR 0250 item 2).
    pub suggested: Option<String>,
}

const EDITED: [&str; 3] = ["music", "musicL", "publisher"];
const KNOWN: [&str; 19] = [
    "podcast",
    "music",
    "video",
    "film",
    "audiobook",
    "newsletter",
    "blog",
    "publisher",
    "course",
    "podcastL",
    "musicL",
    "videoL",
    "filmL",
    "audiobookL",
    "newsletterL",
    "blogL",
    "publisherL",
    "courseL",
    "mixed",
];

/// Find a correctable medium problem in a feed document (ADR 0250 items 1 to 3).
///
/// # Examples
///
/// ```
/// use studio_core::medium_problem::medium_problem;
/// use studio_core::xml::Document;
///
/// let document = Document::parse("<rss><channel><item><enclosure type=\"audio/mpeg\" /></item></channel></rss>")?;
/// assert!(medium_problem(&document).is_some());
/// # Ok::<(), studio_core::xml::ParseError>(())
/// ```
#[must_use]
pub fn medium_problem(document: &Document) -> Option<MediumProblem> {
    find_medium_problem(document, false)
}

/// Find a medium problem in source bytes and include its line when the XML write is unchanged.
#[must_use]
pub fn medium_problem_in_bytes(bytes: &[u8]) -> Option<MediumProblem> {
    let document = Document::parse_bytes(bytes).ok()?;
    find_medium_problem(&document, document.write().as_bytes() == bytes)
}

fn find_medium_problem(document: &Document, can_find_line: bool) -> Option<MediumProblem> {
    let channel = document.root().child(Ns::None, "channel")?;
    let (written, spans) = document.write_with_spans();
    let channel_path = path_to_channel(document.root())?;
    let starts = crate::review::line_starts(&written);
    let location = SpanLocation {
        parent_path: &channel_path,
        spans: &spans,
        can_find_line,
        starts: &starts,
    };

    let direct: Vec<&Element> = channel.elements().collect();
    if direct
        .iter()
        .any(|e| e.is(Ns::Podcast, "medium") && known(e.text().trim()))
    {
        return None;
    }
    let medium = direct.iter().find(|e| e.is(Ns::Podcast, "medium")).copied();
    let bad = direct.iter().find_map(|element| {
        if Ns::Podcast.matches(element.namespace())
            && element.local_name() != "medium"
            && element.local_name() != "mediums"
        {
            let distance = edit_distance(&element.local_name().to_lowercase(), "medium");
            (distance <= 2 && distance > 0).then_some((element, MediumProblemKind::MisspelledTag))
        } else if element.local_name() == "medium" && !Ns::Podcast.matches(element.namespace()) {
            Some((element, MediumProblemKind::OtherNamespace))
        } else {
            None
        }
    });
    if let Some((element, kind)) = bad {
        let value = element.text().trim().to_owned();
        let suggested = EDITED
            .iter()
            .find(|known| known.eq_ignore_ascii_case(&value))
            .map(|known| (*known).to_owned());
        return Some(problem_for_element(
            document, element, kind, suggested, &location,
        ));
    }
    if let Some(element) = medium {
        let text = element.text();
        let value = text.trim();
        if KNOWN.iter().any(|known| known.eq_ignore_ascii_case(value)) {
            return None;
        }
        let suggested = EDITED.iter().min_by_key(|candidate| {
            edit_distance(&value.to_lowercase(), &candidate.to_lowercase())
        })?;
        let distance = edit_distance(&value.to_lowercase(), &suggested.to_lowercase());
        if distance == 0 || distance > 2 {
            return None;
        }
        return Some(problem_for_element(
            document,
            element,
            MediumProblemKind::MisspelledValue,
            Some((*suggested).to_owned()),
            &location,
        ));
    }
    if direct.iter().any(|e| e.local_name() == "medium") {
        return None;
    }
    let has_audio = direct
        .iter()
        .filter(|e| e.is(Ns::None, "item"))
        .any(|item| {
            item.child(Ns::None, "enclosure")
                .and_then(|e| e.attribute("type"))
                .is_some_and(|kind| kind.to_ascii_lowercase().starts_with("audio/"))
        });
    has_audio.then_some(MediumProblem {
        kind: MediumProblemKind::NoTag,
        tag: None,
        line: None,
        suggested: None,
    })
}

struct SpanLocation<'a> {
    parent_path: &'a [usize],
    spans: &'a [crate::xml::ElementSpan],
    can_find_line: bool,
    starts: &'a [usize],
}

fn problem_for_element(
    document: &Document,
    element: &Element,
    kind: MediumProblemKind,
    suggested: Option<String>,
    location: &SpanLocation<'_>,
) -> MediumProblem {
    let tag_path = element_path(document.root(), location.parent_path, element);
    let span = tag_path
        .as_ref()
        .and_then(|path| location.spans.iter().find(|span| span.path == *path));
    let line = if location.can_find_line {
        span.and_then(|span| line_number(location.starts, span.start))
    } else {
        None
    };
    let tag = span
        .and_then(|span| {
            document
                .write()
                .get(span.start..span.end)
                .map(str::to_owned)
        })
        .or_else(|| {
            Some(format!(
                "<{}>{}</{}>",
                element.name(),
                element.text(),
                element.name()
            ))
        });
    MediumProblem {
        kind,
        tag,
        line,
        suggested,
    }
}

fn line_number(starts: &[usize], offset: usize) -> Option<u32> {
    let index = starts
        .partition_point(|start| *start <= offset)
        .checked_sub(1)?;
    u32::try_from(index + 1).ok()
}

fn path_to_channel(root: &Element) -> Option<Vec<usize>> {
    root.elements()
        .position(|element| element.is(Ns::None, "channel"))
        .map(|index| vec![index])
}

fn element_path(root: &Element, channel_path: &[usize], wanted: &Element) -> Option<Vec<usize>> {
    let channel = root.elements().nth(*channel_path.first()?)?;
    channel
        .elements()
        .position(|element| std::ptr::eq(element, wanted))
        .map(|index| vec![channel_path[0], index])
}

fn known(value: &str) -> bool {
    KNOWN.iter().any(|known| known.eq_ignore_ascii_case(value))
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut row: Vec<usize> = (0..=right.chars().count()).collect();
    for (i, a) in left.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, b) in right.chars().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (row[j + 1] + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(a != b));
            diagonal = above;
        }
    }
    *row.last().unwrap_or(&0)
}

/// Remove or replace each element that can cause a medium problem.
pub(crate) fn replace_bad_elements(channel: &mut Element, kind: MediumProblemKind, medium: &str) {
    let mut first_position = None;
    let mut index = 0;
    while index < channel.children().len() {
        let is_bad = match &channel.children()[index] {
            Node::Element(element) => match kind {
                MediumProblemKind::MisspelledTag | MediumProblemKind::OtherNamespace => {
                    (Ns::Podcast.matches(element.namespace())
                        && element.local_name() != "medium"
                        && element.local_name() != "mediums"
                        && edit_distance(&element.local_name().to_lowercase(), "medium") <= 2)
                        || (element.local_name() == "medium"
                            && !Ns::Podcast.matches(element.namespace()))
                        || (element.is(Ns::Podcast, "medium") && !known(element.text().trim()))
                }
                _ => false,
            },
            _ => false,
        };
        if is_bad {
            let nodes = channel.children_mut();
            if first_position.is_none() {
                first_position = Some(index);
            }
            nodes.remove(index);
        } else {
            index += 1;
        }
    }
    if let Some(position) = first_position {
        channel.children_mut().insert(
            position,
            Node::Element(Element::new(
                "podcast:medium",
                Some(crate::xml::PODCAST_NS.to_owned()),
            )),
        );
        if let Node::Element(element) = &mut channel.children_mut()[position] {
            element.set_text(medium);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MediumProblemKind, medium_problem, medium_problem_in_bytes};
    use crate::xml::{Document, Node};

    fn parse(body: &str) -> Document {
        Document::parse(body).expect("test feed parses")
    }

    #[test]
    fn detects_a_misspelled_tag_and_source_line() {
        let body = format!(
            "{}<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:meduim>music</podcast:meduim></channel></rss>",
            "\n".repeat(11)
        );
        let problem = medium_problem_in_bytes(body.as_bytes()).expect("the typo is detected");
        assert_eq!(
            problem.kind,
            MediumProblemKind::MisspelledTag,
            "ADR 0250 item 1: check medium_problem in medium_problem.rs."
        );
        assert_eq!(
            problem.suggested.as_deref(),
            Some("music"),
            "ADR 0250 item 2: check medium_problem in medium_problem.rs."
        );
        assert_eq!(
            problem.line,
            Some(12),
            "ADR 0250 item 2: check medium_problem in medium_problem.rs."
        );
    }

    #[test]
    fn changed_source_bytes_have_no_line_number() {
        let mut document = parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:meduim>music</podcast:meduim></channel></rss>",
        );
        document.root_mut().set_attribute("id", Some("changed"));
        let problem = medium_problem(&document).expect("the typo remains detectable");
        assert_eq!(
            problem.line, None,
            "ADR 0250 item 2: a changed write has no source line."
        );
    }

    #[test]
    fn detects_other_namespace_and_misspelled_value() {
        let other = parse("<rss><channel><medium>music</medium></channel></rss>");
        assert_eq!(
            medium_problem(&other).expect("other namespace").kind,
            MediumProblemKind::OtherNamespace
        );
        let value = parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:medium>msuic</podcast:medium></channel></rss>",
        );
        let problem = medium_problem(&value).expect("misspelled value");
        assert_eq!(problem.kind, MediumProblemKind::MisspelledValue);
        assert_eq!(problem.suggested.as_deref(), Some("music"));
    }

    #[test]
    fn detects_a_missing_medium_only_with_audio_enclosure() {
        let missing =
            parse("<rss><channel><item><enclosure type=\"audio/mpeg\" /></item></channel></rss>");
        let problem = medium_problem(&missing).expect("audio feed needs a type choice");
        assert_eq!(problem.kind, MediumProblemKind::NoTag);
        assert_eq!(problem.suggested, None);
        assert_eq!(
            medium_problem(&parse(
                "<rss><channel><item><enclosure type=\"image/jpeg\" /></item></channel></rss>"
            )),
            None
        );
    }

    #[test]
    fn known_and_distant_values_do_not_give_a_problem() {
        for value in [
            "podcast",
            "audiobook",
            "video",
            "film",
            "newsletter",
            "blog",
            "course",
            "mixed",
            "music",
        ] {
            let xml = format!(
                "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:medium>{value}</podcast:medium></channel></rss>"
            );
            assert_eq!(
                medium_problem(&parse(&xml)),
                None,
                "ADR 0250 item 3: known medium `{value}` has no problem."
            );
        }
        let typo = parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><podcast:mediums>music</podcast:mediums></channel></rss>",
        );
        assert_eq!(
            medium_problem(&typo),
            None,
            "ADR 0250 item 1: distance above two gives no problem."
        );
    }

    #[test]
    fn replacing_a_bad_tag_keeps_other_source_bytes() {
        let source = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\n  <title> A &amp; B </title>\n  <podcast:meduim>music</podcast:meduim>\n  <description><![CDATA[x<y]]></description>\n</channel></rss>";
        let mut document = parse(source);
        let root = document.root_mut();
        let channel = root
            .children_mut()
            .iter_mut()
            .find_map(|node| match node {
                Node::Element(channel) if channel.is(crate::xml::Ns::None, "channel") => {
                    Some(channel)
                }
                _ => None,
            })
            .expect("channel exists");
        super::replace_bad_elements(channel, MediumProblemKind::MisspelledTag, "musicL");
        let expected = source.replace(
            "<podcast:meduim>music</podcast:meduim>",
            "<podcast:medium>musicL</podcast:medium>",
        );
        assert_eq!(
            document.write(),
            expected,
            "ADR 0250 item 7: only the medium element changes."
        );
    }
}
