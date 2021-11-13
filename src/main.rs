#![feature(iter_intersperse)]

use anyhow::anyhow;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod dependency;
mod directory;
mod sync;
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
    /// The directory of the Factorio installation
    #[structopt(long)]
    game_dir: Option<PathBuf>,
    /// The directory where mods are kept. Defaults to factorio-dir/mods
    #[structopt(long)]
    mods_dir: Option<PathBuf>,
    /// Disables all mods in the directory
    #[structopt(short = "o", long)]
    disable_all: bool,
    /// Disables the given mods. Mods are formatted as `Name`
    #[structopt(short, long)]
    disable: Vec<ModIdent>,
    /// Enables all mods in the directory
    #[structopt(short = "a", long)]
    enable_all: bool,
    /// Enables the mods in the given set
    #[structopt(short = "E", long)]
    enable_set: Option<String>,
    /// Enables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    enable: Vec<ModIdent>,
    /// Lists all mods in the directory
    #[structopt(short, long)]
    list: bool,
    /// Removes the given mods from the mods directory. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    remove: Vec<ModIdent>,
    /// A path to a save file to sync with
    #[structopt(short, long)]
    sync: Option<PathBuf>,
}

impl App {
    fn merge_config(&mut self, config_file: ConfigFile) -> Result<()> {
        // Directories
        match [config_file.game_dir, config_file.mods_dir] {
            [Some(game_dir), Some(mods_dir)] => {
                self.game_dir = Some(game_dir);
                self.mods_dir = Some(mods_dir);
            }
            [Some(game_dir), None] => {
                let mut mods_dir = game_dir.clone();
                mods_dir.push("mods");
                if !mods_dir.exists() {
                    return Err(anyhow!("Mods directory is not in the expected location."));
                }
                self.game_dir = Some(game_dir);
                self.mods_dir = Some(mods_dir);
            }
            [None, Some(mods_dir)] => {
                self.mods_dir = Some(mods_dir);
            }
            _ => (),
        };

        // Mod sets
        if let Some(set_name) = &self.enable_set {
            if let Some(sets) = config_file.sets {
                if let Some(set) = sets.get(set_name) {
                    self.enable = set.to_vec()
                } else {
                    return Err(anyhow!("Set `{}` is not defined", set_name));
                }
            }
        }

        Ok(())
    }
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

fn main() -> Result<()> {
    let mut app = App::from_args();

    if let Some(config) = ConfigFile::new(&app.config)? {
        app.merge_config(config)?;
    }

    let mut directory = Directory::new(match app.mods_dir {
        Some(dir) => dir,
        None => {
            return Err(anyhow!(
                "Must specify a directory path via flag or via the configuration file."
            ))
        }
    })?;

    // List mods
    if app.list {
        let mut lines: Vec<String> = directory
            .mods
            .iter()
            .flat_map(|(mod_name, mod_versions)| {
                mod_versions
                    .iter()
                    .map(|version| format!("{} v{}", mod_name, version.version))
                    .collect::<Vec<String>>()
            })
            .collect();

        lines.sort();
        lines.iter().for_each(|line| println!("{}", line));
    }

    // Sync with save
    if let Some(sync_path) = app.sync {
        let save_file = sync::SaveFile::from(sync_path)?;

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

    // Write mod-list.json
    fs::write(
        &directory.mod_list_path,
        serde_json::to_string_pretty(&ModListJson {
            mods: directory.mod_list,
        })?,
    )?;

    Ok(())
}
