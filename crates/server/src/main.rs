//! Server shell of musicindex-studio.
//!
//! A host operator runs this binary to give a public service to web users
//! (ADR 0001). Sign-in and the per-user vault come in the Server
//! Shell phase.

fn main() {
    println!("studio-server {}", env!("CARGO_PKG_VERSION"));
}
