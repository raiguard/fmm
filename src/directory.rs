use anyhow::Result;
use console::style;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::PathBuf;

use semver::{Version, VersionReq};
use zip::ZipArchive;

use crate::config::Config;
use crate::dependency::ModDependencyType;
use crate::get_mod_version;
use crate::types::*;

pub struct Directory {
    pub mods: HashMap<String, Vec<ModEntry>>,
    pub mod_list: Vec<ModListJsonMod>,
    pub mod_list_path: PathBuf,
}

impl Directory {
    pub fn new(mut path: PathBuf) -> Result<Self> {
        // Get all mods in the directory
        let mod_entries = fs::read_dir(&path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;

                if let Some((mod_name, version)) = parse_file_name(&entry.file_name()) {
                    Some((mod_name, ModEntry { entry, version }))
                } else {
                    let info_json = read_info_json(&entry)?;
                    Some((
                        info_json.name,
                        ModEntry {
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
        path.push("mod-list.json");
        let enabled_versions = fs::read_to_string(&path)?;
        let mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

        Ok(Self {
            mods: mod_entries,
            mod_list: mod_list_json.mods,
            mod_list_path: path,
        })
    }

    /// Adds the mod, but keeps it disabled
    pub fn add(&mut self, (mod_name, mod_entry): (String, ModEntry)) {
        // Add or disable mod in mod-list.json
        if let Some(mod_state) = self
            .mod_list
            .iter_mut()
            .find(|mod_state| mod_name == mod_state.name)
        {
            mod_state.enabled = false;
            mod_state.version = None;
        } else {
            self.mod_list.push(ModListJsonMod {
                name: mod_name.clone(),
                enabled: false,
                version: None,
            });
        }

        let entries = self.mods.entry(mod_name).or_default();
        if let Err(index) = entries.binary_search(&mod_entry) {
            entries.insert(index, mod_entry);
        }
    }

    pub fn disable(&mut self, mod_ident: &ModIdent) {
        if mod_ident.name == "base" || self.mods.contains_key(&mod_ident.name) {
            let mod_state = self
                .mod_list
                .iter_mut()
                .find(|mod_state| mod_ident.name == mod_state.name);

            println!("{} {}", style("Disabled").yellow().bold(), &mod_ident);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = false;
                mod_state.version = None;
            }
        } else {
            println!("Could not find {}", &mod_ident);
        }
    }

    pub fn disable_all(&mut self) {
        println!("{}", style("Disabled all mods").yellow().bold());

        for mod_data in self
            .mod_list
            .iter_mut()
            .filter(|mod_state| mod_state.name != "base")
        {
            mod_data.enabled = false;
            mod_data.version = None;
        }
    }

    pub fn enable(&mut self, mod_ident: &ModIdent, config: &Config) -> Option<Vec<ManageOrder>> {
        if let Some(mod_entry) = self
            .mods
            .get(&mod_ident.name)
            .and_then(|mod_entries| get_mod_version(mod_entries, mod_ident))
        {
            let mod_state = self
                .mod_list
                .iter_mut()
                .find(|mod_state| mod_ident.name == mod_state.name);

            let enabled = mod_state.is_some() && mod_state.as_ref().unwrap().enabled;

            if !enabled {
                println!(
                    "{} {} v{}",
                    style("Enabled").green().bold(),
                    mod_ident.name,
                    mod_entry.version
                );

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
                        .map(|dependency| ModIdent {
                            name: dependency.name.clone(),
                            version_req: dependency.version_req.clone(),
                        })
                        .filter_map(|dependency_ident| {
                            self.mods
                                .get(&dependency_ident.name)
                                .and_then(|dependency_entries| {
                                    get_mod_version(dependency_entries, &dependency_ident)
                                })
                                .map(|_| ManageOrder::Enable(dependency_ident.clone()))
                                .or_else(|| {
                                    if config.auto_download {
                                        Some(ManageOrder::Download(dependency_ident.clone()))
                                    } else {
                                        // TODO: Print message when we do not have a mod and are not auto-downloading it
                                        None
                                    }
                                })
                        })
                        .collect(),
                );
            }
        } else if config.auto_download {
            return Some(vec![ManageOrder::Download(mod_ident.clone())]);
        }

        None
    }

    pub fn enable_all(&mut self) {
        println!(
            "{}",
            style("Enabled latest versions of all mods").green().bold()
        );
        for mod_data in self.mod_list.iter_mut() {
            mod_data.enabled = true;
            mod_data.version = None;
        }
    }

    pub fn remove(&mut self, mod_ident: &ModIdent) {
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
                        println!(
                            "{} {} v{}",
                            style("Removed").red().bold(),
                            &mod_ident.name,
                            version.version
                        );
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

fn parse_file_name(file_name: &OsString) -> Option<(String, Version)> {
    let (name, version) = file_name
        .to_str()?
        .trim_end_matches(".zip")
        .rsplit_once("_")?;

    if let Ok(version) = Version::parse(version) {
        Some((name.to_string(), version))
    } else {
        None
    }
}

// TODO: Use errors instead of an option
fn read_info_json(entry: &DirEntry) -> Option<InfoJson> {
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
