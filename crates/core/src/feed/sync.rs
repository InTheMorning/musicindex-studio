//! Write typed values back into the lossless tree (ADR 0012).
//!
//! Each function here changes the tree only where the typed value is
//! different from the tree. An element that did not change keeps its source
//! bytes.

use crate::xml::{self, Document, Element, Node, Ns};

/// One entry of a modeled list, with the element that it came from.
///
/// The origin lets the writer keep the source bytes of an entry that did not
/// change, and keep the elements in it that the core does not model.
///
/// In JSON, the entry is its value with a `key`: its position in its list
/// when the core read it. A new entry has no key (ADR 0022).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entry<T> {
    /// The typed value.
    #[serde(flatten)]
    pub value: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key: Option<usize>,
    #[serde(skip)]
    origin: Option<Element>,
}

impl<T> Entry<T> {
    /// Make a new entry. The writer adds it as a new element.
    pub fn new(value: T) -> Self {
        Self {
            value,
            key: None,
            origin: None,
        }
    }

    /// The position of the entry in its list when the core read it. A new
    /// entry has no key.
    #[must_use]
    pub fn key(&self) -> Option<usize> {
        self.key
    }

    pub(crate) fn read(element: &Element, key: usize) -> Self
    where
        T: Modeled,
    {
        Self {
            value: T::read(element),
            key: Some(key),
            origin: Some(element.clone()),
        }
    }
}

/// A sent model that does not agree with the draft (ADR 0022).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError {
    message: String,
}

impl KeyError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for KeyError {}

/// Give each sent entry with a key the source element of the draft entry
/// with the same key (ADR 0022 items 2 to 4).
///
/// `draft` is `None` when the parent of the list is new. A key in such a
/// list is an error.
pub(crate) fn adopt_list<T: Modeled>(
    sent: &mut [Entry<T>],
    draft: Option<&[Entry<T>]>,
) -> Result<(), KeyError> {
    let mut used = std::collections::HashSet::new();
    for entry in sent {
        let Some(key) = entry.key else {
            entry.origin = None;
            entry.value.adopt(None)?;
            continue;
        };
        let source = draft
            .and_then(|d| d.get(key))
            .ok_or_else(|| KeyError::new(format!("the draft has no {} with key {key}", T::NAME)))?;
        if !used.insert(key) {
            return Err(KeyError::new(format!(
                "the key {key} occurs two times in the {} list",
                T::NAME
            )));
        }
        entry.origin.clone_from(&source.origin);
        entry.value.adopt(Some(&source.value))?;
    }
    Ok(())
}

/// Give an optional child with nested lists the source elements of the
/// draft child.
pub(crate) fn adopt_one<T: Modeled>(
    sent: Option<&mut T>,
    draft: Option<&T>,
) -> Result<(), KeyError> {
    match sent {
        Some(value) => value.adopt(draft),
        None => Ok(()),
    }
}

impl<T> std::ops::Deref for Entry<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Entry<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// A typed value that the core reads from one element and writes to it.
pub(crate) trait Modeled: Sized {
    /// The namespace of the element.
    const NS: Ns;
    /// The local name of the element.
    const NAME: &'static str;

    /// Read the value from its element.
    fn read(element: &Element) -> Self;

    /// Change `element` so that it holds this value.
    ///
    /// The function must not change a part of the element that already holds
    /// the value.
    fn apply(&self, element: &mut Element, names: &mut Names);

    /// Give the entries of each nested list the source elements of the draft
    /// value (ADR 0022). A value with no nested list has nothing to do.
    fn adopt(&mut self, _draft: Option<&Self>) -> Result<(), KeyError> {
        Ok(())
    }
}

/// Read the entries of one kind from the children of `parent`.
pub(crate) fn read_list<T: Modeled>(parent: &Element) -> Vec<Entry<T>> {
    parent
        .elements()
        .filter(|e| e.is(T::NS, T::NAME))
        .enumerate()
        .map(|(key, e)| Entry::read(e, key))
        .collect()
}

/// The namespace and the local name of an element: the pair that
/// [`Element::is`] tests (ADR 0031).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedType {
    namespace: Option<String>,
    local: String,
}

impl HostedType {
    /// The type of `element`.
    fn of(element: &Element) -> Self {
        Self {
            namespace: element.namespace().map(str::to_owned),
            local: element.local_name().to_owned(),
        }
    }

    /// True when this type is the type `(ns, local)`.
    fn is(&self, ns: Ns, local: &str) -> bool {
        self.local == local && ns.matches(self.namespace.as_deref())
    }

    /// True when `element` has this type.
    fn matches(&self, element: &Element) -> bool {
        element.local_name() == self.local && element.namespace() == self.namespace.as_deref()
    }
}

/// The hosted sequence of `parent`: the type of each element child, in
/// sequence (ADR 0031).
fn hosted_sequence(parent: &Element) -> Vec<HostedType> {
    parent.elements().map(HostedType::of).collect()
}

/// The hosted sequence of the channel, and of each item by its GUID
/// (ADR 0031).
///
/// The pipeline gives the hosted document at each change of the draft. The
/// writer uses it to put a new direct child of the channel or of an item at
/// its position in the hosted file.
struct Hosted {
    channel: Vec<HostedType>,
    items: std::collections::HashMap<String, Vec<HostedType>>,
    /// The sequence of the last `item` of the hosted file. A new item uses
    /// it as its model (ADR 0031 item 2).
    model_item: Option<Vec<HostedType>>,
}

impl Hosted {
    /// Read the hosted sequence of the channel and of each item. Gives none
    /// when the document has no `rss/channel` element.
    fn new(doc: &Document) -> Option<Self> {
        let channel = super::channel(doc.root())?;
        let mut items = std::collections::HashMap::new();
        let mut model_item = None;
        for child in channel.elements() {
            let is_item = child.is(Ns::None, "item") || child.is(Ns::Podcast, "liveItem");
            if is_item && let Some(guid) = read_text(child, Ns::None, "guid") {
                items.insert(guid, hosted_sequence(child));
            }
            if child.is(Ns::None, "item") {
                model_item = Some(hosted_sequence(child));
            }
        }
        Some(Self {
            channel: hosted_sequence(channel),
            items,
            model_item,
        })
    }
}

/// The source of the qualified names for new elements.
///
/// The writer uses the prefix that the root element declares for each
/// namespace. It collects each declaration that it must add, and the caller
/// puts them on the root element at the end.
pub(crate) struct Names {
    declared: Vec<(String, String)>,
    added: Vec<(&'static str, &'static str)>,
    /// The attribute sequence of each element name in the source, for new
    /// elements.
    orders: std::collections::HashMap<String, Vec<String>>,
    /// The hosted sequence of the channel and of each item, or none.
    /// `Names` goes to each writer of the tree. A writer of a direct child
    /// of the channel or of an item gets the hosted sequence of its parent
    /// from here. It uses [`Names::hosted_channel`] or
    /// [`Names::hosted_item`] (ADR 0031).
    hosted: Option<Hosted>,
}

impl Names {
    /// Make the write context. `hosted` is the hosted document of the
    /// draft, or none when the draft has no hosted file to compare against
    /// (ADR 0031).
    pub(crate) fn new(root: &Element, hosted: Option<&Document>) -> Self {
        let declared = root
            .attributes()
            .iter()
            .filter_map(|a| {
                let prefix = a.name().strip_prefix("xmlns:")?;
                Some((prefix.to_owned(), a.value().to_owned()))
            })
            .collect();
        Self {
            declared,
            added: Vec::new(),
            orders: xml::attribute_orders(root),
            hosted: hosted.and_then(Hosted::new),
        }
    }

    /// The hosted sequence of the channel, or none (ADR 0031).
    pub(crate) fn hosted_channel(&self) -> Option<Vec<HostedType>> {
        self.hosted.as_ref().map(|h| h.channel.clone())
    }

    /// The hosted sequence of the item with this GUID. An item that the
    /// hosted file does not have gets the sequence of the last hosted item
    /// as its model (ADR 0031 item 2).
    pub(crate) fn hosted_item(&self, guid: Option<&str>) -> Option<Vec<HostedType>> {
        let hosted = self.hosted.as_ref()?;
        guid.and_then(|g| hosted.items.get(g).cloned())
            .or_else(|| hosted.model_item.clone())
    }

    pub(crate) fn element(&mut self, ns: Ns, local: &str) -> Element {
        let Some((default_prefix, default_uri)) = ns.declaration() else {
            return Element::new(local, None);
        };
        let declared = self
            .declared
            .iter()
            .find(|(_, uri)| ns.matches(Some(uri)))
            .map(|(prefix, uri)| (prefix.clone(), uri.clone()));
        let (prefix, uri) = declared.unwrap_or_else(|| {
            if !self.added.contains(&(default_prefix, default_uri)) {
                self.added.push((default_prefix, default_uri));
            }
            (default_prefix.to_owned(), default_uri.to_owned())
        });
        Element::new(format!("{prefix}:{local}"), Some(uri))
    }

    /// Give each new element the attribute sequence of the source, and put
    /// each added declaration on the root element.
    pub(crate) fn finish(self, root: &mut Element) {
        xml::order_new_attributes(root, &self.orders);
        for (prefix, uri) in self.added {
            root.set_attribute(&format!("xmlns:{prefix}"), Some(uri));
        }
    }
}

/// Make the children of `parent` of kind `T` hold `entries`, in sequence.
///
/// The entries fill the positions of the existing elements of that kind, in
/// sequence. An entry more than the existing elements goes after the last
/// element of the kind, or before the first element in `before`, or at the
/// end of the parent. When the parent has no element of that kind, a new
/// entry can go to its position in `hosted` instead (ADR 0031), or, with no
/// such position, after the last child of a kind in `after`
/// (ADR 0033 item 6). An existing element more than the entries goes out.
pub(crate) fn write_list<T: Modeled>(
    parent: &mut Element,
    entries: &[Entry<T>],
    before: &[(Ns, &str)],
    after: &[(Ns, &str)],
    hosted: Option<&[HostedType]>,
    names: &mut Names,
) {
    let rendered: Vec<Element> = entries
        .iter()
        .map(|entry| {
            let mut element = match &entry.origin {
                Some(origin) => origin.clone(),
                None => names.element(T::NS, T::NAME),
            };
            entry.value.apply(&mut element, names);
            element
        })
        .collect();
    let slots: Vec<usize> = positions(parent, |e| e.is(T::NS, T::NAME));
    let children = parent.children_mut();
    let mut rendered = rendered.into_iter();
    // The positions of the new entries. At the end, the writer gives them
    // the format of the document.
    let mut new = Vec::new();
    let mut is_new = entries.iter().map(|e| e.origin.is_none());
    for &slot in &slots {
        match rendered.next() {
            Some(element) => {
                if is_new.next() == Some(true) {
                    new.push(slot);
                }
                if children[slot] != Node::Element(element.clone()) {
                    children[slot] = Node::Element(element);
                }
            }
            None => break,
        }
    }
    // Remove the slots that no entry filled, from the last to the first, so
    // that each index stays correct.
    for &slot in slots.iter().skip(entries.len()).rev() {
        xml::remove_child(children, slot);
    }
    let extra: Vec<Element> = rendered.collect();
    if !extra.is_empty() {
        let (mut index, indent) = insertion_point(
            children,
            &slots,
            entries.len(),
            before,
            after,
            hosted,
            (T::NS, T::NAME),
        );
        for element in extra {
            index = insert(children, index, indent.as_ref(), element);
            if is_new.next() == Some(true) {
                new.push(index - 1);
            }
        }
    }
    for index in new {
        xml::format_new(parent, index);
    }
}

/// Insert `indent` and `element` at `index`. Gives the index after them.
fn insert(
    children: &mut Vec<Node>,
    index: usize,
    indent: Option<&Node>,
    element: Element,
) -> usize {
    let mut index = index;
    if let Some(indent) = indent {
        children.insert(index, indent.clone());
        index += 1;
    }
    children.insert(index, Node::Element(element));
    index + 1
}

/// A function that writes a value into an element.
pub(crate) type Make<'a> = &'a dyn Fn(&mut Element, &mut Names);

/// Make the first child of kind (`ns`, `local`) hold `value`, or remove it.
///
/// `make` writes the value into an element. The element keeps its source
/// bytes when it already holds the value. When the parent has no element of
/// that kind, a new element can go to its position in `hosted` instead
/// (ADR 0031).
pub(crate) fn write_single(
    parent: &mut Element,
    ns: Ns,
    local: &str,
    value: Option<Make<'_>>,
    before: &[(Ns, &str)],
    hosted: Option<&[HostedType]>,
    names: &mut Names,
) {
    let slot = positions(parent, |e| e.is(ns, local)).first().copied();
    match (slot, value) {
        (Some(index), Some(make)) => {
            if let Node::Element(element) = &mut parent.children_mut()[index] {
                make(element, names);
            }
        }
        (Some(index), None) => xml::remove_child(parent.children_mut(), index),
        (None, Some(make)) => {
            let mut element = names.element(ns, local);
            make(&mut element, names);
            let children = parent.children_mut();
            let (index, indent) =
                insertion_point(children, &[], 0, before, &[], hosted, (ns, local));
            let after = insert(children, index, indent.as_ref(), element);
            xml::format_new(parent, after - 1);
        }
        (None, None) => {}
    }
}

/// Write an optional text child, such as `title`.
pub(crate) fn write_text(
    parent: &mut Element,
    ns: Ns,
    local: &str,
    value: Option<&str>,
    before: &[(Ns, &str)],
    hosted: Option<&[HostedType]>,
    names: &mut Names,
) {
    let make = value.map(|v| move |e: &mut Element, _: &mut Names| e.set_text(v));
    write_single(
        parent,
        ns,
        local,
        make.as_ref().map(|m| m as Make<'_>),
        before,
        hosted,
        names,
    );
}

/// Read the first child of kind `T`.
pub(crate) fn read_one<T: Modeled>(parent: &Element) -> Option<T> {
    parent.child(T::NS, T::NAME).map(T::read)
}

/// Writes the modeled children of one parent element.
///
/// A new child goes before the first child of a kind in `before`, or at the
/// end of the parent. The hosted sequence of the parent can give the new
/// child its hosted position. This happens when that sequence has the kind
/// of the child, and the parent has no element of that kind (ADR 0031).
pub(crate) struct Children<'p, 'n> {
    parent: &'p mut Element,
    before: &'static [(Ns, &'static str)],
    names: &'n mut Names,
    hosted: Option<Vec<HostedType>>,
}

impl<'p, 'n> Children<'p, 'n> {
    /// Make a writer for the direct children of `parent`.
    ///
    /// `hosted` is the hosted sequence of `parent`, or none. It is none
    /// when `parent` is new, is not the channel or an item, or has no
    /// hosted sequence (ADR 0031).
    pub(crate) fn new(
        parent: &'p mut Element,
        before: &'static [(Ns, &'static str)],
        names: &'n mut Names,
        hosted: Option<Vec<HostedType>>,
    ) -> Self {
        Self {
            parent,
            before,
            names,
            hosted,
        }
    }

    /// Write an optional text child, such as `title`.
    pub(crate) fn text(&mut self, ns: Ns, local: &str, value: Option<&String>) {
        write_text(
            self.parent,
            ns,
            local,
            value.map(String::as_str),
            self.before,
            self.hosted.as_deref(),
            self.names,
        );
    }

    /// Write the `href` attribute of an optional child, such as `itunes:image`.
    pub(crate) fn href(&mut self, ns: Ns, local: &str, value: Option<&String>) {
        let make =
            value.map(|v| move |e: &mut Element, _: &mut Names| e.set_attribute("href", Some(v)));
        write_single(
            self.parent,
            ns,
            local,
            make.as_ref().map(|m| m as Make<'_>),
            self.before,
            self.hosted.as_deref(),
            self.names,
        );
    }

    /// Write an optional child of kind `T`.
    pub(crate) fn one<T: Modeled>(&mut self, value: Option<&T>) {
        let make = value.map(|v| move |e: &mut Element, n: &mut Names| v.apply(e, n));
        write_single(
            self.parent,
            T::NS,
            T::NAME,
            make.as_ref().map(|m| m as Make<'_>),
            self.before,
            self.hosted.as_deref(),
            self.names,
        );
    }

    /// Write the children of kind `T`.
    pub(crate) fn list<T: Modeled>(&mut self, entries: &[Entry<T>]) {
        write_list(
            self.parent,
            entries,
            self.before,
            &[],
            self.hosted.as_deref(),
            self.names,
        );
    }

    /// Write the children of kind `T`, and put new ones before `before`.
    pub(crate) fn list_before<T: Modeled>(&mut self, entries: &[Entry<T>], before: &[(Ns, &str)]) {
        write_list(
            self.parent,
            entries,
            before,
            &[],
            self.hosted.as_deref(),
            self.names,
        );
    }

    /// Write the children of kind `T`. A new one with no hosted position
    /// goes after the last child of a kind in `after` (ADR 0033 item 6).
    pub(crate) fn list_after<T: Modeled>(&mut self, entries: &[Entry<T>], after: &[(Ns, &str)]) {
        write_list(
            self.parent,
            entries,
            self.before,
            after,
            self.hosted.as_deref(),
            self.names,
        );
    }
}

/// Read the text of an optional child.
pub(crate) fn read_text(parent: &Element, ns: Ns, local: &str) -> Option<String> {
    parent.child(ns, local).map(Element::text)
}

fn positions(parent: &Element, mut matches: impl FnMut(&Element) -> bool) -> Vec<usize> {
    parent
        .children()
        .iter()
        .enumerate()
        .filter_map(|(i, n)| match n {
            Node::Element(e) if matches(e) => Some(i),
            _ => None,
        })
        .collect()
}

/// Find where a new element goes, and the indentation to put before it.
///
/// `element_type` is the namespace and the local name of the new element.
/// `hosted` is the hosted sequence of the parent (ADR 0031). `after` gives
/// the kinds that a new element with no hosted position goes after, when the
/// parent has a child of one of them (ADR 0033 item 6).
fn insertion_point(
    children: &[Node],
    slots: &[usize],
    used: usize,
    before: &[(Ns, &str)],
    after: &[(Ns, &str)],
    hosted: Option<&[HostedType]>,
    element_type: (Ns, &str),
) -> (usize, Option<Node>) {
    // After the last remaining element of the same kind.
    if used > 0
        && let Some(&last) = slots.get(used - 1)
    {
        return (last + 1, xml::indent_before(children, last));
    }
    // At the hosted position, when the parent has no element of this kind
    // (ADR 0031).
    if slots.is_empty()
        && let Some(point) = hosted.and_then(|h| hosted_insertion_point(children, h, element_type))
    {
        return point;
    }
    // After the last child of a preferred kind, when the parent has no
    // element of this kind and no hosted position for it
    // (ADR 0033 item 6).
    if slots.is_empty()
        && let Some(point) = after_last_of(children, after)
    {
        return point;
    }
    // Before the first element of a kind that must come later.
    let first_later = children.iter().position(|n| match n {
        Node::Element(e) => before.iter().any(|&(ns, local)| e.is(ns, local)),
        _ => false,
    });
    if let Some(index) = first_later {
        let indent = xml::indent_before(children, index);
        let start = if indent.is_some() { index - 1 } else { index };
        return (start, indent);
    }
    // At the end, before the whitespace that closes the parent.
    let last_element = children.iter().rposition(|n| matches!(n, Node::Element(_)));
    match last_element {
        Some(last) => (last + 1, xml::indent_before(children, last)),
        None => (children.len(), None),
    }
}

/// The point after the last child of one of the kinds in `anchors`, or none
/// when `children` has no such child (ADR 0033 item 6).
fn after_last_of(children: &[Node], anchors: &[(Ns, &str)]) -> Option<(usize, Option<Node>)> {
    let last = children.iter().rposition(|n| match n {
        Node::Element(e) => anchors.iter().any(|&(ns, local)| e.is(ns, local)),
        _ => false,
    })?;
    Some((last + 1, xml::indent_before(children, last)))
}

/// Find the hosted position of a new element of `element_type` (ADR 0031).
///
/// The function looks at the types of the hosted sequence before the first
/// element of `element_type` there, from the nearest type back to the
/// first. It gives the point after the last child of `children` that has
/// one of these types. It gives none when the hosted sequence does not have
/// `element_type`, or when `children` has no child of an earlier type.
fn hosted_insertion_point(
    children: &[Node],
    hosted: &[HostedType],
    element_type: (Ns, &str),
) -> Option<(usize, Option<Node>)> {
    let (ns, local) = element_type;
    let first = hosted.iter().position(|t| t.is(ns, local))?;
    let after_earlier = hosted[..first].iter().rev().find_map(|earlier| {
        let last = children
            .iter()
            .rposition(|n| matches!(n, Node::Element(e) if earlier.matches(e)))?;
        Some((last + 1, xml::indent_before(children, last)))
    });
    // With no earlier sibling, the element goes before the nearest later
    // sibling (ADR 0031 item 1).
    after_earlier.or_else(|| {
        hosted[first + 1..].iter().find_map(|later| {
            let index = children
                .iter()
                .position(|n| matches!(n, Node::Element(e) if later.matches(e)))?;
            let indent = xml::indent_before(children, index);
            let start = if indent.is_some() { index - 1 } else { index };
            Some((start, indent))
        })
    })
}
