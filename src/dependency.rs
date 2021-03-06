use crate::version::VersionReq;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{de, Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct ModDependency {
    pub dep_type: ModDependencyType,
    pub name: String,
    pub version_req: Option<VersionReq>,
}

impl<'de> Deserialize<'de> for ModDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> de::Visitor<'de> for VersionVisitor {
            type Value = ModDependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("factorio mod dependency")
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
impl FromStr for ModDependency {
    type Err = ModDependencyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Avoid creating the regex object every time
        static DEP_STRING_REGEX: OnceCell<Regex> = OnceCell::new();
        let captures = DEP_STRING_REGEX
            .get_or_init(|| {
                Regex::new(
                    r"^(?:(?P<type>[!?~]|\(\?\)) *)?(?P<name>(?: *[a-zA-Z0-9_-]+)+(?: *$)?)(?: *(?P<version_req>[<>=]=? *(?:\d+\.){1,2}\d+))?$",
                ).unwrap()
            })
            .captures(s)
            .ok_or_else(|| ModDependencyErr::InvalidDependencyString(s.to_string()))?;

        Ok(Self {
            dep_type: match captures.name("type").map(|mtch| mtch.as_str()) {
                None => ModDependencyType::Required,
                Some("!") => ModDependencyType::Incompatible,
                Some("~") | Some("(~)") => ModDependencyType::NoLoadOrder,
                Some("?") => ModDependencyType::Optional,
                Some("(?)") => ModDependencyType::OptionalHidden,
                Some(str) => return Err(ModDependencyErr::UnknownModifier(str.to_string())),
            },
            name: match captures.name("name") {
                Some(mtch) => mtch.as_str().to_string(),
                None => return Err(ModDependencyErr::NameIsUnparsable(s.to_string())),
            },
            version_req: match captures.name("version_req") {
                Some(req_match) => match VersionReq::parse(req_match.as_str()) {
                    Ok(version_req) => Some(version_req),
                    Err(_) => {
                        return Err(ModDependencyErr::InvalidVersionReq(
                            req_match.as_str().to_string(),
                        ))
                    }
                },
                _ => None,
            },
        })
    }
}

impl Display for ModDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            match self.dep_type {
                ModDependencyType::Incompatible => "! ",
                ModDependencyType::NoLoadOrder => "~ ",
                ModDependencyType::Optional => "? ",
                ModDependencyType::OptionalHidden => "(?) ",
                ModDependencyType::Required => "",
            },
            self.name,
            match &self.version_req {
                Some(version_req) => format!(" {}", version_req),
                None => String::new(),
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ModDependencyType {
    Incompatible,
    NoLoadOrder,
    Optional,
    OptionalHidden,
    Required,
}

#[derive(Clone, Debug, Error)]
pub enum ModDependencyErr {
    #[error("Invalid dependency string: `{0}`")]
    InvalidDependencyString(String),
    #[error("Invalid dependency version requirement: `{0}`")]
    InvalidVersionReq(String),
    #[error("Dependency name could not be parsed: `{0}`")]
    NameIsUnparsable(String),
    #[error("Unknown dependency modifier: `{0}`")]
    UnknownModifier(String),
}
