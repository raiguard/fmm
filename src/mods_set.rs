use semver::Version;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

use crate::dependency::{ModDependency, ModDependencyResult};
use crate::input::InputMod;

#[derive(Deserialize, Serialize)]
struct ModListJson {
    mods: Vec<ModListJsonMod>,
}

#[derive(Deserialize, Serialize)]
struct ModListJsonMod {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
    enabled: bool,
}

#[derive(Deserialize, Debug)]
struct InfoJson {
    dependencies: Option<Vec<String>>,
    name: String,
    version: Version,
}

pub struct ModsSet {
    #[allow(unused)]
    dir: PathBuf,
    mods: HashMap<String, Mod>,
}

impl ModsSet {
    // TODO: Better error formatting so the user knows which mod threw the error
    pub fn new(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // Read mod-list.json to a file
        let mut mlj_path = path.clone();
        mlj_path.push("mod-list.json");
        let mlj_contents = std::fs::read_to_string(mlj_path)?;
        let mut enabled_versions: HashMap<String, ModEnabledType> =
            serde_json::from_str::<ModListJson>(&mlj_contents)?
                .mods
                .iter()
                .filter_map(|entry| {
                    Some((
                        entry.name.clone(),
                        match (entry.enabled, &entry.version) {
                            (true, Some(version)) => ModEnabledType::Version(version.clone()),
                            (true, None) => ModEnabledType::Latest,
                            _ => ModEnabledType::Disabled,
                        },
                    ))
                })
                .collect();

        let mut mods: HashMap<String, Mod> = HashMap::new();

        // Iterate all mods in the directory
        for entry in fs::read_dir(path)?.filter_map(|entry| {
            // Exclude mod-list.json and mod-settings.dat
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;
            if file_name != "mod-list.json" && file_name != "mod-settings.dat" {
                Some(entry)
            } else {
                None
            }
        }) {
            let path = entry.path();
            let extension = path.extension();

            // Extract info.json from the zip file or from the directory/symlink
            let info: InfoJson = if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
                // WORKAROUND: The `zip` crate doesn't have nice iterator methods, so we need to
                // early-return out of a `for` loop, necessitating a separate function
                find_info_json_in_zip(entry)
            } else {
                let file_type = entry.file_type()?;
                if file_type.is_symlink() || file_type.is_dir() {
                    // FIXME: Handle the case where there are two levels of nesting
                    let mut path = entry.path();
                    path.push("info.json");
                    let contents = fs::read_to_string(path)?;
                    let json: InfoJson = serde_json::from_str(&contents)?;
                    Ok(json)
                } else {
                    Err("Could not find an info.json file".into())
                }
            }?;

            // Retrive or create mod data
            let mod_data = mods.entry(info.name.clone()).or_insert(Mod {
                name: info.name.clone(),
                versions: vec![],
                enabled: {
                    // Move the enabled status extracted from mod-list.json into the mod object
                    let active_version = enabled_versions.remove(&info.name);
                    match active_version {
                        Some(enabled_type) => enabled_type,
                        None => ModEnabledType::Disabled,
                    }
                },
            });

            // TODO: Optimize to not parse dependencies unless we need to insert the version
            let mod_version = ModVersion {
                version: info.version,
                dependencies: info
                    .dependencies
                    .unwrap_or(vec![])
                    .iter()
                    .map(ModDependency::new)
                    .collect::<ModDependencyResult>()?,
            };

            if let Err(index) = mod_data.versions.binary_search(&mod_version) {
                mod_data.versions.insert(index, mod_version);
            }
        }

        Ok(Self {
            dir: path.clone(),
            mods,
        })
    }

    pub fn disable_all(&mut self, include_base_mod: bool) {
        println!("Disabling all mods");

        self.mods
            .iter_mut()
            .filter(|(mod_name, _)| include_base_mod || mod_name.as_str() != "base")
            .for_each(|(_, mod_data)| mod_data.enabled = ModEnabledType::Disabled);
    }

    pub fn disable(&mut self, mod_ident: &InputMod) -> Result<(), ModsSetErr> {
        println!("Disabling {}", mod_ident.name);

        let mod_data = self.get_mod(&mod_ident.name)?;

        mod_data.enabled = ModEnabledType::Disabled;

        Ok(())
    }

    pub fn enable_all(&mut self) {
        println!("Enabling latest versions of all mods");

        self.mods
            .iter_mut()
            .for_each(|(_, mod_data)| mod_data.enabled = ModEnabledType::Latest);
    }

    pub fn enable(&mut self, mod_ident: &InputMod) -> Result<(), ModsSetErr> {
        println!(
            "Enabling {}{}",
            mod_ident.name,
            match &mod_ident.version {
                ModEnabledType::Version(version) => format!(" v{}", version),
                _ => "".to_string(),
            }
        );

        let mod_data = self.get_mod(&mod_ident.name)?;

        mod_data.enabled = match &mod_ident.version {
            ModEnabledType::Version(version) => {
                if mod_data
                    .versions
                    .binary_search_by(|stored_version| stored_version.version.cmp(version))
                    .is_ok()
                {
                    // TODO: Remove clone?
                    Ok(ModEnabledType::Version(version.clone()))
                } else {
                    Err(ModsSetErr::ModVersionDoesNotExist(version.clone()))
                }
            }
            _ => Ok(ModEnabledType::Latest),
        }?;

        // TODO: Enable dependencies

        Ok(())
    }

    pub fn write(&mut self) -> Result<(), ModsSetErr> {
        let info = ModListJson {
            mods: self
                .mods
                .iter()
                .map(|(_, mod_data)| {
                    let (enabled, version) = match &mod_data.enabled {
                        ModEnabledType::Disabled => (false, None),
                        ModEnabledType::Latest => (true, None),
                        ModEnabledType::Version(version) => (true, Some(version.clone())),
                    };
                    ModListJsonMod {
                        enabled,
                        name: mod_data.name.clone(),
                        version,
                    }
                })
                .collect(),
        };

        let mut path = self.dir.clone();
        path.push("mod-list.json");
        fs::write(
            path,
            serde_json::to_string_pretty(&info).map_err(|_| ModsSetErr::CouldNotWriteChanges)?,
        )
        .map_err(|_| ModsSetErr::CouldNotWriteChanges)?;

        Ok(())
    }

    fn get_mod(&mut self, mod_name: &str) -> Result<&mut Mod, ModsSetErr> {
        self.mods
            .get_mut(mod_name)
            .ok_or(ModsSetErr::ModDoesNotExist)
    }
}

pub enum ModsSetErr {
    CouldNotWriteChanges,
    ModDoesNotExist,
    ModVersionDoesNotExist(Version),
}

impl fmt::Display for ModsSetErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::CouldNotWriteChanges => "Could not write changes".to_string(),
                Self::ModDoesNotExist => "Mod does not exist".to_string(),
                Self::ModVersionDoesNotExist(version) =>
                    format!("Version {} does not exist", version.to_string()),
            }
        )
    }
}

impl fmt::Debug for ModsSetErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl Error for ModsSetErr {}

fn find_info_json_in_zip(entry: DirEntry) -> Result<InfoJson, Box<dyn Error>> {
    let file = File::open(entry.path())?;
    // My hand is forced due to the lack of a proper iterator API in the `zip` crate
    let mut archive = ZipArchive::new(file)?;
    // Thus, we need to use a bare `for` loop and iterate the indices, then act on the file if we find it
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().contains("info.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            // FIXME: Doesn't work with special characters
            let json: InfoJson = serde_json::from_str(&contents)?;
            return Ok(json);
        }
    }
    Err("Mod ZIP does not contain an info.json file".into())
}

#[derive(Debug)]
struct Mod {
    name: String,
    versions: Vec<ModVersion>,
    enabled: ModEnabledType,
}

#[derive(Debug)]
pub enum ModEnabledType {
    Disabled,
    Latest,
    Version(Version),
}

#[derive(Debug)]
struct ModVersion {
    version: Version,
    // TODO: Use a HashSet for quick lookup?
    dependencies: Vec<ModDependency>,
}

impl PartialOrd for ModVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

// impl PartialOrd<Version> for ModVersion {
//     fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
//         self.version.partial_cmp(other)
//     }
// }

impl Ord for ModVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialEq for ModVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

// impl PartialEq<Version> for ModVersion {
//     fn eq(&self, other: &Version) -> bool {
//         self.version == *other
//     }
// }

impl Eq for ModVersion {}
