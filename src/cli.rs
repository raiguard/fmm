use crate::types::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
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

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Enable, disable, or download packaged mods
    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync {
        #[clap(subcommand)]
        cmd: SyncCmd,
        /// Disable all mods before taking other actions
        #[clap(short = 'o', long = "disable-all")]
        disable_all: bool,
        /// Disable mod auto-download
        #[clap(short = 'l', long = "nodownload")]
        no_download: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SyncCmd {
    /// Enable the given mods
    #[clap(short_flag = 'e', long_flag = "enable")]
    Enable {
        /// The mods to enable, formatted as `Name` or `Name@Version`
        mods: Vec<ModIdent>,
    },
    /// Enable the given mod set
    #[clap(short_flag = 'E', long_flag = "enable-set")]
    EnableSet {
        /// The name of the mod set to enable
        set: Option<String>,
    },
    /// Disable the given mods
    #[clap(short_flag = 'd', long_flag = "disable")]
    Disable {
        /// The mods to disable, formatted as `Name` or `Name@Version`
        mods: Vec<ModIdent>,
    },
    /// Disable the given mod set
    #[clap(short_flag = 'D', long_flag = "disable-set")]
    DisableSet {
        /// The name of the mod set to disable
        mods: Option<String>,
    },
    /// Sync active mods and startup settings with the given save file
    #[clap(short_flag = 's', long_flag = "save-file")]
    SaveFile {
        /// Path to the save file
        path: Option<PathBuf>,
    },
    /// Upgrade installed mods
    Upgrade {
        /// The mods to upgrade, or blank to upgrade all mods
        mods: Vec<ModIdent>,
    },
}
