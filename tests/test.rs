use hmm_rs::commands::*;

#[test]
fn test_create_and_clean_haxelib_folder() {
    assert!(clean_command::remove_haxelib_folder().is_err());
    assert!(init_command::create_haxelib_folder().is_ok());
    assert!(init_command::create_haxelib_folder().is_err());
    assert!(clean_command::remove_haxelib_folder().is_ok());
}
