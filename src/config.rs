use directories::BaseDirs;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::types::InputMod;

#[serde_as]
#[derive(Deserialize)]
pub struct ConfigFile {
    pub directory: Option<PathBuf>,
    #[serde_as(as = "Option<HashMap<_, Vec<DisplayFromStr>>>")]
    pub sets: Option<HashMap<String, Vec<InputMod>>>,
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
