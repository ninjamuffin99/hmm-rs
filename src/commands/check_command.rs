use std::path::PathBuf;
use std::{fs::File, path::Path};

use crate::hmm::haxelib::{self, Haxelib, HaxelibType};
use crate::hmm::{self, dependencies::Dependancies};
use anyhow::{anyhow, Context, Result};
use console::Emoji;
use std::io::Read;
use yansi::Paint;

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

pub fn compare_haxelib_to_hmm(deps: &Dependancies) -> Result<Vec<&Haxelib>> {
    let mut incorrect_installs = Vec::new();

    for haxelib in deps.dependencies.iter() {
        // Haxelib folders replace . with , in the folder name

        let comma_replace = haxelib.name.replace(".", ",");
        let haxelib_path = Path::new(".haxelib").join(comma_replace.as_str());

        // assumes an error will occur, and if not, this line will be rewritten at the end of the for loop
        println!("{} {}", haxelib.name.bold().red(), Emoji("❌", "[X]"));
        if !haxelib_path.exists() {
            let err_message = format!("{} not installed", haxelib.name);
            println!("{}", err_message.red());
            incorrect_installs.push(haxelib);
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

                    incorrect_installs.push(haxelib);
                    continue;
                }
            }
            HaxelibType::Git => {
                let repo_path = haxelib_path.join("git");
                let repo = gix::discover(&repo_path)
                    .context(anyhow!("Could not find git repo {:?}", repo_path))?;
                let head_ref = match repo.head_id() {
                    Ok(h) => h,
                    Err(e) => {
                        println!(
                            "{} {}",
                            haxelib.name.red().bold(),
                            "is not at the correct version".red()
                        );
                        println!("Error: {}", e);
                        println!(
                            "Expected: {} | Installed: {}",
                            haxelib.vcs_ref.as_ref().unwrap().red(),
                            "unknown".red()
                        );
                        incorrect_installs.push(haxelib);
                        continue;
                    }
                };
                let head_ref_string = head_ref.to_string();

                current_version = head_ref_string.clone();

                let branch_name = match repo.head_name()? {
                    Some(h) => {
                        current_version = h.shorten().to_string();
                        h.shorten().to_string()
                    }
                    None => String::new(),
                };

                let branch_commit = repo.find_reference(branch_name.as_str())?.id().to_string();

                if haxelib.vcs_ref.as_ref().unwrap() != &head_ref_string
                    && (haxelib.vcs_ref.as_ref().unwrap() != &branch_name
                        && haxelib.vcs_ref.as_ref().unwrap() != &branch_commit)
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
                        "Expected: {} | Installed: {} at {}",
                        haxelib.vcs_ref.as_ref().unwrap().red(),
                        output.red(),
                        branch_commit.red()
                    );

                    incorrect_installs.push(haxelib);
                    continue;
                }

                if repo.is_dirty()? {
                    println!(
                        "{} {}",
                        haxelib.name.red().bold(),
                        "has local changes".red()
                    );
                    incorrect_installs.push(haxelib);
                    continue;
                }
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
    }
    Ok(incorrect_installs)
}
