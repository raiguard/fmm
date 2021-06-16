use semver::Version;
use std::fmt;

#[derive(Debug)]
pub struct InputMod {
    name: String,
    version: InputModVersion,
}

impl std::str::FromStr for InputMod {
    type Err = InputModErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: InputModVersion::Latest,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    if version.pre.len() > 0 || version.build.len() > 0 {
                        Err(Self::Err::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: InputModVersion::Version(version),
                        })
                    }
                } else {
                    Err(Self::Err::InvalidVersion(version.to_string()))
                }
            }
            _ => Err(Self::Err::IncorrectArgCount(parts.len())),
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

#[derive(Debug)]
enum InputModVersion {
    Latest,
    Version(Version),
}
