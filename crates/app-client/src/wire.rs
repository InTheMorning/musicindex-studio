//! The wire shapes of the `/app/v1/` routes (ADR 0110). The server tasks
//! align with this module. A change of a shape changes this module in the
//! same commit.
//!
//! | Route | Request body | Response body |
//! | --- | --- | --- |
//! | `POST /app/v1/pair` | [`PairRequest`] | [`WhoAmI`] |
//! | `GET /app/v1/whoami` | none | [`WhoAmI`] |
//! | `GET /app/v1/shows` | none | array of [`Show`] |
//! | `GET /app/v1/shows/{grant_id}/wrapper` | none | [`Wrapper`] |
//! | `POST /app/v1/shows/{grant_id}/status` | [`StatusRequest`] | [`WriteResult`] |
//! | `PUT /app/v1/shows/{grant_id}/live-value` | [`LiveValueRequest`] | [`WriteResult`] |
//! | `DELETE /app/v1/shows/{grant_id}/live-value` | [`GrantKeyRequest`] | [`WriteResult`] |
//!
//! An error answer has the body `{"error":{"code":"...","message":"..."}}`.

use serde::{Deserialize, Serialize};

/// Body of `POST /app/v1/pair`. The code is the normalized code of 8
/// characters. The body is signed (ADR 0110 item 10).
#[derive(Serialize)]
pub struct PairRequest<'a> {
    /// The pairing code.
    pub code: &'a str,
    /// The live value origin of the app (ADR 0110 item 24).
    pub live_value_origin: &'a str,
}

/// Body of `GET /app/v1/whoami` and of the answer to the pairing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct WhoAmI {
    /// The ID of the app in Studio.
    #[serde(default)]
    pub id: i64,
    /// The name that the user gave the app.
    #[serde(default)]
    pub name: String,
    /// The state of the app, for example `active`.
    #[serde(default)]
    pub state: Option<String>,
    /// The live value origin of the app.
    #[serde(default)]
    pub live_value_origin: Option<String>,
    /// The short form of the public key, for the user to compare.
    #[serde(default)]
    pub short_key: Option<String>,
}

/// A live value of a show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveValue {
    /// The address where the listeners get the live value.
    pub uri: String,
    /// The protocol, 1 to 32 letters, digits, `.` or `-`.
    pub protocol: String,
}

/// One entry of `GET /app/v1/shows`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Show {
    /// The ID of the grant. It is in the path of the other routes.
    pub grant_id: i64,
    /// The ID of the feed in Studio.
    #[serde(default)]
    pub feed_id: i64,
    /// The title of the feed.
    #[serde(default)]
    pub feed_title: Option<String>,
    /// The `guid` of the live item.
    #[serde(default)]
    pub item_guid: String,
    /// The title of the live item.
    #[serde(default)]
    pub title: Option<String>,
    /// The status of the live item: `pending`, `live` or `ended`.
    #[serde(default)]
    pub status: Option<String>,
    /// The start time of the show, as the feed has it.
    #[serde(default)]
    pub start: Option<String>,
    /// The end time of the show, as the feed has it.
    #[serde(default)]
    pub end: Option<String>,
    /// The live value now in the feed.
    #[serde(default)]
    pub live_value: Option<LiveValue>,
    /// The live value origin that the grant fixes.
    #[serde(default)]
    pub live_value_origin: Option<String>,
}

/// Body of `GET /app/v1/shows/{grant_id}/wrapper` (ADR 0110 item 23).
#[derive(Debug, Clone, Deserialize)]
pub struct Wrapper {
    /// The public key of the random sender key, as hexadecimal text.
    pub sender: String,
    /// The NIP-44 version 2 ciphertext of the grant vault key.
    pub ciphertext: String,
}

/// Body of `POST /app/v1/shows/{grant_id}/status`.
#[derive(Serialize)]
pub struct StatusRequest<'a> {
    /// `live` or `ended`.
    pub status: &'a str,
    /// The grant vault key (ADR 0110 item 23).
    pub grant_key: &'a str,
}

/// Body of `PUT /app/v1/shows/{grant_id}/live-value`.
#[derive(Serialize)]
pub struct LiveValueRequest<'a> {
    /// The address on the live value origin.
    pub uri: &'a str,
    /// The protocol.
    pub protocol: &'a str,
    /// The grant vault key.
    pub grant_key: &'a str,
}

/// Body of `DELETE /app/v1/shows/{grant_id}/live-value`.
#[derive(Serialize)]
pub struct GrantKeyRequest<'a> {
    /// The grant vault key.
    pub grant_key: &'a str,
}

/// The answer of a write.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct WriteResult {
    /// The version of the feed after the write.
    #[serde(default)]
    pub version: Option<String>,
    /// The status of the live item after the write.
    #[serde(default)]
    pub status: Option<String>,
    /// The live value after the write.
    #[serde(default)]
    pub live_value: Option<LiveValue>,
}

/// The body of an error answer.
#[derive(Deserialize)]
pub(crate) struct ErrorBody {
    pub error: ErrorDetail,
}

/// The error object of an error answer.
#[derive(Deserialize)]
pub(crate) struct ErrorDetail {
    pub code: String,
}
