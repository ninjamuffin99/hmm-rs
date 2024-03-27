use clap::{Parser, Subcommand};
use console::Emoji;
// use gix::prelude::*;
// use gix::Repository;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use yansi::Paint;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Lists the dependencies in the hmm.json file (or a file of your choice with --path)
    List {
        #[arg(short, long)]
        #[arg(default_value_t = String::from("hmm.json"))]
        path: String,
    },
    /// Creates an empty .haxelib/ folder, and an empty hmm.json file
    Init,
    /// Removes local .haxelib directory, useful for full clean reinstalls
    Clean,
    /// dumps the dependencies in hmm.json to a hxml file
    ToHxml,
    /// Checks if the dependencies are installed at their correct hmm.json versions
    Check,
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
            let file_to_open = path;

            match read_json(file_to_open.as_str()) {
                Ok(dep_read) => {
                    dep_read.print_string_list();
                }
                _ => {}
            }
        }
        Commands::Init => {
            create_haxelib_folder();
            create_empty_hmm_json();
        }
        Commands::Clean => remove_haxelib_folder(),
        Commands::ToHxml => dump_to_hxml(),
        Commands::Check => compare_haxelib_to_hmm(),
        _ => {
            println!("Command not implemented yet.")
        }
    }
}

fn compare_haxelib_to_hmm() {
    let deps = read_json("hmm.json").unwrap();

    for haxelib in deps.dependencies.iter() {
        // Haxelib folders replace . with , in the folder name

        let comma_replace = haxelib.name.replace(".", ",");
        let haxelib_path = Path::new(".haxelib").join(comma_replace.as_str());

        // assumes an error will occur, and if not, this line will be rewritten at the end of the for loop
        println!("{} {}", haxelib.name.bold().red(), Emoji("❌", "[X]"));
        if !haxelib_path.exists() {
            println!("{} not installed", haxelib.name.bold().red());
            continue;
        }

        // Read the .current file
        let current_file = match haxelib_path.join(".dev").exists() {
            true => haxelib_path.join(".dev"),
            false => haxelib_path.join(".current"),
        };
        // println!("Checking version at {}", current_file.display());
        let mut current_version = String::new();
        File::read_to_string(&mut File::open(current_file).unwrap(), &mut current_version).unwrap();
        // println!("Current version: {}", current_version);

        match haxelib.haxelib_type.as_str() {
            "haxelib" => {
                if haxelib.version.as_ref().unwrap() != &current_version {
                    println!(
                        "{} {}",
                        haxelib.name.red().bold(),
                        "is not at the correct version".red()
                    );
                    println!(
                        "Expected: {} | Installed: {}",
                        haxelib.version.as_ref().unwrap().red(),
                        current_version.red()
                    );
                    continue;
                }
            }
            "git" => {
                let repo_path = haxelib_path.join("git");
                let repo = gix::discover(repo_path).unwrap();
                let head_ref = repo.head_id().unwrap();
                let head_ref_string = head_ref.to_string();

                current_version = head_ref_string.clone();

                let branch_name = match repo.head_name().unwrap() {
                    Some(h) => {
                        current_version = h.shorten().to_string();
                        h.shorten().to_string()
                    }
                    None => String::new(),
                };

                if haxelib.vcs_ref.as_ref().unwrap() != &head_ref_string
                    && haxelib.vcs_ref.as_ref().unwrap() != &branch_name
                {
                    println!(
                        "{} {}",
                        haxelib.name.red().bold(),
                        "is not at the correct version".red()
                    );

                    let mut output = String::new();

                    // if branch_name is empty, then it's a detached head/specific commit
                    match branch_name.as_str() {
                        "" => {
                            output.push_str(&head_ref_string);
                        }
                        _ => {
                            output.push_str(&branch_name);
                        }
                    }

                    println!(
                        "Expected: {} | Installed: {}",
                        haxelib.vcs_ref.as_ref().unwrap().red(),
                        output.red()
                    );
                    continue;
                }
            }
            _ => {}
        }

        let inner = format!(
            "{} [{}]: {} {}",
            haxelib.name.green().bold(),
            haxelib.haxelib_type.green().bold(),
            current_version.bright_green(),
            Emoji("✅", "[✔️]")
        );
        print!("\x1B[1A\x1B[2K{}", inner.bright_green().wrap());
        println!();
    }
}

fn dump_to_hxml() {
    let deps = read_json("hmm.json").unwrap();
    let mut hxml = String::new();
    for haxelib in deps.dependencies.iter() {
        let mut lib_string = String::from("-lib ");
        lib_string.push_str(haxelib.name.as_str());

        match haxelib.haxelib_type.as_str() {
            "git" => {
                lib_string
                    .push_str(format!(":git:{}", &haxelib.url.as_ref().unwrap().as_str()).as_str());
                match &haxelib.vcs_ref {
                    Some(r) => lib_string.push_str(format!("#{}", r).as_str()),
                    _ => {}
                }
            }
            "haxelib" => lib_string
                .push_str(format!(":{}", haxelib.version.as_ref().unwrap().as_str()).as_str()),
            _ => {}
        }
        hxml.push_str(&lib_string);
        hxml.push_str("\n");
    }
    println!("{}", hxml);
}

fn create_haxelib_folder() {
    let haxelib_path = Path::new(".haxelib");
    if haxelib_path.exists() {
        println!("Folder .haxelib already exists");
        return;
    }
    println!("Creating .haxelib/ folder");
    std::fs::create_dir(haxelib_path).unwrap();
}

fn remove_haxelib_folder() {
    let haxelib_path = Path::new(".haxelib");
    if !haxelib_path.exists() {
        println!("Folder .haxelib does not exist");
        return;
    }
    println!("Removing .haxelib/ folder");
    std::fs::remove_dir_all(haxelib_path).unwrap();
}

fn create_empty_hmm_json() {
    let empty_deps = Dependancies {
        dependencies: vec![],
    };

    save_json(empty_deps, "hmm.json").unwrap();
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
