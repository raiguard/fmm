use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug)]
struct AppArgs {
    // TODO: `enable` and `disable` can be combined
    enable: Option<ModsInputList>,
    enable_all: bool,
    disable: Option<ModsInputList>,
    disable_all: bool,
    mods_path: String,
}

impl AppArgs {
    fn new(mut pargs: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
        Ok(AppArgs {
            disable_all: pargs.opt_value_from_str("--disable-all")?.unwrap_or(false),
            disable: pargs
                .opt_value_from_fn("--disable", |value| ModsInputList::new(value, false))?,
            enable_all: pargs.opt_value_from_str("--enable-all")?.unwrap_or(false),
            enable: pargs.opt_value_from_fn("--enable", |value| ModsInputList::new(value, true))?,
            mods_path: pargs.value_from_str("--modspath")?,
        })
    }
}

#[derive(Debug)]
struct ModsInputList(Vec<ModData>);

impl ModsInputList {
    fn new(input: &str, to_enable: bool) -> Result<Self, String> {
        // TODO: Throw error on illegal characters detected
        // Legal characters are [a-zA-Z0-9_\- ]
        let mods = input
            .split('|')
            .map(|mod_identifier| {
                let parts: Vec<&str> = mod_identifier.split('@').collect();
                // TODO: qualify mod name somehow
                // TODO: Check for duplicates

                if parts.len() == 0 || parts.len() > 2 {
                    // TODO: More helpful formatting
                    return Err("Invalid number of mod input sections".to_string());
                }

                let name = parts.get(0).unwrap();
                let version = parts.get(1).map(|version| version.to_string());

                Ok(ModData {
                    name: name.to_string(),
                    enabled: to_enable,
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

#[derive(Debug, Deserialize, Serialize)]
struct ModsDirectory {
    mods: Vec<ModData>,
    #[serde(skip)]
    path: PathBuf,
}

impl ModsDirectory {
    fn new(directory: &str) -> Result<ModsDirectory, Box<dyn Error>> {
        let mut path: PathBuf = PathBuf::new();
        path.push(directory);
        path.push("mod-list.json");

        let mut collection: ModsDirectory = serde_json::from_str(&fs::read_to_string(&path)?)?;

        collection.path = path;

        Ok(collection)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct ModData {
    name: String,
    enabled: bool,
    version: Option<String>,
}

pub fn run(pargs: pico_args::Arguments) -> Result<(), Box<dyn Error>> {
    let args = AppArgs::new(pargs)?;

    let collection = ModsDirectory::new(&args.mods_path)?;

    println!("{:#?}", collection);

    print!("{:#?}", args);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_latest() {
        let mods = ModsInputList::new("RecipeBook", true).unwrap();
        assert_eq!(
            mods[0],
            ModData {
                name: "RecipeBook".to_string(),
                enabled: true,
                version: None
            }
        );
    }

    #[test]
    fn one_versioned() {
        let mods = ModsInputList::new("RecipeBook@1.0.0", true).unwrap();
        assert_eq!(
            mods[0],
            ModData {
                name: "RecipeBook".to_string(),
                enabled: true,
                version: Some("1.0.0".to_string()),
            }
        )
    }

    #[test]
    fn invalid_format() {
        let mods = ModsInputList::new("RecipeBook@1.0.0@foo", true);
        assert!(mods.is_err());
    }

    #[test]
    fn simple_mod_list() {}
}
