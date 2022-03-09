use crate::cli;
use crate::ModIdent;
use anyhow::{anyhow, ensure, Result};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub cmd: cli::Cmd,
    pub game_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub portal_auth: Option<PortalAuth>,
    pub sets: ModSets,
    pub sync_latest_versions: bool,
    pub sync_startup_settings: bool,
}

impl Config {
    pub fn new(args: cli::Args) -> Result<Self> {
        let config_file = ConfigFile::new(args.config)?.unwrap_or_default();

        let game_dir = args
            .game_dir
            .or(config_file.game_dir)
            .ok_or_else(|| anyhow!("Did not provide game directory"))?;
        ensure!(game_dir.exists(), "Invalid game directory");

        let mods_dir = args
            .mods_dir
            .or(config_file.mods_dir)
            .or_else(|| Some(game_dir.join("mods")))
            .ok_or_else(|| anyhow!("Did not provide mods directory"))?;
        ensure!(mods_dir.exists(), "Invalid mods directory");

        let portal_auth = config_file.portal.or_else(|| {
            let player_data_path = game_dir.join("player-data.json");
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
            None
        });

        Ok(Self {
            cmd: args.cmd,
            game_dir,
            mods_dir,
            portal_auth,
            sets: config_file.sets,
            sync_latest_versions: config_file.sync_latest_versions,
            sync_startup_settings: config_file.sync_startup_settings.unwrap_or(true),
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
    // TODO: Figure out how to have multiple deserialzers for ModIdent
    #[serde_as(as = "Option<HashMap<_, Vec<DisplayFromStr>>>")]
    sets: ModSets,
    #[serde(default)]
    sync_latest_versions: bool,
    sync_startup_settings: Option<bool>,
}

impl ConfigFile {
    pub fn new(path: Option<PathBuf>) -> Result<Option<Self>> {
        let config_path = path.unwrap_or(
            dirs::config_dir()
                .ok_or_else(|| anyhow!("Failed to find config directory"))?
                .join("fmm")
                .join("fmm.toml"),
        );
        if !config_path.exists() {
            return Ok(None);
        }

        let file =
            fs::read_to_string(config_path).map_err(|_| anyhow!("Failed to open config file"))?;
        let config: ConfigFile =
            toml::from_str(&file).map_err(|_| anyhow!("Failed to parse config file"))?;

        Ok(Some(config))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PlayerDataJson {
    service_username: Option<String>,
    service_token: Option<String>,
}
