use anyhow::{anyhow, Result};
use serde::de;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    build: Option<u32>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, build: Option<u32>) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> de::Visitor<'de> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("factorio version")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                string.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Use iter_intersperse?
        write!(
            f,
            "{}.{}.{}{}",
            self.major,
            self.minor,
            self.patch,
            self.build
                .map(|build| format!(".{}", build))
                .unwrap_or_else(|| "".to_string())
        )
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('.').collect::<Vec<&str>>();

        match parts[..] {
            [major, minor] => Ok(Self {
                major: parse_version_number(major)?,
                minor: parse_version_number(minor)?,
                patch: 0,
                build: None,
            }),
            [major, minor, patch] => Ok(Self {
                major: parse_version_number(major)?,
                minor: parse_version_number(minor)?,
                patch: parse_version_number(patch)?,
                build: None,
            }),
            [major, minor, patch, build] => Ok(Self {
                major: parse_version_number(major)?,
                minor: parse_version_number(minor)?,
                patch: parse_version_number(patch)?,
                build: Some(parse_version_number(build)?),
            }),
            _ => Err(anyhow!(
                "Incorrect number of version parts - expected 3 or 4, got {}",
                parts.len()
            )),
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.build == other.build
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            r => return r,
        };
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            r => return r,
        };
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            r => return r,
        };

        match [self.build, other.build] {
            [Some(build), Some(other_build)] => match build.cmp(&other_build) {
                Ordering::Equal => {}
                r => return r,
            },
            [Some(_), None] => return Ordering::Greater,
            [None, Some(_)] => return Ordering::Less,
            _ => {}
        };

        Ordering::Equal
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

fn parse_version_number(s: &str) -> Result<u32, std::num::ParseIntError> {
    s.parse::<u32>()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionReq {
    cmp: VersionCmp,
    version: Version,
}

impl VersionReq {
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        let mut prefix_len = 2;
        let cmp = match &s[0..2] {
            "<=" => VersionCmp::LtEq,
            ">=" => VersionCmp::GtEq,
            _ => {
                prefix_len = 1;
                match &s[0..1] {
                    "<" => VersionCmp::LessThan,
                    "=" => VersionCmp::Equal,
                    ">" => VersionCmp::GreaterThan,
                    _ => return Err(anyhow!("Invalid version comparator: {}", s)),
                }
            }
        };

        Ok(Self {
            cmp,
            version: Version::from_str(s[prefix_len..].trim())?,
        })
    }

    pub fn matches(&self, v: &Version) -> bool {
        match self.cmp {
            VersionCmp::LessThan => v < &self.version,
            VersionCmp::LtEq => v <= &self.version,
            VersionCmp::Equal => v == &self.version,
            VersionCmp::GtEq => v >= &self.version,
            VersionCmp::GreaterThan => v > &self.version,
        }
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.cmp, self.version)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum VersionCmp {
    LessThan,
    LtEq,
    Equal,
    GtEq,
    GreaterThan,
}

impl Display for VersionCmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VersionCmp::LessThan => "<",
                VersionCmp::LtEq => "<=",
                VersionCmp::Equal => "=",
                VersionCmp::GtEq => ">=",
                VersionCmp::GreaterThan => ">",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        assert!(Version::from_str("1.0.0").is_ok());
        assert!(Version::from_str("1.0.0.0").is_ok());

        // Factorio does not care about leading zeroes
        assert!(Version::from_str("0.13.00").is_ok());
        // Factorio does not care about missing patch version
        assert!(Version::from_str("0.16").is_ok());

        // Too few version parts
        assert!(Version::from_str("1").is_err());
        // Version part is too large
        assert!(Version::from_str("1.0.4294967296").is_err());
    }

    #[test]
    fn to_string() {
        assert_eq!(Version::new(1, 0, 0, None).to_string(), "1.0.0");
        assert_eq!(Version::new(1, 0, 0, Some(0)).to_string(), "1.0.0.0");

        assert_eq!(Version::from_str("0.13.00").unwrap().to_string(), "0.13.0");
    }

    #[test]
    fn ordering() {
        assert!(Version::new(1, 0, 1, None) > Version::new(1, 0, 0, None));
        assert!(Version::new(1, 1, 0, None) > Version::new(1, 0, 0, None));
        assert!(Version::new(1, 1, 1, None) > Version::new(1, 0, 0, None));
        assert!(Version::new(1, 0, 0, None) < Version::new(1, 0, 1, None));
        assert!(Version::new(1, 0, 0, None) < Version::new(1, 1, 0, None));
        assert!(Version::new(1, 0, 0, None) < Version::new(1, 1, 1, None));

        assert!(Version::new(1, 0, 0, None) < Version::new(1, 0, 0, Some(0)));
        assert!(Version::new(1, 0, 0, Some(0)) < Version::new(1, 0, 0, Some(1)));

        assert!(Version::new(1, 0, 0, Some(0)) > Version::new(1, 0, 0, None));
        assert!(Version::new(1, 0, 0, Some(1)) > Version::new(1, 0, 0, Some(0)));
    }

    #[test]
    fn equality() {
        assert!(Version::new(1, 0, 0, None) == Version::new(1, 0, 0, None));
        assert!(Version::new(1, 0, 0, None) != Version::new(1, 1, 0, None));
        assert!(Version::new(1, 0, 0, None) != Version::new(1, 0, 1, None));
        assert!(Version::new(1, 0, 0, None) != Version::new(1, 1, 1, None));
        assert!(Version::new(1, 0, 0, None) != Version::new(1, 1, 1, Some(0)));
        assert!(Version::new(1, 0, 0, Some(0)) != Version::new(1, 1, 1, Some(1)));
    }

    #[test]
    fn version_req_parse() {
        struct TestSet {
            src: &'static str,
            cmp: VersionCmp,
            version: Version,
        }

        let sets = [
            // Standard
            TestSet {
                src: "< 1.0.0",
                cmp: VersionCmp::LessThan,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "<= 1.0.0",
                cmp: VersionCmp::LtEq,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "= 1.0.0",
                cmp: VersionCmp::Equal,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: ">= 1.0.0",
                cmp: VersionCmp::GtEq,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "> 1.0.0",
                cmp: VersionCmp::GreaterThan,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "<1.0.0",
                cmp: VersionCmp::LessThan,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "<=1.0.0",
                cmp: VersionCmp::LtEq,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: "=1.0.0",
                cmp: VersionCmp::Equal,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: ">=1.0.0",
                cmp: VersionCmp::GtEq,
                version: Version::new(1, 0, 0, None),
            },
            TestSet {
                src: ">1.0.0",
                cmp: VersionCmp::GreaterThan,
                version: Version::new(1, 0, 0, None),
            },
        ];

        for set in sets {
            let req = VersionReq::parse(set.src).unwrap();
            assert_eq!(req.cmp, set.cmp);
            assert_eq!(req.version, set.version);
        }
    }

    #[test]
    fn version_req_matches() {
        struct TestSet {
            src: &'static str,
            fails: Vec<Version>,
            passes: Vec<Version>,
        }

        let sets = [
            // Standard
            TestSet {
                src: "< 1.0.0",
                passes: vec![Version::new(0, 9, 0, None)],
                fails: vec![Version::new(1, 1, 0, None)],
            },
            TestSet {
                src: "<= 1.0.0",
                passes: vec![Version::new(0, 9, 0, None), Version::new(1, 0, 0, None)],
                fails: vec![Version::new(1, 1, 0, None)],
            },
            TestSet {
                src: "= 1.0.0",
                passes: vec![Version::new(1, 0, 0, None)],
                fails: vec![Version::new(1, 1, 0, None)],
            },
            TestSet {
                src: ">= 1.0.0",
                passes: vec![Version::new(1, 0, 0, None), Version::new(1, 1, 0, None)],
                fails: vec![Version::new(0, 9, 0, None)],
            },
            TestSet {
                src: "> 1.0.0",
                passes: vec![Version::new(1, 1, 0, None)],
                fails: vec![Version::new(0, 9, 0, None)],
            },
        ];

        for set in sets {
            let req = VersionReq::parse(set.src).unwrap();

            for version in &set.passes {
                assert!(req.matches(version));
            }
            for version in &set.fails {
                assert!(!req.matches(version));
            }
        }
    }
}
