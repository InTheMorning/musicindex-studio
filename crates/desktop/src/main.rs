//! Desktop shell of musicindex-studio.
//!
//! The artist runs this binary on their own machine (ADR 0001). The Tauri
//! window comes in Phase 10.

fn main() {
    println!("studio-desktop {}", env!("CARGO_PKG_VERSION"));
}
