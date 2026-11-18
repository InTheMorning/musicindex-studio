//! Build script of the Tauri app. It also gives the build values of
//! musicindex ADR 0083 R-VERSION-8.

fn main() {
    musicindex_build_info::build::emit("STUDIO");
    tauri_build::build();
}
