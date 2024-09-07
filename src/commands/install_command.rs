use crate::hmm;
use crate::hmm::haxelib::Haxelib;
use crate::hmm::haxelib::HaxelibType;
use anyhow::{anyhow, Context, Result};
use bstr::BString;
use console::Emoji;
use futures_util::StreamExt;
use gix::clone;
use gix::create;
use gix::progress::Discard;
use gix::Url;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::io::Write;
use std::num::NonZero;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use yansi::Paint;
use zip::ZipArchive;

use super::check_command::compare_haxelib_to_hmm;

pub fn install_from_hmm() -> Result<()> {
    let deps = hmm::json::read_json("hmm.json")?;

    let installs_needed = compare_haxelib_to_hmm(&deps)?;
    println!(
        "{} dependencies need to be installed",
        installs_needed.len().to_string().bold()
    );

    for haxelib in installs_needed.iter() {
        match &haxelib.haxelib_type {
            HaxelibType::Haxelib => install_from_haxelib(haxelib)?,
            HaxelibType::Git => install_from_git_using_gix(haxelib)?,
            lib_type => println!(
                "{}: Installing from {:?} not yet implemented",
                haxelib.name.red(),
                lib_type
            ),
        }
    }

    Ok(())
}

pub fn install_from_git_using_gix(haxelib: &Haxelib) -> Result<()> {
    println!("Installing {} from git using gix", haxelib.name);

    let path_with_no_https = haxelib.url.as_ref().unwrap().replace("https://", "");

    let clone_url = Url::from_parts(
        gix::url::Scheme::Https,
        None,
        None,
        None,
        None,
        BString::from(path_with_no_https),
        false,
    )
    .context(format!(
        "error creating gix url for {}",
        haxelib.url.as_ref().unwrap()
    ))?;

    let mut clone_path = PathBuf::from(".haxelib");
    clone_path = clone_path.join(&haxelib.name);

    create_current_file(&clone_path, &String::from("git"))?;

    clone_path = clone_path.join("git");

    match std::fs::create_dir_all(&clone_path) {
        Ok(_) => println!("Created directory: {:?}", clone_path.as_path()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                println!("Directory already exists: {:?}", clone_path.as_path());
            } else {
                return Err(anyhow!(
                    "Error creating directory: {:?}",
                    clone_path.as_path()
                ));
            }
        }
    };

    let opts = create::Options {
        destination_must_be_empty: false,
        fs_capabilities: None,
    };

    let mut da_fetch = clone::PrepareFetch::new(
        clone_url,
        clone_path,
        create::Kind::WithWorktree,
        opts,
        gix::open::Options::default(),
    )
    .context("error preparing clone")?;

    da_fetch = da_fetch.with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        NonZero::new(1).unwrap(),
    ));
    let mut da_checkout = da_fetch.fetch_then_checkout(Discard, &AtomicBool::new(false))?;

    da_checkout
        .0
        .main_worktree(Discard, &AtomicBool::new(false))?;

    Ok(())
}

#[tokio::main]
pub async fn install_from_haxelib(haxelib: &Haxelib) -> Result<()> {
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

    match std::fs::create_dir(&output_dir) {
        Ok(_) => println!("Created directory: {:?}", output_dir.as_path()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                println!("Directory already exists: {:?}", output_dir.as_path());
            } else {
                return Err(anyhow!(
                    "Error creating directory: {:?}",
                    output_dir.as_path()
                ));
            }
        }
    }
    // .current file
    println!(
        "Writing .current file to version: {:?}",
        haxelib.version.as_ref().unwrap().as_str()
    );

    create_current_file(&output_dir, haxelib.version.as_ref().unwrap())?;

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

pub fn create_current_file(path: &Path, content: &String) -> Result<()> {
    std::fs::create_dir_all(path)?;
    let mut current_version_file = File::create(path.join(".current"))?;
    write!(current_version_file, "{}", content)?;
    Ok(())
}
