use semver::{Version, VersionReq};
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
use structopt::StructOpt;
use zip::ZipArchive;

mod dependency;

use crate::dependency::{ModDependency, ModDependencyResult};

// TODO: Figure out why it's not coloring the help info.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "fmm",
    about = "Enable, disable, download, update, create, and delete Factorio mods."
)]
struct App {
    /// A list of mods to disable. TODO: explain format.
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    /// The path to the mods directory
    // TODO: Make optional, introduce config file to specify default path
    #[structopt(short = "f", long)]
    dir: PathBuf,
    /// A list of mods to enable. TODO: explain format.
    #[structopt(short, long)]
    enable: Vec<InputMod>,
}

#[derive(Debug)]
struct InputMod {
    name: String,
    version: InputModVersion,
}

impl std::str::FromStr for InputMod {
    type Err = InputModErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: InputModVersion::Latest,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    // Validate that the version does *not* have prerelease or build data
                    if version.pre.len() > 0 || version.build.len() > 0 {
                        Err(Self::Err::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: InputModVersion::Version(version),
                        })
                    }
                } else {
                    Err(Self::Err::InvalidVersion(version.to_string()))
                }
            }
            _ => Err(Self::Err::IncorrectArgCount(parts.len())),
        }
    }
}

#[derive(Debug)]
enum InputModErr {
    IncorrectArgCount(usize),
    InvalidVersion(String),
}

impl fmt::Display for InputModErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IncorrectArgCount(arg_count) => format!(
                    "Incorrect argument count: Expected 1 or 2, got {}",
                    arg_count
                ),
                Self::InvalidVersion(got_version) =>
                    format!("Invalid version identifier: {}", got_version),
            }
        )
    }
}

#[derive(Debug)]
enum InputModVersion {
    Latest,
    Version(Version),
}

struct ModsSet {
    dir: PathBuf,
    mods: HashMap<String, Mod>,
}

#[derive(Deserialize, Serialize)]
struct ModListJson {
    mods: Vec<ModListJsonMod>,
}

#[derive(Deserialize, Serialize)]
struct ModListJsonMod {
    name: String,
    version: Option<Version>,
    enabled: bool,
}

#[derive(Deserialize, Debug)]
struct InfoJson {
    dependencies: Option<Vec<String>>,
    name: String,
    version: Version,
}

impl ModsSet {
    // TODO: Better error formatting so the user knows which mod threw the error
    pub fn new(path: &PathBuf) -> Result<(), Box<dyn Error>> {
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

        println!("{:#?}", mods);

        Ok(())
    }
}

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
enum ModEnabledType {
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

// TODO: Might not end up being used
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

// TODO: Might not end up being used
impl PartialEq<Version> for ModVersion {
    fn eq(&self, other: &Version) -> bool {
        self.version == *other
    }
}

impl Eq for ModVersion {}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    let set = ModsSet::new(&app.dir)?;

    Ok(())
}
