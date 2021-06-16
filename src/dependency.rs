use once_cell::sync::OnceCell;
use regex::Regex;
use semver::VersionReq;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ModDependency {
    pub dep_type: ModDependencyType,
    pub name: String,
    pub version_req: Option<VersionReq>,
}

impl ModDependency {
    pub fn new(input: &String) -> Result<Self, ModDependencyErr> {
        // Avoid creating the regex object every time
        static RE: OnceCell<Regex> = OnceCell::new();
        let captures = RE
            .get_or_init(|| {
                Regex::new(
                    r"^(?:(?P<type>[!?~]|\(\?\)) *)?(?P<name>(?: *[a-zA-Z0-9_-]+)+(?: *$)?)(?: *(?P<version_req>[<>=]=? *(?:\d+\.){1,2}\d+))?$",
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
                // FIXME: Format version number first to remove leading zeroes to prevent an error. Will need to use a regex replace.
                Some(mtch) => match VersionReq::parse(mtch.as_str()) {
                    Ok(version_req) => Some(version_req),
                    Err(_) => {
                        return Err(ModDependencyErr::InvalidVersionReq(
                            mtch.as_str().to_string(),
                        ))
                    }
                },
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

pub enum ModDependencyErr {
    InvalidDependencyString(String),
    InvalidVersionReq(String),
    NameIsUnparsable(String),
    UnknownModifier(String),
}

impl fmt::Display for ModDependencyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidDependencyString(input) =>
                    format!("Invalid dependency string: `{}`", input),
                Self::InvalidVersionReq(version) =>
                    format!("Invalid dependency version requirement: `{}`", version),
                Self::NameIsUnparsable(input) =>
                    format!("Dependency name could not be parsed: `{}`", input),
                Self::UnknownModifier(modifier) =>
                    format!("Unknown dependency modifier: `{}`", modifier),
            }
        )
    }
}

impl fmt::Debug for ModDependencyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl Error for ModDependencyErr {}
