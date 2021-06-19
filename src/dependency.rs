use itertools::Itertools;
use once_cell::sync::OnceCell;
use regex::Regex;
use semver::VersionReq;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct ModDependency {
    pub dep_type: ModDependencyType,
    pub name: String,
    pub version_req: Option<VersionReq>,
}

impl ModDependency {
    pub fn new(input: &String) -> Result<Self, ModDependencyErr> {
        // Avoid creating the regex object every time
        static DEP_STRING_REGEX: OnceCell<Regex> = OnceCell::new();
        let captures = DEP_STRING_REGEX
            .get_or_init(|| {
                Regex::new(
                    r"^(?:(?P<type>[!?~]|\(\?\)) *)?(?P<name>(?: *[a-zA-Z0-9_-]+)+(?: *$)?)(?: *(?P<version_req>[<>=]=? *(?P<version>(?:\d+\.){1,2}\d+)))?$",
                ).unwrap()
            })
            .captures(input)
            .ok_or(ModDependencyErr::InvalidDependencyString(input.clone()))?;

        Ok(Self {
            dep_type: match captures.name("type").map(|mtch| mtch.as_str()) {
                None => ModDependencyType::Required,
                Some("!") => ModDependencyType::Incompatible,
                Some("~") => ModDependencyType::NoLoadOrder,
                Some("?") => ModDependencyType::Optional,
                Some("(?)") => ModDependencyType::OptionalHidden,
                Some(str) => return Err(ModDependencyErr::UnknownModifier(str.to_string())),
            },
            name: match captures.name("name") {
                Some(mtch) => mtch.as_str().to_string(),
                None => return Err(ModDependencyErr::NameIsUnparsable(input.clone())),
            },
            version_req: match captures.name("version_req") {
                Some(mtch) => {
                    // Factorio does not sanitize leading zeros, so we must do it ourselves
                    // PANIC: We can safely assume that the version capture is valid if version_req exists
                    let version_str = captures.name("version").unwrap().as_str();
                    let sanitized = version_str
                        .split('.')
                        .map(|sub| {
                            Ok(sub
                                .parse::<usize>()
                                .map_err(|_| {
                                    ModDependencyErr::InvalidDependencyString(input.clone())
                                })?
                                .to_string())
                        })
                        .intersperse(Ok(".".to_string()))
                        .collect::<Result<String, ModDependencyErr>>()?;

                    match VersionReq::parse(&sanitized) {
                        Ok(version_req) => Some(version_req),
                        Err(_) => {
                            return Err(ModDependencyErr::InvalidVersionReq(
                                mtch.as_str().to_string(),
                            ))
                        }
                    }
                }
                None => None,
            },
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ModDependencyType {
    Incompatible,
    NoLoadOrder,
    Optional,
    OptionalHidden,
    Required,
}

pub type ModDependencyResult = Result<Vec<ModDependency>, ModDependencyErr>;

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
