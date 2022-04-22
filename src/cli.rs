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
    /// Download, enable, disable, or remove mods
    #[clap(short_flag = 'S', long_flag = "sync")]
    Sync(SyncArgs),
    /// Query your local mod collection and the mod portal
    #[clap(short_flag = 'Q', long_flag = "query")]
    Query(QueryArgs),
}

#[derive(clap::Args, Debug)]
pub struct SyncArgs {
    /// Ignore mod dependencies
    #[clap(short, long)]
    pub ignore_deps: bool,
    /// Ignore startup settings when syncing with a save file
    #[clap(short = 'x', long)]
    pub ignore_startup_settings: bool,

    /// Sync active mods and startup settings to the save file
    #[clap(short = 'f', long)]
    pub save_file: Option<PathBuf>,
    /// Disable all mods before taking other actions
    #[clap(short = 'o', long)]
    pub disable_all: bool,
    /// Disable the given mods
    #[clap(short, long, multiple_values(true), multiple_occurrences(false))]
    pub disable: Vec<ModIdent>,
    /// Download and enable the given mods
    #[clap(short, long, multiple_values(true), multiple_occurrences(false))]
    pub enable: Vec<ModIdent>,
    /// Download and enable the given mod set
    #[clap(short = 'E', long)]
    pub enable_set: Option<String>,
    /// Remove the given mods from the mods directory
    #[clap(short, long, multiple_values(true), multiple_occurrences(false))]
    pub remove: Vec<ModIdent>,
}

#[derive(clap::Args, Debug)]
pub struct QueryArgs {
    pub query: String,
}
