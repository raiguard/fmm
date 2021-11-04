use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::PathBuf;

use semver::{Version, VersionReq};
use zip::ZipArchive;

use crate::dependency::ModDependencyType;
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

    pub fn disable(&mut self, mod_ident: &InputMod) {
        if mod_ident.name == "base" || self.mods.contains_key(&mod_ident.name) {
            let mod_state = self
                .mod_list
                .iter_mut()
                .find(|mod_state| mod_ident.name == mod_state.name);

            println!("Disabled {}", &mod_ident);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = false;
                mod_state.version = None;
            }
        } else {
            println!("Could not find {}", &mod_ident);
        }
    }

    pub fn disable_all(&mut self) {
        println!("Disabled all mods");
        for mod_data in self
            .mod_list
            .iter_mut()
            .filter(|mod_state| mod_state.name != "base")
        {
            mod_data.enabled = false;
            mod_data.version = None;
        }
    }

    pub fn enable(&mut self, mod_ident: &InputMod) -> Option<Vec<InputMod>> {
        let mod_entry = self.mods.get(&mod_ident.name).and_then(|mod_versions| {
            if let Some(version_req) = &mod_ident.version_req {
                mod_versions
                    .iter()
                    .rev()
                    .find(|version| version_req.matches(&version.version))
            } else {
                mod_versions.last()
            }
        });

        if let Some(mod_entry) = mod_entry {
            let mod_state = self
                .mod_list
                .iter_mut()
                .find(|mod_state| mod_ident.name == mod_state.name);

            let enabled = mod_state.is_some() && mod_state.as_ref().unwrap().enabled;

            if !enabled {
                println!("Enabled {} v{}", mod_ident.name, mod_entry.version);

                let version = mod_ident
                    .version_req
                    .as_ref()
                    .map(|_| mod_entry.version.clone());

                if let Some(mod_state) = mod_state {
                    mod_state.enabled = true;
                    mod_state.version = version;
                } else {
                    self.mod_list.push(ModListJsonMod {
                        name: mod_ident.name.to_string(),
                        enabled: true,
                        version,
                    });
                }

                return Some(
                    read_info_json(&mod_entry.entry)
                        .and_then(|info_json| info_json.dependencies)
                        .unwrap_or_default()
                        .iter()
                        .filter(|dependency| {
                            dependency.name != "base"
                                && matches!(
                                    dependency.dep_type,
                                    ModDependencyType::NoLoadOrder | ModDependencyType::Required
                                )
                        })
                        .map(|dependency| InputMod {
                            name: dependency.name.clone(),
                            version_req: dependency.version_req.clone(),
                        })
                        .collect(),
                );
            }
        } else {
            println!("Could not find or read {}", &mod_ident);
        }

        None
    }

    pub fn enable_all(&mut self) {
        println!("Enabled latest versions of all mods");
        for mod_data in self.mod_list.iter_mut() {
            mod_data.enabled = true;
            mod_data.version = None;
        }
    }

    pub fn remove(&mut self, mod_ident: &InputMod) {
        let version_req = mod_ident
            .version_req
            .as_ref()
            .cloned()
            .unwrap_or_else(VersionReq::any);
        if let Some(mod_versions) = self.mods.get(&mod_ident.name) {
            mod_versions
                .iter()
                .filter(|version| version_req.matches(&version.version))
                .for_each(|version| {
                    let result = version.entry.metadata().and_then(|metadata| {
                        if metadata.is_dir() {
                            fs::remove_dir_all(version.entry.path())
                        } else {
                            fs::remove_file(version.entry.path())
                        }
                    });
                    if result.is_ok() {
                        println!("Removed {} v{}", &mod_ident.name, version.version);
                    } else {
                        println!("Could not remove {} v{}", &mod_ident.name, version.version);
                    }
                });
            self.mods.remove(&mod_ident.name);
        }

        if let Some((index, _)) = self
            .mod_list
            .iter()
            .enumerate()
            .find(|(_, mod_state)| mod_ident.name == mod_state.name)
        {
            self.mod_list.remove(index);
        }
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