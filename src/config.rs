use anyhow::{anyhow, Result};
use directories::BaseDirs;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::types::ModIdent;
use crate::App;

#[derive(Debug)]
pub struct Config {
    pub game_dir: Option<PathBuf>,
    pub mods_dir: PathBuf,
    pub portal_auth: Option<PortalAuth>,
    pub sets: ModSets,
}

impl Config {
    pub fn new(app: &App) -> Result<Self> {
        // Input
        let config_file = ConfigFile::new(&app.config)?.unwrap_or_default();

        // Merge config options
        let game_dir = app.game_dir.clone().or(config_file.game_dir);
        Ok(Config {
            game_dir: game_dir.clone(),
            mods_dir: match [
                game_dir.clone(),
                app.mods_dir.clone().or(config_file.mods_dir),
            ] {
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
            sets: config_file.sets,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PortalAuth {
    pub token: String,
    pub username: String,
}

pub type ModSets = Option<HashMap<String, Vec<ModIdent>>>;

#[serde_as]
#[derive(Deserialize, Default)]
struct ConfigFile {
    game_dir: Option<PathBuf>,
    mods_dir: Option<PathBuf>,
    portal: Option<PortalAuth>,
    #[serde_as(as = "Option<HashMap<_, Vec<DisplayFromStr>>>")]
    sets: ModSets,
}

impl ConfigFile {
    pub fn new(path: &Option<PathBuf>) -> Result<Option<Self>, ConfigFileErr> {
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
pub enum ConfigFileErr {
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
