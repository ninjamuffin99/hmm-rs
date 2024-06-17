pub mod commands;
pub mod hmm;

use anyhow::{Ok, Result};

use clap::{Parser, Subcommand};

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
        #[arg(short, long)]
        lib: Option<String>,
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
    /// Installs a haxelib from lib.haxe.org
    Haxelib {
        /// The name of the haxelib to install
        name: String,
        /// The version of the haxelib to install
        version: Option<String>,
    },
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Commands::List { path, lib } => hmm::json::read_json(&path)?.print_string_list(&lib)?,
        Commands::Init => commands::init_command::init_hmm()?,
        Commands::Clean => commands::clean_command::remove_haxelib_folder()?,
        Commands::ToHxml => commands::tohxml_command::dump_to_hxml()?,
        Commands::Check => commands::check_command::check()?,
        Commands::Install => commands::install_command::install_from_hmm()?,
        Commands::Haxelib { name, version } => {
            commands::haxelib_command::install_haxelib(&name, &version)?
        }
    }
    Ok(())
}
