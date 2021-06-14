use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
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
    // NonexistentMod(String),
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
                // Self::NonexistentMod(mod_name) => format!("Mod `{}` does not exist", mod_name),
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
    pub fn new(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        // // Read mod-list.json to a file
        // let mut mlj_path = path.clone();
        // mlj_path.push("mod-list.json");
        // let mlj_contents = std::fs::read_to_string(mlj_path)?;
        // let mod_list_json: ModListJson = serde_json::from_str(&mlj_contents)?;

        // Do one thing if it's a symlink or a directory, and another if it's a zip file
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let extension = path.extension();
            let info: InfoJson = if extension.is_some() && extension.unwrap() == OsStr::new("zip") {
                find_info_json_in_zip(entry)
            } else {
                let file_type = entry.file_type()?;
                if file_type.is_symlink() || file_type.is_dir() {
                    let mut path = entry.path();
                    path.push("info.json");
                    let contents = fs::read_to_string(path)?;
                    let json: InfoJson = serde_json::from_str(&contents)?;
                    Ok(json)
                } else {
                    Err("Could not find an info.json file".into())
                }
            }?;
            println!("{:#?}", info);
        }

        Ok(())
    }
}

fn find_info_json_in_zip(entry: DirEntry) -> Result<InfoJson, Box<dyn Error>> {
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
}

struct Mod {
    name: String,
    versions: Vec<ModVersion>,
    enabled: ModEnabledType,
}

enum ModEnabledType {
    Disabled,
    Latest,
    Version(Version),
}

struct ModVersion {
    version: Version,
    // TODO: Use a HashSet for quick lookup?
    dependencies: Vec<ModDependency>,
}

struct ModDependency {
    name: String,
    version_req: VersionReq,
}

fn main() {
    let app = App::from_args();

    let set = ModsSet::new(&app.dir);
}
