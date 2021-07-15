use semver::Version;
use std::str::FromStr;
use std::{collections::HashSet, fmt};
use thiserror::Error;

use crate::mods_set::ModEnabledType;

#[derive(Debug)]
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

pub struct ModEnabledLists {
    pub did_enable: HashSet<String>,
    pub to_enable: Vec<InputMod>,
}

impl ModEnabledLists {
    pub fn new(to_enable: Vec<InputMod>) -> Self {
        Self {
            did_enable: HashSet::new(),
            to_enable,
        }
    }
}
