use semver::Version;
use std::error::Error;
use std::ops::Deref;

#[derive(Debug)]
pub struct ModsInputList(Vec<ModData>);

impl ModsInputList {
    pub fn new(input: &str) -> Result<Self, Box<dyn Error>> {
        let mods = input
            .split('|')
            .map(|mod_identifier| {
                let parts: Vec<&str> = mod_identifier.split('@').collect();

                if parts.len() == 0 || parts.len() > 2 {
                    return Err("Invalid number of mod input sections".into());
                }

                let name = parts.get(0).unwrap();
                let version: Option<Version> = match parts.get(1) {
                    Some(version) => match Version::parse(version) {
                        Ok(version) => Some(version),
                        Err(_) => return Err("Could not parse version number".into()),
                    },
                    None => None,
                };

                Ok(ModData {
                    name: name.to_string(),
                    version,
                })
            })
            .collect::<Result<Vec<ModData>, String>>()?;

        Ok(ModsInputList(mods))
    }
}

// Use the ModsInputList like a vector by dereferencing
impl Deref for ModsInputList {
    type Target = Vec<ModData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ModData {
    pub name: String,
    pub version: Option<Version>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_latest() {
        let mods = ModsInputList::new("RecipeBook").unwrap();
        assert_eq!(
            mods[0],
            ModData {
                name: "RecipeBook".to_string(),
                version: None
            }
        );
    }

    #[test]
    fn one_versioned() {
        let mods = ModsInputList::new("RecipeBook@1.0.0").unwrap();
        assert_eq!(
            mods[0],
            ModData {
                name: "RecipeBook".to_string(),
                version: Some(Version::parse("1.0.0").unwrap()),
            }
        )
    }

    #[test]
    fn invalid_format() {
        let mods = ModsInputList::new("RecipeBook@1.0.0@foo");
        assert!(mods.is_err());
    }
}
