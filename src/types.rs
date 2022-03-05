use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt;
use std::fs::DirEntry;
use std::str::FromStr;
use thiserror::Error;

use crate::dependency::ModDependency;

#[derive(Deserialize, Serialize)]
pub struct ModListJson {
    pub mods: Vec<ModListJsonMod>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ModListJsonMod {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Version>,
    pub enabled: bool,
}

#[derive(Clone, Eq, Debug, Deserialize, PartialEq)]
pub struct ModIdent {
    pub name: String,
    pub version: Option<Version>,
}

impl FromStr for ModIdent {
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

impl fmt::Display for ModIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            match &self.version {
                Some(version_req) => format!(" {}", version_req),
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

pub struct ModEntry {
    pub entry: DirEntry,
    // This is always guaranteed to have a version
    pub ident: ModIdent,
}

impl crate::HasVersion for ModEntry {
    fn get_version(&self) -> &Version {
        self.ident.version.as_ref().unwrap()
    }
}

impl PartialOrd for ModEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ident
            .version
            .as_ref()
            .unwrap()
            .partial_cmp(other.ident.version.as_ref().unwrap())
    }
}

impl Ord for ModEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ident
            .version
            .as_ref()
            .unwrap()
            .cmp(other.ident.version.as_ref().unwrap())
    }
}

impl PartialEq for ModEntry {
    fn eq(&self, other: &Self) -> bool {
        self.ident.version.as_ref().unwrap() == other.ident.version.as_ref().unwrap()
    }
}

impl Eq for ModEntry {}

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

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct InfoJson {
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub dependencies: Option<Vec<ModDependency>>,
    pub name: String,
    pub version: Version,
}
