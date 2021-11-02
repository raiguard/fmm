#![feature(iter_intersperse)]

use semver::Version;
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
// use dependency::*;
use types::*;

#[derive(StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
struct App {
    #[structopt(long)]
    config: Option<PathBuf>,
    #[structopt(long)]
    dir: Option<PathBuf>,
    #[structopt(short = "o", long)]
    disable_all: bool,
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    #[structopt(short = "a", long)]
    enable_all: bool,
    #[structopt(short, long)]
    enable: Vec<InputMod>,
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
    let directory_mods = fs::read_dir(&dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();

            if let Some((mod_name, version)) = file_name.to_str()?.rsplit_once("_") {
                let (version, _) = version.rsplit_once(".").unwrap_or((version, "")); // Strip file extension

                Some((mod_name.to_string(), Version::parse(version).ok()?))
            } else {
                let info_json = read_info_json(&entry)?;

                Some((info_json.name, info_json.version))
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
    let enabled_versions = std::fs::read_to_string(&mlj_path)?;
    let mut mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

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

    // Enable all mods
    if app.enable_all {
        println!("Enabled latest versions of all mods");
        for mod_data in mod_list_json.mods.iter_mut() {
            mod_data.enabled = true;
            mod_data.version = None;
        }
    }

    // Enable specified mods
    for mod_ident in app.enable {
        if mod_ident.name != "base" {
            let mod_exists = if let Some(mod_versions) = directory_mods.get(&mod_ident.name) {
                mod_ident.version.is_none()
                    || mod_versions.contains(mod_ident.version.as_ref().unwrap())
            } else {
                false
            };

            if mod_exists {
                let mod_state = mod_list_json
                    .mods
                    .iter_mut()
                    .find(|mod_state| mod_ident.name == mod_state.name);

                println!("Enabled {}", &mod_ident);

                if let Some(mod_state) = mod_state {
                    mod_state.enabled = true;
                    mod_state.version = mod_ident.version;
                } else {
                    mod_list_json.mods.push(ModListJsonMod {
                        name: mod_ident.name.to_string(),
                        enabled: true,
                        version: mod_ident.version,
                    });
                }
            } else {
                println!("Could not find {}", &mod_ident);
            }
        }
    }

    // Disable specified mods
    for mod_data in app.disable {
        if mod_data.name == "base" || directory_mods.contains_key(&mod_data.name) {
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

    // Write mod-list.json
    fs::write(&mlj_path, serde_json::to_string_pretty(&mod_list_json)?)?;

    Ok(())
}
