//! JSON API of musicindex-studio (ADR 0002).
//!
//! In local mode the API serves one user on a loopback address, and each
//! `/api/` request needs the access token of ADR 0017. The API also serves
//! the UI files (ADR 0016).

mod access;
mod announcer;
mod downloads;
mod error;
mod routes;
mod ui;
mod workspace;

use std::sync::{Arc, Mutex};

use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{delete, get, post, put};

pub use access::Access;
pub use axum::Router;
pub use error::ApiError;
pub use workspace::{VaultState, Workspace};

/// The body limit of the upload route (ADR 0026, ADR 0032). The other
/// routes keep the default limit of 2 MiB.
const MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024 * 1024;

/// Serve a router until the process stops.
///
/// # Errors
///
/// Gives the I/O error of the listener.
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    axum::serve(listener, router).await
}

/// The router of the local API and the UI.
#[expect(clippy::too_many_lines, reason = "the function is one list of routes")]
pub fn local_router(workspace: Workspace, access: Access) -> Router {
    let queue = Arc::clone(&workspace.downloads);
    let announcer = Arc::clone(&workspace.announcer);
    let shared: routes::Shared = Arc::new(Mutex::new(workspace));
    // The workers lock the workspace for short steps (ADR 0038 item 7).
    queue.attach(&shared);
    // The announcement worker also locks it for short steps (ADR 0052).
    announcer.attach(&shared);
    let api = Router::new()
        .route("/status", get(routes::status))
        .route("/vault", post(routes::create_vault))
        .route("/vault/strength", post(routes::passphrase_strength))
        .route("/vault/unlock", post(routes::unlock_vault))
        .route("/vault/lock", post(routes::lock_vault))
        .route(
            "/connectors",
            get(routes::list_connectors).post(routes::add_connector),
        )
        .route(
            "/connectors/fingerprint",
            post(routes::connector_fingerprint),
        )
        .route("/connectors/nostr-lists", post(routes::nostr_lists))
        .route(
            "/connectors/{id}",
            put(routes::update_connector).delete(routes::delete_connector),
        )
        .route("/connectors/{id}/feeds", get(routes::connector_feeds))
        .route("/connectors/{id}/site", get(routes::connector_site))
        .route("/connectors/{id}/match", post(routes::match_connector))
        .route("/discover", post(routes::discover))
        .route("/images", get(routes::get_image))
        .route("/media/facts", post(routes::media_facts))
        .route("/search", get(routes::search))
        .route("/recipients", get(routes::list_recipients))
        .route("/recipients/saved", post(routes::save_recipient))
        .route(
            "/recipients/saved/{id}",
            delete(routes::remove_saved_recipient),
        )
        .route(
            "/musicindex/feeds/{guid}/recipients",
            get(routes::index_feed_recipients),
        )
        .route("/playlist/tracks", get(routes::playlist_tracks))
        .route(
            "/musicindex/feeds/{guid}/tracks",
            get(routes::index_feed_tracks),
        )
        .route(
            "/musicindex/feeds/{guid}/tracks/{track}/recipients",
            get(routes::index_track_recipients),
        )
        .route("/feeds/import", post(routes::import_feeds))
        .route("/import/msp", post(routes::import_msp))
        .route("/feeds", get(routes::list_feeds).post(routes::add_feed))
        .route("/feeds/new", post(routes::new_album_feed))
        .route("/feeds/{id}", delete(routes::delete_feed))
        .route("/feeds/{id}/links", post(routes::add_link))
        .route("/feeds/{id}/links/{release}", delete(routes::remove_link))
        .route("/feeds/{id}/role", put(routes::set_publisher_role))
        .route("/feeds/{id}/refresh", post(routes::refresh_feed))
        .route(
            "/feeds/{id}/files",
            get(routes::list_release_files)
                .post(routes::upload_file)
                .delete(routes::delete_pending_file)
                .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .route(
            "/feeds/{id}/downloads",
            get(routes::list_downloads).post(routes::start_downloads),
        )
        .route(
            "/feeds/{id}/downloads/{file}",
            delete(routes::stop_download),
        )
        .route("/feeds/{id}/hosts", put(routes::set_files_host))
        .route("/feeds/{id}/feed-host", put(routes::set_feed_host))
        .route("/feeds/{id}/files/{file}/host", put(routes::set_file_host))
        .route("/feeds/{id}/operations", post(routes::add_operation))
        .route(
            "/feeds/{id}/operations/{op}",
            delete(routes::remove_operation),
        )
        .route(
            "/feeds/{id}/draft",
            get(routes::get_draft)
                .put(routes::put_draft)
                .delete(routes::delete_draft),
        )
        .route("/feeds/{id}/live-items", post(routes::add_live_item))
        .route(
            "/feeds/{id}/live-items/{guid}/status",
            post(routes::set_live_status),
        )
        .route("/feeds/{id}/diff", get(routes::get_diff))
        .route("/feeds/{id}/write", post(routes::write_feed))
        .route("/feeds/{id}/snapshots", get(routes::list_snapshots))
        .route(
            "/feeds/{id}/snapshots/{snapshot}/draft",
            post(routes::draft_from_snapshot),
        )
        .route(
            "/library",
            get(routes::get_library).put(routes::put_library),
        )
        .route("/library/limits", put(routes::put_library_limits))
        .route("/library/scan", post(routes::scan_library))
        .route("/feeds/{id}/library-changes", post(routes::library_changes))
        .route(
            "/settings/announcements",
            get(routes::get_announce_settings).put(routes::put_announce_settings),
        )
        .route("/feeds/{id}/announcements", get(routes::list_announcements))
        .route("/hosts/{id}/scan", post(routes::scan_host))
        .route("/apps", get(routes::list_apps).post(routes::add_app))
        .route("/apps/{id}", delete(routes::remove_app))
        .with_state(Arc::clone(&shared));
    // The routes for connected apps (ADR 0054 item 8). The outer layer
    // checks the loopback `Host` only. The app layer checks the app token.
    let app_api = Router::new()
        .route("/whoami", get(routes::whoami))
        .layer(middleware::from_fn_with_state(shared, routes::app_check));
    Router::new()
        .route("/", get(ui::index))
        .route("/ui/{*path}", get(ui::file))
        .nest("/api/v1", api)
        .nest("/app/v1", app_api)
        .layer(middleware::from_fn_with_state(access, access::check))
}

#[cfg(test)]
mod tests;
