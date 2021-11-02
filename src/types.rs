use semver::Version;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt;
use std::fs::DirEntry;
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
pub struct ModListJson {
    pub mods: Vec<ModListJsonMod>,
}

#[derive(Deserialize, Serialize)]
pub struct ModListJsonMod {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Version>,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct InputMod {
    pub name: String,
    pub version: Option<Version>,
}

impl FromStr for InputMod {
    type Err = InputModErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: None,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    if !version.pre.is_empty() || !version.build.is_empty() {
                        Err(InputModErr::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: Some(version),
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
                Some(version) => format!(" v{}", version),
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

#[derive(Debug)]
pub enum ModEntryStructure {
    Directory,
    Symlink,
    Zip,
}

impl ModEntryStructure {
    pub fn parse(entry: &DirEntry) -> Option<Self> {
        let path = entry.path();
        let extension = path.extension();

        if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
            return Some(ModEntryStructure::Zip);
        } else {
            let file_type = entry.file_type().ok()?;
            if file_type.is_symlink() {
                return Some(ModEntryStructure::Symlink);
            } else {
                let mut path = entry.path();
                path.push("info.json");
                if path.exists() {
                    return Some(ModEntryStructure::Directory);
                }
            }
        };

        None
    }
}

#[derive(Deserialize, Debug)]
pub struct InfoJson {
    pub dependencies: Option<Vec<String>>,
    pub name: String,
    pub version: Version,
}
