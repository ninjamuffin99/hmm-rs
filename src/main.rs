use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_json::json;

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
    print_flixel_haxelib().unwrap();
}

fn print_flixel_haxelib() -> Result<()> {
    
    let haxelib = Haxelib {
        name: String::from("flixel"),
        haxelib_type: String::from("git"),
        vcs_ref: String::from("master"),
        dir: String::from(""),
        url: String::from("https://github.com/haxeflixel/flixel"),
        version: String::from(""),
    };

    let j = serde_json::to_string(&haxelib)?;

    println!("{}", j);
    Ok(())
}