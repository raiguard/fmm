use once_cell::sync::OnceCell;
use regex::Regex;
use semver::VersionReq;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct ModDependency {
    pub dep_type: ModDependencyType,
    pub name: String,
    pub version_req: Option<VersionReq>,
}

impl ModDependency {
    pub fn new(input: &str) -> Result<Self, Box<dyn Error>> {
        // Avoid creating the regex object every time
        static RE: OnceCell<Regex> = OnceCell::new();
        let captures = RE
            .get_or_init(|| {
                Regex::new(
                    r"^(?:(?P<type>[!?~]|\(\?\)) *)?(?P<name>(?: *[a-zA-Z0-9_-]+)+(?: *$)?)(?: *(?P<version_req>[<>=]=? *(?:\d+\.){1,2}\d+))?$",
                ).unwrap()
            })
            .captures(input)
            .ok_or("Invalid dependency string")?;

        Ok(Self {
            dep_type: match captures.name("type").map(|mtch| mtch.as_str()) {
                None => ModDependencyType::Required,
                Some("!") => ModDependencyType::Incompatible,
                Some("~") => ModDependencyType::NoLoadOrder,
                Some("?") => ModDependencyType::Optional,
                Some("(?)") => ModDependencyType::OptionalHidden,
                Some(str) => return Err(format!("Unknown dependency modifier: {}", str).into()),
            },
            name: match captures.name("name") {
                Some(mtch) => mtch.as_str().to_string(),
                None => return Err("Name was not parseable".into()),
            },
            version_req: match captures.name("version_req") {
                Some(mtch) => match VersionReq::parse(mtch.as_str()) {
                    Ok(version_req) => Some(version_req),
                    Err(err) => return Err(err.into()),
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
