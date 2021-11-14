use anyhow::anyhow;
use anyhow::Result;
use directories::BaseDirs;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

use crate::directory::Directory;
use crate::types::*;

#[derive(Debug, Default)]
pub struct Actions {
    pub remove: Vec<ModIdent>,
    pub disable: ModifyType,
    pub download: Vec<ModIdent>,
    pub enable: ModifyType,
}

#[derive(Debug)]
pub enum ModifyType {
    All,
    Some(Vec<ModIdent>),
    None,
}

impl Default for ModifyType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub struct Config {
    pub game_dir: Option<PathBuf>,
    pub mods_dir: PathBuf,
    pub portal_auth: Option<PortalAuth>,
}

#[derive(Debug, Deserialize)]
pub struct PortalAuth {
    pub username: String,
    pub token: String,
}

pub fn proc_input() -> Result<(Actions, Config, Directory)> {
    // Input
    let args = Args::from_args();
    let config_file = ConfigFile::new(&args.config)?.unwrap_or_default();

    // Merge config options
    let config = {
        let game_dir = args.game_dir.or(config_file.game_dir);
        Config {
            game_dir: game_dir.clone(),
            mods_dir: match [game_dir.clone(), args.mods_dir.or(config_file.mods_dir)] {
                [_, Some(mods_dir)] if mods_dir.exists() => mods_dir,
                [Some(game_dir), None] if game_dir.exists() => {
                    let mut mods_dir = game_dir;
                    mods_dir.push("mods");
                    if !mods_dir.exists() {
                        return Err(anyhow!("Could not find mods directory"));
                    }
                    mods_dir
                }
                _ => return Err(anyhow!("Invalid game or mods directories")),
            },
            portal_auth: config_file.portal.or_else(|| {
                if let Some(game_dir) = &game_dir {
                    let mut player_data_path = game_dir.clone();
                    player_data_path.push("player-data.json");
                    if player_data_path.exists() {
                        let player_data_json = fs::read_to_string(&player_data_path).ok()?;
                        if let PlayerDataJson {
                            service_token: Some(token),
                            service_username: Some(username),
                        } = serde_json::from_str(&player_data_json).ok()?
                        {
                            return Some(PortalAuth { token, username });
                        }
                    }
                }
                None
            }),
        }
    };

    // Process actions
    let directory = Directory::new(&config.mods_dir)?;
    let mut actions = Actions {
        remove: args.remove,
        ..Default::default()
    };

    // Disabling
    if args.disable_all {
        actions.disable = ModifyType::All;
    } else if !args.disable.is_empty() {
        actions.disable = ModifyType::Some(args.disable);
    }

    // TODO: Downloading

    // Enabling
    if args.enable_all {
        actions.enable = ModifyType::All;
    } else {
        if let Some(set_name) = args.enable_set {
            if let Some(set) = config_file.sets.and_then(|mut sets| sets.remove(&set_name)) {
                actions.enable = ModifyType::Some(set);
            }
        }
        if !args.enable.is_empty() {
            actions.enable = match actions.enable {
                ModifyType::None => ModifyType::Some(args.enable),
                ModifyType::Some(mut mods) => {
                    mods.extend(args.enable);
                    ModifyType::Some(mods)
                }
                ModifyType::All => ModifyType::All,
            };
        }
    }
    // TODO: Get dependencies for all mods and extend the list with them
    // We want to memoize the info.json parsing into the `Directories` struct

    Ok((actions, config, directory))
}

#[derive(StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
struct Args {
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

#[serde_as]
#[derive(Deserialize, Default)]
struct ConfigFile {
    game_dir: Option<PathBuf>,
    mods_dir: Option<PathBuf>,
    portal: Option<PortalAuth>,
    #[serde_as(as = "Option<HashMap<_, Vec<DisplayFromStr>>>")]
    sets: Option<HashMap<String, Vec<ModIdent>>>,
}

impl ConfigFile {
    fn new(path: &Option<PathBuf>) -> Result<Option<Self>, ConfigFileErr> {
        let config_path: Option<PathBuf> = path
            .clone()
            .or({
                BaseDirs::new().map(|base_dirs| {
                    let mut config_path: PathBuf = base_dirs.config_dir().into();
                    config_path.push("fmm");
                    config_path.push("fmm.toml");
                    config_path
                })
            })
            .filter(|config_path| config_path.exists());

        if config_path.is_none() {
            return Ok(None);
        }

        let file = fs::read_to_string(config_path.unwrap()).map_err(|_| ConfigFileErr::Open)?;

        let config: ConfigFile = toml::from_str(&file).map_err(|_| ConfigFileErr::ParseFile)?;
        Ok(Some(config))
    }
}

#[derive(Debug, Error)]
enum ConfigFileErr {
    #[error("Could not open config file.")]
    Open,
    #[error("Could not parse config file.")]
    ParseFile,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PlayerDataJson {
    service_username: Option<String>,
    service_token: Option<String>,
}