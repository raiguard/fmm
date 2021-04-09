use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug)]
struct AppArgs {
    dedup: bool,
    disable_all: bool,
    disable_base: bool,
    disable: Option<ModsInputList>,
    enable_all: bool,
    enable: Option<ModsInputList>,
    mods_path: String,
}

impl AppArgs {
    fn new(mut pargs: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
        Ok(AppArgs {
            dedup: pargs.contains("--dedup"),
            disable_all: pargs.contains("--disable-all"),
            disable_base: pargs.contains("--disable-base"),
            disable: pargs
                .opt_value_from_fn("--disable", |value| ModsInputList::new(value, false))?,
            enable_all: pargs.contains("--enable-all"),
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

#[derive(Debug, Serialize, Deserialize)]
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

        // `binary_search()` depends on a sorted vector
        collection.mods.sort();

        collection.path = path;

        Ok(collection)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ModData {
    name: String,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl PartialOrd for ModData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for ModData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for ModData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ModData {}

fn update_mods(dir: &mut ModsDirectory, mods: &ModsInputList) {
    for mod_data in mods.iter() {
        println!(
            "{} {}{}",
            if mod_data.enabled {
                "Enabling"
            } else {
                "Disabling"
            },
            mod_data.name,
            if let Some(version) = &mod_data.version {
                format!(" v{}", version)
            } else {
                "".to_string()
            }
        );
        match dir.mods.binary_search(mod_data) {
            Ok(index) => {
                let mut saved_mod_data = &mut dir.mods[index];
                saved_mod_data.enabled = mod_data.enabled;
                saved_mod_data.version = mod_data.version.clone();
            }
            Err(index) if mod_data.enabled => dir.mods.insert(index, mod_data.clone()),
            Err(_) => (),
        }
    }
}

pub fn run(pargs: pico_args::Arguments) -> Result<(), Box<dyn Error>> {
    let args = AppArgs::new(pargs)?;

    let mut dir = ModsDirectory::new(&args.mods_path)?;

    // TODO: Is this really needed?
    if args.disable_all && args.enable_all {
        return Err("Disabling all and enabling all makes no sense.".into());
    }

    if args.disable_all {
        println!("Disabling all");
        for mod_data in dir.mods.iter_mut() {
            if args.disable_base || mod_data.name != "base" {
                mod_data.enabled = false;
            }
        }
    }
    if args.enable_all {
        println!("Enabling all");
        for mod_data in dir.mods.iter_mut() {
            mod_data.enabled = true;
        }
    }

    if let Some(mods) = args.disable {
        update_mods(&mut dir, &mods);
    }

    if let Some(mods) = args.enable {
        update_mods(&mut dir, &mods);
    }

    if args.dedup {
        println!("Sorting and deduplicating entries");
        dir.mods.dedup();
    }

    // Write to mod-list.json
    fs::write(&dir.path, serde_json::to_string_pretty(&dir)?)?;

    println!("Finished");

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
    fn simple_mod_list() {
        let dir = ModsDirectory::new(
            "/home/rai/dev/projects/personal/factorio_mod_manager/demo_mods_dir",
        )
        .unwrap();

        let mod_data = ModData {
            name: "aai-industry".to_string(),
            enabled: false,
            version: None,
        };

        assert!(dir.mods.binary_search(&mod_data).is_ok());
    }

    // TODO: Write tests to use a fake mod-list.json and compare the results
    // TODO: There is some weirdness around version-specific queries
}
