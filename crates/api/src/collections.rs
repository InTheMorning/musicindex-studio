//! The routes of the collections of the desktop app (ADR 0149).
//!
//! Only the router of the desktop app has these routes. A server, and a shell
//! with `--data-dir`, have none.

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::collections::{Collections, CollectionsError, CollectionsErrorKind, MAIN_FOLDER};

use crate::routes::blocking_unlocked;
use crate::{ApiError, Work};

/// What the app does when the user selects "Open" (ADR 0149 item 14).
pub type OnOpen = Arc<dyn Fn(String) + Send + Sync>;

/// The collections of the app: the list, the open collection and the call
/// that restarts the app.
#[derive(Clone)]
pub struct CollectionsPlace {
    collections: Collections,
    open: String,
    on_open: OnOpen,
}

impl std::fmt::Debug for CollectionsPlace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollectionsPlace")
            .field("collections", &self.collections)
            .field("open", &self.open)
            .finish_non_exhaustive()
    }
}

impl CollectionsPlace {
    /// The place for the app with the default data folder `root`. `open` is
    /// the folder name of the collection that this process opened.
    #[must_use]
    pub fn new(root: &std::path::Path, open: &str, on_open: OnOpen) -> Self {
        Self {
            collections: Collections::new(root),
            open: open.to_owned(),
            on_open,
        }
    }

    fn source(&self) -> PathBuf {
        self.collections.folder(&self.open)
    }
}

/// The state of the routes.
#[derive(Debug, Clone)]
pub(crate) struct CollectionsState {
    pub(crate) place: CollectionsPlace,
    pub(crate) work: Work,
}

fn api_error(error: &CollectionsError) -> ApiError {
    let (status, code) = match error.kind() {
        CollectionsErrorKind::InvalidName => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_name"),
        CollectionsErrorKind::NameTaken => (StatusCode::CONFLICT, "name_taken"),
        CollectionsErrorKind::NotFound => (StatusCode::NOT_FOUND, "not_found"),
        CollectionsErrorKind::Refused => (StatusCode::CONFLICT, "refused"),
        _ => return ApiError::internal(error.to_string()),
    };
    ApiError::new(status, code, error.to_string())
}

#[derive(Deserialize)]
struct NameBody {
    name: String,
}

/// `GET /api/v1/collections`: the list, with the count of the feeds of each.
async fn list(State(state): State<CollectionsState>) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        let entries = place.collections.list().map_err(|e| api_error(&e))?;
        let mut rows = Vec::with_capacity(entries.len());
        for entry in entries {
            let feeds = place
                .collections
                .feed_count(&entry.folder)
                .map_err(|e| api_error(&e))?;
            rows.push(json!({
                "folder": entry.folder,
                "name": entry.name,
                "feeds": feeds,
                "main": entry.folder == MAIN_FOLDER,
                "open": entry.folder == place.open,
            }));
        }
        Ok(Json(json!({ "collections": rows })))
    })
    .await
}

/// `GET /api/v1/collections/copy-size`: the size for the sheet of a copy.
async fn copy_size(State(state): State<CollectionsState>) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        let size = place
            .collections
            .copy_size(&place.source())
            .map_err(|e| api_error(&e))?;
        Ok(Json(json!({ "size": size })))
    })
    .await
}

/// `POST /api/v1/collections`: make a new collection.
async fn create(
    State(state): State<CollectionsState>,
    Json(body): Json<NameBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        let entry = place
            .collections
            .create(&body.name)
            .map_err(|e| api_error(&e))?;
        Ok((StatusCode::CREATED, Json(json!(entry))))
    })
    .await
}

/// `POST /api/v1/collections/copy`: copy the open collection. A write that
/// runs stops the copy, because the copy must hold one state of the files.
async fn copy(
    State(state): State<CollectionsState>,
    Json(body): Json<NameBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if state.work.writes() > 0 {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "work_running",
            "A write runs. Copy the collection after the write.",
        ));
    }
    let place = state.place;
    blocking_unlocked(move || {
        let entry = place
            .collections
            .copy(&place.source(), &body.name)
            .map_err(|e| api_error(&e))?;
        Ok((StatusCode::CREATED, Json(json!(entry))))
    })
    .await
}

/// `POST /api/v1/collections/{folder}/open`: restart the app on a collection.
async fn open(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let place = state.place;
    if folder == place.open {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "refused",
            "This collection is open.",
        ));
    }
    let entries = place.collections.clone();
    let known = folder.clone();
    let found = blocking_unlocked(move || {
        let list = entries.list().map_err(|e| api_error(&e))?;
        Ok(list.iter().any(|c| c.folder == known))
    })
    .await?;
    if !found {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "no collection has this folder",
        ));
    }
    (place.on_open)(folder);
    Ok((StatusCode::ACCEPTED, Json(json!({}))))
}

/// `DELETE /api/v1/collections/{folder}`: remove the local files of a
/// collection. Nothing changes on a file host (ADR 0149 item 13).
async fn remove(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        if folder == place.open {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "refused",
                "Studio cannot delete the open collection.",
            ));
        }
        place
            .collections
            .delete(&folder)
            .map_err(|e| api_error(&e))?;
        Ok(Json(json!({})))
    })
    .await
}

/// The routes of the collections. They are below the access layer of the
/// local router (ADR 0017).
pub(crate) fn routes(state: CollectionsState) -> Router {
    Router::new()
        .route("/api/v1/collections", get(list).post(create))
        .route("/api/v1/collections/copy-size", get(copy_size))
        .route("/api/v1/collections/copy", post(copy))
        .route("/api/v1/collections/{folder}/open", post(open))
        .route("/api/v1/collections/{folder}", delete(remove))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use axum::body::Body;
    use axum::http::{Request, header};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    use crate::{Access, DroppedFiles, WindowClose, Workspace, local_router_with_drops};

    const TOKEN: &str = "test-token";

    struct Fixture {
        _dir: tempfile::TempDir,
        router: Router,
        opened: Arc<Mutex<Vec<String>>>,
        work: Work,
        root: PathBuf,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("data");
        let open = studio_core::collections::prepare(&root).unwrap();
        let workspace = Workspace::open(&open).unwrap();
        let work = workspace.work();
        let opened = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&opened);
        let on_open: OnOpen = Arc::new(move |folder| seen.lock().unwrap().push(folder));
        let router = local_router_with_drops(
            workspace,
            Access::with_token(TOKEN, 42069),
            DroppedFiles::default(),
            WindowClose::default(),
            Some(CollectionsPlace::new(&root, MAIN_FOLDER, on_open)),
        );
        Fixture {
            _dir: dir,
            router,
            opened,
            work,
            root,
        }
    }

    async fn call(
        router: &Router,
        method: &str,
        path: &str,
        body: Option<Value>,
        token: bool,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, "127.0.0.1:42069");
        if token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {TOKEN}"));
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
            None => request.body(Body::empty()).unwrap(),
        };
        let response = router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    #[tokio::test]
    async fn the_routes_need_the_token_and_the_list_marks_main_and_open() {
        let fixture = fixture();
        let (status, _) = call(&fixture.router, "GET", "/api/v1/collections", None, false).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "ADR 0017");
        let (status, _) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections",
            Some(json!({ "name": "Test" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, list) = call(&fixture.router, "GET", "/api/v1/collections", None, true).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(list["collections"][0]["main"], json!(true));
        assert_eq!(list["collections"][0]["open"], json!(true));
        assert_eq!(list["collections"][1]["name"], json!("Test"));
        assert_eq!(list["collections"][1]["main"], json!(false));
        assert_eq!(list["collections"][1]["feeds"], json!(0));
    }

    #[tokio::test]
    async fn open_calls_the_app_and_does_not_change_the_selection_by_itself() {
        let fixture = fixture();
        call(
            &fixture.router,
            "POST",
            "/api/v1/collections",
            Some(json!({ "name": "Test" })),
            true,
        )
        .await;
        let (status, _) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections/test/open",
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(fixture.opened.lock().unwrap().as_slice(), ["test"]);
        assert_eq!(
            Collections::new(&fixture.root).open().unwrap(),
            MAIN_FOLDER,
            "ADR 0149 item 14: a canceled quit must keep the collection that is open."
        );
        for (folder, wanted) in [
            (MAIN_FOLDER, StatusCode::CONFLICT),
            ("nothing", StatusCode::NOT_FOUND),
        ] {
            let path = format!("/api/v1/collections/{folder}/open");
            let (status, _) = call(&fixture.router, "POST", &path, None, true).await;
            assert_eq!(status, wanted, "{folder}");
        }
    }

    #[tokio::test]
    async fn a_delete_refuses_main_and_removes_a_different_collection() {
        let fixture = fixture();
        call(
            &fixture.router,
            "POST",
            "/api/v1/collections",
            Some(json!({ "name": "Test" })),
            true,
        )
        .await;
        let (status, _) = call(
            &fixture.router,
            "DELETE",
            "/api/v1/collections/main",
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "ADR 0149 item 12");
        let (status, _) = call(
            &fixture.router,
            "DELETE",
            "/api/v1/collections/test",
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(!fixture.root.join("collections/test").exists());
        assert!(fixture.root.join("collections/main").exists());
    }

    #[tokio::test]
    async fn a_copy_gives_the_vault_to_the_copy_and_waits_for_a_write() {
        let fixture = fixture();
        let main = fixture.root.join("collections/main");
        std::fs::write(main.join("vault.json"), "vault").unwrap();
        let (status, size) = call(
            &fixture.router,
            "GET",
            "/api/v1/collections/copy-size",
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(size["size"].as_u64().unwrap() > 0);
        let guard = fixture.work.write();
        let (status, _) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections/copy",
            Some(json!({ "name": "Copy" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        drop(guard);
        let (status, made) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections/copy",
            Some(json!({ "name": "Copy" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(made["folder"], json!("copy"));
        assert_eq!(
            std::fs::read_to_string(fixture.root.join("collections/copy/vault.json")).unwrap(),
            "vault"
        );
    }

    #[tokio::test]
    async fn a_router_with_no_place_has_no_collection_route() {
        let dir = tempfile::tempdir().unwrap();
        let router = crate::local_router(
            Workspace::open(dir.path()).unwrap(),
            Access::with_token(TOKEN, 42069),
        );
        let (status, _) = call(&router, "GET", "/api/v1/collections", None, true).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "ADR 0149 item 4");
        let (_, session) = call(&router, "GET", "/auth/v1/session", None, true).await;
        assert_eq!(session["collections"], json!(false));
    }
}
