pub mod commands;
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
        Commands::Init => commands::init_command::init_hmm()?,
        Commands::Clean => commands::clean_command::remove_haxelib_folder()?,
        Commands::ToHxml => commands::tohxml_command::dump_to_hxml()?,
        Commands::Check => commands::check_command::check()?,
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
