//! musicindex ADR 0083 R-VERSION-9 item 1: `--version` prints the build
//! values in the line of R-VERSION-2.

use std::process::Command;

#[test]
fn the_version_line_gives_the_build_values() {
    let output = Command::new(env!("CARGO_BIN_EXE_studio-server"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    let line = String::from_utf8(output.stdout).unwrap();
    let prefix = concat!(
        "Music Index Studio server ",
        env!("CARGO_PKG_VERSION"),
        " ("
    );
    let values = line
        .trim_end()
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unexpected line {line:?}"));
    // The build script always knows the build time. The test runs in a Git
    // checkout, so the line also has the git revision.
    let (revision, date) = values
        .split_once(", ")
        .unwrap_or_else(|| panic!("no git revision in {line:?}"));
    let hash = revision.strip_suffix("-dirty").unwrap_or(revision);
    assert!(
        hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit()),
        "{line:?}"
    );
    assert_eq!(date.len(), 10, "{line:?}");
    assert_eq!(date.as_bytes()[4], b'-', "{line:?}");
}
