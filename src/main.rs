use pico_args::Arguments;

fn main() {
    let args = Arguments::from_env();

    if let Err(e) = factorio_mod_manager::run(args) {
        eprintln!("Application error: {}", e);
    }
}
