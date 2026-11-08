//! The routes of the people (ADR 0098, ADR 0099).
//!
//! Only the routes of the profile addresses make a request to a host or a
//! relay. The other routes read the lookups that the collection keeps.

use std::collections::{HashMap, HashSet};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::change_set::{
    self, Cause, Outcome, PersonOperation, RecipientKey, RecipientOperation, Wallet,
};
use studio_core::collection::{Collection, CollectionError, CollectionErrorKind, FeedId};
use studio_core::keysend::{self, Lookup};
use studio_core::lookup;
use studio_core::people::{
    self, LinkEvidence, MakePerson, MemberFeed, Person, PersonFields, PersonId, PersonItems,
    PersonSave,
};
use studio_core::pipeline::{PipelineError, PipelineErrorKind};
use studio_core::profiles;

use crate::error::ApiError;
use crate::routes::search_request;
use crate::routes::{
    ApiResult, Shared, blocking, blocking_unlocked, change_set_json, ensure_draft, files_feed,
    now_seconds,
};
use crate::workspace::Workspace;
use studio_core::musicindex::{IndexEntity, field_list, plan_publisher_search, plan_search};

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
pub(crate) fn lookups(collection: &Collection) -> Result<HashMap<String, Lookup>, ApiError> {
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
    // Each wallet gives its members, for the menu of the default address.
    for (wallet, json_wallet) in person.wallets.iter().zip(
        value["wallets"]
            .as_array_mut()
            .map(|a| a.iter_mut())
            .into_iter()
            .flatten(),
    ) {
        json_wallet["members"] = json!(people::wallet_members(wallet, lookups));
    }
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
        // ADR 0140 item 3: where each credit name is in the feeds.
        value["credit_name_uses"] = json!(people::credit_name_uses(collection, person)?);
        // ADR 0132 item 4: the links with a credit of the person.
        value["linked_credits"] = json!(people::linked_credits(collection, person)?);
        // ADR 0111 item 12: the number of feeds that need a correction.
        value["needs_correction"] =
            json!(people::check(collection, person, lookups)?.needs_correction());
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

/// The check of the feeds of one person (ADR 0111 items 9 to 13). The route
/// reads the collection and makes no request.
pub(crate) async fn check_person(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let person = people::person(&w.collection, PersonId(id))?;
        let lookups = lookups(&w.collection)?;
        let check = people::check(&w.collection, &person, &lookups)?;
        Ok(Json(json!(check)))
    })
    .await
}

/// The role list of a credit (ADR 0112 item 3). It needs no open vault.
pub(crate) async fn list_roles() -> Json<Value> {
    let groups: Vec<Value> = studio_core::roles::groups()
        .iter()
        .map(|g| json!({ "name": g.name, "roles": g.roles }))
        .collect();
    Json(json!({ "groups": groups }))
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

/// The body of `POST /people/{id}/save`: the pending changes of the page
/// (ADR 0116). The core applies them in one transaction. Before the save,
/// Studio stores the keysend lookup of each Lightning address of the save, as
/// `add_profile_address` does. The core then finds the person of its node,
/// and the wallet joins the address and the node (ADR 0095, ADR 0117 item 7).
pub(crate) async fn save_person(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(save): Json<PersonSave>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let addresses: Vec<String> = save
            .wallets
            .iter()
            .filter(|wallet| wallet.recipient_type == "lnaddress")
            .map(|wallet| wallet.key.address.clone())
            .collect();
        if let Some(scheme) = w.keysend_scheme
            && !addresses.is_empty()
        {
            keysend::refresh(&w.collection, &w.web, scheme, &addresses, now_seconds())?;
        }
        people::save_person(&w.collection, PersonId(id), &save).map_err(person_error)?;
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
    /// The form sends `null` for a credit name with no key.
    #[serde(default)]
    npub: Option<String>,
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
                people::add_credit_name(
                    c,
                    id,
                    &body.name,
                    &body.href,
                    body.npub.as_deref().unwrap_or_default(),
                )
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

/// Change a link of a person. Only a wallet can change (ADR 0141 item 6).
pub(crate) async fn change_link(
    State(shared): State<Shared>,
    Path((id, kind, link_id)): Path<(i64, String, i64)>,
    Json(body): Json<Value>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = PersonId(id);
        let c = &w.collection;
        let new_id = match kind.as_str() {
            "wallets" => {
                let body: WalletBody = serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::change_wallet(c, id, link_id, &body.key, &body.recipient_type)
            }
            "credit-names" => {
                let body: CreditNameBody =
                    serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::change_credit_name(
                    c,
                    id,
                    link_id,
                    &body.name,
                    &body.href,
                    body.npub.as_deref().unwrap_or_default(),
                )
                .map(|()| link_id)
            }
            "support-links" => {
                let body: SupportLinkBody =
                    serde_json::from_value(body).map_err(|e| bad_body(&e))?;
                people::change_support_link(c, id, link_id, &body.label, &body.url)
                    .map(|()| link_id)
            }
            _ => return Err(unknown_kind(&kind)),
        }
        .map_err(person_error)?;
        Ok(Json(json!({ "id": new_id })))
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

/// Add the items that the user selected to a person (ADR 0103 item 10).
/// The answer is `{ "added": N }`.
pub(crate) async fn add_items(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(items): Json<PersonItems>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let lookups = lookups(&w.collection)?;
        let added = people::add_to_person(&w.collection, PersonId(id), &items, &lookups)
            .map_err(person_error)?;
        Ok((StatusCode::CREATED, Json(json!(added))))
    })
    .await
}

/// The items of the feeds that match a person (ADR 0103 items 11 to 13).
/// The answer is `{ "items": [...] }`.
pub(crate) async fn more_items(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let person = people::person(&w.collection, PersonId(id))?;
        let lookups = lookups(&w.collection)?;
        let items = people::more_items(&w.collection, &person, &lookups)?;
        Ok(Json(json!({ "items": items })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct MergeBody {
    other: i64,
}

/// Merge the person `other` into the person of the path (ADR 0103 items 14
/// to 16).
pub(crate) async fn merge_person(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<MergeBody>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::merge_people(&w.collection, PersonId(id), PersonId(body.other))
            .map_err(person_error)?;
        Ok(StatusCode::NO_CONTENT)
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
            people::dismissed_profile_addresses(&w.collection, PersonId(id))?,
        )))
    })
    .await?;
    let Some((web, policy, relays, scheme, person, lookups, dismissed)) = facts else {
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
            &dismissed,
        );
        Ok(Json(json!({ "suggestions": list })))
    })
    .await
}

/// The suggestions for the picture and the information address of a person
/// (ADR 0113). The answer is `{ "img": [...], "href": [...] }`. The reads of
/// the person and of the feeds, and the join, run under the workspace. The
/// requests to the relays run with no lock of the workspace. A shell with no
/// lookup relay gives no `profile` source.
pub(crate) async fn detail_suggestions(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    let (person, feeds, relays, policy) = blocking(shared.clone(), move |w| {
        let person = people::person(&w.collection, PersonId(id))?;
        let mut feeds = Vec::new();
        for feed_id in &person.members {
            let feed = files_feed(w, *feed_id).ok().flatten();
            feeds.push(MemberFeed {
                id: *feed_id,
                image: feed.as_ref().and_then(|f| f.image.clone()),
                link: feed.as_ref().and_then(|f| f.link.clone()),
            });
        }
        Ok((person, feeds, w.lookup_relays.clone(), w.net_policy.clone()))
    })
    .await?;
    let npubs: Vec<String> = person.keys.iter().map(|k| k.npub.clone()).collect();
    let found = blocking_unlocked(move || {
        Ok(profiles::profile_details_by_key(
            &relays,
            &policy,
            profiles::SOURCE_LIMIT,
            &npubs,
        ))
    })
    .await?;
    blocking(shared, move |w| {
        let list = people::detail_suggestions(&w.collection, &person, &feeds, &found)?;
        Ok(Json(json!(list)))
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

/// The query of the musicindex search of a person.
#[derive(Deserialize)]
pub(crate) struct MusicIndexQuery {
    #[serde(default)]
    q: String,
    /// True for a start text that the person did not edit (ADR 0151 item 3).
    #[serde(default)]
    plain: bool,
}

/// Search the artist and label feeds of musicindex for a name, in three
/// levels (ADR 0117 items 3 and 4). The answer has the `rows`, a `note`, and
/// `searched`, which is true when a request ran (ADR 0151 item 9). The request
/// runs with no lock of the workspace.
pub(crate) async fn search_musicindex(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Query(query): Query<MusicIndexQuery>,
) -> ApiResult {
    let index = blocking(shared, move |w| {
        people::person(&w.collection, PersonId(id)).map_err(person_error)?;
        Ok(w.index.clone())
    })
    .await?;
    let request = match search_request(plan_publisher_search(&query.q, query.plain))? {
        Ok(request) => request,
        Err(note) => {
            return Ok(Json(json!({ "rows": [], "note": note, "searched": false })));
        }
    };
    blocking_unlocked(move || {
        let rows = lookup::search(&index, &request)?;
        Ok(Json(
            json!({ "rows": rows, "note": null, "searched": true }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct IndexValuesQuery {
    /// True when the sheet also lists the credits of the feed and its tracks.
    #[serde(default)]
    credits: bool,
}

/// The one search of the sheets that add a value (ADR 0145 item 3): the artist
/// and label feeds in `rows`, and the other feeds in `feeds`. The requests run
/// with no lock of the workspace.
pub(crate) async fn search_index(
    State(shared): State<Shared>,
    Query(query): Query<MusicIndexQuery>,
) -> ApiResult {
    let request = match search_request(plan_search(&query.q, query.plain))? {
        Ok(request) => request,
        Err(note) => return Ok(Json(json!({ "rows": [], "feeds": [], "note": note }))),
    };
    let index = blocking(shared, move |w| Ok(w.index.clone())).await?;
    blocking_unlocked(move || {
        let rows = lookup::search(&index, &request)?;
        let feeds = index.search(&request)?;
        Ok(Json(json!({ "rows": rows, "feeds": feeds, "note": null })))
    })
    .await
}

/// The query of the field list.
#[derive(Deserialize)]
pub(crate) struct FieldsQuery {
    #[serde(default)]
    q: String,
    /// The cursor in UTF-16 code units. A missing cursor is the text end.
    cursor: Option<usize>,
    /// `publisher` leaves out the fields that give no request for artist and
    /// label feeds.
    scope: Option<String>,
}

/// The rows of the field list for a text and a cursor (musicindex ADR 0052
/// R-QUERY-7). The core gives each edit range. The route sends no request.
pub(crate) async fn field_rows(Query(query): Query<FieldsQuery>) -> ApiResult {
    let cursor = query
        .cursor
        .unwrap_or_else(|| query.q.encode_utf16().count());
    let scope = match query.scope.as_deref() {
        None | Some("") => None,
        Some("publisher") => Some(IndexEntity::Publisher),
        Some(other) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "bad_scope",
                format!("The scope {other} is not known. Use publisher or leave it out."),
            ));
        }
    };
    Ok(Json(json!(field_list(&query.q, cursor, scope))))
}

/// The query of the artist names.
#[derive(Deserialize)]
pub(crate) struct ArtistNamesQuery {
    #[serde(default)]
    from: String,
}

/// The artist names that start with a typed text, for the name rows of the
/// field list. The request runs with no lock of the workspace.
pub(crate) async fn artist_names(
    State(shared): State<Shared>,
    Query(query): Query<ArtistNamesQuery>,
) -> ApiResult {
    let index = blocking(shared, move |w| Ok(w.index.clone())).await?;
    blocking_unlocked(move || {
        let names = index.artist_names(&query.from)?;
        Ok(Json(json!({ "names": names })))
    })
    .await
}

/// The values of one feed of a search (ADR 0145 item 3). Gives 404 when
/// musicindex does not know the feed.
pub(crate) async fn index_values(
    State(shared): State<Shared>,
    Path(guid): Path<String>,
    Query(query): Query<IndexValuesQuery>,
) -> ApiResult {
    let (index, known) = blocking(shared, move |w| {
        Ok((w.index.clone(), lookups(&w.collection)?))
    })
    .await?;
    blocking_unlocked(move || {
        let values = index.publisher_values(&guid)?.ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "not_in_index",
                format!("Music Index has no feed {guid}"),
            )
        })?;
        let mut body = json!(values);
        crate::routes::name_providers(&mut body["routes"], &known);
        if query.credits {
            body["credits"] = json!(index.feed_credits(&guid)?);
        }
        Ok(Json(body))
    })
    .await
}

/// The values of one musicindex feed for a person, with the state of each
/// value (ADR 0117 items 5 and 6). The musicindex and keysend requests run
/// with no lock of the workspace. The collection is read again under the
/// lock, after the requests. Gives 404 when musicindex does not know the
/// feed.
pub(crate) async fn musicindex_values(
    State(shared): State<Shared>,
    Path((id, guid)): Path<(i64, String)>,
) -> ApiResult {
    let (index, web, scheme) = blocking(shared.clone(), move |w| {
        people::person(&w.collection, PersonId(id)).map_err(person_error)?;
        Ok((w.index.clone(), w.web.clone(), w.keysend_scheme))
    })
    .await?;
    let feed = blocking_unlocked(move || {
        let feed = lookup::fetch_values(&index, &web, scheme, &guid)?;
        feed.ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "not_in_index",
                format!("Music Index has no feed {guid}"),
            )
        })
    })
    .await?;
    blocking(shared, move |w| {
        let values =
            lookup::compose_values(&w.collection, PersonId(id), feed).map_err(person_error)?;
        Ok(Json(json!(values)))
    })
    .await
}

/// Dismiss the Lightning address of a profile for a person (ADR 0098 item
/// 18). The address does not come back as a suggestion for that person.
pub(crate) async fn dismiss_profile_address(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<ProfileAddressBody>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        people::dismiss_profile_address(&w.collection, PersonId(id), &body.address)
            .map_err(person_error)?;
        Ok(StatusCode::NO_CONTENT)
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

/// The members of a publisher feed (ADR 0118 item 9). A feed that is not a
/// publisher feed gives no member.
pub(crate) async fn feed_members(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let members = people::feed_members(&w.collection, FeedId(id))?;
        Ok(Json(json!({ "members": members })))
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
