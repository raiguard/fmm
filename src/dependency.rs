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
    pub fn new(input: &str) -> Result<Self, ModDependencyErr> {
        // Avoid creating the regex object every time
        static DEP_STRING_REGEX: OnceCell<Regex> = OnceCell::new();
        let captures = DEP_STRING_REGEX
            .get_or_init(|| {
                Regex::new(
                    r"^(?:(?P<type>[!?~]|\(\?\)) *)?(?P<name>(?: *[a-zA-Z0-9_-]+)+(?: *$)?)(?: *(?P<version_req>[<>=]=?) *(?P<version>(?:\d+\.){1,2}\d+))?$",
                ).unwrap()
            })
            .captures(input)
            .ok_or_else(|| ModDependencyErr::InvalidDependencyString(input.to_string()))?;

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
                None => return Err(ModDependencyErr::NameIsUnparsable(input.to_string())),
            },
            version_req: match [captures.name("version_req"), captures.name("version")] {
                [Some(req_match), Some(version_match)] => {
                    // Factorio does not sanitize leading zeros, so we must do it ourselves
                    let version_str = version_match.as_str();
                    let sanitized = version_str
                        .split('.')
                        .map(|sub| {
                            Ok(sub
                                .parse::<usize>()
                                .map_err(|_| {
                                    ModDependencyErr::InvalidDependencyString(input.to_string())
                                })?
                                .to_string())
                        })
                        .intersperse(Ok(".".to_string()))
                        .collect::<Result<String, ModDependencyErr>>()?;

                    // Assemble version req from parts
                    let mut version_req = String::new();
                    version_req.push_str(req_match.as_str());
                    version_req.push(' ');
                    version_req.push_str(&sanitized);

                    match VersionReq::parse(&version_req) {
                        Ok(version_req) => Some(version_req),
                        Err(_) => {
                            return Err(ModDependencyErr::InvalidVersionReq(
                                req_match.as_str().to_string(),
                            ))
                        }
                    }
                }
                _ => None,
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
