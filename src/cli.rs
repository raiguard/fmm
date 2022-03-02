use crate::types::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
    /// Path to a custom config file
    #[clap(long = "config")]
    pub config: Option<PathBuf>,
    /// Path to the game directory
    #[clap(long = "game-dir")]
    pub game_dir: Option<PathBuf>,
    /// Path to the mods directory
    #[clap(long = "mods-dir")]
    pub mods_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Enable, disable, or download packaged mods
    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync {
        /// Enable the given mods
        #[clap(short = 'e', long = "enable")]
        enable: Vec<ModIdent>,
        /// Enable the given mod set
        #[clap(short = 'E', long = "enable-set")]
        enable_set: Option<String>,
        /// Disable the given mods
        #[clap(short = 'd', long = "disable")]
        disable: Vec<ModIdent>,
        /// Disable the given mod set
        #[clap(short = 'D', long = "disable-set")]
        disable_set: Option<String>,
        /// Disable all  mods before taking other actions
        #[clap(short = 'o', long = "disable-all")]
        disable_all: bool,
        /// Disable mod auto-download
        #[clap(short = 'l', long = "nodownload")]
        no_download: bool,
        /// Sync active mods and startup settings with the given save file
        #[clap(short = 's', long = "save-file")]
        save_file: Option<PathBuf>,
    },
}
