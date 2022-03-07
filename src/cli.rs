use crate::mod_ident::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
    /// Path to a custom config file
    #[clap(long)]
    pub config: Option<PathBuf>,
    /// Path to the game directory
    #[clap(long)]
    pub game_dir: Option<PathBuf>,
    /// Path to the mods directory
    #[clap(long)]
    pub mods_dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Enable, disable, or download packaged mods
    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync {
        /// Ignore mod dependencies
        #[clap(short, long)]
        ignore_deps: bool,
        /// Ignore startup settings when syncing with a save file
        #[clap(short = 'x', long)]
        ignore_startup_settings: bool,
        /// Disable mod auto-download
        #[clap(short = 'l', long)]
        no_download: bool,

        /// Sync active mods and startup settings to the save file
        #[clap(short, long)]
        save_file: Option<PathBuf>,
        /// Disable all mods before taking other actions
        #[clap(short = 'o', long)]
        disable_all: bool,
        /// Disable the given mods
        #[clap(short, long)]
        disable: Vec<ModIdent>,
        /// Enable the given mods
        #[clap(short, long)]
        enable: Vec<ModIdent>,
        /// Enable the given mod set
        #[clap(short = 'E', long)]
        enable_set: Option<String>,
    },
}
