use once_cell::sync::OnceCell;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{cmp::Ordering, collections::HashSet};
use thiserror::Error;
use zip::ZipArchive;

use crate::dependency::{ModDependency, ModDependencyResult, ModDependencyType};
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
    dir: PathBuf,
    mods: HashMap<String, Mod>,
}

impl ModsSet {
    // TODO: Better error formatting so the user knows which mod threw the error
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        // Read mod-list.json to a file
        let mut mlj_path = path.to_owned();
        mlj_path.push("mod-list.json");
        let mlj_contents = std::fs::read_to_string(mlj_path)?;
        let mut enabled_versions: HashMap<String, ModEnabledType> =
            serde_json::from_str::<ModListJson>(&mlj_contents)?
                .mods
                .iter()
                .map(|entry| {
                    (
                        entry.name.clone(),
                        match (entry.enabled, &entry.version) {
                            (true, Some(version)) => ModEnabledType::Version(version.clone()),
                            (true, None) => ModEnabledType::Latest,
                            _ => ModEnabledType::Disabled,
                        },
                    )
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
            // Determine the mod's structure type
            let mod_structure = ModVersionStructure::parse(&entry)?;

            // Extract contents of info.json file
            let info = match mod_structure {
                ModVersionStructure::Zip => {
                    // WORKAROUND: The `zip` crate doesn't have nice iterator methods, so we need to
                    // early-return out of a `for` loop, necessitating a separate function
                    find_info_json_in_zip(&entry)?
                }
                _ => {
                    let mut path = entry.path();
                    path.push("info.json");
                    fs::read_to_string(path)?
                }
            };

            // Remove all non-UTF8 characters from the string
            let info = info.replace(|c: char| !c.is_ascii(), "");

            // Parse info string into a struct
            match serde_json::from_str::<InfoJson>(&info) {
                Ok(info) => {
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
                        entry,
                        dependencies: info
                            .dependencies
                            .unwrap_or_default()
                            .iter()
                            .map(|dep| ModDependency::new(&dep))
                            .collect::<ModDependencyResult>()?,
                        structure: mod_structure,
                        version: info.version,
                    };

                    if let Err(index) = mod_data.versions.binary_search(&mod_version) {
                        mod_data.versions.insert(index, mod_version);
                    }
                }
                Err(_) => {
                    if let Some(file_name) = entry.file_name().to_owned().to_str() {
                        println!("Could not read mod: {}", file_name);
                        let file_name = file_name.replace(|c: char| !c.is_ascii(), "");
                        // Avoid creating the regex object every time
                        static FILENAME_REGEX: OnceCell<Regex> = OnceCell::new();
                        let captures = FILENAME_REGEX
                            .get_or_init(|| {
                                Regex::new(r"^(?P<name>.*)_(?P<version>\d*\.\d*\.\d*\.)$").unwrap()
                            })
                            .captures(&file_name)
                            .ok_or(ModsSetErr::ModFilenameUnreadable);
                        // TODO: Keep a list of invalid mods
                        // Perhaps we need to re-think the arcitecture to keep active versions and the actual versions separate
                    }
                }
            }
        }

        Ok(Self {
            dir: path.to_owned(),
            mods,
        })
    }

    pub fn dedup(&mut self) -> Result<(), ModsSetErr> {
        println!("Deduplicating zipped mod versions");

        for (_, mod_data) in self.mods.iter_mut() {
            for version_data in mod_data.versions.drain(..(mod_data.versions.len() - 1)) {
                if let ModVersionStructure::Zip = version_data.structure {
                    // TODO: Print the removal
                    version_data.remove_from_disk()?;
                }
            }
        }

        Ok(())
    }

    pub fn disable_all(&mut self, include_base_mod: bool) {
        println!("Disabling all mods");

        self.mods
            .iter_mut()
            .filter(|(mod_name, _)| include_base_mod || mod_name.as_str() != "base")
            .for_each(|(_, mod_data)| mod_data.enabled = ModEnabledType::Disabled);
    }

    pub fn disable(&mut self, mod_ident: &InputMod) -> Result<(), ModsSetErr> {
        println!("Disabling {}", mod_ident);

        let mod_data = self.get_mod_mut(&mod_ident.name)?;

        mod_data.enabled = ModEnabledType::Disabled;

        Ok(())
    }

    pub fn enable_all(&mut self) {
        println!("Enabling latest versions of all mods");

        self.mods
            .iter_mut()
            .for_each(|(_, mod_data)| mod_data.enabled = ModEnabledType::Latest);
    }

    pub fn enable(&mut self, mod_ident: &InputMod) -> Result<Vec<InputMod>, ModsSetErr> {
        println!("Enabling {}", mod_ident);

        let mod_data = self.get_mod_mut(&mod_ident.name)?;

        // Enable this mod
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

        // Return a list of dependencies to enable
        let mod_data = self.get_mod(&mod_ident.name)?;
        let active_version = mod_data.get_active_version()?.unwrap();
        active_version
            .dependencies
            .iter()
            .filter(|dependency_ident| {
                dependency_ident.name != "base"
                    && matches!(
                        dependency_ident.dep_type,
                        ModDependencyType::NoLoadOrder | ModDependencyType::Required
                    )
            })
            .map(|dependency_ident| {
                let dependency = self
                    .get_mod(&dependency_ident.name)
                    // TODO: More explanative errors
                    .map_err(|_| ModsSetErr::MatchingDependencyNotFound)?;
                let version = if dependency_ident.version_req.is_none() {
                    dependency.versions.last()
                } else {
                    let req = dependency_ident.version_req.as_ref().unwrap();
                    dependency
                        .versions
                        .iter()
                        .rev()
                        .find(|version| req.matches(&version.version))
                };

                if version.is_none() {
                    return Err(ModsSetErr::MatchingDependencyNotFound);
                }
                Ok(InputMod {
                    name: dependency.name.clone(),
                    version: ModEnabledType::Version(version.unwrap().version.clone()),
                })
            })
            .collect::<Result<Vec<InputMod>, ModsSetErr>>()
    }

    pub fn enable_list(&mut self, initial_to_enable: Vec<InputMod>) -> Result<(), ModsSetErr> {
        let mut to_enable = initial_to_enable;
        let mut did_enable: HashSet<String> = HashSet::new();

        while !to_enable.is_empty() {
            let mut to_enable_next = Vec::new();
            // Enable all of the mods
            for mod_ident in &to_enable {
                if did_enable.get(&mod_ident.name).is_none() {
                    to_enable_next.append(&mut self.enable(&mod_ident)?);
                    did_enable.insert(mod_ident.name.clone());
                }
            }
            // Replace `to_enable`
            to_enable = to_enable_next;
        }
        Ok(())
    }

    pub fn remove(&mut self, mod_ident: &InputMod) -> Result<(), ModsSetErr> {
        println!("Removing {}", mod_ident);

        let mod_data = self.get_mod_mut(&mod_ident.name)?;

        // Extract the matching version from the versions table
        let version_data = mod_data
            .versions
            .remove(mod_data.find_version(&mod_ident.version)?);

        version_data.remove_from_disk()?;

        // TODO: Remove mod entry if there are no more versions

        Ok(())
    }

    pub fn write_mod_list(&mut self) -> Result<(), ModsSetErr> {
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

    fn get_mod_mut(&mut self, mod_name: &str) -> Result<&mut Mod, ModsSetErr> {
        self.mods
            .get_mut(mod_name)
            .ok_or(ModsSetErr::ModDoesNotExist)
    }

    fn get_mod(&self, mod_name: &str) -> Result<&Mod, ModsSetErr> {
        self.mods.get(mod_name).ok_or(ModsSetErr::ModDoesNotExist)
    }
}

#[derive(Debug, Error)]
pub enum ModsSetErr {
    #[error("Could not remove version {0}")]
    CouldNotRemoveVersion(Version),
    #[error("Could not write changes to mod-list.json")]
    CouldNotWriteChanges,
    #[error("Filesystem error")]
    FilesystemError,
    #[error("Invalid mod structure")]
    InvalidModStructure,
    #[error("Matching dependency not found")]
    MatchingDependencyNotFound,
    #[error("Mod does not exist")]
    ModDoesNotExist,
    #[error("Version {0} does not exist")]
    ModVersionDoesNotExist(Version),
    #[error("Could not read mod file name {0}")]
    ModFilenameUnreadable(String),
}

// The `zip` crate doesn't have proper iterator methods, so we must use a bare `for` loop and early return
fn find_info_json_in_zip(entry: &DirEntry) -> Result<String, Box<dyn Error>> {
    let file = File::open(entry.path())?;
    let mut archive = ZipArchive::new(file)?;
    // Thus, we need to use a bare `for` loop and iterate the indices, then act on the file if we find it
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().contains("info.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            return Ok(contents);
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

impl Mod {
    fn find_version(&self, version_ident: &ModEnabledType) -> Result<usize, ModsSetErr> {
        match version_ident {
            ModEnabledType::Version(version) => self
                .versions
                .binary_search_by(|stored_version| stored_version.version.cmp(version))
                .map_err(|_| ModsSetErr::ModVersionDoesNotExist(version.clone())),
            _ => Ok(self.versions.len() - 1),
        }
    }

    fn get_active_version(&self) -> Result<Option<&ModVersion>, ModsSetErr> {
        Ok(self.versions.get(self.find_version(&self.enabled)?))
    }
}

#[derive(Clone, Debug)]
pub enum ModEnabledType {
    Disabled,
    Latest,
    Version(Version),
}

#[derive(Debug)]
struct ModVersion {
    entry: DirEntry,
    // TODO: Use a HashSet for quick lookup?
    dependencies: Vec<ModDependency>,
    structure: ModVersionStructure,
    version: Version,
}

impl ModVersion {
    fn remove_from_disk(&self) -> Result<(), ModsSetErr> {
        let entry = &self.entry;

        if entry
            .metadata()
            .map_err(|_| ModsSetErr::FilesystemError)?
            .is_dir()
        {
            fs::remove_dir_all(entry.path())
        } else {
            fs::remove_file(entry.path())
        }
        .map_err(|_| ModsSetErr::CouldNotRemoveVersion(self.version.clone()))
    }
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

#[derive(Debug)]
enum ModVersionStructure {
    Directory,
    Invalid,
    Symlink,
    Zip,
}

impl ModVersionStructure {
    fn parse(entry: &DirEntry) -> Result<Self, ModsSetErr> {
        let path = entry.path();
        let extension = path.extension();

        if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
            return Ok(ModVersionStructure::Zip);
        } else {
            let file_type = entry.file_type().map_err(|_| ModsSetErr::FilesystemError)?;
            if file_type.is_symlink() {
                return Ok(ModVersionStructure::Symlink);
            } else {
                let mut path = entry.path();
                path.push("info.json");
                if path.exists() {
                    return Ok(ModVersionStructure::Directory);
                }
            }
        };

        Err(ModsSetErr::InvalidModStructure)
    }
}
