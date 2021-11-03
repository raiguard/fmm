#![feature(iter_intersperse)]

use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use zip::ZipArchive;

mod config;
mod dependency;
mod types;

use config::*;
use dependency::*;
use types::*;

#[derive(StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
struct App {
    /// The path to the configuration file
    #[structopt(long)]
    config: Option<PathBuf>,
    /// The mods directory to manipulate. Optional if a configuration file is in use
    #[structopt(long)]
    dir: Option<PathBuf>,
    /// Disables all mods in the directory
    #[structopt(short = "o", long)]
    disable_all: bool,
    /// Disables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    /// Enables all mods in the directory
    #[structopt(short = "a", long)]
    enable_all: bool,
    /// Enables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    enable: Vec<InputMod>,
    /// Removes the given mods from the mods directory. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    remove: Vec<InputMod>,
}

impl App {
    fn merge_config(&mut self, config_file: ConfigFile) {
        if let Some(directory) = config_file.directory {
            self.dir = Some(directory);
        }
    }
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::from_args();

    let config = ConfigFile::new(&app.config)?;
    if let Some(config) = config {
        app.merge_config(config);
    }

    if app.dir.is_none() {
        return Err("Must specify a mods path via flag or via the configuration file.".into());
    }

    let dir = app.dir.unwrap();

    // Get all mods in the directory
    let mut mod_entries = fs::read_dir(&dir)?
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
    let mut mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

    // Remove specified mods
    for mod_ident in app.remove {
        if mod_ident.name != "base" {
            let version_req = mod_ident
                .version_req
                .as_ref()
                .cloned()
                .unwrap_or_else(VersionReq::any);
            if let Some(mod_versions) = mod_entries.get(&mod_ident.name) {
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
                mod_entries.remove(&mod_ident.name);
            }

            if let Some((index, _)) = mod_list_json
                .mods
                .iter()
                .enumerate()
                .find(|(_, mod_state)| mod_ident.name == mod_state.name)
            {
                mod_list_json.mods.remove(index);
            }
        }
    }

    // Disable all mods
    if app.disable_all {
        println!("Disabled all mods");
        for mod_data in mod_list_json
            .mods
            .iter_mut()
            .filter(|mod_state| mod_state.name != "base")
        {
            mod_data.enabled = false;
            mod_data.version = None;
        }
    }

    // Disable specified mods
    for mod_data in app.disable {
        if mod_data.name == "base" || mod_entries.contains_key(&mod_data.name) {
            let mod_state = mod_list_json
                .mods
                .iter_mut()
                .find(|mod_state| mod_data.name == mod_state.name);

            println!("Disabled {}", &mod_data);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = false;
                mod_state.version = None;
            }
        } else {
            println!("Could not find {}", &mod_data);
        }
    }

    // Enable all mods
    if app.enable_all {
        println!("Enabled latest versions of all mods");
        for mod_data in mod_list_json.mods.iter_mut() {
            mod_data.enabled = true;
            mod_data.version = None;
        }
    }

    // Enable specified mods
    let mut to_enable = app.enable.clone();
    while !to_enable.is_empty() {
        let mut to_enable_next = Vec::new();
        for mod_ident in &to_enable {
            if mod_ident.name != "base" {
                let mod_entry = mod_entries.get(&mod_ident.name).and_then(|mod_versions| {
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
                    let mod_state = mod_list_json
                        .mods
                        .iter_mut()
                        .find(|mod_state| mod_ident.name == mod_state.name);

                    let enabled = mod_state.is_some() && mod_state.as_ref().unwrap().enabled;

                    if enabled {
                        println!(
                            "{} v{} is already enabled",
                            mod_ident.name, mod_entry.version
                        );
                    } else {
                        println!("Enabled {} v{}", mod_ident.name, mod_entry.version);

                        let version = mod_ident
                            .version_req
                            .as_ref()
                            .map(|_| mod_entry.version.clone());

                        if let Some(mod_state) = mod_state {
                            mod_state.enabled = true;
                            mod_state.version = version;
                        } else {
                            mod_list_json.mods.push(ModListJsonMod {
                                name: mod_ident.name.to_string(),
                                enabled: true,
                                version,
                            });
                        }

                        to_enable_next.append(
                            &mut read_info_json(&mod_entry.entry)
                                .and_then(|info_json| info_json.dependencies)
                                .unwrap_or_default()
                                .iter()
                                .filter(|dependency| {
                                    dependency.name != "base"
                                        && matches!(
                                            dependency.dep_type,
                                            ModDependencyType::NoLoadOrder
                                                | ModDependencyType::Required
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
            }
        }

        to_enable = to_enable_next;
    }

    // Write mod-list.json
    fs::write(&mlj_path, serde_json::to_string_pretty(&mod_list_json)?)?;

    Ok(())
}
