//! Bring the hosted file into a draft (ADR 0129).
//!
//! The merge compares three models: the base of the draft, the draft and the
//! hosted file. It works on the serialized model, so that each field has one
//! rule. A track is matched by its GUID, and each of its fields merges alone.
//! Each other list is one value.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::feed::Feed;

/// The side that the user selects for a field that both sides changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// The edit in the draft.
    Mine,
    /// The value in the hosted file.
    Hosted,
}

/// One field that differs between the draft and the hosted file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MergeField {
    /// The ID that a choice names.
    pub id: String,
    /// The title of the track, for a field of a track.
    pub track: Option<String>,
    /// The key of the model. An empty key is a whole track.
    pub key: String,
    /// The value in the draft.
    pub mine: Option<Value>,
    /// The value in the hosted file.
    pub hosted: Option<Value>,
}

/// What a merge does.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MergePlan {
    /// The edits of the draft that the new draft keeps.
    pub kept: Vec<MergeField>,
    /// The changes of the hosted file that the new draft gets.
    pub incoming: Vec<MergeField>,
    /// The fields that both sides changed.
    pub conflicts: Vec<MergeField>,
}

/// The result of a merge: the plan and the merged model. The entries that come
/// from the hosted file keep their keys, so the writer keeps their bytes.
#[derive(Debug)]
pub struct Merged {
    /// What the merge does.
    pub plan: MergePlan,
    /// The merged model. A conflict with no choice keeps the hosted value.
    pub model: Value,
}

fn stripped(value: &Value) -> Value {
    let mut value = value.clone();
    crate::review::strip_keys(&mut value);
    value
}

/// The value of a key, with no entry key. A null is the same as no value.
fn plain(map: &Map<String, Value>, key: &str) -> Option<Value> {
    map.get(key).filter(|v| !v.is_null()).map(stripped)
}

/// A value of the draft for the merged model. An entry of a list that is the
/// same in the hosted file keeps its key.
fn from_mine(mine: &Value, hosted: Option<&Value>) -> Value {
    let (Value::Array(entries), Some(Value::Array(others))) = (mine, hosted) else {
        return stripped(mine);
    };
    let mut free: Vec<&Value> = others.iter().collect();
    Value::Array(
        entries
            .iter()
            .map(|entry| {
                let plain_entry = stripped(entry);
                match free.iter().position(|o| stripped(o) == plain_entry) {
                    Some(index) => free.remove(index).clone(),
                    None => plain_entry,
                }
            })
            .collect(),
    )
}

struct Merger<'c> {
    choices: &'c BTreeMap<String, Side>,
    plan: MergePlan,
}

impl Merger<'_> {
    /// Merge the fields of one object. `prefix` starts the ID of a field.
    fn fields(
        &mut self,
        base: &Map<String, Value>,
        mine: &Map<String, Value>,
        hosted: &Map<String, Value>,
        prefix: &str,
        track: Option<&str>,
        skip: &[&str],
    ) -> Map<String, Value> {
        let mut out = hosted.clone();
        let keys: BTreeSet<&String> = base
            .keys()
            .chain(mine.keys())
            .chain(hosted.keys())
            .collect();
        for key in keys {
            if key == "key" || skip.contains(&key.as_str()) {
                continue;
            }
            let (b, m, h) = (plain(base, key), plain(mine, key), plain(hosted, key));
            if m == h {
                continue;
            }
            let field = MergeField {
                id: format!("{prefix}{key}"),
                track: track.map(str::to_owned),
                key: key.clone(),
                mine: m.clone(),
                hosted: h.clone(),
            };
            let take_mine = if m == b {
                self.plan.incoming.push(field);
                false
            } else if h == b {
                self.plan.kept.push(field);
                true
            } else {
                let side = self.choices.get(&field.id).copied();
                self.plan.conflicts.push(field);
                side == Some(Side::Mine)
            };
            if take_mine {
                match mine.get(key) {
                    Some(value) => {
                        out.insert(key.clone(), from_mine(value, hosted.get(key)));
                    }
                    None => {
                        out.remove(key);
                    }
                }
            }
        }
        out
    }

    /// One track that one side has with no match, or a track with a changed
    /// whole value. Gives the entry of the merged model, or none.
    fn whole_track(
        &mut self,
        guid: &str,
        base: Option<&Value>,
        mine: Option<&Value>,
        hosted: Option<&Value>,
    ) -> Option<Value> {
        let plain_of = |v: Option<&Value>| v.map(stripped);
        let (b, m, h) = (plain_of(base), plain_of(mine), plain_of(hosted));
        let title = mine.or(hosted).or(base).and_then(|v| v["title"].as_str());
        let field = MergeField {
            id: format!("items/{guid}"),
            track: title.map(str::to_owned),
            key: String::new(),
            mine: m.clone(),
            hosted: h.clone(),
        };
        let take_mine = if m == h {
            false
        } else if m == b {
            self.plan.incoming.push(field);
            false
        } else if h == b {
            self.plan.kept.push(field);
            true
        } else {
            let side = self.choices.get(&field.id).copied();
            self.plan.conflicts.push(field);
            side == Some(Side::Mine)
        };
        if take_mine { m } else { hosted.cloned() }
    }

    fn tracks(&mut self, base: &[Value], mine: &[Value], hosted: &[Value]) -> Vec<Value> {
        let guid_of = |v: &Value| v["guid"].as_str().unwrap_or_default().to_owned();
        let find = |list: &'_ [Value], guid: &str| {
            list.iter().position(|v| v["guid"].as_str() == Some(guid))
        };
        let guids = |list: &[Value]| list.iter().map(guid_of).collect::<Vec<_>>();
        // The sequence of the draft, when the user changed it. Else the
        // sequence of the hosted file. The tracks of the other side follow.
        let mut order = if guids(mine) == guids(base) {
            guids(hosted)
        } else {
            guids(mine)
        };
        for guid in guids(hosted).into_iter().chain(guids(mine)) {
            if !order.contains(&guid) {
                order.push(guid);
            }
        }
        for guid in guids(base) {
            if !order.contains(&guid) {
                order.push(guid);
            }
        }
        let mut out = Vec::new();
        for guid in order {
            let b = find(base, &guid).map(|i| &base[i]);
            let m = find(mine, &guid).map(|i| &mine[i]);
            let h = find(hosted, &guid).map(|i| &hosted[i]);
            let entry = match (b, m, h) {
                (Some(Value::Object(b)), Some(Value::Object(m)), Some(Value::Object(h))) => {
                    let title = h.get("title").and_then(Value::as_str).map(str::to_owned);
                    let prefix = format!("items/{guid}/");
                    Some(Value::Object(self.fields(
                        b,
                        m,
                        h,
                        &prefix,
                        title.as_deref(),
                        &[],
                    )))
                }
                _ => self.whole_track(&guid, b, m, h),
            };
            out.extend(entry);
        }
        out
    }
}

/// True when each track of the list has a GUID, with no second use of it.
fn has_track_identity(list: &[Value]) -> bool {
    let mut seen = BTreeSet::new();
    list.iter()
        .all(|v| v["guid"].as_str().is_some_and(|guid| seen.insert(guid)))
}

/// Merge the draft `mine` and the hosted file `hosted` from their base.
///
/// # Errors
///
/// Gives a text when a model does not serialize as an object.
pub fn merge(
    base: &Feed,
    mine: &Feed,
    hosted: &Feed,
    choices: &BTreeMap<String, Side>,
) -> Result<Merged, String> {
    let object = |feed: &Feed| match serde_json::to_value(feed) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("the model is not an object".to_owned()),
        Err(error) => Err(error.to_string()),
    };
    let (b, m, h) = (object(base)?, object(mine)?, object(hosted)?);
    let list = |map: &Map<String, Value>| map["items"].as_array().cloned().unwrap_or_default();
    let (bi, mi, hi) = (list(&b), list(&m), list(&h));
    let by_track = has_track_identity(&bi) && has_track_identity(&mi) && has_track_identity(&hi);
    let mut merger = Merger {
        choices,
        plan: MergePlan::default(),
    };
    // Studio sets the build date at each write (ADR 0127).
    let skip: &[&str] = if by_track {
        &["last_build_date", "items"]
    } else {
        &["last_build_date"]
    };
    let mut model = merger.fields(&b, &m, &h, "", None, skip);
    if by_track {
        model.insert(
            "items".to_owned(),
            Value::Array(merger.tracks(&bi, &mi, &hi)),
        );
    }
    Ok(Merged {
        plan: merger.plan,
        model: Value::Object(model),
    })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;
    use crate::xml::Document;

    fn feed(title: &str, description: &str, tracks: &[(&str, &str)]) -> Feed {
        let mut items = String::new();
        for (guid, title) in tracks {
            write!(
                items,
                "<item><title>{title}</title><guid>{guid}</guid></item>"
            )
            .unwrap();
        }
        let xml = format!(
            "<rss version=\"2.0\"><channel><title>{title}</title><description>{description}</description>{items}</channel></rss>"
        );
        Feed::read(&Document::parse(&xml).unwrap()).unwrap()
    }

    fn ids(fields: &[MergeField]) -> Vec<&str> {
        fields.iter().map(|f| f.id.as_str()).collect()
    }

    #[test]
    fn an_edit_of_a_different_field_stays_and_the_hosted_change_comes_in() {
        let base = feed("T", "D", &[("a", "One")]);
        let mine = feed("Mine", "D", &[("a", "One")]);
        let hosted = feed("T", "Outside", &[("a", "One")]);
        let merged = merge(&base, &mine, &hosted, &BTreeMap::new()).unwrap();
        assert_eq!(ids(&merged.plan.kept), ["title"]);
        assert_eq!(ids(&merged.plan.incoming), ["description"]);
        assert_eq!(ids(&merged.plan.conflicts), [""; 0]);
        assert_eq!(merged.model["title"], "Mine");
        assert_eq!(merged.model["description"], "Outside");
    }

    #[test]
    fn a_field_that_both_sides_changed_takes_the_selected_side() {
        let base = feed("T", "D", &[]);
        let mine = feed("Mine", "D", &[]);
        let hosted = feed("Theirs", "D", &[]);
        let none = merge(&base, &mine, &hosted, &BTreeMap::new()).unwrap();
        assert_eq!(ids(&none.plan.conflicts), ["title"]);
        for (side, title) in [(Side::Mine, "Mine"), (Side::Hosted, "Theirs")] {
            let choices = BTreeMap::from([("title".to_owned(), side)]);
            let merged = merge(&base, &mine, &hosted, &choices).unwrap();
            assert_eq!(merged.model["title"], title);
        }
    }

    #[test]
    fn the_same_change_on_both_sides_is_no_conflict() {
        let base = feed("T", "D", &[]);
        let same = feed("New", "D", &[]);
        let merged = merge(&base, &same, &same, &BTreeMap::new()).unwrap();
        assert_eq!(merged.plan, MergePlan::default());
    }

    #[test]
    fn tracks_merge_by_guid_for_each_field_and_each_side_can_add_or_remove() {
        let base = feed("T", "D", &[("a", "One"), ("b", "Two"), ("c", "Three")]);
        // The draft edits a, removes b and adds d. The hosted file adds e.
        let mine = feed("T", "D", &[("a", "Mine"), ("c", "Three"), ("d", "New")]);
        let hosted = feed(
            "T",
            "D",
            &[("a", "One"), ("b", "Two"), ("c", "Three"), ("e", "Theirs")],
        );
        let merged = merge(&base, &mine, &hosted, &BTreeMap::new()).unwrap();
        assert_eq!(ids(&merged.plan.conflicts), [""; 0]);
        let titles: Vec<&str> = merged.model["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["title"].as_str().unwrap())
            .collect();
        assert_eq!(titles, ["Mine", "Three", "New", "Theirs"]);
    }

    #[test]
    fn a_track_that_one_side_removed_and_the_other_changed_is_a_conflict() {
        let base = feed("T", "D", &[("a", "One")]);
        let mine = feed("T", "D", &[]);
        let hosted = feed("T", "D", &[("a", "Changed")]);
        let merged = merge(&base, &mine, &hosted, &BTreeMap::new()).unwrap();
        assert_eq!(ids(&merged.plan.conflicts), ["items/a"]);
        assert_eq!(merged.model["items"].as_array().unwrap().len(), 1);
        let choices = BTreeMap::from([("items/a".to_owned(), Side::Mine)]);
        let merged = merge(&base, &mine, &hosted, &choices).unwrap();
        assert_eq!(merged.model["items"].as_array().unwrap().len(), 0);
    }
}
