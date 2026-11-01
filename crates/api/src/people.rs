//! The routes of the people (ADR 0098, ADR 0099).
//!
//! Only the routes of the profile addresses make a request to a host or a
//! relay. The other routes read the lookups that the collection keeps.

use std::collections::{HashMap, HashSet};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::change_set::{
    self, Cause, Outcome, PersonOperation, RecipientKey, RecipientOperation, Wallet,
};
use studio_core::collection::{Collection, CollectionError, CollectionErrorKind, FeedId};
use studio_core::keysend::Lookup;
use studio_core::people::{self, LinkEvidence, MakePerson, Person, PersonFields, PersonId};
use studio_core::pipeline::{PipelineError, PipelineErrorKind};
use studio_core::profiles;

use crate::error::ApiError;
use crate::routes::{
    ApiResult, Shared, blocking, blocking_unlocked, change_set_json, ensure_draft, files_feed,
};
use crate::workspace::Workspace;

/// The core refuses a value of a person with `Invalid`. This route family
/// names that refusal `invalid_person`. The other errors keep the mapping
/// of the collection.
fn person_error(error: CollectionError) -> ApiError {
    if error.kind() == CollectionErrorKind::Invalid {
        return ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_person",
            error.to_string(),
        );
    }
    error.into()
}

/// The stored keysend lookups, by lowercase address.
fn lookups(collection: &Collection) -> Result<HashMap<String, Lookup>, ApiError> {
    Ok(collection
        .keysend_lookups()?
        .into_iter()
        .map(|lookup| (lookup.address.clone(), lookup))
        .collect())
}

/// A person as JSON, with the default address that applies now and the
/// feeds that use the person. The list gives only the counts.
fn person_json(
    collection: &Collection,
    person: &Person,
    lookups: &HashMap<String, Lookup>,
    counts_only: bool,
) -> Result<Value, ApiError> {
    let uses = people::uses(collection, person, lookups)?;
    let mut value = serde_json::to_value(person).map_err(|e| ApiError::internal(e.to_string()))?;
    value["default_address"] = json!(people::default_address(person, lookups));
    let feeds: HashSet<_> = uses
        .pays
        .iter()
        .chain(&uses.credited)
        .map(|f| f.0)
        .collect();
    value["feeds"] = json!(feeds.len());
    if counts_only {
        value["pays"] = json!(uses.pays.len());
        value["credited"] = json!(uses.credited.len());
    } else {
        value["pays"] = json!(uses.pays);
        value["credited"] = json!(uses.credited);
        value["places"] = json!(uses.places);
    }
    Ok(value)
}

fn created(id: i64) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({ "id": id })))
}

/// Each person, with the counts of the feeds that use the person.
pub(crate) async fn list_people(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let lookups = lookups(&w.collection)?;
        let mut list = Vec::new();
        for person in people::people(&w.collection)? {
            list.push(person_json(&w.collection, &person, &lookups, true)?);
        }
        Ok(Json(Value::Array(list)))
    })
    .await
}

pub(crate) async fn add_person(
    State(shared): State<Shared>,
    Json(fields): Json<PersonFields>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let id = people::add_person(&w.collection, &fields).map_err(person_error)?;
        Ok(created(id.0))
    })
    .await
}

/// One person with each link and the feeds that use the person.
pub(crate) async fn get_person(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let person = people::person(&w.collection, PersonId(id))?;
        let lookups = lookups(&w.collection)?;
        Ok(Json(person_json(&w.collection, &person, &lookups, false)?))
    })
    .await
}

pub(crate) async fn change_person(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(fields): Json<PersonFields>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::change_person(&w.collection, PersonId(id), &fields).map_err(person_error)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

pub(crate) async fn remove_person(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::remove_person(&w.collection, PersonId(id))?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The body of `PUT /people/{id}/default-address`: a recipient key or
/// `null`.
pub(crate) async fn set_default_address(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(key): Json<Option<RecipientKey>>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::set_default_address(&w.collection, PersonId(id), key.as_ref())
            .map_err(person_error)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

#[derive(Deserialize)]
struct WalletBody {
    #[serde(flatten)]
    key: RecipientKey,
    #[serde(default)]
    recipient_type: String,
}

#[derive(Deserialize)]
struct KeyBody {
    npub: String,
}

#[derive(Deserialize)]
struct CreditNameBody {
    #[serde(default)]
    name: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    npub: String,
}

#[derive(Deserialize)]
struct SupportLinkBody {
    #[serde(default)]
    label: String,
    url: String,
}

#[derive(Deserialize)]
struct MemberBody {
    feed_id: FeedId,
}

fn unknown_kind(kind: &str) -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        format!("a person has no links of the kind {kind}"),
    )
}

fn bad_body(error: &serde_json::Error) -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_person",
        error.to_string(),
    )
}

/// Add a link to a person. `kind` is `wallets`, `keys`, `credit-names`,
/// `support-links` or `members`. The result is the ID of the link, and for
/// a member it is the feed ID.
pub(crate) async fn add_link(
    State(shared): State<Shared>,
    Path((id, kind)): Path<(i64, String)>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let id = PersonId(id);
        let c = &w.collection;
        let link = match kind.as_str() {
            "wallets" => {
                let body: WalletBody = serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::add_wallet(c, id, &body.key, &body.recipient_type, LinkEvidence::User)
            }
            "keys" => {
                let body: KeyBody = serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::add_key(c, id, &body.npub)
            }
            "credit-names" => {
                let body: CreditNameBody =
                    serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::add_credit_name(c, id, &body.name, &body.href, &body.npub)
            }
            "support-links" => {
                let body: SupportLinkBody =
                    serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::add_support_link(c, id, &body.label, &body.url)
            }
            "members" => {
                let body: MemberBody = serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::add_member(c, id, body.feed_id).map(|()| body.feed_id.0)
            }
            other => return Err(unknown_kind(other)),
        };
        Ok(created(link.map_err(person_error)?))
    })
    .await
}

/// Remove a link of a person. For `members`, `link_id` is the feed ID.
pub(crate) async fn remove_link(
    State(shared): State<Shared>,
    Path((id, kind, link_id)): Path<(i64, String, i64)>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let id = PersonId(id);
        let c = &w.collection;
        match kind.as_str() {
            "wallets" => people::remove_wallet(c, id, link_id),
            "keys" => people::remove_key(c, id, link_id),
            "credit-names" => people::remove_credit_name(c, id, link_id),
            "support-links" => people::remove_support_link(c, id, link_id),
            "members" => people::remove_member(c, id, FeedId(link_id)),
            other => return Err(unknown_kind(other)),
        }?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// Each suggestion of a person from the feeds (ADR 0098 items 12 to 16).
pub(crate) async fn list_suggestions(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let lookups = lookups(&w.collection)?;
        let list = people::suggestions(&w.collection, &lookups)?;
        Ok(Json(json!(list)))
    })
    .await
}

/// Make a person from the items that the user selected.
pub(crate) async fn make_person(
    State(shared): State<Shared>,
    Json(request): Json<MakePerson>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let lookups = lookups(&w.collection)?;
        let id = people::make_person(&w.collection, &request, &lookups).map_err(person_error)?;
        Ok(created(id.0))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct DismissBody {
    identity: String,
    #[serde(default)]
    items: Vec<String>,
}

/// Keep the dismissal of a suggestion.
pub(crate) async fn dismiss_suggestion(
    State(shared): State<Shared>,
    Json(body): Json<DismissBody>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::dismiss(&w.collection, &body.identity, &body.items).map_err(person_error)?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

/// The Lightning addresses in the profiles of the Nostr keys of a person
/// (ADR 0098 items 17 to 19). The answer is `{ "suggestions": [...] }`. A
/// shell with no keysend scheme gives no suggestion. The requests run
/// with no lock of the workspace.
pub(crate) async fn list_profile_addresses(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    let facts = blocking(shared, move |w| {
        let person = people::person(&w.collection, PersonId(id))?;
        let Some(scheme) = w.keysend_scheme else {
            return Ok(None);
        };
        Ok(Some((
            w.web.clone(),
            w.net_policy.clone(),
            w.lookup_relays.clone(),
            scheme,
            person,
            lookups(&w.collection)?,
        )))
    })
    .await?;
    let Some((web, policy, relays, scheme, person, lookups)) = facts else {
        return Ok(Json(json!({ "suggestions": [] })));
    };
    blocking_unlocked(move || {
        let list = people::profile_addresses(
            &web,
            &policy,
            &relays,
            scheme,
            profiles::SOURCE_LIMIT,
            &person,
            &lookups,
        );
        Ok(Json(json!({ "suggestions": list })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct ProfileAddressBody {
    address: String,
}

/// Add the Lightning address of a profile to a wallet of a person. The
/// core stores the lookup of the address.
pub(crate) async fn add_profile_address(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<ProfileAddressBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let Some(scheme) = w.keysend_scheme else {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_person",
                "this shell makes no keysend lookup",
            ));
        };
        let link =
            people::add_profile_address(&w.collection, &w.web, scheme, PersonId(id), &body.address)
                .map_err(person_error)?;
        Ok(created(link))
    })
    .await
}

/// The people as a JSON file (ADR 0098 items 24 and 25).
pub(crate) async fn export_people(State(shared): State<Shared>) -> Result<Response, ApiError> {
    let export = blocking(shared, |w| {
        let lookups = lookups(&w.collection)?;
        Ok(people::export(&w.collection, &lookups)?)
    })
    .await?;
    Ok((
        [(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"studio-people.json\"",
        )],
        Json(export),
    )
        .into_response())
}

/// The body of the two routes of a person change set.
#[derive(Deserialize)]
pub(crate) struct PersonChanges {
    operation: PersonOperation,
    feed_ids: Vec<i64>,
}

/// Check the body of a person change set, and make the snapshot of the
/// operation. An operation that is not valid gives 422
/// `invalid_person_operation` (ADR 0098 item 20).
fn person_snapshot(
    w: &Workspace,
    id: i64,
    body: &PersonChanges,
) -> Result<(Wallet, RecipientOperation, Vec<FeedId>), ApiError> {
    if body.feed_ids.is_empty() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_change_set",
            "a change set needs at least one feed",
        ));
    }
    // A person that does not exist gives 404, not a refusal of the operation.
    people::person(&w.collection, PersonId(id))?;
    // Only a refused operation is 422. A failure of the collection keeps
    // its own status.
    let invalid = |error: PipelineError| {
        if error.kind() == PipelineErrorKind::InvalidOperation {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_person_operation",
                error.to_string(),
            )
        } else {
            error.into()
        }
    };
    let (wallet, snapshot) = change_set::person_operation(
        &w.collection,
        PersonId(id),
        &body.operation,
        &lookups(&w.collection)?,
    )
    .map_err(invalid)?;
    change_set::check_operation(&snapshot).map_err(invalid)?;
    let feeds: Vec<FeedId> = body.feed_ids.iter().map(|&id| FeedId(id)).collect();
    for &feed in &feeds {
        w.collection.feed(feed)?;
    }
    Ok((wallet, snapshot, feeds))
}

/// Make the change set of a person operation. The route writes nothing to a
/// file host (ADR 0098 item 20).
pub(crate) async fn make_changes(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<PersonChanges>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let (_, _, feeds) = person_snapshot(w, id, &body)?;
        let set = {
            let workspace: &Workspace = w;
            change_set::make_person_change_set(
                &workspace.collection,
                &mut |feed| ensure_draft(workspace, feed).map_err(|e| e.coded()),
                PersonId(id),
                &body.operation,
                &feeds,
            )?
        };
        Ok((StatusCode::CREATED, Json(change_set_json(w, &set))))
    })
    .await
}

/// The outcome of a person operation for each feed. The route applies the
/// operation to a copy of the model of each feed. It saves nothing and makes
/// no draft.
pub(crate) async fn preview_changes(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<PersonChanges>,
) -> ApiResult {
    blocking(shared, move |w| {
        let (wallet, snapshot, feeds) = person_snapshot(w, id, &body)?;
        let mut seen = HashSet::new();
        let mut rows = Vec::new();
        for feed_id in feeds.into_iter().filter(|&feed| seen.insert(feed)) {
            let outcome = match files_feed(w, feed_id)? {
                Some(mut feed) => change_set::apply(&mut feed, &wallet, &snapshot),
                None => Outcome::NotChanged {
                    cause: Cause::CannotStart("the feed has no model".to_owned()),
                    parts: Vec::new(),
                },
            };
            rows.push(json!({ "feed_id": feed_id.0, "outcome": outcome }));
        }
        Ok(Json(json!({ "feeds": rows })))
    })
    .await
}
