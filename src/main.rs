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
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    /// The path to the mods directory
    #[structopt(short = "f", long)]
    dir: Option<PathBuf>,
    /// A list of mods to enable. TODO: explain format.
    #[structopt(short, long)]
    enable: Vec<InputMod>,
}

#[derive(Debug)]
struct InputMod {
    name: String,
    version: ModVersion,
}

impl std::str::FromStr for InputMod {
    type Err = InputModErr;

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
                    // Validate that the version does *not* have prerelease or build data
                    if version.pre.len() > 0 || version.build.len() > 0 {
                        Err(Self::Err::InvalidVersion(version.to_string()))
                    } else {
                        Ok(Self {
                            name: name.to_string(),
                            version: ModVersion::Specific(version),
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
enum ModVersion {
    Latest,
    Specific(Version),
}

fn main() {
    let app = App::from_args();

    println!("{:#?}\n{:#?}\n{:#?}", app.enable, app.disable, app.dir)
}
