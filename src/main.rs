use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct Dependancies {
    dependencies: Vec<Haxelib>,
}

#[derive(Serialize, Deserialize)]
struct Haxelib {
    name: String,
    #[serde(rename = "type")]
    haxelib_type: String,
    #[serde(rename = "ref")]
    vcs_ref: String,
    dir: String,
    url: String,
    version: String,
}

fn main() {
    
    println!("Hello, world!");

    let dep = print_flixel_haxelib().unwrap();
    let j = serde_json::to_string(&dep).unwrap();
    println!("{}", j);
    save_json(dep, "samples/flixel.json").unwrap();
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
        vcs_ref: String::from("master"),
        dir: String::from(""),
        url: String::from("https://github.com/haxeflixel/flixel"),
        version: String::from(""),
    };

    let dep:Dependancies = Dependancies {
        dependencies: vec![haxelib],
    };

    // let j = serde_json::to_string(&dep)?;

    // println!("{}", j);
    Ok(dep)
}