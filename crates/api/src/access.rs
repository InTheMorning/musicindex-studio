//! Access to the local API (ADR 0017).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::error::ApiError;

/// The access rules of one local API.
#[derive(Debug, Clone)]
pub struct Access {
    token: Arc<str>,
    port: u16,
}

impl Access {
    /// Make access rules with a new random token.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the random source is not available.
    pub fn new(port: u16) -> Result<Self, ApiError> {
        let token = random_hex::<32>()?;
        Ok(Self::with_token(&token, port))
    }

    /// Make access rules with a known token.
    #[must_use]
    pub fn with_token(token: &str, port: u16) -> Self {
        Self {
            token: Arc::from(token),
            port,
        }
    }

    /// The token. Give it to the UI in the URL fragment only.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// The UI address with the token in the fragment.
    #[must_use]
    pub fn ui_url(&self) -> String {
        format!("http://127.0.0.1:{}/#token={}", self.port, self.token)
    }

    fn host_is_loopback(&self, host: &str) -> bool {
        ["127.0.0.1", "localhost", "[::1]"]
            .iter()
            .any(|name| host == format!("{name}:{}", self.port))
    }

    fn token_matches(&self, given: &str) -> bool {
        let want = self.token.as_bytes();
        let given = given.as_bytes();
        // Compare in constant time for tokens of the same length.
        want.len() == given.len()
            && want
                .iter()
                .zip(given)
                .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
                == 0
    }
}

/// Refuse a request with a host that is not loopback, and an API request
/// without the token.
pub(crate) async fn check(
    State(access): State<Access>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    if !access.host_is_loopback(host) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "host_not_allowed",
            "ADR 0017: the local API answers only a loopback host name",
        ));
    }
    if request.uri().path().starts_with("/api/") {
        let given = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or_default();
        if !access.token_matches(given) {
            return Err(ApiError::new(
                StatusCode::UNAUTHORIZED,
                "token_required",
                "ADR 0017: open Studio from the address that it gave at start",
            ));
        }
    }
    Ok(next.run(request).await)
}

/// `N` random bytes as lowercase hexadecimal text.
///
/// # Errors
///
/// Gives an [`ApiError`] when the random source is not available.
pub(crate) fn random_hex<const N: usize>() -> Result<String, ApiError> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|_| ApiError::internal("the random source is not available"))?;
    Ok(bytes
        .iter()
        .fold(String::with_capacity(N * 2), |mut text, b| {
            use std::fmt::Write as _;
            let _ = write!(text, "{b:02x}");
            text
        }))
}
