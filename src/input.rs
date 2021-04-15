use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;

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
                let version = parts.get(1).map(|version| version.to_string());

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

pub struct ModsDirectory {
    mods: HashMap<String, ModData>,
    path: PathBuf,
}

#[derive(Debug)]
pub struct ModData {
    name: String,
    version: Option<String>,
}
