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
    /// OAUTH token for uploading mods
    #[clap(long)]
    pub upload_token: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Disable the given mods, or all mods if none are provided
    #[clap(short_flag = 'D')]
    Disable {
        /// The mod(s) to disable, formatted as 'name' or 'name@version'
        mods: Vec<ModIdent>,
    },
    /// Download the given mods
    #[clap(short_flag = 'L')]
    Download {
        /// The mod(s) to download, formatted as 'name' or 'name@version'
        mods: Vec<ModIdent>,
    },
    /// Enable the given mods
    #[clap(short_flag = 'E')]
    Enable {
        /// Ignore mod dependencies
        #[clap(short, long)]
        ignore_deps: bool,
        /// The mod(s) to enable, formatted as 'name' or 'name@version'
        mods: Vec<ModIdent>,
    },
    /// Remove the given mods
    #[clap(short_flag = 'R')]
    Remove {
        /// The mod(s) to remove, formatted as 'name' or 'name@version'
        mods: Vec<ModIdent>,
    },
    /// Search the mod portal
    #[clap(short_flag = 'F')]
    Search { query: String },
    /// Sync enabled mods with the given mod set or save file, downloading if necessary
    #[clap(short_flag = 'S')]
    Sync {
        /// Treat the input as a mod set name instead of a save file
        #[clap(short = 's', long = "set")]
        is_set: bool,
        /// Don't auto-download missing mods
        #[clap(short = 'l', long)]
        no_download: bool,
        /// Keep already-enabled mods enabled
        #[clap(short, long)]
        preserve: bool,
        /// The mod set or save file to sync with
        arg: String,
    },
    /// Query your local mod collection
    #[clap(short_flag = 'Q')]
    Query { mods: Vec<ModIdent> },
    /// Update the given mods, or all mods if none are provided
    #[clap(short_flag = 'U')]
    Update { mods: Vec<String> },
    /// Upload the mod to the portal
    #[clap(short_flag = 'P')]
    Upload { file: PathBuf },
}
