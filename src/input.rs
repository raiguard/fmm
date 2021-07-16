use directories::BaseDirs;
use semver::Version;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

use crate::mods_set::ModEnabledType;

#[derive(Clone, Debug)]
pub struct InputMod {
    pub name: String,
    pub version: ModEnabledType,
}

impl FromStr for InputMod {
    type Err = InputModErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: ModEnabledType::Latest,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    if version.pre.len() > 0 || version.build.len() > 0 {
                        Err(InputModErr::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: ModEnabledType::Version(version),
                        })
                    }
                } else {
                    Err(InputModErr::InvalidVersion(version.to_string()))
                }
            }
            _ => Err(InputModErr::IncorrectArgCount(parts.len())),
        }
    }
}

impl fmt::Display for InputMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            match &self.version {
                ModEnabledType::Version(version) => format!(" v{}", version),
                _ => "".to_string(),
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum InputModErr {
    #[error("Incorrect argument count: expected 1 or 2, got {0}")]
    IncorrectArgCount(usize),
    #[error("Invalid version identifier: `{0}`")]
    InvalidVersion(String),
}

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

#[derive(Debug, Error)]
pub enum ConfigFileErr {
    #[error("Could not open config file.")]
    CouldNotOpenFile,
    #[error("Could not create config file path.")]
    CouldNotCreatePath,
    #[error("Could not create config file.")]
    CouldNotCreateFile,
}
