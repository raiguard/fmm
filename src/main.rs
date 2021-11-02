#![allow(unused)]

use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt)]
#[structopt(name = "fmm")]
struct App {
    #[structopt(short, long)]
    dir: PathBuf,
    #[structopt(short, long)]
    enable: Vec<InputMod>,
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    // Step 1: Get all mods in the directory
    // let mut directory_mods: HashMap<String, Vec<Version>> = HashMap::new();
    let directory_mods = fs::read_dir(&app.dir)?
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
    let mut mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

    // Enable specified mods
    for mod_data in app.enable {
        if directory_mods.contains_key(&mod_data.name) {
            let mod_state = mod_list_json
                .mods
                .iter_mut()
                .find(|mod_state| mod_data.name == mod_state.name);

            println!("Enabled {}", &mod_data);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = true;
                mod_state.version = mod_data.version;
            } else {
                mod_list_json.mods.push(ModListJsonMod {
                    name: mod_data.name.to_string(),
                    enabled: true,
                    version: mod_data.version,
                });
            }
        }
    }

    fs::write(&mlj_path, serde_json::to_string_pretty(&mod_list_json)?);

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

#[derive(Clone, Debug)]
pub struct InputMod {
    pub name: String,
    pub version: Option<Version>,
}

impl FromStr for InputMod {
    type Err = InputModErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: None,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    if !version.pre.is_empty() || !version.build.is_empty() {
                        Err(InputModErr::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: Some(version),
                        })
                    }
                } else {
                    Err(InputModErr::InvalidVersion(version.to_string()))
                }
            }
            _ => Err(InputModErr::IncorrectArgCount(parts.len())),
        }
    }
}

impl fmt::Display for InputMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            match &self.version {
                Some(version) => format!(" v{}", version),
                _ => "".to_string(),
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum InputModErr {
    #[error("Incorrect argument count: expected 1 or 2, got {0}")]
    IncorrectArgCount(usize),
    #[error("Invalid version identifier: `{0}`")]
    InvalidVersion(String),
}
