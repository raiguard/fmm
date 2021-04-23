use crate::dependency::ModDependency;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{cmp::Ordering, collections::HashMap};
use zip::read::ZipArchive;

#[derive(Debug)]
pub struct ModsDirectory {
    pub mods: HashMap<String, Mod>,
    pub path: PathBuf,
}

impl ModsDirectory {
    pub fn new(directory: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut mod_list_json = String::new();
        let mut mods: HashMap<String, Mod> = HashMap::new();

        // Iterate files and directories to assemble mods
        let entries = fs::read_dir(&directory)?;
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_name() == "mod-list.json" {
                    let mut file = File::open(entry.path())?;
                    file.read_to_string(&mut mod_list_json)?;
                } else if entry.file_name() != "mod-settings.dat" {
                    if let Ok(json) = read_info_json(&entry) {
                        let mod_entry = mods.entry(json.name.clone()).or_insert(Mod {
                            name: json.name,
                            versions: vec![],
                            enabled: ModEnabledType::Disabled,
                        });
                        let version = ModVersion {
                            dependencies: if let Some(dependencies) = json.dependencies {
                                if let Ok(dependencies) = parse_dependencies(dependencies) {
                                    Some(dependencies)
                                } else {
                                    None
                                }
                            } else {
                                None
                            },
                            dir_entry: entry,
                            version: json.version,
                        };
                        mod_entry.versions.insert(
                            mod_entry.versions.binary_search(&version).unwrap_err(),
                            version,
                        );
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

    pub fn write(&self) -> Result<(), Box<dyn Error>> {
        let mods: Vec<ModsListJsonMod> = self
            .mods
            .iter()
            .map(|(_, mod_data)| ModsListJsonMod {
                enabled: match &mod_data.enabled {
                    ModEnabledType::Disabled => false,
                    _ => true,
                },
                name: mod_data.name.clone(),
                version: match &mod_data.enabled {
                    ModEnabledType::Version(version) => Some(version.clone()),
                    _ => None,
                },
            })
            .collect();

        let mut path = self.path.clone();
        path.push("mod-list.json");
        fs::write(path, serde_json::to_string_pretty(&ModsListJson { mods })?)?;

        Ok(())
    }

    pub fn disable_all(&mut self, disable_base: bool) {
        println!("Disabled all mods");
        for (_, mod_data) in self.mods.iter_mut() {
            if disable_base || mod_data.name != "base" {
                mod_data.enabled = ModEnabledType::Disabled
            }
        }
    }

    pub fn enable_all(&mut self) {
        println!("Enabled all mods");
        for (_, mod_data) in self.mods.iter_mut() {
            mod_data.enabled = ModEnabledType::Latest
        }
    }

    pub fn toggle_mod(
        &mut self,
        mod_data: &crate::input::ModInputData,
        to_state: bool,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(mod_entry) = self.mods.get_mut(&mod_data.name) {
            mod_entry.enabled = if to_state {
                match &mod_data.version {
                    Some(version) => {
                        println!("Enabled {} v{}", mod_data.name, version);
                        // TODO: Remove clone?
                        ModEnabledType::Version(version.clone())
                    }
                    None => {
                        println!("Enabled {}", mod_data.name);
                        ModEnabledType::Latest
                    }
                }
            } else {
                println!("Disabled {}", mod_data.name);
                ModEnabledType::Disabled
            };

            Ok(())
        } else {
            return Err(format!("Mod `{}` does not exist", mod_data.name).into());
        }
    }

    pub fn dedup(&mut self) -> Result<(), Box<dyn Error>> {
        for (_, mod_data) in &mut self.mods {
            if mod_data.versions.len() > 1 {
                let mod_name = &mod_data.name;
                for version in mod_data.versions.drain(..(mod_data.versions.len() - 1)) {
                    let entry = version.dir_entry;
                    if let Ok(metadata) = entry.metadata() {
                        // This can be inlined to the second `if`, but it's less readable
                        let res = if metadata.is_dir() {
                            fs::remove_dir_all(entry.path())
                        } else {
                            fs::remove_file(entry.path())
                        };
                        if res.is_ok() {
                            println!("Deleted {} v{}", mod_name, version.version);
                        } else {
                            eprintln!("Could not delete {} v{}", mod_name, version.version);
                        }
                    } else {
                        eprintln!(
                            "Could not get metadata for {} v{}",
                            mod_name, version.version
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Mod {
    pub name: String,
    pub versions: Vec<ModVersion>,
    pub enabled: ModEnabledType,
}

#[derive(Debug)]
pub enum ModEnabledType {
    Disabled,
    Latest,
    Version(Version),
}

#[derive(Debug)]
pub struct ModVersion {
    pub dependencies: Option<Vec<ModDependency>>,
    pub dir_entry: DirEntry,
    pub version: Version,
}

impl PartialOrd for ModVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

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

impl Eq for ModVersion {}

#[derive(Deserialize, Debug)]
struct InfoJson {
    dependencies: Option<Vec<String>>,
    name: String,
    version: Version,
}

#[derive(Serialize, Deserialize)]
struct ModsListJson {
    mods: Vec<ModsListJsonMod>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModsListJsonMod {
    enabled: bool,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
}

fn read_info_json(entry: &DirEntry) -> Result<InfoJson, Box<dyn Error>> {
    let metadata = entry.metadata()?;
    if metadata.is_dir() || metadata.file_type().is_symlink() {
        let mut path = entry.path();
        path.push("info.json");
        let contents = fs::read_to_string(path)?;
        let json: InfoJson = serde_json::from_str(&contents)?;
        Ok(json)
    } else if Some(OsStr::new("zip")) == Path::new(&entry.file_name()).extension() {
        let file = File::open(entry.path())?;
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

fn parse_dependencies(dependencies: Vec<String>) -> Result<Vec<ModDependency>, Box<dyn Error>> {
    let mut output = vec![];
    for dependency in dependencies {
        output.push(ModDependency::new(&dependency)?);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::ModDependencyType;
    use semver::VersionReq;

    fn tests_path(suffix: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push(suffix);
        d
    }

    #[test]
    fn mods_directory() {
        // let directory = tests_path("mods_dir_1");
        let directory = PathBuf::from("/home/rai/.factorio/mods");

        ModsDirectory::new(directory).unwrap();
    }

    #[test]
    fn dependency_regex() {
        // TODO: Error case and other formats
        let sets = vec![
            (
                "! bobs logistics >= 0.17.3",
                ModDependency {
                    dep_type: ModDependencyType::Incompatible,
                    name: "bobs logistics".to_string(),
                    version_req: Some(VersionReq::parse(">= 0.17.3").unwrap()),
                },
            ),
            (
                "? RecipeBook = 0.15",
                ModDependency {
                    dep_type: ModDependencyType::Optional,
                    name: "RecipeBook".to_string(),
                    version_req: Some(VersionReq::parse("0.15").unwrap()),
                },
            ),
            (
                "fufucuddlypoops",
                ModDependency {
                    dep_type: ModDependencyType::Required,
                    name: "fufucuddlypoops".to_string(),
                    version_req: None,
                },
            ),
        ];

        for set in sets {
            assert_eq!(ModDependency::new(set.0).unwrap(), set.1);
        }
    }

    // TODO: Test for dedup
}
