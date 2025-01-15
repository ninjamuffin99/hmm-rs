pub mod commands;
pub mod hmm;

use std::path::PathBuf;

use anyhow::{Ok, Result};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,

    /// Sets a custom hmm.json file to use
    #[arg(short, long, value_name = "JSON", default_value = "hmm.json")]
    json: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Lists the dependencies in the hmm.json file (or a file of your choice with --path)
    /// use `hmm-rs check` to see if the dependencies are installed at the correct versions
    List {
        /// Specific libraries you want to list, can be multiple
        /// `hmm-rs list lime openfl` will list lime and openfl
        #[arg(value_name = "LIBS")]
        lib: Option<Vec<String>>,
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
    /// Removes one or more library dependencies from `hmm.json` and the `.haxelib/` folder
    Remove {
        /// The library(s) you wish to remove
        #[arg(short, long)]
        lib: Vec<String>,
    },
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    let path = args.json.unwrap();

    let deps = hmm::json::read_json(&path)?;

    match args.cmd {
        Commands::List { lib } => hmm::json::read_json(&path)?.print_string_list(&lib)?,
        Commands::Init => commands::init_command::init_hmm()?,
        Commands::Clean => commands::clean_command::remove_haxelib_folder()?,
        Commands::ToHxml => commands::tohxml_command::dump_to_hxml(&deps)?,
        Commands::Check => commands::check_command::check(&deps)?,
        Commands::Install => commands::install_command::install_from_hmm(&deps)?,
        Commands::Haxelib { name, version } => {
            commands::haxelib_command::install_haxelib(&name, &version, deps, path)?
        }
        Commands::Remove { lib: _ } => commands::remove_command::remove_haxelibs()?,
    }
    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert();
}
