use std::path::{Path, PathBuf};

use hmm_rs::{commands::*, hmm};
mod common;

#[test]
fn test_create_and_clean_haxelib_folder() {
    assert!(clean_command::remove_haxelib_folder().is_err());
    assert!(init_command::create_haxelib_folder().is_ok());
    assert!(init_command::create_haxelib_folder().is_err());
    assert!(clean_command::remove_haxelib_folder().is_ok());
}

#[test]
fn test_hmm_json_read() {
    // common::setup();
    let flixel_json = format!("{}/flixel.json", common::get_samples_dir());
    assert!(hmm::json::read_json(flixel_json.as_str()).is_ok());
}
