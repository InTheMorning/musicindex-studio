//! Write typed values back into the lossless tree (ADR 0012).
//!
//! Each function here changes the tree only where the typed value is
//! different from the tree. An element that did not change keeps its source
//! bytes.

use crate::xml::{self, Element, Node, Ns};

/// One entry of a modeled list, with the element that it came from.
///
/// The origin lets the writer keep the source bytes of an entry that did not
/// change, and keep the elements in it that the core does not model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<T> {
    /// The typed value.
    pub value: T,
    origin: Option<Element>,
}

impl<T> Entry<T> {
    /// Make a new entry. The writer adds it as a new element.
    pub fn new(value: T) -> Self {
        Self {
            value,
            origin: None,
        }
    }

    pub(crate) fn read(element: &Element) -> Self
    where
        T: Modeled,
    {
        Self {
            value: T::read(element),
            origin: Some(element.clone()),
        }
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
}

/// Read the entries of one kind from the children of `parent`.
pub(crate) fn read_list<T: Modeled>(parent: &Element) -> Vec<Entry<T>> {
    parent
        .elements()
        .filter(|e| e.is(T::NS, T::NAME))
        .map(Entry::read)
        .collect()
}

/// The source of the qualified names for new elements.
///
/// The writer uses the prefix that the root element declares for each
/// namespace. It collects each declaration that it must add, and the caller
/// puts them on the root element at the end.
pub(crate) struct Names {
    declared: Vec<(String, String)>,
    added: Vec<(&'static str, &'static str)>,
}

impl Names {
    pub(crate) fn new(root: &Element) -> Self {
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
        }
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

    /// Put each added declaration on the root element.
    pub(crate) fn finish(self, root: &mut Element) {
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
/// end of the parent. An existing element more than the entries goes out.
pub(crate) fn write_list<T: Modeled>(
    parent: &mut Element,
    entries: &[Entry<T>],
    before: &[(Ns, &str)],
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
    for &slot in &slots {
        match rendered.next() {
            Some(element) => {
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
    if extra.is_empty() {
        return;
    }
    let (mut index, indent) = insertion_point(children, &slots, entries.len(), before);
    for element in extra {
        index = insert(children, index, indent.as_ref(), element);
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
/// bytes when it already holds the value.
pub(crate) fn write_single(
    parent: &mut Element,
    ns: Ns,
    local: &str,
    value: Option<Make<'_>>,
    before: &[(Ns, &str)],
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
            let (index, indent) = insertion_point(children, &[], 0, before);
            insert(children, index, indent.as_ref(), element);
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
    names: &mut Names,
) {
    let make = value.map(|v| move |e: &mut Element, _: &mut Names| e.set_text(v));
    write_single(
        parent,
        ns,
        local,
        make.as_ref().map(|m| m as Make<'_>),
        before,
        names,
    );
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
fn insertion_point(
    children: &[Node],
    slots: &[usize],
    used: usize,
    before: &[(Ns, &str)],
) -> (usize, Option<Node>) {
    // After the last remaining element of the same kind.
    if used > 0
        && let Some(&last) = slots.get(used - 1)
    {
        return (last + 1, xml::indent_before(children, last));
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
