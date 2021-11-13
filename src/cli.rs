use anyhow::anyhow;
use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::config::*;
use crate::types::*;

#[derive(StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
pub struct App {
    /// The path to the configuration file
    #[structopt(long)]
    pub config: Option<PathBuf>,
    /// The directory of the Factorio installation
    #[structopt(long)]
    pub game_dir: Option<PathBuf>,
    /// The directory where mods are kept. Defaults to factorio-dir/mods
    #[structopt(long)]
    pub mods_dir: Option<PathBuf>,
    /// Disables all mods in the directory
    #[structopt(short = "o", long)]
    pub disable_all: bool,
    /// Disables the given mods. Mods are formatted as `Name`
    #[structopt(short, long)]
    pub disable: Vec<ModIdent>,
    /// Enables all mods in the directory
    #[structopt(short = "a", long)]
    pub enable_all: bool,
    /// Enables the mods in the given set
    #[structopt(short = "E", long)]
    pub enable_set: Option<String>,
    /// Enables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    pub enable: Vec<ModIdent>,
    /// Lists all mods in the directory
    #[structopt(short, long)]
    pub list: bool,
    /// Removes the given mods from the mods directory. Mods are formatted as `Name` or `Name@Version`
    #[structopt(short, long)]
    pub remove: Vec<ModIdent>,
    /// A path to a save file to sync with
    #[structopt(short, long)]
    pub sync: Option<PathBuf>,
}

impl App {
    pub fn merge_config(&mut self, config_file: ConfigFile) -> Result<()> {
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
