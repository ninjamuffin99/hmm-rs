use std::io::Write;
use std::str::FromStr;
use std::{fs::File, path::PathBuf};

use super::dependencies::Dependancies;
use anyhow::{Context, Result};

pub fn save_json(deps: Dependancies, path: PathBuf) -> Result<()> {
    println!("{} saved/updated", path.display());
    let j = serde_json::to_string_pretty(&deps)?;
    let mut file = File::create(path)?;
    file.write_all(j.as_bytes())?;
    Ok(())
}

pub fn create_empty_hmm_json() -> Result<()> {
    let empty_deps = Dependancies {
        dependencies: vec![],
    };

    save_json(empty_deps, PathBuf::from_str("hmm.json")?)
}

// Read the JSON, and return the Dependancies struct
pub fn read_json(path: &PathBuf) -> Result<Dependancies> {
    let file = File::open(path).context(format!("JSON {:?} not found", path))?;
    let deps: Dependancies = serde_json::from_reader(file)?;
    Ok(deps)
}
