use crate::dat::PropertyTree;
use crate::dependency::ModDependency;
use crate::mod_settings::ModSettings;
use crate::version::VersionReq;
use crate::{HasDependencies, HasReleases, HasVersion, ModIdent, Version};
use anyhow::{anyhow, bail, ensure, Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug)]
pub struct Directory {
    mods: HashMap<String, DirMod>,
    list: EnabledList,
    pub settings: ModSettings,
}

impl Directory {
    /// Constructs the object from the given mods directory
    pub fn new(path: &Path) -> Result<Self> {
        // Check for mod-list.json and mod-settings.dat
        ensure!(
            path.join("mod-list.json").exists() && path.join("mod-settings.dat").exists(),
            format!("Invalid mods directory: {:?}", path)
        );

        // Assemble mod entries
        let mut mods = HashMap::new();
        for entry in fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name() != "mod-list.json" && entry.file_name() != "mod-settings.dat"
            })
        {
            match InfoJson::from_entry(&entry)
                .context(format!("Could not parse {:?}", entry.file_name()))
            {
                Ok(info_json) => {
                    let ident = ModIdent {
                        name: info_json.name.clone(),
                        version: Some(info_json.version.clone()),
                    };

                    let mod_entry = mods.entry(ident.name.clone()).or_insert(DirMod {
                        name: ident.name.clone(),
                        releases: vec![],
                    });

                    let release = DirModRelease {
                        entry,
                        ident,
                        dependencies: info_json.dependencies,
                    };

                    let index = mod_entry
                        .releases
                        .binary_search(&release)
                        .unwrap_or_else(|index| index);
                    mod_entry.releases.insert(index, release);
                }
                Err(e) => eprintln!("{} {}", style("Error:").red().bold(), e),
            }
        }

        // Parse mod-list.json
        let mlj_path = path.join("mod-list.json");
        let enabled_versions = fs::read_to_string(&mlj_path)?;
        let mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

        Ok(Self {
            mods,
            list: EnabledList {
                mods: mod_list_json.mods,
                path: mlj_path,
            },
            settings: ModSettings::new(path)?,
        })
    }

    /// Adds the mod, but keeps it disabled
    pub fn add(&mut self, ident: ModIdent, entry: DirEntry) {
        if self.list.get(&ident).is_none() {
            self.list.add(&ident);
        }

        let mod_data = self
            .mods
            .entry(ident.name.clone())
            .or_insert_with(|| DirMod {
                name: ident.name.clone(),
                releases: vec![],
            });
        let release = DirModRelease {
            entry,
            ident,
            dependencies: None,
        };
        if let Err(index) = mod_data.releases.binary_search(&release) {
            mod_data.releases.insert(index, release);
        }
    }

    pub fn contains(&self, ident: &ModIdent) -> bool {
        self.mods
            .get(&ident.name)
            .map(|mod_data| mod_data.get_release(ident).is_some())
            .unwrap_or(false)
    }

    pub fn disable(&mut self, ident: &ModIdent) {
        if ident.name == "base" || self.mods.contains_key(&ident.name) {
            self.list.disable(ident);
        } else {
            // TODO: Centralize printing
            eprintln!("Could not find {}", &ident);
        }
    }

    pub fn disable_all(&mut self) {
        self.list.disable_all();
    }

    pub fn enable(&mut self, ident: &ModIdent) -> Result<()> {
        if self
            .mods
            .get(&ident.name)
            .and_then(|mod_data| mod_data.get_release(ident))
            .is_none()
        {
            bail!("{} is not in the local mods directory", ident);
        }

        if self.list.get(ident).is_none() {
            self.list.add(ident);
        }
        self.list.enable(ident)?;

        Ok(())
    }

    pub fn get(&self, ident: &ModIdent) -> Option<&DirMod> {
        self.mods.get(&ident.name)
    }

    pub fn get_mut(&mut self, ident: &ModIdent) -> Option<&mut DirMod> {
        self.mods.get_mut(&ident.name)
    }

    pub fn get_newest_matching(&self, dependency: &ModDependency) -> Option<&DirModRelease> {
        self.mods.get(&dependency.name).and_then(|mod_data| {
            // TODO: This is extremely similar to the HasReleases trait method
            if let Some(version_req) = &dependency.version_req {
                mod_data
                    .releases
                    .iter()
                    .rev()
                    .find(|release| version_req.matches(release.get_version()))
            } else {
                mod_data.releases.last()
            }
        })
    }

    pub fn save(&self) -> Result<()> {
        self.list.save()?;
        self.settings.write()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct DirMod {
    name: String,
    releases: Vec<DirModRelease>,
}

impl HasReleases<DirModRelease> for DirMod {
    fn get_release_list(&self) -> &[DirModRelease] {
        &self.releases
    }
}

#[derive(Debug)]
pub struct DirModRelease {
    pub entry: DirEntry,
    // This is always guaranteed to have a version
    pub ident: ModIdent,

    pub dependencies: Option<Vec<ModDependency>>,
}

impl HasDependencies for DirModRelease {
    fn get_dependencies(&self) -> Option<&Vec<ModDependency>> {
        self.dependencies.as_ref()
    }
}

impl HasVersion for DirModRelease {
    fn get_version(&self) -> &Version {
        self.ident.get_guaranteed_version()
    }
}

impl PartialOrd for DirModRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ident
            .get_guaranteed_version()
            .partial_cmp(other.ident.get_guaranteed_version())
    }
}

impl Ord for DirModRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ident
            .get_guaranteed_version()
            .cmp(other.ident.get_guaranteed_version())
    }
}

impl PartialEq for DirModRelease {
    fn eq(&self, other: &Self) -> bool {
        self.ident.get_guaranteed_version() == other.ident.get_guaranteed_version()
    }
}

impl Eq for DirModRelease {}

#[derive(Debug)]
enum DirModReleaseType {
    Directory,
    Symlink,
    Zip,
}

impl DirModReleaseType {
    fn parse(entry: &DirEntry) -> Result<Self> {
        let path = entry.path();
        let extension = path.extension();

        if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
            return Ok(DirModReleaseType::Zip);
        } else {
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Ok(DirModReleaseType::Symlink);
            } else {
                let mut path = entry.path();
                path.push("info.json");
                if path.exists() {
                    return Ok(DirModReleaseType::Directory);
                }
            }
        };

        bail!("Could not parse mod entry structure");
    }
}

#[derive(Deserialize, Debug)]
struct InfoJson {
    dependencies: Option<Vec<ModDependency>>,
    name: String,
    version: Version,
}

impl InfoJson {
    fn from_entry(entry: &DirEntry) -> Result<Self> {
        // TODO: Store the structure in the entry for later use
        let contents = match DirModReleaseType::parse(entry)? {
            DirModReleaseType::Directory | DirModReleaseType::Symlink => {
                let mut path = entry.path();
                path.push("info.json");
                fs::read_to_string(path)?
            }
            DirModReleaseType::Zip => {
                let mut archive = ZipArchive::new(File::open(entry.path())?)?;
                let filename = archive
                    .file_names()
                    .find(|name| name.contains("info.json"))
                    .map(ToString::to_string)
                    .ok_or_else(|| anyhow!("Could not locate info.json in zip archive"))?;
                let mut file = archive.by_name(&filename)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                contents
            }
        };

        serde_json::from_str::<InfoJson>(&contents).map_err(|_| anyhow!("Invalid info.json format"))
    }
}

#[derive(Debug)]
struct EnabledList {
    mods: Vec<ModListJsonMod>,
    path: PathBuf,
}

impl EnabledList {
    fn add(&mut self, ident: &ModIdent) -> &ModListJsonMod {
        self.mods.push(ModListJsonMod {
            name: ident.name.clone(),
            version: ident.version.clone(),
            enabled: false,
        });
        self.mods.last().unwrap()
    }

    fn get(&self, ident: &ModIdent) -> Option<&ModListJsonMod> {
        self.mods
            .iter()
            .find(|mod_state| ident.name == mod_state.name)
    }

    fn get_mut(&mut self, ident: &ModIdent) -> Option<&mut ModListJsonMod> {
        self.mods
            .iter_mut()
            .find(|mod_state| ident.name == mod_state.name)
    }

    fn disable(&mut self, ident: &ModIdent) {
        if let Some(mod_state) = self.get_mut(ident) {
            mod_state.enabled = false;
            mod_state.version = None;

            println!("{} {}", style("Disabled").yellow().bold(), &ident);
        }
    }

    fn disable_all(&mut self) {
        self.mods
            .iter_mut()
            .filter(|entry| entry.name != "base")
            .for_each(|entry| {
                entry.enabled = false;
                entry.version = None;
            });

        println!("{}", style("Disabled all mods").yellow().bold());
    }

    fn enable(&mut self, ident: &ModIdent) -> Result<()> {
        let mod_data = self
            .get_mut(ident)
            .ok_or_else(|| anyhow!("{} is not present in mod-list.json", ident))?;

        mod_data.enabled = true;
        mod_data.version = ident.version.clone();

        println!("{} {}", style("Enabled").green().bold(), ident);

        Ok(())
    }

    fn save(&self) -> Result<()> {
        fs::write(
            &self.path,
            serde_json::to_string_pretty(&ModListJson {
                mods: self.mods.clone(),
            })?,
        )?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct ModListJson {
    pub mods: Vec<ModListJsonMod>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ModListJsonMod {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
    enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_init() {
        let dir = Directory::new(&std::env::current_dir().unwrap().join("test-mods")).unwrap();

        println!("{:#?}", dir);
    }
}
