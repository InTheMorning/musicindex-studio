//! The download queue (ADR 0038 items 6 to 12).
//!
//! Two worker threads download remote library files into the library. A
//! worker locks the workspace only to read the source and the limits, and
//! to call `download::finish`. It does not lock the workspace during a
//! transfer. A download writes only into the library, never to a host.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::Duration;

use serde::Serialize;
use studio_core::collection::{FeedId, LibraryFileId};
use studio_core::download::{self, DownloadErrorKind};

use crate::workspace::Workspace;

/// The number of worker threads (ADR 0038 item 7).
const WORKERS: usize = 2;

/// How long an idle worker waits before it examines the workspace again.
const IDLE_WAIT: Duration = Duration::from_millis(500);

/// The state of a download in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum State {
    /// The download waits for a worker.
    Queued,
    /// A worker transfers the file.
    Running,
    /// The download failed. The message says why.
    Failed,
}

/// The progress of one download, as the queue keeps it in memory.
#[derive(Debug, Clone)]
pub(crate) struct Progress {
    /// The feed of the library file.
    pub(crate) feed_id: FeedId,
    /// The state.
    pub(crate) state: State,
    /// The bytes that are in the partial file.
    pub(crate) bytes: u64,
    /// The size of the complete file, when Studio knows it.
    pub(crate) total: Option<u64>,
    /// The reason of a failure.
    pub(crate) message: Option<String>,
    stop: Arc<AtomicBool>,
}

#[derive(Debug, Default)]
struct Inner {
    /// The files that wait for a worker, in order.
    jobs: VecDeque<LibraryFileId>,
    /// The progress of each file in the queue, by library file ID.
    entries: HashMap<i64, Progress>,
    /// The files that a worker transfers now, so that two workers never
    /// write the same partial file.
    active: HashSet<i64>,
    /// True after the first start of the workers.
    started: bool,
}

/// The queue of downloads. The workspace keeps it in an `Arc`.
#[derive(Debug)]
pub(crate) struct DownloadQueue {
    inner: Mutex<Inner>,
    wake: Condvar,
    /// The workspace of the router. The workers lock it for short steps.
    /// The reference is weak, so a dropped router lets the workers end.
    workspace: Mutex<Option<Weak<Mutex<Workspace>>>>,
}

impl DownloadQueue {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner::default()),
            wake: Condvar::new(),
            workspace: Mutex::new(None),
        })
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Give the queue the workspace of the router. Workers start when
    /// downloads wait already, for example after a restart.
    pub(crate) fn attach(self: &Arc<Self>, workspace: &Arc<Mutex<Workspace>>) {
        *self
            .workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(workspace));
        if !self.inner().jobs.is_empty() {
            self.start_workers();
        }
    }

    /// Put a file in the queue. Gives false, and does nothing, when the
    /// file is queued or running already. A failed file goes in again.
    pub(crate) fn add(self: &Arc<Self>, file: LibraryFileId, feed_id: FeedId, total: u64) -> bool {
        {
            let mut inner = self.inner();
            if inner
                .entries
                .get(&file.0)
                .is_some_and(|entry| entry.state != State::Failed)
            {
                return false;
            }
            inner.entries.insert(
                file.0,
                Progress {
                    feed_id,
                    state: State::Queued,
                    bytes: 0,
                    total: (total > 0).then_some(total),
                    message: None,
                    stop: Arc::new(AtomicBool::new(false)),
                },
            );
            inner.jobs.push_back(file);
        }
        self.start_workers();
        self.wake.notify_one();
        true
    }

    /// Stop a download and take it out of the queue. The partial file stays.
    pub(crate) fn stop(&self, file: LibraryFileId) {
        let mut inner = self.inner();
        if let Some(entry) = inner.entries.remove(&file.0) {
            entry.stop.store(true, Ordering::Relaxed);
        }
        inner.jobs.retain(|job| *job != file);
    }

    /// The download of a file, when the queue has it.
    pub(crate) fn get(&self, file: LibraryFileId) -> Option<Progress> {
        self.inner().entries.get(&file.0).cloned()
    }

    /// Each download in the queue, with its library file ID.
    pub(crate) fn entries(&self) -> Vec<(i64, Progress)> {
        let mut all: Vec<_> = self
            .inner()
            .entries
            .iter()
            .map(|(id, entry)| (*id, entry.clone()))
            .collect();
        all.sort_by_key(|(id, _)| *id);
        all
    }

    fn start_workers(self: &Arc<Self>) {
        let attached = self
            .workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();
        if !attached {
            return;
        }
        {
            let mut inner = self.inner();
            if inner.started {
                return;
            }
            inner.started = true;
        }
        for _ in 0..WORKERS {
            let queue = Arc::clone(self);
            std::thread::spawn(move || queue.work());
        }
    }

    /// The next file for a worker. Gives `None` when the workspace is gone.
    fn next(&self) -> Option<LibraryFileId> {
        let mut inner = self.inner();
        loop {
            self.workspace()?;
            let position = inner
                .jobs
                .iter()
                .position(|job| !inner.active.contains(&job.0));
            if let Some(position) = position {
                let file = inner.jobs.remove(position)?;
                if inner.entries.contains_key(&file.0) {
                    inner.active.insert(file.0);
                    return Some(file);
                }
                // The user stopped the download while it waited.
                continue;
            }
            inner = self
                .wake
                .wait_timeout(inner, IDLE_WAIT)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
    }

    fn workspace(&self) -> Option<Arc<Mutex<Workspace>>> {
        self.workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade)
    }

    fn work(self: Arc<Self>) {
        while let Some(file) = self.next() {
            if let Some(workspace) = self.workspace() {
                self.download(&workspace, file);
            }
            self.inner().active.remove(&file.0);
            self.wake.notify_all();
        }
    }

    fn update(&self, file: LibraryFileId, change: impl FnOnce(&mut Progress)) {
        if let Some(entry) = self.inner().entries.get_mut(&file.0) {
            change(entry);
        }
    }

    fn progress(&self, file: LibraryFileId, done: u64, total: Option<u64>) {
        self.update(file, |entry| {
            entry.bytes = done;
            entry.total = total.or(entry.total);
        });
    }

    /// Remove the partial file, and record the same code as the upload
    /// route gives (ADR 0055 item 21).
    fn fail_storage_full(
        &self,
        workspace: &Arc<Mutex<Workspace>>,
        file: LibraryFileId,
        part: &std::path::Path,
    ) {
        let _ = std::fs::remove_file(part);
        let message = crate::workspace::storage_full().coded();
        self.fail(workspace, file, &message);
    }

    /// Record a failure in memory and in the collection.
    fn fail(&self, workspace: &Arc<Mutex<Workspace>>, file: LibraryFileId, message: &str) {
        self.update(file, |entry| {
            entry.state = State::Failed;
            entry.message = Some(message.to_owned());
        });
        if let Ok(w) = workspace.lock() {
            let _ = w.collection.fail_download(file, message);
        }
    }

    /// One download, from the source to `download::finish`.
    fn download(&self, workspace: &Arc<Mutex<Workspace>>, file: LibraryFileId) {
        let Some(stop) = self.get(file).map(|entry| Arc::clone(&entry.stop)) else {
            return;
        };
        self.update(file, |entry| entry.state = State::Running);

        // Step 1: with the lock, read the source and the limits.
        let prepared = {
            let Ok(w) = workspace.lock() else { return };
            (|| -> Result<_, String> {
                let row = w.collection.library_file(file).map_err(|e| e.to_string())?;
                if row.local {
                    return Ok(None);
                }
                let hosts = w.hosts().map_err(|e| format!("{e:?}"))?;
                let library = w.library().map_err(|e| format!("{e:?}"))?;
                let feed = w.collection.feed(row.feed_id).map_err(|e| e.to_string())?;
                let folder = feed
                    .folder
                    .ok_or_else(|| format!("feed {} has no release folder", row.feed_id.0))?;
                let source = download::source_of(&w.collection, &hosts, &row)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "the file has no copy on a host and no address".to_owned())?;
                download::check_limits(&w.collection, &library, row.size)
                    .map_err(|e| e.to_string())?;
                let max = download::max_file_bytes(&w.collection).map_err(|e| e.to_string())?;
                let part = download::part_path(&library, &folder, &row.path);
                // The storage limit of the server (ADR 0055 item 21).
                let rules = w.rules.clone();
                if let Some(rules) = &rules {
                    rules.check_storage().map_err(|e| e.coded())?;
                }
                Ok(Some((
                    hosts,
                    library,
                    source,
                    max,
                    part,
                    w.web.clone(),
                    rules,
                )))
            })()
        };
        let (hosts, library, source, max, part, web, rules) = match prepared {
            Ok(Some(prepared)) => prepared,
            Ok(None) => {
                // The bytes are in the library already.
                if let Ok(w) = workspace.lock() {
                    let _ = w.collection.remove_download(file);
                }
                self.inner().entries.remove(&file.0);
                return;
            }
            Err(message) => return self.fail(workspace, file, &message),
        };

        // Step 2: with no lock, transfer the bytes. On the server, the
        // transfer stops when the user folder reaches its limit. The folder is
        // counted after each 8 MiB (ADR 0055 item 21).
        let full = AtomicBool::new(false);
        let mut counted = 0_u64;
        let result = download::fetch(
            &hosts,
            &web,
            &source,
            &part,
            max,
            &mut |done, total| {
                self.progress(file, done, total);
                if let Some(rules) = &rules
                    && done.saturating_sub(counted) >= crate::workspace::STORAGE_CHECK_BYTES
                {
                    counted = done;
                    if rules.check_storage().is_err() {
                        full.store(true, Ordering::Relaxed);
                        stop.store(true, Ordering::Relaxed);
                    }
                }
            },
            &stop,
        );
        if full.load(Ordering::Relaxed) {
            return self.fail_storage_full(workspace, file, &part);
        }
        match result {
            Err(error) if error.kind() == DownloadErrorKind::Stopped => return,
            Err(error) => return self.fail(workspace, file, &error.to_string()),
            Ok(_) => {}
        }

        // Step 3: with no lock, read the size and the SHA-256. A large
        // file takes some seconds, and other requests must not wait.
        let facts = match download::hash_part(&part) {
            Ok(facts) => facts,
            Err(error) => return self.fail(workspace, file, &error.to_string()),
        };

        // Step 4: with the lock, move the bytes to their name and record them.
        let finished = {
            let Ok(w) = workspace.lock() else { return };
            w.collection
                .library_file(file)
                .map_err(|e| e.to_string())
                .and_then(|row| {
                    download::finish(&w.collection, &library, &row, &part, &facts)
                        .map_err(|e| e.to_string())
                })
        };
        match finished {
            Ok(()) => {
                self.inner().entries.remove(&file.0);
            }
            Err(message) => self.fail(workspace, file, &message),
        }
    }
}
