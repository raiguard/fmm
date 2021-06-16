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

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    let mut set = ModsSet::new(&app.dir)?;

    for mod_ident in app.disable.iter() {
        set.disable(mod_ident)?;
    }

    for mod_ident in app.enable.iter() {
        set.enable(mod_ident)?;
    }

    Ok(())
}
