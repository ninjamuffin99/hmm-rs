use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    List {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

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

impl Dependancies {
    fn print_string_list(&self) {
        for haxelib in self.dependencies.iter() {
            let version_or_ref = match &haxelib.version {
                Some(v) => format!("version: {}", v),
                None => match &haxelib.vcs_ref {
                    Some(r) => format!("ref: {}", r),
                    None => format!("No version or ref"),
                },
            };

            let mut haxelib_output = format!(
                "{} [{haxelib_type}] \n{} \n",
                haxelib.name,
                version_or_ref,
                haxelib_type = haxelib.haxelib_type
            );

            match haxelib.haxelib_type.as_str() {
                "git" => match &haxelib.url {
                    Some(u) => haxelib_output.push_str(&format!("url: {}\n", u)),
                    None => {}
                },
                "haxelib" => {
                    let haxelib_url = format!("https://lib.haxe.org/p/{}", haxelib.name);
                    haxelib_output.push_str(&format!("url: {}\n", haxelib_url))
                }
                _ => {}
            }

            println!("{}", haxelib_output);
        }
    }
}

fn main() {
    // println!("Hello, world!");

    match_commands();

    // let dep = print_flixel_haxelib().unwrap();
    // println!("{}", dep.to_string());
    // save_json(dep, "samples/flixel.json").unwrap();

    // match read_json("samples/hmm.json") {
    //     Ok(dep_read) => {
    //         println!("Read: {}", dep_read.to_string());
    //     },
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //     }
    // }

    // let dep_read = read_json("samples/hmm.json").unwrap();
    // let k = serde_json::to_string(&dep_read).unwrap();
    // println!("{}", k);
}

fn match_commands() {
    let args = Args::parse();
    match args.cmd {
        Commands::List { path } => {
            let file_to_open = match path {
                Some(dir_path) => dir_path,
                None => "hmm.json".into(),
            };

            match read_json(file_to_open.to_str().unwrap()) {
                Ok(dep_read) => {
                    dep_read.print_string_list();
                }
                _ => {}
            }
        }
    }
}

fn read_json(path: &str) -> std::io::Result<Dependancies> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error: {:?} not found", path);
            return Err(e);
        }
    };
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

    let dep: Dependancies = Dependancies {
        dependencies: vec![haxelib],
    };

    // let j = serde_json::to_string(&dep)?;

    // println!("{}", j);
    Ok(dep)
}
