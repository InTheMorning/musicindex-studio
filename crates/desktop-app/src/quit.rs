//! The rule of "Quit" while work runs (ADR 0124 items 16 to 20).
//!
//! This module has no Tauri code and no I/O. The app reads the work count,
//! gives the state and an event to [`step`], and does the action.

/// The work that runs: the writes and the uploads in parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkCount {
    /// The requests that write to a file host.
    pub writes: usize,
    /// The uploads in parts.
    pub uploads: usize,
}

impl WorkCount {
    /// True when no work runs.
    #[must_use]
    pub fn is_idle(self) -> bool {
        self.writes == 0 && self.uploads == 0
    }
}

/// The state of the quit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuitState {
    /// The app runs. No quit is in progress.
    #[default]
    Running,
    /// The dialog shows.
    Asking,
    /// The app waits until no work runs.
    Waiting,
}

/// An event of the quit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitEvent {
    /// The user selects "Quit" in the tray.
    Quit,
    /// The user selects "Cancel" in the dialog, or closes it.
    Cancel,
    /// The user selects "Quit Now".
    QuitNow,
    /// The user selects "Quit After Write".
    QuitAfterWrite,
    /// The user selects "Open Studio" in the tray.
    OpenStudio,
    /// The wait finds that no work runs.
    NoWork,
}

/// What the app does after an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitAction {
    /// Send the shutdown signal and end with no wait.
    EndNow,
    /// Show the dialog of the system.
    ShowDialog,
    /// Hide the window, show the menu of the wait and watch the work.
    Wait,
    /// Do nothing more. The app continues.
    Continue,
    /// Send the shutdown signal, wait for the end of the API and end.
    StopApiAndEnd,
}

/// The next state and the action for `event` in `state` (ADR 0124 items
/// 16 to 20). A "Quit" while the dialog shows does nothing.
#[must_use]
pub fn step(work: WorkCount, state: QuitState, event: QuitEvent) -> (QuitState, QuitAction) {
    use QuitAction as A;
    use QuitEvent as E;
    use QuitState as S;
    match (state, event) {
        (S::Running, E::Quit) if work.is_idle() => (S::Running, A::EndNow),
        (S::Running, E::Quit) => (S::Asking, A::ShowDialog),
        (S::Asking, E::QuitAfterWrite) if work.is_idle() => (S::Waiting, A::StopApiAndEnd),
        (S::Asking, E::QuitAfterWrite) => (S::Waiting, A::Wait),
        (S::Asking, E::Cancel) | (S::Waiting, E::OpenStudio) => (S::Running, A::Continue),
        (_, E::QuitNow) => (state, A::EndNow),
        (S::Waiting, E::NoWork) => (S::Waiting, A::StopApiAndEnd),
        // The remaining pairs change nothing.
        (state, _) => (state, A::Continue),
    }
}

/// The title of the dialog. It counts the work (ADR 0124 item 18).
#[must_use]
pub fn dialog_title(work: WorkCount) -> String {
    let writing = (work.writes > 0).then(|| {
        let noun = if work.writes == 1 { "Feed" } else { "Feeds" };
        format!("Writing {} {noun}", work.writes)
    });
    let receiving = (work.uploads > 0).then(|| {
        let noun = if work.uploads == 1 {
            "Music File"
        } else {
            "Music Files"
        };
        format!("Receiving {} {noun}", work.uploads)
    });
    match (writing, receiving) {
        (Some(w), Some(r)) => format!("Studio Is {w} And {r}"),
        (Some(part), None) | (None, Some(part)) => format!("Studio Is {part}"),
        (None, None) => "Studio Is Working".to_string(),
    }
}

/// The message of the dialog (ADR 0124 item 18).
pub const DIALOG_MESSAGE: &str =
    "If you quit now, the write stops. You then must do the write again.";
/// The default button of the dialog.
pub const QUIT_AFTER_WRITE: &str = "Quit After Write";
/// The destructive button of the dialog.
pub const QUIT_NOW: &str = "Quit Now";
/// The button that closes the dialog.
pub const CANCEL: &str = "Cancel";

/// The event for the label of the button that the user selected. A label
/// that is not known, and a closed dialog, give "Cancel".
#[must_use]
pub fn event_for_label(label: &str) -> QuitEvent {
    match label {
        QUIT_AFTER_WRITE => QuitEvent::QuitAfterWrite,
        QUIT_NOW => QuitEvent::QuitNow,
        _ => QuitEvent::Cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use QuitAction as A;
    use QuitEvent as E;
    use QuitState as S;

    const BUSY: WorkCount = WorkCount {
        writes: 1,
        uploads: 0,
    };
    const IDLE: WorkCount = WorkCount {
        writes: 0,
        uploads: 0,
    };

    #[test]
    fn quit_with_no_work_ends_now_and_with_work_asks() {
        assert_eq!(
            step(IDLE, S::Running, E::Quit),
            (S::Running, A::EndNow),
            "ADR 0124 item 16: Quit with no work must end the app with no dialog."
        );
        assert_eq!(
            step(BUSY, S::Running, E::Quit),
            (S::Asking, A::ShowDialog),
            "ADR 0124 item 18: Quit with work must show the dialog."
        );
        let uploads = WorkCount {
            writes: 0,
            uploads: 1,
        };
        assert_eq!(step(uploads, S::Running, E::Quit).1, A::ShowDialog);
    }

    #[test]
    fn each_event_in_the_state_running() {
        for work in [IDLE, BUSY] {
            assert_eq!(step(work, S::Running, E::Cancel), (S::Running, A::Continue));
            assert_eq!(step(work, S::Running, E::QuitNow), (S::Running, A::EndNow));
            assert_eq!(
                step(work, S::Running, E::QuitAfterWrite),
                (S::Running, A::Continue)
            );
            assert_eq!(
                step(work, S::Running, E::OpenStudio),
                (S::Running, A::Continue)
            );
            assert_eq!(step(work, S::Running, E::NoWork), (S::Running, A::Continue));
        }
    }

    #[test]
    fn each_event_in_the_state_asking() {
        for work in [IDLE, BUSY] {
            assert_eq!(
                step(work, S::Asking, E::Quit),
                (S::Asking, A::Continue),
                "ADR 0124 item 18: Quit while the dialog shows must do nothing."
            );
            assert_eq!(
                step(work, S::Asking, E::Cancel),
                (S::Running, A::Continue),
                "ADR 0124 item 19: Cancel must let the app continue."
            );
            assert_eq!(
                step(work, S::Asking, E::QuitNow),
                (S::Asking, A::EndNow),
                "ADR 0124 item 19: Quit Now must end the app."
            );
            assert_eq!(
                step(work, S::Asking, E::OpenStudio),
                (S::Asking, A::Continue)
            );
            assert_eq!(step(work, S::Asking, E::NoWork), (S::Asking, A::Continue));
        }
        assert_eq!(
            step(BUSY, S::Asking, E::QuitAfterWrite),
            (S::Waiting, A::Wait),
            "ADR 0124 item 20: Quit After Write must wait."
        );
        assert_eq!(
            step(IDLE, S::Asking, E::QuitAfterWrite),
            (S::Waiting, A::StopApiAndEnd),
            "ADR 0124 item 20: Quit After Write with no work left must end the app."
        );
    }

    #[test]
    fn each_event_in_the_state_waiting() {
        for work in [IDLE, BUSY] {
            assert_eq!(step(work, S::Waiting, E::Quit), (S::Waiting, A::Continue));
            assert_eq!(step(work, S::Waiting, E::Cancel), (S::Waiting, A::Continue));
            assert_eq!(
                step(work, S::Waiting, E::QuitNow),
                (S::Waiting, A::EndNow),
                "ADR 0124 item 20: Quit Now in the wait must end the app."
            );
            assert_eq!(
                step(work, S::Waiting, E::QuitAfterWrite),
                (S::Waiting, A::Continue)
            );
            assert_eq!(
                step(work, S::Waiting, E::OpenStudio),
                (S::Running, A::Continue),
                "ADR 0124 item 20: Open Studio in the wait must cancel the quit."
            );
            assert_eq!(
                step(work, S::Waiting, E::NoWork),
                (S::Waiting, A::StopApiAndEnd),
                "ADR 0124 item 20: the end of the work must stop the API and end the app."
            );
        }
    }

    #[test]
    fn the_title_counts_the_work() {
        let title = |writes, uploads| dialog_title(WorkCount { writes, uploads });
        assert_eq!(title(1, 0), "Studio Is Writing 1 Feed");
        assert_eq!(title(2, 0), "Studio Is Writing 2 Feeds");
        assert_eq!(title(0, 1), "Studio Is Receiving 1 Music File");
        assert_eq!(title(0, 3), "Studio Is Receiving 3 Music Files");
        assert_eq!(
            title(1, 1),
            "Studio Is Writing 1 Feed And Receiving 1 Music File"
        );
    }

    #[test]
    fn a_button_label_gives_its_event() {
        assert_eq!(event_for_label(QUIT_AFTER_WRITE), E::QuitAfterWrite);
        assert_eq!(event_for_label(QUIT_NOW), E::QuitNow);
        assert_eq!(event_for_label(CANCEL), E::Cancel);
        assert_eq!(event_for_label("something else"), E::Cancel);
    }
}
