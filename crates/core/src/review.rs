//! The review of a draft: field changes, marked lines and their cover
//! (ADR 0114).
//!
//! [`review`] compares the live file with the draft in two ways. It
//! compares the typed models of the two files, and it gives a list of
//! field changes in groups. It also compares the two files line by line,
//! and it marks the changed part of each changed line. The cover of the
//! lines tells which changed lines a field change explains. A guard count
//! gives the changes that no field change explains.
//!
//! The module describes a draft. It changes no draft and no write.

use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

use serde::Serialize;
use serde_json::{Map, Value};
use similar::{
    Algorithm, ChangeTag, DiffTag, InlineChangeMode, InlineChangeOptions, TextDiff,
    capture_diff_slices,
};

use crate::feed::Feed;
use crate::xml::{Document, Element, ElementSpan, Ns};

/// The kind of a [`FieldGroup`] (ADR 0114 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    /// One channel field that is not a list, such as `title` or `owner`.
    /// The name is the key of the field.
    Field,
    /// The channel credits of one name.
    Credits,
    /// One item. The group has its position and its title.
    Track,
    /// One live item. The group has its position and its title.
    Live,
    /// Each other channel list, such as `values` or `fundings`. The name is
    /// the key of the list.
    List,
}

/// What happened to a value (ADR 0114 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    /// The draft has an entry or an object that the live file does not
    /// have.
    Added,
    /// The live file has an entry or an object that the draft does not
    /// have.
    Removed,
    /// A value is different. A missing value shows as `None`.
    Changed,
}

/// The element that explains a changed line, as a path from the root and
/// the part of the element that the change touches.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Place {
    path: Vec<usize>,
    /// True for the whole element. False for its start tag only.
    whole: bool,
}

/// One difference between the model of the live file and the model of the
/// draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldChange {
    /// What happened.
    pub op: ChangeOp,
    /// The path of the value in the group, with the keys of the model, such
    /// as `npub` or `enclosure.url`. A scalar field that is a group has an
    /// empty path. An entry that the group lists has an empty path. An
    /// entry of a list in the group has the path of that list.
    pub path: String,
    /// The entry of the change: the role of a credit or the name of a
    /// recipient.
    pub entry: Option<String>,
    /// The name of the credit, for a change of an entry of a `persons`
    /// list. It is the name in the draft, or in the live file when the
    /// draft has no name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The value in the live file.
    pub old: Option<Value>,
    /// The value in the draft.
    pub new: Option<Value>,
    #[serde(skip)]
    old_at: Option<Place>,
    #[serde(skip)]
    new_at: Option<Place>,
}

/// The changes of one element, such as a title, a credit name or a track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldGroup {
    /// The kind of the group.
    pub kind: GroupKind,
    /// The name, as the kind says.
    pub name: Option<String>,
    /// The position of a track or a live item, from 1.
    pub position: Option<usize>,
    /// The changes of the group.
    pub changes: Vec<FieldChange>,
}

/// The kind of a [`DiffLine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineKind {
    /// The line is in the two files.
    Same,
    /// The line is only in the live file.
    Removed,
    /// The line is only in the draft.
    Added,
}

/// A piece of a line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinePart {
    /// The text, with no line end.
    pub text: String,
    /// True when this part is the changed part of the line.
    pub changed: bool,
}

/// One line of the two files (ADR 0114 items 6 and 10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    /// The kind of the line.
    pub kind: LineKind,
    /// The line number in the live file, from 1.
    pub old: Option<usize>,
    /// The line number in the draft, from 1.
    pub new: Option<usize>,
    /// The parts of the line, in sequence.
    pub parts: Vec<LinePart>,
    /// For a removed line in a pair: the index in the list of the added
    /// line that replaces it.
    pub pair: Option<usize>,
    /// True when a field change explains the line.
    pub covered: bool,
}

/// The review of one draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    /// The field changes, in groups.
    pub fields: Vec<FieldGroup>,
    /// Each line of the two files.
    pub lines: Vec<DiffLine>,
    /// The number of changes that no field change explains.
    pub uncovered: usize,
}

/// Review a draft: compare the live file `base` with the draft `draft`.
///
/// An empty `base` is a feed with no live file. When a file does not parse
/// or its model does not read, the review has no field changes and each
/// changed line is uncovered.
#[must_use]
pub fn review(base: &[u8], draft: &[u8]) -> Review {
    let base_text = String::from_utf8_lossy(base);
    let draft_text = String::from_utf8_lossy(draft);
    let old = Model::load(base, &base_text);
    let new = Model::load(draft, &draft_text);
    let mut fields = match (&old, &new) {
        (Some(old), Some(new)) => diff_models(old, new),
        _ => Vec::new(),
    };
    let mut lines = diff_lines(&base_text, &draft_text);
    cover(
        &mut lines,
        &fields,
        (&base_text, old.as_ref()),
        (&draft_text, new.as_ref()),
    );
    let uncovered = count_uncovered(&lines);
    fields.shrink_to_fit();
    Review {
        fields,
        lines,
        uncovered,
    }
}

// ---------------------------------------------------------------------
// The table of the model keys (ADR 0114 item 5)
// ---------------------------------------------------------------------

/// The fields of one modeled object.
type Table = &'static [Field];

/// A key of the serialized model and the element that holds its value.
struct Field {
    key: &'static str,
    via: Via,
}

/// Where `Feed::read` reads a key from.
enum Via {
    /// The model computes the value from other keys. No element holds it.
    Computed,
    /// An attribute of the element of the object.
    Attr,
    /// The text of the element of the object.
    SelfText,
    /// A child element, with its text or its `href` attribute.
    Child(Ns, &'static str),
    /// A child element that is an object.
    Object(Ns, &'static str, Table),
    /// An object that is in the same element as its parent object.
    Within(Table),
    /// The child elements of one name.
    List(Ns, &'static str, Table),
}

const fn attr(key: &'static str) -> Field {
    Field {
        key,
        via: Via::Attr,
    }
}

const fn text(key: &'static str) -> Field {
    Field {
        key,
        via: Via::SelfText,
    }
}

const fn child(key: &'static str, ns: Ns, local: &'static str) -> Field {
    Field {
        key,
        via: Via::Child(ns, local),
    }
}

const fn object(key: &'static str, ns: Ns, local: &'static str, table: Table) -> Field {
    Field {
        key,
        via: Via::Object(ns, local, table),
    }
}

const fn list(key: &'static str, ns: Ns, local: &'static str, table: Table) -> Field {
    Field {
        key,
        via: Via::List(ns, local, table),
    }
}

const fn computed(key: &'static str) -> Field {
    Field {
        key,
        via: Via::Computed,
    }
}

const REMOTE_ITEM: Table = &[
    attr("feed_guid"),
    attr("feed_url"),
    attr("item_guid"),
    attr("medium"),
    attr("title"),
    attr("rel"),
];

const RECIPIENT: Table = &[
    attr("name"),
    attr("recipient_type"),
    attr("address"),
    attr("split"),
    attr("custom_key"),
    attr("custom_value"),
    attr("fee"),
];

const ENCLOSURE: Table = &[attr("url"), attr("length"), attr("media_type")];

const LOCKED: Table = &[text("value"), attr("owner")];

const SEASON: Table = &[text("value"), attr("name")];

const PERSON: Table = &[
    text("name"),
    attr("role"),
    attr("group"),
    attr("img"),
    attr("href"),
    attr("npub"),
];

const FUNDING: Table = &[text("label"), attr("url")];

const SOURCE: Table = &[attr("uri"), attr("content_type")];

const TRANSCRIPT: Table = &[
    attr("url"),
    attr("media_type"),
    attr("language"),
    attr("rel"),
];

const TXT: Table = &[text("value"), attr("purpose")];

const BLOCK: Table = &[text("value"), attr("id")];

const LICENSE: Table = &[text("identifier"), attr("url")];

const IMAGE: Table = &[
    attr("href"),
    attr("alt"),
    attr("aspect_ratio"),
    attr("width"),
    attr("height"),
    attr("media_type"),
    attr("purpose"),
];

const ATOM_LINK: Table = &[attr("href"), attr("rel"), attr("media_type")];

const CONTENT_LINK: Table = &[text("label"), attr("href")];

const LIVE_VALUE: Table = &[attr("uri"), attr("protocol")];

const RSS_IMAGE: Table = &[
    child("url", Ns::None, "url"),
    child("title", Ns::None, "title"),
    child("link", Ns::None, "link"),
    child("description", Ns::None, "description"),
];

const OWNER: Table = &[
    child("name", Ns::Itunes, "name"),
    child("email", Ns::Itunes, "email"),
];

const REMOTE_ITEMS_ONLY: Table = &[list("remote_items", Ns::Podcast, "remoteItem", REMOTE_ITEM)];

const TIME_SPLIT: Table = &[
    attr("start_time"),
    attr("duration"),
    attr("remote_start_time"),
    attr("remote_percentage"),
    list("remote_items", Ns::Podcast, "remoteItem", REMOTE_ITEM),
    list("recipients", Ns::Podcast, "valueRecipient", RECIPIENT),
];

const VALUE_BLOCK: Table = &[
    attr("value_type"),
    attr("method"),
    attr("suggested"),
    list("recipients", Ns::Podcast, "valueRecipient", RECIPIENT),
    list("time_splits", Ns::Podcast, "valueTimeSplit", TIME_SPLIT),
];

const ALTERNATE_ENCLOSURE: Table = &[
    attr("media_type"),
    attr("length"),
    attr("bitrate"),
    attr("title"),
    attr("rel"),
    attr("default"),
    list("sources", Ns::Podcast, "source", SOURCE),
];

const ITEM: Table = &[
    child("title", Ns::None, "title"),
    child("description", Ns::None, "description"),
    child("guid", Ns::None, "guid"),
    object("enclosure", Ns::None, "enclosure", ENCLOSURE),
    list(
        "alternate_enclosures",
        Ns::Podcast,
        "alternateEnclosure",
        ALTERNATE_ENCLOSURE,
    ),
    child("pub_date", Ns::None, "pubDate"),
    computed("pub_date_time"),
    child("duration", Ns::Itunes, "duration"),
    computed("duration_seconds"),
    child("explicit", Ns::Itunes, "explicit"),
    object("season", Ns::Podcast, "season", SEASON),
    child("episode", Ns::Podcast, "episode"),
    child("itunes_episode", Ns::Itunes, "episode"),
    child("image", Ns::Itunes, "image"),
    list("images", Ns::Podcast, "image", IMAGE),
    list("persons", Ns::Podcast, "person", PERSON),
    list("fundings", Ns::Podcast, "funding", FUNDING),
    list("transcripts", Ns::Podcast, "transcript", TRANSCRIPT),
    list("txts", Ns::Podcast, "txt", TXT),
    object("license", Ns::Podcast, "license", LICENSE),
    list("values", Ns::Podcast, "value", VALUE_BLOCK),
];

const LIVE_ITEM: Table = &[
    attr("status"),
    attr("start"),
    attr("end"),
    Field {
        key: "item",
        via: Via::Within(ITEM),
    },
    object("live_value", Ns::Podcast, "liveValue", LIVE_VALUE),
    list("content_links", Ns::Podcast, "contentLink", CONTENT_LINK),
];

/// The channel of a feed. This table has the keys of `Feed`.
const FEED: Table = &[
    child("title", Ns::None, "title"),
    child("description", Ns::None, "description"),
    child("link", Ns::None, "link"),
    child("language", Ns::None, "language"),
    child("managing_editor", Ns::None, "managingEditor"),
    child("web_master", Ns::None, "webMaster"),
    child("last_build_date", Ns::None, "lastBuildDate"),
    object("rss_image", Ns::None, "image", RSS_IMAGE),
    child("podcast_guid", Ns::Podcast, "guid"),
    child("medium", Ns::Podcast, "medium"),
    object("locked", Ns::Podcast, "locked", LOCKED),
    child("author", Ns::Itunes, "author"),
    child("explicit", Ns::Itunes, "explicit"),
    object("owner", Ns::Itunes, "owner", OWNER),
    child("keywords", Ns::Itunes, "keywords"),
    child("image", Ns::Itunes, "image"),
    child("new_feed_url", Ns::Itunes, "new-feed-url"),
    child("itunes_block", Ns::Itunes, "block"),
    list("blocks", Ns::Podcast, "block", BLOCK),
    list("atom_links", Ns::Atom, "link", ATOM_LINK),
    list("images", Ns::Podcast, "image", IMAGE),
    list("persons", Ns::Podcast, "person", PERSON),
    list("fundings", Ns::Podcast, "funding", FUNDING),
    list("txts", Ns::Podcast, "txt", TXT),
    object("license", Ns::Podcast, "license", LICENSE),
    object("publisher", Ns::Podcast, "publisher", REMOTE_ITEMS_ONLY),
    object("podroll", Ns::Podcast, "podroll", REMOTE_ITEMS_ONLY),
    list("remote_items", Ns::Podcast, "remoteItem", REMOTE_ITEM),
    list("values", Ns::Podcast, "value", VALUE_BLOCK),
    list("items", Ns::None, "item", ITEM),
    list("live_items", Ns::Podcast, "liveItem", LIVE_ITEM),
    computed("op3"),
];

// ---------------------------------------------------------------------
// The models and the element cursors
// ---------------------------------------------------------------------

/// A file with its model as a JSON value.
struct Model {
    /// The model with no `key` of an entry. A key is the position of an
    /// entry in its own file.
    json: Value,
    /// The document, or `None` for a feed with no live file.
    doc: Option<Document>,
}

impl Model {
    fn load(bytes: &[u8], text: &str) -> Option<Self> {
        if bytes.is_empty() {
            return Some(Self {
                json: model_json(&Feed::default())?,
                doc: None,
            });
        }
        let doc = Document::parse(text).ok()?;
        let feed = Feed::read(&doc).ok()?;
        Some(Self {
            json: model_json(&feed)?,
            doc: Some(doc),
        })
    }
}

fn model_json(feed: &Feed) -> Option<Value> {
    let mut json = serde_json::to_value(feed).ok()?;
    strip_keys(&mut json);
    Some(json)
}

/// Remove the `key` of each entry. No field of the model has this name.
pub(crate) fn strip_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("key");
            map.values_mut().for_each(strip_keys);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_keys),
        _ => {}
    }
}

/// An element of a document, with its path from the root.
#[derive(Clone)]
struct Cur<'d> {
    element: &'d Element,
    path: Vec<usize>,
}

impl<'d> Cur<'d> {
    /// The child elements with this name, in sequence.
    fn children(&self, ns: Ns, local: &str) -> Vec<Cur<'d>> {
        let element: &'d Element = self.element;
        element
            .elements()
            .enumerate()
            .filter(|(_, child)| child.is(ns, local))
            .map(|(index, child)| {
                let mut path = self.path.clone();
                path.push(index);
                Cur {
                    element: child,
                    path,
                }
            })
            .collect()
    }

    fn child(&self, ns: Ns, local: &str) -> Option<Cur<'d>> {
        self.children(ns, local).into_iter().next()
    }

    fn place(&self, whole: bool) -> Place {
        Place {
            path: self.path.clone(),
            whole,
        }
    }
}

/// The `channel` of a feed document.
fn channel_cursor(doc: &Document) -> Option<Cur<'_>> {
    let root = Cur {
        element: doc.root(),
        path: Vec::new(),
    };
    if !root.element.is(Ns::None, "rss") {
        return None;
    }
    root.child(Ns::None, "channel")
}

/// One modeled object and the element that it comes from.
#[derive(Clone)]
struct Obj<'a, 'd> {
    map: &'a Map<String, Value>,
    cur: Option<Cur<'d>>,
}

impl<'d> Obj<'_, 'd> {
    fn child(&self, ns: Ns, local: &str) -> Option<Cur<'d>> {
        self.cur.as_ref().and_then(|cur| cur.child(ns, local))
    }

    fn whole(&self) -> Value {
        Value::Object(self.map.clone())
    }
}

// ---------------------------------------------------------------------
// The field changes
// ---------------------------------------------------------------------

fn diff_models(old: &Model, new: &Model) -> Vec<FieldGroup> {
    let (Some(old_map), Some(new_map)) = (old.json.as_object(), new.json.as_object()) else {
        return Vec::new();
    };
    let old_obj = Obj {
        map: old_map,
        cur: old.doc.as_ref().and_then(channel_cursor),
    };
    let new_obj = Obj {
        map: new_map,
        cur: new.doc.as_ref().and_then(channel_cursor),
    };
    let mut groups = Vec::new();
    for field in FEED {
        match field.key {
            "persons" => credit_groups(field, &old_obj, &new_obj, &mut groups),
            "items" => item_groups(field, GroupKind::Track, &old_obj, &new_obj, &mut groups),
            "live_items" => item_groups(field, GroupKind::Live, &old_obj, &new_obj, &mut groups),
            key => {
                let mut changes = Vec::new();
                walk_field(
                    field,
                    Some(&old_obj),
                    Some(&new_obj),
                    "",
                    Who::default(),
                    &mut changes,
                );
                let kind = if matches!(field.via, Via::List(..)) {
                    GroupKind::List
                } else {
                    GroupKind::Field
                };
                push_group(&mut groups, kind, Some(key.to_owned()), None, changes);
            }
        }
    }
    groups
}

fn push_group(
    groups: &mut Vec<FieldGroup>,
    kind: GroupKind,
    name: Option<String>,
    position: Option<usize>,
    changes: Vec<FieldChange>,
) {
    if changes.is_empty() {
        return;
    }
    if kind == GroupKind::Credits
        && let Some(group) = groups
            .iter_mut()
            .find(|g| g.kind == GroupKind::Credits && g.name == name)
    {
        group.changes.extend(changes);
        return;
    }
    groups.push(FieldGroup {
        kind,
        name,
        position,
        changes,
    });
}

/// The entries of the list `key` of `parent`.
fn entries<'a, 'd>(
    parent: Option<&Obj<'a, 'd>>,
    key: &str,
    ns: Ns,
    local: &str,
) -> Vec<Obj<'a, 'd>> {
    let Some(parent) = parent else {
        return Vec::new();
    };
    let Some(values) = parent.map.get(key).and_then(Value::as_array) else {
        return Vec::new();
    };
    let cursors = parent
        .cur
        .as_ref()
        .map(|cur| cur.children(ns, local))
        .unwrap_or_default();
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            Some(Obj {
                map: value.as_object()?,
                cur: cursors.get(index).cloned(),
            })
        })
        .collect()
}

fn credit_groups(field: &Field, old: &Obj, new: &Obj, groups: &mut Vec<FieldGroup>) {
    let Via::List(ns, local, sub) = &field.via else {
        return;
    };
    let olds = entries(Some(old), field.key, *ns, local);
    let news = entries(Some(new), field.key, *ns, local);
    for matched in match_entries(&olds, &news) {
        let mut changes = Vec::new();
        match_changes(
            &matched,
            sub,
            &olds,
            &news,
            "",
            field.key,
            Who::default(),
            &mut changes,
        );
        let name = match matched {
            Match::Pair(_, j) | Match::Added(j) => string_at(news[j].map, "name"),
            Match::Removed(i) => string_at(olds[i].map, "name"),
        };
        push_group(groups, GroupKind::Credits, name, None, changes);
    }
}

fn item_groups(field: &Field, kind: GroupKind, old: &Obj, new: &Obj, groups: &mut Vec<FieldGroup>) {
    let Via::List(ns, local, sub) = &field.via else {
        return;
    };
    let olds = entries(Some(old), field.key, *ns, local);
    let news = entries(Some(new), field.key, *ns, local);
    let guid_of = |obj: &Obj| match kind {
        GroupKind::Live => obj
            .map
            .get("item")
            .and_then(Value::as_object)
            .and_then(|item| string_at(item, "guid")),
        _ => string_at(obj.map, "guid"),
    };
    let title_of = |obj: &Obj| match kind {
        GroupKind::Live => obj
            .map
            .get("item")
            .and_then(Value::as_object)
            .and_then(|item| string_at(item, "title")),
        _ => string_at(obj.map, "title"),
    };
    let old_guids: Vec<_> = olds.iter().map(guid_of).collect();
    let new_guids: Vec<_> = news.iter().map(guid_of).collect();
    let mut used = HashSet::new();
    let mut matches = Vec::new();
    for (j, guid) in new_guids.iter().enumerate() {
        let found = match guid {
            Some(guid) => old_guids
                .iter()
                .enumerate()
                .find(|(i, g)| g.as_ref() == Some(guid) && !used.contains(i))
                .map(|(i, _)| i),
            None => {
                (old_guids.get(j).is_some_and(Option::is_none) && !used.contains(&j)).then_some(j)
            }
        };
        match found {
            Some(i) => {
                used.insert(i);
                matches.push(Match::Pair(i, j));
            }
            None => matches.push(Match::Added(j)),
        }
    }
    matches.extend(
        (0..olds.len())
            .filter(|i| !used.contains(i))
            .map(Match::Removed),
    );
    for matched in matches {
        let mut changes = Vec::new();
        match_changes(
            &matched,
            sub,
            &olds,
            &news,
            "",
            field.key,
            Who::default(),
            &mut changes,
        );
        let (position, name) = match matched {
            Match::Pair(_, j) | Match::Added(j) => (j + 1, title_of(&news[j])),
            Match::Removed(i) => (i + 1, title_of(&olds[i])),
        };
        push_group(groups, kind, name, Some(position), changes);
    }
}

fn string_at(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// How an entry of the live file and an entry of the draft go together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Match {
    /// The old entry at the first index is the new entry at the second.
    Pair(usize, usize),
    /// The old entry has no new entry.
    Removed(usize),
    /// The new entry has no old entry.
    Added(usize),
}

/// Match the entries of a list through a sequence diff of their values. In
/// a replacement with as many old as new entries, the k-th old entry is the
/// k-th new entry (ADR 0114 item 4 of the architecture of task 077a).
fn match_entries(olds: &[Obj], news: &[Obj]) -> Vec<Match> {
    let text = |objs: &[Obj]| -> Vec<String> {
        objs.iter()
            .map(|obj| Value::Object(obj.map.clone()).to_string())
            .collect()
    };
    let (old_text, new_text) = (text(olds), text(news));
    let mut matches = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &old_text, &new_text) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        match tag {
            DiffTag::Equal => {}
            DiffTag::Delete => matches.extend(old_range.map(Match::Removed)),
            DiffTag::Insert => matches.extend(new_range.map(Match::Added)),
            DiffTag::Replace if old_range.len() == new_range.len() => {
                matches.extend(old_range.zip(new_range).map(|(i, j)| Match::Pair(i, j)));
            }
            DiffTag::Replace => {
                matches.extend(old_range.map(Match::Removed));
                matches.extend(new_range.map(Match::Added));
            }
        }
    }
    matches
}

/// Match the recipients of a block by address, custom key and custom value,
/// because a save sorts them and a recipient can change its place (ADR 0144
/// item 7). With as many other old as new recipients, the k-th old is the k-th
/// new, so a new address shows as a change.
fn match_recipients(olds: &[Obj], news: &[Obj]) -> Vec<Match> {
    let identity =
        |obj: &Obj| ["address", "custom_key", "custom_value"].map(|key| string_at(obj.map, key));
    let mut other_news: Vec<usize> = (0..news.len()).collect();
    let mut matches = Vec::new();
    let mut other_olds = Vec::new();
    for (i, old) in olds.iter().enumerate() {
        let found = other_news
            .iter()
            .position(|&j| identity(&news[j]) == identity(old));
        match found {
            Some(at) => matches.push(Match::Pair(i, other_news.remove(at))),
            None => other_olds.push(i),
        }
    }
    if other_olds.len() == other_news.len() {
        matches.extend(
            other_olds
                .into_iter()
                .zip(other_news)
                .map(|(i, j)| Match::Pair(i, j)),
        );
    } else {
        matches.extend(other_olds.into_iter().map(Match::Removed));
        matches.extend(other_news.into_iter().map(Match::Added));
    }
    matches
}

/// The role and the credit name that a change inherits from its entry.
#[derive(Debug, Clone, Copy, Default)]
struct Who<'a> {
    entry: Option<&'a str>,
    name: Option<&'a str>,
}

/// The name of a credit entry. Only a `persons` list has one.
fn credit_name(list_key: &str, map: &Map<String, Value>) -> Option<String> {
    (list_key == "persons")
        .then(|| string_at(map, "name"))
        .flatten()
}

/// The entry label of a list entry: the role of a credit or the name of a
/// recipient.
fn entry_label(list_key: &str, map: &Map<String, Value>) -> Option<String> {
    match list_key {
        "persons" => string_at(map, "role"),
        "recipients" => string_at(map, "name"),
        _ => None,
    }
}

/// The changes of one match of a list. `path` is the path of the list in
/// the group.
#[allow(clippy::too_many_arguments)]
fn match_changes(
    matched: &Match,
    sub: Table,
    olds: &[Obj],
    news: &[Obj],
    path: &str,
    list_key: &str,
    inherited: Who,
    out: &mut Vec<FieldChange>,
) {
    let label_of = |obj: &Obj| entry_label(list_key, obj.map);
    let name_of = |obj: &Obj| credit_name(list_key, obj.map);
    match *matched {
        Match::Pair(i, j) => {
            let label = label_of(&news[j]).or_else(|| label_of(&olds[i]));
            let name = name_of(&news[j]).or_else(|| name_of(&olds[i]));
            let who = Who {
                entry: label.as_deref().or(inherited.entry),
                name: name.as_deref().or(inherited.name),
            };
            walk_table(sub, Some(&olds[i]), Some(&news[j]), path, who, out);
        }
        Match::Added(j) => out.push(FieldChange {
            op: ChangeOp::Added,
            path: path.to_owned(),
            entry: label_of(&news[j]).or_else(|| inherited.entry.map(str::to_owned)),
            name: name_of(&news[j]).or_else(|| inherited.name.map(str::to_owned)),
            old: None,
            new: Some(news[j].whole()),
            old_at: None,
            new_at: news[j].cur.as_ref().map(|cur| cur.place(true)),
        }),
        Match::Removed(i) => out.push(FieldChange {
            op: ChangeOp::Removed,
            path: path.to_owned(),
            entry: label_of(&olds[i]).or_else(|| inherited.entry.map(str::to_owned)),
            name: name_of(&olds[i]).or_else(|| inherited.name.map(str::to_owned)),
            old: Some(olds[i].whole()),
            new: None,
            old_at: olds[i].cur.as_ref().map(|cur| cur.place(true)),
            new_at: None,
        }),
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

fn walk_table(
    table: Table,
    old: Option<&Obj>,
    new: Option<&Obj>,
    path: &str,
    who: Who,
    out: &mut Vec<FieldChange>,
) {
    for field in table {
        walk_field(field, old, new, &join(path, field.key), who, out);
    }
}

/// A value of an object, with a JSON null as no value.
fn value_at<'a>(obj: Option<&Obj<'a, '_>>, key: &str) -> Option<&'a Value> {
    obj.and_then(|o| o.map.get(key)).filter(|v| !v.is_null())
}

/// The changes of one field. `path` is the path of the field in the group.
fn walk_field(
    field: &Field,
    old: Option<&Obj>,
    new: Option<&Obj>,
    path: &str,
    who: Who,
    out: &mut Vec<FieldChange>,
) {
    let (old_value, new_value) = (value_at(old, field.key), value_at(new, field.key));
    if old_value == new_value {
        return;
    }
    let leaf = |old_at: Option<Place>, new_at: Option<Place>| FieldChange {
        op: ChangeOp::Changed,
        path: path.to_owned(),
        entry: who.entry.map(str::to_owned),
        name: who.name.map(str::to_owned),
        old: old_value.cloned(),
        new: new_value.cloned(),
        old_at,
        new_at,
    };
    let whole_of = |obj: Option<&Obj>| obj.and_then(|o| o.cur.as_ref()).map(|c| c.place(true));
    let start_of = |obj: Option<&Obj>| obj.and_then(|o| o.cur.as_ref()).map(|c| c.place(false));
    match &field.via {
        Via::Computed => {}
        Via::Attr => out.push(leaf(start_of(old), start_of(new))),
        Via::SelfText => out.push(leaf(whole_of(old), whole_of(new))),
        Via::Child(ns, local) => {
            let at =
                |obj: Option<&Obj>| obj.and_then(|o| o.child(*ns, local)).map(|c| c.place(true));
            out.push(leaf(at(old), at(new)));
        }
        Via::Object(ns, local, sub) => {
            let old_child = old.and_then(|o| o.child(*ns, local));
            let new_child = new.and_then(|o| o.child(*ns, local));
            let old_sub = old_value.and_then(Value::as_object).map(|map| Obj {
                map,
                cur: old_child,
            });
            let new_sub = new_value.and_then(Value::as_object).map(|map| Obj {
                map,
                cur: new_child,
            });
            walk_pair(sub, old_sub.as_ref(), new_sub.as_ref(), path, who, out);
        }
        Via::Within(sub) => {
            let old_sub = old_value.and_then(Value::as_object).map(|map| Obj {
                map,
                cur: old.and_then(|o| o.cur.clone()),
            });
            let new_sub = new_value.and_then(Value::as_object).map(|map| Obj {
                map,
                cur: new.and_then(|o| o.cur.clone()),
            });
            walk_pair(sub, old_sub.as_ref(), new_sub.as_ref(), path, who, out);
        }
        Via::List(ns, local, sub) => {
            let olds = entries(old, field.key, *ns, local);
            let news = entries(new, field.key, *ns, local);
            let matches = if field.key == "recipients" {
                match_recipients(&olds, &news)
            } else {
                match_entries(&olds, &news)
            };
            for matched in matches {
                match_changes(&matched, sub, &olds, &news, path, field.key, who, out);
            }
        }
    }
}

/// The changes of an object that can be on one side only.
fn walk_pair(
    sub: Table,
    old: Option<&Obj>,
    new: Option<&Obj>,
    path: &str,
    who: Who,
    out: &mut Vec<FieldChange>,
) {
    match (old, new) {
        (Some(_), Some(_)) => walk_table(sub, old, new, path, who, out),
        (None, Some(new)) => out.push(FieldChange {
            op: ChangeOp::Added,
            path: path.to_owned(),
            entry: who.entry.map(str::to_owned),
            name: who.name.map(str::to_owned),
            old: None,
            new: Some(new.whole()),
            old_at: None,
            new_at: new.cur.as_ref().map(|cur| cur.place(true)),
        }),
        (Some(old), None) => out.push(FieldChange {
            op: ChangeOp::Removed,
            path: path.to_owned(),
            entry: who.entry.map(str::to_owned),
            name: who.name.map(str::to_owned),
            old: Some(old.whole()),
            new: None,
            old_at: old.cur.as_ref().map(|cur| cur.place(true)),
            new_at: None,
        }),
        (None, None) => {}
    }
}

// ---------------------------------------------------------------------
// The lines
// ---------------------------------------------------------------------

/// Each line of the two files, with the marked parts.
fn diff_lines(base: &str, draft: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(base, draft);
    let mut options = InlineChangeOptions::new();
    options.mode(InlineChangeMode::Chars).semantic_cleanup(true);
    let mut lines: Vec<DiffLine> = Vec::new();
    for op in diff.ops() {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if tag == DiffTag::Equal {
            for (i, j) in old_range.zip(new_range) {
                let text = diff.old_slice(i).unwrap_or_default();
                lines.push(DiffLine {
                    kind: LineKind::Same,
                    old: Some(i + 1),
                    new: Some(j + 1),
                    parts: make_parts(std::iter::once((false, text))),
                    pair: None,
                    covered: false,
                });
            }
            continue;
        }
        let mut removed = Vec::new();
        let mut added = Vec::new();
        for change in diff.iter_inline_changes_with_options(op, options) {
            let parts = make_parts(change.values().iter().copied());
            match change.tag() {
                ChangeTag::Delete => removed.push((change.old_index().unwrap_or(0), parts)),
                ChangeTag::Insert => added.push((change.new_index().unwrap_or(0), parts)),
                ChangeTag::Equal => {}
            }
        }
        removed.sort_by_key(|(index, _)| *index);
        added.sort_by_key(|(index, _)| *index);
        let paired = !removed.is_empty() && removed.len() == added.len();
        let line = |kind, index: usize, parts| DiffLine {
            kind,
            old: (kind == LineKind::Removed).then_some(index + 1),
            new: (kind == LineKind::Added).then_some(index + 1),
            parts,
            pair: None,
            covered: false,
        };
        if paired {
            for ((old_index, old_parts), (new_index, new_parts)) in removed.into_iter().zip(added) {
                let at = lines.len();
                let mut first = line(LineKind::Removed, old_index, old_parts);
                first.pair = Some(at + 1);
                lines.push(first);
                lines.push(line(LineKind::Added, new_index, new_parts));
            }
        } else {
            lines.extend(
                removed
                    .into_iter()
                    .map(|(index, parts)| line(LineKind::Removed, index, parts)),
            );
            lines.extend(
                added
                    .into_iter()
                    .map(|(index, parts)| line(LineKind::Added, index, parts)),
            );
        }
    }
    mark_whole_lines(&mut lines);
    lines
}

/// Make the parts of a line. A part has no line end, and two neighbors
/// never have the same mark.
fn make_parts<'s>(values: impl Iterator<Item = (bool, &'s str)>) -> Vec<LinePart> {
    let mut parts: Vec<LinePart> = Vec::new();
    for (changed, text) in values {
        if text.is_empty() {
            continue;
        }
        match parts.last_mut() {
            Some(last) if last.changed == changed => last.text.push_str(text),
            _ => parts.push(LinePart {
                text: text.to_owned(),
                changed,
            }),
        }
    }
    while let Some(last) = parts.last_mut() {
        let kept = last.text.trim_end_matches(['\n', '\r']).len();
        last.text.truncate(kept);
        if last.text.is_empty() {
            parts.pop();
        } else {
            break;
        }
    }
    parts
}

fn has_changed_part(line: &DiffLine) -> bool {
    line.parts.iter().any(|part| part.changed)
}

/// A removed or added line with no changed part is a whole changed line,
/// unless it is in a pair and only its partner has changed parts. In that
/// pair, the line with no changed part gets only an addition or only a
/// removal on the other line.
fn mark_whole_lines(lines: &mut [DiffLine]) {
    let mut whole = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.kind == LineKind::Same || has_changed_part(line) {
            continue;
        }
        match line.pair {
            Some(partner) if has_changed_part(&lines[partner]) => {}
            Some(partner) => whole.extend([index, partner]),
            None if index > 0 && lines[index - 1].pair == Some(index) => {
                if !has_changed_part(&lines[index - 1]) {
                    whole.push(index);
                }
            }
            None => whole.push(index),
        }
    }
    for index in whole {
        let line = &mut lines[index];
        for part in &mut line.parts {
            part.changed = true;
        }
    }
}

// ---------------------------------------------------------------------
// The cover
// ---------------------------------------------------------------------

/// The byte offset where each line of `text` starts.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        starts.push(offset);
        offset += line.len();
    }
    starts
}

/// The lines that touch the byte range `start..end`, as indexes.
fn touched(starts: &[usize], start: usize, end: usize) -> Option<RangeInclusive<usize>> {
    if starts.is_empty() || end <= start {
        return None;
    }
    let first = starts.partition_point(|&s| s <= start).saturating_sub(1);
    let last = starts.partition_point(|&s| s < end).checked_sub(1)?;
    (first <= last).then_some(first..=last)
}

/// The lines of one file that the field changes explain, as line numbers
/// from 1.
fn covered_lines<'c>(
    text: &str,
    model: Option<&Model>,
    places: impl Iterator<Item = &'c Place>,
) -> HashSet<usize> {
    let mut covered = HashSet::new();
    let Some(doc) = model.and_then(|m| m.doc.as_ref()) else {
        return covered;
    };
    let (written, spans) = doc.write_with_spans();
    // When the written text is not the file, a byte range does not name a
    // line of the file. Then no line is covered.
    if written != text {
        return covered;
    }
    let by_path: HashMap<&[usize], &ElementSpan> =
        spans.iter().map(|s| (s.path.as_slice(), s)).collect();
    let starts = line_starts(text);
    for place in places {
        let Some(span) = by_path.get(place.path.as_slice()) else {
            continue;
        };
        let end = if place.whole { span.end } else { span.tag_end };
        if let Some(range) = touched(&starts, span.start, end) {
            covered.extend(range.map(|index| index + 1));
        }
    }
    covered
}

fn cover(
    lines: &mut [DiffLine],
    fields: &[FieldGroup],
    old: (&str, Option<&Model>),
    new: (&str, Option<&Model>),
) {
    let changes = || fields.iter().flat_map(|group| &group.changes);
    let old_covered = covered_lines(old.0, old.1, changes().filter_map(|c| c.old_at.as_ref()));
    let new_covered = covered_lines(new.0, new.1, changes().filter_map(|c| c.new_at.as_ref()));
    // With no live file, each line of the draft is new, and the field
    // changes give each field as added. Then each line is covered.
    let new_feed = old.0.is_empty();
    for line in lines {
        line.covered = match (line.kind, line.old, line.new) {
            (LineKind::Added, _, _) if new_feed => true,
            (LineKind::Removed, Some(number), _) => old_covered.contains(&number),
            (LineKind::Added, _, Some(number)) => new_covered.contains(&number),
            _ => false,
        };
    }
}

/// True for a changed line that no field change explains.
fn is_uncovered(line: &DiffLine) -> bool {
    line.kind != LineKind::Same && !line.covered
}

/// The number of runs of adjacent changed lines that no field change
/// explains. Each such line is in one run (ADR 0114 item 5).
fn count_uncovered(lines: &[DiffLine]) -> usize {
    lines
        .chunk_by(|a, b| is_uncovered(a) == is_uncovered(b))
        .filter(|run| is_uncovered(&run[0]))
        .count()
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    const HEAD: &str = "<?xml version=\"1.0\"?>\n\
        <rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">\n\
        <channel>\n";

    fn feed(channel: &str) -> String {
        format!("{HEAD}{channel}</channel>\n</rss>\n")
    }

    fn six_credits(npub: bool) -> String {
        let roles = [
            "musician",
            "songwriter",
            "composer",
            "guitarist",
            "keyboardist",
            "producer",
        ];
        let mut channel = String::from("<title>Album</title>\n");
        for role in roles {
            let key = if npub { " npub=\"npub1abc\"" } else { "" };
            writeln!(
                channel,
                "<podcast:person role=\"{role}\"{key}>HeyCitizen</podcast:person>"
            )
            .unwrap();
        }
        feed(&channel)
    }

    fn run(base: &str, draft: &str) -> Review {
        review(base.as_bytes(), draft.as_bytes())
    }

    #[test]
    fn six_credits_of_one_name_give_one_group_and_no_guard_row() {
        let review = run(&six_credits(false), &six_credits(true));
        assert_eq!(review.fields.len(), 1, "{:?}", review.fields);
        let group = &review.fields[0];
        assert_eq!(group.kind, GroupKind::Credits);
        assert_eq!(group.name.as_deref(), Some("HeyCitizen"));
        assert_eq!(group.changes.len(), 6);
        for change in &group.changes {
            assert_eq!(change.op, ChangeOp::Changed);
            assert_eq!(change.path, "npub");
            assert_eq!(change.old, None);
            assert_eq!(change.new, Some(Value::from("npub1abc")));
        }
        assert_eq!(group.changes[0].entry.as_deref(), Some("musician"));
        assert_eq!(review.uncovered, 0);
        let changed: Vec<_> = review
            .lines
            .iter()
            .filter(|l| l.kind != LineKind::Same)
            .collect();
        assert_eq!(changed.len(), 12);
        assert!(changed.iter().all(|l| l.covered));
    }

    #[test]
    fn the_parts_of_a_credit_line_mark_only_the_added_attribute() {
        let review = run(&six_credits(false), &six_credits(true));
        for line in review.lines.iter().filter(|l| l.kind == LineKind::Added) {
            let marked: Vec<_> = line.parts.iter().filter(|p| p.changed).collect();
            assert_eq!(marked.len(), 1, "{line:?}");
            assert_eq!(marked[0].text.trim(), "npub=\"npub1abc\"");
        }
        for line in review.lines.iter().filter(|l| l.kind == LineKind::Removed) {
            assert!(line.parts.iter().all(|p| !p.changed), "{line:?}");
        }
        let joined: String = review
            .lines
            .iter()
            .find(|l| l.kind == LineKind::Added)
            .unwrap()
            .parts
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        assert!(!joined.contains('\n'));
    }

    #[test]
    fn a_removed_line_of_a_pair_comes_before_its_added_line() {
        let review = run(&six_credits(false), &six_credits(true));
        let mut pairs = 0;
        for (index, line) in review.lines.iter().enumerate() {
            if line.kind == LineKind::Removed {
                assert_eq!(line.pair, Some(index + 1));
                assert_eq!(review.lines[index + 1].kind, LineKind::Added);
                pairs += 1;
            }
        }
        assert_eq!(pairs, 6);
    }

    #[test]
    fn a_changed_title_marks_only_the_changed_words() {
        let base = feed("<title>Blue Sky Morning</title>\n");
        let draft = feed("<title>Blue Rain Morning</title>\n");
        let review = run(&base, &draft);
        let added = review
            .lines
            .iter()
            .find(|l| l.kind == LineKind::Added)
            .unwrap();
        let marked: Vec<_> = added.parts.iter().filter(|p| p.changed).collect();
        assert_eq!(marked.len(), 1, "{added:?}");
        assert_eq!(marked[0].text, "Rain");
    }

    #[test]
    fn a_namespace_declaration_is_a_change_that_the_fields_do_not_explain() {
        let base = feed("<title>Old</title>\n");
        let draft = feed("<title>New</title>\n").replace(
            "<rss version=\"2.0\"",
            "<rss version=\"2.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"",
        );
        let review = run(&base, &draft);
        assert_eq!(review.fields.len(), 1);
        assert_eq!(review.fields[0].kind, GroupKind::Field);
        assert_eq!(review.fields[0].name.as_deref(), Some("title"));
        assert_eq!(review.fields[0].changes[0].path, "");
        assert_eq!(review.uncovered, 1);
    }

    fn track_with_credits(npub: bool) -> String {
        let key = if npub { " npub=\"npub1abc\"" } else { "" };
        feed(&format!(
            "<item>\n<title>Song</title>\n<guid>a</guid>\n\
             <podcast:person role=\"vocalist\"{key}>Ann</podcast:person>\n\
             <podcast:person role=\"drummer\"{key}>Bob</podcast:person>\n</item>\n"
        ))
    }

    #[test]
    fn a_track_credit_change_has_the_role_and_the_name_of_the_credit() {
        let review = run(&track_with_credits(false), &track_with_credits(true));
        assert_eq!(review.fields.len(), 1, "{:?}", review.fields);
        let group = &review.fields[0];
        assert_eq!(group.kind, GroupKind::Track);
        assert_eq!(group.changes.len(), 2);
        assert_eq!(group.changes[0].entry.as_deref(), Some("vocalist"));
        assert_eq!(group.changes[0].name.as_deref(), Some("Ann"));
        assert_eq!(group.changes[1].entry.as_deref(), Some("drummer"));
        assert_eq!(group.changes[1].name.as_deref(), Some("Bob"));
    }

    #[test]
    fn a_channel_credit_change_has_a_name_and_a_recipient_change_has_none() {
        let credits = run(&six_credits(false), &six_credits(true));
        assert!(
            credits.fields[0]
                .changes
                .iter()
                .all(|c| c.name.as_deref() == Some("HeyCitizen"))
        );
        let value = |address: &str| {
            feed(&format!(
                "<podcast:value type=\"lightning\" method=\"keysend\">\n\
                 <podcast:valueRecipient name=\"Ann\" type=\"node\" address=\"{address}\" split=\"100\"/>\n\
                 </podcast:value>\n"
            ))
        };
        let review = run(&value("aaa"), &value("bbb"));
        let changes: Vec<_> = review.fields.iter().flat_map(|g| &g.changes).collect();
        assert!(!changes.is_empty(), "{:?}", review.fields);
        assert!(changes.iter().all(|c| c.name.is_none()), "{changes:?}");
        assert_eq!(changes[0].entry.as_deref(), Some("Ann"));
    }

    #[test]
    fn a_recipient_at_a_new_place_shows_only_the_changes_of_its_values() {
        let value = |rows: [(&str, &str); 2]| {
            let [first, second] = rows.map(|(name, split)| {
                format!(
                    "<podcast:valueRecipient name=\"{name}\" type=\"node\" address=\"{name}\" split=\"{split}\"/>\n"
                )
            });
            feed(&format!(
                "<podcast:value type=\"lightning\" method=\"keysend\">\n{first}{second}</podcast:value>\n"
            ))
        };
        let review = run(
            &value([("a", "10"), ("b", "10")]),
            &value([("b", "99"), ("a", "10")]),
        );
        let changes: Vec<_> = review.fields.iter().flat_map(|g| &g.changes).collect();
        assert_eq!(changes.len(), 1, "ADR 0144 item 7: {changes:?}");
        assert_eq!(changes[0].op, ChangeOp::Changed);
        assert_eq!(changes[0].entry.as_deref(), Some("b"));
        assert_eq!(changes[0].new.as_ref().and_then(Value::as_str), Some("99"));
    }

    fn two_items(order: [&str; 2], second_title: &str) -> String {
        let item = |guid: &str, title: &str| {
            format!("<item>\n<title>{title}</title>\n<guid>{guid}</guid>\n</item>\n")
        };
        let titles = |guid: &str| {
            if guid == "a" {
                "Alpha".to_owned()
            } else {
                second_title.to_owned()
            }
        };
        feed(&format!(
            "{}{}",
            item(order[0], &titles(order[0])),
            item(order[1], &titles(order[1]))
        ))
    }

    #[test]
    fn items_match_by_guid_and_ignore_their_sequence() {
        let review = run(
            &two_items(["a", "b"], "Beta"),
            &two_items(["b", "a"], "Beta"),
        );
        assert!(review.fields.is_empty(), "{:?}", review.fields);
    }

    #[test]
    fn a_removed_item_is_one_removed_change_in_its_track_group() {
        let base = two_items(["a", "b"], "Beta");
        let draft = feed("<item>\n<title>Alpha</title>\n<guid>a</guid>\n</item>\n");
        let review = run(&base, &draft);
        assert_eq!(review.fields.len(), 1, "{:?}", review.fields);
        let group = &review.fields[0];
        assert_eq!(group.kind, GroupKind::Track);
        assert_eq!(group.position, Some(2));
        assert_eq!(group.name.as_deref(), Some("Beta"));
        assert_eq!(group.changes.len(), 1);
        assert_eq!(group.changes[0].op, ChangeOp::Removed);
        assert_eq!(group.changes[0].path, "");
        assert_eq!(review.uncovered, 0);
    }

    #[test]
    fn a_new_feed_adds_each_field_of_the_draft() {
        let review = review(b"", feed("<title>New</title>\n").as_bytes());
        assert_eq!(review.fields.len(), 1);
        assert_eq!(review.fields[0].name.as_deref(), Some("title"));
        assert_eq!(review.fields[0].changes[0].old, None);
        assert!(review.lines.iter().all(|l| l.kind == LineKind::Added));
        // The added lines are one run, and the title line is in it.
        assert_eq!(review.uncovered, 0);
    }

    #[test]
    fn a_line_next_to_a_covered_line_is_an_uncovered_change() {
        // The title changes, and the line after it gets a comment that
        // the model does not hold. The two changed lines are adjacent.
        let review = run(
            &feed("<title>Old</title>\n<link>https://a.example/</link>\n"),
            &feed("<title>New</title>\n<link>https://a.example/</link><!-- x -->\n"),
        );
        assert_eq!(review.fields.len(), 1);
        assert_eq!(review.uncovered, 1);
    }

    #[test]
    fn a_file_that_does_not_parse_gives_no_field_and_one_uncovered_change() {
        let review = run("<rss><channel>", &six_credits(true));
        assert_eq!(review.fields, Vec::new());
        assert!(review.uncovered > 0);
    }

    /// A feed with each element and each attribute that the model reads.
    const FULL: &str = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
<title>t</title><description>d</description><link>l</link><language>en</language>
<managingEditor>m</managingEditor><webMaster>w</webMaster>
<image><url>u</url><title>t</title><link>l</link><description>d</description></image>
<podcast:guid>g</podcast:guid><podcast:medium>music</podcast:medium>
<podcast:locked owner="o">yes</podcast:locked>
<itunes:author>a</itunes:author><itunes:explicit>false</itunes:explicit>
<itunes:owner><itunes:name>n</itunes:name><itunes:email>e</itunes:email></itunes:owner>
<itunes:keywords>k</itunes:keywords><itunes:image href="h"/>
<itunes:new-feed-url>x</itunes:new-feed-url><itunes:block>no</itunes:block>
<podcast:block id="i">no</podcast:block>
<atom:link href="h" rel="self" type="application/rss+xml"/>
<podcast:image href="h" alt="a" aspect-ratio="1/1" width="1" height="1" type="image/png" purpose="artwork"/>
<podcast:person role="r" group="g" img="i" href="h" npub="n">N</podcast:person>
<podcast:funding url="u">F</podcast:funding>
<podcast:txt purpose="p">T</podcast:txt>
<podcast:license url="u">MIT</podcast:license>
<podcast:publisher><podcast:remoteItem feedGuid="g" feedUrl="u" itemGuid="i" medium="m" title="t" rel="r"/></podcast:publisher>
<podcast:podroll><podcast:remoteItem feedGuid="g"/></podcast:podroll>
<podcast:remoteItem feedGuid="g"/>
<podcast:value type="lightning" method="keysend" suggested="0.1">
<podcast:valueRecipient name="n" type="node" address="a" split="1" customKey="k" customValue="v" fee="false"/>
<podcast:valueTimeSplit startTime="1" duration="2" remoteStartTime="3" remotePercentage="4">
<podcast:remoteItem feedGuid="g"/><podcast:valueRecipient name="n"/>
</podcast:valueTimeSplit>
</podcast:value>
<item>
@@ITEM@@</item>
<podcast:liveItem status="live" start="s" end="e">
@@ITEM@@<podcast:liveValue uri="u" protocol="p"/>
<podcast:contentLink href="h">C</podcast:contentLink>
</podcast:liveItem>
</channel>
</rss>
"#;

    /// The elements of an item. A live item has the same elements.
    const ITEM_BODY: &str = r#"<title>t</title><description>d</description><guid>g</guid>
<enclosure url="u" length="1" type="audio/mpeg"/>
<podcast:alternateEnclosure type="audio/flac" length="1" bitrate="1" title="t" rel="r" default="true"><podcast:source uri="u" contentType="c"/></podcast:alternateEnclosure>
<pubDate>Fri, 6 Feb 2026 09:54:10 +0000</pubDate><itunes:duration>60</itunes:duration>
<itunes:explicit>no</itunes:explicit><podcast:season name="s">1</podcast:season>
<podcast:episode>1</podcast:episode><itunes:episode>1</itunes:episode><itunes:image href="h"/>
<podcast:image href="h"/><podcast:person>P</podcast:person><podcast:funding url="u">F</podcast:funding>
<podcast:transcript url="u" type="t" language="en" rel="r"/><podcast:txt>T</podcast:txt>
<podcast:license>L</podcast:license>
<podcast:value type="lightning"><podcast:valueRecipient name="n"/><podcast:valueTimeSplit startTime="1"><podcast:remoteItem feedGuid="g"/><podcast:valueRecipient name="n"/></podcast:valueTimeSplit></podcast:value>
"#;

    fn full() -> String {
        FULL.replace("@@ITEM@@", ITEM_BODY)
    }

    /// Fail when a key of the model is not in the table, or the other way.
    fn assert_table(table: Table, value: &Value, context: &str) {
        let Some(map) = value.as_object() else {
            panic!("{context}: the model value is not an object");
        };
        for key in map.keys() {
            assert!(
                table.iter().any(|field| field.key == key),
                "ADR 0114 item 5: the model key {context}{key} has no element in the table of \
                 review.rs. Add the element that Feed::read reads it from."
            );
        }
        for field in table {
            let Some(inner) = map.get(field.key) else {
                panic!(
                    "ADR 0114 item 5: the table of review.rs has the key {context}{} and the \
                     model has not. Remove the key from the table.",
                    field.key
                );
            };
            let context = format!("{context}{}.", field.key);
            match &field.via {
                Via::Object(_, _, sub) | Via::Within(sub) => {
                    assert_table(sub, inner, &context);
                }
                Via::List(_, _, sub) => {
                    let entries = inner.as_array().expect("a list is an array");
                    assert!(
                        !entries.is_empty(),
                        "the sample feed has no entry in {context}"
                    );
                    for entry in entries {
                        assert_table(sub, entry, &context);
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn the_table_has_each_key_of_a_model_with_each_field_set() {
        let doc = Document::parse(&full()).unwrap();
        let feed = Feed::read(&doc).unwrap();
        let json = model_json(&feed).unwrap();
        assert_table(FEED, &json, "");
    }

    #[test]
    fn a_change_of_each_leaf_of_the_sample_has_a_place_in_both_files() {
        // The sample feed with a changed title in the item and the live
        // item covers the lines of those elements.
        let base = full();
        let draft = base
            .replacen(
                "<title>t</title><description>d</description><guid>g</guid>",
                "<title>T2</title><description>d</description><guid>g</guid>",
                1,
            )
            .replace(
                "<podcast:liveItem status=\"live\"",
                "<podcast:liveItem status=\"ended\"",
            );
        let review = run(&base, &draft);
        assert_eq!(review.uncovered, 0, "{:?}", review.fields);
        let kinds: Vec<_> = review.fields.iter().map(|g| g.kind).collect();
        assert_eq!(kinds, [GroupKind::Track, GroupKind::Live]);
        assert_eq!(review.fields[1].changes[0].path, "status");
    }
}
