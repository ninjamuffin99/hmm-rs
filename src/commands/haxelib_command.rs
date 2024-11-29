use anyhow::Result;

use crate::{
    commands,
    hmm::haxelib::{Haxelib, HaxelibType},
};

pub fn install_haxelib(name: &str, version: &Option<String>) -> Result<()> {
    println!("Installing haxelib: {} {:?}", name, version);
    let haxelib_install = Haxelib {
        name: name.to_string(),
        haxelib_type: HaxelibType::Haxelib,
        vcs_ref: version.clone(),
        dir: None,
        url: None,
        version: version.clone(),
    };
    commands::install_command::install_from_haxelib(&haxelib_install)?;
    Ok(())
}
