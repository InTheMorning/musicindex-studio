//! JSON API of musicindex-studio (ADR 0002).
//!
//! In local mode the API serves one user on a loopback address, and each
//! `/api/` request needs the access token of ADR 0017. The API also serves
//! the UI files (ADR 0016).

mod access;
mod announcer;
mod downloads;
mod error;
mod media;
mod routes;
mod ui;
mod uploads;
mod workspace;

use std::sync::{Arc, Mutex};

use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{delete, get, post, put};

pub use access::Access;
pub use axum::Router;
pub use error::ApiError;
pub use workspace::{AppChange, AppHook, ServerRules, VaultState, Workspace, WorkspaceHandle};

/// Serve a router until the process stops.
///
/// # Errors
///
/// Gives the I/O error of the listener.
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    axum::serve(listener, router).await
}

/// A QR code of the text, as SVG text (ADR 0056 item 13). Both shells use
/// it, so the UI needs no QR code. The text is empty when the QR code
/// cannot hold the text.
#[must_use]
pub fn qr_svg(text: &str) -> String {
    use qrcode::render::svg;
    // The light color is white also in the dark theme, because a scanner
    // needs the contrast.
    qrcode::QrCode::new(text.as_bytes()).map_or_else(
        |_| String::new(),
        |code| {
            code.render::<svg::Color>()
                .min_dimensions(240, 240)
                .quiet_zone(true)
                .dark_color(svg::Color("#000000"))
                .light_color(svg::Color("#ffffff"))
                .build()
        },
    )
}

/// The UI routes: `/`, `/manifest.webmanifest` and `/ui/{*path}`, with the security headers
/// (ADR 0016).
pub fn ui_router() -> Router {
    Router::new()
        .route("/", get(ui::index))
        .route("/manifest.webmanifest", get(ui::manifest))
        .route("/ui/{*path}", get(ui::file))
}

/// The routes of local mode that need no token: the session route
/// (ADR 0055 item 20).
async fn local_session() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "mode": "local" }))
}

/// The router of the local API and the UI. It has the UI routes, the
/// session route and the routes of one workspace, all under the access
/// layer (ADR 0017).
pub fn local_router(workspace: Workspace, access: Access) -> Router {
    let (workspace_routes, _handle) = workspace_router(workspace);
    ui_router()
        .route("/auth/v1/session", get(local_session))
        .merge(workspace_routes)
        .layer(middleware::from_fn_with_state(access, access::check))
}

/// The routes of one workspace: `/api/v1` and `/app/v1`, with the workers
/// attached. The router has no access layer and no UI routes. The caller
/// adds its own access rules (ADR 0055 item 4).
#[expect(clippy::too_many_lines, reason = "the function is one list of routes")]
pub fn workspace_router(workspace: Workspace) -> (Router, WorkspaceHandle) {
    let queue = Arc::clone(&workspace.downloads);
    let announcer = Arc::clone(&workspace.announcer);
    let shared: routes::Shared = Arc::new(Mutex::new(workspace));
    // The workers lock the workspace for short steps (ADR 0038 item 7).
    queue.attach(&shared);
    // The announcement worker also locks it for short steps (ADR 0052).
    announcer.attach(&shared);
    // The clean step removes the uploads that expired (ADR 0059 item 4).
    uploads::spawn_cleaner(&shared);
    // The sync removes the grant vaults of grants that ended (ADR 0058).
    workspace::spawn_grant_sync(&shared);
    let api = Router::new()
        .route("/status", get(routes::status))
        .route("/vault", post(routes::create_vault))
        .route("/vault/strength", post(routes::passphrase_strength))
        .route("/vault/unlock", post(routes::unlock_vault))
        .route("/vault/lock", post(routes::lock_vault))
        .route("/vault/passphrase", post(routes::set_passphrase))
        .route("/vault/unlock-key", post(routes::unlock_vault_with_key))
        .route(
            "/vault/nostr-wrapper",
            get(routes::get_nostr_wrapper).post(routes::add_nostr_wrapper),
        )
        .route("/vault/signer-key", post(routes::signer_key))
        .route(
            "/signer",
            get(routes::get_signer).delete(routes::delete_signer),
        )
        .route("/signer/connect", post(routes::connect_signer))
        .route("/signer/sign", post(routes::sign_with_signer))
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
        .route("/connectors/{id}/picker", get(routes::connector_picker))
        .route("/connectors/{id}/site", get(routes::connector_site))
        .route("/connectors/{id}/match", post(routes::match_connector))
        .route("/discover", post(routes::discover))
        .route("/images", get(routes::get_image))
        .route("/media/facts", post(routes::media_facts))
        .route("/media/tickets", post(media::make_ticket))
        .route("/search", get(routes::search))
        .route("/recipients", get(routes::list_recipients))
        .route("/recipients/uses", post(routes::recipient_uses))
        .route(
            "/change-sets",
            get(routes::list_change_sets).post(routes::make_change_set),
        )
        .route(
            "/change-sets/{id}",
            get(routes::get_change_set).delete(routes::discard_change_set),
        )
        .route("/feeds/{id}/level", get(routes::feed_level))
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
            get(routes::list_release_files).delete(routes::delete_pending_file),
        )
        .route(
            "/feeds/{id}/uploads",
            get(uploads::list_uploads).post(uploads::start_upload),
        )
        .route(
            "/uploads/{upload}",
            get(uploads::get_upload).delete(uploads::delete_upload),
        )
        .route(
            "/uploads/{upload}/parts/{n}",
            put(uploads::put_part).layer(DefaultBodyLimit::max(
                usize::try_from(uploads::PART_SIZE).unwrap_or(usize::MAX),
            )),
        )
        .route("/uploads/{upload}/finish", post(uploads::finish_upload))
        .route(
            "/feeds/{id}/downloads",
            get(routes::list_downloads).post(routes::start_downloads),
        )
        .route(
            "/feeds/{id}/downloads/{file}",
            delete(routes::stop_download),
        )
        .route("/feeds/{id}/picker/release", get(routes::picker_release))
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
        .route("/feeds/{id}/listing", post(routes::set_listing))
        .route(
            "/feeds/{id}/listing/{platform}",
            post(routes::set_platform_listing),
        )
        .route("/feeds/{id}/grants", get(routes::list_feed_grants))
        .route(
            "/feeds/{id}/grants/{guid}",
            put(routes::put_draft_grant).delete(routes::delete_draft_grant),
        )
        .route(
            "/feeds/{id}/grants/{guid}/end",
            post(routes::end_feed_grant),
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
        .route("/nostr/profiles", get(routes::nostr_profiles))
        .route(
            "/settings/nostr",
            get(routes::get_nostr_settings).put(routes::put_nostr_settings),
        )
        .route("/feeds/{id}/announcements", get(routes::list_announcements))
        .route("/hosts/{id}/scan", post(routes::scan_host))
        .route("/apps", get(routes::list_apps).post(routes::add_app))
        .route("/apps/{id}", delete(routes::remove_app))
        .with_state(Arc::clone(&shared));
    // The routes for connected apps (ADR 0054 items 8 and 15). The outer layer
    // checks the loopback `Host` only. The app layer checks the app token.
    let app_api = Router::new()
        .route("/whoami", get(routes::whoami))
        .route("/shows", get(routes::list_shows))
        .route("/shows/{grant_id}/status", post(routes::set_show_status))
        .route(
            "/shows/{grant_id}/live-value",
            put(routes::put_show_live_value).delete(routes::delete_show_live_value),
        )
        .with_state(Arc::clone(&shared))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&shared),
            routes::app_check,
        ));
    // The audio route is outside `/api/v1`. The player cannot send the
    // access token, so a media ticket in the query is the access
    // (ADR 0062 item 29).
    let audio = Router::new()
        .route("/media/audio", get(media::audio))
        .with_state(Arc::clone(&shared));
    let router = Router::new()
        .nest("/api/v1", api)
        .nest("/app/v1", app_api)
        .merge(audio);
    (router, WorkspaceHandle::new(shared))
}

#[cfg(test)]
mod tests;
