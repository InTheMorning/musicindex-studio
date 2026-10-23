//! API errors as JSON.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use studio_core::address::AddressError;
use studio_core::collection::{CollectionError, CollectionErrorKind};
use studio_core::connector::{ConnectorError, ConnectorErrorKind};
use studio_core::media::{MediaError, MediaErrorKind};
use studio_core::pipeline::{PipelineError, PipelineErrorKind};
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
    pub(crate) fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
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
            _ => (StatusCode::BAD_GATEWAY, "connector"),
        };
        Self::new(status, code, error.to_string())
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
            PipelineErrorKind::InvalidModel => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_model"),
            PipelineErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            PipelineErrorKind::InvalidFeed => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_feed"),
            PipelineErrorKind::NoPublicAddress => {
                (StatusCode::UNPROCESSABLE_ENTITY, "no_public_address")
            }
            PipelineErrorKind::FileExists => (StatusCode::CONFLICT, "file_exists"),
            _ => (StatusCode::BAD_GATEWAY, "pipeline"),
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
            _ => None,
        };
        match code {
            Some(code) => Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, error.to_string()),
            None => Self::new(StatusCode::BAD_GATEWAY, "network", error.to_string()),
        }
    }
}
