//! Change sets: one recipient operation in many feeds (ADR 0091).
//!
//! [`apply`] changes the model of one feed. [`make_change_set`] puts the
//! change in the draft of each feed of a scope, and keeps the draft from
//! before the change. [`discard_change_set`] puts these drafts back.
//! Nothing here writes to a host: the write goes through
//! `pipeline::write_draft` for each feed (ADR 0007).

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use serde::{Deserialize, Serialize};

use crate::collection::{
    ChangeSetFeedRow, Collection, CollectionError, CollectionErrorKind, FeedId, SavedRecipientRow,
    now,
};
use crate::feed::{Entry, Feed, Recipient, ValueBlock};
use crate::pipeline::{self, PipelineError};
use crate::publisher::Catalog;

/// The identity of a recipient (ADR 0027 item 3). An empty custom key or
/// custom value is the same as none.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipientKey {
    /// `address`.
    pub address: String,
    /// `customKey`.
    pub custom_key: Option<String>,
    /// `customValue`.
    pub custom_value: Option<String>,
}

impl RecipientKey {
    /// The address, the custom key and the custom value, with empty text for
    /// a missing custom key or custom value.
    #[must_use]
    pub fn parts(&self) -> (&str, &str, &str) {
        (
            &self.address,
            self.custom_key.as_deref().unwrap_or(""),
            self.custom_value.as_deref().unwrap_or(""),
        )
    }

    /// True when `recipient` has this identity.
    #[must_use]
    pub fn matches(&self, recipient: &Recipient) -> bool {
        let other = (
            recipient.address.as_deref().unwrap_or(""),
            recipient.custom_key.as_deref().unwrap_or(""),
            recipient.custom_value.as_deref().unwrap_or(""),
        );
        self.parts() == other
    }
}

/// The members of one wallet (ADR 0095 items 1 and 5). An operation acts on
/// each recipient that a member matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallet {
    /// The key of the wallet. The change set keeps it as its recipient.
    pub key: RecipientKey,
    /// The keys that name the wallet: the key itself, and each Lightning
    /// address whose lookup gives the key.
    pub members: Vec<RecipientKey>,
}

impl Wallet {
    /// A wallet whose only member is `key`.
    #[must_use]
    pub fn single(key: RecipientKey) -> Self {
        Self {
            members: vec![key.clone()],
            key,
        }
    }

    /// True when a member matches `recipient`.
    #[must_use]
    pub fn matches(&self, recipient: &Recipient) -> bool {
        self.members.iter().any(|m| m.matches(recipient))
    }

    /// True when a member is equal to `key`.
    #[must_use]
    pub fn has(&self, key: &RecipientKey) -> bool {
        self.members.iter().any(|m| m.parts() == key.parts())
    }
}

/// One operation on a recipient (ADR 0091 items 5 to 7 and 9, ADR 0095).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecipientOperation {
    /// Replace the address, the custom pair and the type.
    ChangeAddress {
        /// The new `address`.
        address: String,
        /// The new `customKey`.
        custom_key: Option<String>,
        /// The new `customValue`.
        custom_value: Option<String>,
        /// The new `type`.
        recipient_type: Option<String>,
    },
    /// Replace `name`.
    Rename {
        /// The new `name`.
        name: String,
    },
    /// Delete the recipient.
    Remove,
    /// Add the recipient to each value block.
    Add {
        /// `name`.
        name: Option<String>,
        /// `type`.
        recipient_type: Option<String>,
        /// `address`.
        address: String,
        /// `customKey`.
        custom_key: Option<String>,
        /// `customValue`.
        custom_value: Option<String>,
        /// The split of the new recipient, from 1 to 99.
        percent: u64,
    },
    /// Replace each `node` recipient of the wallet with an `lnaddress`
    /// recipient (ADR 0095 item 11).
    UseLnaddress {
        /// The Lightning address.
        address: String,
    },
}

/// The reason why a feed does not change (ADR 0091 item 17).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cause", content = "reason", rename_all = "snake_case")]
pub enum Cause {
    /// No value block has the recipient.
    NoRecipient,
    /// A value block has a recipient of the wallet, but no recipient of the
    /// wallet that is not an `lnaddress` recipient (Use the Lightning
    /// address).
    NoNodeRecipient,
    /// Each value block has the recipient before the change (Add).
    HasRecipient,
    /// A value block has the new address before the change.
    HasNewAddress,
    /// The feed has no value block (Add).
    NoValueBlock,
    /// Each other split of a block is a fee, or the block has no other
    /// recipient (Add and Remove).
    OnlyFees,
    /// A split that is not a fee is not a non-negative integer (Add and
    /// Remove).
    SplitNotNumber,
    /// The feed has no draft, and Studio could not make one. The text is
    /// the reason.
    CannotStart(String),
    /// The percentage of Add is not from 1 to 99.
    BadPercent,
    /// The feed has a change of an open change set that is not written.
    InOpenChangeSet,
}

/// The result of an operation for one feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    /// The model changed.
    Changed {
        /// True when the channel value block changed.
        channel: bool,
        /// The number of items with a changed value block.
        items: usize,
        /// The number of live items with a changed value block.
        live_items: usize,
    },
    /// The model did not change.
    NotChanged {
        /// The cause.
        cause: Cause,
    },
}

impl Outcome {
    /// True for [`Outcome::Changed`].
    #[must_use]
    pub fn is_changed(&self) -> bool {
        matches!(self, Self::Changed { .. })
    }

    fn not_changed(cause: Cause) -> Self {
        Self::NotChanged { cause }
    }
}

/// Scale `splits` so that their total is `total`. Each split keeps its
/// relative share. The function rounds down, and gives the remainder to the
/// largest split. With two or more equal largest splits, the first one gets
/// the remainder (ADR 0091 items 13 and 14).
///
/// A fee split is not an input. When the total of `splits` is 0, the
/// function gives `splits` as they are, because no share exists.
#[must_use]
pub fn scale(splits: &[u64], total: u64) -> Vec<u64> {
    let sum: u128 = splits.iter().map(|&s| u128::from(s)).sum();
    if sum == 0 {
        return splits.to_vec();
    }
    let mut scaled: Vec<u64> = splits
        .iter()
        .map(|&s| {
            let share = u128::from(s) * u128::from(total) / sum;
            u64::try_from(share).unwrap_or(u64::MAX)
        })
        .collect();
    let given: u64 = scaled.iter().sum();
    let remainder = total.saturating_sub(given);
    // `max_by_key` gives the last of equal values, so search by hand.
    let mut largest = 0;
    for (index, &split) in splits.iter().enumerate() {
        if split > splits[largest] {
            largest = index;
        }
    }
    scaled[largest] += remainder;
    scaled
}

fn is_fee(recipient: &Recipient) -> bool {
    recipient
        .fee
        .as_deref()
        .is_some_and(|f| f.trim().eq_ignore_ascii_case("true"))
}

fn parse_split(recipient: &Recipient) -> Option<u64> {
    let text = recipient.split.as_deref()?.trim();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn non_empty(text: Option<&String>) -> Option<String> {
    text.filter(|t| !t.is_empty()).cloned()
}

/// The result for one value block.
enum BlockResult {
    Changed,
    Skipped(Cause),
}

/// Set the splits of the non-fee recipients that `positions` names.
fn set_splits(block: &mut ValueBlock, positions: &[usize], splits: &[u64], old: &[u64]) {
    for ((&position, &split), &before) in positions.iter().zip(splits).zip(old) {
        if split != before {
            block.recipients[position].value.split = Some(split.to_string());
        }
    }
}

/// Write `method="lnaddress"` on a block that an operation changed, when
/// its method is none or `keysend` (ADR 0095 item 13).
fn set_lnaddress_method(block: &mut ValueBlock) {
    if block
        .method
        .as_deref()
        .is_none_or(|m| m.eq_ignore_ascii_case("keysend"))
    {
        block.method = Some("lnaddress".to_owned());
    }
}

fn is_lnaddress_type(recipient_type: Option<&str>) -> bool {
    recipient_type == Some("lnaddress")
}

fn apply_block(
    block: &mut ValueBlock,
    wallet: &Wallet,
    operation: &RecipientOperation,
) -> BlockResult {
    let has_wallet = |block: &ValueBlock| block.recipients.iter().any(|e| wallet.matches(&e.value));
    match operation {
        RecipientOperation::ChangeAddress {
            address,
            custom_key,
            custom_value,
            recipient_type,
        } => {
            if !has_wallet(block) {
                return BlockResult::Skipped(Cause::NoRecipient);
            }
            let new_key = RecipientKey {
                address: address.clone(),
                custom_key: custom_key.clone(),
                custom_value: custom_value.clone(),
            };
            if block
                .recipients
                .iter()
                .any(|e| new_key.matches(&e.value) && !wallet.matches(&e.value))
            {
                return BlockResult::Skipped(Cause::HasNewAddress);
            }
            for entry in &mut block.recipients {
                if wallet.matches(&entry.value) {
                    entry.value.address = Some(address.clone());
                    entry.value.custom_key = non_empty(custom_key.as_ref());
                    entry.value.custom_value = non_empty(custom_value.as_ref());
                    entry.value.recipient_type.clone_from(recipient_type);
                }
            }
            if is_lnaddress_type(recipient_type.as_deref()) {
                set_lnaddress_method(block);
            }
            BlockResult::Changed
        }
        RecipientOperation::Rename { name } => {
            if !has_wallet(block) {
                return BlockResult::Skipped(Cause::NoRecipient);
            }
            for entry in &mut block.recipients {
                if wallet.matches(&entry.value) {
                    entry.value.name = Some(name.clone());
                }
            }
            BlockResult::Changed
        }
        RecipientOperation::Remove => {
            if !has_wallet(block) {
                return BlockResult::Skipped(Cause::NoRecipient);
            }
            remove_from_block(block, wallet)
        }
        RecipientOperation::Add {
            name,
            recipient_type,
            address,
            custom_key,
            custom_value,
            percent,
        } => {
            if has_wallet(block) {
                return BlockResult::Skipped(Cause::HasRecipient);
            }
            let added = Recipient {
                name: name.clone(),
                recipient_type: recipient_type.clone(),
                address: Some(address.clone()),
                split: Some(percent.to_string()),
                custom_key: non_empty(custom_key.as_ref()),
                custom_value: non_empty(custom_value.as_ref()),
                fee: None,
            };
            let result = add_to_block(block, added, *percent);
            if matches!(result, BlockResult::Changed)
                && is_lnaddress_type(recipient_type.as_deref())
            {
                set_lnaddress_method(block);
            }
            result
        }
        RecipientOperation::UseLnaddress { address } => {
            use_lnaddress_in_block(block, wallet, address)
        }
    }
}

/// Replace each recipient of the wallet that is not an `lnaddress`
/// recipient (ADR 0095 item 11).
fn use_lnaddress_in_block(block: &mut ValueBlock, wallet: &Wallet, address: &str) -> BlockResult {
    let to_change = |e: &Entry<Recipient>| {
        wallet.matches(&e.value) && !is_lnaddress_type(e.value.recipient_type.as_deref())
    };
    if !block.recipients.iter().any(to_change) {
        let has_wallet = block.recipients.iter().any(|e| wallet.matches(&e.value));
        return BlockResult::Skipped(if has_wallet {
            Cause::NoNodeRecipient
        } else {
            Cause::NoRecipient
        });
    }
    for entry in &mut block.recipients {
        if to_change(entry) {
            entry.value.recipient_type = Some("lnaddress".to_owned());
            entry.value.address = Some(address.to_owned());
            entry.value.custom_key = None;
            entry.value.custom_value = None;
        }
    }
    set_lnaddress_method(block);
    BlockResult::Changed
}

/// Remove the recipient. The other non-fee splits scale to the non-fee
/// total before the change (ADR 0091 item 14).
fn remove_from_block(block: &mut ValueBlock, wallet: &Wallet) -> BlockResult {
    let mut total = 0_u64;
    let mut positions = Vec::new();
    let mut splits = Vec::new();
    let mut removes_share = false;
    for (position, entry) in block.recipients.iter().enumerate() {
        if is_fee(&entry.value) {
            continue;
        }
        let Some(split) = parse_split(&entry.value) else {
            return BlockResult::Skipped(Cause::SplitNotNumber);
        };
        total = total.saturating_add(split);
        if wallet.matches(&entry.value) {
            removes_share = true;
        } else {
            positions.push(position);
            splits.push(split);
        }
    }
    if removes_share && splits.iter().sum::<u64>() == 0 {
        return BlockResult::Skipped(Cause::OnlyFees);
    }
    let scaled = scale(&splits, total);
    set_splits(block, &positions, &scaled, &splits);
    block.recipients.retain(|e| !wallet.matches(&e.value));
    BlockResult::Changed
}

/// Add the recipient with `percent`. The other non-fee splits scale to
/// `100 - percent` (ADR 0091 item 13).
fn add_to_block(block: &mut ValueBlock, added: Recipient, percent: u64) -> BlockResult {
    let mut positions = Vec::new();
    let mut splits = Vec::new();
    for (position, entry) in block.recipients.iter().enumerate() {
        if is_fee(&entry.value) {
            continue;
        }
        let Some(split) = parse_split(&entry.value) else {
            return BlockResult::Skipped(Cause::SplitNotNumber);
        };
        positions.push(position);
        splits.push(split);
    }
    if splits.iter().sum::<u64>() == 0 {
        return BlockResult::Skipped(Cause::OnlyFees);
    }
    let scaled = scale(&splits, 100 - percent);
    set_splits(block, &positions, &scaled, &splits);
    block.recipients.push(Entry::new(added));
    BlockResult::Changed
}

/// The cause to report when no block changes. A cause that tells the user
/// to fix a split comes before a plain "has it".
fn pick_cause(causes: &[Cause], operation: &RecipientOperation) -> Cause {
    if causes.is_empty() {
        return match operation {
            RecipientOperation::Add { .. } => Cause::NoValueBlock,
            _ => Cause::NoRecipient,
        };
    }
    for wanted in [
        Cause::SplitNotNumber,
        Cause::OnlyFees,
        Cause::HasRecipient,
        Cause::HasNewAddress,
        Cause::NoNodeRecipient,
    ] {
        if causes.contains(&wanted) {
            return wanted;
        }
    }
    Cause::NoRecipient
}

/// Apply `operation` to each value block of the channel, of each item and
/// of each live item. The function does not change a value time split, and
/// it does not change an attribute that the operation does not name
/// (ADR 0091 items 5 to 10).
///
/// When no block changes, the feed does not change.
pub fn apply(feed: &mut Feed, wallet: &Wallet, operation: &RecipientOperation) -> Outcome {
    if let RecipientOperation::Add { percent, .. } = operation
        && !(1..=99).contains(percent)
    {
        return Outcome::not_changed(Cause::BadPercent);
    }
    let mut causes = Vec::new();
    let mut run = |blocks: &mut [Entry<ValueBlock>]| -> bool {
        let mut changed = false;
        for entry in blocks {
            // Add reports the blocks that have the key. The other
            // operations report only the blocks that can name a cause.
            match apply_block(&mut entry.value, wallet, operation) {
                BlockResult::Changed => changed = true,
                BlockResult::Skipped(cause) => causes.push(cause),
            }
        }
        changed
    };
    let channel = run(&mut feed.values);
    let items = feed
        .items
        .iter_mut()
        .map(|item| run(&mut item.value.values))
        .filter(|&changed| changed)
        .count();
    let live_items = feed
        .live_items
        .iter_mut()
        .map(|live| run(&mut live.value.item.values))
        .filter(|&changed| changed)
        .count();
    if channel || items > 0 || live_items > 0 {
        return Outcome::Changed {
            channel,
            items,
            live_items,
        };
    }
    // Blocks without the key give `NoRecipient` for the three operations
    // that need the key. They are not a cause by themselves when another
    // block names a better one.
    Outcome::not_changed(pick_cause(&causes, operation))
}

/// One feed of a change set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSetFeed {
    /// The feed.
    pub feed_id: FeedId,
    /// The outcome for the feed.
    pub outcome: Outcome,
    /// True when the feed had a draft before the change set.
    pub had_draft: bool,
    /// The Unix time of the write of the feed after the change.
    pub written_at: Option<i64>,
}

/// A change set (ADR 0091 item 17).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    /// The ID.
    pub id: i64,
    /// The recipient that the operation names.
    pub recipient: RecipientKey,
    /// The operation.
    pub operation: RecipientOperation,
    /// The Unix time when the user made the change set.
    pub made_at: i64,
    /// The feeds, with their outcomes.
    pub feeds: Vec<ChangeSetFeed>,
}

impl ChangeSet {
    /// True while a changed feed has no write (ADR 0091 item 19).
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.feeds
            .iter()
            .any(|f| f.outcome.is_changed() && f.written_at.is_none())
    }
}

fn invalid(message: impl Into<String>) -> PipelineError {
    PipelineError::from(CollectionError::with_kind(
        CollectionErrorKind::Invalid,
        message,
    ))
}

fn to_json<T: Serialize>(value: &T) -> Result<String, PipelineError> {
    serde_json::to_string(value).map_err(|e| invalid(format!("cannot store the change set: {e}")))
}

fn from_json<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T, PipelineError> {
    serde_json::from_str(text).map_err(|e| invalid(format!("cannot read the change set: {e}")))
}

/// Remove a draft that a change set made, with its host operations.
fn remove_new_draft(collection: &Collection, id: FeedId) -> Result<(), PipelineError> {
    pipeline::discard_draft(collection, id)?;
    collection.delete_draft(id)?;
    Ok(())
}

/// Apply `operation` to the draft of each feed of `feeds`, and keep the
/// draft of each feed from before the change (ADR 0091 items 17 to 19).
///
/// `start` makes a draft for a feed that has none, so that the core needs
/// no connector. A feed that `apply` does not change keeps its draft as it
/// was; a draft that `start` made for it goes out again. A failure of
/// `start` is the outcome [`Cause::CannotStart`] for that feed.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails or a draft is not a
/// feed. A change set that was part-made then stays; the caller can discard
/// it.
pub fn make_change_set(
    collection: &Collection,
    start: &mut dyn FnMut(FeedId) -> Result<(), String>,
    wallet: &Wallet,
    operation: &RecipientOperation,
    feeds: &[FeedId],
) -> Result<ChangeSet, PipelineError> {
    check_operation(operation)?;
    // A closed change set has no more work (ADR 0091 item 19).
    collection.delete_closed_change_sets()?;
    let made_at = now();
    let id = collection.add_change_set(&to_json(operation)?, &to_json(&wallet.key)?, made_at)?;
    let mut seen = HashSet::new();
    let mut changed = false;
    let busy: HashSet<FeedId> = change_sets(collection)?
        .iter()
        .flat_map(|set| &set.feeds)
        .filter(|f| f.outcome.is_changed() && f.written_at.is_none())
        .map(|f| f.feed_id)
        .collect();
    for &feed_id in feeds {
        if !seen.insert(feed_id) {
            continue;
        }
        if busy.contains(&feed_id) {
            // The feed keeps the draft of the open change set. No draft
            // is made.
            collection.add_change_set_feed(
                id,
                &ChangeSetFeedRow {
                    feed_id,
                    outcome: to_json(&Outcome::not_changed(Cause::InOpenChangeSet))?,
                    before: None,
                    had_draft: collection.draft(feed_id)?.is_some(),
                    written_at: None,
                },
            )?;
            continue;
        }
        let before = collection.draft(feed_id)?;
        let had_draft = before.is_some();
        let outcome = change_feed(collection, start, wallet, operation, feed_id, had_draft)?;
        changed |= outcome.is_changed();
        let kept = if outcome.is_changed() { before } else { None };
        collection.add_change_set_feed(
            id,
            &ChangeSetFeedRow {
                feed_id,
                outcome: to_json(&outcome)?,
                before: kept,
                had_draft,
                written_at: None,
            },
        )?;
    }
    // A change set with no changed feed does not change the saved
    // recipient. Thus it holds nothing that Discard must put back.
    if changed {
        change_saved_recipient(collection, id, wallet, operation)?;
    }
    change_set(collection, id)
}

/// Refuse an operation that has no type or no address (ADR 0095 items 12
/// and 14).
fn check_operation(operation: &RecipientOperation) -> Result<(), PipelineError> {
    match operation {
        RecipientOperation::ChangeAddress { recipient_type, .. }
        | RecipientOperation::Add { recipient_type, .. }
            if recipient_type
                .as_deref()
                .is_none_or(|t| t.trim().is_empty()) =>
        {
            Err(invalid("the recipient needs a type"))
        }
        RecipientOperation::UseLnaddress { address } if address.trim().is_empty() => {
            Err(invalid("the Lightning address is empty"))
        }
        _ => Ok(()),
    }
}

/// The saved recipients from before a change set, as the column
/// `saved_before` keeps them. A row from before ADR 0095 holds one object.
#[derive(Deserialize)]
#[serde(untagged)]
enum SavedBefore {
    Many(Vec<SavedRecipientRow>),
    One(SavedRecipientRow),
}

/// Change the saved recipient of each member for Change address and Rename,
/// and keep the saved recipients from before the change in the change set
/// (ADR 0091 item 12, ADR 0095 item 9). Other operations, and a member that
/// is not saved, change nothing.
fn change_saved_recipient(
    collection: &Collection,
    id: i64,
    wallet: &Wallet,
    operation: &RecipientOperation,
) -> Result<(), PipelineError> {
    let (new_identity, name) = match operation {
        RecipientOperation::ChangeAddress {
            address,
            custom_key,
            custom_value,
            recipient_type,
        } => (
            Some((
                address.as_str(),
                custom_key.as_deref().unwrap_or_default(),
                custom_value.as_deref().unwrap_or_default(),
                recipient_type.as_deref(),
            )),
            None,
        ),
        RecipientOperation::Rename { name } => (None, Some(name.as_str())),
        RecipientOperation::Remove
        | RecipientOperation::Add { .. }
        | RecipientOperation::UseLnaddress { .. } => return Ok(()),
    };
    let mut before = Vec::new();
    for member in &wallet.members {
        if let Some(row) = collection.saved_recipient_row(member.parts())?
            && !before.contains(&row)
        {
            before.push(row);
        }
    }
    if before.is_empty() {
        return Ok(());
    }
    collection.set_change_set_saved_before(id, &to_json(&before)?)?;
    for row in &before {
        collection.change_saved_recipient(
            (&row.address, &row.custom_key, &row.custom_value),
            new_identity,
            name,
        )?;
    }
    Ok(())
}

/// Change the draft of one feed. Gives the outcome.
fn change_feed(
    collection: &Collection,
    start: &mut dyn FnMut(FeedId) -> Result<(), String>,
    wallet: &Wallet,
    operation: &RecipientOperation,
    feed_id: FeedId,
    had_draft: bool,
) -> Result<Outcome, PipelineError> {
    if !had_draft && let Err(reason) = start(feed_id) {
        // A failed start can leave a part of a draft.
        remove_new_draft(collection, feed_id)?;
        return Ok(Outcome::not_changed(Cause::CannotStart(reason)));
    }
    // Calculate the outcome before the edit, so that an unchanged draft
    // keeps its bytes.
    let (mut model, _) = pipeline::draft_model(collection, feed_id)?;
    let outcome = apply(&mut model, wallet, operation);
    if outcome.is_changed() {
        pipeline::edit_draft(collection, feed_id, |feed| {
            apply(feed, wallet, operation);
        })?;
    } else if !had_draft {
        remove_new_draft(collection, feed_id)?;
    }
    Ok(outcome)
}

/// One change set by its ID, open or not. A change set stays in the
/// collection after the write of its last feed, until the user discards it.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the change set does not exist, when the
/// collection fails or when a stored row is not valid.
pub fn find_change_set(collection: &Collection, id: i64) -> Result<ChangeSet, PipelineError> {
    change_set(collection, id)
}

fn change_set(collection: &Collection, id: i64) -> Result<ChangeSet, PipelineError> {
    let row = collection
        .change_set_row(id)?
        .ok_or_else(|| invalid(format!("change set {id} does not exist")))?;
    let mut feeds = Vec::new();
    for feed in collection.change_set_feed_rows(id)? {
        feeds.push(ChangeSetFeed {
            feed_id: feed.feed_id,
            outcome: from_json(&feed.outcome)?,
            had_draft: feed.had_draft,
            written_at: feed.written_at,
        });
    }
    Ok(ChangeSet {
        id,
        recipient: from_json(&row.recipient)?,
        operation: from_json(&row.operation)?,
        made_at: row.made_at,
        feeds,
    })
}

/// The open change sets, the oldest first (ADR 0091 item 19).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails or a stored row is
/// not valid.
pub fn change_sets(collection: &Collection) -> Result<Vec<ChangeSet>, PipelineError> {
    let mut open = Vec::new();
    for row in collection.change_set_rows()? {
        let set = change_set(collection, row.id)?;
        if set.is_open() {
            open.push(set);
        }
    }
    Ok(open)
}

/// Put back the draft of each feed from before the change, byte for byte,
/// and the saved recipient from before the change.
/// A feed that had no draft then has no draft. A feed that Studio wrote
/// after the change does not change. Then the change set goes (ADR 0091
/// items 19 and 20).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails. The change set then
/// stays.
pub fn discard_change_set(collection: &Collection, id: i64) -> Result<(), PipelineError> {
    for row in collection.change_set_feed_rows(id)? {
        let outcome: Outcome = from_json(&row.outcome)?;
        if !outcome.is_changed() || row.written_at.is_some() {
            continue;
        }
        match row.before {
            Some(before) if row.had_draft => collection.save_draft(row.feed_id, &before)?,
            _ => remove_new_draft(collection, row.feed_id)?,
        }
    }
    if let Some(text) = collection.change_set_saved_before(id)? {
        let rows = match from_json::<SavedBefore>(&text)? {
            SavedBefore::Many(rows) => rows,
            SavedBefore::One(row) => vec![row],
        };
        for row in rows.iter().rev() {
            collection.restore_saved_recipient(row)?;
        }
    }
    collection.delete_change_set(id)?;
    Ok(())
}

/// The feeds of a level: the publisher feed, each feed below it through
/// `children` (all levels), and the releases of each of these feeds. Each
/// feed is in the list one time (ADR 0091 item 9).
///
/// `catalogs` gives the catalog of each feed, by feed ID.
#[must_use]
pub fn level_feeds<S: BuildHasher>(
    catalogs: &HashMap<FeedId, Catalog, S>,
    publisher: FeedId,
) -> Vec<FeedId> {
    let mut found = vec![publisher];
    let mut seen: HashSet<FeedId> = HashSet::from([publisher]);
    let mut index = 0;
    // First the publisher feeds, so that a cycle in the links ends.
    while index < found.len() {
        if let Some(catalog) = catalogs.get(&found[index]) {
            for &child in &catalog.children {
                if seen.insert(child) {
                    found.push(child);
                }
            }
        }
        index += 1;
    }
    let publishers = found.clone();
    for feed in publishers {
        if let Some(catalog) = catalogs.get(&feed) {
            for &release in &catalog.releases {
                if seen.insert(release) {
                    found.push(release);
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Item, LiveItem, ValueTimeSplit};

    fn recipient(address: &str, split: &str) -> Entry<Recipient> {
        Entry::new(Recipient {
            name: Some(format!("name {address}")),
            recipient_type: Some("node".into()),
            address: Some(address.into()),
            split: Some(split.into()),
            ..Recipient::default()
        })
    }

    fn fee(address: &str, split: &str) -> Entry<Recipient> {
        let mut entry = recipient(address, split);
        entry.value.fee = Some("true".into());
        entry
    }

    fn block(recipients: Vec<Entry<Recipient>>) -> Entry<ValueBlock> {
        Entry::new(ValueBlock {
            value_type: Some("lightning".into()),
            method: Some("keysend".into()),
            recipients,
            ..ValueBlock::default()
        })
    }

    fn key(address: &str) -> RecipientKey {
        RecipientKey {
            address: address.into(),
            custom_key: None,
            custom_value: None,
        }
    }

    fn wallet(address: &str) -> Wallet {
        Wallet::single(key(address))
    }

    /// A feed with a channel block, an item block, a live item block and
    /// a value time split. Each block has the recipient "a".
    fn feed() -> Feed {
        let mut channel = block(vec![recipient("a", "50"), recipient("b", "50")]);
        channel.value.time_splits = vec![Entry::new(ValueTimeSplit {
            start_time: Some("0".into()),
            recipients: vec![recipient("a", "100")],
            ..ValueTimeSplit::default()
        })];
        let item = Item {
            values: vec![block(vec![recipient("a", "80"), recipient("c", "20")])],
            ..Item::default()
        };
        let live = LiveItem {
            item: Item {
                values: vec![block(vec![recipient("a", "1"), recipient("d", "1")])],
                ..Item::default()
            },
            ..LiveItem::default()
        };
        Feed {
            values: vec![channel],
            items: vec![Entry::new(item)],
            live_items: vec![Entry::new(live)],
            ..Feed::default()
        }
    }

    fn splits(block: &Entry<ValueBlock>) -> Vec<(String, String)> {
        block
            .value
            .recipients
            .iter()
            .map(|e| {
                (
                    e.value.address.clone().unwrap_or_default(),
                    e.value.split.clone().unwrap_or_default(),
                )
            })
            .collect()
    }

    fn pair(address: &str, split: &str) -> (String, String) {
        (address.to_owned(), split.to_owned())
    }

    #[test]
    fn scale_gives_the_remainder_to_the_largest_split() {
        assert_eq!(scale(&[50, 30], 100), [63, 37]);
        assert_eq!(scale(&[2, 1], 90), [60, 30]);
        assert_eq!(scale(&[1, 1, 1], 100), [34, 33, 33]);
        assert_eq!(scale(&[1, 3, 3], 10), [1, 5, 4]);
    }

    #[test]
    fn scale_gives_the_remainder_to_the_first_of_equal_largest_splits() {
        assert_eq!(scale(&[1, 1], 3), [2, 1]);
        assert_eq!(scale(&[2, 5, 5], 10), [1, 5, 4]);
        assert_eq!(scale(&[0, 0], 10), [0, 0]);
    }

    #[test]
    fn change_address_replaces_the_pair_and_the_type_in_each_block() {
        let mut feed = feed();
        let operation = RecipientOperation::ChangeAddress {
            address: "new".into(),
            custom_key: Some("k".into()),
            custom_value: Some("v".into()),
            recipient_type: Some("lnaddress".into()),
        };
        let outcome = apply(&mut feed, &wallet("a"), &operation);
        assert_eq!(
            outcome,
            Outcome::Changed {
                channel: true,
                items: 1,
                live_items: 1
            }
        );
        let blocks = [
            &feed.values[0].value,
            &feed.items[0].value.values[0].value,
            &feed.live_items[0].value.item.values[0].value,
        ];
        for block in blocks {
            let first = &block.recipients[0].value;
            assert_eq!(first.address.as_deref(), Some("new"));
            assert_eq!(first.custom_key.as_deref(), Some("k"));
            assert_eq!(first.custom_value.as_deref(), Some("v"));
            assert_eq!(first.recipient_type.as_deref(), Some("lnaddress"));
            assert_eq!(first.name.as_deref(), Some("name a"));
        }
        assert_eq!(
            splits(&feed.values[0]),
            [pair("new", "50"), pair("b", "50")]
        );
        assert_eq!(
            splits(&feed.items[0].value.values[0]),
            [pair("new", "80"), pair("c", "20")]
        );
        let time_split = &feed.values[0].value.time_splits[0].value;
        assert_eq!(time_split.recipients[0].value.address.as_deref(), Some("a"));
    }

    #[test]
    fn rename_changes_only_the_name() {
        let mut feed = feed();
        let before = feed.clone();
        let operation = RecipientOperation::Rename { name: "Ann".into() };
        assert!(apply(&mut feed, &wallet("a"), &operation).is_changed());
        let mut expected = before;
        expected.values[0].value.recipients[0].value.name = Some("Ann".into());
        expected.items[0].value.values[0].value.recipients[0]
            .value
            .name = Some("Ann".into());
        expected.live_items[0].value.item.values[0].value.recipients[0]
            .value
            .name = Some("Ann".into());
        assert_eq!(feed.values, expected.values);
        assert_eq!(feed.items[0].value.values, expected.items[0].value.values);
    }

    #[test]
    fn remove_scales_the_other_non_fee_splits_to_the_old_total() {
        let mut feed = Feed {
            values: vec![block(vec![
                recipient("x", "50"),
                recipient("y", "30"),
                fee("f", "5"),
                recipient("a", "20"),
            ])],
            ..Feed::default()
        };
        let outcome = apply(&mut feed, &wallet("a"), &RecipientOperation::Remove);
        assert_eq!(
            outcome,
            Outcome::Changed {
                channel: true,
                items: 0,
                live_items: 0
            }
        );
        assert_eq!(
            splits(&feed.values[0]),
            [pair("x", "63"), pair("y", "37"), pair("f", "5")]
        );
    }

    fn add(percent: u64) -> RecipientOperation {
        RecipientOperation::Add {
            name: Some("New".into()),
            recipient_type: Some("node".into()),
            address: "n".into(),
            custom_key: Some(String::new()),
            custom_value: None,
            percent,
        }
    }

    #[test]
    fn add_gives_the_new_recipient_its_percent_and_scales_the_others() {
        let mut feed = Feed {
            values: vec![
                block(vec![recipient("x", "50"), recipient("y", "50")]),
                block(vec![recipient("x", "2"), recipient("y", "1")]),
            ],
            ..Feed::default()
        };
        assert!(apply(&mut feed, &wallet("n"), &add(10)).is_changed());
        assert_eq!(
            splits(&feed.values[0]),
            [pair("x", "45"), pair("y", "45"), pair("n", "10")]
        );
        assert_eq!(
            splits(&feed.values[1]),
            [pair("x", "60"), pair("y", "30"), pair("n", "10")]
        );
        let added = &feed.values[0].value.recipients[2].value;
        assert_eq!(added.custom_key, None);
        assert_eq!(added.name.as_deref(), Some("New"));
    }

    #[test]
    fn each_cause_has_a_test() {
        let rename = RecipientOperation::Rename { name: "z".into() };
        let none =
            |feed: &mut Feed, op: &RecipientOperation, k: &str| match apply(feed, &wallet(k), op) {
                Outcome::NotChanged { cause } => cause,
                Outcome::Changed { .. } => panic!("the feed must not change"),
            };
        // No recipient.
        assert_eq!(none(&mut feed(), &rename, "zz"), Cause::NoRecipient);
        assert_eq!(
            none(&mut feed(), &RecipientOperation::Remove, "zz"),
            Cause::NoRecipient
        );
        // Has the recipient.
        assert_eq!(none(&mut feed(), &add(10), "a"), Cause::HasRecipient);
        // Has the new address.
        let change = RecipientOperation::ChangeAddress {
            address: "b".into(),
            custom_key: None,
            custom_value: None,
            recipient_type: None,
        };
        let mut only_channel = Feed {
            values: vec![block(vec![recipient("a", "50"), recipient("b", "50")])],
            ..Feed::default()
        };
        let before = only_channel.clone();
        assert_eq!(none(&mut only_channel, &change, "a"), Cause::HasNewAddress);
        assert_eq!(only_channel, before);
        // No value block.
        assert_eq!(
            none(&mut Feed::default(), &add(10), "n"),
            Cause::NoValueBlock
        );
        // Only fees.
        let mut fees = Feed {
            values: vec![block(vec![fee("f", "5")])],
            ..Feed::default()
        };
        assert_eq!(none(&mut fees, &add(10), "n"), Cause::OnlyFees);
        let mut alone = Feed {
            values: vec![block(vec![recipient("a", "100"), fee("f", "5")])],
            ..Feed::default()
        };
        assert_eq!(
            none(&mut alone, &RecipientOperation::Remove, "a"),
            Cause::OnlyFees
        );
        // A split that is not a number.
        let mut text = Feed {
            values: vec![block(vec![recipient("a", "1.5"), recipient("b", "x")])],
            ..Feed::default()
        };
        assert_eq!(
            none(&mut text, &RecipientOperation::Remove, "a"),
            Cause::SplitNotNumber
        );
        assert_eq!(none(&mut text, &add(10), "n"), Cause::SplitNotNumber);
        // A percent outside 1 to 99.
        assert_eq!(none(&mut feed(), &add(0), "n"), Cause::BadPercent);
        assert_eq!(none(&mut feed(), &add(100), "n"), Cause::BadPercent);
    }

    #[test]
    fn a_block_that_cannot_change_stays_while_other_blocks_change() {
        let mut feed = Feed {
            values: vec![
                block(vec![recipient("x", "100")]),
                block(vec![recipient("n", "50"), recipient("x", "50")]),
            ],
            ..Feed::default()
        };
        assert!(apply(&mut feed, &wallet("n"), &add(10)).is_changed());
        assert_eq!(splits(&feed.values[0]), [pair("x", "90"), pair("n", "10")]);
        assert_eq!(splits(&feed.values[1]), [pair("n", "50"), pair("x", "50")]);
    }

    fn wallet_of_two() -> Wallet {
        Wallet {
            key: key("a"),
            members: vec![key("a"), key("b")],
        }
    }

    #[test]
    fn an_operation_acts_on_each_member_of_a_wallet() {
        let mut feed = Feed {
            values: vec![block(vec![
                recipient("a", "30"),
                recipient("b", "20"),
                recipient("c", "50"),
            ])],
            ..Feed::default()
        };
        let remove = apply(&mut feed, &wallet_of_two(), &RecipientOperation::Remove);
        assert!(remove.is_changed());
        assert_eq!(splits(&feed.values[0]), [pair("c", "100")]);
    }

    #[test]
    fn use_lnaddress_replaces_the_node_recipients_and_sets_the_method() {
        let mut ln = recipient("b", "20");
        ln.value.recipient_type = Some("lnaddress".into());
        let mut node = recipient("a", "30");
        node.value.custom_key = Some("k".into());
        node.value.custom_value = Some("v".into());
        node.value.fee = Some("false".into());
        let mut feed = Feed {
            values: vec![block(vec![node, ln, recipient("c", "50")])],
            ..Feed::default()
        };
        let operation = RecipientOperation::UseLnaddress {
            address: "b".into(),
        };
        let wallet = Wallet {
            key: RecipientKey {
                address: "a".into(),
                custom_key: Some("k".into()),
                custom_value: Some("v".into()),
            },
            members: vec![
                RecipientKey {
                    address: "a".into(),
                    custom_key: Some("k".into()),
                    custom_value: Some("v".into()),
                },
                key("b"),
            ],
        };
        assert!(apply(&mut feed, &wallet, &operation).is_changed());
        let block = &feed.values[0].value;
        assert_eq!(block.method.as_deref(), Some("lnaddress"));
        let first = &block.recipients[0].value;
        assert_eq!(first.recipient_type.as_deref(), Some("lnaddress"));
        assert_eq!(first.address.as_deref(), Some("b"));
        assert_eq!(first.custom_key, None);
        assert_eq!(first.custom_value, None);
        assert_eq!(first.name.as_deref(), Some("name a"));
        assert_eq!(first.split.as_deref(), Some("30"));
        assert_eq!(first.fee.as_deref(), Some("false"));
        // A second run has no node recipient of the wallet.
        let cause = apply(&mut feed, &wallet, &operation);
        assert_eq!(
            cause,
            Outcome::NotChanged {
                cause: Cause::NoNodeRecipient
            }
        );
    }

    #[test]
    fn a_block_with_method_lnaddress_keeps_its_method() {
        let mut feed = Feed {
            values: vec![block(vec![recipient("a", "100")])],
            ..Feed::default()
        };
        feed.values[0].value.method = Some("lnaddress".into());
        let operation = RecipientOperation::UseLnaddress {
            address: "x@y.z".into(),
        };
        assert!(apply(&mut feed, &wallet("a"), &operation).is_changed());
        assert_eq!(feed.values[0].value.method.as_deref(), Some("lnaddress"));
    }
}
