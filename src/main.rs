use semver::Version;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

// TODO: Figure out why it's not coloring the help info.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "fmm",
    about = "Enable, disable, download, update, create, and delete Factorio mods."
)]
struct App {
    /// A list of mods to disable. TODO: explain format.
    // TODO: This should be Vec<ModIdent> instead
    #[structopt(short, long)]
    disable: Vec<ModIdent>,
    /// A list of mods to enable. TODO: explain format.
    // TODO: This should be Vec<ModIdent> instead
    #[structopt(short, long)]
    enable: Vec<ModIdent>,
    // /// The path to the mods directory
    // #[structopt(short = "-dir", long)]
    // dir: PathBuf,
}

#[derive(Debug)]
struct ModIdent {
    name: String,
    version: ModVersion,
}

impl std::str::FromStr for ModIdent {
    type Err = ModIdentErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        match parts[..] {
            [name] => Ok(Self {
                name: name.to_string(),
                version: ModVersion::Latest,
            }),
            [name, version] => {
                let parsed_version = Version::parse(version);
                if let Ok(version) = parsed_version {
                    Ok(Self {
                        name: name.to_string(),
                        version: ModVersion::Ver(version),
                    })
                } else {
                    Err(Self::Err::InvalidVersion(version.to_string()))
                }
            }
            _ => Err(Self::Err::IncorrectArgCount(parts.len())),
        }
    }
}

#[derive(Debug)]
enum ModIdentErr {
    IncorrectArgCount(usize),
    InvalidVersion(String),
    // NonexistentMod(String),
}

impl fmt::Display for ModIdentErr {
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
enum ModVersion {
    Latest,
    Ver(Version),
}

fn main() {
    #[allow(unused)]
    let app = App::from_args();
}
