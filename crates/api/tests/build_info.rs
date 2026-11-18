//! musicindex ADR 0083 R-VERSION-9 item 1: the status route gives the build
//! values that the binary sets. This test is a separate process, because the
//! build values are set once in each process.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use studio_api::{Access, BuildInfo, Workspace, local_router, set_build_info};
use tower::ServiceExt;

#[tokio::test]
async fn the_status_route_gives_the_build_values_of_the_binary() {
    set_build_info(BuildInfo::new(
        "Music Index Studio",
        "9.8.7",
        Some("a1b2c3d"),
        Some("2026-11-18T12:00:00Z"),
    ));
    let data = tempfile::tempdir().unwrap();
    let router = local_router(
        Workspace::open(data.path()).unwrap(),
        Access::with_token("token", 4100),
    );
    let request = Request::builder()
        .uri("/api/v1/status")
        .header(header::HOST, "127.0.0.1:4100")
        .header(header::AUTHORIZATION, "Bearer token")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["version"], "9.8.7");
    assert_eq!(body["git_revision"], "a1b2c3d");
    assert_eq!(body["built_at"], "2026-11-18T12:00:00Z");
    assert_eq!(
        body["version_line"],
        "Music Index Studio 9.8.7 (a1b2c3d, 2026-11-18)"
    );
}
