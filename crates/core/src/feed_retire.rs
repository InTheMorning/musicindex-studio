//! The retire of a feed and the state of its three steps (ADR 0131 items 5
//! to 9 and 13).
//!
//! A retire sets "Hide from apps" in the draft (ADR 0063). The user reviews
//! and writes it as a usual write. The state of the three steps comes from
//! facts. The read of the index goes out only when the caller runs
//! [`read_removal`]. Studio cannot check Podcast Index, so the state has no
//! field for it (ADR 0131 item 7).

use crate::collection::{Collection, FeedId};
use crate::feed::Feed;
use crate::feed_check::same_but_spelling;
use crate::feed_reads::IndexReads;
use crate::library::Library;
use crate::media::plain_address;
use crate::musicindex::IndexPresence;
use crate::outside::{self, LastWrittenMove};
use crate::pipeline::{self, PipelineError, PipelineErrorKind};
use crate::publisher::{self, PublisherError};

/// Step 1 of ADR 0131 item 6: the feed asks to be removed. The answer comes
/// from the draft and from the last written file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetireAsk {
    /// Neither the draft nor the last written file hides the feed.
    No,
    /// Only the draft hides the feed. The user did not write it yet.
    DraftOnly,
    /// The last written file hides the feed, and the draft keeps the switch
    /// or has no draft.
    Written,
    /// The last written file hides the feed, but the draft clears the
    /// switch. The user did not write the clear yet (ADR 0131 item 8).
    WrittenCleared,
}

/// Step 2 of ADR 0131 item 6: musicindex removed the entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "answer")]
pub enum IndexRemoval {
    /// The read gave a 404. musicindex holds no record of the feed.
    Removed,
    /// The read gave a record. musicindex holds the entry.
    NotYet,
    /// The read gave a record at an address that is not the address of this
    /// file, and Studio did not move the feed from that address (ADR 0228
    /// items 20 and 21). This file is a leftover copy. The page line is
    /// "Music Index reads this feed from {address}. This file is a leftover
    /// copy."
    LeftoverCopy {
        /// The address that Music Index reads.
        index_url: String,
    },
    /// The read gave a 404 with `superseded_by`. musicindex holds the feed
    /// with a different GUID, so it did not remove the feed. The page line
    /// is "Music Index holds this feed with a different GUID."
    HeldWithOtherGuid {
        /// The new GUID that musicindex names.
        new_guid: String,
    },
    /// The read failed. The value is the address of the request.
    NotKnownFailed {
        /// The address of the request that failed.
        address: String,
    },
    /// The feed has no `podcast:guid`, so Studio sent no request.
    NotKnownNoGuid,
    /// No read ran yet.
    NotRead,
}

/// Step 3 of ADR 0131 item 6: the delete of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileDelete {
    /// The feed file is on its host, as far as Studio knows.
    NotDone,
    /// The delete record of the feed exists (ADR 0131 item 14).
    Done,
}

/// The state of the three steps.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RetireState {
    /// Step 1.
    pub ask: RetireAsk,
    /// Step 2.
    pub removal: IndexRemoval,
    /// True when step 2 is done: only a plain 404. The UI offers **Delete
    /// the File** as the next step of a retire only when this is true.
    pub removal_done: bool,
    /// Step 3.
    pub delete: FileDelete,
}

/// The facts of step 1 and step 3, and the `podcast:guid` for the read of
/// step 2. The function sends no request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetireFacts {
    /// Step 1.
    pub ask: RetireAsk,
    /// Step 3.
    pub delete: FileDelete,
    /// The `podcast:guid` of the draft, or of the last read of the feed.
    pub guid: Option<String>,
    /// The public address of this file. `None` when the feed has none, or
    /// when the address is a placeholder.
    pub public_address: Option<String>,
    /// The last move of the feed that Studio completed.
    pub last_move: Option<LastWrittenMove>,
}

/// Collect the local facts of the retire state of a feed.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn retire_facts(
    collection: &Collection,
    library: &Library,
    id: FeedId,
) -> Result<RetireFacts, PipelineError> {
    let row = collection.feed(id)?;
    let draft = if collection.draft(id)?.is_some() {
        Some(pipeline::draft_model(collection, id)?.0)
    } else {
        None
    };
    let deleted = collection.feed_host_delete(id)?.is_some();
    let written = (deleted && draft.as_ref().is_some_and(Feed::hidden))
        || pipeline::hosted_copy(library, &row).is_some_and(|feed| feed.hidden());
    let ask = match (written, draft.as_ref().map(Feed::hidden)) {
        (false, Some(true)) => RetireAsk::DraftOnly,
        (true, Some(false)) => RetireAsk::WrittenCleared,
        (true, _) => RetireAsk::Written,
        (false, _) => RetireAsk::No,
    };
    let guid = draft
        .and_then(|feed| feed.podcast_guid)
        .or(row.podcast_guid.clone())
        .filter(|guid| !guid.is_empty());
    let delete = if deleted {
        FileDelete::Done
    } else {
        FileDelete::NotDone
    };
    let public_address = outside::address_of(&row);
    let last_move = outside::last_written_move(collection, &row)?;
    Ok(RetireFacts {
        ask,
        delete,
        guid,
        public_address,
        last_move,
    })
}

/// Read the record of the feed from the index. This is the only request of
/// the retire (ADR 0131 item 6). A feed with no `podcast:guid` sends none.
///
/// A record at an address that differs from `public_address` gives
/// [`IndexRemoval::LeftoverCopy`], unless Studio moved the feed from that
/// address (ADR 0228 items 20 to 22). With no known public address the
/// answer stays [`IndexRemoval::NotYet`].
pub fn read_removal(
    guid: Option<&str>,
    public_address: Option<&str>,
    last_move: Option<&LastWrittenMove>,
    reads: &dyn IndexReads,
) -> IndexRemoval {
    let Some(guid) = guid else {
        return IndexRemoval::NotKnownNoGuid;
    };
    match reads.feed_presence(guid) {
        Ok(IndexPresence::Absent) => IndexRemoval::Removed,
        Ok(IndexPresence::Held(record)) => {
            leftover_address(record.url.as_deref(), public_address, last_move)
                .map_or(IndexRemoval::NotYet, |index_url| {
                    IndexRemoval::LeftoverCopy { index_url }
                })
        }
        Ok(IndexPresence::Superseded(new_guid)) => IndexRemoval::HeldWithOtherGuid { new_guid },
        Err(address) => IndexRemoval::NotKnownFailed { address },
    }
}

/// The address that Music Index reads, when this file is a leftover copy
/// (ADR 0228 items 20 to 22). Both addresses get the same plain form first.
fn leftover_address(
    index_url: Option<&str>,
    public_address: Option<&str>,
    last_move: Option<&LastWrittenMove>,
) -> Option<String> {
    let index_url = plain_address(index_url?.trim()).into_owned();
    let own = plain_address(public_address?.trim());
    if index_url.is_empty()
        || index_url == own
        || same_but_spelling(&index_url, &own)
        || outside::index_is_move_source(Some(&index_url), last_move)
    {
        return None;
    }
    Some(index_url)
}

/// Refuse a delete while Music Index may hold the feed.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::IndexHoldsFeed`] unless Music Index removed the feed.
pub fn ensure_index_removed(removal: &IndexRemoval) -> Result<(), PipelineError> {
    let next_step = match removal {
        IndexRemoval::Removed | IndexRemoval::LeftoverCopy { .. } => return Ok(()),
        IndexRemoval::NotYet => "retire the feed and wait until Music Index removes the entry",
        IndexRemoval::HeldWithOtherGuid { .. } => {
            "retire the feed that has the new GUID in Music Index"
        }
        IndexRemoval::NotKnownFailed { .. } => {
            "restore access to Music Index and check the feed again"
        }
        IndexRemoval::NotKnownNoGuid => "add a GUID and check the feed in Music Index",
        IndexRemoval::NotRead => "check the feed in Music Index",
    };
    Err(PipelineError::new(
        PipelineErrorKind::IndexHoldsFeed,
        format!("ADR 0200 item 6: Music Index may hold this feed. To continue, {next_step}."),
    ))
}

/// Join the facts and the read into the state.
#[must_use]
pub fn retire_state(facts: &RetireFacts, removal: IndexRemoval) -> RetireState {
    RetireState {
        ask: facts.ask,
        removal_done: matches!(
            removal,
            IndexRemoval::Removed | IndexRemoval::LeftoverCopy { .. }
        ),
        removal,
        delete: facts.delete,
    }
}

/// Set or clear "Hide from apps" in the draft of a feed (ADR 0131 items 5
/// and 8). The draft must exist. The function writes nothing on a host.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::InvalidOperation`] when the draft is a delete
/// draft. Gives [`PipelineErrorKind::NoDraft`] when the feed has no draft.
pub fn set_retire(collection: &Collection, id: FeedId, on: bool) -> Result<(), PipelineError> {
    if collection.has_draft_feed_delete(id)? {
        return Err(PipelineError::new(
            PipelineErrorKind::InvalidOperation,
            "ADR 0131 item 12: a delete draft has no retire; discard it first",
        ));
    }
    let (mut model, revision) = pipeline::draft_model(collection, id)?;
    model.set_hidden(on);
    pipeline::replace_model(collection, id, &revision, model)?;
    Ok(())
}

/// A publisher feed that links to the feed (ADR 0131 item 13).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PublisherLine {
    /// The ID of the publisher feed.
    pub id: FeedId,
    /// The title of the publisher feed. `None` when it has none.
    pub title: Option<String>,
}

/// The names for the final sheet of a delete (ADR 0131 items 10 and 13).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeleteNames {
    /// The name of the host.
    pub host_name: String,
    /// The path of the feed file on the host.
    pub path: String,
    /// The text that the user must type: the title of the feed, or the file
    /// name when the feed has no title.
    pub confirm_text: String,
    /// Each publisher feed in the collection that links to the feed.
    pub publishers: Vec<PublisherLine>,
}

/// The names of the final sheet for a feed with a connector.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::ReadOnly`] when the feed has no host.
pub fn delete_names(
    collection: &Collection,
    library: &Library,
    id: FeedId,
) -> Result<DeleteNames, PublisherError> {
    let row = collection.feed(id)?;
    let (Some(connector_id), Some(path)) = (row.connector_id, row.path.clone()) else {
        return Err(PipelineError::new(
            PipelineErrorKind::ReadOnly,
            format!("feed {} has no host, so Studio cannot delete it", id.0),
        )
        .into());
    };
    let host_name = collection.connector(connector_id)?.name;
    let file_name = path.rsplit_once('/').map_or(path.as_str(), |(_, n)| n);
    let confirm_text = row
        .title
        .as_deref()
        .filter(|title| !title.is_empty())
        .unwrap_or(file_name)
        .to_owned();
    let mut publishers = Vec::new();
    for publisher_id in publisher::publishers_linking(collection, library, id)? {
        publishers.push(PublisherLine {
            id: publisher_id,
            title: collection.feed(publisher_id)?.title,
        });
    }
    Ok(DeleteNames {
        host_name,
        path,
        confirm_text,
        publishers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_index_removed_accepts_only_removed_or_a_leftover_copy() {
        assert!(ensure_index_removed(&IndexRemoval::Removed).is_ok());
        assert!(
            ensure_index_removed(&IndexRemoval::LeftoverCopy {
                index_url: "https://other.example/a.xml".to_owned()
            })
            .is_ok()
        );
        let held = [
            IndexRemoval::NotYet,
            IndexRemoval::HeldWithOtherGuid {
                new_guid: "new-guid".to_owned(),
            },
            IndexRemoval::NotKnownFailed {
                address: "https://index.example/feed".to_owned(),
            },
            IndexRemoval::NotKnownNoGuid,
            IndexRemoval::NotRead,
        ];
        for removal in held {
            let error = ensure_index_removed(&removal).unwrap_err();
            assert_eq!(error.kind(), PipelineErrorKind::IndexHoldsFeed);
            assert!(error.to_string().starts_with("ADR 0200 item 6:"));
            assert!(error.to_string().contains("To continue,"));
        }
    }
}
