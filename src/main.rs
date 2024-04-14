pub mod hmm;

use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use console::Emoji;
use futures_util::StreamExt;
use hmm::haxelib::{Haxelib, HaxelibType};
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use yansi::Paint;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Lists the dependencies in the hmm.json file (or a file of your choice with --path)
    /// use `hmm-rs check` to see if the dependencies are installed at the correct versions
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
    /// Installs the dependencies from hmm.json, if they aren't already installed.
    Install,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Commands::List { path } => hmm::json::read_json(&path)?.print_string_list()?,
        Commands::Init => {
            create_haxelib_folder()?;
            hmm::json::create_empty_hmm_json()?
        }
        Commands::Clean => remove_haxelib_folder()?,
        Commands::ToHxml => dump_to_hxml()?,
        Commands::Check => match compare_haxelib_to_hmm()? {
            0 => println!("All dependencies are installed at their proper versions"),
            installs => println!(
                "{} dependencie(s) are installed at incorrect versions",
                installs
            ),
        },
        Commands::Install => install_from_hmm()?,
    }
    Ok(())
}

fn install_from_hmm() -> Result<()> {
    let deps = hmm::json::read_json("hmm.json")?;

    for haxelib in deps.dependencies.iter() {
        match &haxelib.haxelib_type {
            HaxelibType::Haxelib => install_from_haxelib(haxelib)?,
            lib_type => println!(
                "{}: Installing from {:?} not yet implemented",
                haxelib.name.red(),
                lib_type
            ),
        }
    }

    Ok(())
}

#[tokio::main]
async fn install_from_haxelib(haxelib: &Haxelib) -> Result<()> {
    // braindead...
    let mut target_url = String::from("https://lib.haxe.org/p/");
    target_url.push_str(haxelib.name.as_str());
    target_url.push_str("/");
    target_url.push_str(haxelib.version.as_ref().unwrap().as_str());
    target_url.push_str("/");
    target_url.push_str("download");

    println!(
        "Downloading: {} - {} - {}",
        haxelib.name.bold(),
        "lib.haxe.org".yellow().bold(),
        target_url.bold()
    );

    let client = reqwest::Client::new();

    let tmp_dir = env::temp_dir().join(format!("{}.zip", haxelib.name));
    println!("Temp directory: {:?}", tmp_dir.bold());

    let response = client.get(target_url).send().await?;
    let total_size = response.content_length().unwrap();
    println!("Size: {}", human_bytes(total_size as f64));
    // yoinked from haxeget !
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.yellow/red}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
             .unwrap());

    let mut file = File::create(tmp_dir.as_path())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        file.write_all(&chunk).unwrap();
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    let finish_message = format!(
        "{}: {} done downloading from {}",
        haxelib.name.green().bold(),
        haxelib.version.as_ref().unwrap().bright_green(),
        "Haxelib".yellow().bold()
    );
    pb.finish_with_message(finish_message);

    let version_as_commas = haxelib.version.as_ref().unwrap().replace(".", ",");
    let mut output_dir: PathBuf = [".haxelib", haxelib.name.as_str()].iter().collect();

    // .current file
    println!(
        "Writing .current file to version: {:?}",
        haxelib.version.as_ref().unwrap().as_str()
    );

    let mut current_version_file = File::create(&output_dir.join(".current"))?;
    write!(
        current_version_file,
        "{}",
        haxelib.version.as_ref().unwrap()
    )?;

    // unzipping
    output_dir = output_dir.join(version_as_commas.as_str());
    println!("Unzipping to: {:?}", output_dir.as_path());

    let archive = File::open(tmp_dir.as_path())?;
    let mut zip_file = ZipArchive::new(archive).context("Error opening zip file")?;
    zip_file
        .extract(output_dir.as_path())
        .context("Error extracting zip file")?;

    // removing the zip file
    print!("Deleting temp file: {:?}", tmp_dir.as_path());
    std::fs::remove_file(tmp_dir.as_path())?;
    println!("");
    println!(
        "{}: {} installed {}",
        haxelib.name.green().bold(),
        haxelib.version.as_ref().unwrap().bright_green(),
        Emoji("✅", "[✔️]")
    );
    // print an empty line, for readability between downloads
    println!("");
    Ok(())
}

fn compare_haxelib_to_hmm() -> Result<u32> {
    let deps = hmm::json::read_json("hmm.json")?;

    let mut incorrect_installs = 0;

    for haxelib in deps.dependencies.iter() {
        // Haxelib folders replace . with , in the folder name

        let comma_replace = haxelib.name.replace(".", ",");
        let haxelib_path = Path::new(".haxelib").join(comma_replace.as_str());

        // assumes an error will occur, and if not, this line will be rewritten at the end of the for loop
        println!("{} {}", haxelib.name.bold().red(), Emoji("❌", "[X]"));
        if !haxelib_path.exists() {
            println!("{} not installed", haxelib.name.bold().red());
            incorrect_installs += 1;
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

                    incorrect_installs += 1;
                    continue;
                }
            }
            HaxelibType::Git => {
                let repo_path = haxelib_path.join("git");
                let repo = gix::discover(&repo_path)
                    .context(anyhow!("Could not find git repo {:?}", repo_path))?;
                let head_ref = repo.head_id()?;
                let head_ref_string = head_ref.to_string();

                current_version = head_ref_string.clone();

                let branch_name = match repo.head_name()? {
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

                    incorrect_installs += 1;
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

fn dump_to_hxml() -> Result<()> {
    let deps = hmm::json::read_json("hmm.json")?;
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

fn create_haxelib_folder() -> Result<()> {
    let haxelib_path = Path::new(".haxelib");
    if haxelib_path.exists() {
        let err_message = format!(
            "{} \n{}",
            "A .haxelib folder already exists in this directory, so it won't be created.",
            "use `hmm-rs clean` to remove the folder"
        );
        Err(anyhow!(err_message))?
    }
    println!("Creating .haxelib/ folder");
    std::fs::create_dir(haxelib_path).context("Failed to create .haxelib folder")
}

fn remove_haxelib_folder() -> Result<()> {
    let haxelib_path = Path::new(".haxelib");
    if !haxelib_path.exists() {
        Err(anyhow!(
            "A .haxelib folder does not exist in this directory, so it cannot be removed."
                .bright_red()
                .bold()
        ))?
    }
    println!("Removing .haxelib/ folder");
    std::fs::remove_dir_all(haxelib_path).context("Failed to remove .haxelib folder")
}
