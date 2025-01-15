use crate::hmm::dependencies::Dependancies;
use crate::hmm::haxelib::HaxelibType;
use anyhow::Result;

pub fn dump_to_hxml(deps: &Dependancies) -> Result<()> {
    let mut hxml = String::new();
    for haxelib in deps.dependencies.iter() {
        let mut lib_string = String::from("-lib ");
        lib_string.push_str(haxelib.name.as_str());

        match haxelib.haxelib_type {
            HaxelibType::Git => {
                lib_string
                    .push_str(format!(":git:{}", &haxelib.url.as_ref().unwrap().as_str()).as_str());
                match &haxelib.vcs_ref {
                    Some(r) => lib_string.push_str(format!("#{}", r).as_str()),
                    _ => {}
                }
            }
            HaxelibType::Haxelib => lib_string
                .push_str(format!(":{}", haxelib.version.as_ref().unwrap().as_str()).as_str()),
            _ => {}
        }
        hxml.push_str(&lib_string);
        hxml.push_str("\n");
    }
    println!("{}", hxml);
    Ok(())
}
