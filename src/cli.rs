use crate::types::*;
use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Parser)]
#[clap(author, version, about)]
pub struct App {
    /// The path to the configuration file
    #[clap(long)]
    pub config: Option<PathBuf>,
    /// Disables all mods in the directory
    #[clap(short = 'o', long)]
    pub disable_all: bool,
    /// Disables the given mods. Mods are formatted as `Name`
    #[clap(short, long)]
    pub disable: Vec<ModIdent>,
    /// Enables all mods in the directory
    #[clap(short = 'a', long)]
    pub enable_all: bool,
    /// Enables the mods in the given set
    #[clap(short = 'E', long)]
    pub enable_set: Option<String>,
    /// Enables the given mods. Mods are formatted as `Name` or `Name@Version`
    #[clap(short, long)]
    pub enable: Vec<ModIdent>,
    /// The game directory to manipulate. Optional if a configuration file is in use
    #[clap(long)]
    pub game_dir: Option<PathBuf>,
    /// Lists all mods in the directory
    #[clap(short, long)]
    pub list: bool,
    /// The mods directory to manipulate. Optional if a configuration file is in use
    #[clap(long)]
    pub mods_dir: Option<PathBuf>,
    /// Removes the given mods from the mods directory. Mods are formatted as `Name` or `Name@Version`
    #[clap(short, long)]
    pub remove: Vec<ModIdent>,
    /// A path to a save file to sync with
    #[clap(short, long)]
    pub sync: Option<PathBuf>,
}
