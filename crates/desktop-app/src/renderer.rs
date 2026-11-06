//! The `WebKitGTK` renderer of the app on Linux (ADR 0124 item 28).
//!
//! With an NVIDIA graphics card, the DMA-BUF renderer of `WebKitGTK` can fail
//! to import each frame, and the page loads very slowly. `WebKitGTK` reads its
//! setting from the environment. The workspace forbids `unsafe` code, and a
//! change of the environment of a running process is `unsafe`. The app thus
//! starts itself again one time, with the variable set.

use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

const VARIABLE: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
const DRM_FOLDER: &str = "/sys/class/drm";
const NVIDIA_VENDOR: &str = "0x10de";

/// True when the app must start again with the variable. A value from the
/// user always stays.
fn must_restart(variable: Option<&OsString>, vendors: &[String]) -> bool {
    variable.is_none() && vendors.iter().any(|v| v.trim() == NVIDIA_VENDOR)
}

/// The vendor of each graphics card in `folder`. A connector such as
/// `card0-HDMI-A-1` is not a card.
fn card_vendors(folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("card") && !name.contains('-')
        })
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("device/vendor")).ok())
        .collect()
}

/// Start the app again with the variable when this computer needs it. The
/// call returns only when no restart is necessary or the restart fails. A
/// failure leaves the app with the default renderer.
pub fn restart_if_necessary() {
    let variable = std::env::var_os(VARIABLE);
    if !must_restart(variable.as_ref(), &card_vendors(Path::new(DRM_FOLDER))) {
        return;
    }
    let Ok(program) = std::env::current_exe() else {
        return;
    };
    let error = Command::new(program)
        .args(std::env::args_os().skip(1))
        .env(VARIABLE, "1")
        .exec();
    eprintln!("studio-desktop-app: could not start again with {VARIABLE}: {error}");
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "ADR 0124 item 28: only an NVIDIA card with no value from the user restarts the app. Check must_restart in renderer.rs.";

    #[test]
    fn only_an_nvidia_card_with_no_value_of_the_user_restarts_the_app() {
        let nvidia = vec!["0x10de\n".to_string()];
        let intel = vec!["0x8086\n".to_string()];
        let hybrid = vec!["0x8086\n".to_string(), "0x10de\n".to_string()];
        let set = OsString::from("0");
        assert!(must_restart(None, &nvidia), "{OWNER}");
        assert!(must_restart(None, &hybrid), "{OWNER}");
        assert!(!must_restart(None, &intel), "{OWNER}");
        assert!(!must_restart(None, &[]), "{OWNER}");
        assert!(!must_restart(Some(&set), &nvidia), "{OWNER}");
    }

    #[test]
    fn a_connector_is_not_a_graphics_card() {
        let folder = tempfile::tempdir().expect("the test needs a temporary folder");
        for (name, vendor) in [("card0", "0x10de\n"), ("card0-HDMI-A-1", "0x8086\n")] {
            let device = folder.path().join(name).join("device");
            std::fs::create_dir_all(&device).expect("the test needs a device folder");
            std::fs::write(device.join("vendor"), vendor).expect("the test needs a vendor file");
        }
        assert_eq!(card_vendors(folder.path()), vec!["0x10de\n".to_string()]);
        assert_eq!(
            card_vendors(&folder.path().join("missing")),
            Vec::<String>::new()
        );
    }
}
