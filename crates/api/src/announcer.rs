//! The announcement worker (ADR 0052 items 6, 14 and 15).
//!
//! One thread sends each due announcement to the hub. The thread locks the
//! workspace only to take a due announcement and to record the result. It
//! does not lock the workspace during the request.

use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

use studio_core::announce;

use crate::workspace::Workspace;

/// The longest wait of an idle worker before it examines the queue again.
const IDLE_WAIT: Duration = Duration::from_secs(1);

/// The time now, in seconds after the Unix epoch. The worker and the tests
/// read the clock through this function.
pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The worker of the announcement queue. The workspace keeps it in an
/// `Arc`.
#[derive(Debug)]
pub(crate) struct Announcer {
    /// True when someone woke the worker and it did not run yet.
    woken: Mutex<bool>,
    wake: Condvar,
    /// The workspace of the router. The reference is weak, so a dropped
    /// router lets the worker end.
    workspace: Mutex<Option<Weak<Mutex<Workspace>>>>,
}

/// One step of the worker.
enum Step {
    /// An announcement to send, with the web client and the hub.
    Send(Box<announce::Announcement>, studio_core::web::Web, String),
    /// No announcement is due. The value is the time of the next one.
    Idle(Option<u64>),
}

impl Announcer {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            woken: Mutex::new(false),
            wake: Condvar::new(),
            workspace: Mutex::new(None),
        })
    }

    /// Give the worker the workspace of the router, and start the thread.
    pub(crate) fn attach(self: &Arc<Self>, workspace: &Arc<Mutex<Workspace>>) {
        *self
            .workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(workspace));
        let announcer = Arc::clone(self);
        std::thread::spawn(move || announcer.work());
    }

    /// Tell the worker to examine the queue now.
    pub(crate) fn wake(&self) {
        *self
            .woken
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.wake.notify_all();
    }

    fn workspace(&self) -> Option<Arc<Mutex<Workspace>>> {
        self.workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade)
    }

    /// Wait until `until`, for a maximum of [`IDLE_WAIT`], or until a wake.
    fn sleep(&self, until: Option<u64>) {
        let wait = until.map_or(IDLE_WAIT, |at| {
            Duration::from_secs(at.saturating_sub(now())).min(IDLE_WAIT)
        });
        let mut woken = self
            .woken
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*woken && !wait.is_zero() {
            woken = self
                .wake
                .wait_timeout(woken, wait)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        *woken = false;
    }

    /// The next step, with the workspace locked. The lock ends here.
    fn step(workspace: &Mutex<Workspace>) -> Option<Step> {
        let w = workspace.lock().ok()?;
        match announce::take_due(&w.collection, now()) {
            Ok(Some(announcement)) => Some(Step::Send(
                Box::new(announcement),
                w.hub_web.clone(),
                w.hub.clone(),
            )),
            Ok(None) => Some(Step::Idle(
                announce::next_due_at(&w.collection).ok().flatten(),
            )),
            Err(error) => {
                eprintln!("studio: the announcement queue failed: {error}");
                Some(Step::Idle(None))
            }
        }
    }

    fn work(self: Arc<Self>) {
        loop {
            // The strong reference lasts for one step only, so that a
            // dropped router ends the thread.
            let idle = {
                let Some(workspace) = self.workspace() else {
                    return;
                };
                match Self::step(&workspace) {
                    None => return,
                    Some(Step::Send(announcement, web, hub)) => {
                        // Step 2: with no lock, send the request.
                        let result = announce::send(&web, &hub, &announcement);
                        Self::record(&workspace, &announcement, &result);
                        None
                    }
                    Some(Step::Idle(next)) => Some(next),
                }
            };
            if let Some(next) = idle {
                self.sleep(next);
            }
        }
    }

    /// Step 3: with the lock, record the result.
    fn record(
        workspace: &Mutex<Workspace>,
        announcement: &announce::Announcement,
        result: &Result<(), announce::AnnounceError>,
    ) {
        let Ok(w) = workspace.lock() else { return };
        let recorded = match result {
            Ok(()) => announce::record_sent(&w.collection, announcement.id, now()),
            Err(error) => announce::record_failure(
                &w.collection,
                announcement.id,
                &error.to_string(),
                &w.announce_timing,
                now(),
            ),
        };
        if let Err(error) = recorded {
            eprintln!("studio: the announcement result was not recorded: {error}");
        }
    }
}
