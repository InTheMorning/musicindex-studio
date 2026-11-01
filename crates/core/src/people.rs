//! The people of one user (ADR 0098).
//!
//! A person has a name and links: wallets, Nostr keys, credit names, support
//! links and publisher feeds. The user makes each link. This module writes
//! no feed, and it keeps no secret. A value from a person is a start value
//! in a feed (ADR 0098 item 3).

use std::collections::HashMap;
use std::hash::BuildHasher;

use rusqlite::{Connection, OptionalExtension, params};

use crate::change_set::RecipientKey;
use crate::collection::{Collection, CollectionError, CollectionErrorKind, FeedId, now};
use crate::feed::Recipient;
use crate::keysend::{self, Lookup};

/// The ID of a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// A wallet that the user linked to a person.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'user', ?7)",
        params![
            id.0,
            address,
            custom_key,
            custom_value,
            recipient_type.trim(),
            position,
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
) -> Result<i64, CollectionError> {
    let lookups = lookup_map(collection)?;
    insert_wallet(collection.db(), id, key, recipient_type, &lookups)
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
        &lookups,
    )?;
    tx.commit()?;
    Ok(id)
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
        add_wallet(&collection, ann, &key("node-1"), "node").unwrap();
        found(&collection, "ann@example.com", "node-1");
        // The lookup of the Lightning address gives the wallet key of Ann.
        let error = add_wallet(&collection, bob, &key("ann@example.com"), "lnaddress").unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        let error = add_wallet(&collection, bob, &key("node-1"), "node").unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Conflict);
        // A second key of the same wallet in the same person is permitted.
        add_wallet(&collection, ann, &key("ann@example.com"), "lnaddress").unwrap();
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
        add_wallet(&collection, ann, &key("first"), "node").unwrap();
        let second = add_wallet(&collection, ann, &key("second"), "node").unwrap();
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
        add_wallet(&collection, ann, &key("node-1"), "node").unwrap();
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
        add_wallet(&collection, ann, &key("node-1"), "node").unwrap();
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
        add_wallet(&collection, ann, &key("node-1"), "node").unwrap();
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
}
