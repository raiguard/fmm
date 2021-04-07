use std::error::Error;

const HELP: &str = "\
fmm

USAGE:
    fmm [SUBCOMMAND] [OPTIONS]
";

enum ModIdentifier {
    Latest(String),
    Versioned(String, String),
}

#[derive(Debug)]
struct AppArgs {
    enable: Option<Vec<String>>,
    enable_all: bool,
    disable: Option<Vec<String>>,
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

fn parse_mod_list(input: &str) -> Result<Vec<String>, String> {
    // TODO: Actually handle errors
    // TODO: Handle mod version (`ModIdentifier` enum)
    Ok(input
        .split('|')
        .map(|mod_identifier| mod_identifier.to_string())
        .collect())
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    print!("{:#?}", args);

    Ok(())
}

#[cfg(test)]
mod tests {}
