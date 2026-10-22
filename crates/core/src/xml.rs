//! Lossless XML tree (ADR 0012).
//!
//! The reader keeps the source bytes of each tag, text run, comment and
//! declaration. A document that nobody changed writes the same bytes again.
//! A change to the name or the attributes of an element drops the source
//! bytes of its start tag, and the writer then makes a new start tag.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use quick_xml::escape::{escape, resolve_predefined_entity};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

/// The namespace URI of the RSS 2.0 iTunes extension.
pub const ITUNES_NS: &str = "http://www.itunes.com/dtds/podcast-1.0.dtd";

/// The namespace URI that the Podcasting 2.0 specification gives.
pub const PODCAST_NS: &str = "https://podcastindex.org/namespace/1.0";

/// Namespace URIs that feeds use for the Podcasting 2.0 namespace.
///
/// Some generators declare the URL of the specification document instead of
/// the namespace URI. The reader treats each of these as the podcast namespace.
pub const PODCAST_NS_ALIASES: [&str; 3] = [
    PODCAST_NS,
    "https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md",
    "http://podcastindex.org/namespace/1.0",
];

/// The namespace URI of Atom.
pub const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

/// A namespace that the feed model knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ns {
    /// No namespace: the plain RSS 2.0 elements.
    None,
    /// The iTunes extension.
    Itunes,
    /// The Podcasting 2.0 namespace, with each alias URI.
    Podcast,
    /// Atom.
    Atom,
}

impl Ns {
    /// True when `uri` is a URI of this namespace.
    #[must_use]
    pub fn matches(self, uri: Option<&str>) -> bool {
        match self {
            Ns::None => uri.is_none(),
            Ns::Itunes => uri == Some(ITUNES_NS),
            Ns::Podcast => uri.is_some_and(|u| PODCAST_NS_ALIASES.contains(&u)),
            Ns::Atom => uri == Some(ATOM_NS),
        }
    }

    /// The URI and the default prefix to declare when a document has no
    /// declaration for this namespace.
    #[must_use]
    pub fn declaration(self) -> Option<(&'static str, &'static str)> {
        match self {
            Ns::None => None,
            Ns::Itunes => Some(("itunes", ITUNES_NS)),
            Ns::Podcast => Some(("podcast", PODCAST_NS)),
            Ns::Atom => Some(("atom", ATOM_NS)),
        }
    }
}

/// An error from [`Document::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    position: u64,
    message: String,
}

impl ParseError {
    fn new(position: u64, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }

    /// The byte offset in the source where the reader stopped.
    #[must_use]
    pub fn position(&self) -> u64 {
        self.position
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "XML error at byte {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A parsed XML document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// True when the source starts with a UTF-8 byte order mark.
    bom: bool,
    prolog: Vec<Node>,
    root: Element,
    epilog: Vec<Node>,
}

/// One node in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// An element.
    Element(Element),
    /// Character data, escaped or in a CDATA section.
    Text(Text),
    /// A comment, a processing instruction, the XML declaration or a
    /// document type, kept as its source bytes.
    Markup(String),
}

/// Character data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    raw: String,
    cdata: bool,
}

impl Text {
    /// Make escaped character data from a plain value.
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self {
            raw: escape(value).into_owned(),
            cdata: false,
        }
    }

    /// Make a CDATA section from a plain value.
    ///
    /// A value that contains `]]>` cannot be in one CDATA section, so the
    /// function gives escaped character data for it.
    #[must_use]
    pub fn new_cdata(value: &str) -> Self {
        if value.contains("]]>") {
            return Self::new(value);
        }
        Self {
            raw: value.to_owned(),
            cdata: true,
        }
    }

    /// The value with each entity reference resolved.
    ///
    /// An entity that XML does not define, such as `&nbsp;`, stays as written.
    #[must_use]
    pub fn value(&self) -> Cow<'_, str> {
        if self.cdata {
            return Cow::Borrowed(&self.raw);
        }
        unescape_tolerant(&self.raw)
    }

    /// True when the source wrote this text as a CDATA section.
    #[must_use]
    pub fn is_cdata(&self) -> bool {
        self.cdata
    }

    fn is_whitespace(&self) -> bool {
        !self.cdata && self.raw.chars().all(char::is_whitespace)
    }

    fn write(&self, out: &mut String) {
        if self.cdata {
            out.push_str("<![CDATA[");
            out.push_str(&self.raw);
            out.push_str("]]>");
        } else {
            out.push_str(&self.raw);
        }
    }
}

/// An attribute with its value resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    name: String,
    value: String,
}

impl Attribute {
    /// The qualified name, as the source wrote it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value with each entity reference resolved.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// The source form of the end of an element.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceEnd {
    /// The element is new. The writer makes its tags.
    None,
    /// The source wrote an empty-element tag.
    SelfClosing,
    /// The source bytes of the end tag.
    Tag(String),
}

/// An element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    name: String,
    namespace: Option<String>,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
    /// The source bytes of the start tag, or `None` after a change.
    source_start: Option<String>,
    source_end: SourceEnd,
    /// The space before `/>` in the source, such as `" "`. A changed start
    /// tag keeps it, so that a change of an attribute does not change it.
    close_gap: String,
    /// True for an element that the writer made, not the parser.
    created: bool,
}

impl Element {
    /// Make an element with no attributes and no children.
    ///
    /// `name` is the qualified name and `namespace` is the URI that its
    /// prefix has in the document.
    #[must_use]
    pub fn new(name: impl Into<String>, namespace: Option<String>) -> Self {
        Self {
            name: name.into(),
            namespace,
            attributes: Vec::new(),
            children: Vec::new(),
            source_start: None,
            source_end: SourceEnd::None,
            close_gap: String::new(),
            created: true,
        }
    }

    /// The qualified name, as the source wrote it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The local name, without a prefix.
    #[must_use]
    pub fn local_name(&self) -> &str {
        self.name.rsplit(':').next().unwrap_or(&self.name)
    }

    /// The namespace URI of the element.
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// True when the element has this namespace and local name.
    #[must_use]
    pub fn is(&self, ns: Ns, local: &str) -> bool {
        self.local_name() == local && ns.matches(self.namespace())
    }

    /// The attributes in source sequence.
    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// The value of an attribute by its qualified name.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }

    /// Set, add or remove an attribute.
    ///
    /// The element keeps its source bytes when the value does not change.
    pub fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        let position = self.attributes.iter().position(|a| a.name == name);
        match (position, value) {
            (Some(i), Some(v)) if self.attributes[i].value == v => {}
            (Some(i), Some(v)) => {
                v.clone_into(&mut self.attributes[i].value);
                self.drop_start_tag();
            }
            (Some(i), None) => {
                self.attributes.remove(i);
                self.drop_start_tag();
            }
            (None, Some(v)) => {
                self.attributes.push(Attribute {
                    name: name.to_owned(),
                    value: v.to_owned(),
                });
                self.drop_start_tag();
            }
            (None, None) => {}
        }
    }

    /// The child nodes.
    #[must_use]
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// The child nodes, for a change.
    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    /// The child elements.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            _ => None,
        })
    }

    /// The first child element with this namespace and local name.
    #[must_use]
    pub fn child(&self, ns: Ns, local: &str) -> Option<&Element> {
        self.elements().find(|e| e.is(ns, local))
    }

    /// The text of the element: all its character data, joined.
    #[must_use]
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|n| match n {
                Node::Text(t) => Some(t.value()),
                _ => None,
            })
            .collect()
    }

    /// Replace the content of the element with one text node.
    ///
    /// The new text uses a CDATA section when the first text of the source
    /// used one. The element keeps its source bytes when the text does not
    /// change.
    pub fn set_text(&mut self, value: &str) {
        if self.text() == value && self.elements().next().is_none() {
            return;
        }
        let cdata = self.children.iter().any(|n| match n {
            Node::Text(t) => t.cdata,
            _ => false,
        });
        let text = if cdata {
            Text::new_cdata(value)
        } else {
            Text::new(value)
        };
        self.children = vec![Node::Text(text)];
    }

    fn drop_start_tag(&mut self) {
        self.source_start = None;
    }

    fn write(&self, out: &mut String) {
        match (&self.source_start, &self.source_end) {
            (Some(start), SourceEnd::SelfClosing) if self.children.is_empty() => {
                out.push_str(start);
            }
            (Some(start), SourceEnd::Tag(_)) => out.push_str(start),
            _ if self.children.is_empty() && !matches!(self.source_end, SourceEnd::Tag(_)) => {
                self.write_new_start(out, true);
                return;
            }
            _ => self.write_new_start(out, false),
        }
        if self.children.is_empty() && matches!(self.source_end, SourceEnd::SelfClosing) {
            return;
        }
        write_nodes(&self.children, out);
        if let SourceEnd::Tag(end) = &self.source_end {
            out.push_str(end);
        } else {
            out.push_str("</");
            out.push_str(&self.name);
            out.push('>');
        }
    }

    fn write_new_start(&self, out: &mut String, empty: bool) {
        out.push('<');
        out.push_str(&self.name);
        for a in &self.attributes {
            out.push(' ');
            out.push_str(&a.name);
            out.push_str("=\"");
            out.push_str(&escape(a.value.as_str()));
            out.push('"');
        }
        if empty {
            out.push_str(&self.close_gap);
            out.push_str("/>");
        } else {
            out.push('>');
        }
    }
}

/// The whitespace before `/>` at the end of a start tag.
fn close_gap(raw: &str) -> String {
    raw.strip_suffix("/>").map_or_else(String::new, |before| {
        before[before.trim_end().len()..].to_owned()
    })
}

/// Resolve entity references and keep each unknown reference as written.
fn unescape_tolerant(raw: &str) -> Cow<'_, str> {
    if !raw.contains('&') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let resolved = rest.find(';').and_then(|semi| {
            let name = &rest[1..semi];
            let value = if let Some(number) = name.strip_prefix('#') {
                let code = match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => number.parse().ok(),
                };
                char::from_u32(code?)?.to_string()
            } else {
                resolve_predefined_entity(name)?.to_owned()
            };
            Some((value, semi + 1))
        });
        if let Some((value, used)) = resolved {
            out.push_str(&value);
            rest = &rest[used..];
        } else {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

fn write_nodes(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Element(e) => e.write(out),
            Node::Text(t) => t.write(out),
            Node::Markup(m) => out.push_str(m),
        }
    }
}

impl Document {
    /// Parse a UTF-8 document.
    ///
    /// # Errors
    ///
    /// Gives a [`ParseError`] when the source is not well-formed XML or has
    /// no root element.
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let (bom, source) = match source.strip_prefix('\u{feff}') {
            Some(rest) => (true, rest),
            None => (false, source),
        };
        let mut doc = Parser::new(source).run()?;
        doc.bom = bom;
        Ok(doc)
    }

    /// Parse bytes. The bytes must be UTF-8 (ADR 0012).
    ///
    /// # Errors
    ///
    /// Gives a [`ParseError`] when the bytes are not UTF-8 or not
    /// well-formed XML.
    pub fn parse_bytes(source: &[u8]) -> Result<Self, ParseError> {
        let text = std::str::from_utf8(source).map_err(|e| {
            ParseError::new(
                u64::try_from(e.valid_up_to()).unwrap_or(u64::MAX),
                "the feed is not UTF-8",
            )
        })?;
        Self::parse(text)
    }

    /// The root element.
    #[must_use]
    pub fn root(&self) -> &Element {
        &self.root
    }

    /// The root element, for a change.
    pub fn root_mut(&mut self) -> &mut Element {
        &mut self.root
    }

    /// Write the document.
    #[must_use]
    pub fn write(&self) -> String {
        let mut out = String::new();
        if self.bom {
            out.push('\u{feff}');
        }
        write_nodes(&self.prolog, &mut out);
        self.root.write(&mut out);
        write_nodes(&self.epilog, &mut out);
        out
    }
}

/// Namespace declarations in scope for one element.
type Scope = Vec<(String, String)>;

struct Parser<'s> {
    source: &'s str,
    reader: Reader<&'s [u8]>,
    /// Open elements, from the root to the current element.
    stack: Vec<(Element, Scope)>,
    prolog: Vec<Node>,
    epilog: Vec<Node>,
    root: Option<Element>,
}

impl<'s> Parser<'s> {
    fn new(source: &'s str) -> Self {
        let mut reader = Reader::from_str(source);
        reader.config_mut().trim_text(false);
        reader.config_mut().check_end_names = true;
        Self {
            source,
            reader,
            stack: Vec::new(),
            prolog: Vec::new(),
            epilog: Vec::new(),
            root: None,
        }
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError::new(self.reader.buffer_position(), message)
    }

    fn run(mut self) -> Result<Document, ParseError> {
        loop {
            let start = self.position();
            let event = self
                .reader
                .read_event()
                .map_err(|e| ParseError::new(self.reader.error_position(), e.to_string()))?;
            let raw = &self.source[start..self.position()];
            match event {
                Event::Start(tag) => {
                    let (element, scope) = self.open(&tag, raw)?;
                    self.stack.push((element, scope));
                }
                Event::Empty(tag) => {
                    let (mut element, _) = self.open(&tag, raw)?;
                    element.source_end = SourceEnd::SelfClosing;
                    self.attach(Node::Element(element))?;
                }
                Event::End(_) => {
                    let (mut element, _) = self
                        .stack
                        .pop()
                        .ok_or_else(|| self.error("end tag without a start tag"))?;
                    element.source_end = SourceEnd::Tag(raw.to_owned());
                    self.attach(Node::Element(element))?;
                }
                Event::Text(_) | Event::GeneralRef(_) => self.attach_text(raw)?,
                Event::CData(data) => {
                    self.attach(Node::Text(Text {
                        raw: (*data).to_owned(),
                        cdata: true,
                    }))?;
                }
                Event::Comment(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {
                    self.attach(Node::Markup(raw.to_owned()))?;
                }
                Event::Eof => break,
            }
        }
        if !self.stack.is_empty() {
            return Err(self.error("the document ends before the root element closes"));
        }
        let root = self
            .root
            .take()
            .ok_or_else(|| self.error("the document has no root element"))?;
        Ok(Document {
            bom: false,
            prolog: self.prolog,
            root,
            epilog: self.epilog,
        })
    }

    fn position(&self) -> usize {
        usize::try_from(self.reader.buffer_position()).unwrap_or(usize::MAX)
    }

    fn open(&self, tag: &BytesStart<'_>, raw: &str) -> Result<(Element, Scope), ParseError> {
        let name = tag.name().as_ref().to_owned();
        let mut scope = self
            .stack
            .last()
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        let mut attributes = Vec::new();
        for attribute in tag.attributes() {
            let attribute = attribute.map_err(|e| self.error(e.to_string()))?;
            let key = attribute.key.as_ref().to_owned();
            // An entity that XML does not define stays as written, as in text.
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_or_else(|_| attribute.value.to_string(), Cow::into_owned);
            if key == "xmlns" {
                scope.push((String::new(), value.clone()));
            } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                scope.push((prefix.to_owned(), value.clone()));
            }
            attributes.push(Attribute { name: key, value });
        }
        let prefix = name.rsplit_once(':').map_or("", |(p, _)| p);
        let namespace = scope
            .iter()
            .rev()
            .find(|(p, _)| p == prefix)
            .map(|(_, uri)| uri.clone())
            .filter(|uri| !uri.is_empty());
        let element = Element {
            name,
            namespace,
            attributes,
            children: Vec::new(),
            source_start: Some(raw.to_owned()),
            source_end: SourceEnd::None,
            close_gap: close_gap(raw),
            created: false,
        };
        Ok((element, scope))
    }

    fn attach_text(&mut self, raw: &str) -> Result<(), ParseError> {
        let siblings = match self.stack.last_mut() {
            Some((parent, _)) => &mut parent.children,
            None if self.root.is_none() => &mut self.prolog,
            None => &mut self.epilog,
        };
        if let Some(Node::Text(text)) = siblings.last_mut()
            && !text.cdata
        {
            text.raw.push_str(raw);
            return Ok(());
        }
        self.attach(Node::Text(Text {
            raw: raw.to_owned(),
            cdata: false,
        }))
    }

    fn attach(&mut self, node: Node) -> Result<(), ParseError> {
        if let Some((parent, _)) = self.stack.last_mut() {
            parent.children.push(node);
            return Ok(());
        }
        match node {
            Node::Element(element) => {
                if self.root.is_some() {
                    return Err(self.error("the document has more than one root element"));
                }
                self.root = Some(element);
            }
            Node::Text(text) if !text.is_whitespace() => {
                return Err(self.error("text outside the root element"));
            }
            other if self.root.is_none() => self.prolog.push(other),
            other => self.epilog.push(other),
        }
        Ok(())
    }
}

/// Remove the child at `index`, with the whitespace text before it.
///
/// This keeps the layout of the parent when an element goes out.
pub(crate) fn remove_child(children: &mut Vec<Node>, index: usize) {
    children.remove(index);
    if index > 0
        && let Some(Node::Text(text)) = children.get(index - 1)
        && text.is_whitespace()
    {
        children.remove(index - 1);
    }
}

/// The whitespace text just before the child at `index`, if there is one.
///
/// The writer copies it before a new element, so that the new element has
/// the indentation of its neighbors.
/// The attribute names of the first source element of each qualified name,
/// in their source sequence.
pub(crate) fn attribute_orders(root: &Element) -> HashMap<String, Vec<String>> {
    fn walk(element: &Element, orders: &mut HashMap<String, Vec<String>>) {
        if !element.created && !element.attributes.is_empty() {
            orders
                .entry(element.name.clone())
                .or_insert_with(|| element.attributes.iter().map(|a| a.name.clone()).collect());
        }
        for node in &element.children {
            if let Node::Element(child) = node {
                walk(child, orders);
            }
        }
    }
    let mut orders = HashMap::new();
    walk(root, &mut orders);
    orders
}

/// Give each element that the writer made the attribute sequence of the
/// first source element with the same name. An attribute that the source
/// element does not have goes after the others, in its own sequence.
pub(crate) fn order_new_attributes(element: &mut Element, orders: &HashMap<String, Vec<String>>) {
    if element.created
        && let Some(order) = orders.get(&element.name)
    {
        element.attributes.sort_by_key(|a| {
            order
                .iter()
                .position(|name| *name == a.name)
                .unwrap_or(usize::MAX)
        });
    }
    for node in &mut element.children {
        if let Node::Element(child) = node {
            order_new_attributes(child, orders);
        }
    }
}

/// Give a new child element, and each new element in it, the indentation
/// and the close style of the document.
///
/// The indentation step is the difference between the space before the new
/// child and the space before the end tag of `parent`. The space before
/// `/>` comes from the first element in `parent` that closes itself. A
/// compact parent, with no such space, does not change.
pub(crate) fn format_new(parent: &mut Element, index: usize) {
    let Some(Node::Text(outer)) = indent_before(&parent.children, index) else {
        return;
    };
    let closing = match parent.children.last() {
        Some(Node::Text(t)) if t.is_whitespace() => t.raw.clone(),
        _ => return,
    };
    let Some(unit) = outer.raw.strip_prefix(closing.as_str()).map(str::to_owned) else {
        return;
    };
    if unit.is_empty() {
        return;
    }
    let gap = find_close_gap(parent).unwrap_or_default();
    if let Some(Node::Element(child)) = parent.children.get_mut(index) {
        indent_tree(child, &outer.raw, &unit, &gap);
    }
}

fn find_close_gap(element: &Element) -> Option<String> {
    element.children.iter().find_map(|n| match n {
        Node::Element(e) if e.source_start.as_deref().is_some_and(|s| s.ends_with("/>")) => {
            Some(e.close_gap.clone())
        }
        Node::Element(e) => find_close_gap(e),
        _ => None,
    })
}

fn indent_tree(element: &mut Element, outer: &str, unit: &str, gap: &str) {
    if element.source_start.is_some() {
        return;
    }
    if element.children.is_empty() {
        gap.clone_into(&mut element.close_gap);
        return;
    }
    if !element
        .children
        .iter()
        .all(|n| matches!(n, Node::Element(_)))
    {
        return;
    }
    let inner = format!("{outer}{unit}");
    let mut children = Vec::with_capacity(element.children.len() * 2 + 1);
    for node in std::mem::take(&mut element.children) {
        children.push(Node::Text(Text::new(&inner)));
        let mut node = node;
        if let Node::Element(child) = &mut node {
            indent_tree(child, &inner, unit, gap);
        }
        children.push(node);
    }
    children.push(Node::Text(Text::new(outer)));
    element.children = children;
}

pub(crate) fn indent_before(children: &[Node], index: usize) -> Option<Node> {
    let node = children.get(index.checked_sub(1)?)?;
    match node {
        Node::Text(t) if t.is_whitespace() => Some(node.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_changed_attribute_keeps_the_space_before_the_close() {
        let mut doc = Document::parse("<rss><a x=\"1\" /><b x=\"1\"/></rss>").unwrap();
        for child in doc.root_mut().children_mut() {
            if let Node::Element(e) = child {
                e.set_attribute("x", Some("2"));
            }
        }
        assert_eq!(doc.write(), "<rss><a x=\"2\" /><b x=\"2\"/></rss>");
    }

    #[test]
    fn write_after_parse_keeps_the_source_bytes() {
        let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- c -->\n\
            <rss  version='2.0' xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">\n\
            <channel><title><![CDATA[A & B]]></title><x a=\"1\"   />\
            <d>&lt;p&gt; &nbsp; &#233;</d></channel ></rss>\n";
        let doc = Document::parse(source).unwrap();
        assert_eq!(doc.write(), source);
    }

    #[test]
    fn a_byte_order_mark_stays() {
        let source = "\u{feff}<?xml version=\"1.0\"?>\n<a/>";
        assert_eq!(Document::parse(source).unwrap().write(), source);
    }

    #[test]
    fn text_value_resolves_entities_and_keeps_unknown_ones() {
        let doc = Document::parse("<a>&lt;p&gt; &nbsp;&#233;</a>").unwrap();
        assert_eq!(doc.root().text(), "<p> &nbsp;\u{e9}");
    }

    #[test]
    fn podcast_namespace_resolves_by_uri_not_by_prefix() {
        let doc = Document::parse(
            "<rss xmlns:p=\"https://podcastindex.org/namespace/1.0\" \
             xmlns:podcast=\"https://example.com/other\">\
             <p:guid/><podcast:guid/></rss>",
        )
        .unwrap();
        let children: Vec<_> = doc.root().elements().collect();
        assert!(children[0].is(Ns::Podcast, "guid"));
        assert!(!children[1].is(Ns::Podcast, "guid"));
    }

    #[test]
    fn github_namespace_alias_is_the_podcast_namespace() {
        let doc = Document::parse(
            "<rss xmlns:podcast=\"https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md\">\
             <podcast:medium>music</podcast:medium></rss>",
        )
        .unwrap();
        assert!(doc.root().child(Ns::Podcast, "medium").is_some());
    }

    #[test]
    fn malformed_xml_gives_an_error() {
        assert!(Document::parse("<rss><channel></rss>").is_err());
        assert!(Document::parse("").is_err());
        assert!(Document::parse("<a></a><b></b>").is_err());
        assert!(Document::parse_bytes(b"<a>\xff</a>").is_err());
    }

    #[test]
    fn a_changed_attribute_makes_a_new_start_tag_only() {
        let mut doc = Document::parse("<a  x='1'><b>t</b></a >").unwrap();
        doc.root_mut().set_attribute("x", Some("2"));
        assert_eq!(doc.write(), "<a x=\"2\"><b>t</b></a >");
    }

    #[test]
    fn an_unchanged_attribute_keeps_the_source_bytes() {
        let mut doc = Document::parse("<a  x='1'/>").unwrap();
        doc.root_mut().set_attribute("x", Some("1"));
        assert_eq!(doc.write(), "<a  x='1'/>");
    }

    #[test]
    fn set_text_keeps_a_cdata_section() {
        let mut doc = Document::parse("<a><![CDATA[old]]></a>").unwrap();
        doc.root_mut().set_text("new & more");
        assert_eq!(doc.write(), "<a><![CDATA[new & more]]></a>");
    }
}
