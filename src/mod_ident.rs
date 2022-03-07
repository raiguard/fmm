use crate::Version;
use anyhow::anyhow;
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Eq, Debug, Deserialize, PartialEq)]
pub struct ModIdent {
    pub name: String,
    pub version: Option<Version>,
}

impl ModIdent {
    /// In cases where `version` is guaranteed to be Some(), get the version
    pub fn get_guaranteed_version(&self) -> &Version {
        self.version
            .as_ref()
            .expect("Version was not present in guaranteed case")
    }
}

impl FromStr for ModIdent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: None,
            }),
            [name, version] => {
                let parsed_version = Version::from_str(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    Ok(Self {
                        name: name.to_string(),
                        version: Some(version),
                    })
                } else {
                    Err(anyhow!(
                        "Invalid version identifier: {}",
                        version.to_string()
                    ))
                }
            }
            _ => Err(anyhow!(
                "Incorrect mod format - must be 'Name' or 'Name@Version'"
            )),
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
