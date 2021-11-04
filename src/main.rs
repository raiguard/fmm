#![feature(iter_intersperse)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod dependency;
mod directory;
mod types;

use config::*;
use dependency::*;
use directory::*;
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
    /// Disables the given mods. Mods are formatted as `Name`
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::from_args();

    let config = ConfigFile::new(&app.config)?;
    if let Some(config) = config {
        app.merge_config(config);
    }

    if app.dir.is_none() {
        return Err("Must specify a mods path via flag or via the configuration file.".into());
    }

    let mut directory = Directory::new(app.dir.unwrap())?;

    // Remove specified mods
    for mod_ident in app.remove {
        if mod_ident.name != "base" {
            directory.remove(&mod_ident);
        }
    }

    // Disable all mods
    if app.disable_all {
        directory.disable_all();
    }

    // Disable specified mods
    for mod_data in app.disable {
        if mod_data.name == "base" || directory.mods.contains_key(&mod_data.name) {
            let mod_state = directory
                .mod_list
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
        directory.enable_all();
    }

    // Enable specified mods
    let mut to_enable = app.enable.clone();
    while !to_enable.is_empty() {
        let mut to_enable_next = Vec::new();
        for mod_ident in &to_enable {
            if mod_ident.name != "base" {
                let mod_entry = directory
                    .mods
                    .get(&mod_ident.name)
                    .and_then(|mod_versions| {
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
                    let mod_state = directory
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
                            directory.mod_list.push(ModListJsonMod {
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
    fs::write(
        &directory.mod_list_path,
        serde_json::to_string_pretty(&ModListJson {
            mods: directory.mod_list,
        })?,
    )?;

    Ok(())
}
