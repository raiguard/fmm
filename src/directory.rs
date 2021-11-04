use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::PathBuf;

use semver::Version;
use zip::ZipArchive;

use crate::types::*;

pub struct Directory {
    pub mods: HashMap<String, Vec<ModVersion>>,
    pub mod_list: Vec<ModListJsonMod>,
    pub mod_list_path: PathBuf,
}

impl Directory {
    pub fn new(dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Get all mods in the directory
        let mod_entries = fs::read_dir(&dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_name = entry.file_name();

                if let Some((mod_name, version)) = file_name.to_str()?.rsplit_once("_") {
                    let (version, _) = version.rsplit_once(".").unwrap_or((version, "")); // Strip file extension
                    Some((
                        mod_name.to_string(),
                        ModVersion {
                            entry,
                            version: Version::parse(version).ok()?,
                        },
                    ))
                } else {
                    let info_json = read_info_json(&entry)?;

                    Some((
                        info_json.name,
                        ModVersion {
                            entry,
                            version: info_json.version,
                        },
                    ))
                }
            })
            .fold(HashMap::new(), |mut directory_mods, (mod_name, version)| {
                let versions = directory_mods.entry(mod_name).or_insert_with(Vec::new);

                let index = versions
                    .binary_search(&version)
                    .unwrap_or_else(|index| index);
                versions.insert(index, version);

                directory_mods
            });

        // Parse mod-list.json
        let mut mlj_path = dir;
        mlj_path.push("mod-list.json");
        let enabled_versions = fs::read_to_string(&mlj_path)?;
        let mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

        Ok(Self {
            mods: mod_entries,
            mod_list: mod_list_json.mods,
            mod_list_path: mlj_path,
        })
    }
}

// TODO: Use errors instead of an option
pub fn read_info_json(entry: &DirEntry) -> Option<InfoJson> {
    let contents = match ModEntryStructure::parse(entry)? {
        ModEntryStructure::Directory | ModEntryStructure::Symlink => {
            let mut path = entry.path();
            path.push("info.json");
            fs::read_to_string(path).ok()?
        }
        ModEntryStructure::Zip => {
            let mut archive = ZipArchive::new(File::open(entry.path()).ok()?).ok()?;
            let filename = archive
                .file_names()
                .find(|name| name.contains("info.json"))
                .map(ToString::to_string)?;
            let mut file = archive.by_name(&filename).ok()?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok()?;
            contents
        }
    };

    serde_json::from_str::<InfoJson>(&contents).ok()
}
