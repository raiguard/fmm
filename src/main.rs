mod dependency;
mod input;
mod mods_set;

use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::input::InputMod;
use crate::mods_set::ModsSet;

// TODO: Figure out why it's not coloring the help info.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "fmm",
    about = "Enable, disable, download, update, create, and delete Factorio mods."
)]
struct App {
    /// The path to the mods directory
    // TODO: Make optional, introduce config file to specify default path
    #[structopt(short = "f", long)]
    dir: PathBuf,
    /// Disable all mods.
    #[structopt(short = "o", long)]
    disable_all: bool,
    /// A list of mods to disable. TODO: explain format.
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    /// Enable the latest versions of all mods.
    #[structopt(short = "l", long)]
    enable_all: bool,
    /// A list of mods to enable. TODO: explain format.
    #[structopt(short, long)]
    enable: Vec<InputMod>,
    /// Include the base mod when calling `disable-all`.
    #[structopt(long)]
    include_base_mod: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    let mut set = ModsSet::new(&app.dir)?;

    if app.disable_all {
        set.disable_all(app.include_base_mod);
    }

    if app.enable_all {
        set.enable_all();
    }

    for mod_ident in app.disable.iter() {
        set.disable(mod_ident)?;
    }

    for mod_ident in app.enable.iter() {
        set.enable(mod_ident)?;
    }

    set.write_mod_list();
    set.write_mod_list()?;

    Ok(())
}
