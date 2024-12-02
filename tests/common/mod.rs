use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn setup() {
    // some setup code, like creating required files/directories, starting
    // servers, etc.
    let crate_dir = PathBuf::new().join(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = crate_dir.join("tests");
    let samples_dir = tests_dir.join("samples");
}

pub fn get_samples_dir() -> String {
    let crate_dir = PathBuf::new().join(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = crate_dir.join("tests");
    tests_dir.join("samples").to_string_lossy().to_string()
}
