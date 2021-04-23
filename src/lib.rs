mod dependency;
mod directory;
mod input;

use directory::ModsDirectory;
use input::ModsInputList;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
struct AppArgs {
    dedup: bool,
    disable_all: bool,
    disable_base: bool,
    disable: Option<ModsInputList>,
    enable_all: bool,
    enable: Option<ModsInputList>,
    mods_path: PathBuf,
    // FOR THE FUTURE:
    // - Auto depencency activation
    // - Deduplicate mod versions
    // - Download from the portal?
    // - Upload to the portal?
    // - Changelog bump
    // - Mod packaging
}

impl AppArgs {
    fn new(mut pargs: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
        Ok(AppArgs {
            dedup: pargs.contains("--dedup"),
            disable_all: pargs.contains("--disable-all"),
            disable_base: pargs.contains("--disable-base"),
            disable: pargs.opt_value_from_fn("--disable", |value| ModsInputList::new(value))?,
            enable_all: pargs.contains("--enable-all"),
            enable: pargs.opt_value_from_fn("--enable", |value| ModsInputList::new(value))?,
            mods_path: pargs.value_from_str("--modspath")?, // TODO: environment var and config file
        })
    }
}

pub fn run(pargs: pico_args::Arguments) -> Result<(), Box<dyn Error>> {
    let args = AppArgs::new(pargs)?;

    let mut directory = ModsDirectory::new(args.mods_path)?;

    if args.disable_all {
        directory.disable_all();
    }

    if args.enable_all {
        directory.enable_all();
    }

    if let Some(mods) = args.disable {
        for mod_data in mods.iter() {
            directory.toggle_mod(mod_data, false)?;
        }
    }

    if let Some(mods) = args.enable {
        for mod_data in mods.iter() {
            directory.toggle_mod(mod_data, true)?;
        }
    }

    if args.dedup {
        directory.dedup()?;
    }

    directory.write()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::ModInputData;

    // TODO: Populate some mock mod directories
    fn tests_path(suffix: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push(suffix);
        println!("{:?}", d);
        d
    }

    #[test]
    fn simple_mods_dir() {
        let dir = ModsDirectory::new(tests_path("mods_dir_1")).unwrap();

        // assert!(dir.mods.binary_search(&mod_data).is_ok());
    }
}
