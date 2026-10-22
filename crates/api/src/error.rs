//! API errors as JSON.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use studio_core::collection::{CollectionError, CollectionErrorKind};
use studio_core::connector::{ConnectorError, ConnectorErrorKind};
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
            PipelineErrorKind::NoDraft => (StatusCode::CONFLICT, "no_draft"),
            PipelineErrorKind::DiffNotApproved => (StatusCode::CONFLICT, "diff_not_approved"),
            PipelineErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            PipelineErrorKind::InvalidFeed => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_feed"),
            _ => (StatusCode::BAD_GATEWAY, "pipeline"),
        };
        Self::new(status, code, error.to_string())
    }
}
