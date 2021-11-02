use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub directory: Option<PathBuf>,
}

impl ConfigFile {
    pub fn new(path: &Option<PathBuf>) -> Result<Option<Self>, ConfigFileErr> {
        // Pass custom config path, or create the default config file it it doesn't exist
        let config_path: Option<PathBuf> = path.clone().or({
            if let Some(base_dirs) = BaseDirs::new() {
                let mut config_path: PathBuf = base_dirs.config_dir().into();
                config_path.push("fmm");
                if !config_path.exists() {
                    fs::create_dir_all(&config_path)
                        .map_err(|_| ConfigFileErr::CouldNotCreatePath)?;
                }
                config_path.push("fmm.toml");
                if !config_path.exists() {
                    config_path.push("fmm.toml");
                    File::create(&config_path).map_err(|_| ConfigFileErr::CouldNotCreateFile)?;
                };
                Some(config_path)
            } else {
                None
            }
        });
        if config_path.is_none() {
            return Ok(None);
        }

        let file = std::fs::read_to_string(config_path.unwrap())
            .map_err(|_| ConfigFileErr::CouldNotOpenFile)?;

        // FIXME: Don't unwrap here. Use anyhow?
        let config: ConfigFile = toml::from_str(&file).unwrap();
        Ok(Some(config))
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum ConfigFileErr {
    #[error("Could not open config file.")]
    CouldNotOpenFile,
    #[error("Could not create config file path.")]
    CouldNotCreatePath,
    #[error("Could not create config file.")]
    CouldNotCreateFile,
}
