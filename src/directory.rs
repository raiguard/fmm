use crate::dependency::ModDependency;
use crate::mod_settings::ModSettings;
use crate::{HasDependencies, HasReleases, HasVersion, ModIdent, Version};
use anyhow::{anyhow, bail, ensure, Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
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
            let mod_path = entry.path();
            match InfoJson::from_entry(&mod_path)
                .context(format!("Could not parse {:?}", entry.file_name()))
            {
                Ok(info_json) => {
                    let ident = ModIdent {
                        name: info_json.name.clone(),
                        version: Some(info_json.version.clone()),
                    };

                    let mod_entry = mods
                        .entry(ident.name.clone())
                        .or_insert(DirMod { releases: vec![] });

                    let release = DirModRelease {
                        path: mod_path,
                        ident,
                        dependencies: info_json.dependencies,
                    };

                    let index = mod_entry
                        .releases
                        .binary_search(&release)
                        .unwrap_or_else(|index| index);
                    mod_entry.releases.insert(index, release);
                }
                Err(e) => eprintln!("{} {:#}", style("Error:").red().bold(), e),
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
    pub fn add(&mut self, ident: ModIdent, path: PathBuf) {
        if self.list.get(&ident).is_none() {
            self.list.add(&ident);
        }

        let mod_data = self
            .mods
            .entry(ident.name.clone())
            .or_insert_with(|| DirMod { releases: vec![] });
        let release = DirModRelease {
            path,
            ident,
            dependencies: None,
        };
        if let Err(index) = mod_data.releases.binary_search(&release) {
            mod_data.releases.insert(index, release);
        }
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

    pub fn get_newest_matching(&self, dependency: &ModDependency) -> Option<&DirModRelease> {
        self.mods.get(&dependency.name).and_then(|mod_data| {
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

    pub fn remove(&mut self, ident: &ModIdent) -> Result<()> {
        let mod_data = self
            .mods
            .get_mut(&ident.name)
            .ok_or_else(|| anyhow!("{} not found in mods directory", ident))?;

        let release = mod_data
            .get_release(ident)
            .ok_or_else(|| anyhow!("{} not found in mods directory", ident))?;

        fs::remove_file(&release.path)?;
        mod_data.remove_release(ident)?;

        if mod_data.get_release_list().is_empty() {
            self.mods.remove(&ident.name);
            self.list.remove(ident);
        }

        println!("{} {}", style("Removed").magenta().bold(), ident);

        Ok(())
    }
}

#[derive(Debug)]
pub struct DirMod {
    releases: Vec<DirModRelease>,
}

impl HasReleases<DirModRelease> for DirMod {
    fn get_release_list(&self) -> &Vec<DirModRelease> {
        &self.releases
    }

    fn get_release_list_mut(&mut self) -> &mut Vec<DirModRelease> {
        &mut self.releases
    }
}

#[derive(Debug)]
pub struct DirModRelease {
    pub path: PathBuf,
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
    fn parse(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let extension = path.extension();

        if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
            return Ok(DirModReleaseType::Zip);
        } else {
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                return Ok(DirModReleaseType::Symlink);
            } else if path.join("info.json").exists() {
                return Ok(DirModReleaseType::Directory);
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
    fn from_entry(path: &Path) -> Result<Self> {
        let contents = match DirModReleaseType::parse(path)? {
            DirModReleaseType::Directory | DirModReleaseType::Symlink => {
                fs::read(path.join("info.json"))
            }
            DirModReleaseType::Zip => {
                let mut archive = ZipArchive::new(File::open(path)?)?;
                let filename = archive
                    .file_names()
                    .find(|name| name.contains("info.json"))
                    .map(ToString::to_string)
                    .ok_or_else(|| anyhow!("Could not locate info.json in zip archive"))?;
                let bytes = archive.by_name(&filename)?.bytes().collect();
                bytes
            }
        }?;

        serde_json::from_slice::<InfoJson>(&contents)
            .map_err(|_| anyhow!("Invalid info.json format"))
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

    fn remove(&mut self, ident: &ModIdent) {
        if let Some((index, _)) = self
            .mods
            .iter_mut()
            .enumerate()
            .find(|(_, mod_data)| mod_data.name == ident.name)
        {
            self.mods.remove(index);
        }
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
