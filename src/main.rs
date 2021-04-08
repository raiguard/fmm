const HELP: &str = "\
fmm

USAGE:
    fmm [SUBCOMMAND] [OPTIONS]
";

fn main() {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    };

    if let Err(e) = factorio_mod_manager::run(pargs) {
        eprintln!("Application error: {}", e);
    }
}
