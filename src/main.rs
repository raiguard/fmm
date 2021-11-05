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
    /// Enables the mods in the given set
    #[structopt(short = "s", long)]
    enable_set: Option<String>,
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
        // Pull mods to enable from the defined set, if any
        if let Some(set_name) = &app.enable_set {
            if let Some(sets) = &config.sets {
                if let Some(set) = sets.get(set_name) {
                    app.enable = set.to_vec()
                } else {
                    return Err(format!("Set `{}` is not defined", set_name).into());
                }
            }
        }

        app.merge_config(config);
    }

    let mut directory = Directory::new(match app.dir {
        Some(dir) => dir,
        None => {
            return Err("Must specify a mods path via flag or via the configuration file.".into())
        }
    })?;

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
    for mod_ident in app.disable {
        directory.disable(&mod_ident);
    }

    // Enable all mods
    if app.enable_all {
        directory.enable_all();
    }

    // Enable specified mods
    let mut to_enable = app.enable;
    while !to_enable.is_empty() {
        to_enable = to_enable
            .iter_mut()
            .filter(|mod_ident| mod_ident.name != "base")
            .filter_map(|mod_ident| directory.enable(mod_ident))
            .flatten()
            .collect();
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
