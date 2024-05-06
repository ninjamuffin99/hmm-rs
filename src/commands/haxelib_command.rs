use anyhow::Result;

pub fn install_haxelib(name: &str, version: &Option<String>) -> Result<()> {
    println!("Installing haxelib: {} {:?}", name, version);
    Ok(())
}
