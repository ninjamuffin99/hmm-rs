use std::process::id;
use std::str::FromStr;
use std::{fs::File, path::Path};

use crate::hmm::haxelib::{Haxelib, HaxelibType};
use crate::hmm::{self, dependencies::Dependancies};
use anyhow::{anyhow, Context, Result};
use bstr::BStr;
use console::Emoji;
use gix::{Object, ObjectId};
use std::io::Read;
use yansi::Paint;

pub struct HaxelibStatus<'a> {
    pub lib: &'a Haxelib,
    pub install_type: InstallType,
}

// First, define the install type enum
#[derive(Debug, PartialEq)]
pub enum InstallType {
    Missing,          // Needs to be installed
    Outdated,         // Installed but wrong version
    AlreadyInstalled, // Correctly installed
    Conflict,         // Version conflicts between dependencies
}

impl<'a> HaxelibStatus<'a> {
    pub fn new(lib: &'a Haxelib, install_type: InstallType) -> Self {
        Self { lib, install_type }
    }
}

pub fn check() -> Result<()> {
    let deps = hmm::json::read_json("hmm.json")?;

    match compare_haxelib_to_hmm(&deps)? {
        installs => println!(
            "{} / {} dependencie(s) are installed at the correct versions",
            deps.dependencies.len() - installs.len(),
            deps.dependencies.len()
        ),
    }
    Ok(())
}

pub fn compare_haxelib_to_hmm(deps: &Dependancies) -> Result<Vec<HaxelibStatus>> {
    let mut install_status = Vec::new();

    for haxelib in deps.dependencies.iter() {
        // Haxelib folders replace . with , in the folder name

        let comma_replace = haxelib.name.replace(".", ",");
        let haxelib_path = Path::new(".haxelib").join(comma_replace.as_str());

        // assumes an error will occur, and if not, this line will be rewritten at the end of the for loop
        println!("{} {}", haxelib.name.bold().red(), Emoji("❌", "[X]"));
        if !haxelib_path.exists() {
            let err_message = format!("{} not installed", haxelib.name);
            println!("{}", err_message.red());

            install_status.push(HaxelibStatus::new(haxelib, InstallType::Missing));
            continue;
        }

        // Read the .current file
        let current_file = match haxelib_path.join(".dev").exists() {
            true => haxelib_path.join(".dev"),
            false => haxelib_path.join(".current"),
        };
        // println!("Checking version at {}", current_file.display());
        let mut current_version = String::new();
        File::read_to_string(
            &mut File::open(&current_file).context(anyhow!("Could not open {:?}", current_file))?,
            &mut current_version,
        )?;
        // println!("Current version: {}", current_version);

        match haxelib.haxelib_type {
            HaxelibType::Haxelib => {
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

                    install_status.push(HaxelibStatus::new(haxelib, InstallType::Outdated));
                    continue;
                }
            }
            HaxelibType::Git => {
                let repo_path = haxelib_path.join("git");

                let repo = match gix::discover(&repo_path) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("{}", e.to_string().red());
                        println!(
                            "Expected: {} | Installed: {}",
                            haxelib.vcs_ref.as_ref().unwrap().red(),
                            "None".red()
                        );
                        install_status.push(HaxelibStatus::new(haxelib, InstallType::Missing));
                        continue;
                    }
                };

                let head_ref = repo.head_ref()?.unwrap();

                // If our head ref is a tag or branch, we check if we already have it in our history
                // If it's not a tag, we check via commit id
                let intended_commit = match repo.find_reference(haxelib.vcs_ref.as_ref().unwrap()) {
                    Ok(r) => r.id(),
                    Err(_) => {
                        let commit = repo.find_commit(
                            ObjectId::from_str(haxelib.vcs_ref.as_ref().unwrap())
                                .expect("Commit Tag improperly setup?"),
                        );
                        commit.unwrap().id()
                    }
                };

                if head_ref.id() != intended_commit {
                    println!(
                        "{} {}",
                        haxelib.name.red().bold(),
                        "is not at the correct version".red()
                    );

                    println!(
                        "Expected: {} | Installed: {} at {}",
                        haxelib.vcs_ref.as_ref().unwrap().red(),
                        head_ref.name().shorten().red(),
                        head_ref.id().red()
                    );

                    install_status.push(HaxelibStatus::new(haxelib, InstallType::Outdated));
                    continue;
                }

                if repo.is_dirty()? {
                    println!(
                        "{} {}",
                        haxelib.name.red().bold(),
                        "has local changes".red()
                    );
                    install_status.push(HaxelibStatus::new(haxelib, InstallType::Conflict));
                    continue;
                }

                // we have a correct version, so we're going to update the current_version to to the vcs_ref
                current_version = haxelib.vcs_ref.as_ref().unwrap().to_string();
            }
            _ => {}
        }

        let inner = format!(
            "{} [{:?}]: {} {}",
            haxelib.name.green().bold(),
            haxelib.haxelib_type.green().bold(),
            current_version.bright_green(),
            Emoji("✅", "[✔️]")
        );
        print!("\x1B[1A\x1B[2K{}", inner.bright_green().wrap());
        println!();
        install_status.push(HaxelibStatus::new(haxelib, InstallType::AlreadyInstalled));
    }
    Ok(install_status)
}
