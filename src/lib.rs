use pico_args::Arguments;
use std::error::Error;

enum Subcommand {
    Help,
}

impl Subcommand {
    fn new(input: Option<String>) -> Subcommand {
        match input {
            Some(raw) => match raw.as_str() {
                "help" => Subcommand::Help,
                _ => Subcommand::Help,
            },
            None => Subcommand::Help,
        }
    }
}

pub fn run(mut args: Arguments) -> Result<(), Box<dyn Error>> {
    let subcommand = Subcommand::new(args.subcommand()?);

    match subcommand {
        Subcommand::Help => println!("Display help!"),
    }

    Ok(())
}
