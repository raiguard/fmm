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

    let directory = ModsDirectory::new(args.mods_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::ModData;

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

        let mod_data = ModData {
            name: "aai-industry".to_string(),
            version: None,
        };

        // assert!(dir.mods.binary_search(&mod_data).is_ok());
    }
}
