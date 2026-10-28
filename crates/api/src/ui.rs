//! The UI files, included in the binary (ADR 0016).

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

/// Each UI file: its path under `/ui/`, its media type and its bytes.
const FILES: &[(&str, &str, &[u8])] = &[
    (
        "app.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/app.js"),
    ),
    (
        "draft.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/draft.js"),
    ),
    (
        "editor.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/editor.js"),
    ),
    (
        "announce.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/announce.js"),
    ),
    (
        "playlist.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/playlist.js"),
    ),
    (
        "upload.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/upload.js"),
    ),
    (
        "live.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/live.js"),
    ),
    (
        "account.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/account.js"),
    ),
    (
        "signing.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/signing.js"),
    ),
    (
        "signer.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/signer.js"),
    ),
    (
        "shell.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/shell.js"),
    ),
    (
        "cover.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/cover.js"),
    ),
    (
        "listing.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/listing.js"),
    ),
    (
        "explicit.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/explicit.js"),
    ),
    (
        "credits.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/credits.js"),
    ),
    (
        "artwork.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/artwork.js"),
    ),
    (
        "pages.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/pages.js"),
    ),
    (
        "picker.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/picker.js"),
    ),
    (
        "audio.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/audio.js"),
    ),
    (
        "addfeeds.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/addfeeds.js"),
    ),
    (
        "values.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/values.js"),
    ),
    (
        "theme.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/theme.js"),
    ),
    (
        "styles.css",
        "text/css; charset=utf-8",
        include_bytes!("../ui/styles.css"),
    ),
    (
        "vendor/htm-preact-standalone.module.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../ui/vendor/htm-preact-standalone.module.js"),
    ),
    (
        "assets/figtree-latin.woff2",
        "font/woff2",
        include_bytes!("../ui/assets/figtree-latin.woff2"),
    ),
    (
        "assets/figtree-latin-ext.woff2",
        "font/woff2",
        include_bytes!("../ui/assets/figtree-latin-ext.woff2"),
    ),
    (
        "assets/favicon.svg",
        "image/svg+xml",
        include_bytes!("../ui/assets/favicon.svg"),
    ),
    (
        "assets/musicindex-lockup-horizontal.svg",
        "image/svg+xml",
        include_bytes!("../ui/assets/musicindex-lockup-horizontal.svg"),
    ),
    (
        "assets/icon-192.png",
        "image/png",
        include_bytes!("../ui/assets/icon-192.png"),
    ),
    (
        "assets/icon-512.png",
        "image/png",
        include_bytes!("../ui/assets/icon-512.png"),
    ),
    (
        "assets/icon-maskable-512.png",
        "image/png",
        include_bytes!("../ui/assets/icon-maskable-512.png"),
    ),
    (
        "assets/apple-touch-icon.png",
        "image/png",
        include_bytes!("../ui/assets/apple-touch-icon.png"),
    ),
    // The licenses of the vendored files.
    (
        "assets/Figtree-OFL.txt",
        "text/plain; charset=utf-8",
        include_bytes!("../ui/assets/Figtree-OFL.txt"),
    ),
    (
        "vendor/preact-LICENSE.txt",
        "text/plain; charset=utf-8",
        include_bytes!("../ui/vendor/preact-LICENSE.txt"),
    ),
    (
        "vendor/htm-LICENSE.txt",
        "text/plain; charset=utf-8",
        include_bytes!("../ui/vendor/htm-LICENSE.txt"),
    ),
];

/// The UI loads nothing from other servers (ADR 0016). The policy makes the
/// browser enforce this.
const SECURITY_HEADERS: [(header::HeaderName, &str); 3] = [
    (
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
    ),
    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    (header::REFERRER_POLICY, "no-referrer"),
];

pub(crate) async fn index() -> Response {
    (
        SECURITY_HEADERS,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../ui/index.html"),
    )
        .into_response()
}

/// The web app manifest. The icons in it are UI files. It has no token and
/// no session, because the browser fetches it without credentials.
pub(crate) async fn manifest() -> Response {
    (
        SECURITY_HEADERS,
        [(header::CONTENT_TYPE, "application/manifest+json")],
        include_str!("../ui/manifest.webmanifest"),
    )
        .into_response()
}

pub(crate) async fn file(Path(path): Path<String>) -> Response {
    match FILES.iter().find(|(name, _, _)| *name == path) {
        Some((_, media_type, bytes)) => (
            SECURITY_HEADERS,
            [(header::CONTENT_TYPE, *media_type)],
            *bytes,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// The UI file paths, for tests.
#[cfg(test)]
pub(crate) fn paths() -> impl Iterator<Item = &'static str> {
    FILES.iter().map(|(name, _, _)| *name)
}

/// The text UI files, for tests.
#[cfg(test)]
pub(crate) fn text_files() -> impl Iterator<Item = (&'static str, &'static str)> {
    let index = [
        ("index.html", include_str!("../ui/index.html")),
        (
            "manifest.webmanifest",
            include_str!("../ui/manifest.webmanifest"),
        ),
    ];
    index.into_iter().chain(
        FILES.iter().filter_map(|(name, _, bytes)| {
            std::str::from_utf8(bytes).ok().map(|text| (*name, text))
        }),
    )
}
