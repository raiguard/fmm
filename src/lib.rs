use std::error::Error;

const HELP: &str = "\
fmm

USAGE:
    fmm [SUBCOMMAND] [OPTIONS]
";

#[derive(Debug)]
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

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    };

    let args = AppArgs {
        disable_all: pargs.opt_value_from_str("--disable-all")?.unwrap_or(false),
        disable: pargs.opt_value_from_fn("--disable", parse_mod_list)?,
        enable_all: pargs.opt_value_from_str("--enable-all")?.unwrap_or(false),
        enable: pargs.opt_value_from_fn("--enable", parse_mod_list)?,
        mods_path: pargs.value_from_str("--modspath")?,
    };

    Ok(args)
}

fn parse_mod_list(input: &str) -> Result<Vec<ModIdentifier>, String> {
    let results: Result<Vec<ModIdentifier>, String> = input
        .split('|')
        .map(|mod_identifier| {
            let parts: Vec<&str> = mod_identifier.split('@').collect();
            match parts[..] {
                [mod_name] => Ok(ModIdentifier::Latest(mod_name.to_string())),
                [mod_name, mod_version] => Ok(ModIdentifier::Versioned(
                    mod_name.to_string(),
                    mod_version.to_string(),
                )),
                _ => Err("Invalid mod identifier format".to_string()),
            }
        })
        .collect();

    results
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    print!("{:#?}", args);

    Ok(())
}

#[cfg(test)]
mod tests {}
