//! JSON API of musicindex-studio (ADR 0002).
//!
//! In local mode the API serves one user on a loopback address, and each
//! `/api/` request needs the access token of ADR 0017. The API also serves
//! the UI files (ADR 0016).

mod access;
mod error;
mod routes;
mod ui;
mod workspace;

use std::sync::{Arc, Mutex};

use axum::middleware;
use axum::routing::{get, post};

pub use access::Access;
pub use axum::Router;
pub use error::ApiError;
pub use workspace::{VaultState, Workspace};

/// Serve a router until the process stops.
///
/// # Errors
///
/// Gives the I/O error of the listener.
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    axum::serve(listener, router).await
}

/// The router of the local API and the UI.
pub fn local_router(workspace: Workspace, access: Access) -> Router {
    let shared: routes::Shared = Arc::new(Mutex::new(workspace));
    let api = Router::new()
        .route("/status", get(routes::status))
        .route("/vault", post(routes::create_vault))
        .route("/vault/unlock", post(routes::unlock_vault))
        .route("/vault/lock", post(routes::lock_vault))
        .route(
            "/connectors",
            get(routes::list_connectors).post(routes::add_connector),
        )
        .route("/feeds", get(routes::list_feeds).post(routes::add_feed))
        .route("/feeds/{id}/refresh", post(routes::refresh_feed))
        .with_state(shared);
    Router::new()
        .route("/", get(ui::index))
        .route("/ui/{*path}", get(ui::file))
        .nest("/api/v1", api)
        .layer(middleware::from_fn_with_state(access, access::check))
}

#[cfg(test)]
mod tests;
