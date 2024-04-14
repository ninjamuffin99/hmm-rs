use super::haxelib::{Haxelib, HaxelibType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct Dependancies {
    pub dependencies: Vec<Haxelib>,
}

impl fmt::Display for Dependancies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl Dependancies {
    pub fn print_string_list(&self) -> Result<()> {
        for haxelib in self.dependencies.iter() {
            let version_or_ref = match &haxelib.version {
                Some(v) => format!("version: {}", v),
                None => match &haxelib.vcs_ref {
                    Some(r) => format!("ref: {}", r),
                    None => format!("No version or ref"),
                },
            };

            let mut haxelib_output = format!(
                "{} [{haxelib_type:?}] \n{} \n",
                haxelib.name,
                version_or_ref,
                haxelib_type = haxelib.haxelib_type
            );

            match haxelib.haxelib_type {
                HaxelibType::Git => match &haxelib.url {
                    Some(u) => haxelib_output.push_str(&format!("url: {}\n", u)),
                    None => {}
                },
                HaxelibType::Haxelib => {
                    let haxelib_url = format!("https://lib.haxe.org/p/{}", haxelib.name);
                    haxelib_output.push_str(&format!("url: {}\n", haxelib_url))
                }
                _ => {}
            }

            println!("{}", haxelib_output);
        }
        Ok(())
    }
}
