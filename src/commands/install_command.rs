use crate::commands::check_command::InstallType;
use crate::hmm;
use crate::hmm::haxelib::Haxelib;
use crate::hmm::haxelib::HaxelibType;
use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use bstr::BStr;
use bstr::BString;
use console::Emoji;
use futures_util::StreamExt;
use gix::clone;
use gix::create;
use gix::progress::Discard;
use gix::ObjectId;
use gix::Url;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::io::Write;
use std::num::NonZero;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use yansi::Paint;
use zip::ZipArchive;

use super::check_command::compare_haxelib_to_hmm;
use super::check_command::HaxelibStatus;

pub fn install_from_hmm() -> Result<()> {
    let deps = hmm::json::read_json("hmm.json")?;

    let installs_needed = compare_haxelib_to_hmm(&deps)?;
    println!(
        "{} dependencies need to be installed",
        installs_needed.len().to_string().bold()
    );

    for install_status in installs_needed.iter() {
        match &install_status.install_type {
            InstallType::Missing => handle_install(install_status)?,
            InstallType::Outdated => match &install_status.lib.haxelib_type {
                HaxelibType::Haxelib => install_from_haxelib(install_status.lib)?,
                HaxelibType::Git => install_from_git_using_gix_checkout(install_status.lib)?,
                lib_type => println!(
                    "{}: Installing from {:?} not yet implemented",
                    install_status.lib.name.red(),
                    lib_type
                ),
            },
            InstallType::AlreadyInstalled => (), // do nothing on things already installed at the right version
            _ => println!(
                "{} {:?}: Not implemented",
                install_status.lib.name, install_status.install_type
            ),
        }
    }

    Ok(())
}

pub fn handle_install(haxelib_status: &HaxelibStatus) -> Result<()> {
    match &haxelib_status.lib.haxelib_type {
        HaxelibType::Haxelib => install_from_haxelib(haxelib_status.lib)?,
        HaxelibType::Git => install_from_git_using_gix_clone(haxelib_status.lib)?,
        lib_type => println!(
            "{}: Installing from {:?} not yet implemented",
            haxelib_status.lib.name.red(),
            lib_type
        ),
    }

    Ok(())
}

pub fn install_from_git_using_gix_clone(haxelib: &Haxelib) -> Result<()> {
    println!("Installing {} from git using clone", haxelib.name);

    let haxelib_url = haxelib
        .url
        .as_ref()
        .ok_or(anyhow!("No url provided for {}", haxelib.name))?;

    let path_with_no_https = haxelib_url.replace("https://", "");

    let clone_url = Url::from_parts(
        gix::url::Scheme::Https,
        None,
        None,
        None,
        None,
        BString::from(path_with_no_https),
        false,
    )
    .context(format!("error creating gix url for {}", haxelib_url))?;

    let mut clone_path = PathBuf::from(".haxelib");
    clone_path = clone_path.join(&haxelib.name);

    create_current_file(&clone_path, &String::from("git"))?;

    clone_path = clone_path.join("git");

    match std::fs::create_dir_all(&clone_path) {
        core::result::Result::Ok(_) => println!("Created directory: {:?}", clone_path.as_path()),
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

    let mut da_fetch = clone::PrepareFetch::new(
        clone_url,
        clone_path,
        create::Kind::WithWorktree,
        create::Options {
            destination_must_be_empty: false,
            fs_capabilities: None,
        },
        gix::open::Options::default(),
    )
    .context("error preparing clone")?;

    let mut da_checkout = da_fetch
        .fetch_then_checkout(Discard, &AtomicBool::new(false))?
        .0;

    let repo = da_checkout
        .main_worktree(Discard, &AtomicBool::new(false))
        .expect("Error checking out worktree")
        .0;

    match haxelib.vcs_ref.as_ref() {
        Some(target_ref) => {
            let reflog_msg = BString::from("derp?");

            let target_object = ObjectId::from_str(target_ref)
                .context(format!("error converting {} to ObjectId", target_ref))?;

            repo.head_ref()
                .unwrap()
                .unwrap()
                .set_target_id(target_object, reflog_msg)?;
        }
        None => (),
    }

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
        core::result::Result::Ok(_) => println!("Created directory: {:?}", output_dir.as_path()),
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

pub fn install_from_git_using_gix_checkout(haxelib: &Haxelib) -> Result<()> {
    Ok(())
}

pub fn create_current_file(path: &Path, content: &String) -> Result<()> {
    std::fs::create_dir_all(path)?;
    let mut current_version_file = File::create(path.join(".current"))?;
    write!(current_version_file, "{}", content)?;
    Ok(())
}
