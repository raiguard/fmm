use std::error::Error;

const HELP: &str = "\
fmm

USAGE:
    fmm [SUBCOMMAND] [OPTIONS]
";

#[derive(Debug)]
enum Command {
    DisableAll,
    Disable(Vec<String>),
    EnableAll,
    Enable(Vec<String>),
    Help,
    // Download,
    // Upload,
    // Update,
}

impl Command {
    fn new(input: Option<String>) -> Command {
        match input {
            Some(raw) => match raw.as_str() {
                "disable-all" => Command::DisableAll,
                "disable" => Command::Disable(vec!["RecipeBook".to_string()]),
                "enable-all" => Command::EnableAll,
                "enable" => Command::Enable(vec!["RecipeBook".to_string()]),
                "help" => Command::Help,
                _ => Command::Help,
            },
            None => Command::Help,
        }
    }
}

#[derive(Debug)]
struct AppArgs {
    command: Command,
    mods_path: String,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    };

    let args = AppArgs {
        command: Command::new(pargs.subcommand()?),
        mods_path: pargs.value_from_str("--modspath")?,
    };

    Ok(args)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    print!("{:#?}", args);

    Ok(())
}

#[cfg(test)]
mod tests {}
