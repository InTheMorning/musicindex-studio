//! The files of the last drop on the window of the desktop app (ADR 0124
//! items 22 to 24).
//!
//! On Linux, `WebKitGTK` gives the page no file from a drop. The app gets the
//! path, puts it here, and the page gets the bytes one time from the local
//! router. The page cannot name a path. The server has no such route.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use tokio::io::AsyncReadExt;

use crate::ApiError;
use crate::access::random_hex;

/// The size of one read of the file.
const CHUNK_BYTES: usize = 1024 * 1024;

/// A cloneable holder of the files of the last drop.
#[derive(Debug, Clone, Default)]
pub struct DroppedFiles {
    files: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl DroppedFiles {
    /// Keep `paths` in the place of the files of the earlier drop, and give
    /// the ID of the route of each path.
    ///
    /// # Errors
    ///
    /// Returns an error when the system gives no random bytes.
    pub fn replace(&self, paths: Vec<PathBuf>) -> Result<Vec<String>, ApiError> {
        let named = paths
            .into_iter()
            .map(|path| Ok((random_hex::<16>()?, path)))
            .collect::<Result<Vec<_>, ApiError>>()?;
        let ids = named.iter().map(|(id, _)| id.clone()).collect();
        if let Ok(mut files) = self.files.lock() {
            *files = named.into_iter().collect();
        }
        Ok(ids)
    }

    fn take(&self, id: &str) -> Option<PathBuf> {
        self.files.lock().ok()?.remove(id)
    }
}

/// `GET /api/v1/dropped-files/{id}`: the bytes of the file, one time.
pub(crate) async fn serve(
    State(dropped): State<DroppedFiles>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let gone = || {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "dropped_file_not_found",
            "Studio has no dropped file with this ID.",
        )
    };
    let path = dropped.take(&id).ok_or_else(gone)?;
    let file = tokio::fs::File::open(path).await.map_err(|_| gone())?;
    let chunks = stream::unfold(Some(file), |file| async move {
        let mut file = file?;
        let mut chunk = vec![0_u8; CHUNK_BYTES];
        match file.read(&mut chunk).await {
            Ok(0) => None,
            Ok(count) => {
                chunk.truncate(count);
                Some((Ok(Bytes::from(chunk)), Some(file)))
            }
            Err(error) => Some((Err(error), None)),
        }
    });
    Ok(Body::from_stream(chunks).into_response())
}

#[cfg(test)]
mod tests {
    use axum::http::{Request, header};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    use crate::{Access, Workspace, local_router_with_drops, workspace_router};

    const TOKEN: &str = "0123456789abcdef";
    const HOST: &str = "127.0.0.1:42069";

    fn request(id: &str, token: Option<&str>) -> Request<Body> {
        let mut request =
            Request::get(format!("/api/v1/dropped-files/{id}")).header(header::HOST, HOST);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        request.body(Body::empty()).unwrap()
    }

    fn router(data: &tempfile::TempDir, dropped: DroppedFiles) -> axum::Router {
        let workspace = Workspace::open(data.path()).unwrap();
        local_router_with_drops(
            workspace,
            Access::with_token(TOKEN, 42069),
            dropped,
            crate::WindowClose::default(),
        )
    }

    #[tokio::test]
    async fn the_route_gives_the_dropped_file_one_time_and_needs_the_token() {
        let data = tempfile::tempdir().unwrap();
        let file = data.path().join("song.flac");
        std::fs::write(&file, b"the bytes").unwrap();
        let dropped = DroppedFiles::default();
        let other = data.path().join("other.flac");
        std::fs::write(&other, b"other bytes").unwrap();
        let ids = dropped.replace(vec![file, other]).unwrap();
        let (id, other_id) = (ids[0].clone(), ids[1].clone());
        let router = router(&data, dropped);

        let refused = router.clone().oneshot(request(&id, None)).await.unwrap();
        assert_eq!(
            refused.status(),
            StatusCode::UNAUTHORIZED,
            "ADR 0124 item 22: the route must keep the access rule of ADR 0017."
        );

        let first = router
            .clone()
            .oneshot(request(&id, Some(TOKEN)))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let body = first.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"the bytes");

        let other = router
            .clone()
            .oneshot(request(&other_id, Some(TOKEN)))
            .await
            .unwrap();
        let body = other.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"other bytes",
            "ADR 0125 item 4: the route must give each file of a drop."
        );

        let second = router.oneshot(request(&id, Some(TOKEN))).await.unwrap();
        assert_eq!(
            second.status(),
            StatusCode::NOT_FOUND,
            "ADR 0124 item 22: the route must give the file one time only."
        );
    }

    #[tokio::test]
    async fn a_new_drop_replaces_the_file_of_the_earlier_drop() {
        let data = tempfile::tempdir().unwrap();
        let file = data.path().join("song.flac");
        std::fs::write(&file, b"the bytes").unwrap();
        let dropped = DroppedFiles::default();
        let old = dropped.replace(vec![file.clone()]).unwrap().remove(0);
        let new = dropped.replace(vec![file]).unwrap().remove(0);
        let router = router(&data, dropped);
        let answer = router
            .clone()
            .oneshot(request(&old, Some(TOKEN)))
            .await
            .unwrap();
        assert_eq!(
            answer.status(),
            StatusCode::NOT_FOUND,
            "ADR 0124 item 22: only the file of the last drop has a route."
        );
        let answer = router.oneshot(request(&new, Some(TOKEN))).await.unwrap();
        assert_eq!(answer.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn the_routes_of_the_server_do_not_have_the_route() {
        let data = tempfile::tempdir().unwrap();
        let workspace = Workspace::open(data.path()).unwrap();
        let (router, _handle) = workspace_router(workspace);
        let answer = router.oneshot(request("any", None)).await.unwrap();
        assert_eq!(
            answer.status(),
            StatusCode::NOT_FOUND,
            "ADR 0124 item 22: only the local router has this route. \
             Add it in local_router_with_drops, not in workspace_router."
        );
    }
}
