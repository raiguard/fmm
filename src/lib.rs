use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

const HELP: &str = "\
fmm

USAGE:
    fmm [SUBCOMMAND] [OPTIONS]
";

#[derive(Debug, PartialEq)]
enum ModIdentifier {
    Latest(String),
    Versioned(String, String),
}

#[derive(Debug)]
struct AppArgs {
    enable: Option<Vec<ModIdentifier>>,
    enable_all: bool,
    disable: Option<Vec<ModIdentifier>>,
    disable_all: bool,
    mods_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModsCollection {
    mods: Vec<ModData>,
    #[serde(skip)]
    path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModData {
    name: String,
    enabled: bool,
    version: Option<String>,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    };

    let args = AppArgs {
        disable_all: pargs.opt_value_from_str("--disable-all")?.unwrap_or(false),
        disable: pargs.opt_value_from_fn("--disable", parse_mod_input)?,
        enable_all: pargs.opt_value_from_str("--enable-all")?.unwrap_or(false),
        enable: pargs.opt_value_from_fn("--enable", parse_mod_input)?,
        mods_path: pargs.value_from_str("--modspath")?,
    };

    Ok(args)
}

fn parse_mod_input(input: &str) -> Result<Vec<ModIdentifier>, String> {
    // TODO: Throw error on illegal characters detected
    // Legal characters are [a-zA-Z0-9_\- ]
    input
        .split('|')
        .map(|mod_identifier| {
            let parts: Vec<&str> = mod_identifier.split('@').collect();
            // TODO: qualify mod name somehow
            // TODO: Check for duplicates
            match parts[..] {
                [mod_name] => Ok(ModIdentifier::Latest(mod_name.to_string())),
                [mod_name, mod_version] => Ok(ModIdentifier::Versioned(
                    mod_name.to_string(),
                    mod_version.to_string(),
                )),
                _ => Err("Invalid mod identifier format".to_string()),
            }
        })
        .collect::<Result<Vec<ModIdentifier>, String>>()
}

fn parse_mod_list(directory: &str) -> Result<ModsCollection, Box<dyn Error>> {
    let mut path: PathBuf = PathBuf::new();
    path.push(directory);
    path.push("mod-list.json");

    let mut collection: ModsCollection = serde_json::from_str(&fs::read_to_string(&path)?)?;

    collection.path = path;

    Ok(collection)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    let collection = parse_mod_list(&args.mods_path)?;

    println!("{:#?}", collection);

    print!("{:#?}", args);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_latest() {
        assert_eq!(
            parse_mod_input("RecipeBook"),
            Ok(vec![ModIdentifier::Latest("RecipeBook".to_string())])
        )
    }

    #[test]
    fn invalid_format() {
        assert_eq!(
            parse_mod_input("RecipeBook@1.2.3|Foo@bar@set"),
            Err("Invalid mod identifier format".to_string())
        )
    }

    #[test]
    fn simple_mod_list() {}
}
