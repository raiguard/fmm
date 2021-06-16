use semver::Version;
use std::fmt;

use crate::mods_set::ModEnabledType;

#[derive(Debug)]
pub struct InputMod {
    pub name: String,
    pub version: ModEnabledType,
}

impl std::str::FromStr for InputMod {
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

#[derive(Debug)]
pub enum InputModErr {
    IncorrectArgCount(usize),
    InvalidVersion(String),
}

impl fmt::Display for InputModErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IncorrectArgCount(arg_count) => format!(
                    "Incorrect argument count: Expected 1 or 2, got {}",
                    arg_count
                ),
                Self::InvalidVersion(got_version) =>
                    format!("Invalid version identifier: {}", got_version),
            }
        )
    }
}

impl std::error::Error for InputModErr {}
