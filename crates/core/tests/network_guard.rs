//! A guard for the network policy (ADR 0057).

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            if name != "testing" {
                rust_files(&path, found);
            }
        } else if Path::new(&name).extension().is_some_and(|e| e == "rs")
            && name != "tests.rs"
            && name != "web.rs"
        {
            found.push(path);
        }
    }
}

#[test]
fn production_code_does_not_call_web_new() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            rust_files(&src, &mut files);
        }
    }
    assert!(files.len() > 10, "the guard found too few source files");
    let mut found = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let production = text
            .split_once("\n#[cfg(test)]\nmod ")
            .map_or(text.as_str(), |(before, _)| before);
        for (number, line) in production.lines().enumerate() {
            if line.contains("Web::new()") {
                found.push(format!("{}:{}", file.display(), number + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "ADR 0057 item 1: production code must not call `Web::new()`, because it has no network \
         policy. Pass the `Web` of the workspace, or call `Web::with_policy` with an explicit \
         policy. Found: {found:?}"
    );
}
