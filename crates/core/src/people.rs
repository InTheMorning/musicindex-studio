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
use crate::collection::{Collection, CollectionError, CollectionErrorKind, FeedId, now};
use crate::feed::Recipient;
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

fn lookup_map(collection: &Collection) -> Result<HashMap<String, Lookup>, CollectionError> {
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
        img: fields.img.trim().to_owned(),
        href: fields.href.trim().to_owned(),
    })
}

fn insert_person(db: &Connection, fields: &PersonFields) -> Result<PersonId, CollectionError> {
    let fields = clean_fields(fields)?;
    db.execute(
        "INSERT INTO people (name, usual_role, img, href, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            fields.name,
            fields.usual_role,
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
    let fields = clean_fields(fields)?;
    let changed = collection.db().execute(
        "UPDATE people SET name = ?2, usual_role = ?3, img = ?4, href = ?5 WHERE id = ?1",
        params![
            id.0,
            fields.name,
            fields.usual_role,
            fields.img,
            fields.href
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

fn wallets_of(db: &Connection, id: PersonId) -> Result<Vec<PersonWallet>, CollectionError> {
    let mut statement = db.prepare(
        "SELECT id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at
         FROM person_wallets WHERE person_id = ?1 ORDER BY position, id",
    )?;
    let rows = statement.query_map([id.0], |r| {
        let custom_key: String = r.get(2)?;
        let custom_value: String = r.get(3)?;
        Ok(PersonWallet {
            id: r.get(0)?,
            key: RecipientKey {
                address: r.get(1)?,
                custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                custom_value: Some(custom_value).filter(|t| !t.is_empty()),
            },
            recipient_type: r.get(4)?,
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
            "SELECT name, usual_role, img, href, default_address, default_custom_key, default_custom_value
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
            recipient_type.trim(),
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
    let url = url.trim();
    let lower = url.to_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(CollectionError::invalid(
            "a support link starts with https:// or http://",
        ));
    }
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
    let db = collection.db();
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
    let db = collection.db();
    let person = read_person(db, id)?;
    let (address, custom_key, custom_value) = match key {
        None => (None, "", ""),
        Some(key) => {
            if !is_member(&person, key, &lookups) {
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
    identity: Identity,
    credits: BTreeMap<(String, String, String), CreditAgg>,
    /// The feeds of each name, for the most frequent name.
    names: BTreeMap<String, BTreeSet<i64>>,
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

fn credit_groups(
    collection: &Collection,
    known_identities: &HashSet<String>,
    known_keys: &HashSet<String>,
) -> Result<BTreeMap<String, Group>, CollectionError> {
    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for row in collection.feed_credits()? {
        let Some(identity) = Identity::of(&row.name, &row.href, &row.npub) else {
            continue;
        };
        let key = identity.key();
        if known_identities.contains(&key)
            || matches!(&identity, Identity::Npub(npub) if known_keys.contains(npub))
        {
            continue;
        }
        let group = groups.entry(key).or_insert_with(|| Group {
            identity,
            credits: BTreeMap::new(),
            names: BTreeMap::new(),
        });
        // A credit with only an `npub` gives no name to its group.
        if !row.name.trim().is_empty() {
            group
                .names
                .entry(row.name.trim().to_owned())
                .or_default()
                .insert(row.feed_id.0);
        }
        let credit = group
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
}

impl FeedPlaces {
    fn new(feed_id: FeedId) -> Self {
        Self {
            feed_id,
            pays: Vec::new(),
            roles: Vec::new(),
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
            "SELECT feed_id, address, custom_key, custom_value FROM feed_recipients
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
            ))
        })?;
        for row in rows {
            let (feed, key) = row?;
            if wallets.contains(&parts(&keysend::wallet_key(&key, lookups))) {
                pays.insert(feed);
                let place = places
                    .entry(feed)
                    .or_insert_with(|| FeedPlaces::new(FeedId(feed)));
                if !place.pays.contains(&key) {
                    place.pays.push(key);
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
            places
                .entry(row.feed_id.0)
                .or_insert_with(|| FeedPlaces::new(row.feed_id))
                .roles
                .push(row.role.trim().to_owned());
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

    let mut list = Vec::new();
    for (identity_key, group) in groups {
        // The most frequent name. With a tie, the first name A to Z. A group
        // of credits with only an `npub` uses the `npub` as its name.
        let name = group
            .names
            .iter()
            .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| b.0.cmp(a.0)))
            .map(|(name, _)| name.clone());
        let group_name = name.as_deref().map(normal_name).unwrap_or_default();
        let name = name.unwrap_or_else(|| match &group.identity {
            Identity::Npub(npub) => npub.clone(),
            Identity::NameHref { name, .. } | Identity::Name(name) => name.clone(),
        });

        let joined = join_recipients(&recipients, &group_name);

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
        if matches!(group.identity, Identity::Npub(_)) {
            evidence.push(Evidence::SameNpub);
        }
        if joined.iter().any(|r| r.joined_by == JoinedBy::Name) {
            evidence.push(Evidence::SameName);
        }
        if joined.iter().any(|r| r.joined_by == JoinedBy::Wallet) {
            evidence.push(Evidence::SameWallet);
        }
        list.push(Suggestion {
            identity: identity_key,
            name,
            credits,
            recipients: joined,
            credit_feeds: credit_feeds.len(),
            payee_feeds: payee_feeds.len(),
            evidence,
            item_keys,
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

// ---- The Lightning address of a profile (ADR 0098 items 17 to 19) ----

/// The Lightning address in the profile of a key of a person, with a
/// keysend lookup that gives a wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileAddress {
    /// The Lightning address, in lowercase.
    pub address: String,
    /// The key of the wallet that the lookup gives.
    pub wallet: RecipientKey,
    /// True when `wallet` is the wallet key of a wallet of the person.
    pub in_person_wallet: bool,
}

/// The Lightning addresses of the Nostr keys of `person` that can be
/// added to a wallet (ADR 0098 items 17 to 19).
///
/// The function reads the `lud16` of the newest kind 0 of each key. It
/// leaves out an address that is a member of a wallet of the person
/// already. It requests the keysend data of each other address and gives
/// only an address with a `Found` result. It keeps no lookup, so the
/// collection has no lookup of a suggestion that the user did not add
/// (ADR 0097 item 3). Each request follows `policy` (ADR 0057).
#[must_use]
pub fn profile_addresses<S: BuildHasher>(
    web: &Web,
    policy: &NetPolicy,
    relays: &[String],
    scheme: &str,
    limit: Duration,
    person: &Person,
    lookups: &HashMap<String, Lookup, S>,
) -> Vec<ProfileAddress> {
    let npubs: Vec<String> = person.keys.iter().map(|k| k.npub.clone()).collect();
    let wanted: Vec<String> = profiles::lightning_addresses(relays, policy, limit, &npubs)
        .into_iter()
        .filter(|address| {
            let key = RecipientKey {
                address: address.clone(),
                custom_key: None,
                custom_value: None,
            };
            !is_member(person, &key, lookups)
        })
        .collect();
    let results: Vec<(&String, LookupResult)> = thread::scope(|scope| {
        let handles: Vec<_> = wanted
            .iter()
            .map(|address| {
                scope.spawn(move || {
                    let web = web.clone();
                    let scheme = scheme.to_owned();
                    let text = address.clone();
                    let result = profiles::within(keysend::LOOKUP_LIMIT, move || {
                        keysend::request(&web, &scheme, &text)
                    })
                    .unwrap_or(LookupResult::Failed);
                    (address, result)
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
        .filter_map(|(address, result)| {
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
                in_person_wallet: owned.contains(&parts(&wallet)),
                wallet,
            })
        })
        .collect()
}

/// Add the Lightning address of a profile to a wallet of a person. The
/// function requests and stores the keysend lookup of `address`. With a
/// `Found` result, it links the wallet of the lookup with the type
/// `lnaddress` and the evidence `nostr`.
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
    add_wallet(collection, id, &key, "lnaddress", LinkEvidence::Nostr)
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

    #[test]
    fn credits_with_the_same_npub_make_one_suggestion() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann Lee", "artist", "", "npub1aaa");
        credit(&collection, 2, "Ann Lee", "artist", "", "npub1aaa");
        credit(&collection, 3, "A. Lee", "mixer", "", "npub1aaa");
        let list = suggest(&collection);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Ann Lee");
        assert_eq!(list[0].evidence, [Evidence::SameNpub]);
        let same: Vec<(&str, bool)> = list[0]
            .credits
            .iter()
            .map(|c| (c.name.as_str(), c.same_name))
            .collect();
        assert_eq!(same, [("A. Lee", false), ("Ann Lee", true)]);
        assert_eq!(list[0].credits[1].roles, ["artist"]);
        assert_eq!(list[0].credits[1].img, "img.png");
        assert_eq!(list[0].credit_feeds, 3);
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
    fn a_credit_with_an_href_and_one_without_make_two_suggestions() {
        let collection = Collection::open_in_memory().unwrap();
        credit(&collection, 1, "Ann", "", "https://ann.example", "");
        credit(&collection, 2, "Ann", "", "", "");
        let identities: Vec<String> = suggest(&collection)
            .into_iter()
            .map(|s| s.identity)
            .collect();
        assert_eq!(identities.len(), 2);
        assert!(identities.contains(&"name:ann".to_owned()));
        assert!(identities.contains(&"href:ann\u{1f}https://ann.example".to_owned()));
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
        );
        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(list[0].address, ann_address);
        assert_eq!(list[0].wallet.address, "node-1");
        assert!(!list[0].in_person_wallet);
        assert_eq!(collection.keysend_lookups().unwrap().len(), 0);
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
}
