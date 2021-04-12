use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use zip::read::ZipArchive;

#[derive(Debug)]
struct AppArgs {
    dedup: bool,
    disable_all: bool,
    disable_base: bool,
    // disable: Option<ModsInputList>,
    enable_all: bool,
    // enable: Option<ModsInputList>,
    mods_path: PathBuf,
}

impl AppArgs {
    fn new(mut pargs: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
        Ok(AppArgs {
            dedup: pargs.contains("--dedup"),
            disable_all: pargs.contains("--disable-all"),
            disable_base: pargs.contains("--disable-base"),
            // disable: pargs
            //     .opt_value_from_fn("--disable", |value| ModsInputList::new(value, false))?,
            enable_all: pargs.contains("--enable-all"),
            // enable: pargs.opt_value_from_fn("--enable", |value| ModsInputList::new(value, true))?,
            mods_path: pargs.value_from_str("--modspath")?, // TODO: environment var and config file
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ModsListJson {
    mods: Vec<ModsListJsonMod>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModsListJsonMod {
    name: String,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

#[derive(Debug)]
struct ModsDirectory {
    mods: HashMap<String, Mod>,
    path: PathBuf,
}

#[derive(Deserialize, Debug)]
struct InfoJson {
    name: String,
    version: String,
}

#[derive(Debug)]
struct Mod {
    name: String,
    versions: Vec<String>,
    enabled: ModEnabledType,
}

#[derive(Debug)]
enum ModEnabledType {
    Disabled,
    Latest,
    Version(String),
}

fn read_info_json(entry: DirEntry) -> Result<InfoJson, Box<dyn Error>> {
    let metadata = entry.metadata()?;
    if metadata.is_dir() || metadata.file_type().is_symlink() {
        let mut path = entry.path();
        path.push("info.json");
        let contents = fs::read_to_string(path)?;
        let json: InfoJson = serde_json::from_str(&contents)?;
        Ok(json)
    } else if Some(OsStr::new("zip")) == Path::new(&entry.file_name()).extension() {
        let file = std::fs::File::open(entry.path())?;
        let mut archive = ZipArchive::new(file)?;
        // My hand is forced due to the lack of a proper iterator API in the `zip` crate
        // Thus, we need to use a bare `for` loop and iterate the indices, then act on the file if we find it
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("info.json") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let json: InfoJson = serde_json::from_str(&contents)?;
                return Ok(json);
            }
        }
        Err("Mod ZIP does not contain an info.json file".into())
    } else {
        Err("Is not a directory or zip file".into())
    }
}

impl ModsDirectory {
    fn new(directory: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut mod_list_json = String::new();
        let mut mods: HashMap<String, Mod> = HashMap::new();

        // Iterate files and directories to assemble mods
        let entries = fs::read_dir(&directory)?;
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_name() == "mod-list.json" {
                    let mut file = fs::File::open(entry.path())?;
                    file.read_to_string(&mut mod_list_json)?;
                } else if entry.file_name() != "mod-settings.dat" {
                    if let Ok(json) = read_info_json(entry) {
                        // TODO: This clone is bad
                        let mod_entry = mods.entry(json.name.clone()).or_insert(Mod {
                            name: json.name,
                            versions: vec![],
                            enabled: ModEnabledType::Disabled,
                        });
                        mod_entry.versions.push(json.version);
                    }
                }
            }
        }

        if mod_list_json.is_empty() {
            return Err("Unable to read mod-list.json".into());
        }

        // Parse mod-list.json to get active mod versions
        let mod_list = serde_json::from_str::<ModsListJson>(&mod_list_json)?.mods;
        for mod_data in mod_list {
            if mod_data.enabled {
                if let Some(mod_entry) = mods.get_mut(&mod_data.name) {
                    mod_entry.enabled = match mod_data.version {
                        Some(version) => ModEnabledType::Version(version),
                        None => ModEnabledType::Latest,
                    }
                }
            }
        }

        Ok(Self {
            mods,
            path: directory,
        })
    }
}

pub fn run(pargs: pico_args::Arguments) -> Result<(), Box<dyn Error>> {
    let args = AppArgs::new(pargs)?;

    let directory = ModsDirectory::new(args.mods_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tests_path(suffix: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push(suffix);
        d
    }

    #[test]
    // let directory = tests_path("mods_dir_1");
    fn mods_directory() {
        let directory = PathBuf::from("/home/rai/.factorio/mods");

        let directory = ModsDirectory::new(directory).unwrap();

        println!("{:#?}", directory.mods);
    }
}

// struct ModsInputList(Vec<ModData>);

// impl ModsInputList {
//     fn new(input: &str) -> Result<Self, String> {
//         // TODO: Throw error on illegal characters detected
//         // Legal characters are [a-zA-Z0-9_\- ]
//         let mods = input
//             .split('|')
//             .map(|mod_identifier| {
//                 let parts: Vec<&str> = mod_identifier.split('@').collect();
//                 // TODO: qualify mod name somehow
//                 // TODO: Check for duplicates

//                 if parts.len() == 0 || parts.len() > 2 {
//                     // TODO: More helpful formatting
//                     return Err("Invalid number of mod input sections".to_string());
//                 }

//                 let name = parts.get(0).unwrap();
//                 let version = parts.get(1).map(|version| version.to_string());

//                 Ok(ModData {
//                     name: name.to_string(),
//                     version,
//                 })
//             })
//             .collect::<Result<Vec<ModData>, String>>()?;

//         Ok(ModsInputList(mods))
//     }
// }

// // Use the ModsInputList like a vector by dereferencing
// impl Deref for ModsInputList {
//     type Target = Vec<ModData>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// struct ModsDirectory {
//     mods: HashMap<String, ModData>,
//     path: PathBuf,
// }

// struct ModData {
//     enabled: bool,
//     version: Option<String>,
// }

// fn update_mods(dir: &mut ModsDirectory, mods: &ModsInputList) {
//     for mod_data in mods.iter() {
//         println!(
//             "{} {}{}",
//             if mod_data.enabled {
//                 "Enabling"
//             } else {
//                 "Disabling"
//             },
//             mod_data.name,
//             if let Some(version) = &mod_data.version {
//                 format!(" v{}", version)
//             } else {
//                 "".to_string()
//             }
//         );
//         match dir.mods.binary_search(mod_data) {
//             Ok(index) => {
//                 let mut saved_mod_data = &mut dir.mods[index];
//                 saved_mod_data.enabled = mod_data.enabled;
//                 saved_mod_data.version = mod_data.version.clone();
//             }
//             Err(index) if mod_data.enabled => dir.mods.insert(index, mod_data.clone()),
//             Err(_) => (),
//         }
//     }
// }

// pub fn run(pargs: pico_args::Arguments) -> Result<(), Box<dyn Error>> {
//     let args = AppArgs::new(pargs)?;

//     let mut dir = ModsDirectory::new(&args.mods_path)?;

//     // TODO: Is this really needed?
//     if args.disable_all && args.enable_all {
//         return Err("Disabling all and enabling all makes no sense.".into());
//     }

//     if args.disable_all {
//         println!("Disabling all");
//         for mod_data in dir.mods.iter_mut() {
//             if args.disable_base || mod_data.name != "base" {
//                 mod_data.enabled = false;
//             }
//         }
//     }
//     if args.enable_all {
//         println!("Enabling all");
//         for mod_data in dir.mods.iter_mut() {
//             mod_data.enabled = true;
//         }
//     }

//     if let Some(mods) = args.disable {
//         update_mods(&mut dir, &mods);
//     }

//     if let Some(mods) = args.enable {
//         update_mods(&mut dir, &mods);
//     }

//     if args.dedup {
//         println!("Sorting and deduplicating entries");
//         dir.mods.dedup();
//     }

//     // Write to mod-list.json
//     fs::write(&dir.path, serde_json::to_string_pretty(&dir)?)?;

//     println!("Finished");

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn tests_path(suffix: &str) -> PathBuf {
//         let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//         d.push("resources/tests");
//         d.push(suffix);
//         println!("{:?}", d);
//         d
//     }

//     #[test]
//     fn one_latest() {
//         let mods = ModsInputList::new("RecipeBook", true).unwrap();
//         assert_eq!(
//             mods[0],
//             ModData {
//                 name: "RecipeBook".to_string(),
//                 enabled: true,
//                 version: None
//             }
//         );
//     }

//     #[test]
//     fn one_versioned() {
//         let mods = ModsInputList::new("RecipeBook@1.0.0", true).unwrap();
//         assert_eq!(
//             mods[0],
//             ModData {
//                 name: "RecipeBook".to_string(),
//                 enabled: true,
//                 version: Some("1.0.0".to_string()),
//             }
//         )
//     }

//     #[test]
//     fn invalid_format() {
//         let mods = ModsInputList::new("RecipeBook@1.0.0@foo", true);
//         assert!(mods.is_err());
//     }

//     #[test]
//     fn simple_mod_list() {
//         let dir = ModsDirectory::new(&tests_path("mods_dir_1")).unwrap();

//         let mod_data = ModData {
//             name: "aai-industry".to_string(),
//             enabled: false,
//             version: None,
//         };

//         assert!(dir.mods.binary_search(&mod_data).is_ok());
//     }
// }
