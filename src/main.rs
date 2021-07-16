mod dependency;
mod input;
mod mods_set;

use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::input::{ConfigFile, InputMod};
use crate::mods_set::ModsSet;

// TODO: Figure out why it's not coloring the help info.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "fmm",
    about = "Enable, disable, download, update, create, and delete Factorio mods."
)]
struct App {
    config: Option<PathBuf>,
    /// Deduplicate zipped mod versions, leaving only the latest version
    #[structopt(long)]
    dedup: bool,
    /// The path to the mods directory
    // TODO: Make optional, introduce config file to specify default path
    #[structopt(short = "f", long)]
    dir: Option<PathBuf>,
    /// Disable all mods.
    #[structopt(short = "o", long)]
    disable_all: bool,
    /// A list of mods to disable. Format is `mod_name` or `mod_name@version`.
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    /// Enable the latest versions of all mods.
    #[structopt(short = "l", long)]
    enable_all: bool,
    /// A list of mods to enable. Format is `mod_name` or `mod_name@version`.
    #[structopt(short, long)]
    enable: Vec<InputMod>,
    /// Include the base mod when calling `disable-all`.
    #[structopt(long)]
    include_base_mod: bool,
    /// A list of mods to remove. If no version is provided, the latest version will be removed.
    #[structopt(short, long)]
    remove: Vec<InputMod>,
}

impl App {
    fn merge_config(&mut self, config_file: ConfigFile) {
        if let Some(directory) = config_file.directory {
            self.dir = Some(directory);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::from_args();

    // Find or create config file
    let config_file = ConfigFile::new(&app.config)?;
    if let Some(config_file) = config_file {
        app.merge_config(config_file);
    }

    // FIXME: If they don't provide a path in the toml or in the arguments, this will panic
    let mut set = ModsSet::new(&app.dir.unwrap())?;

    for mod_ident in app.remove.iter() {
        set.remove(mod_ident)?;
    }

    if app.dedup {
        set.dedup()?;
    }

    if app.disable_all {
        set.disable_all(app.include_base_mod);
    }

    if app.enable_all {
        set.enable_all();
    }

    for mod_ident in app.disable.iter() {
        set.disable(mod_ident)?;
    }

    set.enable_list(app.enable)?;

    set.write_mod_list()?;

    Ok(())
}
