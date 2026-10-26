//! API errors as JSON.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use studio_core::address::AddressError;
use studio_core::collection::{CollectionError, CollectionErrorKind};
use studio_core::connector::{ConnectorError, ConnectorErrorKind};
use studio_core::download::{DownloadError, DownloadErrorKind};
use studio_core::folder_scan::ScanError;
use studio_core::inventory::{InventoryError, InventoryErrorKind};
use studio_core::media::{MediaError, MediaErrorKind};
use studio_core::migrate::MigrateError;
use studio_core::pipeline::{PipelineError, PipelineErrorKind};
use studio_core::publisher::{PublisherError, PublisherErrorKind};
use studio_core::vault::{VaultError, VaultErrorKind};

/// An error that the API sends as `{"error": {"code", "message"}}`.
///
/// The message never contains a secret.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    /// The HTTP status of the error, for the tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    /// The code of the error, such as `wrong_key`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The code and the message as one text, for a record in a collection.
    pub(crate) fn coded(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }

    pub(crate) fn vault_locked() -> Self {
        Self::new(StatusCode::LOCKED, "vault_locked", "unlock the vault first")
    }

    pub(crate) fn vault_required() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "vault_required",
            "make the vault first: this step keeps a secret",
        )
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": { "code": self.code, "message": self.message } });
        (self.status, Json(body)).into_response()
    }
}

impl From<CollectionError> for ApiError {
    fn from(error: CollectionError) -> Self {
        let (status, code) = match error.kind() {
            CollectionErrorKind::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            CollectionErrorKind::InUse => (StatusCode::CONFLICT, "in_use"),
            // ADR 0027: a saved recipient needs an address.
            CollectionErrorKind::Invalid => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_recipient"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "collection"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<VaultError> for ApiError {
    fn from(error: VaultError) -> Self {
        let (status, code) = match error.kind() {
            VaultErrorKind::PassphraseTooShort => {
                (StatusCode::UNPROCESSABLE_ENTITY, "passphrase_too_short")
            }
            VaultErrorKind::PassphraseTooWeak => {
                (StatusCode::UNPROCESSABLE_ENTITY, "passphrase_too_weak")
            }
            VaultErrorKind::WrongKey => (StatusCode::UNPROCESSABLE_ENTITY, "wrong_key"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "vault"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<ConnectorError> for ApiError {
    fn from(error: ConnectorError) -> Self {
        let (status, code) = match error.kind() {
            ConnectorErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            ConnectorErrorKind::InvalidPath => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_path"),
            ConnectorErrorKind::InvalidConfig => {
                (StatusCode::UNPROCESSABLE_ENTITY, "invalid_config")
            }
            ConnectorErrorKind::Unauthorized => (StatusCode::BAD_GATEWAY, "host_unauthorized"),
            ConnectorErrorKind::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "host_unavailable")
            }
            // ADR 0057 item 6: the network policy refused the host.
            ConnectorErrorKind::NotPublic => {
                (StatusCode::UNPROCESSABLE_ENTITY, "address_not_public")
            }
            // ADR 0046 item 5: the provider test refused the host.
            ConnectorErrorKind::NoConditions => (StatusCode::UNPROCESSABLE_ENTITY, "no_conditions"),
            // ADR 0050 item 12: the user did not sign a planned event.
            ConnectorErrorKind::SignatureMissing => (StatusCode::CONFLICT, "signature_required"),
            _ => (StatusCode::BAD_GATEWAY, "connector"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<InventoryError> for ApiError {
    fn from(error: InventoryError) -> Self {
        let (status, code) = match error.kind() {
            // ADR 0017: the host needs the vault, and the vault is locked.
            InventoryErrorKind::VaultLocked => (StatusCode::LOCKED, "vault_locked"),
            InventoryErrorKind::UnknownHost | InventoryErrorKind::NoHost => {
                (StatusCode::UNPROCESSABLE_ENTITY, "unknown_host")
            }
            _ => (StatusCode::BAD_GATEWAY, "inventory"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<MigrateError> for ApiError {
    fn from(error: MigrateError) -> Self {
        Self::internal(error.to_string())
    }
}

impl From<PipelineError> for ApiError {
    fn from(error: PipelineError) -> Self {
        let (status, code) = match error.kind() {
            PipelineErrorKind::ReadOnly => (StatusCode::CONFLICT, "read_only"),
            PipelineErrorKind::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            PipelineErrorKind::Network => (StatusCode::BAD_GATEWAY, "network"),
            PipelineErrorKind::NoDraft => (StatusCode::CONFLICT, "no_draft"),
            PipelineErrorKind::DiffNotApproved => (StatusCode::CONFLICT, "diff_not_approved"),
            PipelineErrorKind::StaleRevision => (StatusCode::CONFLICT, "stale_revision"),
            PipelineErrorKind::BadAddress => (StatusCode::UNPROCESSABLE_ENTITY, "bad_address"),
            PipelineErrorKind::BadLiveItem => (StatusCode::UNPROCESSABLE_ENTITY, "bad_live_item"),
            PipelineErrorKind::InvalidModel => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_model"),
            PipelineErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            PipelineErrorKind::InvalidFeed => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_feed"),
            PipelineErrorKind::NoPublicAddress => {
                (StatusCode::UNPROCESSABLE_ENTITY, "no_public_address")
            }
            PipelineErrorKind::FileExists => (StatusCode::CONFLICT, "file_exists"),
            // A feed with no release folder cannot take an import, the
            // same as a workspace with no library folder (ADR 0036
            // item 1 and item 5).
            PipelineErrorKind::NoFolder => (StatusCode::CONFLICT, "no_library"),
            PipelineErrorKind::NotReachable => (StatusCode::BAD_GATEWAY, "not_reachable"),
            PipelineErrorKind::VaultLocked => (StatusCode::LOCKED, "vault_locked"),
            // ADR 0038 item 12: a copy waits for the download of its file.
            PipelineErrorKind::WaitingForDownload => (StatusCode::CONFLICT, "waiting_for_download"),
            // ADR 0050 item 12: the signed events for the planned events.
            PipelineErrorKind::SignatureMissing => (StatusCode::CONFLICT, "signature_required"),
            PipelineErrorKind::SignatureInvalid => {
                (StatusCode::UNPROCESSABLE_ENTITY, "signature_invalid")
            }
            PipelineErrorKind::PlanExpired => (StatusCode::CONFLICT, "plan_expired"),
            // ADR 0054 items 13 to 16: the refusals of a granted write.
            PipelineErrorKind::GrantNotActive => (StatusCode::CONFLICT, "grant_ended"),
            PipelineErrorKind::DraftOpen => (StatusCode::CONFLICT, "draft_open"),
            PipelineErrorKind::BadStatus => (StatusCode::CONFLICT, "bad_status"),
            PipelineErrorKind::NotCovered => (StatusCode::UNPROCESSABLE_ENTITY, "not_covered"),
            _ => (StatusCode::BAD_GATEWAY, "pipeline"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<PublisherError> for ApiError {
    fn from(error: PublisherError) -> Self {
        // A pipeline failure keeps the mapping of its own error (ADR 0041).
        if error.kind() == PublisherErrorKind::Pipeline {
            return match error.pipeline() {
                Some(pipeline) => pipeline.clone().into(),
                None => Self::internal(error.to_string()),
            };
        }
        let (status, code) = match error.kind() {
            PublisherErrorKind::NotPublisher => (StatusCode::UNPROCESSABLE_ENTITY, "not_publisher"),
            PublisherErrorKind::NotRelease => (StatusCode::UNPROCESSABLE_ENTITY, "not_release"),
            PublisherErrorKind::ReadOnly => (StatusCode::CONFLICT, "read_only"),
            PublisherErrorKind::NoRole => (StatusCode::CONFLICT, "role_required"),
            PublisherErrorKind::NoGuid => (StatusCode::UNPROCESSABLE_ENTITY, "no_guid"),
            PublisherErrorKind::NoAddress => {
                (StatusCode::UNPROCESSABLE_ENTITY, "no_public_address")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "publisher"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<MediaError> for ApiError {
    fn from(error: MediaError) -> Self {
        let (status, code) = match error.kind() {
            // ADR 0026, ADR 0032 items 1 to 5: the content is not a media
            // type that Studio accepts.
            MediaErrorKind::Unsupported => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media")
            }
            _ => (StatusCode::BAD_GATEWAY, "media"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<AddressError> for ApiError {
    fn from(error: AddressError) -> Self {
        match error {
            AddressError::Collection(e) => e.into(),
            AddressError::Connector(e) => e.into(),
            _ => Self::internal(error.to_string()),
        }
    }
}

impl From<studio_core::web::WebError> for ApiError {
    fn from(error: studio_core::web::WebError) -> Self {
        let code = match error.kind() {
            studio_core::web::WebErrorKind::InvalidAddress => Some("invalid_address"),
            // ADR 0025, ADR 0032 item 13: the core read a file that is not
            // an image, or an image that is larger than the limit.
            studio_core::web::WebErrorKind::NotImage => Some("not_image"),
            studio_core::web::WebErrorKind::TooLarge => Some("too_large"),
            // ADR 0057 item 6: the network policy refused the address.
            studio_core::web::WebErrorKind::NotPublic => Some("address_not_public"),
            _ => None,
        };
        match code {
            Some(code) => Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, error.to_string()),
            None => Self::new(StatusCode::BAD_GATEWAY, "network", error.to_string()),
        }
    }
}

impl From<DownloadError> for ApiError {
    fn from(error: DownloadError) -> Self {
        let (status, code) = match error.kind() {
            // ADR 0038 item 10: the download breaks a limit setting.
            DownloadErrorKind::TooLarge | DownloadErrorKind::NoSpace => {
                (StatusCode::UNPROCESSABLE_ENTITY, "download_limit")
            }
            DownloadErrorKind::NoSource => (StatusCode::UNPROCESSABLE_ENTITY, "no_source"),
            DownloadErrorKind::Collection => (StatusCode::INTERNAL_SERVER_ERROR, "collection"),
            _ => (StatusCode::BAD_GATEWAY, "download"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<ScanError> for ApiError {
    fn from(error: ScanError) -> Self {
        match error {
            ScanError::Collection(error) => error.into(),
            ScanError::Io(error) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "library_io",
                error.to_string(),
            ),
        }
    }
}

/// The codes and statuses are the same as the codes of the NIP-46 routes
/// of the server (ADR 0056 items 13 to 16).
impl From<studio_core::nip46::Nip46Error> for ApiError {
    fn from(error: studio_core::nip46::Nip46Error) -> Self {
        use studio_core::nip46::Nip46Error;
        match &error {
            Nip46Error::NotPublic(message) => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "address_not_public",
                message.as_str(),
            ),
            Nip46Error::Timeout => Self::new(
                StatusCode::GATEWAY_TIMEOUT,
                "signer_timeout",
                "the signer gave no answer in time",
            ),
            Nip46Error::Unavailable => Self::new(
                StatusCode::BAD_GATEWAY,
                "signer_unavailable",
                "Studio cannot reach the relay of the signer",
            ),
            Nip46Error::InvalidAddress => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "bunker_invalid",
                "the bunker address is not correct",
            ),
            Nip46Error::Rejected(_) => Self::new(
                StatusCode::FORBIDDEN,
                "signer_rejected",
                "the signer refused the request",
            ),
            Nip46Error::Invalid(_) => Self::new(
                StatusCode::BAD_GATEWAY,
                "signer_invalid",
                "the signer gave a wrong answer",
            ),
        }
    }
}

impl From<studio_core::nip46::PlannedError> for ApiError {
    fn from(error: studio_core::nip46::PlannedError) -> Self {
        use studio_core::nip46::PlannedError;
        match error {
            PlannedError::Invalid => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_event",
                "the event is not an unsigned Nostr event",
            ),
            PlannedError::KindNotPermitted => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "kind_not_permitted",
                "the signer signs only the events of a write: kind 35128 and kind 24242",
            ),
            PlannedError::KeyDiffers => Self::new(
                StatusCode::CONFLICT,
                "signer_key_differs",
                "the key of the signer is not the key of the host",
            ),
            PlannedError::Signer(error) => error.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_that_refuses_or_is_away_gives_its_own_code() {
        let unauthorized = ConnectorError::new(ConnectorErrorKind::Unauthorized, "refused");
        let error = ApiError::from(unauthorized);
        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error.code, "host_unauthorized");

        let unavailable = ConnectorError::new(ConnectorErrorKind::Unavailable, "away");
        let error = ApiError::from(unavailable);
        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.code, "host_unavailable");
    }
}
