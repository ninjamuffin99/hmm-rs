use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::File;
use std::io::prelude::*;
use std::fmt;

#[derive(Serialize, Deserialize)]
struct Dependancies {
    dependencies: Vec<Haxelib>,
}

#[derive(Serialize, Deserialize)]
struct Haxelib {
    name: String,
    #[serde(rename = "type")]
    haxelib_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ref")]
    vcs_ref: Option<String>,
    dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl fmt::Display for Dependancies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

fn main() {
    
    // println!("Hello, world!");

    let dep = print_flixel_haxelib().unwrap();
    println!("{}", dep.to_string());
    save_json(dep, "samples/flixel.json").unwrap();
    
    match read_json("samples/hmm.json") {
        Ok(dep_read) => {
            println!("Read: {}", dep_read.to_string());
        }, 
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    // let dep_read = read_json("samples/hmm.json").unwrap();
    // let k = serde_json::to_string(&dep_read).unwrap();
    // println!("{}", k);
}

fn read_json(path: &str) -> std::io::Result<Dependancies> {
    let file = File::open(path)?;
    let deps: Dependancies = serde_json::from_reader(file)?;
    Ok(deps)
}

fn save_json(deps: Dependancies, path: &str) -> std::io::Result<()> {
    println!("Saving to {}", path);
    let j = serde_json::to_string_pretty(&deps)?;
    let mut file = File::create(path)?;
    file.write_all(j.as_bytes())?;
    Ok(())
}

fn print_flixel_haxelib() -> Result<Dependancies> {
    
    let haxelib = Haxelib {
        name: String::from("flixel"),
        haxelib_type: String::from("git"),
        vcs_ref: Option::Some(String::from("master")),
        dir: Option::None,
        url: Option::Some(String::from("https://github.com/haxeflixel/flixel")),
        version: Option::None,
    };

    let dep:Dependancies = Dependancies {
        dependencies: vec![haxelib],
    };

    // let j = serde_json::to_string(&dep)?;

    // println!("{}", j);
    Ok(dep)
}