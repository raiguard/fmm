fn main() {
    if let Err(e) = factorio_mod_manager::run() {
        eprintln!("Application error: {}", e);
    }
}
