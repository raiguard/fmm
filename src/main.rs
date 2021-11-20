#![feature(iter_intersperse)]

use anyhow::Result;
use reqwest::blocking::Client;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod dependency;
mod directory;
mod download;
mod sync;
mod types;

use config::*;
use directory::*;
use types::*;

#[derive(Clone, StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
pub struct App {
    /// The path to the configuration file
    #[structopt(long)]
    config: Option<PathBuf>,
    /// Disables all mods in the directory
    #[structopt(short = "o", long)]
    disable_all: bool,
    /// Disables the given mods. Mods are formatted as `Name`
    #[structopt(short, long)]
    disable: Vec<ModIdent>,
    /// Downloads the given mods from the mod portal. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short = "D", long)]
    download: Vec<ModIdent>,
    /// Enables all mods in the directory
    #[structopt(short = "a", long)]
    enable_all: bool,
    /// Enables the mods in the given set
    #[structopt(short = "E", long)]
    enable_set: Option<String>,
    /// Enables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    enable: Vec<ModIdent>,
    /// The game directory to manipulate. Optional if a configuration file is in use
    #[structopt(long)]
    game_dir: Option<PathBuf>,
    /// Lists all mods in the directory
    #[structopt(short, long)]
    list: bool,
    /// The mods directory to manipulate. Optional if a configuration file is in use
    #[structopt(long)]
    mods_dir: Option<PathBuf>,
    /// Removes the given mods from the mods directory. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    remove: Vec<ModIdent>,
    /// A path to a save file to sync with
    #[structopt(short, long)]
    sync: Option<PathBuf>,
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

fn main() -> Result<()> {
    let mut app = App::from_args();

    let config = Config::new(&app)?;

    let mut directory = Directory::new(config.mods_dir.clone())?;

    // List mods
    if app.list {
        let mut lines: Vec<String> = directory
            .mods
            .0
            .iter()
            .flat_map(|(mod_name, mod_versions)| {
                mod_versions
                    .iter()
                    .map(|version| format!("{} v{}", mod_name, version.version))
                    .collect::<Vec<String>>()
            })
            .collect();

        lines.sort();

        for line in lines {
            println!("{}", line)
        }
    }

    // Enable set
    if let Some(set_name) = app.enable_set {
        if let Some(sets) = config.sets.as_ref() {
            if let Some(set) = sets.get(&set_name) {
                app.enable = set.to_vec();
            }
        }
    }

    // Sync with save
    if let Some(sync_path) = app.sync {
        let save_file = sync::SaveFile::from(sync_path)?;

        // TODO: Prepend instead of overwriting
        app.enable = save_file.mods;
    }

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

    // Download mods
    let client = Client::new();
    for mod_ident in app.download {
        download::download_mod(&client, &config, &mod_ident)?;
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
