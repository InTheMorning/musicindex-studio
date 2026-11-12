//! The routes of the collections of the desktop app (ADR 0149, ADR 0176).
//!
//! Only the router of the desktop app has these routes. A server, and a shell
//! with `--data-dir`, have none.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::account_archive::{LibraryPolicy, Limits};
use studio_core::collection::Collection;
use studio_core::collections::{Collections, CollectionsError, CollectionsErrorKind};
use studio_core::vault::VaultFile;

use crate::import::ImportStore;
use crate::routes::blocking_unlocked;
use crate::{ApiError, Work, WorkspaceHandle};

/// What the app does when the user selects "Open" (ADR 0149 item 14).
pub type OnOpen = Arc<dyn Fn(String) + Send + Sync>;

/// The collections of the app: the list, the open collection and the call
/// that restarts the app.
#[derive(Clone)]
pub struct CollectionsPlace {
    collections: Collections,
    open: String,
    on_open: OnOpen,
    /// The files of the replace reads, in the data folder, so that the new
    /// folder is on the same file system.
    imports: Arc<ImportStore>,
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
            imports: Arc::new(ImportStore::new(root.join("replace-staging"))),
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
    pub(crate) handle: WorkspaceHandle,
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

/// `PATCH /api/v1/collections/{folder}`: change the name (ADR 0176 item 5).
async fn rename(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
    Json(body): Json<NameBody>,
) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        let entry = place
            .collections
            .rename(&folder, &body.name)
            .map_err(|e| api_error(&e))?;
        Ok(Json(json!(entry)))
    })
    .await
}

/// The folder of a collection that is listed and not open, or the error.
fn closed_folder(place: &CollectionsPlace, folder: &str) -> Result<PathBuf, ApiError> {
    let list = place.collections.list().map_err(|e| api_error(&e))?;
    if !list.iter().any(|c| c.folder == folder) {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "no collection has this folder",
        ));
    }
    if folder == place.open {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "refused",
            "This collection is open. Use the export of the account.",
        ));
    }
    Ok(place.collections.folder(folder))
}

/// The unlock methods of the vault file in `folder`. A folder with no vault
/// has none. The call opens no vault (ADR 0176 item 7).
fn vault_methods_of(
    folder: &std::path::Path,
) -> Result<Vec<studio_core::vault::VaultMethod>, ApiError> {
    if !folder.join("vault.json").exists() {
        return Ok(Vec::new());
    }
    Ok(VaultFile::load(&folder.join("vault.json"))?.methods())
}

/// `GET /api/v1/collections/{folder}/export-facts`: the facts of the export
/// sheet for a collection that is not open (ADR 0176 item 7).
async fn export_facts(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    state
        .handle
        .run(move |w| {
            let path = closed_folder(&place, &folder)?;
            let methods = vault_methods_of(&path)?;
            let counts = place
                .collections
                .account_counts(&folder)
                .map_err(|e| api_error(&e))?;
            let library = place
                .collections
                .archive_library(&folder)
                .map_err(|e| api_error(&e))?;
            Ok(Json(crate::account::facts_of(
                w,
                &path,
                Some(&library),
                counts,
                &methods,
            )?))
        })
        .await
}

#[derive(Deserialize)]
struct ExportBody {
    #[serde(default = "yes")]
    include_assets: bool,
}

fn yes() -> bool {
    true
}

/// `POST /api/v1/collections/{folder}/export`: make a media ticket for the
/// archive of a collection that is not open (ADR 0176 item 7). The route
/// needs a recent unlock of the vault that is open. It does not open the
/// vault of the exported collection.
async fn export(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
    Json(body): Json<ExportBody>,
) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    state
        .handle
        .run(move |w| {
            let path = closed_folder(&place, &folder)?;
            let methods = vault_methods_of(&path)?;
            let library = place
                .collections
                .archive_library(&folder)
                .map_err(|e| api_error(&e))?;
            crate::account::issue_export(
                w,
                &path,
                Some(path.clone()),
                Some(&library),
                &methods,
                body.include_assets,
            )
        })
        .await
}

#[derive(Deserialize)]
struct SizeBody {
    size: u64,
}

/// `POST /api/v1/collections/{folder}/replace`: start the upload of the
/// account file for a replace (ADR 0176 item 8).
async fn replace_start(
    State(state): State<CollectionsState>,
    Path(folder): Path<String>,
    Json(body): Json<SizeBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        closed_folder(&place, &folder)?;
        let started = place
            .imports
            .start(body.size, Limits::DEFAULT.max_total_bytes, None)?;
        Ok((StatusCode::CREATED, Json(started)))
    })
    .await
}

/// `PUT /api/v1/collection-uploads/{upload}/parts/{n}`: send part `n`.
async fn replace_part(
    State(state): State<CollectionsState>,
    Path((upload, n)): Path<(String, u32)>,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let store = Arc::clone(&state.place.imports);
    blocking_unlocked(move || store.put_part(&upload, n, &body))
        .await
        .map(Json)
}

/// `DELETE /api/v1/collection-uploads/{upload}`: stop an upload.
async fn replace_cancel(
    State(state): State<CollectionsState>,
    Path(upload): Path<String>,
) -> Result<StatusCode, ApiError> {
    let store = Arc::clone(&state.place.imports);
    blocking_unlocked(move || store.cancel(&upload)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct FolderBody {
    folder: String,
}

/// Move the files of the checked account file into the new folder, and
/// reset the connected apps (ADR 0110 item 41).
fn fill_from(staged: &std::path::Path, target: &std::path::Path) -> Result<(), CollectionsError> {
    let io = |what: &str, e: &dyn std::fmt::Display| {
        CollectionsError::new(CollectionsErrorKind::Io, format!("{what}: {e}"))
    };
    for name in ["vault.json", "collection.sqlite", "grants", "library"] {
        let from = staged.join(name);
        if from.exists() {
            std::fs::rename(&from, target.join(name)).map_err(|e| io(name, &e))?;
        }
    }
    let database = target.join("collection.sqlite");
    let now = i64::try_from(crate::announcer::now()).unwrap_or(i64::MAX);
    let collection = Collection::open(&database).map_err(|e| io("collection.sqlite", &e))?;
    studio_core::apps::reset_after_import(&collection, now).map_err(|e| io("apps", &e))
}

/// `POST /api/v1/collection-uploads/{upload}/replace`: check the file, read
/// it into a new folder and move the entry of `folder` to it (ADR 0176
/// items 8 and 9). A file that is not good changes nothing. The vault of the
/// file stays locked: the user opens it with a method of the file.
async fn replace_finish(
    State(state): State<CollectionsState>,
    Path(upload): Path<String>,
    Json(body): Json<FolderBody>,
) -> Result<Json<Value>, ApiError> {
    let place = state.place;
    blocking_unlocked(move || {
        closed_folder(&place, &body.folder)?;
        let store = Arc::clone(&place.imports);
        store.check(
            &upload,
            Limits::DEFAULT.max_total_bytes,
            LibraryPolicy::Relative,
        )?;
        let staged = store.staged(&upload)?;
        let replaced = place
            .collections
            .replace(&body.folder, |target| fill_from(&staged.folder, target));
        store.finish(&upload);
        let entry = replaced.map_err(|e| api_error(&e))?;
        let feeds = place
            .collections
            .feed_count(&entry.folder)
            .map_err(|e| api_error(&e))?;
        Ok(Json(
            json!({ "folder": entry.folder, "name": entry.name, "feeds": feeds }),
        ))
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
        .route("/api/v1/collections/{folder}", delete(remove).patch(rename))
        .route(
            "/api/v1/collections/{folder}/export-facts",
            get(export_facts),
        )
        .route("/api/v1/collections/{folder}/export", post(export))
        .route("/api/v1/collections/{folder}/replace", post(replace_start))
        .route(
            "/api/v1/collection-uploads/{upload}/parts/{n}",
            put(replace_part).layer(DefaultBodyLimit::max(
                usize::try_from(crate::uploads::PART_SIZE).unwrap_or(usize::MAX),
            )),
        )
        .route(
            "/api/v1/collection-uploads/{upload}",
            delete(replace_cancel),
        )
        .route(
            "/api/v1/collection-uploads/{upload}/replace",
            post(replace_finish),
        )
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

    const MAIN_FOLDER: &str = studio_core::collections::MAIN_FOLDER;

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
    async fn the_routes_need_the_token_and_the_list_marks_the_open_collection() {
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
        assert_eq!(list["collections"][0]["open"], json!(true));
        assert_eq!(list["collections"][1]["name"], json!("Test"));
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
    async fn a_delete_refuses_the_open_collection_and_removes_a_different_one() {
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
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "ADR 0176 item 3: main is open"
        );
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
    async fn a_rename_changes_the_name_of_the_open_collection_too() {
        let fixture = fixture();
        let (status, entry) = call(
            &fixture.router,
            "PATCH",
            "/api/v1/collections/main",
            Some(json!({ "name": "Production" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(entry["folder"], json!("main"));
        let (_, list) = call(&fixture.router, "GET", "/api/v1/collections", None, true).await;
        assert_eq!(list["collections"][0]["name"], json!("Production"));
        assert!(list["collections"][0].get("main").is_none());
        let (status, _) = call(
            &fixture.router,
            "PATCH",
            "/api/v1/collections/main",
            Some(json!({ "name": " " })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// A collection with a vault that a passphrase opens, a mark in its
    /// database and a library file.
    async fn account_collection(fixture: &Fixture, name: &str, mark: &str) -> PathBuf {
        let (_, made) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections",
            Some(json!({ "name": name })),
            true,
        )
        .await;
        let folder = fixture
            .root
            .join("collections")
            .join(made["folder"].as_str().unwrap());
        studio_core::vault::OpenVault::create("correct horse battery")
            .unwrap()
            .file()
            .save(&folder.join("vault.json"))
            .unwrap();
        Collection::open(&folder.join("collection.sqlite"))
            .unwrap()
            .set_setting("mark", &json!(mark))
            .unwrap();
        std::fs::create_dir_all(folder.join("library")).unwrap();
        std::fs::write(folder.join("library/a.xml"), mark).unwrap();
        folder
    }

    async fn send_bytes(router: &Router, method: &str, path: &str, bytes: Vec<u8>) -> StatusCode {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, "127.0.0.1:42069")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::from(bytes))
            .unwrap();
        router.clone().oneshot(request).await.unwrap().status()
    }

    async fn replace_with(fixture: &Fixture, folder: &str, bytes: Vec<u8>) -> (StatusCode, Value) {
        let (status, started) = call(
            &fixture.router,
            "POST",
            &format!("/api/v1/collections/{folder}/replace"),
            Some(json!({ "size": bytes.len() })),
            true,
        )
        .await;
        if status != StatusCode::CREATED {
            return (status, started);
        }
        let upload = started["upload"].as_str().unwrap();
        let part = usize::try_from(started["part_size"].as_u64().unwrap()).unwrap();
        for (n, chunk) in bytes.chunks(part).enumerate() {
            let path = format!("/api/v1/collection-uploads/{upload}/parts/{n}");
            assert_eq!(
                send_bytes(&fixture.router, "PUT", &path, chunk.to_vec()).await,
                StatusCode::OK
            );
        }
        call(
            &fixture.router,
            "POST",
            &format!("/api/v1/collection-uploads/{upload}/replace"),
            Some(json!({ "folder": folder })),
            true,
        )
        .await
    }

    #[tokio::test]
    async fn a_replace_reads_the_file_into_a_new_folder_and_a_bad_file_changes_nothing() {
        let fixture = fixture();
        let closed = account_collection(&fixture, "Closed", "old").await;
        let source = account_collection(&fixture, "Source", "new").await;
        let mut bytes = Vec::new();
        studio_core::account_archive::write_archive(
            &source,
            &studio_core::account_archive::Manifest::new(None, Vec::new()),
            true,
            &mut bytes,
        )
        .unwrap();
        let list_before = std::fs::read_to_string(fixture.root.join("collections.json")).unwrap();
        let (status, _) = replace_with(&fixture, "closed", b"not an archive".to_vec()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            std::fs::read_to_string(fixture.root.join("collections.json")).unwrap(),
            list_before,
            "ADR 0176 item 8: a bad file must change nothing"
        );
        assert!(closed.join("vault.json").exists());
        let (status, _) = replace_with(&fixture, "main", bytes.clone()).await;
        assert_eq!(status, StatusCode::CONFLICT, "ADR 0176 item 3");
        let (status, made) = replace_with(&fixture, "closed", bytes).await;
        assert_eq!(status, StatusCode::OK, "{made}");
        assert_eq!(made["name"], json!("Closed"));
        let replaced = fixture
            .root
            .join("collections")
            .join(made["folder"].as_str().unwrap());
        assert!(!closed.exists(), "the previous folder must go");
        assert_eq!(
            std::fs::read(replaced.join("vault.json")).unwrap(),
            std::fs::read(source.join("vault.json")).unwrap(),
            "ADR 0176 item 9: the vault is the vault of the file"
        );
        let mark = Collection::open(&replaced.join("collection.sqlite"))
            .unwrap()
            .setting("mark")
            .unwrap();
        assert_eq!(mark, Some(json!("new")));
        assert_eq!(
            std::fs::read_to_string(replaced.join("library/a.xml")).unwrap(),
            "new"
        );
    }

    fn tar_files(bytes: &[u8]) -> std::collections::BTreeMap<String, Vec<u8>> {
        use std::io::Read as _;
        let mut files = std::collections::BTreeMap::new();
        for entry in tar::Archive::new(bytes).entries().unwrap() {
            let mut entry = entry.unwrap();
            let name = entry.path().unwrap().display().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            files.insert(name, data);
        }
        files
    }

    async fn download(router: &Router, ticket: &str) -> Vec<u8> {
        let request = Request::builder()
            .uri(format!("/media/download?ticket={ticket}"))
            .header(header::HOST, "127.0.0.1:42069")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec()
    }

    #[tokio::test]
    async fn an_export_of_a_closed_collection_gives_the_archive_of_the_open_one() {
        let fixture = fixture();
        let closed = account_collection(&fixture, "Closed", "mark").await;
        // The open vault needs a recent unlock. The closed vault stays shut.
        let (status, _) = call(
            &fixture.router,
            "POST",
            "/api/v1/vault",
            Some(json!({ "passphrase": "correct horse battery" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, facts) = call(
            &fixture.router,
            "GET",
            "/api/v1/collections/closed/export-facts",
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{facts}");
        assert_eq!(facts["movable"], json!(true));
        let (status, ticket) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections/closed/export",
            Some(json!({ "include_assets": true })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ticket}");
        let closed_files =
            tar_files(&download(&fixture.router, ticket["ticket"].as_str().unwrap()).await);
        // The same folder, as the open collection of a second app.
        let router = crate::local_router(
            Workspace::open(&closed).unwrap(),
            Access::with_token(TOKEN, 42069),
        );
        let (status, _) = call(
            &router,
            "POST",
            "/api/v1/vault/unlock",
            Some(json!({ "passphrase": "correct horse battery" })),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (_, ticket) = call(
            &router,
            "POST",
            "/api/v1/account/export",
            Some(json!({ "include_assets": true })),
            true,
        )
        .await;
        let mut open_files =
            tar_files(&download(&router, ticket["ticket"].as_str().unwrap()).await);
        // Opening a collection makes an empty folder for the grant vaults.
        open_files.remove("grants");
        assert_eq!(
            closed_files.keys().collect::<Vec<_>>(),
            open_files.keys().collect::<Vec<_>>(),
            "ADR 0176 item 7"
        );
        for name in ["vault.json", "library/a.xml"] {
            assert_eq!(closed_files[name], open_files[name], "{name}");
        }
        let (status, _) = call(
            &fixture.router,
            "POST",
            "/api/v1/collections/main/export",
            Some(json!({})),
            true,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "the open collection uses the account export"
        );
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
