//! The musicindex API (Stophammer): feed addresses by GUID, search, and the
//! payment routes of a feed and of a track (ADR 0027 item 6).

use serde::{Deserialize, Serialize};

use crate::feed::Recipient;
use crate::web::{Web, WebError, WebErrorKind, path_segment};
use musicindex_search::field_list::{FieldListRow, Library, field_list_rows};
use musicindex_search::note::{SearchNote as CrateNote, scope_note, search_note};
pub use musicindex_search::plan::IndexEntity;
pub use musicindex_search::plan::IndexSearchRequest;
use musicindex_search::plan::{IndexSearchPlan, IndexSkip};
use musicindex_search::query::{QueryFieldName, SearchQuery, TermKind, quote_field_value};
use musicindex_search::request::index_filter_pairs;

/// The public musicindex API.
pub const DEFAULT_API: &str = "https://api.musicindex.org";

/// The largest musicindex response that Studio reads, in bytes.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

/// The number of tracks of a feed that [`MusicIndex::feed_credits`] reads. Each
/// track is one request.
const MAX_CREDIT_TRACKS: usize = 25;

/// A feed that a musicindex search found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHit {
    /// `podcast:guid` of the feed.
    pub feed_guid: String,
    /// The feed title.
    pub title: Option<String>,
    /// The release artist that musicindex gives.
    pub artist: Option<String>,
    /// The address of the feed image.
    pub image_url: Option<String>,
}

/// A track that a feed lists, from musicindex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexTrack {
    /// The GUID of the track.
    pub track_guid: String,
    /// The title of the track.
    pub title: String,
}

/// The payment routes of a feed or of a track, from musicindex
/// (ADR 0027 item 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexRoutes {
    /// The title of the feed or of the track.
    pub title: String,
    /// Each payment route, as a recipient.
    pub recipients: Vec<Recipient>,
    /// Each track of the feed. Empty for the routes of one track.
    pub tracks: Vec<IndexTrack>,
}

/// A publisher feed that a musicindex search found, with its link facts
/// (ADR 0117 item 1). The lists are empty when musicindex gives none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherHit {
    /// `podcast:guid` of the feed.
    pub feed_guid: String,
    /// The feed title.
    pub title: Option<String>,
    /// The address of the feed image.
    pub image_url: Option<String>,
    /// The release artist that musicindex gives.
    pub release_artist: Option<String>,
    /// The source of the release artist, such as `placeholder`.
    pub release_artist_source: Option<String>,
    /// The number of two-way links of the feed. Missing when musicindex
    /// gives no link facts.
    pub two_way_link_count: Option<i64>,
    /// The roles that the confirmed links agree on.
    pub agreed_roles: Vec<String>,
    /// The release artists of the confirmed links.
    pub confirmed_release_artists: Vec<String>,
}

/// The values of a publisher feed in musicindex, for the fields of a person
/// (ADR 0117 item 2). The addresses are not checked here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublisherValues {
    /// The feed title.
    pub title: Option<String>,
    /// The feed address.
    pub feed_url: Option<String>,
    /// The address of the feed image.
    pub image_url: Option<String>,
    /// The address of the first `website` link.
    pub website: Option<String>,
    /// The `nostr_npub` value of the feed, as the first source ID.
    pub npub: Option<String>,
    /// Each payment route of the feed, as a recipient.
    pub routes: Vec<Recipient>,
}

/// One credit of a feed or a track in musicindex (ADR 0146 item 8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexCredit {
    /// The name.
    pub name: String,
    /// The role as the source gives it.
    pub role: Option<String>,
    /// The group of the role.
    pub group: Option<String>,
    /// The picture address.
    pub img: Option<String>,
    /// The information address.
    pub href: Option<String>,
    /// The Nostr key.
    pub npub: Option<String>,
}

/// The Nostr keys that musicindex records for a feed (ADR 0074 item 9).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FeedKeys {
    /// The release artist that musicindex gives.
    pub artist: Option<String>,
    /// Each `nostr_npub` entity ID of the feed.
    pub feed_keys: Vec<String>,
    /// Each contributor with a key, as a name and a key.
    pub contributors: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct KeysData {
    release_artist: Option<String>,
    source_ids: Option<Vec<EntityIdData>>,
    source_contributors: Option<Vec<ContributorData>>,
}

#[derive(Deserialize)]
struct EntityIdData {
    scheme: String,
    value: String,
}

#[derive(Deserialize)]
struct ContributorData {
    name: String,
    npub: Option<String>,
}

/// One row of `GET /v1/artist-names`.
#[derive(Deserialize)]
struct ArtistNameRow {
    name: String,
}

/// The most pages of copies that one read follows.
pub const COPIES_PAGE_CAP: usize = 20;

/// One page of the copies of a feed.
#[derive(Deserialize)]
struct CopiesPage {
    data: Vec<IndexCopy>,
    #[serde(default)]
    pagination: Option<Pagination>,
}

/// The cursor of a page.
#[derive(Deserialize)]
struct Pagination {
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    has_more: bool,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

/// The rows of a search answer. An answer that cannot decode is an error, not
/// an empty list (ADR 0151 item 7).
fn decode_search<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, WebError> {
    serde_json::from_slice::<Envelope<T>>(bytes)
        .map(|envelope| envelope.data)
        .map_err(|e| {
            WebError::decode(format!(
                "musicindex gave an answer that Studio cannot read: {e}"
            ))
        })
}

/// A note in the place of the rows of a search (musicindex ADR 0049 R-QUERY-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchNote {
    /// The cause: `library_only`, `conflict`, `exclusions_only`, `not_supported`,
    /// `no_index_part`, `needs_a_word` or `not_for_publishers`.
    pub code: &'static str,
    /// The text that the UI shows.
    pub message: String,
}

/// What one search text does (ADR 0151).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchPlan {
    /// Send this request.
    Send(IndexSearchRequest),
    /// Send nothing, and give no rows. A note can give the cause.
    Skip(Option<SearchNote>),
    /// Send nothing. The text has a field that is not correct, and this is the
    /// message.
    FieldError(String),
}

/// The plan of a search text. The crate reads the text, and this function
/// holds no rule of the text (musicindex ADR 0049 R-QUERY-4). With `plain`, the
/// text is a start text, and it never gives a field or an exclusion (ADR 0151
/// item 3).
#[must_use]
pub fn plan_search(text: &str, plain: bool) -> SearchPlan {
    plan_for(text, plain, None)
}

/// The plan of a search text for a search of artist and label feeds only.
/// When the crate gives no request for these feeds, the search sends nothing,
/// and the note of the crate names each field that causes this (ADR 0151 item
/// 9).
#[must_use]
pub fn plan_publisher_search(text: &str, plain: bool) -> SearchPlan {
    plan_for(text, plain, Some(IndexEntity::Publisher))
}

/// The plan of `text`. The words of each note come from the crate (musicindex
/// ADR 0052 R-QUERY-8). Studio has no Library in the sense of musicindex ADR
/// 0049. A `scope` gives the note of the crate when that entity gets no
/// request, after the general note.
fn plan_for(text: &str, plain: bool, scope: Option<IndexEntity>) -> SearchPlan {
    // A text with nothing to search has no note.
    if text.trim().is_empty() {
        return SearchPlan::Skip(None);
    }
    let query = SearchQuery::parse(text);
    if plain {
        let only_words = query
            .terms()
            .iter()
            .all(|term| !term.excluded && matches!(term.kind, TermKind::Word(_)));
        if !only_words {
            return SearchPlan::Send(IndexSearchRequest::words(text.trim()));
        }
    }
    // The first field error in text order (musicindex ADR 0052 R-QUERY-8).
    if let Some((_, error)) = query.field_errors().first() {
        return SearchPlan::FieldError(error.message());
    }
    let index_plan = IndexSearchPlan::for_query(&query);
    if let Some(note) = search_note(&query, &index_plan, Library::Absent) {
        return SearchPlan::Skip(Some(SearchNote::from_crate(note_code(&index_plan), &note)));
    }
    if let Some(entity) = scope
        && let Some(note) = scope_note(&query, &index_plan, entity)
    {
        return SearchPlan::Skip(Some(SearchNote::from_crate("not_for_publishers", &note)));
    }
    match index_plan {
        IndexSearchPlan::Send(request) => SearchPlan::Send(request),
        IndexSearchPlan::Skip(_) => SearchPlan::Skip(None),
    }
}

/// One row of the field list for the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "row", rename_all = "snake_case")]
pub enum FieldRow {
    /// A field of the query text.
    Field {
        /// The field name with its colon, for example `kind:`.
        token: &'static str,
        /// The names of the fixed values. The list is empty when the field
        /// has none.
        values: Vec<&'static str>,
        /// A short text that names the correct values.
        values_text: &'static str,
        /// One example of the field, with its token. For a text field it is
        /// the token and the quoted placeholder, which runs no search
        /// (musicindex ADR 0058 R-QUERY-11).
        example: &'static str,
        /// The placeholder text of a text field, which the UI selects after
        /// the example goes into the field. `None` when the example runs.
        placeholder: Option<&'static str>,
        /// What the field matches.
        description: &'static str,
        /// The text that replaces the edit range: the token.
        insert: String,
    },
    /// One fixed value of a field.
    Value {
        /// The token of the field, for example `kind:`.
        field: &'static str,
        /// The value word.
        value: &'static str,
        /// What the value means.
        description: &'static str,
        /// The text that replaces the edit range: the value, with quotes when
        /// it needs them.
        insert: String,
    },
}

/// A range of the query text, in UTF-16 code units of the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Utf16Range {
    /// The first code unit of the range.
    pub start: usize,
    /// The code unit after the range.
    pub end: usize,
}

/// The field and the typed prefix of the names that the UI can ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NameQuery {
    /// The token of the field: `artist:`, `release:` or `playlist:`.
    pub field: &'static str,
    /// The typed text of the value. The crate has read its quotes.
    pub prefix: String,
}

/// The field list for one query text and one cursor (musicindex ADR 0052
/// R-QUERY-7). The core gives each range, and the UI has no parser (ADR 0151).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldList {
    /// The rows, in the order of the crate.
    pub rows: Vec<FieldRow>,
    /// The range that a row replaces, or `None` when the cursor is in a phrase
    /// or in a word that is not a field.
    pub edit: Option<Utf16Range>,
    /// The name rows that the UI can ask for after `rows`.
    pub names: Option<NameQuery>,
    /// The message of the field under the cursor, when its value is not correct.
    pub error: Option<String>,
}

/// The text that puts `name` into the value of a name field: the name, with
/// quotes when it needs them. The crate gives the quotes (musicindex ADR 0049).
#[must_use]
pub fn name_insert(name: &str) -> String {
    quote_field_value(name)
}

/// The byte offset of the UTF-16 offset `units` in `text`. An offset after
/// the end gives the end. An offset inside a surrogate pair gives the start of
/// that character.
fn utf16_to_byte(text: &str, units: usize) -> usize {
    let mut seen = 0;
    for (byte, ch) in text.char_indices() {
        let next = seen + ch.len_utf16();
        if next > units {
            return byte;
        }
        seen = next;
    }
    text.len()
}

/// The UTF-16 offset of the byte offset `byte` in `text`. The offset is on a
/// character boundary.
fn byte_to_utf16(text: &str, byte: usize) -> usize {
    text[..byte].chars().map(char::len_utf16).sum()
}

/// `true` when a search with a word and the example of `field` sends a request
/// for artist and label feeds. The crate decides this.
fn field_applies_to_publishers(field: QueryFieldName) -> bool {
    let text = format!("able {}", field.info().example);
    match IndexSearchPlan::for_query(&SearchQuery::parse(&text)) {
        IndexSearchPlan::Send(request) => request.for_entity(IndexEntity::Publisher).is_some(),
        IndexSearchPlan::Skip(_) => false,
    }
}

/// The rows of the field list for `text` and the cursor `cursor_utf16`, in
/// UTF-16 code units. Studio has no Library, so no row is for a Library field.
/// With the scope `Publisher`, each field that gives no request for artist and
/// label feeds has no row, and its value rows and its names are gone too.
#[must_use]
pub fn field_list(text: &str, cursor_utf16: usize, scope: Option<IndexEntity>) -> FieldList {
    let answer = field_list_rows(text, utf16_to_byte(text, cursor_utf16), Library::Absent);
    let keep = |field: QueryFieldName| {
        scope != Some(IndexEntity::Publisher) || field_applies_to_publishers(field)
    };
    let rows = answer
        .rows
        .iter()
        .filter_map(|row| match *row {
            FieldListRow::Field(info) => keep(info.name).then(|| FieldRow::Field {
                token: info.token,
                values: info.name.fixed_values().iter().map(|v| v.word).collect(),
                values_text: info.values_text,
                example: info.example,
                placeholder: info.placeholder,
                description: info.description,
                insert: info.token.to_owned(),
            }),
            FieldListRow::Value { field, value } => keep(field).then(|| FieldRow::Value {
                field: field.info().token,
                value: value.word,
                description: value.description,
                insert: quote_field_value(value.word),
            }),
        })
        .collect();
    FieldList {
        rows,
        edit: answer.edit.map(|range| Utf16Range {
            start: byte_to_utf16(text, range.start),
            end: byte_to_utf16(text, range.end),
        }),
        names: answer
            .names
            .filter(|names| keep(names.field))
            .map(|names| NameQuery {
                field: names.field.info().token,
                prefix: names.prefix,
            }),
        error: answer.error,
    }
}

/// The code of the note of `plan`, for the API. The UI shows only the words.
fn note_code(plan: &IndexSearchPlan) -> &'static str {
    match plan {
        IndexSearchPlan::Send(_) => "needs_a_word",
        IndexSearchPlan::Skip(IndexSkip::LibraryOnly) => "library_only",
        IndexSearchPlan::Skip(IndexSkip::Conflict) => "conflict",
        IndexSearchPlan::Skip(IndexSkip::Unsupported(_)) => "not_supported",
        IndexSearchPlan::Skip(IndexSkip::ExclusionsOnly) => "exclusions_only",
        IndexSearchPlan::Skip(IndexSkip::NoIndexPart) => "no_index_part",
    }
}

impl SearchNote {
    /// The note of the crate, as one text: the title, then the text.
    fn from_crate(code: &'static str, note: &CrateNote) -> Self {
        Self {
            code,
            message: format!("{}. {}", note.title, note.text),
        }
    }
}

#[derive(Deserialize)]
struct FeedData {
    feed_url: Option<String>,
    title: Option<String>,
    last_build_date: Option<i64>,
}

/// The body of a 404 for a feed GUID that a GUID change replaced.
#[derive(Debug, serde::Deserialize)]
struct SupersededBody {
    superseded_by: Option<String>,
}

/// What musicindex answers for the record of a feed GUID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexPresence {
    /// The record of the feed.
    Held(IndexFeed),
    /// A plain 404. musicindex holds no record of the GUID.
    Absent,
    /// A 404 with `superseded_by`. A GUID change replaced the GUID. The
    /// value is the new GUID.
    Superseded(String),
}

/// The address, the title and the build time that musicindex knows for a
/// feed GUID.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexFeed {
    /// The feed address.
    pub url: Option<String>,
    /// The title of the feed.
    pub title: Option<String>,
    /// The `lastBuildDate` that musicindex read, in Unix seconds. It is
    /// none when musicindex read no date.
    pub last_build_date: Option<i64>,
}

/// One copy of a feed: a different URL that gives the same feed GUID
/// (Stophammer ADR 0058).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IndexCopy {
    /// The URL of the copy.
    pub url: String,
    /// True when no one has resolved the copy.
    #[serde(default)]
    pub open: bool,
    /// True when the copy has other payment recipients than the feed.
    #[serde(default)]
    pub differs_recipients: bool,
}

#[derive(Deserialize)]
struct SearchItem {
    entity_type: String,
    entity_id: String,
    title: Option<String>,
    release_artist: Option<String>,
    feed_image_url: Option<String>,
}

/// One `payment_routes` entry of a feed or a track response.
#[derive(Deserialize)]
struct RouteData {
    recipient_name: Option<String>,
    route_type: String,
    address: String,
    split: i64,
    custom_key: Option<String>,
    custom_value: Option<String>,
    fee: bool,
}

impl From<RouteData> for Recipient {
    fn from(route: RouteData) -> Self {
        Self {
            name: route.recipient_name,
            recipient_type: Some(route.route_type),
            address: Some(route.address),
            split: Some(route.split.to_string()),
            custom_key: route.custom_key,
            custom_value: route.custom_value,
            fee: route.fee.then(|| "true".to_owned()),
        }
    }
}

/// One `GET /v1/feeds/{guid}` response, with only the fields that
/// [`MusicIndex::feed_routes`] uses.
#[derive(Deserialize)]
struct FeedRoutesData {
    title: String,
    payment_routes: Option<Vec<RouteData>>,
    tracks: Option<Vec<TrackSummaryData>>,
}

#[derive(Deserialize)]
struct TrackSummaryData {
    track_guid: String,
    title: String,
}

/// One `source_contributors` entry of a feed or a track response.
#[derive(Deserialize)]
struct CreditData {
    name: String,
    role: Option<String>,
    group_name: Option<String>,
    img: Option<String>,
    href: Option<String>,
    npub: Option<String>,
}

/// The credits of a feed or a track response, and the tracks of a feed.
#[derive(Deserialize)]
struct CreditsData {
    source_contributors: Option<Vec<CreditData>>,
    tracks: Option<Vec<TrackSummaryData>>,
}

/// One `GET /v1/feeds/{guid}/tracks/{track_guid}` response, with only the
/// fields that [`MusicIndex::track_routes`] uses.
#[derive(Deserialize)]
struct TrackRoutesData {
    title: String,
    payment_routes: Option<Vec<RouteData>>,
}

/// One `GET /v1/search?type=publisher` row, with the link facts.
#[derive(Deserialize)]
struct PublisherRowData {
    entity_type: String,
    entity_id: String,
    title: Option<String>,
    release_artist: Option<String>,
    release_artist_source: Option<String>,
    feed_image_url: Option<String>,
    two_way_link_count: Option<i64>,
    agreed_roles: Option<Vec<String>>,
    confirmed_release_artists: Option<Vec<String>>,
}

/// One `source_links` entry of a feed response.
#[derive(Deserialize)]
struct LinkData {
    link_type: String,
    url: Option<String>,
}

/// One `GET /v1/feeds/{guid}` response, with only the fields that
/// [`MusicIndex::publisher_values`] uses.
#[derive(Deserialize)]
struct PublisherFeedData {
    title: Option<String>,
    feed_url: Option<String>,
    image_url: Option<String>,
    source_links: Option<Vec<LinkData>>,
    source_ids: Option<Vec<EntityIdData>>,
    payment_routes: Option<Vec<RouteData>>,
}

impl From<PublisherFeedData> for PublisherValues {
    fn from(data: PublisherFeedData) -> Self {
        Self {
            title: data.title,
            feed_url: data.feed_url,
            image_url: data.image_url,
            website: data
                .source_links
                .unwrap_or_default()
                .into_iter()
                .find(|link| link.link_type == "website")
                .and_then(|link| link.url),
            npub: data
                .source_ids
                .unwrap_or_default()
                .into_iter()
                .find(|id| id.scheme == "nostr_npub")
                .map(|id| id.value),
            routes: data
                .payment_routes
                .unwrap_or_default()
                .into_iter()
                .map(Recipient::from)
                .collect(),
        }
    }
}

impl From<FeedRoutesData> for IndexRoutes {
    fn from(data: FeedRoutesData) -> Self {
        Self {
            title: data.title,
            recipients: data
                .payment_routes
                .unwrap_or_default()
                .into_iter()
                .map(Recipient::from)
                .collect(),
            tracks: data
                .tracks
                .unwrap_or_default()
                .into_iter()
                .map(|t| IndexTrack {
                    track_guid: t.track_guid,
                    title: t.title,
                })
                .collect(),
        }
    }
}

impl From<TrackRoutesData> for IndexRoutes {
    fn from(data: TrackRoutesData) -> Self {
        Self {
            title: data.title,
            recipients: data
                .payment_routes
                .unwrap_or_default()
                .into_iter()
                .map(Recipient::from)
                .collect(),
            tracks: Vec::new(),
        }
    }
}

/// Keep only the hex digits and the hyphens of a GUID, so that the request
/// path holds no other text.
fn filter_guid(guid: &str) -> String {
    guid.chars()
        .filter(|c| c.is_ascii_hexdigit() || *c == '-')
        .collect()
}

/// A client of one musicindex API.
#[derive(Debug, Clone)]
pub struct MusicIndex {
    web: Web,
    base: String,
}

impl MusicIndex {
    /// Make a client for the API at `base`, such as [`DEFAULT_API`].
    #[must_use]
    pub fn new(web: Web, base: &str) -> Self {
        Self {
            web,
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    /// The base address of the API. The check of the time of a server asks
    /// it (ADR 0127 item 7).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base
    }

    /// The address of the request for the record of a feed GUID.
    #[must_use]
    pub fn feed_address(&self, feed_guid: &str) -> String {
        format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid))
    }

    /// The address of the request for the copies of a feed GUID.
    #[must_use]
    pub fn copies_address(&self, feed_guid: &str) -> String {
        format!("{}/copies", self.feed_address(feed_guid))
    }

    /// The feed address that musicindex knows for a feed GUID. Gives `None`
    /// when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_url(&self, feed_guid: &str) -> Result<Option<String>, WebError> {
        Ok(self.feed(feed_guid)?.and_then(|feed| feed.url))
    }

    /// The address and the title that musicindex knows for a feed GUID.
    /// Gives `None` when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed(&self, feed_guid: &str) -> Result<Option<IndexFeed>, WebError> {
        Ok(match self.feed_presence(feed_guid)? {
            IndexPresence::Held(feed) => Some(feed),
            IndexPresence::Absent | IndexPresence::Superseded(_) => None,
        })
    }

    /// The record of a feed GUID, or the reason that musicindex has none. A
    /// 404 with `superseded_by` in its body names the GUID that replaced the
    /// GUID of the request. That is not an absent feed (ADR 0131 item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_presence(&self, feed_guid: &str) -> Result<IndexPresence, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get(&url, MAX_RESPONSE_BYTES) {
            Ok(file) => Ok(serde_json::from_slice::<Envelope<FeedData>>(&file.bytes)
                .ok()
                .map_or(IndexPresence::Absent, |e| {
                    IndexPresence::Held(IndexFeed {
                        url: e.data.feed_url,
                        title: e.data.title,
                        last_build_date: e.data.last_build_date,
                    })
                })),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => {
                Ok(serde_json::from_slice::<SupersededBody>(e.body())
                    .ok()
                    .and_then(|body| body.superseded_by)
                    .filter(|guid| !guid.is_empty())
                    .map_or(IndexPresence::Absent, IndexPresence::Superseded))
            }
            Err(e) => Err(e),
        }
    }

    /// The copies of a feed GUID (ADR 0175). The function follows the cursor
    /// of each page to the last page, or to [`COPIES_PAGE_CAP`] pages.
    /// Gives `None` when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when a request fails.
    pub fn feed_copies(&self, feed_guid: &str) -> Result<Option<Vec<IndexCopy>>, WebError> {
        let url = format!("{}/v1/feeds/{}/copies", self.base, filter_guid(feed_guid));
        let mut copies = Vec::new();
        let mut cursor: Option<String> = None;
        for page in 0..COPIES_PAGE_CAP {
            let query: Vec<(&str, &str)> = cursor
                .as_deref()
                .map(|c| ("cursor", c))
                .into_iter()
                .collect();
            let file = match self.web.get_with_query(&url, &query, MAX_RESPONSE_BYTES) {
                Ok(file) => file,
                Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => {
                    return Ok((page > 0).then_some(copies));
                }
                Err(e) => return Err(e),
            };
            let Ok(body) = serde_json::from_slice::<CopiesPage>(&file.bytes) else {
                return Ok((page > 0).then_some(copies));
            };
            copies.extend(body.data);
            cursor = body
                .pagination
                .filter(|p| p.has_more)
                .and_then(|p| p.cursor)
                .filter(|c| !c.is_empty());
            if cursor.is_none() {
                break;
            }
        }
        Ok(Some(copies))
    }

    /// The title, the payment routes and the tracks of a feed, from the
    /// `payment_routes` and `tracks` of the musicindex feed (ADR 0027
    /// item 6). Gives `None` when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_routes(&self, feed_guid: &str) -> Result<Option<IndexRoutes>, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get_with_query(
            &url,
            &[("include", "payment_routes,tracks"), ("limit", "200")],
            MAX_RESPONSE_BYTES,
        ) {
            Ok(file) => Ok(
                serde_json::from_slice::<Envelope<FeedRoutesData>>(&file.bytes)
                    .ok()
                    .map(|e| e.data.into()),
            ),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The title and the payment routes of one track, from the
    /// `payment_routes` of the musicindex track (ADR 0027 item 6). The
    /// result has no tracks. Gives `None` when musicindex does not know the
    /// track.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn track_routes(
        &self,
        feed_guid: &str,
        track_guid: &str,
    ) -> Result<Option<IndexRoutes>, WebError> {
        let url = format!(
            "{}/v1/feeds/{}/tracks/{}",
            self.base,
            filter_guid(feed_guid),
            path_segment(track_guid)
        );
        match self
            .web
            .get_with_query(&url, &[("include", "payment_routes")], MAX_RESPONSE_BYTES)
        {
            Ok(file) => Ok(
                serde_json::from_slice::<Envelope<TrackRoutesData>>(&file.bytes)
                    .ok()
                    .map(|e| e.data.into()),
            ),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The Nostr keys of a feed, from `source_ids` and
    /// `source_contributors` (ADR 0074 item 9). Gives `None` when
    /// musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_keys(&self, feed_guid: &str) -> Result<Option<FeedKeys>, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get_with_query(
            &url,
            &[("include", "source_ids,source_contributors")],
            MAX_RESPONSE_BYTES,
        ) {
            Ok(file) => Ok(serde_json::from_slice::<Envelope<KeysData>>(&file.bytes)
                .ok()
                .map(|e| FeedKeys {
                    artist: e.data.release_artist,
                    feed_keys: e
                        .data
                        .source_ids
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| id.scheme == "nostr_npub")
                        .map(|id| id.value)
                        .collect(),
                    contributors: e
                        .data
                        .source_contributors
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|c| c.npub.map(|npub| (c.name, npub)))
                        .collect(),
                })),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The credits of a feed and of its first tracks, each name and role one
    /// time (ADR 0146 item 8). A feed that musicindex does not know has none.
    /// A track that fails gives no credit.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request of the feed fails.
    pub fn feed_credits(&self, feed_guid: &str) -> Result<Vec<IndexCredit>, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        let file = match self.web.get_with_query(
            &url,
            &[("include", "source_contributors,tracks"), ("limit", "200")],
            MAX_RESPONSE_BYTES,
        ) {
            Ok(file) => file,
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(e),
        };
        let Ok(feed) = serde_json::from_slice::<Envelope<CreditsData>>(&file.bytes) else {
            return Ok(Vec::new());
        };
        let mut found = feed.data.source_contributors.unwrap_or_default();
        for track in feed
            .data
            .tracks
            .unwrap_or_default()
            .iter()
            .take(MAX_CREDIT_TRACKS)
        {
            let url = format!("{url}/tracks/{}", path_segment(&track.track_guid));
            let credits = self
                .web
                .get_with_query(
                    &url,
                    &[("include", "source_contributors")],
                    MAX_RESPONSE_BYTES,
                )
                .ok()
                .and_then(|file| serde_json::from_slice::<Envelope<CreditsData>>(&file.bytes).ok())
                .and_then(|e| e.data.source_contributors);
            found.extend(credits.unwrap_or_default());
        }
        let mut seen = std::collections::HashSet::new();
        Ok(found
            .into_iter()
            .filter(|c| seen.insert((c.name.to_lowercase(), c.role.clone())))
            .map(|c| IndexCredit {
                name: c.name,
                role: c.role,
                group: c.group_name,
                img: c.img,
                href: c.href,
                npub: c.npub,
            })
            .collect())
    }

    /// Send one `GET /v1/search`. The crate gives `q` and each filter, and no
    /// sort goes with them, so the rows keep the sequence of relevance
    /// (ADR 0151 item 8).
    fn send_search(
        &self,
        request: &IndexSearchRequest,
        fixed: &[(&str, &str)],
    ) -> Result<Vec<u8>, WebError> {
        let url = format!("{}/v1/search", self.base);
        let filters = index_filter_pairs(&request.filters);
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(q) = &request.q {
            query.push(("q", q));
        }
        query.extend_from_slice(fixed);
        query.extend(filters.iter().map(|(name, value)| (*name, value.as_str())));
        Ok(self
            .web
            .get_with_query(&url, &query, MAX_RESPONSE_BYTES)?
            .bytes)
    }

    /// Search musicindex for feeds, with the request of a query text. A
    /// request that a feed cannot take gives no rows and sends nothing
    /// (ADR 0151).
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails or its answer cannot decode.
    pub fn search(&self, request: &IndexSearchRequest) -> Result<Vec<SearchHit>, WebError> {
        let Some(request) = request.for_entity(IndexEntity::Feed) else {
            return Ok(Vec::new());
        };
        let bytes = self.send_search(&request, &[("type", "feed"), ("limit", "25")])?;
        let items: Vec<SearchItem> = decode_search(&bytes)?;
        Ok(items
            .into_iter()
            .filter(|i| i.entity_type == "feed")
            .map(|i| SearchHit {
                feed_guid: i.entity_id,
                title: i.title,
                artist: i.release_artist,
                image_url: i.feed_image_url,
            })
            .collect())
    }

    /// Search musicindex for feeds by a text that is only words, with no field
    /// (ADR 0151 item 3).
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails or its answer cannot decode.
    pub fn search_words(&self, text: &str) -> Result<Vec<SearchHit>, WebError> {
        match plan_search(text, true) {
            SearchPlan::Send(request) => self.search(&request),
            SearchPlan::Skip(_) | SearchPlan::FieldError(_) => Ok(Vec::new()),
        }
    }

    /// Search musicindex for publisher feeds, with their link facts (ADR 0117
    /// item 1). The search sends at most 25 rows. A request that a publisher
    /// row cannot take gives no rows and sends nothing (ADR 0151).
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails or its answer cannot decode.
    pub fn search_publishers(
        &self,
        request: &IndexSearchRequest,
    ) -> Result<Vec<PublisherHit>, WebError> {
        let Some(request) = request.for_entity(IndexEntity::Publisher) else {
            return Ok(Vec::new());
        };
        let bytes = self.send_search(
            &request,
            &[
                ("type", "publisher"),
                ("include", "link_facts"),
                ("limit", "25"),
            ],
        )?;
        let rows: Vec<PublisherRowData> = decode_search(&bytes)?;
        Ok(rows
            .into_iter()
            .filter(|row| row.entity_type == "publisher")
            .map(|row| PublisherHit {
                feed_guid: row.entity_id,
                title: row.title,
                image_url: row.feed_image_url,
                release_artist: row.release_artist,
                release_artist_source: row.release_artist_source,
                two_way_link_count: row.two_way_link_count,
                agreed_roles: row.agreed_roles.unwrap_or_default(),
                confirmed_release_artists: row.confirmed_release_artists.unwrap_or_default(),
            })
            .collect())
    }

    /// The artist names that start with `prefix`, for the name rows of the
    /// field list. The route starts at the first sort key at or after the
    /// prefix and does not filter, so this keeps only the names that start with
    /// the prefix, with no case difference. It gives at most 8 different names,
    /// in the order of the answer. An empty prefix gives no name and sends
    /// nothing.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails or its answer cannot decode.
    pub fn artist_names(&self, prefix: &str) -> Result<Vec<String>, WebError> {
        const MOST: usize = 8;
        if prefix.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!("{}/v1/artist-names", self.base);
        let bytes = self
            .web
            .get_with_query(
                &url,
                &[("from", prefix), ("limit", "20")],
                MAX_RESPONSE_BYTES,
            )?
            .bytes;
        let rows: Vec<ArtistNameRow> = decode_search(&bytes)?;
        let wanted = prefix.to_lowercase();
        let mut names: Vec<String> = Vec::new();
        for row in rows {
            if row.name.to_lowercase().starts_with(&wanted) && !names.contains(&row.name) {
                names.push(row.name);
                if names.len() == MOST {
                    break;
                }
            }
        }
        Ok(names)
    }

    /// The values of a publisher feed for a person: its picture, its
    /// website, its Nostr key and its payment routes (ADR 0117 item 2). Gives
    /// `None` when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn publisher_values(&self, feed_guid: &str) -> Result<Option<PublisherValues>, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get_with_query(
            &url,
            &[("include", "source_links,source_ids,payment_routes")],
            MAX_RESPONSE_BYTES,
        ) {
            Ok(file) => Ok(
                serde_json::from_slice::<Envelope<PublisherFeedData>>(&file.bytes)
                    .ok()
                    .map(|e| e.data.into()),
            ),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestServer;

    #[test]
    fn a_guid_gives_its_feed_address_and_an_unknown_guid_gives_none() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
                200,
                br#"{"data":{"feed_url":"https://example.com/a.xml","title":"A"}}"#.to_vec(),
            ),
            (
                "/v1/search",
                200,
                br#"{"data":[{"entity_type":"feed","entity_id":"g1","title":"A","release_artist":"Ann"},{"entity_type":"track","entity_id":"t1"}]}"#.to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        assert_eq!(
            index
                .feed_url("917393e3-1b1e-5cef-ace4-edaa54e1f810")
                .unwrap()
                .as_deref(),
            Some("https://example.com/a.xml")
        );
        assert_eq!(
            index
                .feed_url("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
        let hits = index.search(&IndexSearchRequest::words("ann")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].feed_guid, "g1");
        assert_eq!(hits[0].artist.as_deref(), Some("Ann"));
    }

    #[test]
    fn a_feed_gives_its_build_time_and_its_copies_and_a_404_gives_none() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
                200,
                br#"{"data":{"feed_url":"https://example.com/a.xml","title":"A","last_build_date":1710288000}}"#.to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f811",
                200,
                br#"{"data":{"feed_url":"https://example.com/b.xml","title":"B","last_build_date":null}}"#.to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/copies",
                200,
                br#"{"data":[
                    {"url":"https://mirror.example/a.xml","first_seen":1,"open":true,"differs_recipients":true,"resolution":null},
                    {"url":"https://mirror.example/b.xml","open":false,"differs_recipients":false}
                ],"pagination":{"cursor":null,"has_more":false},"meta":{},"copies_over_limit":0}"#.to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let feed = index
            .feed("917393e3-1b1e-5cef-ace4-edaa54e1f810")
            .unwrap()
            .unwrap();
        assert_eq!(feed.last_build_date, Some(1_710_288_000));
        let none = index
            .feed("917393e3-1b1e-5cef-ace4-edaa54e1f811")
            .unwrap()
            .unwrap();
        assert_eq!(none.last_build_date, None);

        let copies = index
            .feed_copies("917393e3-1b1e-5cef-ace4-edaa54e1f810")
            .unwrap()
            .unwrap();
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0].url, "https://mirror.example/a.xml");
        assert!(copies[0].open && copies[0].differs_recipients);
        assert!(!copies[1].open && !copies[1].differs_recipients);
        assert_eq!(
            index
                .feed_copies("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
    }

    #[test]
    fn the_copies_follow_the_cursor_to_the_last_page_and_stop_at_the_cap() {
        const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/copies",
                200,
                br#"{"data":[{"url":"https://m.example/1.xml","open":true,"differs_recipients":false}],
                    "pagination":{"cursor":"next page","has_more":true}}"#.to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/copies?cursor=next%20page",
                200,
                br#"{"data":[{"url":"https://m.example/2.xml","open":true,"differs_recipients":true}],
                    "pagination":{"cursor":null,"has_more":false}}"#.to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f811/copies",
                200,
                br#"{"data":[{"url":"https://m.example/loop.xml"}],
                    "pagination":{"cursor":"same","has_more":true}}"#.to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f811/copies?cursor=same",
                200,
                br#"{"data":[{"url":"https://m.example/loop.xml"}],
                    "pagination":{"cursor":"same","has_more":true}}"#.to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let copies = index.feed_copies(GUID).unwrap().unwrap();
        let urls: Vec<&str> = copies.iter().map(|c| c.url.as_str()).collect();
        assert_eq!(urls, ["https://m.example/1.xml", "https://m.example/2.xml"]);
        assert!(copies[1].differs_recipients);

        // A cursor that never ends stops at the page cap.
        let looped = index
            .feed_copies("917393e3-1b1e-5cef-ace4-edaa54e1f811")
            .unwrap()
            .unwrap();
        assert_eq!(looped.len(), COPIES_PAGE_CAP);
    }

    #[test]
    fn feed_routes_and_track_routes_give_the_recipients_and_the_tracks_and_a_404_gives_none() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
                200,
                br#"{"data":{"title":"Album","payment_routes":[
                    {"recipient_name":"Ann","route_type":"node","address":"02aa","split":95,"fee":false,"custom_key":null,"custom_value":null},
                    {"recipient_name":null,"route_type":"node","address":"02bb","split":5,"fee":true,"custom_key":"7629169","custom_value":"acct"}
                ],"tracks":[{"track_guid":"aa11","title":"Track One"}]}}"#
                    .to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/tracks/Track%201%2FA",
                200,
                br#"{"data":{"title":"Track One","payment_routes":[
                    {"recipient_name":"Ann","route_type":"node","address":"02aa","split":95,"fee":false,"custom_key":null,"custom_value":null}
                ]}}"#
                    .to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));

        let feed = index
            .feed_routes("917393e3-1b1e-5cef-ace4-edaa54e1f810")
            .unwrap()
            .unwrap();
        assert_eq!(feed.title, "Album");
        assert_eq!(feed.recipients.len(), 2);
        assert_eq!(feed.recipients[0].name.as_deref(), Some("Ann"));
        assert_eq!(feed.recipients[0].split.as_deref(), Some("95"));
        assert_eq!(feed.recipients[0].fee, None);
        assert_eq!(feed.recipients[1].fee.as_deref(), Some("true"));
        assert_eq!(feed.recipients[1].custom_key.as_deref(), Some("7629169"));
        assert_eq!(feed.tracks.len(), 1);
        assert_eq!(feed.tracks[0].track_guid, "aa11");
        assert_eq!(feed.tracks[0].title, "Track One");

        let track = index
            .track_routes("917393e3-1b1e-5cef-ace4-edaa54e1f810", "Track 1/A")
            .unwrap()
            .unwrap();
        assert_eq!(track.title, "Track One");
        assert_eq!(track.recipients.len(), 1);
        assert_eq!(track.tracks, Vec::new());

        assert_eq!(
            index
                .feed_routes("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
        assert_eq!(
            index
                .track_routes("00000000-0000-0000-0000-000000000000", "aa11")
                .unwrap(),
            None
        );
    }

    #[test]
    fn a_publisher_search_reads_the_link_facts_and_skips_other_rows() {
        let server = TestServer::start(vec![(
            "/v1/search",
            200,
            br#"{"data":[
                {"entity_type":"publisher","entity_id":"c0ffee-01","title":"Ann","release_artist":"Ann","release_artist_source":"itunes_author","feed_image_url":"https://img.example/a.png","two_way_link_count":3,"agreed_roles":["artist"],"confirmed_release_artists":["Ann"]},
                {"entity_type":"feed","entity_id":"f1","title":"Other"},
                {"entity_type":"publisher","entity_id":"c0ffee-02","title":"Bo"}
            ]}"#
                .to_vec(),
        )]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let hits = index
            .search_publishers(&IndexSearchRequest::words("ann"))
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].feed_guid, "c0ffee-01");
        assert_eq!(hits[0].two_way_link_count, Some(3));
        assert_eq!(hits[0].agreed_roles, ["artist"]);
        assert_eq!(hits[0].confirmed_release_artists, ["Ann"]);
        assert_eq!(
            hits[0].release_artist_source.as_deref(),
            Some("itunes_author")
        );
        assert_eq!(hits[1].two_way_link_count, None);
        assert_eq!(hits[1].agreed_roles, Vec::<String>::new());
    }

    #[test]
    fn feed_credits_join_the_feed_and_its_tracks_with_each_name_and_role_one_time() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/c0ffee-01",
                200,
                br#"{"data":{"source_contributors":[{"name":"Ann","role":"vocalist","group_name":"Music","href":"https://a.example"}],
                    "tracks":[{"track_guid":"t1","title":"One"},{"track_guid":"t2","title":"Two"}]}}"#
                    .to_vec(),
            ),
            (
                "/v1/feeds/c0ffee-01/tracks/t1",
                200,
                br#"{"data":{"source_contributors":[{"name":"ann","role":"vocalist"},{"name":"Bo","role":"bassist","npub":"npub1bo"}]}}"#
                    .to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let credits = index.feed_credits("c0ffee-01").unwrap();
        let names: Vec<_> = credits.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Ann", "Bo"], "ADR 0146 item 8");
        assert_eq!(credits[0].group.as_deref(), Some("Music"));
        assert_eq!(credits[1].npub.as_deref(), Some("npub1bo"));
        assert_eq!(index.feed_credits("c0ffee-09").unwrap().len(), 0);
    }

    #[test]
    fn publisher_values_give_the_website_the_npub_and_the_routes() {
        let server = TestServer::start(vec![(
            "/v1/feeds/c0ffee-01",
            200,
            br#"{"data":{"title":"Ann","feed_url":"https://feed.example/ann.xml","image_url":"https://img.example/a.png",
                "source_links":[{"link_type":"other","url":"https://x.example"},{"link_type":"website","url":"https://ann.example"}],
                "source_ids":[{"scheme":"other","value":"x"},{"scheme":"nostr_npub","value":"npub1ann"}],
                "payment_routes":[{"recipient_name":"Ann","route_type":"node","address":"02aa","split":100,"fee":false}]}}"#
                .to_vec(),
        )]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let values = index.publisher_values("c0ffee-01").unwrap().unwrap();
        assert_eq!(values.title.as_deref(), Some("Ann"));
        assert_eq!(
            values.feed_url.as_deref(),
            Some("https://feed.example/ann.xml")
        );
        assert_eq!(
            values.image_url.as_deref(),
            Some("https://img.example/a.png")
        );
        assert_eq!(values.website.as_deref(), Some("https://ann.example"));
        assert_eq!(values.npub.as_deref(), Some("npub1ann"));
        assert_eq!(values.routes.len(), 1);
        assert_eq!(values.routes[0].address.as_deref(), Some("02aa"));
        assert_eq!(
            index
                .publisher_values("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
    }

    fn sent_q(text: &str, plain: bool) -> Option<String> {
        match plan_search(text, plain) {
            SearchPlan::Send(request) => request.q,
            other => panic!("{text} must send a request, got {other:?}"),
        }
    }

    #[test]
    fn adr_0151_a_start_text_never_gives_a_field() {
        assert_eq!(sent_q("In:Flux", true).as_deref(), Some("In:Flux"));
        assert_eq!(sent_q("kind:nope", true).as_deref(), Some("kind:nope"));
        assert_eq!(sent_q("Ann Smith", true).as_deref(), Some("Ann Smith"));
        assert_eq!(sent_q("Ann -Smith", true).as_deref(), Some("Ann -Smith"));
        assert_eq!(plan_search("", true), SearchPlan::Skip(None));
    }

    #[test]
    fn adr_0151_a_typed_text_reads_its_fields() {
        assert!(matches!(
            plan_search("kind:nope", false),
            SearchPlan::FieldError(message) if message.starts_with("kind:")
        ));
        assert!(matches!(
            plan_search("in:flux", false),
            SearchPlan::FieldError(_)
        ));
        assert_eq!(sent_q("\"in:flux\"", false).as_deref(), Some("\"in:flux\""));
        // The `kind:` values of a text such as `kind:ep` send no request, and
        // the crate gives the note (musicindex ADR 0052 R-QUERY-8).
        assert_eq!(
            plan_search("kind:ep", false),
            SearchPlan::Skip(Some(SearchNote {
                code: "no_index_part",
                message: "No results. Music Index has no rows for these fields.".to_owned(),
            }))
        );
    }

    #[test]
    fn adr_0151_a_field_error_gives_only_the_first_message() {
        let SearchPlan::FieldError(message) = plan_search("kind:nope after:soon", false) else {
            panic!("a field error");
        };
        assert!(message.starts_with("kind:"), "{message}");
        assert!(!message.contains("after:"), "{message}");
    }

    #[test]
    fn adr_0151_each_skip_with_a_cause_gives_its_note() {
        let code = |text: &str| match plan_search(text, false) {
            SearchPlan::Skip(Some(note)) => note.code,
            other => panic!("{text} must give a note, got {other:?}"),
        };
        assert_eq!(code("ann is:ready"), "library_only");
        assert_eq!(code("in:index is:ready"), "conflict");
        assert_eq!(code("-remix"), "exclusions_only");
        assert_eq!(code("ann -kind:single"), "not_supported");
        assert_eq!(code("in:index"), "no_index_part");
        let message = |text: &str| match plan_search(text, false) {
            SearchPlan::Skip(Some(note)) => note.message,
            other => panic!("{text} must give a note, got {other:?}"),
        };
        assert_eq!(
            message("ann is:ready"),
            "This search uses a Library field. Remove is:, playlist:, has: or in:library to search Music Index."
        );
        assert_eq!(
            message("ann -kind:single"),
            "Music Index cannot exclude kind: yet. Remove the - from the field."
        );
    }

    #[test]
    fn adr_0151_a_publisher_search_names_each_field_with_no_request() {
        let note = |text: &str| match plan_publisher_search(text, false) {
            SearchPlan::Skip(Some(note)) => note.message,
            other => panic!("{text} must give a note, got {other:?}"),
        };
        for (text, token) in [
            ("kind:single", "kind:"),
            ("able after:2026-01", "after:"),
            ("able before:2026-01", "before:"),
            ("able duration:>5m", "duration:"),
            ("able release:x", "release:"),
        ] {
            assert_eq!(
                note(text),
                format!(
                    "{token} does not apply to artists and labels. Remove the field to search artists and labels."
                ),
                "{text}"
            );
        }
        assert_eq!(
            note("able kind:single after:2026-01 release:x after:2026-02"),
            "kind:, after: and release: do not apply to artists and labels. Remove the fields to search artists and labels."
        );
        // The general note of the crate comes first (musicindex ADR 0052
        // R-QUERY-8).
        assert_eq!(
            note("able kind:ep"),
            "No results. Music Index has no rows for these fields."
        );
    }

    #[test]
    fn adr_0151_a_publisher_search_keeps_each_other_plan() {
        assert!(matches!(
            plan_publisher_search("able artist:ann -release:x", false),
            SearchPlan::Send(_)
        ));
        assert!(matches!(
            plan_publisher_search("kind:single", true),
            SearchPlan::Send(_)
        ));
        assert!(matches!(
            plan_publisher_search("kind:nope", false),
            SearchPlan::FieldError(_)
        ));
        assert!(matches!(
            plan_publisher_search("in:index", false),
            SearchPlan::Skip(Some(note)) if note.code == "no_index_part"
        ));
        assert!(matches!(
            plan_publisher_search(r"artist:\", false),
            SearchPlan::Skip(Some(note)) if note.code == "no_index_part"
        ));
        assert!(matches!(
            plan_publisher_search("able is:ready", false),
            SearchPlan::Skip(Some(note)) if note.code == "library_only"
        ));
    }

    #[test]
    fn adr_0151_a_search_answer_that_cannot_decode_is_an_error() {
        let server = crate::testing::TestServer::start(Vec::new());
        server.set("/v1/search", 200, b"not json".to_vec());
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let words = IndexSearchRequest::words("ann");
        assert_eq!(
            index.search(&words).unwrap_err().kind(),
            WebErrorKind::Decode
        );
        assert_eq!(
            index.search_publishers(&words).unwrap_err().kind(),
            WebErrorKind::Decode
        );
    }

    fn tokens(list: &FieldList) -> Vec<&'static str> {
        list.rows
            .iter()
            .filter_map(|row| match row {
                FieldRow::Field { token, .. } => Some(*token),
                FieldRow::Value { .. } => None,
            })
            .collect()
    }

    fn value_words(list: &FieldList) -> Vec<&'static str> {
        list.rows
            .iter()
            .filter_map(|row| match row {
                FieldRow::Value { value, .. } => Some(*value),
                FieldRow::Field { .. } => None,
            })
            .collect()
    }

    #[test]
    fn a_name_with_a_space_gets_quotes() {
        assert_eq!(name_insert("Bjork"), "Bjork");
        assert_eq!(name_insert("The Ben"), "\"The Ben\"");
    }

    #[test]
    fn a_field_start_gives_the_field_row_and_its_edit_range() {
        let list = field_list("ki", 2, None);
        assert_eq!(tokens(&list), vec!["kind:"]);
        assert_eq!(list.edit, Some(Utf16Range { start: 0, end: 2 }));
        let FieldRow::Field { values, insert, .. } = &list.rows[0] else {
            panic!("a field row");
        };
        assert!(values.contains(&"single"));
        assert_eq!(insert, "kind:");
    }

    #[test]
    fn a_text_field_row_gives_its_placeholder_and_inserts_only_its_token() {
        let list = field_list("art", 3, None);
        assert_eq!(tokens(&list), vec!["artist:"]);
        let FieldRow::Field {
            example,
            placeholder,
            insert,
            ..
        } = &list.rows[0]
        else {
            panic!("a field row");
        };
        assert_eq!(
            (*example, *placeholder, insert.as_str()),
            ("artist:\"artist name\"", Some("artist name"), "artist:"),
            "musicindex ADR 0058 R-QUERY-11: a text field row must give its placeholder, \
             and the row must put only the token into the field."
        );
        let FieldRow::Field { placeholder, .. } = &field_list("ki", 2, None).rows[0] else {
            panic!("a field row");
        };
        assert_eq!(*placeholder, None, "kind: has an example that runs.");
    }

    #[test]
    fn a_value_start_gives_the_value_rows() {
        let list = field_list("kind:s", 6, None);
        assert_eq!(value_words(&list), vec!["single"]);
        assert_eq!(list.edit, Some(Utf16Range { start: 5, end: 6 }));
        assert_eq!(list.error, None);
    }

    #[test]
    fn the_edit_range_is_in_utf16_code_units() {
        let text = "Bj\u{f6}rk \u{1f3b5} ki";
        let end = text.encode_utf16().count();
        assert!(end < text.len());
        let list = field_list(text, end, None);
        assert_eq!(tokens(&list), vec!["kind:"]);
        assert_eq!(
            list.edit,
            Some(Utf16Range {
                start: end - 2,
                end
            })
        );
    }

    #[test]
    fn a_cursor_inside_a_surrogate_pair_moves_back_and_a_far_cursor_clamps() {
        // The note is 2 code units and starts at offset 2.
        let text = "a \u{1f3b5}";
        let inside = field_list(text, 3, None);
        let before = field_list(text, 2, None);
        assert_eq!(inside, before);
        let far = field_list("ki", 99, None);
        assert_eq!(tokens(&far), vec!["kind:"]);
        assert_eq!(far.edit, Some(Utf16Range { start: 0, end: 2 }));
    }

    #[test]
    fn a_library_field_gives_no_row() {
        for text in ["is", "in", "has", "playlist"] {
            let list = field_list(text, text.len(), None);
            assert!(
                !tokens(&list)
                    .iter()
                    .any(|t| ["is:", "in:", "has:", "playlist:"].contains(t)),
                "{text}"
            );
        }
        assert_eq!(field_list("is:", 3, None).rows.len(), 0);
    }

    #[test]
    fn a_wrong_value_under_the_cursor_gives_its_error() {
        let list = field_list("kind:nope", 7, None);
        assert!(list.error.as_deref().unwrap().starts_with("kind:"));
        assert_eq!(list.rows.len(), 0);
        assert_eq!(field_list("kind:single", 3, None).error, None);
    }

    #[test]
    fn the_publisher_scope_leaves_out_the_fields_that_give_no_request() {
        let publisher = Some(IndexEntity::Publisher);
        let all = tokens(&field_list("", 0, None));
        let some = tokens(&field_list("", 0, publisher));
        for token in ["kind:", "after:", "release:"] {
            assert!(all.contains(&token), "{token} shows with no scope");
            assert!(!some.contains(&token), "{token} has no publisher row");
        }
        assert!(some.contains(&"artist:"));
        assert_eq!(field_list("kind:", 5, publisher).rows.len(), 0);
        assert!(field_list("kind:", 5, None).rows.len() > 1);
        assert_eq!(field_list("release:ab", 10, publisher).names, None);
    }

    #[test]
    fn an_artist_field_gives_the_names_request() {
        let names = field_list("artist:bj", 9, None).names.unwrap();
        assert_eq!(names.field, "artist:");
        assert_eq!(names.prefix, "bj");
        let quoted = field_list("artist:\"bj \"", 12, None).names.unwrap();
        assert_eq!(quoted.prefix, "bj ");
        assert_eq!(
            field_list("artist:bj", 9, Some(IndexEntity::Publisher))
                .names
                .unwrap()
                .prefix,
            "bj"
        );
        assert_eq!(field_list("kind:s", 6, None).names, None);
    }

    #[test]
    fn artist_names_keep_the_prefix_matches_once_and_at_most_eight() {
        let rows: Vec<String> = ["Bj\u{f6}rk", "BJ\u{d6}RK", "Bjork", "Bjork", "Bjorn", "Bjy"]
            .into_iter()
            .chain(["B1", "B2", "B3", "B4", "B5", "B6"])
            .map(|name| format!(r#"{{"name":"{name}","sort_name":"x"}}"#))
            .collect();
        let body = format!(r#"{{"data":[{}]}}"#, rows.join(","));
        let server = TestServer::start(vec![("/v1/artist-names", 200, body.into_bytes())]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        assert_eq!(
            index.artist_names("bj").unwrap(),
            vec!["Bj\u{f6}rk", "BJ\u{d6}RK", "Bjork", "Bjorn", "Bjy"]
        );
        let many: Vec<String> = (0..12)
            .map(|n| format!(r#"{{"name":"Ab{n}","sort_name":"x"}}"#))
            .collect();
        server.set(
            "/v1/artist-names",
            200,
            format!(r#"{{"data":[{}]}}"#, many.join(",")).into_bytes(),
        );
        assert_eq!(index.artist_names("AB").unwrap().len(), 8);
    }

    #[test]
    fn an_empty_prefix_sends_no_name_request() {
        let index = MusicIndex::new(Web::new(), "http://127.0.0.1:9");
        assert_eq!(index.artist_names("").unwrap(), Vec::<String>::new());
    }
}
