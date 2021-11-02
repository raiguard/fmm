#![allow(unused)]

use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "fmm")]
struct App {
    dir: PathBuf,
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    // Step 1: Get all mods in the directory
    // let mut directory_mods: HashMap<String, Vec<Version>> = HashMap::new();
    let directory_mods = std::fs::read_dir(&app.dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();

            // TODO: Folders can be versionless, in which case we have to parse their info.json
            let (mod_name, version) = file_name.to_str()?.rsplit_once("_")?;
            let (version, _) = version.rsplit_once(".").unwrap_or((version, "")); // Strip file extension

            Some((mod_name.to_string(), Version::parse(version).ok()?))
        })
        .fold(HashMap::new(), |mut directory_mods, (mod_name, version)| {
            let versions = directory_mods.entry(mod_name).or_insert_with(Vec::new);

            let index = versions
                .binary_search(&version)
                .unwrap_or_else(|index| index);
            versions.insert(index, version);

            directory_mods
        });

    // Step 2: Parse mod-list.json
    let mut mlj_path = app.dir;
    mlj_path.push("mod-list.json");
    let enabled_versions = std::fs::read_to_string(&mlj_path)?;
    let enabled_versions: ModListJson = serde_json::from_str(&enabled_versions)?;

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct ModListJson {
    mods: Vec<ModListJsonMod>,
}

#[derive(Deserialize, Serialize)]
struct ModListJsonMod {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
    enabled: bool,
}
