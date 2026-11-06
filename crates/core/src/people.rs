//! The people of one user (ADR 0098).
//!
//! A person has a name and links: wallets, Nostr keys, credit names, support
//! links and publisher feeds. The user makes each link. This module writes
//! no feed, and it keeps no secret. A value from a person is a start value
//! in a feed (ADR 0098 item 3).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::BuildHasher;
use std::slice::from_ref;
use std::thread;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::change_set::RecipientKey;
use crate::collection::{Collection, CollectionError, CollectionErrorKind, CreditRow, FeedId, now};
use crate::feed::{CreditPlace, Recipient};
use crate::keysend::{self, Lookup, LookupResult};
use crate::net::NetPolicy;
use crate::profiles;
use crate::web::Web;

/// The ID of a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PersonId(pub i64);

/// The key of a credit (ADR 0098 item 12): the `npub`, else the name with
/// the information address, else the name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identity {
    /// A Nostr key, trimmed and in lowercase.
    Npub(String),
    /// A normal name with a trimmed information address.
    NameHref {
        /// The normal name.
        name: String,
        /// The `href`.
        href: String,
    },
    /// A normal name.
    Name(String),
}

impl Identity {
    /// The identity of a credit, or `None` when the credit has no key, no
    /// name and no address.
    #[must_use]
    pub fn of(name: &str, href: &str, npub: &str) -> Option<Self> {
        let npub = npub.trim().to_lowercase();
        if !npub.is_empty() {
            return Some(Self::Npub(npub));
        }
        let name = normal_name(name);
        if name.is_empty() {
            return None;
        }
        let href = href.trim();
        if href.is_empty() {
            Some(Self::Name(name))
        } else {
            Some(Self::NameHref {
                name,
                href: href.to_owned(),
            })
        }
    }

    /// The text that the collection keeps: `npub:{npub}`,
    /// `href:{name}\u{1f}{href}` or `name:{name}`.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Npub(npub) => format!("npub:{npub}"),
            Self::NameHref { name, href } => format!("href:{name}\u{1f}{href}"),
            Self::Name(name) => format!("name:{name}"),
        }
    }
}

/// A name with no spaces at its two ends, one space for each series of
/// spaces, and lowercase letters. Two names are the same when their normal
/// names are equal.
#[must_use]
pub fn normal_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The proof of a link (ADR 0098 item 24): the statement of the user, the
/// keysend lookup or a Nostr signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkEvidence {
    /// The user said so.
    User,
    /// A keysend lookup gave the link.
    Keysend,
    /// A signed Nostr profile gave the link.
    Nostr,
}

impl LinkEvidence {
    /// The text that the collection keeps.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Keysend => "keysend",
            Self::Nostr => "nostr",
        }
    }
}

/// A wallet that the user linked to a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonWallet {
    /// The row ID.
    pub id: i64,
    /// The key that the user linked.
    pub key: RecipientKey,
    /// The recipient type, or empty text.
    pub recipient_type: String,
    /// The place in the list of the wallets of the person.
    pub position: i64,
    /// `user`, `keysend` or `nostr`.
    pub evidence: String,
    /// The Unix time of the link.
    pub linked_at: i64,
}

/// A Nostr key that the user linked to a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonKey {
    /// The row ID.
    pub id: i64,
    /// The `npub`, in lowercase.
    pub npub: String,
    /// `user`, `keysend` or `nostr`.
    pub evidence: String,
    /// The Unix time of the link.
    pub linked_at: i64,
}

/// A credit name of a person: the identity of a credit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreditName {
    /// The row ID.
    pub id: i64,
    /// [`Identity::key`] of the credit.
    pub identity: String,
    /// The trimmed name, or empty text.
    pub name: String,
    /// The trimmed `href`, or empty text.
    pub href: String,
    /// The trimmed `npub`, or empty text.
    pub npub: String,
    /// `user`, `keysend` or `nostr`.
    pub evidence: String,
    /// The Unix time of the link.
    pub linked_at: i64,
}

/// A support link of a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportLink {
    /// The row ID.
    pub id: i64,
    /// The label, or empty text.
    pub label: String,
    /// The URL.
    pub url: String,
    /// The place in the list of the support links.
    pub position: i64,
}

/// A person with all links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Person {
    /// The ID.
    pub id: PersonId,
    /// The name.
    pub name: String,
    /// The usual role, as raw text.
    pub usual_role: String,
    /// The usual group for the usual role, as raw text (ADR 0112 item 7).
    pub usual_group: String,
    /// The picture address, or empty text.
    pub img: String,
    /// The information address, or empty text.
    pub href: String,
    /// The default address that the user selected. Use [`default_address`]
    /// for the address that Studio uses.
    pub default_address: Option<RecipientKey>,
    /// The wallets, by `position`.
    pub wallets: Vec<PersonWallet>,
    /// The Nostr keys, A to Z.
    pub keys: Vec<PersonKey>,
    /// The credit names, A to Z by identity.
    pub credit_names: Vec<CreditName>,
    /// The support links, by `position`.
    pub support_links: Vec<SupportLink>,
    /// The publisher feeds of which the person is a member.
    pub members: Vec<FeedId>,
}

/// The fields of a person that the user edits.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct PersonFields {
    /// The name. A name of only spaces is refused.
    pub name: String,
    /// The usual role.
    pub usual_role: String,
    /// The usual group for the usual role.
    pub usual_group: String,
    /// The picture address.
    pub img: String,
    /// The information address.
    pub href: String,
}

fn missing(id: PersonId) -> CollectionError {
    CollectionError::with_kind(
        CollectionErrorKind::NotFound,
        format!("person {} does not exist", id.0),
    )
}

fn conflict(message: impl Into<String>) -> CollectionError {
    CollectionError::with_kind(CollectionErrorKind::Conflict, message)
}

fn parts(key: &RecipientKey) -> (String, String, String) {
    let (address, custom_key, custom_value) = key.parts();
    (
        address.to_owned(),
        custom_key.to_owned(),
        custom_value.to_owned(),
    )
}

pub(crate) fn lookup_map(
    collection: &Collection,
) -> Result<HashMap<String, Lookup>, CollectionError> {
    Ok(collection
        .keysend_lookups()?
        .into_iter()
        .map(|lookup| (lookup.address.clone(), lookup))
        .collect())
}

fn check_person(db: &Connection, id: PersonId) -> Result<(), CollectionError> {
    let found: bool = db.query_row(
        "SELECT EXISTS (SELECT 1 FROM people WHERE id = ?1)",
        [id.0],
        |r| r.get(0),
    )?;
    if found { Ok(()) } else { Err(missing(id)) }
}

/// Give `NotFound` when a delete changed no row.
fn check_removed(changed: usize, what: &str, id: i64) -> Result<(), CollectionError> {
    if changed == 0 {
        return Err(CollectionError::with_kind(
            CollectionErrorKind::NotFound,
            format!("{what} {id} does not exist"),
        ));
    }
    Ok(())
}

fn clean_fields(fields: &PersonFields) -> Result<PersonFields, CollectionError> {
    let name = fields.name.trim();
    if name.is_empty() {
        return Err(CollectionError::invalid("a person needs a name"));
    }
    Ok(PersonFields {
        name: name.to_owned(),
        usual_role: fields.usual_role.trim().to_owned(),
        usual_group: fields.usual_group.trim().to_owned(),
        img: fields.img.trim().to_owned(),
        href: fields.href.trim().to_owned(),
    })
}

fn insert_person(db: &Connection, fields: &PersonFields) -> Result<PersonId, CollectionError> {
    let fields = clean_fields(fields)?;
    db.execute(
        "INSERT INTO people (name, usual_role, usual_group, img, href, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            fields.name,
            fields.usual_role,
            fields.usual_group,
            fields.img,
            fields.href,
            now()
        ],
    )?;
    Ok(PersonId(db.last_insert_rowid()))
}

/// Add a person with no link.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a name of only spaces.
pub fn add_person(
    collection: &Collection,
    fields: &PersonFields,
) -> Result<PersonId, CollectionError> {
    insert_person(collection.db(), fields)
}

/// Change the fields of a person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a name of only spaces, and
/// [`CollectionErrorKind::NotFound`] for an ID that does not exist.
pub fn change_person(
    collection: &Collection,
    id: PersonId,
    fields: &PersonFields,
) -> Result<(), CollectionError> {
    update_fields(collection.db(), id, fields)
}

fn update_fields(
    db: &Connection,
    id: PersonId,
    fields: &PersonFields,
) -> Result<(), CollectionError> {
    let fields = clean_fields(fields)?;
    let changed = db.execute(
        "UPDATE people SET name = ?2, usual_role = ?3, usual_group = ?6, img = ?4, href = ?5 WHERE id = ?1",
        params![
            id.0,
            fields.name,
            fields.usual_role,
            fields.img,
            fields.href,
            fields.usual_group
        ],
    )?;
    if changed == 0 {
        return Err(missing(id));
    }
    Ok(())
}

/// Remove a person with each link.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] for an ID that does not exist.
pub fn remove_person(collection: &Collection, id: PersonId) -> Result<(), CollectionError> {
    let changed = collection
        .db()
        .execute("DELETE FROM people WHERE id = ?1", [id.0])?;
    if changed == 0 {
        return Err(missing(id));
    }
    Ok(())
}

/// Set the name of each person that has a wallet in `keys`. A name of only
/// spaces changes nothing (ADR 0098 item 23).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn rename_with_wallets(
    collection: &Collection,
    keys: &[RecipientKey],
    name: &str,
) -> Result<(), CollectionError> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(());
    }
    for key in keys {
        let (address, custom_key, custom_value) = key.parts();
        collection.db().execute(
            "UPDATE people SET name = ?1 WHERE id IN
                 (SELECT person_id FROM person_wallets
                  WHERE address = ?2 AND custom_key = ?3 AND custom_value = ?4)",
            params![name, address, custom_key, custom_value],
        )?;
    }
    Ok(())
}

/// The type of a wallet row from its address (ADR 0098 item 26). A Lightning
/// address has `@`, and a node key has none. A row from before this rule can
/// hold a different type, so a read also applies it. A row with no type stays.
fn wallet_type(address: &str, stored: &str) -> String {
    match stored.trim() {
        "lnaddress" | "node" if address.contains('@') => "lnaddress".to_owned(),
        "lnaddress" | "node" => "node".to_owned(),
        other => other.to_owned(),
    }
}

fn wallets_of(db: &Connection, id: PersonId) -> Result<Vec<PersonWallet>, CollectionError> {
    let mut statement = db.prepare(
        "SELECT id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at
         FROM person_wallets WHERE person_id = ?1 ORDER BY position, id",
    )?;
    let rows = statement.query_map([id.0], |r| {
        let custom_key: String = r.get(2)?;
        let custom_value: String = r.get(3)?;
        let address: String = r.get(1)?;
        let stored: String = r.get(4)?;
        Ok(PersonWallet {
            id: r.get(0)?,
            recipient_type: wallet_type(&address, &stored),
            key: RecipientKey {
                address,
                custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                custom_value: Some(custom_value).filter(|t| !t.is_empty()),
            },
            position: r.get(5)?,
            evidence: r.get(6)?,
            linked_at: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn read_person(db: &Connection, id: PersonId) -> Result<Person, CollectionError> {
    let mut person = db
        .query_row(
            "SELECT name, usual_role, img, href, default_address, default_custom_key, default_custom_value, usual_group
             FROM people WHERE id = ?1",
            [id.0],
            |r| {
                let address: Option<String> = r.get(4)?;
                let custom_key: String = r.get(5)?;
                let custom_value: String = r.get(6)?;
                Ok(Person {
                    id,
                    name: r.get(0)?,
                    usual_role: r.get(1)?,
                    usual_group: r.get(7)?,
                    img: r.get(2)?,
                    href: r.get(3)?,
                    default_address: address.map(|address| RecipientKey {
                        address,
                        custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                        custom_value: Some(custom_value).filter(|t| !t.is_empty()),
                    }),
                    wallets: Vec::new(),
                    keys: Vec::new(),
                    credit_names: Vec::new(),
                    support_links: Vec::new(),
                    members: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| missing(id))?;
    person.wallets = wallets_of(db, id)?;

    let mut statement = db.prepare(
        "SELECT id, npub, evidence, linked_at FROM person_keys WHERE person_id = ?1 ORDER BY npub",
    )?;
    person.keys = statement
        .query_map([id.0], |r| {
            Ok(PersonKey {
                id: r.get(0)?,
                npub: r.get(1)?,
                evidence: r.get(2)?,
                linked_at: r.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut statement = db.prepare(
        "SELECT id, identity, name, href, npub, evidence, linked_at
         FROM person_credit_names WHERE person_id = ?1 ORDER BY identity",
    )?;
    person.credit_names = statement
        .query_map([id.0], |r| {
            Ok(CreditName {
                id: r.get(0)?,
                identity: r.get(1)?,
                name: r.get(2)?,
                href: r.get(3)?,
                npub: r.get(4)?,
                evidence: r.get(5)?,
                linked_at: r.get(6)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut statement = db.prepare(
        "SELECT id, label, url, position FROM person_support_links
         WHERE person_id = ?1 ORDER BY position, id",
    )?;
    person.support_links = statement
        .query_map([id.0], |r| {
            Ok(SupportLink {
                id: r.get(0)?,
                label: r.get(1)?,
                url: r.get(2)?,
                position: r.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut statement =
        db.prepare("SELECT feed_id FROM person_members WHERE person_id = ?1 ORDER BY feed_id")?;
    person.members = statement
        .query_map([id.0], |r| Ok(FeedId(r.get(0)?)))?
        .collect::<Result<_, _>>()?;
    Ok(person)
}

/// Each person, A to Z by normal name and then by ID.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn people(collection: &Collection) -> Result<Vec<Person>, CollectionError> {
    let db = collection.db();
    let mut statement = db.prepare("SELECT id FROM people")?;
    let ids: Vec<i64> = statement
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    drop(statement);
    let mut list = Vec::new();
    for id in ids {
        list.push(read_person(db, PersonId(id))?);
    }
    list.sort_by(|a, b| {
        normal_name(&a.name)
            .cmp(&normal_name(&b.name))
            .then(a.id.0.cmp(&b.id.0))
    });
    Ok(list)
}

/// One person by ID.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] for an ID that does not exist.
pub fn person(collection: &Collection, id: PersonId) -> Result<Person, CollectionError> {
    read_person(collection.db(), id)
}

/// The person that has a wallet with the same wallet key as `key`.
fn owner_of_wallet<S: BuildHasher>(
    db: &Connection,
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<Option<PersonId>, CollectionError> {
    let wanted = parts(&keysend::wallet_key(key, lookups));
    let mut statement =
        db.prepare("SELECT person_id, address, custom_key, custom_value FROM person_wallets")?;
    let rows = statement.query_map([], |r| {
        let custom_key: String = r.get(2)?;
        let custom_value: String = r.get(3)?;
        Ok((
            PersonId(r.get(0)?),
            RecipientKey {
                address: r.get(1)?,
                custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                custom_value: Some(custom_value).filter(|t| !t.is_empty()),
            },
        ))
    })?;
    for row in rows {
        let (person, stored) = row?;
        if parts(&keysend::wallet_key(&stored, lookups)) == wanted {
            return Ok(Some(person));
        }
    }
    Ok(None)
}

/// The person that has a wallet with the same wallet key as `key`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn person_of_wallet<S: BuildHasher>(
    collection: &Collection,
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<Option<PersonId>, CollectionError> {
    owner_of_wallet(collection.db(), key, lookups)
}

fn insert_wallet<S: BuildHasher>(
    db: &Connection,
    id: PersonId,
    key: &RecipientKey,
    recipient_type: &str,
    evidence: LinkEvidence,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<i64, CollectionError> {
    check_person(db, id)?;
    if key.address.trim().is_empty() {
        return Err(CollectionError::invalid("a wallet needs an address"));
    }
    if let Some(owner) = owner_of_wallet(db, key, lookups)?
        && owner != id
    {
        return Err(conflict(format!(
            "the wallet is a wallet of person {} already",
            owner.0
        )));
    }
    let (address, custom_key, custom_value) = key.parts();
    let existing: Option<i64> = db
        .query_row(
            "SELECT id FROM person_wallets
             WHERE address = ?1 AND custom_key = ?2 AND custom_value = ?3",
            params![address, custom_key, custom_value],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let position: i64 = db.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM person_wallets WHERE person_id = ?1",
        [id.0],
        |r| r.get(0),
    )?;
    db.execute(
        "INSERT INTO person_wallets
             (person_id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id.0,
            address,
            custom_key,
            custom_value,
            wallet_type(address, recipient_type),
            position,
            evidence.as_str(),
            now()
        ],
    )?;
    Ok(db.last_insert_rowid())
}

/// Link a wallet to a person. The user makes the link. A key that is in the
/// person already gives its row ID.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Conflict`] when a different person has a
/// wallet with the same wallet key, [`CollectionErrorKind::Invalid`] for an
/// empty address and [`CollectionErrorKind::NotFound`] for an unknown person.
pub fn add_wallet(
    collection: &Collection,
    id: PersonId,
    key: &RecipientKey,
    recipient_type: &str,
    evidence: LinkEvidence,
) -> Result<i64, CollectionError> {
    let lookups = lookup_map(collection)?;
    insert_wallet(collection.db(), id, key, recipient_type, evidence, &lookups)
}

/// True when `key` is a member of a wallet of `person`.
fn is_member<S: BuildHasher>(
    person: &Person,
    key: &RecipientKey,
    lookups: &HashMap<String, Lookup, S>,
) -> bool {
    person.wallets.iter().any(|w| {
        keysend::wallet_of(&w.key, lookups)
            .1
            .iter()
            .any(|m| m.parts() == key.parts())
    })
}

/// A member of a wallet, for the menu of the default address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WalletMember {
    /// The key of the member.
    pub key: RecipientKey,
    /// `lnaddress` or `node`, or the type of the wallet row for its own key.
    pub recipient_type: String,
}

/// The members of `wallet` (ADR 0095 item 5). The list always holds
/// `wallet.key`, with the recipient type of the wallet row. Each other
/// member, and a wallet row with no type, has the type `lnaddress` for an
/// address with `@`, else `node`.
#[must_use]
pub fn wallet_members<S: BuildHasher>(
    wallet: &PersonWallet,
    lookups: &HashMap<String, Lookup, S>,
) -> Vec<WalletMember> {
    let type_of = |key: &RecipientKey| {
        if key.parts() == wallet.key.parts() && !wallet.recipient_type.is_empty() {
            wallet.recipient_type.clone()
        } else if key.address.contains('@') {
            "lnaddress".to_owned()
        } else {
            "node".to_owned()
        }
    };
    let wallet_key = keysend::wallet_key(&wallet.key, lookups);
    let mut members: Vec<WalletMember> = keysend::members(&wallet_key, lookups)
        .into_iter()
        .map(|key| {
            let recipient_type = type_of(&key);
            WalletMember {
                key,
                recipient_type,
            }
        })
        .collect();
    if !members.iter().any(|m| m.key.parts() == wallet.key.parts()) {
        members.insert(
            0,
            WalletMember {
                key: wallet.key.clone(),
                recipient_type: type_of(&wallet.key),
            },
        );
    }
    members
}

/// Remove a wallet of a person. The default address goes back to none when
/// no wallet of the person has it any more.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] when the person has no such
/// wallet.
pub fn remove_wallet(
    collection: &Collection,
    id: PersonId,
    wallet_id: i64,
) -> Result<(), CollectionError> {
    let lookups = lookup_map(collection)?;
    let tx = collection.db().unchecked_transaction()?;
    let changed = tx.execute(
        "DELETE FROM person_wallets WHERE id = ?1 AND person_id = ?2",
        params![wallet_id, id.0],
    )?;
    check_removed(changed, "wallet", wallet_id)?;
    let person = read_person(&tx, id)?;
    if let Some(default) = &person.default_address
        && !is_member(&person, default, &lookups)
    {
        tx.execute(
            "UPDATE people SET default_address = NULL, default_custom_key = '', default_custom_value = ''
             WHERE id = ?1",
            [id.0],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Change a wallet of a person (ADR 0141 items 6 to 8). The new key replaces the
/// whole wallet, also each address that the keysend lookup joined to it, and it
/// keeps the place of the wallet. A default address in the old wallet moves to
/// the new key. A change writes no feed.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] for an unknown wallet,
/// [`CollectionErrorKind::Invalid`] for an empty address and
/// [`CollectionErrorKind::Conflict`] when a different person has the new wallet.
pub fn change_wallet(
    collection: &Collection,
    id: PersonId,
    wallet_id: i64,
    key: &RecipientKey,
    recipient_type: &str,
) -> Result<i64, CollectionError> {
    let lookups = lookup_map(collection)?;
    let tx = collection.db().unchecked_transaction()?;
    let person = read_person(&tx, id)?;
    let old = person
        .wallets
        .iter()
        .find(|w| w.id == wallet_id)
        .ok_or_else(|| {
            CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("wallet {wallet_id} does not exist"),
            )
        })?;
    let had_default = person.default_address.as_ref().is_some_and(|default| {
        wallet_members(old, &lookups)
            .iter()
            .any(|m| m.key.parts() == default.parts())
    });
    tx.execute("DELETE FROM person_wallets WHERE id = ?1", [wallet_id])?;
    let new_id = insert_wallet(&tx, id, key, recipient_type, LinkEvidence::User, &lookups)?;
    tx.execute(
        "UPDATE person_wallets SET position = ?2 WHERE id = ?1",
        params![new_id, old.position],
    )?;
    if had_default {
        write_default_address(&tx, id, Some(key), &lookups)?;
    }
    tx.commit()?;
    Ok(new_id)
}

fn insert_key(db: &Connection, id: PersonId, npub: &str) -> Result<i64, CollectionError> {
    check_person(db, id)?;
    let npub = npub.trim().to_lowercase();
    if !npub.starts_with("npub1") {
        return Err(CollectionError::invalid("a Nostr key starts with npub1"));
    }
    let owner: Option<(i64, i64)> = db
        .query_row(
            "SELECT id, person_id FROM person_keys WHERE npub = ?1",
            [&npub],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match owner {
        Some((row, person)) if person == id.0 => Ok(row),
        Some((_, person)) => Err(conflict(format!(
            "the Nostr key is a key of person {person} already"
        ))),
        None => {
            db.execute(
                "INSERT INTO person_keys (person_id, npub, evidence, linked_at)
                 VALUES (?1, ?2, 'user', ?3)",
                params![id.0, npub, now()],
            )?;
            Ok(db.last_insert_rowid())
        }
    }
}

/// Link a Nostr key to a person. A key that is in the person already gives
/// its row ID.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a text that does not start
/// with `npub1`, and [`CollectionErrorKind::Conflict`] for the key of a
/// different person.
pub fn add_key(collection: &Collection, id: PersonId, npub: &str) -> Result<i64, CollectionError> {
    insert_key(collection.db(), id, npub)
}

/// Remove a Nostr key of a person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] when the person has no such key.
pub fn remove_key(
    collection: &Collection,
    id: PersonId,
    key_id: i64,
) -> Result<(), CollectionError> {
    let changed = collection.db().execute(
        "DELETE FROM person_keys WHERE id = ?1 AND person_id = ?2",
        params![key_id, id.0],
    )?;
    check_removed(changed, "Nostr key", key_id)
}

fn insert_credit_name(
    db: &Connection,
    id: PersonId,
    name: &str,
    href: &str,
    npub: &str,
) -> Result<i64, CollectionError> {
    check_person(db, id)?;
    let Some(identity) = Identity::of(name, href, npub) else {
        return Err(CollectionError::invalid(
            "a credit name needs a name or a Nostr key",
        ));
    };
    let key = identity.key();
    let owner: Option<(i64, i64)> = db
        .query_row(
            "SELECT id, person_id FROM person_credit_names WHERE identity = ?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match owner {
        Some((row, person)) if person == id.0 => Ok(row),
        Some((_, person)) => Err(conflict(format!(
            "the credit name is a credit name of person {person} already"
        ))),
        None => {
            db.execute(
                "INSERT INTO person_credit_names
                     (person_id, identity, name, href, npub, evidence, linked_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'user', ?6)",
                params![id.0, key, name.trim(), href.trim(), npub.trim(), now()],
            )?;
            Ok(db.last_insert_rowid())
        }
    }
}

/// Link a credit name to a person. A credit name that is in the person
/// already gives its row ID.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] when the credit has no identity,
/// and [`CollectionErrorKind::Conflict`] for the identity of a different
/// person.
pub fn add_credit_name(
    collection: &Collection,
    id: PersonId,
    name: &str,
    href: &str,
    npub: &str,
) -> Result<i64, CollectionError> {
    insert_credit_name(collection.db(), id, name, href, npub)
}

/// Change a credit name of a person (ADR 0146 item 6). A Nostr key that the
/// person holds with the old value of the credit name goes with that value.
/// No feed changes.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] when the credit has no identity,
/// [`CollectionErrorKind::Conflict`] when a different credit name has the
/// identity, and [`CollectionErrorKind::NotFound`] when the person has no such
/// credit name.
pub fn change_credit_name(
    collection: &Collection,
    id: PersonId,
    credit_name_id: i64,
    name: &str,
    href: &str,
    npub: &str,
) -> Result<(), CollectionError> {
    let db = collection.db();
    let Some(identity) = Identity::of(name, href, npub) else {
        return Err(CollectionError::invalid(
            "a credit name needs a name or a Nostr key",
        ));
    };
    let key = identity.key();
    let owner: Option<i64> = db
        .query_row(
            "SELECT id FROM person_credit_names WHERE identity = ?1 AND id <> ?2",
            params![key, credit_name_id],
            |r| r.get(0),
        )
        .optional()?;
    if owner.is_some() {
        return Err(conflict("a different credit name has these values"));
    }
    let old_npub: Option<String> = db
        .query_row(
            "SELECT npub FROM person_credit_names WHERE id = ?1 AND person_id = ?2",
            params![credit_name_id, id.0],
            |r| r.get(0),
        )
        .optional()?;
    let changed = db.execute(
        "UPDATE person_credit_names SET identity = ?1, name = ?2, href = ?3, npub = ?4
         WHERE id = ?5 AND person_id = ?6",
        params![
            key,
            name.trim(),
            href.trim(),
            npub.trim(),
            credit_name_id,
            id.0
        ],
    )?;
    check_removed(changed, "credit name", credit_name_id)?;
    if let Some(old) = old_npub.filter(|old| !old.is_empty() && old != npub.trim()) {
        db.execute(
            "DELETE FROM person_keys WHERE person_id = ?1 AND npub = ?2",
            params![id.0, old],
        )?;
    }
    Ok(())
}

/// Remove a credit name of a person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] when the person has no such
/// credit name.
pub fn remove_credit_name(
    collection: &Collection,
    id: PersonId,
    credit_name_id: i64,
) -> Result<(), CollectionError> {
    let changed = collection.db().execute(
        "DELETE FROM person_credit_names WHERE id = ?1 AND person_id = ?2",
        params![credit_name_id, id.0],
    )?;
    check_removed(changed, "credit name", credit_name_id)
}

/// The trimmed URL of a support link.
fn web_url(url: &str) -> Result<&str, CollectionError> {
    let url = url.trim();
    let lower = url.to_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(url)
    } else {
        Err(CollectionError::invalid(
            "a support link starts with https:// or http://",
        ))
    }
}

/// Change a support link of a person (ADR 0146 item 6). No feed changes.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a URL that does not start with
/// `https://` or `http://`, and [`CollectionErrorKind::NotFound`] when the
/// person has no such link.
pub fn change_support_link(
    collection: &Collection,
    id: PersonId,
    link_id: i64,
    label: &str,
    url: &str,
) -> Result<(), CollectionError> {
    let changed = collection.db().execute(
        "UPDATE person_support_links SET label = ?1, url = ?2 WHERE id = ?3 AND person_id = ?4",
        params![label.trim(), web_url(url)?, link_id, id.0],
    )?;
    check_removed(changed, "support link", link_id)
}

/// Add a support link to a person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a URL that does not start with
/// `https://` or `http://`.
pub fn add_support_link(
    collection: &Collection,
    id: PersonId,
    label: &str,
    url: &str,
) -> Result<i64, CollectionError> {
    let db = collection.db();
    check_person(db, id)?;
    let url = web_url(url)?;
    let position: i64 = db.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM person_support_links WHERE person_id = ?1",
        [id.0],
        |r| r.get(0),
    )?;
    db.execute(
        "INSERT INTO person_support_links (person_id, label, url, position)
         VALUES (?1, ?2, ?3, ?4)",
        params![id.0, label.trim(), url, position],
    )?;
    Ok(db.last_insert_rowid())
}

/// Remove a support link of a person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] when the person has no such link.
pub fn remove_support_link(
    collection: &Collection,
    id: PersonId,
    link_id: i64,
) -> Result<(), CollectionError> {
    let changed = collection.db().execute(
        "DELETE FROM person_support_links WHERE id = ?1 AND person_id = ?2",
        params![link_id, id.0],
    )?;
    check_removed(changed, "support link", link_id)
}

/// Make a person a member of a publisher feed. A member in the feed
/// already changes nothing.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a feed that is not a publisher
/// feed, and [`CollectionErrorKind::NotFound`] for an unknown feed.
pub fn add_member(
    collection: &Collection,
    id: PersonId,
    feed_id: FeedId,
) -> Result<(), CollectionError> {
    insert_member(collection.db(), id, feed_id)
}

fn insert_member(db: &Connection, id: PersonId, feed_id: FeedId) -> Result<(), CollectionError> {
    check_person(db, id)?;
    let medium: Option<Option<String>> = db
        .query_row("SELECT medium FROM feeds WHERE id = ?1", [feed_id.0], |r| {
            r.get(0)
        })
        .optional()?;
    match medium {
        None => Err(CollectionError::with_kind(
            CollectionErrorKind::NotFound,
            format!("feed {} does not exist", feed_id.0),
        )),
        Some(medium) if medium.as_deref() != Some("publisher") => Err(CollectionError::invalid(
            "only a publisher feed has members",
        )),
        Some(_) => {
            db.execute(
                "INSERT OR IGNORE INTO person_members (person_id, feed_id) VALUES (?1, ?2)",
                params![id.0, feed_id.0],
            )?;
            Ok(())
        }
    }
}

/// Remove a person from a publisher feed.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] when the person is not a member.
pub fn remove_member(
    collection: &Collection,
    id: PersonId,
    feed_id: FeedId,
) -> Result<(), CollectionError> {
    let changed = collection.db().execute(
        "DELETE FROM person_members WHERE person_id = ?1 AND feed_id = ?2",
        params![id.0, feed_id.0],
    )?;
    check_removed(changed, "membership of feed", feed_id.0)
}

/// Set the default address of a person, or remove it with `None`. The key
/// must be a member of a wallet of the person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a key that is in no wallet of
/// the person.
pub fn set_default_address(
    collection: &Collection,
    id: PersonId,
    key: Option<&RecipientKey>,
) -> Result<(), CollectionError> {
    let lookups = lookup_map(collection)?;
    write_default_address(collection.db(), id, key, &lookups)
}

fn write_default_address(
    db: &Connection,
    id: PersonId,
    key: Option<&RecipientKey>,
    lookups: &HashMap<String, Lookup>,
) -> Result<(), CollectionError> {
    let person = read_person(db, id)?;
    let (address, custom_key, custom_value) = match key {
        None => (None, "", ""),
        Some(key) => {
            if !is_member(&person, key, lookups) {
                return Err(CollectionError::invalid(
                    "the default address must be an address of a wallet of the person",
                ));
            }
            let (address, custom_key, custom_value) = key.parts();
            (Some(address), custom_key, custom_value)
        }
    };
    db.execute(
        "UPDATE people SET default_address = ?2, default_custom_key = ?3, default_custom_value = ?4
         WHERE id = ?1",
        params![id.0, address, custom_key, custom_value],
    )?;
    Ok(())
}

/// The pending changes of the page of a person (ADR 0116). `None` means
/// no change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct PersonSave {
    /// New fields.
    pub fields: Option<PersonFields>,
    /// A new default address. `Some(None)` removes it.
    #[serde(deserialize_with = "double_option")]
    pub default_address: Option<Option<RecipientKey>>,
    /// The complete list of links to publisher feeds (ADR 0132 item 9). An
    /// empty list removes each link.
    pub members: Option<Vec<FeedId>>,
    /// Each wallet to link to the person (ADR 0117 item 7).
    pub wallets: Vec<WalletAdd>,
    /// Each Nostr key to link to the person (ADR 0117 item 7).
    pub keys: Vec<String>,
}

/// A wallet that the save links to a person (ADR 0117 item 7).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WalletAdd {
    /// The wallet key.
    pub key: RecipientKey,
    /// The recipient type, such as `node` or `lnaddress`.
    pub recipient_type: String,
}

/// Read a present value, also `null`, as `Some`. A missing key stays `None`
/// through `#[serde(default)]`.
#[expect(
    clippy::option_option,
    reason = "ADR 0116: no change, remove and set are three different cases"
)]
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Apply the pending changes of a person in one transaction (ADR 0116
/// item 5). A failure of one part changes nothing. The save writes no feed.
/// The wallets and the keys are linked with the evidence of the user, before
/// the default address (ADR 0117 item 7).
///
/// # Errors
///
/// Gives the error of the first part that fails: [`CollectionErrorKind::Invalid`]
/// for a name of only spaces, a default address that is in no wallet, a
/// feed that is not a publisher feed, or a wallet or key that is not valid,
/// [`CollectionErrorKind::Conflict`] for a wallet or a key of a different
/// person, and [`CollectionErrorKind::NotFound`] for an unknown person or
/// feed.
pub fn save_person(
    collection: &Collection,
    id: PersonId,
    save: &PersonSave,
) -> Result<(), CollectionError> {
    let lookups = lookup_map(collection)?;
    let tx = collection.db().unchecked_transaction()?;
    check_person(&tx, id)?;
    if let Some(fields) = &save.fields {
        update_fields(&tx, id, fields)?;
    }
    if let Some(members) = &save.members {
        tx.execute("DELETE FROM person_members WHERE person_id = ?1", [id.0])?;
        for feed in members {
            insert_member(&tx, id, *feed)?;
        }
    }
    for wallet in &save.wallets {
        insert_wallet(
            &tx,
            id,
            &wallet.key,
            &wallet.recipient_type,
            LinkEvidence::User,
            &lookups,
        )?;
    }
    for npub in &save.keys {
        insert_key(&tx, id, npub)?;
    }
    if let Some(key) = &save.default_address {
        write_default_address(&tx, id, key.as_ref(), &lookups)?;
    }
    tx.commit()?;
    Ok(())
}

/// The default address of a person: the stored default when a wallet of the
/// person still has it, else the key of the first wallet by `position`
/// (ADR 0098 item 7).
#[must_use]
pub fn default_address<S: BuildHasher>(
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
) -> Option<RecipientKey> {
    person
        .default_address
        .clone()
        .filter(|key| is_member(person, key, lookups))
        .or_else(|| person.wallets.first().map(|w| w.key.clone()))
}

/// Save a recipient as a person (ADR 0027 item 5, ADR 0098). When a person
/// has the wallet of the recipient, nothing changes and the result is that
/// person. Else the core makes a person with the name of the recipient, or
/// its address, and one wallet.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] when the recipient has no address.
pub fn save_recipient(
    collection: &Collection,
    recipient: &Recipient,
) -> Result<PersonId, CollectionError> {
    // The same rule as `Feed::recipients`: an address of only spaces is no
    // address.
    let Some(address) = recipient
        .address
        .as_deref()
        .filter(|a| !a.trim().is_empty())
    else {
        return Err(CollectionError::invalid(
            "a saved recipient needs an address",
        ));
    };
    let key = RecipientKey {
        address: address.to_owned(),
        custom_key: recipient.custom_key.clone().filter(|t| !t.is_empty()),
        custom_value: recipient.custom_value.clone().filter(|t| !t.is_empty()),
    };
    let lookups = lookup_map(collection)?;
    let tx = collection.db().unchecked_transaction()?;
    if let Some(owner) = owner_of_wallet(&tx, &key, &lookups)? {
        return Ok(owner);
    }
    let name = recipient
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(address);
    let id = insert_person(
        &tx,
        &PersonFields {
            name: name.to_owned(),
            ..PersonFields::default()
        },
    )?;
    insert_wallet(
        &tx,
        id,
        &key,
        recipient.recipient_type.as_deref().unwrap_or(""),
        LinkEvidence::User,
        &lookups,
    )?;
    tx.commit()?;
    Ok(id)
}

/// How a recipient joined a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinedBy {
    /// The normal name of the recipient is the normal name of the group.
    Name,
    /// The wallet key of the recipient is the wallet key of a recipient
    /// that joined by name.
    Wallet,
}

/// The reason to join an item to a suggestion (ADR 0098 item 13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    /// The identity of the suggestion is an `npub`.
    SameNpub,
    /// A recipient joined by name.
    SameName,
    /// A recipient joined by wallet.
    SameWallet,
}

/// A distinct credit `(name, href, npub)` of a suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuggestedCredit {
    /// The trimmed name.
    pub name: String,
    /// The trimmed `href`, or empty text.
    pub href: String,
    /// The trimmed `npub`, or empty text.
    pub npub: String,
    /// The first `img` that is not empty, or empty text.
    pub img: String,
    /// The distinct raw roles that are not empty, A to Z.
    pub roles: Vec<String>,
    /// The feeds that have the credit, by ID.
    pub feeds: Vec<FeedId>,
    /// False when the normal name is not the normal name of the suggestion.
    pub same_name: bool,
}

/// A distinct recipient key of a suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuggestedRecipient {
    /// The key of the recipient.
    pub key: RecipientKey,
    /// The name from the most recent read, or empty text.
    pub name: String,
    /// The recipient type from the most recent read, or empty text.
    pub recipient_type: String,
    /// The feeds that have the recipient, by ID.
    pub feeds: Vec<FeedId>,
    /// False when the normal name is not the normal name of the suggestion.
    pub same_name: bool,
    /// How the recipient joined.
    pub joined_by: JoinedBy,
}

/// The items of the feeds that can be one person (ADR 0098 items 12 and 13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    /// [`Identity::key`] of the group.
    pub identity: String,
    /// The most frequent name of the credits.
    pub name: String,
    /// The credits, A to Z.
    pub credits: Vec<SuggestedCredit>,
    /// The recipients, A to Z by key.
    pub recipients: Vec<SuggestedRecipient>,
    /// The number of distinct feeds of the credits.
    pub credit_feeds: usize,
    /// The number of distinct feeds of the recipients.
    pub payee_feeds: usize,
    /// `SameNpub`, `SameName` and `SameWallet`, in this sequence.
    pub evidence: Vec<Evidence>,
    /// The key of each item, for [`dismiss`].
    pub item_keys: Vec<String>,
    /// The person whose normal name, or one of whose credit names, has the
    /// normal name of the suggestion (ADR 0111 item 6). With two matches,
    /// the person with the lowest ID.
    pub person_id: Option<PersonId>,
}

/// The items that the user selects to make a person (ADR 0098 item 14).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MakePerson {
    /// The name of the person.
    pub name: String,
    /// The selected credits: name, `href` and `npub`.
    pub credits: Vec<(String, String, String)>,
    /// The selected recipients, each with its recipient type.
    pub recipients: Vec<(RecipientKey, String)>,
    /// The selected Nostr keys, more than the keys of the credits.
    pub keys: Vec<String>,
    /// The publisher feed of which the person is a member.
    pub member_of: Option<FeedId>,
}

const SEP: char = '\u{1f}';

fn credit_item_key(name: &str, href: &str, npub: &str) -> String {
    format!("credit:{name}{SEP}{href}{SEP}{npub}")
}

fn recipient_item_key(key: &RecipientKey) -> String {
    let (address, custom_key, custom_value) = key.parts();
    format!("recipient:{address}{SEP}{custom_key}{SEP}{custom_value}")
}

fn strings(db: &Connection, sql: &str) -> Result<Vec<String>, CollectionError> {
    let mut statement = db.prepare(sql)?;
    Ok(statement
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}

fn feed_ids(feeds: &BTreeSet<i64>) -> Vec<FeedId> {
    feeds.iter().copied().map(FeedId).collect()
}

#[derive(Default)]
struct CreditAgg {
    img: String,
    roles: BTreeSet<String>,
    feeds: BTreeSet<i64>,
}

struct Group {
    /// The identity key of the suggestion (ADR 0103 item 5).
    key: String,
    /// The distinct `npub` values of the credits, trimmed and in lowercase.
    npubs: BTreeSet<String>,
    credits: BTreeMap<(String, String, String), CreditAgg>,
    /// The feeds of each name, for the most frequent name.
    names: BTreeMap<String, BTreeSet<i64>>,
}

impl Group {
    fn new(key: String) -> Self {
        Self {
            key,
            npubs: BTreeSet::new(),
            credits: BTreeMap::new(),
            names: BTreeMap::new(),
        }
    }

    fn add(&mut self, row: CreditRow) {
        let npub = row.npub.trim().to_lowercase();
        if !npub.is_empty() {
            self.npubs.insert(npub);
        }
        // A credit with only an `npub` gives no name to its group.
        if !row.name.trim().is_empty() {
            self.names
                .entry(row.name.trim().to_owned())
                .or_default()
                .insert(row.feed_id.0);
        }
        let credit = self
            .credits
            .entry((row.name, row.href, row.npub))
            .or_default();
        if credit.img.is_empty() {
            credit.img = row.img;
        }
        if !row.role.is_empty() {
            credit.roles.insert(row.role);
        }
        credit.feeds.insert(row.feed_id.0);
    }
}

struct RecipientAgg {
    key: RecipientKey,
    name: String,
    recipient_type: String,
    seen_at: i64,
    feeds: BTreeSet<i64>,
    wallet: (String, String, String),
}

fn recipient_aggs<S: BuildHasher>(
    db: &Connection,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<Vec<RecipientAgg>, CollectionError> {
    let mut statement = db.prepare(
        "SELECT feed_id, address, custom_key, custom_value, name, recipient_type, seen_at
         FROM feed_recipients ORDER BY address, custom_key, custom_value, feed_id",
    )?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;
    let mut found: BTreeMap<(String, String, String), RecipientAgg> = BTreeMap::new();
    for row in rows {
        let (feed_id, address, custom_key, custom_value, name, recipient_type, seen_at) = row?;
        let id = (address.clone(), custom_key.clone(), custom_value.clone());
        let agg = found.entry(id).or_insert_with(|| {
            let key = RecipientKey {
                address,
                custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                custom_value: Some(custom_value).filter(|t| !t.is_empty()),
            };
            RecipientAgg {
                wallet: parts(&keysend::wallet_key(&key, lookups)),
                key,
                name: String::new(),
                recipient_type: String::new(),
                seen_at: i64::MIN,
                feeds: BTreeSet::new(),
            }
        });
        agg.feeds.insert(feed_id);
        if seen_at >= agg.seen_at {
            agg.seen_at = seen_at;
            name.unwrap_or_default().trim().clone_into(&mut agg.name);
            recipient_type
                .unwrap_or_default()
                .trim()
                .clone_into(&mut agg.recipient_type);
        }
    }
    Ok(found.into_values().collect())
}

fn owned_wallets<S: BuildHasher>(
    db: &Connection,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<HashSet<(String, String, String)>, CollectionError> {
    let mut statement =
        db.prepare("SELECT address, custom_key, custom_value FROM person_wallets")?;
    let rows = statement.query_map([], |r| {
        let custom_key: String = r.get(1)?;
        let custom_value: String = r.get(2)?;
        Ok(RecipientKey {
            address: r.get(0)?,
            custom_key: Some(custom_key).filter(|t| !t.is_empty()),
            custom_value: Some(custom_value).filter(|t| !t.is_empty()),
        })
    })?;
    let mut set = HashSet::new();
    for row in rows {
        set.insert(parts(&keysend::wallet_key(&row?, lookups)));
    }
    Ok(set)
}

fn dismissals(db: &Connection) -> Result<HashMap<String, HashSet<String>>, CollectionError> {
    let mut statement = db.prepare("SELECT identity, items FROM person_dismissals")?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut map = HashMap::new();
    for row in rows {
        let (identity, items) = row?;
        // A row that is not a list of texts dismisses nothing.
        let items: Vec<String> = serde_json::from_str(&items).unwrap_or_default();
        map.insert(identity, items.into_iter().collect());
    }
    Ok(map)
}

/// Group the credits as ADR 0103 items 1 to 3 say. Two or more `npub` values
/// with one normal name make one group for each `npub`, and one group for the
/// credits of that name with no `npub`.
fn credit_groups(
    collection: &Collection,
    known_identities: &HashSet<String>,
    known_keys: &HashSet<String>,
) -> Result<Vec<Group>, CollectionError> {
    let mut by_name: BTreeMap<String, Vec<CreditRow>> = BTreeMap::new();
    let mut unnamed: Vec<CreditRow> = Vec::new();
    for row in collection.feed_credits()? {
        let Some(identity) = Identity::of(&row.name, &row.href, &row.npub) else {
            continue;
        };
        if known_identities.contains(&identity.key())
            || matches!(&identity, Identity::Npub(npub) if known_keys.contains(npub))
        {
            continue;
        }
        let name = normal_name(&row.name);
        if name.is_empty() {
            unnamed.push(row);
        } else {
            by_name.entry(name).or_default().push(row);
        }
    }

    let mut groups: Vec<Group> = Vec::new();
    for (name, rows) in by_name {
        let npubs: BTreeSet<String> = rows
            .iter()
            .map(|r| r.npub.trim().to_lowercase())
            .filter(|n| !n.is_empty())
            .collect();
        let mut split: BTreeMap<String, Group> = BTreeMap::new();
        for row in rows {
            let npub = row.npub.trim().to_lowercase();
            let key = if npubs.len() >= 2 && !npub.is_empty() {
                format!("npub:{npub}")
            } else {
                format!("name:{name}")
            };
            split
                .entry(key.clone())
                .or_insert_with(|| Group::new(key))
                .add(row);
        }
        groups.extend(split.into_values());
    }
    // A credit with an `npub` and no name joins the group of that `npub`.
    // Else its `npub` makes a group.
    for row in unnamed {
        let npub = row.npub.trim().to_lowercase();
        if let Some(group) = groups.iter_mut().find(|g| g.npubs.contains(&npub)) {
            group.add(row);
        } else {
            let mut group = Group::new(format!("npub:{npub}"));
            group.add(row);
            groups.push(group);
        }
    }
    Ok(groups)
}

/// The recipients that join a group: first by name, then by the wallet key
/// of a recipient that joined by name.
fn join_recipients(recipients: &[RecipientAgg], group_name: &str) -> Vec<SuggestedRecipient> {
    // A group with no name gets no recipient by name, so also none by wallet.
    let by_name: Vec<bool> = recipients
        .iter()
        .map(|r| !group_name.is_empty() && normal_name(&r.name) == group_name)
        .collect();
    let name_wallets: HashSet<&(String, String, String)> = recipients
        .iter()
        .zip(&by_name)
        .filter(|(_, joined)| **joined)
        .map(|(r, _)| &r.wallet)
        .collect();
    let mut joined = Vec::new();
    for (agg, by_name) in recipients.iter().zip(&by_name) {
        let joined_by = if *by_name {
            JoinedBy::Name
        } else if name_wallets.contains(&agg.wallet) {
            JoinedBy::Wallet
        } else {
            continue;
        };
        joined.push(SuggestedRecipient {
            key: agg.key.clone(),
            name: agg.name.clone(),
            recipient_type: agg.recipient_type.clone(),
            feeds: feed_ids(&agg.feeds),
            same_name: normal_name(&agg.name) == group_name,
            joined_by,
        });
    }
    joined
}

/// The name of each group, its normal name, and whether a payee can join it
/// (ADR 0103 item 4). A payee joins the group with no `npub` when a name has
/// more than one group, else it joins each group of the name.
fn plan_groups(groups: &[Group]) -> Vec<(Option<String>, String, bool)> {
    let names: Vec<(Option<String>, String)> = groups
        .iter()
        .map(|group| {
            // The most frequent name. With a tie, the first name A to Z.
            let name = group
                .names
                .iter()
                .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| b.0.cmp(a.0)))
                .map(|(name, _)| name.clone());
            let group_name = name.as_deref().map(normal_name).unwrap_or_default();
            (name, group_name)
        })
        .collect();
    groups
        .iter()
        .enumerate()
        .zip(&names)
        .map(|((i, group), (name, group_name))| {
            let same: Vec<usize> = (0..groups.len())
                .filter(|j| names[*j].1 == names[i].1)
                .collect();
            let pays = same.len() == 1
                || !same.iter().any(|j| groups[*j].npubs.is_empty())
                || group.npubs.is_empty();
            (name.clone(), group_name.clone(), pays)
        })
        .collect()
}

/// The feeds that use a person (ADR 0099 item 6).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PersonUses {
    /// Each feed whose recipients have a member of a wallet of the person.
    pub pays: Vec<FeedId>,
    /// Each feed whose credits have a credit name or a Nostr key of the
    /// person.
    pub credited: Vec<FeedId>,
    /// The places of the person in each feed of `pays` and `credited`, by
    /// feed ID (ADR 0099 item 5).
    pub places: Vec<FeedPlaces>,
}

/// The places of a person in one feed (ADR 0099 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedPlaces {
    /// The feed.
    pub feed_id: FeedId,
    /// Each address of the person in the value blocks of the feed, each one
    /// time. The keys are the keys that the feed has.
    pub pays: Vec<RecipientKey>,
    /// Each role of a credit of the person in the feed, each one time, A to
    /// Z. A credit with no role gives empty text.
    pub roles: Vec<String>,
    /// The places of each address of `pays`, in the same sequence (ADR 0143).
    pub pay_places: Vec<Vec<CreditPlace>>,
    /// Each place of the credits of `roles` (ADR 0143).
    pub role_places: Vec<CreditPlace>,
}

impl FeedPlaces {
    fn new(feed_id: FeedId) -> Self {
        Self {
            feed_id,
            pays: Vec::new(),
            roles: Vec::new(),
            pay_places: Vec::new(),
            role_places: Vec::new(),
        }
    }
}

/// The feeds that pay a wallet of `person` and the feeds that credit the
/// person. This function reads the index of the collection and makes no
/// request.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn uses<S: BuildHasher>(
    collection: &Collection,
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<PersonUses, CollectionError> {
    let wallets: HashSet<(String, String, String)> = person
        .wallets
        .iter()
        .map(|w| parts(&keysend::wallet_key(&w.key, lookups)))
        .collect();
    let mut pays = BTreeSet::new();
    let mut places: BTreeMap<i64, FeedPlaces> = BTreeMap::new();
    if !wallets.is_empty() {
        let mut statement = collection.db().prepare(
            "SELECT feed_id, address, custom_key, custom_value, places FROM feed_recipients
             ORDER BY feed_id, address, custom_key, custom_value",
        )?;
        let rows = statement.query_map([], |r| {
            let custom_key: String = r.get(2)?;
            let custom_value: String = r.get(3)?;
            Ok((
                r.get::<_, i64>(0)?,
                RecipientKey {
                    address: r.get(1)?,
                    custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                    custom_value: Some(custom_value).filter(|t| !t.is_empty()),
                },
                r.get::<_, String>(4)?,
            ))
        })?;
        for row in rows {
            let (feed, key, found) = row?;
            if wallets.contains(&parts(&keysend::wallet_key(&key, lookups))) {
                pays.insert(feed);
                let place = places
                    .entry(feed)
                    .or_insert_with(|| FeedPlaces::new(FeedId(feed)));
                if !place.pays.contains(&key) {
                    place.pays.push(key);
                    // A row with places that are not correct gives no places.
                    place
                        .pay_places
                        .push(serde_json::from_str(&found).unwrap_or_default());
                }
            }
        }
    }

    let identities: HashSet<&str> = person
        .credit_names
        .iter()
        .map(|c| c.identity.as_str())
        .collect();
    let npubs: HashSet<&str> = person.keys.iter().map(|k| k.npub.as_str()).collect();
    let mut credited = BTreeSet::new();
    for row in collection.feed_credits()? {
        let named = Identity::of(&row.name, &row.href, &row.npub)
            .is_some_and(|i| identities.contains(i.key().as_str()));
        let keyed = !row.npub.is_empty() && npubs.contains(row.npub.to_lowercase().as_str());
        if named || keyed {
            credited.insert(row.feed_id.0);
            let place = places
                .entry(row.feed_id.0)
                .or_insert_with(|| FeedPlaces::new(row.feed_id));
            place.roles.push(row.role.trim().to_owned());
            for found in row.places {
                if !place.role_places.contains(&found) {
                    place.role_places.push(found);
                }
            }
        }
    }
    for place in places.values_mut() {
        place.roles.sort();
        place.roles.dedup();
    }
    Ok(PersonUses {
        pays: feed_ids(&pays),
        credited: feed_ids(&credited),
        places: places.into_values().collect(),
    })
}

/// The state of a feed in the check of a person (ADR 0111 item 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckState {
    /// The feed has at least one finding that is not neutral.
    NeedsCorrection,
    /// The feed has only the neutral finding `not_default`.
    NotDefault,
    /// The feed agrees with the person.
    Correct,
}

/// One difference between a feed and a person (ADR 0111 item 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Finding {
    /// A credit of the person has no key, or a key that the person does not
    /// have.
    CreditKey {
        /// The role of the credit, or empty text.
        role: String,
        /// The group of the credit, or empty text.
        group: String,
        /// The `npub` of the credit, or empty text.
        npub: String,
        /// The places of the credit (ADR 0120 item 2).
        places: Vec<CreditPlace>,
    },
    /// A credit of the person has a picture or an information address that
    /// differs from the value of the person.
    CreditDetails {
        /// The role of the credit, or empty text.
        role: String,
        /// True when the picture differs.
        img: bool,
        /// True when the information address differs.
        href: bool,
        /// The picture of the feed, or empty text (ADR 0120 item 5).
        feed_img: String,
        /// The information address of the feed, or empty text.
        feed_href: String,
        /// The picture of the person, or empty text.
        person_img: String,
        /// The information address of the person, or empty text.
        person_href: String,
        /// The places of the credit (ADR 0120 item 2).
        places: Vec<CreditPlace>,
    },
    /// A credit of the person has no role.
    CreditRole {
        /// The places of the credit (ADR 0120 item 6).
        places: Vec<CreditPlace>,
    },
    /// A recipient of a wallet of the person does not use the default
    /// address. This finding is neutral.
    NotDefault {
        /// The key of the recipient.
        key: RecipientKey,
    },
    /// A recipient has the name of the person, but not a wallet of the
    /// person.
    OtherWallet {
        /// The key of the recipient.
        key: RecipientKey,
    },
}

/// The check of one feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedCheck {
    /// The feed.
    pub feed_id: FeedId,
    /// The state, from `findings`.
    pub state: CheckState,
    /// The differences. A `correct` feed has none.
    pub findings: Vec<Finding>,
}

/// The check of the feeds of a person, by feed ID.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PersonCheck {
    /// Each feed that pays a wallet of the person, credits the person or has
    /// a finding.
    pub feeds: Vec<FeedCheck>,
}

impl PersonCheck {
    /// The number of feeds with the state `needs_correction`.
    #[must_use]
    pub fn needs_correction(&self) -> usize {
        self.feeds
            .iter()
            .filter(|f| f.state == CheckState::NeedsCorrection)
            .count()
    }
}

/// The state of a feed from its findings.
fn check_state(findings: &[Finding]) -> CheckState {
    if findings
        .iter()
        .any(|f| !matches!(f, Finding::NotDefault { .. }))
    {
        CheckState::NeedsCorrection
    } else if findings.is_empty() {
        CheckState::Correct
    } else {
        CheckState::NotDefault
    }
}

/// The credits of a person (ADR 0111 item 10). A credit of the person has a
/// credit name or a key of the person, or the normal name of the person or
/// of a credit name. `person_names` holds those normal names.
fn person_credits(
    collection: &Collection,
    person: &Person,
    person_names: &HashSet<String>,
) -> Result<Vec<CreditRow>, CollectionError> {
    let identities: HashSet<&str> = person
        .credit_names
        .iter()
        .map(|c| c.identity.as_str())
        .collect();
    let npubs: HashSet<&str> = person.keys.iter().map(|k| k.npub.as_str()).collect();
    Ok(collection
        .feed_credits()?
        .into_iter()
        .filter(|row| {
            let npub = row.npub.to_lowercase();
            let named = Identity::of(&row.name, &row.href, &row.npub)
                .is_some_and(|i| identities.contains(i.key().as_str()))
                || person_names.contains(&normal_name(&row.name));
            named || (!npub.is_empty() && npubs.contains(npub.as_str()))
        })
        .collect())
}

/// The normal names of a person and of the credit names of the person.
fn person_normal_names(person: &Person) -> HashSet<String> {
    std::iter::once(person.name.as_str())
        .chain(person.credit_names.iter().map(|c| c.name.as_str()))
        .map(normal_name)
        .filter(|n| !n.is_empty())
        .collect()
}

/// A person with a membership of a publisher feed, or with a credit in its
/// channel (ADR 0118 item 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedMember {
    /// The ID of the person.
    pub person_id: PersonId,
    /// The name of the person.
    pub name: String,
    /// The picture address of the person, or empty text.
    pub img: String,
    /// The usual role of the person, as raw text.
    pub usual_role: String,
    /// True when the person is a member of the feed in Studio.
    pub member: bool,
    /// True when the channel of the feed has a credit of the person.
    pub in_credits: bool,
}

/// The members of a publisher feed, by name (ADR 0118 item 9). A feed that
/// is not a publisher feed has no member.
///
/// `feed_credits` has no column for the place of a credit. Each indexed
/// credit of a publisher feed therefore counts as a channel credit.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] for a feed that does not exist.
pub fn feed_members(
    collection: &Collection,
    feed_id: FeedId,
) -> Result<Vec<FeedMember>, CollectionError> {
    if collection.feed(feed_id)?.medium.as_deref() != Some("publisher") {
        return Ok(Vec::new());
    }
    let mut list = Vec::new();
    for person in people(collection)? {
        let member = person.members.contains(&feed_id);
        let in_credits = person_credits(collection, &person, &person_normal_names(&person))?
            .iter()
            .any(|row| row.feed_id == feed_id);
        if member || in_credits {
            list.push(FeedMember {
                person_id: person.id,
                name: person.name,
                img: person.img,
                usual_role: person.usual_role,
                member,
                in_credits,
            });
        }
    }
    Ok(list)
}

/// Where a credit name of a person is in the feeds (ADR 0140 item 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreditNameUse {
    /// The ID of the credit name.
    pub id: i64,
    /// The number of feeds with a credit of this identity.
    pub feeds: usize,
    /// Each role of those credits, in sequence, with no empty role.
    pub roles: Vec<String>,
}

/// The use of each credit name of `person`, in the sequence of the credit
/// names. The function reads only the credit index.
///
/// # Errors
///
/// Gives a [`CollectionError`] when the collection cannot be read.
pub fn credit_name_uses(
    collection: &Collection,
    person: &Person,
) -> Result<Vec<CreditNameUse>, CollectionError> {
    let mut found: HashMap<String, (HashSet<i64>, BTreeSet<String>)> = HashMap::new();
    for row in collection.feed_credits()? {
        let Some(identity) = Identity::of(&row.name, &row.href, &row.npub) else {
            continue;
        };
        let (feeds, roles) = found.entry(identity.key()).or_default();
        feeds.insert(row.feed_id.0);
        if !row.role.is_empty() {
            roles.insert(row.role);
        }
    }
    Ok(person
        .credit_names
        .iter()
        .map(|credit| {
            let (feeds, roles) = found.get(&credit.identity).cloned().unwrap_or_default();
            CreditNameUse {
                id: credit.id,
                feeds: feeds.len(),
                roles: roles.into_iter().collect(),
            }
        })
        .collect())
}

/// The publisher feeds of `person` that have a credit of the person (ADR 0132
/// item 4). The test is the test of [`feed_members`].
///
/// # Errors
///
/// Gives a [`CollectionError`] when the collection cannot be read.
pub fn linked_credits(
    collection: &Collection,
    person: &Person,
) -> Result<Vec<FeedId>, CollectionError> {
    let credits = person_credits(collection, person, &person_normal_names(person))?;
    Ok(person
        .members
        .iter()
        .copied()
        .filter(|feed| credits.iter().any(|row| row.feed_id == *feed))
        .collect())
}

/// The findings of the credits of a person, with the feed of each. A credit
/// with no finding gives the feed and `None`.
fn credit_findings(
    collection: &Collection,
    person: &Person,
    person_names: &HashSet<String>,
) -> Result<Vec<(i64, Option<Finding>)>, CollectionError> {
    let npubs: HashSet<&str> = person.keys.iter().map(|k| k.npub.as_str()).collect();
    let img = person.img.trim();
    let href = person.href.trim();
    let mut found = Vec::new();
    for row in person_credits(collection, person, person_names)? {
        let npub = row.npub.to_lowercase();
        let feed = row.feed_id.0;
        found.push((feed, None));
        if !npubs.is_empty() && !npubs.contains(npub.as_str()) {
            found.push((
                feed,
                Some(Finding::CreditKey {
                    role: row.role.clone(),
                    group: row.group.clone(),
                    npub: row.npub.clone(),
                    places: row.places.clone(),
                }),
            ));
        }
        let img_differs = !img.is_empty() && row.img != img;
        let href_differs = !href.is_empty() && row.href != href;
        if img_differs || href_differs {
            found.push((
                feed,
                Some(Finding::CreditDetails {
                    role: row.role.clone(),
                    img: img_differs,
                    href: href_differs,
                    feed_img: row.img.clone(),
                    feed_href: row.href.clone(),
                    person_img: img.to_owned(),
                    person_href: href.to_owned(),
                    places: row.places.clone(),
                }),
            ));
        }
        if row.role.is_empty() {
            found.push((
                feed,
                Some(Finding::CreditRole {
                    places: row.places.clone(),
                }),
            ));
        }
    }
    Ok(found)
}

/// Compare each feed with `person` (ADR 0111 items 9 and 10). A credit of
/// the person has a credit name or a key of the person, or the normal name
/// of the person or of a credit name. This function reads the index of the
/// collection and makes no request.
///
/// The check of payees needs a default address, because the correction of a
/// payee writes it.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn check<S: BuildHasher>(
    collection: &Collection,
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<PersonCheck, CollectionError> {
    let wallets: HashSet<(String, String, String)> = person
        .wallets
        .iter()
        .map(|w| parts(&keysend::wallet_key(&w.key, lookups)))
        .collect();
    let default = default_address(person, lookups).map(|key| parts(&key));
    let names = person_normal_names(person);
    let mut found: BTreeMap<i64, Vec<Finding>> = BTreeMap::new();
    let mut add = |feed: i64, finding: Option<Finding>| {
        let list = found.entry(feed).or_default();
        if let Some(finding) = finding
            && !list.contains(&finding)
        {
            list.push(finding);
        }
    };

    if let Some(default) = &default {
        let mut statement = collection.db().prepare(
            "SELECT feed_id, address, custom_key, custom_value, name FROM feed_recipients
             ORDER BY feed_id, address, custom_key, custom_value",
        )?;
        let rows = statement.query_map([], |r| {
            let custom_key: String = r.get(2)?;
            let custom_value: String = r.get(3)?;
            Ok((
                r.get::<_, i64>(0)?,
                RecipientKey {
                    address: r.get(1)?,
                    custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                    custom_value: Some(custom_value).filter(|t| !t.is_empty()),
                },
                r.get::<_, Option<String>>(4)?,
            ))
        })?;
        for row in rows {
            let (feed, key, name) = row?;
            if wallets.contains(&parts(&keysend::wallet_key(&key, lookups))) {
                let finding = (parts(&key) != *default).then_some(Finding::NotDefault { key });
                add(feed, finding);
            } else if name.is_some_and(|n| names.contains(&normal_name(&n))) {
                add(feed, Some(Finding::OtherWallet { key }));
            }
        }
    }

    for (feed, finding) in credit_findings(collection, person, &names)? {
        add(feed, finding);
    }

    let feeds = found
        .into_iter()
        .map(|(feed, findings)| {
            let state = check_state(&findings);
            FeedCheck {
                feed_id: FeedId(feed),
                state,
                findings,
            }
        })
        .collect();
    Ok(PersonCheck { feeds })
}

/// The normal name of each person and of each credit name, with the ID of
/// its person (ADR 0111 item 6).
fn person_names(collection: &Collection) -> Result<Vec<(String, PersonId)>, CollectionError> {
    let mut names = Vec::new();
    for person in people(collection)? {
        names.push((normal_name(&person.name), person.id));
        names.extend(
            person
                .credit_names
                .iter()
                .filter(|c| !c.name.trim().is_empty())
                .map(|c| (normal_name(&c.name), person.id)),
        );
    }
    Ok(names)
}

/// The suggestions of people from the credits and the recipients of the
/// feeds (ADR 0098 items 12 to 16). This function makes no person and no
/// link.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn suggestions<S: BuildHasher>(
    collection: &Collection,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<Vec<Suggestion>, CollectionError> {
    let db = collection.db();
    let known_identities: HashSet<String> =
        strings(db, "SELECT identity FROM person_credit_names")?
            .into_iter()
            .collect();
    let known_keys: HashSet<String> = strings(db, "SELECT npub FROM person_keys")?
        .into_iter()
        .collect();
    let owned_wallets = owned_wallets(db, lookups)?;
    let dismissed = dismissals(db)?;
    let groups = credit_groups(collection, &known_identities, &known_keys)?;

    let recipients: Vec<RecipientAgg> = recipient_aggs(db, lookups)?
        .into_iter()
        .filter(|r| !owned_wallets.contains(&r.wallet))
        .collect();

    let plans = plan_groups(&groups);
    let names = person_names(collection)?;
    let mut list = Vec::new();
    for (group, (name, group_name, pays)) in groups.into_iter().zip(plans) {
        let identity_key = group.key.clone();
        // A group of credits with only an `npub` uses the `npub` as its name.
        let name = name.unwrap_or_else(|| {
            group
                .npubs
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| group.key.clone())
        });

        let joined = if pays {
            join_recipients(&recipients, &group_name)
        } else {
            Vec::new()
        };

        let mut credit_feeds = BTreeSet::new();
        let mut credits = Vec::new();
        for ((name, href, npub), agg) in group.credits {
            credit_feeds.extend(agg.feeds.iter().copied());
            credits.push(SuggestedCredit {
                same_name: name.trim().is_empty() || normal_name(&name) == group_name,
                name,
                href,
                npub,
                img: agg.img,
                roles: agg.roles.into_iter().collect(),
                feeds: feed_ids(&agg.feeds),
            });
        }
        let payee_feeds: BTreeSet<i64> = joined
            .iter()
            .flat_map(|r| r.feeds.iter().map(|f| f.0))
            .collect();

        let item_keys: Vec<String> = credits
            .iter()
            .map(|c| credit_item_key(&c.name, &c.href, &c.npub))
            .chain(joined.iter().map(|r| recipient_item_key(&r.key)))
            .collect();
        if let Some(items) = dismissed.get(&identity_key)
            && item_keys.iter().all(|k| items.contains(k))
        {
            continue;
        }

        let mut evidence = Vec::new();
        if group.npubs.len() == 1 {
            evidence.push(Evidence::SameNpub);
        }
        if joined.iter().any(|r| r.joined_by == JoinedBy::Name) {
            evidence.push(Evidence::SameName);
        }
        if joined.iter().any(|r| r.joined_by == JoinedBy::Wallet) {
            evidence.push(Evidence::SameWallet);
        }
        let wanted = normal_name(&name);
        let person_id = names
            .iter()
            .filter(|(n, _)| *n == wanted)
            .map(|(_, id)| *id)
            .min_by_key(|id| id.0);
        list.push(Suggestion {
            identity: identity_key,
            name,
            credits,
            recipients: joined,
            credit_feeds: credit_feeds.len(),
            payee_feeds: payee_feeds.len(),
            evidence,
            item_keys,
            person_id,
        });
    }
    list.sort_by(|a, b| {
        (b.credit_feeds + b.payee_feeds)
            .cmp(&(a.credit_feeds + a.payee_feeds))
            .then_with(|| normal_name(&a.name).cmp(&normal_name(&b.name)))
            .then_with(|| a.identity.cmp(&b.identity))
    });
    Ok(list)
}

/// Keep a dismissal of a suggestion (ADR 0098 item 15). `items` are the
/// item keys of the suggestion. A suggestion with a new item comes back.
/// A second dismissal of the same identity replaces the first.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn dismiss(
    collection: &Collection,
    identity: &str,
    items: &[String],
) -> Result<(), CollectionError> {
    let items = serde_json::to_string(items)
        .map_err(|e| CollectionError::invalid(format!("the items cannot be kept: {e}")))?;
    collection.db().execute(
        "INSERT OR REPLACE INTO person_dismissals (identity, items, dismissed_at)
         VALUES (?1, ?2, ?3)",
        params![identity, items, now()],
    )?;
    Ok(())
}

/// Make a person from the items that the user selected (ADR 0098 item 14).
/// One transaction makes the person and each link. A conflict of one link
/// makes no person.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for a name of only spaces or an
/// item that cannot be a link, and [`CollectionErrorKind::Conflict`] when a
/// different person has a link already.
pub fn make_person<S: BuildHasher>(
    collection: &Collection,
    request: &MakePerson,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<PersonId, CollectionError> {
    let tx = collection.db().unchecked_transaction()?;
    let id = insert_person(
        &tx,
        &PersonFields {
            name: request.name.clone(),
            ..PersonFields::default()
        },
    )?;

    let mut npubs: BTreeSet<String> = BTreeSet::new();
    for (name, href, npub) in &request.credits {
        insert_credit_name(&tx, id, name, href, npub)?;
        if !npub.trim().is_empty() {
            npubs.insert(npub.trim().to_lowercase());
        }
    }
    npubs.extend(request.keys.iter().map(|k| k.trim().to_lowercase()));
    for npub in &npubs {
        insert_key(&tx, id, npub)?;
    }
    for (key, recipient_type) in &request.recipients {
        insert_wallet(&tx, id, key, recipient_type, LinkEvidence::User, lookups)?;
    }

    // The most frequent role of the selected credits. With a tie, the first
    // role A to Z. `img` and `href` come from the first selected credit
    // that has one.
    let rows = collection.feed_credits()?;
    let mut roles: BTreeMap<&str, usize> = BTreeMap::new();
    let mut img = String::new();
    let mut href = String::new();
    for (name, credit_href, npub) in &request.credits {
        let selected = (name.trim(), credit_href.trim(), npub.trim());
        if href.is_empty() {
            selected.1.clone_into(&mut href);
        }
        for row in rows
            .iter()
            .filter(|r| (r.name.as_str(), r.href.as_str(), r.npub.as_str()) == selected)
        {
            if !row.role.is_empty() {
                *roles.entry(row.role.as_str()).or_default() += 1;
            }
            if img.is_empty() {
                img.clone_from(&row.img);
            }
        }
    }
    let usual_role = roles
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(role, _)| (*role).to_owned())
        .unwrap_or_default();
    tx.execute(
        "UPDATE people SET usual_role = ?2, img = ?3, href = ?4 WHERE id = ?1",
        params![id.0, usual_role, img, href],
    )?;

    if let Some(feed) = request.member_of {
        add_member(collection, id, feed)?;
    }
    tx.commit()?;
    Ok(id)
}

// ---- Add items to a person, more items and merge (ADR 0103) ----

/// The items that the user selects to add to a person (ADR 0103 item 10).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PersonItems {
    /// The selected credits: name, `href` and `npub`.
    pub credits: Vec<(String, String, String)>,
    /// The selected recipients, each with its recipient type.
    pub recipients: Vec<(RecipientKey, String)>,
    /// The selected Nostr keys, more than the keys of the credits.
    pub keys: Vec<String>,
}

/// The result of [`add_to_person`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AddedItems {
    /// The number of new links.
    pub added: usize,
}

/// The number of credit names, keys and wallets of a person.
fn link_count(db: &Connection, id: PersonId) -> Result<usize, CollectionError> {
    let count: i64 = db.query_row(
        "SELECT (SELECT COUNT(*) FROM person_credit_names WHERE person_id = ?1)
              + (SELECT COUNT(*) FROM person_keys WHERE person_id = ?1)
              + (SELECT COUNT(*) FROM person_wallets WHERE person_id = ?1)",
        [id.0],
        |r| r.get(0),
    )?;
    Ok(usize::try_from(count).unwrap_or_default())
}

/// Add the items that the user selected to a person (ADR 0103 items 8 to
/// 10). One transaction adds each item that the person does not have. The
/// function changes no field of the person. A conflict of one item adds no
/// item.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::NotFound`] for an unknown person,
/// [`CollectionErrorKind::Invalid`] for an item that cannot be a link, and
/// [`CollectionErrorKind::Conflict`] when a different person has a link
/// already.
pub fn add_to_person<S: BuildHasher>(
    collection: &Collection,
    id: PersonId,
    items: &PersonItems,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<AddedItems, CollectionError> {
    let tx = collection.db().unchecked_transaction()?;
    check_person(&tx, id)?;
    let before = link_count(&tx, id)?;

    let mut npubs: BTreeSet<String> = BTreeSet::new();
    for (name, href, npub) in &items.credits {
        insert_credit_name(&tx, id, name, href, npub)?;
        if !npub.trim().is_empty() {
            npubs.insert(npub.trim().to_lowercase());
        }
    }
    npubs.extend(items.keys.iter().map(|k| k.trim().to_lowercase()));
    for npub in &npubs {
        insert_key(&tx, id, npub)?;
    }
    for (key, recipient_type) in &items.recipients {
        insert_wallet(&tx, id, key, recipient_type, LinkEvidence::User, lookups)?;
    }

    let added = link_count(&tx, id)?.saturating_sub(before);
    tx.commit()?;
    Ok(AddedItems { added })
}

/// Why an item of the feeds matches a person (ADR 0103 item 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoreCause {
    /// The normal name of the item is the normal name of the person.
    SameName,
    /// The normal name of the item is the normal name of a credit name of
    /// the person.
    SameCreditName,
    /// The `npub` of the credit is a key of the person.
    SameNpub,
    /// The wallet key of the payee is the wallet key of a wallet of the
    /// person.
    SameWallet,
}

/// An item of the feeds that matches a person and is not a link of a
/// person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MoreItem {
    /// A distinct credit `(name, href, npub)`.
    Credit {
        /// The trimmed name.
        name: String,
        /// The trimmed `href`, or empty text.
        href: String,
        /// The trimmed `npub`, or empty text.
        npub: String,
        /// The distinct raw roles that are not empty, A to Z.
        roles: Vec<String>,
        /// The feeds that have the credit, by ID.
        feeds: Vec<FeedId>,
        /// The cause of the match.
        cause: MoreCause,
    },
    /// A distinct recipient key.
    Payee {
        /// The key of the recipient.
        key: RecipientKey,
        /// The name from the most recent read, or empty text.
        name: String,
        /// The recipient type from the most recent read, or empty text.
        recipient_type: String,
        /// The feeds that have the recipient, by ID.
        feeds: Vec<FeedId>,
        /// The cause of the match.
        cause: MoreCause,
    },
}

type KeyParts = (String, String, String);
type WalletSets = (HashSet<KeyParts>, HashSet<KeyParts>);

/// The exact keys of all stored wallets, and the wallet keys of the wallets
/// of people other than `id`.
fn wallet_sets<S: BuildHasher>(
    db: &Connection,
    id: PersonId,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<WalletSets, CollectionError> {
    let mut stored = HashSet::new();
    let mut others = HashSet::new();
    let mut statement =
        db.prepare("SELECT person_id, address, custom_key, custom_value FROM person_wallets")?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (owner, address, custom_key, custom_value) = row?;
        let key = RecipientKey {
            address: address.clone(),
            custom_key: Some(custom_key.clone()).filter(|t| !t.is_empty()),
            custom_value: Some(custom_value.clone()).filter(|t| !t.is_empty()),
        };
        if owner != id.0 {
            others.insert(parts(&keysend::wallet_key(&key, lookups)));
        }
        stored.insert((address, custom_key, custom_value));
    }
    Ok((stored, others))
}

/// The items of the feeds that match a person and are not a link of a
/// person (ADR 0103 items 11 to 13). The credits come first, then the
/// payees. The function reads only the collection.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn more_items<S: BuildHasher>(
    collection: &Collection,
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<Vec<MoreItem>, CollectionError> {
    let db = collection.db();
    let known_identities: HashSet<String> =
        strings(db, "SELECT identity FROM person_credit_names")?
            .into_iter()
            .collect();
    // A key of the person makes a credit match. A key of a different person
    // would be a conflict.
    let other_keys: HashSet<String> = {
        let mut statement = db.prepare("SELECT npub FROM person_keys WHERE person_id <> ?1")?;
        statement
            .query_map([person.id.0], |r| r.get(0))?
            .collect::<Result<_, _>>()?
    };
    let (stored, others) = wallet_sets(db, person.id, lookups)?;

    let person_name = normal_name(&person.name);
    let credit_names: HashSet<String> = person
        .credit_names
        .iter()
        .map(|c| normal_name(&c.name))
        .filter(|n| !n.is_empty())
        .collect();
    let npubs: HashSet<&str> = person.keys.iter().map(|k| k.npub.as_str()).collect();
    let wallets: HashSet<(String, String, String)> = person
        .wallets
        .iter()
        .map(|w| parts(&keysend::wallet_key(&w.key, lookups)))
        .collect();
    let by_name = |name: &str| -> Option<MoreCause> {
        let name = normal_name(name);
        if name.is_empty() {
            None
        } else if name == person_name {
            Some(MoreCause::SameName)
        } else if credit_names.contains(&name) {
            Some(MoreCause::SameCreditName)
        } else {
            None
        }
    };

    let mut found: BTreeMap<(String, String, String), CreditAgg> = BTreeMap::new();
    for row in collection.feed_credits()? {
        let Some(identity) = Identity::of(&row.name, &row.href, &row.npub) else {
            continue;
        };
        if known_identities.contains(&identity.key())
            || matches!(&identity, Identity::Npub(npub) if other_keys.contains(npub))
        {
            continue;
        }
        let credit = found.entry((row.name, row.href, row.npub)).or_default();
        if !row.role.is_empty() {
            credit.roles.insert(row.role);
        }
        credit.feeds.insert(row.feed_id.0);
    }
    let mut list = Vec::new();
    for ((name, href, npub), agg) in found {
        let key = npub.trim().to_lowercase();
        let cause = if !key.is_empty() && npubs.contains(key.as_str()) {
            MoreCause::SameNpub
        } else if let Some(cause) = by_name(&name) {
            cause
        } else {
            continue;
        };
        list.push(MoreItem::Credit {
            name: name.trim().to_owned(),
            href: href.trim().to_owned(),
            npub: npub.trim().to_owned(),
            roles: agg.roles.into_iter().collect(),
            feeds: feed_ids(&agg.feeds),
            cause,
        });
    }
    for agg in recipient_aggs(db, lookups)? {
        let exact = parts(&agg.key);
        if stored.contains(&exact) || others.contains(&agg.wallet) {
            continue;
        }
        let cause = if wallets.contains(&agg.wallet) {
            MoreCause::SameWallet
        } else if let Some(cause) = by_name(&agg.name) {
            cause
        } else {
            continue;
        };
        list.push(MoreItem::Payee {
            feeds: feed_ids(&agg.feeds),
            key: agg.key,
            name: agg.name,
            recipient_type: agg.recipient_type,
            cause,
        });
    }
    Ok(list)
}

/// Move each link of the person `other` to the person `keep`, then delete
/// `other` (ADR 0103 items 14 to 16). One transaction does all of it. The
/// fields and the default address of `keep` stay. A membership that `keep`
/// has already is not copied. A change set of `other` gets `keep`. The
/// function changes no feed.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] when `keep` is `other`, and
/// [`CollectionErrorKind::NotFound`] for an unknown person.
pub fn merge_people(
    collection: &Collection,
    keep: PersonId,
    other: PersonId,
) -> Result<(), CollectionError> {
    if keep == other {
        return Err(CollectionError::invalid(
            "a person cannot merge with the same person",
        ));
    }
    let tx = collection.db().unchecked_transaction()?;
    check_person(&tx, keep)?;
    check_person(&tx, other)?;
    for table in ["person_wallets", "person_support_links"] {
        // The links of `other` go after the links of `keep`.
        tx.execute(
            &format!(
                "UPDATE {table} SET person_id = ?1,
                     position = position + (
                         SELECT COALESCE(MAX(position) + 1, 0) FROM {table} WHERE person_id = ?1)
                 WHERE person_id = ?2"
            ),
            params![keep.0, other.0],
        )?;
    }
    for table in ["person_keys", "person_credit_names"] {
        tx.execute(
            &format!("UPDATE {table} SET person_id = ?1 WHERE person_id = ?2"),
            params![keep.0, other.0],
        )?;
    }
    // A membership that `keep` has stays in `other` and goes with it.
    tx.execute(
        "UPDATE OR IGNORE person_members SET person_id = ?1 WHERE person_id = ?2",
        params![keep.0, other.0],
    )?;
    tx.execute(
        "UPDATE change_sets SET person_id = ?1 WHERE person_id = ?2",
        params![keep.0, other.0],
    )?;
    tx.execute(
        "UPDATE OR IGNORE person_address_dismissals SET person_id = ?1 WHERE person_id = ?2",
        params![keep.0, other.0],
    )?;
    tx.execute("DELETE FROM people WHERE id = ?1", [other.0])?;
    tx.commit()?;
    Ok(())
}

// ---- The Lightning address of a profile (ADR 0098 items 17 to 19) ----

/// Dismiss the Lightning address of a profile for `person` (the button
/// "Dismiss", ADR 0098 item 18). [`profile_addresses`] then leaves it out.
/// The address is kept in lowercase. A second dismissal changes nothing.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] for an empty address, a missing
/// person error for an unknown person, and a [`CollectionError`] when
/// SQLite fails.
pub fn dismiss_profile_address(
    collection: &Collection,
    person: PersonId,
    address: &str,
) -> Result<(), CollectionError> {
    let address = address.trim().to_lowercase();
    if address.is_empty() {
        return Err(CollectionError::invalid("the address is empty"));
    }
    let exists: bool = collection.db().query_row(
        "SELECT EXISTS (SELECT 1 FROM people WHERE id = ?1)",
        [person.0],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(missing(person));
    }
    collection.db().execute(
        "INSERT OR IGNORE INTO person_address_dismissals (person_id, address, dismissed_at)
         VALUES (?1, ?2, ?3)",
        params![person.0, address, now()],
    )?;
    Ok(())
}

/// The Lightning addresses that the user dismissed for `person`, in
/// lowercase.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn dismissed_profile_addresses(
    collection: &Collection,
    person: PersonId,
) -> Result<HashSet<String>, CollectionError> {
    let mut statement = collection
        .db()
        .prepare("SELECT address FROM person_address_dismissals WHERE person_id = ?1")?;
    let rows = statement.query_map([person.0], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The Lightning address in the profile of a key of a person, with a
/// keysend lookup that gives a wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileAddress {
    /// The Lightning address, in lowercase.
    pub address: String,
    /// The key, as `npub`, whose profile gave the address.
    pub npub: String,
    /// The key of the wallet that the lookup gives.
    pub wallet: RecipientKey,
    /// True when `wallet` is the wallet key of a wallet of the person.
    pub in_person_wallet: bool,
}

/// The Lightning addresses of the Nostr keys of `person` that can be
/// added to a wallet (ADR 0098 items 17 to 19).
///
/// The function reads the `lud16` of the newest kind 0 of each key. It
/// leaves out an address in `dismissed` and an address that is a member of
/// a wallet of the person already. It requests the keysend data of each other address and gives
/// only an address with a `Found` result. It keeps no lookup, so the
/// collection has no lookup of a suggestion that the user did not add
/// (ADR 0097 item 3). Each request follows `policy` (ADR 0057).
// The dismissed addresses come in as their own set, so that no keysend
// request goes to an address that the user refused.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn profile_addresses<S: BuildHasher, T: BuildHasher>(
    web: &Web,
    policy: &NetPolicy,
    relays: &[String],
    scheme: &str,
    limit: Duration,
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
    dismissed: &HashSet<String, T>,
) -> Vec<ProfileAddress> {
    let npubs: Vec<String> = person.keys.iter().map(|k| k.npub.clone()).collect();
    let wanted: Vec<(String, String)> =
        profiles::lightning_addresses_by_key(relays, policy, limit, &npubs)
            .into_iter()
            .filter(|(_, address)| !dismissed.contains(&address.trim().to_lowercase()))
            .filter(|(_, address)| {
                let key = RecipientKey {
                    address: address.clone(),
                    custom_key: None,
                    custom_value: None,
                };
                !is_member(person, &key, lookups)
            })
            .collect();
    let results: Vec<(&String, &String, LookupResult)> = thread::scope(|scope| {
        let handles: Vec<_> = wanted
            .iter()
            .map(|(npub, address)| {
                scope.spawn(move || {
                    let web = web.clone();
                    let scheme = scheme.to_owned();
                    let text = address.clone();
                    let result = profiles::within(keysend::LOOKUP_LIMIT, move || {
                        keysend::request(&web, &scheme, &text)
                    })
                    .unwrap_or(LookupResult::Failed);
                    (address, npub, result)
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    });
    let owned: HashSet<(String, String, String)> = person
        .wallets
        .iter()
        .map(|w| parts(&keysend::wallet_key(&w.key, lookups)))
        .collect();
    results
        .into_iter()
        .filter_map(|(address, npub, result)| {
            let LookupResult::Found {
                pubkey,
                custom_key,
                custom_value,
            } = result
            else {
                return None;
            };
            let wallet = RecipientKey {
                address: pubkey,
                custom_key,
                custom_value,
            };
            Some(ProfileAddress {
                address: address.clone(),
                npub: npub.clone(),
                in_person_wallet: owned.contains(&parts(&wallet)),
                wallet,
            })
        })
        .collect()
}

// ---- The picture and the information address (ADR 0113) ----

/// The source of a suggested value (ADR 0113 item 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DetailSource {
    /// The newest kind 0 of a key of the person.
    Profile {
        /// The key, as `npub`.
        npub: String,
    },
    /// The credits of the person in the feeds.
    Credit {
        /// The feeds, from low to high.
        feed_ids: Vec<FeedId>,
    },
    /// A publisher feed of which the person is a member.
    Member {
        /// The publisher feed.
        feed_id: FeedId,
    },
}

/// A suggested value with each source that gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetailSuggestion {
    /// An `http` or `https` address.
    pub value: String,
    /// The sources, in the sequence profile, credit, member.
    pub sources: Vec<DetailSource>,
}

/// The suggestions for the picture and for the information address.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DetailSuggestions {
    /// The suggestions for the picture address.
    pub img: Vec<DetailSuggestion>,
    /// The suggestions for the information address.
    pub href: Vec<DetailSuggestion>,
}

/// The image and the link of a publisher feed, from the last read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberFeed {
    /// The publisher feed.
    pub id: FeedId,
    /// The image of the feed.
    pub image: Option<String>,
    /// The `link` of the feed.
    pub link: Option<String>,
}

/// The values of one field, by source, before they join.
#[derive(Default)]
struct DetailValues {
    profile: Vec<(String, String)>,
    credit: Vec<(i64, String)>,
    member: Vec<(FeedId, String)>,
}

fn is_web_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

/// The sources of one value, before they become [`DetailSource`] items.
#[derive(Default)]
struct Joined {
    profile: Vec<String>,
    credit: BTreeSet<i64>,
    member: Vec<FeedId>,
}

/// The entry of `value` in `joined`, or `None` when the value is empty, is
/// `current` or is not an `http` or `https` address.
fn slot<'a>(
    joined: &'a mut HashMap<String, Joined>,
    order: &mut Vec<String>,
    current: &str,
    value: &str,
) -> Option<&'a mut Joined> {
    let value = value.trim();
    if value.is_empty() || value == current || !is_web_value(value) {
        return None;
    }
    if !joined.contains_key(value) {
        order.push(value.to_owned());
    }
    Some(joined.entry(value.to_owned()).or_default())
}

/// Join equal values, leave out `current` and sort (ADR 0113 items 3 and 4).
fn join_details(current: &str, values: DetailValues) -> Vec<DetailSuggestion> {
    let current = current.trim();
    let mut order: Vec<String> = Vec::new();
    let mut joined: HashMap<String, Joined> = HashMap::new();
    for (npub, value) in values.profile {
        if let Some(j) = slot(&mut joined, &mut order, current, &value)
            && !j.profile.contains(&npub)
        {
            j.profile.push(npub);
        }
    }
    for (feed, value) in values.credit {
        if let Some(j) = slot(&mut joined, &mut order, current, &value) {
            j.credit.insert(feed);
        }
    }
    for (feed, value) in values.member {
        if let Some(j) = slot(&mut joined, &mut order, current, &value)
            && !j.member.contains(&feed)
        {
            j.member.push(feed);
        }
    }
    let mut list: Vec<(usize, usize, DetailSuggestion)> = order
        .into_iter()
        .filter_map(|value| {
            let j = joined.remove(&value)?;
            let rank = if !j.profile.is_empty() {
                0
            } else if !j.credit.is_empty() {
                1
            } else {
                2
            };
            let count = j.credit.len();
            let mut sources: Vec<DetailSource> = j
                .profile
                .into_iter()
                .map(|npub| DetailSource::Profile { npub })
                .collect();
            if !j.credit.is_empty() {
                sources.push(DetailSource::Credit {
                    feed_ids: j.credit.into_iter().map(FeedId).collect(),
                });
            }
            sources.extend(
                j.member
                    .into_iter()
                    .map(|feed_id| DetailSource::Member { feed_id }),
            );
            Some((rank, count, DetailSuggestion { value, sources }))
        })
        .collect();
    // A stable sort keeps the first-seen sequence of equal keys.
    list.sort_by_key(|(rank, count, _)| (*rank, std::cmp::Reverse(*count)));
    list.into_iter().map(|(_, _, s)| s).collect()
}

/// The suggestions for the picture and the information address of a person
/// (ADR 0113). The credits follow ADR 0111 item 10, as [`check`] does.
/// `feeds` holds the publisher feeds. The function uses only those in
/// `person.members`. `profiles` holds the values that the caller read from
/// the relays. This function makes no request.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn detail_suggestions(
    collection: &Collection,
    person: &Person,
    feeds: &[MemberFeed],
    profiles: &[profiles::ProfileDetails],
) -> Result<DetailSuggestions, CollectionError> {
    let mut img = DetailValues::default();
    let mut href = DetailValues::default();
    let own_keys: HashSet<&str> = person.keys.iter().map(|k| k.npub.as_str()).collect();
    for details in profiles
        .iter()
        .filter(|d| own_keys.contains(d.npub.as_str()))
    {
        if let Some(value) = &details.picture {
            img.profile.push((details.npub.clone(), value.clone()));
        }
        if let Some(value) = &details.website {
            href.profile.push((details.npub.clone(), value.clone()));
        }
    }
    for row in person_credits(collection, person, &person_normal_names(person))? {
        img.credit.push((row.feed_id.0, row.img));
        href.credit.push((row.feed_id.0, row.href));
    }
    for feed in feeds.iter().filter(|f| person.members.contains(&f.id)) {
        if let Some(value) = &feed.image {
            img.member.push((feed.id, value.clone()));
        }
        if let Some(value) = &feed.link {
            href.member.push((feed.id, value.clone()));
        }
    }
    Ok(DetailSuggestions {
        img: join_details(&person.img, img),
        href: join_details(&person.href, href),
    })
}

/// Add the Lightning address of a profile to a wallet of a person. The
/// function requests and stores the keysend lookup of `address`. With a
/// `Found` result, it links the wallet of the lookup with the type
/// `node` and the evidence `nostr`.
///
/// # Errors
///
/// Gives [`CollectionErrorKind::Invalid`] when the lookup gives no wallet,
/// [`CollectionErrorKind::Conflict`] when a different person has the wallet
/// and [`CollectionErrorKind::NotFound`] for an unknown person.
pub fn add_profile_address(
    collection: &Collection,
    web: &Web,
    scheme: &str,
    id: PersonId,
    address: &str,
) -> Result<i64, CollectionError> {
    check_person(collection.db(), id)?;
    let address = address.trim().to_lowercase();
    let lookups = keysend::refresh(collection, web, scheme, from_ref(&address), now())?;
    let key = lookups
        .get(&address)
        .and_then(Lookup::key)
        .ok_or_else(|| CollectionError::invalid("the address has no keysend data"))?;
    add_wallet(collection, id, &key, "node", LinkEvidence::Nostr)
}

// ---- The export (ADR 0098 items 24 and 25) ----

/// A feed of which a person is a member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportMembership {
    /// The ID of the feed.
    pub feed_id: FeedId,
    /// The title, from the last read.
    pub title: Option<String>,
    /// `podcast:guid`, from the last read.
    pub podcast_guid: Option<String>,
}

/// A Lightning address that a stored lookup gives for a wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportAddress {
    /// The Lightning address.
    pub address: String,
    /// Always `keysend`.
    pub evidence: &'static str,
    /// The Unix time of the lookup.
    pub checked_at: i64,
}

/// A wallet of a person, with the other addresses of the wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportWallet {
    /// The wallet as the user linked it.
    #[serde(flatten)]
    pub wallet: PersonWallet,
    /// Each Lightning address that the stored lookups give for the wallet.
    pub addresses: Vec<ExportAddress>,
}

/// A person in the export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportPerson {
    /// The ID.
    pub id: PersonId,
    /// The name.
    pub name: String,
    /// The usual role, as raw text.
    pub usual_role: String,
    /// The usual group for the usual role, as raw text.
    pub usual_group: String,
    /// The picture address, or empty text.
    pub img: String,
    /// The information address, or empty text.
    pub href: String,
    /// The default address that applies now ([`default_address`]).
    pub default_address: Option<RecipientKey>,
    /// The wallets, with the addresses of each wallet.
    pub wallets: Vec<ExportWallet>,
    /// The Nostr keys.
    pub keys: Vec<PersonKey>,
    /// The credit names.
    pub credit_names: Vec<CreditName>,
    /// The support links.
    pub support_links: Vec<SupportLink>,
    /// The publisher feeds of the person, with their titles.
    pub memberships: Vec<ExportMembership>,
}

/// The export of the people (ADR 0098 items 24 and 25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PeopleExport {
    /// Always `studio-people`.
    pub format: &'static str,
    /// The version of the format.
    pub version: u32,
    /// The Unix time of the export.
    pub exported_at: i64,
    /// Each person, A to Z.
    pub people: Vec<ExportPerson>,
}

/// The export of each person with each link, its evidence and its time.
/// The tables of the people hold no secret, so the export holds none.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn export<S: BuildHasher>(
    collection: &Collection,
    lookups: &HashMap<String, Lookup, S>,
) -> Result<PeopleExport, CollectionError> {
    let mut list = Vec::new();
    for person in people(collection)? {
        let wallets = person
            .wallets
            .iter()
            .map(|wallet| {
                let (key, members) = keysend::wallet_of(&wallet.key, lookups);
                let addresses = members
                    .iter()
                    .filter(|m| m.parts() != wallet.key.parts() && m.parts() != key.parts())
                    .filter_map(|m| lookups.get(&m.address.trim().to_lowercase()))
                    .map(|lookup| ExportAddress {
                        address: lookup.address.clone(),
                        evidence: "keysend",
                        checked_at: lookup.checked_at,
                    })
                    .collect();
                ExportWallet {
                    wallet: wallet.clone(),
                    addresses,
                }
            })
            .collect();
        let mut memberships = Vec::new();
        for feed in &person.members {
            let row = collection.feed(*feed)?;
            memberships.push(ExportMembership {
                feed_id: row.id,
                title: row.title,
                podcast_guid: row.podcast_guid,
            });
        }
        list.push(ExportPerson {
            id: person.id,
            default_address: default_address(&person, lookups),
            name: person.name,
            usual_role: person.usual_role,
            usual_group: person.usual_group,
            img: person.img,
            href: person.href,
            wallets,
            keys: person.keys,
            credit_names: person.credit_names,
            support_links: person.support_links,
            memberships,
        });
    }
    Ok(PeopleExport {
        format: "studio-people",
        version: 1,
        exported_at: now(),
        people: list,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::Version;
    use crate::feed::Feed;
    use crate::keysend::LookupStatus;
    use crate::xml::Document;

    fn key(address: &str) -> RecipientKey {
        RecipientKey {
            address: address.into(),
            custom_key: None,
            custom_value: None,
        }
    }

    fn fields(name: &str) -> PersonFields {
        PersonFields {
            name: name.into(),
            ..PersonFields::default()
        }
    }

    /// A lookup that gives the node `node` for the address `address`.
    fn found(collection: &Collection, address: &str, node: &str) {
        collection
            .set_keysend_lookup(&Lookup {
                address: address.into(),
                pubkey: Some(node.into()),
                custom_key: None,
                custom_value: None,
                status: LookupStatus::Found,
                checked_at: 1,
            })
            .unwrap();
    }

    #[test]
    fn identity_prefers_the_npub_then_the_name_with_the_address_then_the_name() {
        assert_eq!(
            Identity::of(" Ann ", "https://a.example", " NPUB1abc "),
            Some(Identity::Npub("npub1abc".into()))
        );
        assert_eq!(
            Identity::of("  Mary   KATE ", " https://a.example ", ""),
            Some(Identity::NameHref {
                name: "mary kate".into(),
                href: "https://a.example".into()
            })
        );
        assert_eq!(
            Identity::of("Mary Kate", "  ", ""),
            Some(Identity::Name("mary kate".into()))
        );
        assert_eq!(Identity::of("   ", "https://a.example", ""), None);
        assert_eq!(
            Identity::of("Ann", "https://a.example", "").unwrap().key(),
            "href:ann\u{1f}https://a.example"
        );
        assert_eq!(Identity::of("Ann", "", "").unwrap().key(), "name:ann");
    }

    #[test]
    fn a_name_of_only_spaces_is_refused() {
        let collection = Collection::open_in_memory().unwrap();
        let error = add_person(&collection, &fields("   ")).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
        let id = add_person(&collection, &fields(" Ann ")).unwrap();
        assert_eq!(person(&collection, id).unwrap().name, "Ann");
        let error = change_person(&collection, id, &fields(" ")).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
        assert_eq!(
            person(&collection, PersonId(99)).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
    }

    #[test]
    fn people_are_in_the_sequence_of_their_normal_names() {
        let collection = Collection::open_in_memory().unwrap();
        add_person(&collection, &fields("bob")).unwrap();
        add_person(&collection, &fields("Ann")).unwrap();
        add_person(&collection, &fields("  ANN ")).unwrap();
        let names: Vec<(String, i64)> = people(&collection)
            .unwrap()
            .into_iter()
            .map(|p| (p.name, p.id.0))
            .collect();
        assert_eq!(
            names,
            [("Ann".into(), 2), ("ANN".into(), 3), ("bob".into(), 1)]
        );
    }

    #[test]
    fn a_credit_name_gives_its_feeds_and_its_roles() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "composer", "", "npub1ann");
        credit(&collection, 1, "Ann", "guitarist", "", "npub1ann");
        credit(&collection, 2, "Annie", "", "", "npub1ann");
        credit(&collection, 3, "Ann", "mixer", "", "");
        let id = add_person(&collection, &fields("Ann")).unwrap();
        let keyed = add_credit_name(&collection, id, "Ann", "", "npub1ann").unwrap();
        let unused = add_credit_name(&collection, id, "Nobody", "", "").unwrap();
        let uses = credit_name_uses(&collection, &person(&collection, id).unwrap()).unwrap();
        assert_eq!(
            uses,
            vec![
                CreditNameUse {
                    id: unused,
                    feeds: 0,
                    roles: vec![]
                },
                CreditNameUse {
                    id: keyed,
                    feeds: 2,
                    roles: vec!["composer".into(), "guitarist".into()]
                },
            ]
        );
    }

    #[test]
    fn a_change_of_a_credit_name_replaces_its_values_and_its_old_key() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        add_key(&collection, ann, "npub1old").unwrap();
        let row = add_credit_name(&collection, ann, "Ann", "", "npub1old").unwrap();
        add_credit_name(&collection, bob, "Bob", "", "").unwrap();

        change_credit_name(
            &collection,
            ann,
            row,
            "Anna",
            "https://a.example",
            "npub1new",
        )
        .unwrap();
        let p = person(&collection, ann).unwrap();
        assert_eq!(p.credit_names.len(), 1);
        assert_eq!(p.credit_names[0].name, "Anna", "ADR 0146 item 6");
        assert_eq!(p.credit_names[0].href, "https://a.example");
        assert_eq!(p.credit_names[0].npub, "npub1new");
        assert!(p.keys.is_empty(), "ADR 0146 item 6: the old key goes");

        let same = change_credit_name(&collection, ann, row, "Bob", "", "");
        assert_eq!(same.unwrap_err().kind(), CollectionErrorKind::Conflict);
        let other = change_credit_name(&collection, bob, row, "Cy", "", "");
        assert_eq!(other.unwrap_err().kind(), CollectionErrorKind::NotFound);
    }

    #[test]
    fn a_change_of_a_support_link_keeps_its_row_and_checks_its_address() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let row = add_support_link(&collection, ann, "Tip", "https://a.example").unwrap();
        change_support_link(&collection, ann, row, "Tips", "https://b.example").unwrap();
        let p = person(&collection, ann).unwrap();
        assert_eq!(p.support_links[0].id, row);
        assert_eq!(p.support_links[0].label, "Tips", "ADR 0146 item 6");
        assert_eq!(p.support_links[0].url, "https://b.example");
        let bad = change_support_link(&collection, ann, row, "Tips", "ftp://b.example");
        assert_eq!(bad.unwrap_err().kind(), CollectionErrorKind::Invalid);
    }

    #[test]
    fn a_change_of_a_wallet_keeps_its_place_and_moves_the_default() {
        let collection = Collection::open_in_memory().unwrap();
        let lookups = HashMap::new();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let first =
            add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        add_wallet(&collection, ann, &key("node-2"), "node", LinkEvidence::User).unwrap();
        let effective =
            |collection: &Collection| default_address(&person(collection, ann).unwrap(), &lookups);

        // With no selected default, the first wallet stays the default by its place.
        change_wallet(&collection, ann, first, &key("ann@example.com"), "node").unwrap();
        let p = person(&collection, ann).unwrap();
        let rows: Vec<(String, String)> = p
            .wallets
            .iter()
            .map(|w| (w.key.address.clone(), w.recipient_type.clone()))
            .collect();
        assert_eq!(
            rows,
            vec![
                ("ann@example.com".to_owned(), "lnaddress".to_owned()),
                ("node-2".to_owned(), "node".to_owned()),
            ]
        );
        assert_eq!(effective(&collection), Some(key("ann@example.com")));

        // A selected default moves with its wallet.
        set_default_address(&collection, ann, Some(&key("node-2"))).unwrap();
        change_wallet(&collection, ann, p.wallets[1].id, &key("node-3"), "node").unwrap();
        assert_eq!(effective(&collection), Some(key("node-3")));

        // A wallet with no default leaves the default where it is.
        let p = person(&collection, ann).unwrap();
        change_wallet(&collection, ann, p.wallets[0].id, &key("node-4"), "node").unwrap();
        assert_eq!(effective(&collection), Some(key("node-3")));
    }

    #[test]
    fn a_change_of_a_wallet_refuses_the_wallet_of_a_different_person_and_changes_nothing() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        let wallet =
            add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        add_wallet(&collection, bob, &key("node-2"), "node", LinkEvidence::User).unwrap();
        change_wallet(&collection, ann, wallet, &key("node-2"), "node").unwrap_err();
        assert_eq!(
            person(&collection, ann).unwrap().wallets[0].key,
            key("node-1")
        );
        change_wallet(&collection, ann, 9999, &key("node-9"), "node").unwrap_err();
    }

    #[test]
    fn a_wallet_takes_its_type_from_its_address() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(
            &collection,
            ann,
            &key("03abc"),
            "lnaddress",
            LinkEvidence::User,
        )
        .unwrap();
        add_wallet(
            &collection,
            ann,
            &key("ann@example.com"),
            "node",
            LinkEvidence::User,
        )
        .unwrap();
        // A row from before this rule keeps its text, and a read corrects it.
        collection
            .db()
            .execute(
                "INSERT INTO person_wallets
                     (person_id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at)
                 VALUES (?1, '03old', '', '', 'lnaddress', 2, 'user', 0)",
                [ann.0],
            )
            .unwrap();
        let types: Vec<(String, String)> = person(&collection, ann)
            .unwrap()
            .wallets
            .into_iter()
            .map(|w| (w.key.address, w.recipient_type))
            .collect();
        assert_eq!(
            types,
            vec![
                ("03abc".to_owned(), "node".to_owned()),
                ("ann@example.com".to_owned(), "lnaddress".to_owned()),
                ("03old".to_owned(), "node".to_owned()),
            ]
        );
    }

    #[test]
    fn a_wallet_is_in_a_maximum_of_one_person() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        found(&collection, "ann@example.com", "node-1");
        // The lookup of the Lightning address gives the wallet key of Ann.
        let error = add_wallet(
            &collection,
            bob,
            &key("ann@example.com"),
            "lnaddress",
            LinkEvidence::User,
        )
        .unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        let error =
            add_wallet(&collection, bob, &key("node-1"), "node", LinkEvidence::User).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        // A second key of the same wallet in the same person is permitted.
        add_wallet(
            &collection,
            ann,
            &key("ann@example.com"),
            "lnaddress",
            LinkEvidence::User,
        )
        .unwrap();
        let wallets = person(&collection, ann).unwrap().wallets;
        assert_eq!(wallets.len(), 2);
        assert_eq!((wallets[0].position, wallets[1].position), (0, 1));
        assert_eq!(person(&collection, bob).unwrap().wallets, Vec::new());
        let lookups = lookup_map(&collection).unwrap();
        assert_eq!(
            person_of_wallet(&collection, &key("ann@example.com"), &lookups).unwrap(),
            Some(ann)
        );
        assert_eq!(
            person_of_wallet(&collection, &key("other"), &lookups).unwrap(),
            None
        );
    }

    #[test]
    fn add_key_add_credit_name_and_add_member_refuse_the_wrong_cases() {
        let mut collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();

        assert_eq!(
            add_key(&collection, ann, "xyz").unwrap_err().kind(),
            CollectionErrorKind::Invalid
        );
        let first = add_key(&collection, ann, " NPUB1abc ").unwrap();
        assert_eq!(add_key(&collection, ann, "npub1abc").unwrap(), first);
        assert_eq!(person(&collection, ann).unwrap().keys[0].npub, "npub1abc");
        assert_eq!(
            add_key(&collection, bob, "npub1abc").unwrap_err().kind(),
            CollectionErrorKind::Conflict
        );

        assert_eq!(
            add_credit_name(&collection, ann, "  ", "https://a.example", "")
                .unwrap_err()
                .kind(),
            CollectionErrorKind::Invalid
        );
        add_credit_name(&collection, ann, "Ann  Lee", "", "").unwrap();
        assert_eq!(
            add_credit_name(&collection, bob, " ann lee ", "", "")
                .unwrap_err()
                .kind(),
            CollectionErrorKind::Conflict
        );
        // The same name with an address is a different identity.
        add_credit_name(&collection, bob, "Ann Lee", "https://a.example", "").unwrap();

        let album = collection.add_feed(None, None).unwrap();
        let publisher = collection.add_feed(None, None).unwrap();
        let doc = |medium: &str| {
            Document::parse(&format!(
                "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
                 <podcast:medium>{medium}</podcast:medium></channel></rss>"
            ))
            .unwrap()
        };
        collection
            .record_state(
                album,
                &Feed::read(&doc("music")).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        collection
            .record_state(
                publisher,
                &Feed::read(&doc("publisher")).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        assert_eq!(
            add_member(&collection, ann, album).unwrap_err().kind(),
            CollectionErrorKind::Invalid
        );
        assert_eq!(
            add_member(&collection, ann, FeedId(99)).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
        add_member(&collection, ann, publisher).unwrap();
        add_member(&collection, ann, publisher).unwrap();
        assert_eq!(person(&collection, ann).unwrap().members, [publisher]);

        assert_eq!(
            add_support_link(&collection, ann, "Tip", "ftp://x")
                .unwrap_err()
                .kind(),
            CollectionErrorKind::Invalid
        );
        add_support_link(&collection, ann, "Tip", "https://tip.example").unwrap();
    }

    #[test]
    fn the_default_address_is_the_first_wallet_until_the_user_sets_one() {
        let collection = Collection::open_in_memory().unwrap();
        let lookups = HashMap::new();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("first"), "node", LinkEvidence::User).unwrap();
        let second =
            add_wallet(&collection, ann, &key("second"), "node", LinkEvidence::User).unwrap();
        let read = || person(&collection, ann).unwrap();
        assert_eq!(default_address(&read(), &lookups), Some(key("first")));

        assert_eq!(
            set_default_address(&collection, ann, Some(&key("other")))
                .unwrap_err()
                .kind(),
            CollectionErrorKind::Invalid
        );
        set_default_address(&collection, ann, Some(&key("second"))).unwrap();
        assert_eq!(default_address(&read(), &lookups), Some(key("second")));

        remove_wallet(&collection, ann, second).unwrap();
        assert_eq!(read().default_address, None);
        assert_eq!(default_address(&read(), &lookups), Some(key("first")));
        assert_eq!(
            remove_wallet(&collection, ann, second).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
    }

    #[test]
    fn a_default_address_can_be_a_lightning_address_of_a_wallet() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        found(&collection, "ann@example.com", "node-1");
        set_default_address(&collection, ann, Some(&key("ann@example.com"))).unwrap();
        let lookups = lookup_map(&collection).unwrap();
        assert_eq!(
            default_address(&person(&collection, ann).unwrap(), &lookups),
            Some(key("ann@example.com"))
        );
    }

    #[test]
    fn remove_person_removes_each_link_of_the_person() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        add_key(&collection, ann, "npub1abc").unwrap();
        add_credit_name(&collection, ann, "Ann", "", "").unwrap();
        add_support_link(&collection, ann, "", "https://tip.example").unwrap();
        collection
            .db()
            .execute(
                "INSERT INTO feeds (id, medium, created_at) VALUES (7, 'publisher', 0)",
                [],
            )
            .unwrap();
        add_member(&collection, ann, FeedId(7)).unwrap();
        remove_person(&collection, ann).unwrap();
        for table in [
            "person_wallets",
            "person_keys",
            "person_credit_names",
            "person_support_links",
            "person_members",
        ] {
            let count: i64 = collection
                .db()
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 0, "{table} must have no row.");
        }
        // The feed stays.
        assert_eq!(collection.feeds().unwrap().len(), 1);
    }

    #[test]
    fn save_recipient_gives_the_person_that_has_the_wallet() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        found(&collection, "ann@example.com", "node-1");
        let recipient = Recipient {
            name: Some("Other name".into()),
            address: Some("ann@example.com".into()),
            ..Recipient::default()
        };
        assert_eq!(save_recipient(&collection, &recipient).unwrap(), ann);
        assert_eq!(person(&collection, ann).unwrap().name, "Ann");
        assert_eq!(people(&collection).unwrap().len(), 1);
        // A recipient with no name gives a person with its address as name.
        let nameless = Recipient {
            address: Some("zed".into()),
            ..Recipient::default()
        };
        let zed = save_recipient(&collection, &nameless).unwrap();
        assert_eq!(person(&collection, zed).unwrap().name, "zed");
    }

    fn feed(collection: &Collection, id: i64) {
        collection
            .db()
            .execute(
                "INSERT OR IGNORE INTO feeds (id, medium, created_at) VALUES (?1, 'music', 0)",
                [id],
            )
            .unwrap();
    }

    fn credit(
        collection: &Collection,
        feed_id: i64,
        name: &str,
        role: &str,
        href: &str,
        npub: &str,
    ) {
        feed(collection, feed_id);
        collection
            .db()
            .execute(
                "INSERT INTO feed_credits (feed_id, name, role, href, npub, img, seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                params![
                    feed_id,
                    name,
                    role,
                    href,
                    npub,
                    if npub.is_empty() { "" } else { "img.png" }
                ],
            )
            .unwrap();
    }

    fn payee(collection: &Collection, feed_id: i64, address: &str, name: &str) {
        feed(collection, feed_id);
        collection
            .db()
            .execute(
                "INSERT INTO feed_recipients (feed_id, address, name, seen_at)
                 VALUES (?1, ?2, ?3, 0)",
                params![feed_id, address, name],
            )
            .unwrap();
    }

    /// A publisher feed with the ID `id`. A credit helper keeps the medium.
    fn publisher(collection: &Collection, id: i64) {
        collection
            .db()
            .execute(
                "INSERT OR IGNORE INTO feeds (id, medium, created_at) VALUES (?1, 'publisher', 0)",
                [id],
            )
            .unwrap();
    }

    #[test]
    fn feed_members_give_a_membership_a_credit_or_both_with_the_condition_of_each() {
        let collection = Collection::open_in_memory().unwrap();
        publisher(&collection, 1);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_person(&collection, &fields("Bob")).unwrap();
        let cy = add_person(&collection, &fields("Cy")).unwrap();
        add_person(&collection, &fields("Dee")).unwrap();
        add_member(&collection, ann, FeedId(1)).unwrap();
        add_member(&collection, cy, FeedId(1)).unwrap();
        credit(&collection, 1, "Bob", "", "", "");
        credit(&collection, 1, "Cy", "", "", "");
        // A credit of another feed is not a member of this feed.
        credit(&collection, 2, "Dee", "", "", "");
        let list = feed_members(&collection, FeedId(1)).unwrap();
        let condition: Vec<(&str, bool, bool)> = list
            .iter()
            .map(|m| (m.name.as_str(), m.member, m.in_credits))
            .collect();
        assert_eq!(
            condition,
            [
                ("Ann", true, false),
                ("Bob", false, true),
                ("Cy", true, true)
            ],
            "ADR 0118 item 9: a membership, a credit, or both must give a member with each condition."
        );
        assert_eq!(list[0].person_id, ann);
        assert_eq!(feed_members(&collection, FeedId(2)).unwrap(), Vec::new());
        assert!(feed_members(&collection, FeedId(99)).is_err());
    }

    #[test]
    fn linked_credits_give_only_a_link_with_a_credit_of_the_person() {
        let collection = Collection::open_in_memory().unwrap();
        publisher(&collection, 1);
        publisher(&collection, 2);
        publisher(&collection, 3);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_member(&collection, ann, FeedId(1)).unwrap();
        add_member(&collection, ann, FeedId(2)).unwrap();
        // A credit that only the name matches counts. A credit on a feed with
        // no link does not.
        credit(&collection, 1, "Ann", "", "", "");
        credit(&collection, 3, "Ann", "", "", "");
        let found = person(&collection, ann).unwrap();
        assert_eq!(
            linked_credits(&collection, &found).unwrap(),
            [FeedId(1)],
            "ADR 0132 item 4: the condition must agree with the members of the feed."
        );
    }

    fn suggest(collection: &Collection) -> Vec<Suggestion> {
        suggestions(collection, &lookup_map(collection).unwrap()).unwrap()
    }

    fn counts(collection: &Collection) -> (usize, i64) {
        let links: i64 = [
            "person_wallets",
            "person_keys",
            "person_credit_names",
            "person_members",
        ]
        .iter()
        .map(|t| {
            collection
                .db()
                .query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap()
        })
        .sum();
        (people(collection).unwrap().len(), links)
    }

    fn credit_row(
        collection: &Collection,
        feed_id: i64,
        name: &str,
        role: &str,
        img: &str,
        npub: &str,
    ) {
        feed(collection, feed_id);
        collection
            .db()
            .execute(
                "INSERT INTO feed_credits (feed_id, name, role, href, npub, img, seen_at)
                 VALUES (?1, ?2, ?3, '', ?4, ?5, 0)",
                params![feed_id, name, role, npub, img],
            )
            .unwrap();
    }

    fn checked(collection: &Collection, id: PersonId) -> PersonCheck {
        let person = person(collection, id).unwrap();
        check(collection, &person, &lookup_map(collection).unwrap()).unwrap()
    }

    fn feed_check(list: &PersonCheck, feed_id: i64) -> &FeedCheck {
        list.feeds
            .iter()
            .find(|f| f.feed_id.0 == feed_id)
            .expect("ADR 0111 item 9: the check must list this feed. Check people::check.")
    }

    #[test]
    fn check_finds_a_wrong_key_a_missing_key_a_picture_and_a_missing_role() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(
            &collection,
            &PersonFields {
                name: "Ann".into(),
                img: "ann.png".into(),
                ..PersonFields::default()
            },
        )
        .unwrap();
        add_key(&collection, ann, "npub1ann").unwrap();
        credit_row(&collection, 1, "ann", "vocals", "ann.png", "npub1city");
        credit_row(&collection, 2, "Ann", "vocals", "ann.png", "");
        credit_row(&collection, 3, "Ann", "vocals", "old.png", "npub1ann");
        credit_row(&collection, 4, "Ann", "", "ann.png", "NPUB1ANN");
        credit_row(&collection, 5, "Ann", "vocals", "ann.png", "npub1ann");
        credit_row(&collection, 6, "Bob", "vocals", "x.png", "npub1bob");
        let list = checked(&collection, ann);
        assert_eq!(
            feed_check(&list, 1).findings,
            [Finding::CreditKey {
                role: "vocals".into(),
                group: String::new(),
                npub: "npub1city".into(),
                places: Vec::new(),
            }],
            "ADR 0111 item 10: a credit with the name of the person and a wrong key must be a finding."
        );
        assert_eq!(
            feed_check(&list, 2).findings,
            [Finding::CreditKey {
                role: "vocals".into(),
                group: String::new(),
                npub: String::new(),
                places: Vec::new(),
            }]
        );
        assert_eq!(
            feed_check(&list, 3).findings,
            [Finding::CreditDetails {
                role: "vocals".into(),
                img: true,
                href: false,
                feed_img: "old.png".into(),
                feed_href: String::new(),
                person_img: "ann.png".into(),
                person_href: String::new(),
                places: Vec::new(),
            }]
        );
        assert_eq!(
            feed_check(&list, 4).findings,
            [Finding::CreditRole { places: Vec::new() }]
        );
        assert_eq!(feed_check(&list, 4).state, CheckState::NeedsCorrection);
        assert_eq!(feed_check(&list, 5).state, CheckState::Correct);
        assert!(list.feeds.iter().all(|f| f.feed_id.0 != 6));
        assert_eq!(list.needs_correction(), 4);
    }

    #[test]
    fn a_difference_gives_the_values_and_the_places_of_the_credit() {
        let mut collection = Collection::open_in_memory().unwrap();
        let ann = add_person(
            &collection,
            &PersonFields {
                name: "Ann".into(),
                img: "ann.png".into(),
                ..PersonFields::default()
            },
        )
        .unwrap();
        let feed_id = collection.add_feed(None, None).unwrap();
        let doc = Document::parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:person role=\"vocals\" img=\"old.png\">Ann</podcast:person>\
             <item><guid>t1</guid><title>Neon Diner</title>\
             <podcast:person role=\"vocals\" img=\"old.png\">Ann</podcast:person></item>\
             </channel></rss>",
        )
        .unwrap();
        collection
            .record_state(feed_id, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        let list = checked(&collection, ann);
        assert_eq!(
            feed_check(&list, feed_id.0).findings,
            [Finding::CreditDetails {
                role: "vocals".into(),
                img: true,
                href: false,
                feed_img: "old.png".into(),
                feed_href: String::new(),
                person_img: "ann.png".into(),
                person_href: String::new(),
                places: vec![
                    CreditPlace::Channel,
                    CreditPlace::Item {
                        guid: "t1".into(),
                        title: "Neon Diner".into(),
                    },
                ],
            }],
            "ADR 0120 item 6: a finding gives the four values and the places of the credit."
        );
    }

    #[test]
    fn a_person_with_no_key_gives_no_key_finding() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        credit_row(&collection, 1, "Ann", "vocals", "", "");
        credit_row(&collection, 2, "Ann", "vocals", "", "npub1other");
        let list = checked(&collection, ann);
        assert_eq!(feed_check(&list, 1).state, CheckState::Correct);
        assert_eq!(
            feed_check(&list, 2).state,
            CheckState::Correct,
            "ADR 0111 item 9: the key check needs a key of the person."
        );
    }

    #[test]
    fn check_gives_not_default_for_a_member_and_other_wallet_for_a_payee_of_the_name() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        found(&collection, "ann@example.com", "node-1");
        payee(&collection, 1, "node-1", "Ann");
        payee(&collection, 2, "ann@example.com", "Ann");
        payee(&collection, 3, "node-9", " ANN ");
        payee(&collection, 4, "node-8", "Bob");
        let list = checked(&collection, ann);
        assert_eq!(feed_check(&list, 1).state, CheckState::Correct);
        assert_eq!(feed_check(&list, 2).state, CheckState::NotDefault);
        assert_eq!(
            feed_check(&list, 2).findings,
            [Finding::NotDefault {
                key: key("ann@example.com")
            }],
            "ADR 0111 item 9: a different member of the wallet must be the neutral finding."
        );
        assert_eq!(feed_check(&list, 3).state, CheckState::NeedsCorrection);
        assert_eq!(
            feed_check(&list, 3).findings,
            [Finding::OtherWallet { key: key("node-9") }]
        );
        assert!(list.feeds.iter().all(|f| f.feed_id.0 != 4));
        assert_eq!(list.needs_correction(), 1);
    }

    #[test]
    fn credits_with_one_name_and_one_npub_make_one_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann Lee", "artist", "", "npub1aaa");
        credit(&collection, 2, "Ann Lee", "artist", "", "npub1aaa");
        credit(&collection, 3, "ann  lee", "mixer", "", "NPUB1AAA");
        credit(&collection, 4, "", "", "", "npub1aaa");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Ann Lee");
        assert_eq!(list[0].evidence, [Evidence::SameNpub]);
        // The credit with no name has `same_name`, as ADR 0103 item 4 says.
        assert!(list[0].credits.iter().all(|c| c.same_name));
        let first = list[0]
            .credits
            .iter()
            .find(|c| c.name == "Ann Lee" && c.npub == "npub1aaa")
            .unwrap();
        assert_eq!(first.roles, ["artist"]);
        assert_eq!(first.img, "img.png");
        assert_eq!(list[0].credit_feeds, 4);
    }

    #[test]
    fn credits_with_one_npub_and_two_names_make_one_suggestion_for_each_name() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann Lee", "artist", "", "npub1aaa");
        credit(&collection, 2, "A. Lee", "mixer", "", "npub1aaa");
        let list = suggest(&collection);
        let same: Vec<(&str, &str, bool)> = list
            .iter()
            .map(|s| (s.identity.as_str(), s.name.as_str(), s.credits[0].same_name))
            .collect();
        assert_eq!(
            same,
            [
                ("name:a. lee", "A. Lee", true),
                ("name:ann lee", "Ann Lee", true)
            ]
        );
    }

    #[test]
    fn a_recipient_joins_by_name_and_by_wallet() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Mary Kate Ultra", "artist", "", "");
        payee(&collection, 2, "node-1", "mary kate  ultra");
        payee(&collection, 3, "mk@example.com", "M K U");
        found(&collection, "mk@example.com", "node-1");
        payee(&collection, 4, "unrelated", "Other");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        let s = &list[0];
        assert_eq!(s.evidence, [Evidence::SameName, Evidence::SameWallet]);
        let joined: Vec<(&str, JoinedBy, bool)> = s
            .recipients
            .iter()
            .map(|r| (r.key.address.as_str(), r.joined_by, r.same_name))
            .collect();
        // The Lightning address has the wallet key of the node.
        assert_eq!(
            joined,
            [
                ("mk@example.com", JoinedBy::Wallet, false),
                ("node-1", JoinedBy::Name, true)
            ]
        );
        assert_eq!((s.credit_feeds, s.payee_feeds), (1, 2));
    }

    #[test]
    fn credits_of_one_name_make_one_suggestion_with_or_without_an_href() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "https://ann.example", "");
        credit(&collection, 2, "ann", "", "", "");
        credit(&collection, 3, "Ann", "", "", "npub1ann");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].identity, "name:ann");
        assert_eq!(list[0].credits.len(), 3);
        assert_eq!(list[0].evidence, [Evidence::SameNpub]);
    }

    #[test]
    fn two_npubs_with_one_name_make_two_suggestions_and_one_for_the_rest() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "", "npub1aaa");
        credit(&collection, 2, "Ann", "", "", "npub1bbb");
        credit(&collection, 3, "Ann", "", "", "");
        credit(&collection, 4, "Ann", "", "https://ann.example", "");
        payee(&collection, 5, "node-1", "ann");
        let list = suggest(&collection);
        let mut identities: Vec<&str> = list.iter().map(|s| s.identity.as_str()).collect();
        identities.sort_unstable();
        assert_eq!(identities, ["name:ann", "npub:npub1aaa", "npub:npub1bbb"]);
        let rest = list.iter().find(|s| s.identity == "name:ann").unwrap();
        assert_eq!(rest.credits.len(), 2);
        assert!(rest.evidence.contains(&Evidence::SameName));
        // The payee joins the suggestion with no `npub` only.
        for s in &list {
            let expected = usize::from(s.identity == "name:ann");
            assert_eq!(s.recipients.len(), expected, "{}", s.identity);
        }
        let by_npub = list.iter().find(|s| s.identity == "npub:npub1aaa").unwrap();
        assert_eq!(by_npub.evidence, [Evidence::SameNpub]);
        assert!(by_npub.credits.iter().all(|c| c.same_name));
    }

    #[test]
    fn a_payee_joins_each_suggestion_of_a_name_when_all_have_an_npub() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "", "npub1aaa");
        credit(&collection, 2, "Ann", "", "", "npub1bbb");
        payee(&collection, 3, "node-1", "Ann");
        let list = suggest(&collection);
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|s| s.recipients.len() == 1));
    }

    fn publisher_and_album(collection: &mut Collection) -> (FeedId, FeedId) {
        let album = collection.add_feed(None, None).unwrap();
        let publisher = collection.add_feed(None, None).unwrap();
        for (feed, medium) in [(album, "music"), (publisher, "publisher")] {
            let doc = Document::parse(&format!(
                "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
                 <podcast:medium>{medium}</podcast:medium></channel></rss>"
            ))
            .unwrap();
            collection
                .record_state(feed, &Feed::read(&doc).unwrap(), &Version::new("v1"))
                .unwrap();
        }
        (album, publisher)
    }

    #[test]
    fn one_save_applies_the_fields_the_default_address_and_the_membership() {
        let mut collection = Collection::open_in_memory().unwrap();
        let (_, publisher) = publisher_and_album(&mut collection);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let key = RecipientKey {
            address: "ann@example.com".to_owned(),
            custom_key: None,
            custom_value: None,
        };
        add_wallet(&collection, ann, &key, "lnaddress", LinkEvidence::User).unwrap();
        let save = PersonSave {
            fields: Some(fields("Ann B")),
            default_address: Some(Some(key.clone())),
            members: Some(vec![publisher]),
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap();
        let saved = person(&collection, ann).unwrap();
        assert_eq!(saved.name, "Ann B");
        assert_eq!(saved.default_address, Some(key));
        assert_eq!(saved.members, [publisher]);
    }

    #[test]
    fn a_failed_save_changes_nothing() {
        let mut collection = Collection::open_in_memory().unwrap();
        let (album, _) = publisher_and_album(&mut collection);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let save = PersonSave {
            fields: Some(fields("Ann B")),
            members: Some(vec![album]),
            ..PersonSave::default()
        };
        let error = save_person(&collection, ann, &save).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
        assert_eq!(person(&collection, ann).unwrap().name, "Ann");
    }

    #[test]
    fn a_save_links_the_wallets_and_keys_or_changes_nothing() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        add_wallet(
            &collection,
            bob,
            &key("02other"),
            "node",
            LinkEvidence::User,
        )
        .unwrap();
        let new_wallet = WalletAdd {
            key: key("02new"),
            recipient_type: "node".to_owned(),
        };
        let conflict = WalletAdd {
            key: key("02other"),
            recipient_type: "node".to_owned(),
        };
        let save = PersonSave {
            wallets: vec![new_wallet.clone(), conflict],
            keys: vec!["npub1ann".to_owned()],
            ..PersonSave::default()
        };
        let error = save_person(&collection, ann, &save).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        let unchanged = person(&collection, ann).unwrap();
        assert!(unchanged.wallets.is_empty(), "a failed save adds no wallet");
        assert!(unchanged.keys.is_empty(), "a failed save adds no key");

        let save = PersonSave {
            wallets: vec![new_wallet],
            keys: vec!["npub1ann".to_owned()],
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap();
        let saved = person(&collection, ann).unwrap();
        assert_eq!(saved.wallets.len(), 1);
        assert_eq!(saved.wallets[0].key.address, "02new");
        assert_eq!(saved.wallets[0].evidence, "user");
        assert_eq!(saved.keys.len(), 1);
        assert_eq!(saved.keys[0].npub, "npub1ann");
        assert_eq!(person(&collection, bob).unwrap().wallets.len(), 1);
    }

    #[test]
    fn a_save_replaces_the_links_and_an_empty_list_removes_each() {
        let mut collection = Collection::open_in_memory().unwrap();
        let (_, first) = publisher_and_album(&mut collection);
        let (_, second) = publisher_and_album(&mut collection);
        let (album, third) = publisher_and_album(&mut collection);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_member(&collection, ann, first).unwrap();
        save_person(&collection, ann, &PersonSave::default()).unwrap();
        assert_eq!(person(&collection, ann).unwrap().members, [first]);
        let save = PersonSave {
            members: Some(vec![first, second]),
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap();
        assert_eq!(person(&collection, ann).unwrap().members, [first, second]);
        let save = PersonSave {
            members: Some(vec![second, third]),
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap();
        assert_eq!(person(&collection, ann).unwrap().members, [second, third]);
        // ADR 0132 item 9: a feed that is not a publisher feed changes nothing.
        let save = PersonSave {
            members: Some(vec![first, album]),
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap_err();
        assert_eq!(person(&collection, ann).unwrap().members, [second, third]);
        let save = PersonSave {
            members: Some(Vec::new()),
            ..PersonSave::default()
        };
        save_person(&collection, ann, &save).unwrap();
        assert_eq!(
            person(&collection, ann).unwrap().members,
            Vec::<FeedId>::new()
        );
    }

    #[test]
    fn a_wallet_gives_its_members_with_their_types() {
        let collection = Collection::open_in_memory().unwrap();
        collection
            .set_keysend_lookup(&Lookup {
                address: "ann@example.com".into(),
                pubkey: Some("node-1".into()),
                custom_key: Some("696969".into()),
                custom_value: Some("abc".into()),
                status: LookupStatus::Found,
                checked_at: 1,
            })
            .unwrap();
        let lookups = lookup_map(&collection).unwrap();
        let wallet = PersonWallet {
            id: 1,
            key: key("ann@example.com"),
            recipient_type: "lnaddress".to_owned(),
            position: 0,
            evidence: "user".to_owned(),
            linked_at: 0,
        };
        let members = wallet_members(&wallet, &lookups);
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].key.address, "node-1");
        assert_eq!(members[0].key.custom_key.as_deref(), Some("696969"));
        assert_eq!(members[0].key.custom_value.as_deref(), Some("abc"));
        assert_eq!(members[0].recipient_type, "node");
        assert_eq!(members[1].key.address, "ann@example.com");
        assert_eq!(members[1].recipient_type, "lnaddress");
        // A wallet with no lookup has only its own key.
        let lone = PersonWallet {
            key: key("node-9"),
            recipient_type: "node".to_owned(),
            ..wallet
        };
        assert_eq!(wallet_members(&lone, &lookups).len(), 1);
    }

    #[test]
    fn a_credit_with_only_an_npub_gets_its_npub_as_name_and_no_recipient_with_no_name() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "", "", "", "npub1solo");
        payee(&collection, 2, "node-1", "");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "npub1solo");
        assert_eq!(list[0].recipients, Vec::new());
        assert!(list[0].credits[0].same_name);
    }

    #[test]
    fn a_group_of_recipients_with_no_credit_is_no_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        payee(&collection, 1, "node-1", "Ann");
        payee(&collection, 2, "node-2", "Ann");
        assert_eq!(suggest(&collection), Vec::new());
    }

    #[test]
    fn the_feed_counts_use_distinct_feeds() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "artist", "", "");
        credit(&collection, 1, "Ann", "writer", "", "");
        credit(&collection, 2, "Ann", "artist", "", "");
        payee(&collection, 1, "node-1", "Ann");
        payee(&collection, 3, "node-1", "Ann");
        payee(&collection, 3, "node-2", "Ann");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].credits.len(), 1);
        assert_eq!(list[0].credits[0].roles, ["artist", "writer"]);
        assert_eq!(list[0].credits[0].feeds, [FeedId(1), FeedId(2)]);
        assert_eq!((list[0].credit_feeds, list[0].payee_feeds), (2, 2));
    }

    #[test]
    fn a_dismissed_suggestion_comes_back_with_a_new_item() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "", "");
        let before = suggest(&collection);
        dismiss(&collection, &before[0].identity, &before[0].item_keys).unwrap();
        assert_eq!(suggest(&collection), Vec::new());
        payee(&collection, 2, "node-1", "ann");
        let after = suggest(&collection);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].recipients.len(), 1);
        // A second dismissal replaces the first.
        dismiss(&collection, &after[0].identity, &after[0].item_keys).unwrap();
        assert_eq!(suggest(&collection), Vec::new());
    }

    #[test]
    fn the_links_of_a_person_are_not_in_a_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "", "");
        credit(&collection, 1, "Bob", "", "", "npub1bob");
        credit(&collection, 1, "Cy", "", "", "");
        payee(&collection, 1, "node-1", "Cy");
        payee(&collection, 1, "node-2", "Cy");
        let ann = add_person(&collection, &fields("Person")).unwrap();
        add_credit_name(&collection, ann, "ANN", "", "").unwrap();
        add_key(&collection, ann, "npub1bob").unwrap();
        add_wallet(&collection, ann, &key("node-1"), "node", LinkEvidence::User).unwrap();
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Cy");
        let addresses: Vec<&str> = list[0]
            .recipients
            .iter()
            .map(|r| r.key.address.as_str())
            .collect();
        assert_eq!(addresses, ["node-2"]);
    }

    #[test]
    fn suggestions_and_dismissals_make_no_person_and_no_link() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "artist", "", "npub1ann");
        payee(&collection, 1, "node-1", "Ann");
        let list = suggest(&collection);
        dismiss(&collection, &list[0].identity, &list[0].item_keys).unwrap();
        assert_eq!(counts(&collection), (0, 0));
    }

    #[test]
    fn suggestion_gives_the_person_with_the_same_normal_name() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann  Lee", "artist", "", "npub1ann");
        credit(&collection, 2, "Zed", "artist", "", "npub1zed");
        credit(&collection, 3, "Quinn", "artist", "", "npub1quinn");
        add_person(&collection, &fields("bob")).unwrap();
        let second = add_person(&collection, &fields("ann lee")).unwrap();
        let first = add_person(&collection, &fields("Other")).unwrap();
        add_credit_name(&collection, first, " ANN LEE ", "", "").unwrap();
        add_credit_name(&collection, second, "Zed", "", "").unwrap();
        let list = suggest(&collection);
        let by_name = |name: &str| list.iter().find(|s| s.name == name).unwrap().person_id;
        // The lowest ID wins when two people match.
        assert_eq!(by_name("Ann  Lee"), Some(PersonId(second.0.min(first.0))));
        assert_eq!(by_name("Quinn"), None);
    }

    #[test]
    fn make_person_makes_the_selected_links_only() {
        let collection = Collection::open_in_memory().unwrap();
        credit(
            &collection,
            1,
            "Ann",
            "artist",
            "https://ann.example",
            "npub1ann",
        );
        credit(
            &collection,
            2,
            "Ann",
            "artist",
            "https://ann.example",
            "npub1ann",
        );
        credit(
            &collection,
            2,
            "Ann",
            "mixer",
            "https://ann.example",
            "npub1ann",
        );
        payee(&collection, 1, "node-1", "Ann");
        payee(&collection, 1, "node-2", "Ann");
        collection
            .db()
            .execute("UPDATE feeds SET medium = 'publisher' WHERE id = 2", [])
            .unwrap();
        let lookups = HashMap::new();
        let id = make_person(
            &collection,
            &MakePerson {
                name: " Ann Lee ".into(),
                credits: vec![(
                    "Ann".into(),
                    "https://ann.example".into(),
                    "npub1ann".into(),
                )],
                recipients: vec![(key("node-1"), "node".into())],
                keys: vec!["NPUB1ann".into(), "npub1extra".into()],
                member_of: Some(FeedId(2)),
            },
            &lookups,
        )
        .unwrap();
        let p = person(&collection, id).unwrap();
        assert_eq!(p.name, "Ann Lee");
        assert_eq!(p.usual_role, "artist");
        assert_eq!(p.img, "img.png");
        assert_eq!(p.href, "https://ann.example");
        let npubs: Vec<&str> = p.keys.iter().map(|k| k.npub.as_str()).collect();
        assert_eq!(npubs, ["npub1ann", "npub1extra"]);
        assert_eq!(p.credit_names.len(), 1);
        assert_eq!(p.wallets.len(), 1);
        assert_eq!(p.wallets[0].key, key("node-1"));
        assert_eq!(p.wallets[0].recipient_type, "node");
        assert_eq!(p.members, [FeedId(2)]);
        assert_eq!(suggest(&collection), Vec::new());
    }

    #[test]
    fn a_conflict_makes_no_person() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, ann, &key("node-2"), "node", LinkEvidence::User).unwrap();
        let before = counts(&collection);
        let error = make_person(
            &collection,
            &MakePerson {
                name: "Bob".into(),
                credits: vec![("Bob".into(), String::new(), String::new())],
                recipients: vec![
                    (key("node-1"), "node".into()),
                    (key("node-2"), "node".into()),
                ],
                ..MakePerson::default()
            },
            &HashMap::new(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        assert_eq!(counts(&collection), before);
    }

    /// A relay that holds a kind 0 event with `lud16` for each address, and
    /// the `npub` of the key of each event.
    fn profile_relay(profiles: &[(u8, &str)]) -> (crate::testing::relay::TestRelay, Vec<String>) {
        use nostr::nips::nip19::ToBech32;
        use nostr::prelude::{Keys, Kind, SecretKey, Timestamp, UnsignedEvent};
        let relay = crate::testing::relay::TestRelay::start();
        let mut npubs = Vec::new();
        for (seed, address) in profiles {
            let keys = Keys::new(SecretKey::from_slice(&[*seed; 32]).expect("a secret key"));
            let unsigned = UnsignedEvent::new(
                keys.public_key(),
                Timestamp::from(100),
                Kind::Metadata,
                Vec::new(),
                format!(r#"{{"lud16":"{address}"}}"#),
            );
            relay.insert(crate::nostr_events::sign_with_keys(unsigned, &keys).expect("an event"));
            npubs.push(keys.public_key().to_bech32().expect("an npub"));
        }
        (relay, npubs)
    }

    const KEYSEND: &str = r#"{"status":"OK","tag":"keysend","pubkey":"node-1","customData":[{"customKey":"696969","customValue":"x1"}]}"#;

    /// A credit with an `href`, for the tests of the detail suggestions.
    fn credit_with_href(collection: &Collection, feed_id: i64, name: &str, img: &str, href: &str) {
        feed(collection, feed_id);
        collection
            .db()
            .execute(
                "INSERT INTO feed_credits (feed_id, name, role, href, npub, img, seen_at)
                 VALUES (?1, ?2, 'vocals', ?3, '', ?4, 0)",
                params![feed_id, name, href, img],
            )
            .unwrap();
    }

    fn member_feed(id: i64, image: Option<&str>, link: Option<&str>) -> MemberFeed {
        MemberFeed {
            id: FeedId(id),
            image: image.map(str::to_owned),
            link: link.map(str::to_owned),
        }
    }

    fn details(
        npub: &str,
        picture: Option<&str>,
        website: Option<&str>,
    ) -> profiles::ProfileDetails {
        profiles::ProfileDetails {
            npub: npub.to_owned(),
            picture: picture.map(str::to_owned),
            website: website.map(str::to_owned),
        }
    }

    #[test]
    fn detail_suggestions_join_equal_values_and_leave_out_the_current_value() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(
            &collection,
            &PersonFields {
                name: "Ann".into(),
                img: " https://cur.example/a.png ".into(),
                ..PersonFields::default()
            },
        )
        .unwrap();
        add_key(&collection, ann, "npub1ann").unwrap();
        credit_with_href(
            &collection,
            1,
            "Ann",
            "https://cur.example/a.png",
            "https://a.example",
        );
        credit_with_href(
            &collection,
            2,
            "Ann",
            "https://pic.example/a.png",
            "https://a.example",
        );
        credit_with_href(
            &collection,
            3,
            "Ann",
            " https://pic.example/a.png ",
            "https://a.example",
        );
        credit_with_href(
            &collection,
            4,
            "Bob",
            "https://bob.example/b.png",
            "https://b.example",
        );
        let person = person(&collection, ann).unwrap();
        let list = detail_suggestions(
            &collection,
            &person,
            &[],
            &[details("npub1ann", Some("https://pic.example/a.png"), None)],
        )
        .unwrap();
        assert_eq!(
            list.img,
            [DetailSuggestion {
                value: "https://pic.example/a.png".into(),
                sources: vec![
                    DetailSource::Profile {
                        npub: "npub1ann".into()
                    },
                    DetailSource::Credit {
                        feed_ids: vec![FeedId(2), FeedId(3)]
                    },
                ],
            }],
            "ADR 0113 item 3: equal values must join and the current value must not be a suggestion. Check people::detail_suggestions."
        );
        assert_eq!(list.href.len(), 1, "{:?}", list.href);
        assert_eq!(list.href[0].value, "https://a.example");
    }

    #[test]
    fn detail_suggestions_follow_the_sequence_profile_credit_member() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_key(&collection, ann, "npub1ann").unwrap();
        credit_with_href(&collection, 1, "Ann", "https://one.example/1.png", "");
        credit_with_href(&collection, 2, "Ann", "https://two.example/2.png", "");
        credit_with_href(&collection, 3, "Ann", "https://two.example/2.png", "");
        collection
            .db()
            .execute(
                "INSERT INTO feeds (id, medium, created_at) VALUES (9, 'publisher', 0)",
                [],
            )
            .unwrap();
        add_member(&collection, ann, FeedId(9)).unwrap();
        let person = person(&collection, ann).unwrap();
        let list = detail_suggestions(
            &collection,
            &person,
            &[
                member_feed(9, Some("https://member.example/m.png"), None),
                member_feed(8, Some("https://other.example/o.png"), None),
            ],
            &[details(
                "npub1ann",
                Some("https://profile.example/p.png"),
                None,
            )],
        )
        .unwrap();
        let values: Vec<&str> = list.img.iter().map(|s| s.value.as_str()).collect();
        assert_eq!(
            values,
            [
                "https://profile.example/p.png",
                "https://two.example/2.png",
                "https://one.example/1.png",
                "https://member.example/m.png",
            ],
            "ADR 0113 item 4: the sequence must be profile, credit by feed count, member. Check people::detail_suggestions."
        );
        assert_eq!(
            list.img[3].sources,
            [DetailSource::Member { feed_id: FeedId(9) }]
        );
    }

    #[test]
    fn detail_suggestions_refuse_a_value_that_is_not_a_web_address() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        credit_with_href(&collection, 1, "Ann", "ann.png", "ftp://a.example");
        credit_with_href(&collection, 2, "Ann", "HTTPS://up.example/a.png", "");
        let person = person(&collection, ann).unwrap();
        let list = detail_suggestions(&collection, &person, &[], &[]).unwrap();
        assert_eq!(list.img.len(), 1, "{:?}", list.img);
        assert_eq!(list.img[0].value, "HTTPS://up.example/a.png");
        assert!(
            list.href.is_empty(),
            "ADR 0113 item 3: only an http or https address counts. Check people::detail_suggestions."
        );
    }

    #[test]
    fn a_profile_with_a_picture_and_a_website_gives_profile_sources_and_a_silent_relay_keeps_the_others()
     {
        use nostr::nips::nip19::ToBech32;
        use nostr::prelude::{Keys, Kind, SecretKey, Timestamp, UnsignedEvent};
        let collection = Collection::open_in_memory().unwrap();
        let keys = Keys::new(SecretKey::from_slice(&[5; 32]).expect("a secret key"));
        let npub = keys.public_key().to_bech32().expect("an npub");
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::from(100),
            Kind::Metadata,
            Vec::new(),
            r#"{"picture":"https://p.example/a.png","website":"https://w.example"}"#.to_owned(),
        );
        let relay = crate::testing::relay::TestRelay::start();
        relay.insert(crate::nostr_events::sign_with_keys(unsigned, &keys).expect("an event"));
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_key(&collection, ann, &npub).unwrap();
        credit_with_href(
            &collection,
            1,
            "Ann",
            "https://c.example/a.png",
            "https://c.example",
        );
        let person = person(&collection, ann).unwrap();
        let read = |relays: &[String]| {
            let found = profiles::profile_details_by_key(
                relays,
                &NetPolicy::Open,
                profiles::SOURCE_LIMIT,
                std::slice::from_ref(&npub),
            );
            detail_suggestions(&collection, &person, &[], &found).unwrap()
        };
        let with = read(&[relay.url()]);
        assert_eq!(with.img.len(), 2, "{:?}", with.img);
        assert_eq!(
            with.img[0].sources,
            [DetailSource::Profile { npub: npub.clone() }],
            "ADR 0113 item 2: the picture of the profile must give a profile source."
        );
        assert_eq!(with.href[0].value, "https://w.example");
        // A relay that does not answer gives no profile source.
        let without = read(&[]);
        assert_eq!(without.img.len(), 1, "{:?}", without.img);
        assert!(
            matches!(without.img[0].sources[0], DetailSource::Credit { .. }),
            "ADR 0113 item 5: a relay that does not answer must leave the other sources."
        );
    }

    #[test]
    fn a_profile_address_is_a_suggestion_only_when_its_lookup_is_found() {
        let collection = Collection::open_in_memory().unwrap();
        let server = crate::testing::TestServer::start(vec![
            ("/.well-known/keysend/ann", 200, KEYSEND.as_bytes().to_vec()),
            ("/.well-known/keysend/bob", 404, b"no".to_vec()),
        ]);
        let port = server.port();
        let ann_address = format!("ann@127.0.0.1:{port}");
        let bob_address = format!("bob@127.0.0.1:{port}");
        let (relay, npubs) = profile_relay(&[(7, &ann_address), (8, &bob_address)]);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        for npub in &npubs {
            add_key(&collection, ann, npub).unwrap();
        }
        let person = person(&collection, ann).unwrap();
        let list = profile_addresses(
            &Web::new(),
            &NetPolicy::Open,
            &[relay.url()],
            "http",
            profiles::SOURCE_LIMIT,
            &person,
            &HashMap::new(),
            &HashSet::new(),
        );
        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(list[0].address, ann_address);
        assert_eq!(list[0].npub, npubs[0]);
        assert_eq!(list[0].wallet.address, "node-1");
        assert!(!list[0].in_person_wallet);
        assert_eq!(collection.keysend_lookups().unwrap().len(), 0);
    }

    #[test]
    fn a_dismissed_profile_address_is_not_a_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        let server = crate::testing::TestServer::start(vec![(
            "/.well-known/keysend/ann",
            200,
            KEYSEND.as_bytes().to_vec(),
        )]);
        let ann_address = format!("ann@127.0.0.1:{}", server.port());
        let (relay, npubs) = profile_relay(&[(7, &ann_address)]);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        add_key(&collection, ann, &npubs[0]).unwrap();
        dismiss_profile_address(
            &collection,
            ann,
            &format!("  {} ", ann_address.to_uppercase()),
        )
        .unwrap();
        dismiss_profile_address(&collection, ann, &ann_address).unwrap();
        let dismissed = dismissed_profile_addresses(&collection, ann).unwrap();
        assert_eq!(
            dismissed,
            HashSet::from([ann_address.to_lowercase()]),
            "ADR 0098 item 18: a dismissal keeps the address one time, in lowercase. Check dismiss_profile_address."
        );
        let list = profile_addresses(
            &Web::new(),
            &NetPolicy::Open,
            &[relay.url()],
            "http",
            profiles::SOURCE_LIMIT,
            &person(&collection, ann).unwrap(),
            &HashMap::new(),
            &dismissed,
        );
        assert!(
            list.is_empty(),
            "ADR 0098 item 18: \"Dismiss\" must hide the address for the person. Check the filter of profile_addresses. {list:?}"
        );
        assert!(dismiss_profile_address(&collection, PersonId(99), &ann_address).is_err());
        assert!(dismiss_profile_address(&collection, ann, " ").is_err());
    }

    #[test]
    fn a_profile_address_of_a_wallet_of_the_person_is_not_a_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        let server = crate::testing::TestServer::start(vec![
            ("/.well-known/keysend/ann", 200, KEYSEND.as_bytes().to_vec()),
            ("/.well-known/keysend/bob", 200, KEYSEND.as_bytes().to_vec()),
        ]);
        let port = server.port();
        let ann_address = format!("ann@127.0.0.1:{port}");
        let bob_address = format!("bob@127.0.0.1:{port}");
        let (relay, npubs) = profile_relay(&[(7, &ann_address), (8, &bob_address)]);
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        for npub in &npubs {
            add_key(&collection, ann, npub).unwrap();
        }
        add_wallet(
            &collection,
            ann,
            &key(&ann_address),
            "lnaddress",
            LinkEvidence::User,
        )
        .unwrap();
        let person = person(&collection, ann).unwrap();
        let ask = |lookups: &HashMap<String, Lookup>| {
            profile_addresses(
                &Web::new(),
                &NetPolicy::Open,
                &[relay.url()],
                "http",
                profiles::SOURCE_LIMIT,
                &person,
                lookups,
                &HashSet::new(),
            )
        };
        // With no stored lookup, only the address itself is a member.
        let list = ask(&HashMap::new());
        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(list[0].address, bob_address);
        // With a stored lookup, `bob` has the wallet key of the person.
        let stored = Lookup {
            address: ann_address.clone(),
            pubkey: Some("node-1".into()),
            custom_key: Some("696969".into()),
            custom_value: Some("x1".into()),
            status: LookupStatus::Found,
            checked_at: 1,
        };
        let lookups = HashMap::from([(ann_address, stored)]);
        let list = ask(&lookups);
        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(list[0].address, bob_address);
        assert!(list[0].in_person_wallet);
    }

    #[test]
    fn add_to_person_adds_only_the_new_items_and_keeps_the_fields() {
        let collection = Collection::open_in_memory().unwrap();
        let lookups = HashMap::new();
        let id = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, id, &key("node-1"), "node", LinkEvidence::User).unwrap();
        set_default_address(&collection, id, Some(&key("node-1"))).unwrap();
        let items = PersonItems {
            credits: vec![("Ann".into(), String::new(), "npub1ann".into())],
            recipients: vec![
                (key("node-1"), "node".into()),
                (key("node-2"), "node".into()),
            ],
            keys: vec!["npub1extra".into()],
        };
        let added = add_to_person(&collection, id, &items, &lookups).unwrap();
        // The credit name, the key of the credit, the extra key and node-2.
        assert_eq!(added, AddedItems { added: 4 });
        let again = add_to_person(&collection, id, &items, &lookups).unwrap();
        assert_eq!(again, AddedItems { added: 0 });
        let p = person(&collection, id).unwrap();
        assert_eq!(p.name, "Ann");
        assert_eq!(p.default_address, Some(key("node-1")));
        assert_eq!(p.wallets.len(), 2);
        assert_eq!(p.keys.len(), 2);
        assert_eq!(p.credit_names.len(), 1);
    }

    #[test]
    fn add_to_person_adds_nothing_on_a_conflict() {
        let collection = Collection::open_in_memory().unwrap();
        let lookups = HashMap::new();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        add_key(&collection, bob, "npub1bob").unwrap();
        let before = counts(&collection);
        let error = add_to_person(
            &collection,
            ann,
            &PersonItems {
                credits: vec![("Ann".into(), String::new(), String::new())],
                recipients: vec![(key("node-1"), "node".into())],
                keys: vec!["npub1bob".into()],
            },
            &lookups,
        )
        .unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        assert_eq!(counts(&collection), before);
        let error = add_to_person(&collection, PersonId(99), &PersonItems::default(), &lookups)
            .unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::NotFound);
    }

    fn more(collection: &Collection, id: PersonId) -> Vec<MoreItem> {
        let p = person(collection, id).unwrap();
        more_items(collection, &p, &lookup_map(collection).unwrap()).unwrap()
    }

    fn cause_of(item: &MoreItem) -> MoreCause {
        match item {
            MoreItem::Credit { cause, .. } | MoreItem::Payee { cause, .. } => *cause,
        }
    }

    #[test]
    fn more_items_give_each_match_with_its_cause_and_no_link() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "artist", "", "");
        credit(&collection, 1, "Annie", "mixer", "", "");
        credit(&collection, 2, "Someone", "artist", "", "npub1ann");
        credit(&collection, 2, "Other", "artist", "", "npub1other");
        credit(&collection, 3, "Ann", "artist", "https://ann.example", "");
        credit(
            &collection,
            3,
            "annie",
            "mixer",
            "https://annie.example",
            "",
        );
        payee(&collection, 1, "node-1", "ANN");
        payee(&collection, 1, "node-2", "Annie");
        payee(&collection, 1, "node-3", "Zed");
        payee(&collection, 2, "node-4", "Zed");
        payee(&collection, 2, "node-5", "Other");
        let id = add_person(&collection, &fields("Ann")).unwrap();
        add_credit_name(&collection, id, "Annie", "", "").unwrap();
        add_key(&collection, id, "npub1ann").unwrap();
        add_wallet(&collection, id, &key("node-3"), "node", LinkEvidence::User).unwrap();
        // A link of a different person is not in the list.
        let bob = add_person(&collection, &fields("Bob")).unwrap();
        add_credit_name(&collection, bob, "Ann", "https://ann.example", "").unwrap();
        add_wallet(&collection, bob, &key("node-5"), "node", LinkEvidence::User).unwrap();
        let before = counts(&collection);

        let list = more(&collection, id);
        let credits: Vec<(&str, MoreCause)> = list
            .iter()
            .filter_map(|i| match i {
                MoreItem::Credit { name, cause, .. } => Some((name.as_str(), *cause)),
                MoreItem::Payee { .. } => None,
            })
            .collect();
        assert_eq!(
            credits,
            vec![
                ("Ann", MoreCause::SameName),
                ("Someone", MoreCause::SameNpub),
                ("annie", MoreCause::SameCreditName),
            ],
            "{list:?}"
        );
        let payees: Vec<(String, MoreCause)> = list
            .iter()
            .filter_map(|i| match i {
                MoreItem::Payee { key, .. } => Some((key.address.clone(), cause_of(i))),
                MoreItem::Credit { .. } => None,
            })
            .collect();
        assert_eq!(
            payees,
            vec![
                ("node-1".to_owned(), MoreCause::SameName),
                ("node-2".to_owned(), MoreCause::SameCreditName),
            ],
            "{list:?}"
        );
        assert_eq!(counts(&collection), before);
    }

    #[test]
    fn more_items_give_a_payee_of_the_same_wallet() {
        let collection = Collection::open_in_memory().unwrap();
        payee(&collection, 1, "ann@wallet.example", "Zed");
        payee(&collection, 1, "node-1", "Zed");
        payee(&collection, 1, "node-9", "Zed");
        found(&collection, "ann@wallet.example", "node-1");
        let id = add_person(&collection, &fields("Ann")).unwrap();
        add_wallet(&collection, id, &key("node-1"), "node", LinkEvidence::User).unwrap();
        // The node key is a link. The Lightning address has the same wallet
        // key through its lookup. A different node key has no cause.
        let list = more(&collection, id);
        assert_eq!(list.len(), 1, "{list:?}");
        match &list[0] {
            MoreItem::Payee { key, cause, .. } => {
                assert_eq!(key.address, "ann@wallet.example");
                assert_eq!(*cause, MoreCause::SameWallet);
            }
            MoreItem::Credit { .. } => panic!("a payee was expected"),
        }
        // When a different person has the wallet, the item is not in the list.
        let other = add_person(&collection, &fields("Other")).unwrap();
        assert_eq!(more(&collection, other), Vec::<MoreItem>::new());
    }

    #[test]
    fn merge_people_moves_each_link_and_keeps_the_fields_of_the_kept_person() {
        let collection = Collection::open_in_memory().unwrap();
        feed(&collection, 1);
        feed(&collection, 2);
        collection
            .db()
            .execute("UPDATE feeds SET medium = 'publisher'", [])
            .unwrap();
        let keep = add_person(&collection, &fields("Ann")).unwrap();
        let other = add_person(&collection, &fields("Annie")).unwrap();
        add_wallet(
            &collection,
            keep,
            &key("node-1"),
            "node",
            LinkEvidence::User,
        )
        .unwrap();
        add_wallet(
            &collection,
            other,
            &key("node-2"),
            "node",
            LinkEvidence::User,
        )
        .unwrap();
        add_wallet(
            &collection,
            other,
            &key("node-3"),
            "node",
            LinkEvidence::User,
        )
        .unwrap();
        set_default_address(&collection, keep, Some(&key("node-1"))).unwrap();
        add_key(&collection, other, "npub1annie").unwrap();
        add_credit_name(&collection, other, "Annie", "", "").unwrap();
        add_support_link(&collection, keep, "A", "https://a.example").unwrap();
        add_support_link(&collection, other, "B", "https://b.example").unwrap();
        add_member(&collection, keep, FeedId(1)).unwrap();
        add_member(&collection, other, FeedId(1)).unwrap();
        add_member(&collection, other, FeedId(2)).unwrap();
        let set = collection
            .add_change_set("add", "node-2", 1, Some(other.0))
            .unwrap();

        merge_people(&collection, keep, other).unwrap();

        let p = person(&collection, keep).unwrap();
        assert_eq!(p.name, "Ann");
        assert_eq!(p.default_address, Some(key("node-1")));
        let addresses: Vec<&str> = p.wallets.iter().map(|w| w.key.address.as_str()).collect();
        assert_eq!(addresses, ["node-1", "node-2", "node-3"]);
        let positions: Vec<i64> = p.wallets.iter().map(|w| w.position).collect();
        assert_eq!(positions, [0, 1, 2]);
        assert_eq!(p.keys.len(), 1);
        assert_eq!(p.credit_names.len(), 1);
        let labels: Vec<&str> = p.support_links.iter().map(|l| l.label.as_str()).collect();
        assert_eq!(labels, ["A", "B"]);
        assert_eq!(p.members, vec![FeedId(1), FeedId(2)]);
        assert_eq!(
            person(&collection, other).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
        let owner = collection.change_set_row(set).unwrap().map(|c| c.person_id);
        assert_eq!(owner, Some(Some(keep.0)));
    }

    #[test]
    fn merge_people_refuses_the_same_person_and_an_unknown_person() {
        let collection = Collection::open_in_memory().unwrap();
        let ann = add_person(&collection, &fields("Ann")).unwrap();
        assert_eq!(
            merge_people(&collection, ann, ann).unwrap_err().kind(),
            CollectionErrorKind::Invalid
        );
        assert_eq!(
            merge_people(&collection, ann, PersonId(99))
                .unwrap_err()
                .kind(),
            CollectionErrorKind::NotFound
        );
    }
}
